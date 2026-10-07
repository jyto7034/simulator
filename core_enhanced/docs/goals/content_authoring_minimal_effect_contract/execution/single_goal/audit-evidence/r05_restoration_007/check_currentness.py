import json, hashlib, sys, datetime, subprocess
from pathlib import Path
root=Path('/mnt/f/work/simulator')
target=root/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/AUDIT_TARGET_007.json'
expected='9c9737b6ab314134e0c39ddac995eaab27b0605be23d64e4722bc830d905fe96'
def digest(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(4*1024*1024),b''):h.update(b)
 return h.hexdigest()
d=json.loads(target.read_text()); failures=[]; total=0
for row in d['manifest']:
 p=root/row['path']; want=row['sha256']
 if want is None:
  if p.exists():failures.append({'path':str(p),'reason':'required absence exists'})
 elif not p.is_file():failures.append({'path':str(p),'reason':'missing'})
 else:
  actual=digest(p); total+=p.stat().st_size
  if actual!=want or p.stat().st_size!=row['bytes']:failures.append({'path':str(p),'actual':actual,'expected':want,'bytes':p.stat().st_size})
selfhash=digest(target)
if selfhash!=expected:failures.append({'target_self':selfhash})
canonical=hashlib.sha256(json.dumps(d['manifest'],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()
r={'checked_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'stage':sys.argv[1],'target':str(target.relative_to(root)),'target_sha256':selfhash,'manifest_rows':len(d['manifest']),'unique_paths':len({r['path'] for r in d['manifest']}),'bytes_hashed':total,'canonical_recomputed':canonical,'canonical_expected':d['canonical_manifest_sha256'],'external_actual':d['external_actual'],'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'failures':failures,'status':'current / unchanged' if not failures else 'stale'}
Path(__file__).with_name('CURRENTNESS_'+sys.argv[1]+'.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r,indent=2))
