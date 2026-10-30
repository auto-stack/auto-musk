"""Keep the latest five completed Plans; remove older completed, issue-free scopes.

The registry is reviewed metadata, not a substitute for checking Plan/debt records.
Deletion never follows reparse points or changes shared inode attributes.
"""
import argparse, collections, contextlib, ctypes, datetime as dt, hashlib, json, os, re, shutil, subprocess
from pathlib import Path


def digest(path):
    return hashlib.sha256(path.read_bytes().replace(b'\r\n',b'\n')).hexdigest()


def plain(path):
    return not path.is_symlink() and not os.path.isjunction(path)


def within(path, root):
    p=Path(os.path.abspath(path));r=Path(os.path.abspath(root))
    if p==r or not p.is_relative_to(r): raise ValueError('Not a strict child of root: '+str(p))
    return p


def source_valid(plan):
    try:
        path=Path(plan['document'])
        if not path.is_file() or not plain(path): return False
        text=path.read_text(encoding='utf-8-sig')
        match=re.search(r'^status:\s*([^\s#]+)',text,re.M)
        return bool(match and match[1]=='archived' and digest(path)==plan.get('document_sha256'))
    except (OSError,KeyError,UnicodeError): return False


def select(cfg):
    if cfg.get('schema')!='artifact-retention/v2' or cfg.get('keep_completed_plans')!=5:
        raise ValueError('Expected reviewed latest-five policy')
    plans=cfg['plans'];scopes=[];ranked=[];validated=set()
    for owner,p in plans.items():
        for scope in p['scopes']:
            x=Path(scope)
            if not scope or not x.parts or x.drive or x.is_absolute() or '..' in x.parts or x.parts[0].casefold()=='.retention': raise ValueError('Unsafe scope')
            scopes.append((owner,x))
        if p.get('status')=='archived':
            try:
                stamp=dt.datetime.fromisoformat(p['completed_at'])
                if stamp.tzinfo is None: raise ValueError('Naive completion timestamp')
                if source_valid(p): ranked.append((stamp,owner));validated.add(owner)
            except (KeyError,ValueError): pass
    for i,(a,x) in enumerate(scopes):
        for b,y in scopes[i+1:]:
            if x==y or x.is_relative_to(y) or y.is_relative_to(x): raise ValueError('Overlapping scopes')
    latest=[owner for stamp,owner in sorted(ranked,reverse=True)[:5]]
    removable=[];protected={}
    for owner,p in plans.items():
        reason=''
        if p.get('status')=='archived' and owner not in validated: reason='unverified completion source or timestamp'
        elif owner in latest: reason='latest-five'
        elif p.get('status')!='archived': reason='unfinished-or-unknown'
        elif p.get('issues_clear') is not True or p.get('hold'): reason=p.get('hold') or 'unresolved-or-unreviewed'
        elif not source_valid(p): reason='completion source changed or unavailable'
        if reason: protected[owner]=reason
        else: removable.append(owner)
    return latest,removable,protected


def process_text():
    if os.name!='nt': return []
    p=subprocess.run(['powershell.exe','-NoProfile','-NonInteractive','-Command',
        'Get-CimInstance Win32_Process | Select-Object ExecutablePath,CommandLine | ConvertTo-Json -Compress'],
        capture_output=True,text=True,check=True,creationflags=subprocess.CREATE_NO_WINDOW)
    rows=json.loads(p.stdout)
    if isinstance(rows,dict): rows=[rows]
    return [(r.get('ExecutablePath') or '').replace('/','\\').lower()+' '+
            (r.get('CommandLine') or '').replace('/','\\').lower() for r in rows]


def busy(root,plan,processes):
    for s in plan['scopes']:
        prefix=str(root/s).replace('/','\\').lower()
        if any(prefix+'\\' in line or prefix+'"' in line or prefix+' ' in line or line.endswith(prefix) for line in processes): return True
    return False


def registered_worktrees(root,cfg):
    trees=[]
    for repo in sorted({o.split('/')[0] for o in cfg['plans'] if re.fullmatch(r'[A-Za-z0-9_.-]+/PLAN-\d+',o)}):
        source=root.parent/repo
        if not (source/'.git').exists(): continue
        proc=subprocess.run(['git','-C',str(source),'worktree','list','--porcelain'],capture_output=True,text=True,check=True)
        trees.extend(Path(line[9:]) for line in proc.stdout.splitlines() if line.startswith('worktree '))
    return trees


