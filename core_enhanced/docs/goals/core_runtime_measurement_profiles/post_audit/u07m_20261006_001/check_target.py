import json,hashlib,pathlib,datetime,sys,tarfile,subprocess,tempfile
R=pathlib.Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_runtime_measurement_profiles';S=H/'execution/single_goal';T=S/'target/u07m_001';A=pathlib.Path(__file__).parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda p:json.loads(p.read_text())
m=load(T/'MANIFEST.json');err=[]
def verify(rows,root=R):
 for x in rows:
  p=root/x['path']
  if not p.is_file() or p.stat().st_size!=x['bytes'] or sha(p)!=x['sha256']:err.append(str(p))
def archive(p,rows):
 expected={x['path']:x for x in rows};seen=set()
 with tarfile.open(p) as t:
  for x in t:
   if not x.isfile() or x.name not in expected or x.name in seen:err.append('archive membership '+str(p)+' '+x.name);continue
   row=expected[x.name];h=hashlib.sha256()
   with t.extractfile(x) as f:
    while c:=f.read(1024*1024):h.update(c)
   if x.size!=row['bytes'] or h.hexdigest()!=row['sha256']:err.append('archive bytes '+str(p)+' '+x.name)
   seen.add(x.name)
 if seen!=set(expected):err.append('archive missing '+str(p))
 return len(seen)
verify(load(T/'MEMBERSHIP.json'));verify(m['archives_and_patches'],T)
if sha(T/'MANIFEST.json')!='50c3c228d2c8f0915ab6133014bcf7783bb44d3b8880a8ba7c784ffe130a5733':err.append('manifest')
paths=set()
for f in ('core_enhanced/src','core_enhanced/tests','game_server/src','game_server/tests','game_server/config','game_server/examples','config','game_resources/data','game_resources/live_data'):
 paths.update(str(p.relative_to(R)) for p in (R/f).rglob('*') if p.is_file())
paths.update(x['path'] for x in m['product_source']['files'] if x['path'] not in paths)
if paths!={x['path'] for x in m['product_source']['files']}:err.append('product new membership')
records={str(p.relative_to(R)) for p in H.rglob('*') if p.is_file() and not {'target','post_audit','__pycache__'}&set(p.relative_to(H).parts)}
if records!={x['path'] for x in load(T/'RECORDS.json')}:err.append('record membership')
if subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip()!=m['vcs_revision']:err.append('HEAD')
out={'captured_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'target':str(T.relative_to(R)),'manifest_sha256':sha(T/'MANIFEST.json'),'membership_count':len(load(T/'MEMBERSHIP.json')),'source_count':len(m['source_files']),'product_count':len(m['product_source']['files']),'records_count':len(records),'errors':err}
if sys.argv[1]=='entry':
 out['archives']={}
 for name,rows in [('SOURCE_BEFORE.tar.gz',m['before_files']),('SOURCE_AFTER.tar.gz',m['source_files']),('RECORDS.tar.gz',load(T/'RECORDS.json')),('REFERENCES.tar.gz',load(T/'REFERENCES.json'))]:out['archives'][name]=archive(T/name,rows)
 for x in load(T/'EXTERNAL_ARCHIVES.json'):
  p=R/x['path'];pt=p.parent;pm=load(pt/'MANIFEST.json');rows=pm['before_files'] if p.name=='SOURCE_BEFORE.tar.gz' else pm['source_files'] if p.name=='SOURCE_AFTER.tar.gz' else load(pt/(p.name.split('.')[0]+'.json'));out['archives'][x['path']]=archive(p,rows)
 with tempfile.TemporaryDirectory(dir=A,prefix='restore_') as temp:
  root=pathlib.Path(temp)
  with tarfile.open(T/'SOURCE_BEFORE.tar.gz') as t:t.extractall(root,filter='data')
  out['restoration']=[]
  for reverse,rows in [(False,m['source_files']),(True,m['before_files'])]:
   result=subprocess.run(['patch',*(['-R'] if reverse else []),'-p1','--fuzz=0','--batch','--input',str(T/'FORWARD.patch')],cwd=root,capture_output=True)
   (A/('RESTORE_REVERSE.log' if reverse else 'RESTORE_FORWARD.log')).write_bytes(result.stdout+result.stderr)
   verify(rows,root)
   names={str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()}
   if result.returncode or names!={x['path'] for x in rows}:err.append('restore '+str(reverse))
   out['restoration'].append({'reverse':reverse,'exit':result.returncode,'members':len(names)})
status=subprocess.check_output(['git','status','--porcelain=v1','-uall'],cwd=R)
(A/('GIT_STATUS_'+sys.argv[1]+'.txt')).write_bytes(status)
out['status_sha256']=hashlib.sha256(status).hexdigest();out['currentness']='current / unchanged' if not err else 'stale'
(A/('CURRENTNESS_'+sys.argv[1]+'.json')).write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))
