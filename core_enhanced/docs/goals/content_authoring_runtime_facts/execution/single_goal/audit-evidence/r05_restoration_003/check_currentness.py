import json, hashlib, sys, datetime, subprocess
from pathlib import Path
root=Path('/mnt/f/work/simulator')
target=root/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal/AUDIT_TARGET_005.json'
expected='9d4be8c4371fb2967f1fa39facff9a457cfc6c6c2a69f4daefa25decd4a0f3c9'
def digest(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(4194304),b''): h.update(b)
 return h.hexdigest()
d=json.loads(target.read_text()); failures=[]; total=0
for row in d['manifest']:
 p=root/row['path']; want=row['sha256']
 if want is None:
  if p.exists() or p.is_symlink(): failures.append({'path':row['path'],'error':'required absence exists'})
 elif not p.is_file(): failures.append({'path':row['path'],'error':'missing'})
 else:
  actual=digest(p); size=p.stat().st_size;total+=size
  if actual!=want or size!=row['bytes']: failures.append({'path':row['path'],'actual':actual,'expected':want,'bytes':size})
selfhash=digest(target)
if selfhash!=expected: failures.append({'target_self':selfhash})
canonical=hashlib.sha256(json.dumps(d['manifest'],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()
if canonical!=d['canonical_manifest_sha256']: failures.append({'canonical':canonical})
unique=len({r['path'] for r in d['manifest']})
if unique!=len(d['manifest']): failures.append({'duplicate_paths':True})
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
if head!=d['head']: failures.append({'head':head})
r={'checked_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'stage':sys.argv[1],'target':str(target.relative_to(root)),'target_sha256':selfhash,'manifest_rows':len(d['manifest']),'unique_paths':unique,'bytes_hashed':total,'canonical_recomputed':canonical,'canonical_expected':d['canonical_manifest_sha256'],'external_actual':d['external_actual'],'head':head,'failures':failures,'status':'current / unchanged' if not failures else 'stale'}
Path(__file__).with_name('CURRENTNESS_'+sys.argv[1]+'.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r,indent=2))
