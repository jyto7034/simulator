from pathlib import Path
import json,hashlib,tempfile,tarfile,subprocess,datetime
R=Path('/mnt/f/work/simulator');G=R/'core_enhanced/docs/goals/core_resonance_lifecycle';A=G/'post_audit/u05_20261004_002';T=G/'execution/single_goal/target/u05_002';m=json.loads((T/'MANIFEST.json').read_text());errors=[];runs=[]
with tempfile.TemporaryDirectory(prefix='u05-audit002-restore-') as tmp:
 root=Path(tmp)
 with tarfile.open(T/'SOURCE_BEFORE.tar.gz') as t:t.extractall(root,filter='data')
 for name,reverse,expected in [('forward',False,m['source_files']),('reverse',True,m['before_files'])]:
  p=subprocess.run(['patch',*(['-R'] if reverse else []),'-p1','--fuzz=0','--batch','--input',str(T/'FORWARD.patch')],cwd=root,capture_output=True,text=True)
  (A/('RESTORE_'+name.upper()+'.log')).write_text(p.stdout+p.stderr)
  observed=[{'path':str(f.relative_to(root)),'bytes':f.stat().st_size,'sha256':hashlib.sha256(f.read_bytes()).hexdigest()} for f in sorted(root.rglob('*')) if f.is_file()]
  observed.sort(key=lambda x:x['path'])
  ok=p.returncode==0 and observed==expected
  if not ok:errors.append(name)
  runs.append({'direction':name,'raw_exit':p.returncode,'members':len(observed),'exact_bytes_and_membership':observed==expected,'fuzz':0})
out={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'runs':runs,'errors':errors,'passed':not errors}
(A/'RESTORE_VERIFICATION.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))
