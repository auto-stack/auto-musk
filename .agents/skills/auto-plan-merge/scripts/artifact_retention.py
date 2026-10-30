"""Maintain registered evidence scopes; dry-run by default. Requires Python 3.12+."""
import argparse, collections, ctypes, datetime as dt, hashlib, json, os, shutil, stat, subprocess
from pathlib import Path


def scan(root):
    out, links = [], []
    def walk(folder):
        with os.scandir(folder) as entries:
            for e in entries:
                s=Path(e.path).stat(follow_symlinks=False)
                if getattr(s,'st_file_attributes',0)&0x400 or e.is_symlink(): links.append(e.path)
                elif e.is_dir(follow_symlinks=False): walk(e.path)
                else: out.append((Path(e.path),s))
    walk(root)
    return out,links


def sha(p):
    h=hashlib.sha256()
    with p.open('rb') as f:
        for b in iter(lambda:f.read(4*1024*1024),b''): h.update(b)
    return h.hexdigest()


def allocated(p):
    if os.name!='nt': return p.stat().st_blocks*512
    hi=ctypes.c_ulong();fn=ctypes.windll.kernel32.GetCompressedFileSizeW
    fn.argtypes=[ctypes.c_wchar_p,ctypes.POINTER(ctypes.c_ulong)];fn.restype=ctypes.c_ulong
    ctypes.windll.kernel32.SetLastError(0);lo=fn(str(p),ctypes.byref(hi))
    if lo==0xffffffff and ctypes.windll.kernel32.GetLastError(): raise OSError(str(p))
    return (hi.value<<32)|lo


def sig(s): return s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns


