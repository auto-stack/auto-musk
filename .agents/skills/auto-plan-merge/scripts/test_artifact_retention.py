import datetime as dt, importlib.util, json, os, stat, tempfile, subprocess
from pathlib import Path
spec=importlib.util.spec_from_file_location('retention',Path(__file__).with_name('artifact_retention.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
with tempfile.TemporaryDirectory() as temp:
 base=Path(temp);root=base/'artifacts';(root/'.retention').mkdir(parents=True);docs=base/'docs';docs.mkdir();plans={}
 def put(rel,data=b'evidence'):
  p=root/rel;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(data);return p
 for n in range(7):
  p=docs/f'{n}.md';p.write_text('---\nstatus: archived\n---\n')
  owner=f'repo/PLAN-{n:03d}';scope=f'plan-{n:03d}';put(scope+'/data.bin')
  plans[owner]={'status':'archived','issues_clear':True,'completed_at':f'2026-10-{10+n:02d}T10:00:00+08:00','document':str(p),'document_sha256':m.digest(p),'scopes':[scope]}
 # Two oldest should be removed, newest five retained. Copies count as one Plan.
 plans['repo/PLAN-000']['scopes'].append('plan-000-other');put('plan-000-other/data')
 survivor=put('plan-006/survivor.exe',b'immutable replay');old=root/'plan-000/shared.exe';os.link(survivor,old);survivor.chmod(stat.S_IREAD);mode=survivor.stat().st_mode
 plans['repo/PLAN-active']={'status':'executing','issues_clear':True,'scopes':['active']};put('active/data')
 held_doc=docs/'held.md';held_doc.write_text('status: archived\n');plans['repo/PLAN-held']={'status':'archived','issues_clear':False,'hold':'open bug','completed_at':'2026-10-01T00:00:00+08:00','document':str(held_doc),'document_sha256':m.digest(held_doc),'scopes':['held']};held=put('held/data')
 stale_doc=docs/'stale.md';stale_doc.write_text('status: archived\n');plans['repo/PLAN-stale']={'status':'archived','issues_clear':True,'completed_at':'2026-10-02T00:00:00+08:00','document':str(stale_doc),'document_sha256':m.digest(stale_doc),'scopes':['stale']};put('stale/data');stale_doc.write_text('status: executing\n')
 cfg={'schema':'artifact-retention/v2','root':str(root),'keep_completed_plans':5,'plans':plans};(root/'.retention/policy.json').write_text(json.dumps(cfg))
 # An old directory's fresh mtime must not make it recent.
 os.utime(root/'plan-000',None)
 preview=m.run(root);assert len(preview['latest_five'])==5 and set(preview['candidates'])=={'repo/PLAN-000','repo/PLAN-001'};assert old.exists()
 # Real junction to outside root must be unlinked, never traversed.
 outside=base/'outside';outside.mkdir();sentinel=outside/'important.txt';sentinel.write_text('keep outside');link=root/'plan-000/escape'
 if os.name=='nt':
  command=['powershell.exe','-NoProfile','-NonInteractive','-Command',"New-Item -ItemType Junction -Path $args[0] -Target $args[1] | Out-Null",str(link),str(outside)]
  # Use symlink when available; Windows junction fallback with literal known temp paths.
  try:os.symlink(outside,link,target_is_directory=True)
  except OSError:
   subprocess.run(['powershell.exe','-NoProfile','-NonInteractive','-Command',f"New-Item -ItemType Junction -Path '{link}' -Target '{outside}' | Out-Null"],check=True,capture_output=True)
 else:os.symlink(outside,link,target_is_directory=True)
 result=m.run(root,True);assert not result['errors'],result['errors'];assert not (root/'plan-000').exists() and not (root/'plan-001').exists();assert not (root/'plan-000-other').exists()
 assert sentinel.read_text()=='keep outside';assert survivor.read_bytes()==b'immutable replay' and survivor.stat().st_mode==mode;assert held.exists() and (root/'active/data').exists() and (root/'stale/data').exists()
 second=m.run(root,True);assert not second['errors'] and second['reclaimed_file_data_bytes']==0
 # Invalid newest completion cannot evict the fifth actually verified completion.
 newest=Path(plans['repo/PLAN-006']['document']);newest.write_text('status: executing\n')
 selected,removable,protected=m.select(cfg);assert 'repo/PLAN-001' in selected and 'repo/PLAN-006' in protected
 newest.write_text('---\nstatus: archived\n---\n')
 saved=plans['repo/PLAN-000'].pop('completed_at');selected,removable,protected=m.select(cfg)
 assert 'repo/PLAN-000' not in removable and 'repo/PLAN-000' in protected
 plans['repo/PLAN-000']['completed_at']=saved
 # Scope escape and overlapping protected parent must fail before deletion.
 cfg['plans']['bad']={'status':'archived','scopes':['../outside']};(root/'.retention/policy.json').write_text(json.dumps(cfg))
 try:m.run(root,True)
 except ValueError:pass
 else:raise AssertionError('scope escape accepted')
 survivor.chmod(stat.S_IWRITE|stat.S_IREAD)
 print('PASS: latest five globally; alias grouping; archive time not directory mtime; dry-run; old complete deletion; active/held/stale protection; readonly external shared inode; real reparse no-follow; repeat; escape')
