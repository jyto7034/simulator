import json,hashlib,tarfile,tempfile,subprocess,datetime
from pathlib import Path
T=Path('core_enhanced/docs/goals/core_relic_numeric_variants/execution/single_goal/target/u08_current_013').resolve();A=T.parents[2]/'post_audit/u08_current_013_pass002';A=Path('core_enhanced/docs/goals/core_relic_numeric_variants/post_audit/u08_current_013_pass002')
m=json.loads((T/'MANIFEST.json').read_text());bad=[]
def check(root,rows,tag):
 wanted={r['path']:r['sha256'] for r in rows};actual={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in root.rglob('*') if p.is_file()};
 if actual!=wanted:bad.append(tag)
 return len(actual)
with tempfile.TemporaryDirectory(prefix='u08-independent-restore-') as temp:
 root=Path(temp)
 with tarfile.open(T/'SOURCE_BEFORE.tar.gz') as tf:tf.extractall(root,filter='data')
 counts=[check(root,m['before_files'],'before')]
 commands=[]
 for patch,rows in [('FORWARD.patch',m['source_files']),('REVERSE.patch',m['before_files'])]:
  p=subprocess.run(['git','apply','--binary','-p2','--whitespace=nowarn',str(T/patch)],cwd=root,text=True,capture_output=True);commands.append({'patch':patch,'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr});
  if p.returncode:bad.append(patch)
  counts.append(check(root,rows,patch+' resulting_bytes_membership'))
external=[]
for e in json.loads((T/'EXTERNAL_ARCHIVES.json').read_text()):
 path=Path(e['archive']['path']);data=json.loads(Path(e['members_manifest']['path']).read_text());
 for key in filter(None,e['members_pointer'].strip('/').split('/')):data=data[int(key)] if isinstance(data,list) else data[key]
 wanted={r['path']:r for r in data};seen={}
 with tarfile.open(path) as tf:
  for member in tf:
   if member.isfile():seen[member.name]={'bytes':member.size,'sha256':hashlib.sha256(tf.extractfile(member).read()).hexdigest()}
 valid=set(wanted)==set(seen) and all(all(seen[p][k]==r[k] for k in ['bytes','sha256']) for p,r in wanted.items())
 if not valid:bad.append(str(path))
 external.append({'path':str(path),'members':len(seen),'exact':valid})
out={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'commands':commands,'counts':counts,'isolated':True,'workspace_reverse_applied':False,'external_archives':external,'failures':bad};(A/'RESTORE_INDEPENDENT.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))