def _run(root,apply=False,dedup=False):
    root=Path(os.path.abspath(root))
    if root.is_symlink() or os.path.isjunction(root): raise ValueError('linked root')
    cfg=json.loads((root/'.retention/policy.json').read_text(encoding='utf-8-sig'))
    if os.path.normcase(os.path.abspath(cfg['root']))!=os.path.normcase(str(root)): raise ValueError('root mismatch')
    scopes=cfg['scopes'];busy=set();now=dt.datetime.now(dt.timezone.utc)
    for x in scopes:
        if not x or Path(x).is_absolute() or '..' in Path(x).parts: raise ValueError('unsafe scope')
    if os.name=='nt':
        proc=subprocess.run(['powershell.exe','-NoProfile','-NonInteractive','-Command',
            'Get-CimInstance Win32_Process | Select-Object ExecutablePath,CommandLine | ConvertTo-Json -Compress'],
            capture_output=True,text=True,check=True,creationflags=subprocess.CREATE_NO_WINDOW)
        for row in json.loads(proc.stdout):
            for x in scopes:
                prefix=str(root/x).lower()+'\\'
                if any(prefix in (row.get(k) or '').replace('/','\\').lower() for k in ['ExecutablePath','CommandLine']): busy.add(x)
    files,links=scan(root)
    def owner(p):
        r=p.relative_to(root).as_posix();found=[x for x in scopes if r==x or r.startswith(x+'/')]
        return max(found,key=len) if found else None
    def checked(p,s):
        if os.path.commonpath([str(root),os.path.abspath(p)])!=str(root): raise ValueError('outside root')
        for a in [p,*p.parents]:
            if a==root.parent: break
            if a.is_symlink() or os.path.isjunction(a): raise ValueError('reparse point')
        if sig(p.stat())!=sig(s): raise ValueError('changed file')
    receipt=dict(started_at=now.isoformat(),root=str(root),apply=apply,free_before=shutil.disk_usage(root).free,
        actions=[],errors=[],released_file_storage_bytes=0,released_logical_bytes=0,links_skipped=len(links),busy=sorted(busy))
    def allowed(p):
        s=owner(p)
        return s and s not in busy and scopes[s].get('status')=='archived' and not scopes[s].get('hold')
    inode_paths=collections.defaultdict(list)
    for path,info in files: inode_paths[(info.st_dev,info.st_ino)].append((path,info))
    safe_inodes={key for key,rows in inode_paths.items()
        if len(rows)>=max(info.st_nlink for path,info in rows) and all(allowed(path) for path,info in rows)}
    receipt['shared_inode_skipped']=0
    journal=None
    if apply:
        jp=root/'.retention'/('actions-'+now.strftime('%Y%m%dT%H%M%S%fZ')+'-'+str(os.getpid())+'.jsonl')
        journal=jp.open('x',encoding='utf8',buffering=1)
        receipt['action_journal']=str(jp)
    def event(action):
        if journal:
            try: journal.write(json.dumps(action)+'\n');journal.flush()
            except OSError as error: raise RuntimeError('Action journal failed; stop maintenance') from error
    def record(action):
        event(dict(action,state='done'));receipt['actions'].append(action)
    caches={p.parent for p,s in files if p.name=='CACHEDIR.TAG' and p.parent.name.lower()=='target'}
    exes=collections.defaultdict(list);restore_readonly={}
    for p,s in files:
        scope=owner(p)
        if not scope or scope in busy: continue
        rule=scopes[scope]
        if rule.get('status')!='archived' or rule.get('hold'): continue
        rel=p.relative_to(root).as_posix();reason=None
        if any(a.name=='node_modules' for a in p.parents): reason='node_modules'
        elif any(a in caches for a in p.parents): reason='cargo-target'
        if p.suffix.lower()=='.pdb': reason='pdb'
        if dt.datetime.fromisoformat(rule['expires_at'])<=now and rel in rule.get('expire_files',{}):
            expected=rule['expire_files'][rel]
            if not isinstance(expected,dict) or expected.get('size')!=s.st_size or expected.get('mtime_ns')!=s.st_mtime_ns:
                receipt['errors'].append([str(p),'expired file metadata changed or missing; preserved'])
                continue
            if not expected.get('sha256') or sha(p)!=expected['sha256']:
                receipt['errors'].append([str(p),'expired content hash changed or missing; preserved'])
                continue
            reason='expired'
        if reason and not s.st_mode&stat.S_IWRITE and s.st_nlink>1 and (s.st_dev,s.st_ino) not in safe_inodes:
            receipt['shared_inode_skipped']+=1
            continue
        if reason and (p.suffix.lower() not in ['.exe','.dll'] or reason=='expired'):
            try:
                checked(p,s);last=p.stat().st_nlink==1;disk=allocated(p) if last else 0
                if apply:
                    event(dict(state='intent',kind=reason,path=str(p),bytes=s.st_size))
                    if not s.st_mode&stat.S_IWRITE:
                        if s.st_nlink>1: restore_readonly[(s.st_dev,s.st_ino)]=(s.st_size,s.st_mtime_ns,sha(p))
                        os.chmod(p,s.st_mode|stat.S_IWRITE)
                    p.unlink();receipt['released_file_storage_bytes']+=disk
                    receipt['released_logical_bytes']+=s.st_size if last else 0
                record(dict(kind=reason,path=str(p),bytes=s.st_size))
            except (OSError,ValueError) as e: receipt['errors'].append([str(p),str(e)])
        elif p.suffix.lower()=='.exe':
            if (s.st_dev,s.st_ino) in safe_inodes: exes[s.st_size].append((p,s))
            else: receipt['shared_inode_skipped']+=1
    if dedup:
        hashed={};keepers={};groups=0
        for size,rows in exes.items():
            if len(rows)<2: continue
            for p,s in rows:
                try:
                    checked(p,s);key=(s.st_dev,s.st_ino)
                    if key not in hashed: hashed[key]=sha(p)
                    digest=hashed[key];identity=(size,digest)
                    if identity not in keepers: keepers[identity]=(p,s);continue
                    keeper,ks=keepers[identity]
                    if os.path.samefile(p,keeper): continue
                    checked(p,s);checked(keeper,ks)
                    # Full hashes of both files bind identity; timestamps are rechecked before replacement.
                    if sig(p.stat())!=sig(s) or sig(keeper.stat())!=sig(ks): raise ValueError('concurrent writer')
                    last=p.stat().st_nlink==1;disk=allocated(p) if last else 0
                    if apply:
                        event(dict(state='intent',kind='dedup',path=str(p),canonical=str(keeper),sha256=digest,bytes=size))
                        temp=p.with_name(p.name+'.retention-link-tmp')
                        if temp.exists(): raise ValueError('temporary path exists')
                        os.link(keeper,temp)
                        try:
                            if not s.st_mode&stat.S_IWRITE and p.stat().st_nlink>1:
                                restore_readonly[(s.st_dev,s.st_ino)]=(s.st_size,s.st_mtime_ns,digest)
                            os.chmod(p,s.st_mode|stat.S_IWRITE);os.replace(temp,p)
                        finally:
                            if temp.exists(): temp.unlink()
                        os.chmod(keeper,stat.S_IREAD)
                        if not os.path.samefile(p,keeper) or sha(p)!=digest: raise ValueError('verification failed')
                        receipt['released_file_storage_bytes']+=disk
                        receipt['released_logical_bytes']+=size if last else 0
                    record(dict(kind='dedup',path=str(p),canonical=str(keeper),sha256=digest,bytes=size))
                except (OSError,ValueError) as e: receipt['errors'].append([str(p),str(e)])
            groups+=1
            if groups%25==0: print('hashed size groups',groups,flush=True)
    after,_=scan(root);unique={};totals=collections.defaultdict(dict);restored=set()
    for p,s in after:
        key=(s.st_dev,s.st_ino);unique[key]=s.st_size;scope=owner(p)
        if apply and key in restore_readonly and key not in restored:
            size,mtime,digest=restore_readonly[key]
            if s.st_size==size and s.st_mtime_ns==mtime and sha(p)==digest:
                checked(p,s);os.chmod(p,stat.S_IREAD);restored.add(key)
            else: receipt['errors'].append([str(p),'shared survivor changed; readonly restoration withheld'])
        if scope: totals[scope][key]=s.st_size
    receipt.update(finished_at=dt.datetime.now(dt.timezone.utc).isoformat(),free_after=shutil.disk_usage(root).free,
        remaining_unique_logical_bytes=sum(unique.values()),scope_unique_bytes={x:sum(v.values()) for x,v in totals.items()},
        over_budget_scopes=[x for x,v in totals.items() if sum(v.values())>cfg['per_plan_budget_bytes']],
        over_total_budget=sum(unique.values())>cfg['total_budget_bytes'])
    if apply:
        dest=root/'.retention/receipts';dest.mkdir(parents=True,exist_ok=True)
        out=dest/(now.strftime('%Y%m%dT%H%M%S%fZ')+'.json');out.write_text(json.dumps(receipt,indent=2),encoding='utf8')
        print('RECEIPT',out)
    if journal: journal.close()
    print(json.dumps({k:v for k,v in receipt.items() if k not in ['actions','scope_unique_bytes']}))
    return receipt



