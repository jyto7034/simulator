import json, hashlib, pathlib, subprocess, datetime, sys
root=pathlib.Path('/mnt/f/work/simulator')
base=root/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal'
target=base/'AUDIT_TARGET_004.json'
d=json.loads(target.read_text()); errors=[]
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def check(row, prefix=root):
 p=prefix/row['path']; expected=row.get('sha256')
 if expected is None:
  if p.exists(): errors.append({'unexpected_present':str(p)})
  return
 if not p.is_file(): errors.append({'missing':str(p)});return
 actual=sha(p)
 if actual!=expected or ('bytes'in row and p.stat().st_size!=row['bytes']):errors.append({'mismatch':str(p),'expected':expected,'actual':actual})
for r in d['manifest']:check(r)
external=0
for r in d['external_actual']:
 p=root/r['directory']; expected={x['path'] for x in r['files']}; actual={str(x.relative_to(p)) for x in p.rglob('*') if x.is_file()}
 if expected!=actual:errors.append({'inventory':str(p),'missing':sorted(expected-actual),'extra':sorted(actual-expected)})
 for x in r['files']:check(x,p);external+=1
for r in d['default_release_binaries']+d['validation_results']:check(r)
old=json.loads((base/'AUDIT_TARGET_003.json').read_text());old_external=[]
for x in old['external_actual']:
 p=pathlib.Path(x['directory']);old_external.append({'path':str(p),'exists':p.exists()})
 if p.exists():errors.append({'old_external_unexpected_present':str(p)})
env={}
for k,c in [('head',['git','rev-parse','HEAD']),('branch',['git','branch','--show-current']),('rustc',['rustc','--version']),('cargo',['cargo','--version']),('environment',['uname','-a'])]:
 env[k]=subprocess.check_output(c,cwd=root,text=True).strip()
 expected=d[k] if k in ['head','branch'] else d['toolchain'][k]
 if env[k]!=expected:errors.append({'environment':k,'expected':expected,'actual':env[k]})
actual_sha=sha(target)
if actual_sha!='965e2accba9aeaca67d9aedae39a94f15438b865aa05d4e6f01575a1cfab4f13':errors.append({'self_sha':actual_sha})
result={'captured_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'target_sha256':actual_sha,'manifest_count':len(d['manifest']),'hashed_members':sum(bool(x.get('sha256')) for x in d['manifest']),'required_absences':sum(x.get('sha256') is None for x in d['manifest']),'external_directories':len(d['external_actual']),'external_files':external,'release_binaries':d['default_release_binaries'],'old_external':old_external,'environment':env,'errors':errors,'result':'pass' if not errors else 'fail'}
out=base/'audit-evidence/r03_successor_004'/sys.argv[1];out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2));sys.exit(bool(errors))