def registered_scope(root,plan,trees):
    return any((root/s)==tree or tree.is_relative_to(root/s) or (root/s).is_relative_to(tree)
               for s in plan['scopes'] for tree in trees)


def erase_entry(path,root,boundary):
    """Remove exactly this directory entry; readonly shared aliases stay unchanged."""
    path=within(path,root);boundary=within(boundary,root)
    if path!=boundary and not path.is_relative_to(boundary): raise ValueError('Outside selected scope')
    if os.name!='nt':
        # POSIX unlink does not require changing readonly file attributes.
        if path.is_symlink() or not path.is_dir(): path.unlink()
        else: path.rmdir()
        return
    k=ctypes.WinDLL('kernel32',use_last_error=True)
    create=k.CreateFileW;create.argtypes=[ctypes.c_wchar_p,ctypes.c_ulong,ctypes.c_ulong,ctypes.c_void_p,ctypes.c_ulong,ctypes.c_ulong,ctypes.c_void_p];create.restype=ctypes.c_void_p
    close=k.CloseHandle;close.argtypes=[ctypes.c_void_p]
    final=k.GetFinalPathNameByHandleW;final.argtypes=[ctypes.c_void_p,ctypes.c_wchar_p,ctypes.c_ulong,ctypes.c_ulong];final.restype=ctypes.c_ulong
    disposition=k.SetFileInformationByHandle;disposition.argtypes=[ctypes.c_void_p,ctypes.c_int,ctypes.c_void_p,ctypes.c_ulong];disposition.restype=ctypes.c_int
    # DELETE + read attributes, share read/write/delete, open reparse point itself.
    h=create(str(path),0x10080,7,None,3,0x02200000,None)
    if h==ctypes.c_void_p(-1).value: raise ctypes.WinError(ctypes.get_last_error())
    try:
        buf=ctypes.create_unicode_buffer(32768);n=final(h,buf,len(buf),0)
        if not n or n>=len(buf): raise ctypes.WinError(ctypes.get_last_error())
        actual=buf.value
        if actual.startswith('\\\\?\\'): actual=actual[4:]
        if os.path.normcase(os.path.abspath(actual))!=os.path.normcase(str(path)):
            raise ValueError('Resolved handle escaped selected path: '+str(path))
        # FileDispositionInfoEx: DELETE | POSIX_SEMANTICS | IGNORE_READONLY_ATTRIBUTE.
        # This unlinks the selected alias without chmod on an inode shared elsewhere.
        flags=ctypes.c_ulong(0x13)
        if not disposition(h,21,ctypes.byref(flags),ctypes.sizeof(flags)): raise ctypes.WinError(ctypes.get_last_error())
    finally: close(h)


def remove_scope(path,root,event):
    path=within(path,root);counts=dict(files=0,directories=0,reparse_points=0,reclaimed_file_data_bytes=0)
    for parent in path.parents:
        if not plain(parent): raise ValueError('Linked ancestor: '+str(parent))
        if parent==root: break
    def walk(p):
        info=p.stat(follow_symlinks=False)
        linked=bool(getattr(info,'st_file_attributes',0)&0x400) or p.is_symlink()
        directory=bool(getattr(info,'st_file_attributes',0)&0x10) if os.name=='nt' else p.is_dir()
        if directory and not linked:
            with os.scandir(p) as it: children=[Path(e.path) for e in it]
            for child in children: walk(child)
        # Validate again through the opened handle at the moment of mutation.
        if not directory and not linked:
            current=p.stat(follow_symlinks=False)
            size=current.st_size if current.st_nlink==1 else 0
        else: size=0
        erase_entry(p,root,path)
        kind='reparse_points' if linked else 'directories' if directory else 'files'
        counts[kind]+=1;counts['reclaimed_file_data_bytes']+=size
        if counts['files'] and counts['files']%1000==0 and kind=='files': event(dict(state='progress',scope=str(path),**counts))
    if path.exists() or path.is_symlink() or os.path.isjunction(path): walk(path)
    return counts


def measure(root):
    unique={};files=0;links=0;transient=0
    def walk(p):
        nonlocal files,links,transient
        try:
            with os.scandir(p) as it:
                for e in it:
                    try:
                        s=Path(e.path).stat(follow_symlinks=False)
                        if getattr(s,'st_file_attributes',0)&0x400 or e.is_symlink(): links+=1
                        elif e.is_dir(follow_symlinks=False): walk(e.path)
                        else: unique[(s.st_dev,s.st_ino)]=s.st_size;files+=1
                    except FileNotFoundError: transient+=1
        except FileNotFoundError: transient+=1
    walk(root)
    return dict(unique_file_data_bytes=sum(unique.values()),files=files,reparse_points=links,transient_paths_skipped=transient)


