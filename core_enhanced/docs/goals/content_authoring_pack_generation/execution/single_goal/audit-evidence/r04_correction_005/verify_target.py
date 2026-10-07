from pathlib import Path
import hashlib,json,subprocess,datetime,sys
R=Path('/mnt/f/work/simulator');D=R/'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal';O=D/'audit-evidence/r04_correction_005'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def verify(p,expected):
 x=json.loads(p.read_text());errors=[];checks=0;absences=0
 def member(m,base=R):
  nonlocal checks,absences
  checks+=1;q=base/m['path']
  if m['sha256'] is None:
   absences+=1
   if q.exists() or q.is_symlink():errors.append([str(q),'required absence'])
  elif not q.is_file() or q.stat().st_size!=m['bytes'] or sha(q)!=m['sha256']:errors.append([str(q),'identity mismatch'])
 for m in x['manifest']:member(m)
 ext=0
 for inv in x['external_actual']:
  base=R/inv['directory'];actual={str(q.relative_to(base)) for q in base.rglob('*') if q.is_file()};wanted={m['path'] for m in inv['files']}
  if actual!=wanted:errors.append([str(base),'inventory mismatch',sorted(actual-wanted),sorted(wanted-actual)])
  for m in inv['files']:member(m,base);ext+=1
 for m in x['default_release_binaries']:member(m)
 # All additional identity references, including prerequisite reports/registries, are real captured bytes.
 def refs(v):
  if isinstance(v,dict):
   if {'path','bytes','sha256'}<=v.keys():member(v)
   else:
    for z in v.values():refs(z)
  elif isinstance(v,list):
   for z in v:refs(z)
 for k,v in x.items():
  if k not in ['manifest','external_actual','default_release_binaries']:refs(v)
 env={'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip(),'branch':subprocess.check_output(['git','branch','--show-current'],cwd=R,text=True).strip(),'toolchain':{a:subprocess.check_output([a,'--version'],cwd=R,text=True).strip() for a in ['rustc','cargo']}}
 if any(env[k]!=x[k] for k in env):errors.append(['environment mismatch',env])
 if sha(p)!=expected:errors.append(['target self mismatch',sha(p)])
 return {'target':str(p.relative_to(R)),'sha256':sha(p),'manifest_members':len(x['manifest']),'required_absences':absences,'external_inventories':len(x['external_actual']),'external_files':ext,'default_release_binaries':len(x['default_release_binaries']),'checks_including_overlap':checks,'environment':env,'errors':errors,'currentness':'current / unchanged' if not errors else 'stale'}
x=json.loads((D/'AUDIT_TARGET_006.json').read_text());res={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'R02':verify(D/'AUDIT_TARGET_006.json','3e57d1f541f7f1ac94ee9638c8cf2f6ff6821e2c81668350da698c2921f56ee1')}
p=x['required_predecessors'][0];res['R01']=verify(R/p['target']['path'],p['target']['sha256'])
reg=json.loads((R/p['outputs']['path']).read_text());res['prerequisite_registry']=reg
errors=[]
def check_registry(v):
 if isinstance(v,dict):
  if {'path','sha256','bytes'}<=v.keys():
   q=R/v['path']
   if not q.is_file() or sha(q)!=v['sha256'] or q.stat().st_size!=v['bytes']:errors.append(v['path'])
  else:
   for z in v.values():check_registry(z)
 elif isinstance(v,list):
  for z in v:check_registry(z)
check_registry(reg);res['registry_errors']=errors
(O/sys.argv[1]).write_text(json.dumps(res,indent=2)+'\n');print(json.dumps({k:v for k,v in res.items() if k!='prerequisite_registry'},indent=2))