def run(root,apply=False,dedup=False):
    root=Path(os.path.abspath(root))
    for p in [root/'.retention',root,*root.parents]:
        if p.is_symlink() or os.path.isjunction(p): raise ValueError('linked maintenance path')
    lockpath=root/'.retention/maintenance.lock'
    if lockpath.is_symlink() or os.path.isjunction(lockpath) or (lockpath.exists() and lockpath.stat().st_nlink>1): raise ValueError('shared maintenance lock')
    with lockpath.open('a+b') as lock:
        if lock.seek(0,2)==0: lock.write(b'0');lock.flush()
        lock.seek(0)
        if os.name=='nt':
            import msvcrt
            msvcrt.locking(lock.fileno(),msvcrt.LK_NBLCK,1)
        else:
            import fcntl
            fcntl.flock(lock.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)
        try: return _run(root,apply,dedup)
        finally:
            lock.seek(0)
            if os.name=='nt': msvcrt.locking(lock.fileno(),msvcrt.LK_UNLCK,1)
            else: fcntl.flock(lock.fileno(),fcntl.LOCK_UN)

if __name__=='__main__':
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--root',default='D:/autostack/plan-artifacts')
    ap.add_argument('--apply',action='store_true');ap.add_argument('--deduplicate',action='store_true');a=ap.parse_args()
    r=run(a.root,a.apply,a.deduplicate)
    raise SystemExit(1 if r['errors'] else 2 if r['over_total_budget'] or r['over_budget_scopes'] else 0)