@contextlib.contextmanager
def maintenance_lock(root):
    for p in [root/'.retention',root,*root.parents]:
        if not plain(p): raise ValueError('Linked maintenance path')
    p=root/'.retention/maintenance.lock'
    if not plain(p) or (p.exists() and p.stat().st_nlink>1): raise ValueError('Shared maintenance lock')
    with p.open('a+b') as f:
        if f.seek(0,2)==0: f.write(b'0');f.flush()
        f.seek(0)
        if os.name=='nt':
            import msvcrt
            msvcrt.locking(f.fileno(),msvcrt.LK_NBLCK,1)
        else:
            import fcntl
            fcntl.flock(f.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)
        try: yield
        finally:
            f.seek(0)
            if os.name=='nt': msvcrt.locking(f.fileno(),msvcrt.LK_UNLCK,1)
            else: fcntl.flock(f.fileno(),fcntl.LOCK_UN)


def run(root,apply=False):
    root=Path(os.path.abspath(root))
    with maintenance_lock(root):
        cfg=json.loads((root/'.retention/policy.json').read_text(encoding='utf-8-sig'))
        if os.path.normcase(os.path.abspath(cfg['root']))!=os.path.normcase(str(root)): raise ValueError('Root mismatch')
        latest,candidates,protected=select(cfg);procs=process_text();trees=registered_worktrees(root,cfg)
        for owner in candidates[:]:
            if busy(root,cfg['plans'][owner],procs) or registered_scope(root,cfg['plans'][owner],trees): candidates.remove(owner);protected[owner]='active-process-or-registered-worktree'
        receipt=dict(started_at=dt.datetime.now(dt.timezone.utc).isoformat(),apply=apply,latest_five=latest,
            protected=protected,candidates=candidates,deleted_scopes=[],errors=[],reclaimed_file_data_bytes=0,
            free_before=shutil.disk_usage(root).free)
        print(json.dumps(dict(latest_five=latest,candidates=candidates,protected=protected)),flush=True)
        if apply:
            stamp=dt.datetime.now(dt.timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
            journal_path=root/'.retention'/('last-five-'+stamp+'.jsonl')
            with journal_path.open('x',encoding='utf8',buffering=1) as journal:
                def event(data): journal.write(json.dumps(data)+'\n');journal.flush()
                for owner in candidates:
                    plan=cfg['plans'][owner]
                    if not source_valid(plan) or busy(root,plan,process_text()) or registered_scope(root,plan,registered_worktrees(root,cfg)):
                        receipt['protected'][owner]='source changed or active process at mutation';continue
                    for scope in plan['scopes']:
                        try:
                            event(dict(state='intent',owner=owner,scope=scope));os.fsync(journal.fileno())
                            counts=remove_scope(root/scope,root,event)
                            event(dict(state='done',owner=owner,scope=scope,**counts))
                            receipt['deleted_scopes'].append(dict(owner=owner,scope=scope,**counts))
                            receipt['reclaimed_file_data_bytes']+=counts['reclaimed_file_data_bytes']
                            print(json.dumps(dict(deleted=scope,**counts)),flush=True)
                        except (OSError,ValueError) as error:
                            receipt['errors'].append(dict(scope=scope,error=str(error)));event(dict(state='error',scope=scope,error=str(error)))
            receipt['journal']=str(journal_path)
            receipt['remaining']=measure(root);receipt['over_total_budget_alert']=receipt['remaining']['unique_file_data_bytes']>cfg.get('budget_alerts',{}).get('total_GiB',10)*1024**3;receipt['free_after']=shutil.disk_usage(root).free
            receipt['finished_at']=dt.datetime.now(dt.timezone.utc).isoformat()
            out=root/'.retention/receipts'/('last-five-'+stamp+'.json');out.parent.mkdir(exist_ok=True)
            out.write_text(json.dumps(receipt,indent=2,ensure_ascii=False),encoding='utf8');receipt['receipt']=str(out)
        print(json.dumps(receipt,ensure_ascii=False),flush=True)
        return receipt


if __name__=='__main__':
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--root',default='D:/autostack/plan-artifacts');ap.add_argument('--apply',action='store_true')
    args=ap.parse_args();result=run(args.root,args.apply);raise SystemExit(1 if result['errors'] else 0)
