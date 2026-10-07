from pathlib import Path
import hashlib,json,subprocess,sys,datetime
ROOT=Path('/mnt/f/work/simulator'); BASE=ROOT/'core_enhanced/docs/goals'; OUT=BASE/'content_authoring_tool_ports/execution/single_goal/audit-evidence/fresh_001'
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1048576),b''):h.update(b)
 return h.hexdigest()
def check(r,base=ROOT):
 p=base/r['path']; actual=sha(p) if p.is_file() else None
 return None if actual==r.get('sha256') and (actual is not None or not p.exists()) and ('bytes' not in r or p.stat().st_size==r['bytes']) else {'path':str(p),'expected':r,'actual':actual}
results=[]
for topic,n,expected in [('content_authoring_tool_ports','001','030f634b050c9bf6a292d4a61ac6308e80cd73c093fca5593cb7d575c61dc4b1'),('content_authoring_minimal_effect_contract','004','965e2accba9aeaca67d9aedae39a94f15438b865aa05d4e6f01575a1cfab4f13'),('content_authoring_pack_generation','004','ccfbd726948fb4ceb2a7de99016b4526fa2972fe481ac5c3d95c25ee77724cae')]:
 p=BASE/topic/'execution/single_goal'/f'AUDIT_TARGET_{n}.json';d=json.loads(p.read_text()); errors=[]
 for r in d['manifest']+d['default_release_binaries']+d['validation_results']:
  e=check(r)
  if e:errors.append(e)
 inventories=[]
 for x in d['external_actual']:
  b=ROOT/x['directory']; actual=sorted(str(f.relative_to(b)) for f in b.rglob('*') if f.is_file());expected_files=sorted(f['path'] for f in x['files'])
  if actual!=expected_files:errors.append({'inventory':str(b),'extra':sorted(set(actual)-set(expected_files)),'missing':sorted(set(expected_files)-set(actual))})
  for r in x['files']:
   e=check(r,b)
   if e:errors.append(e)
  inventories.append({'directory':x['directory'],'files':len(actual)})
 actual_env={k:subprocess.check_output(c,cwd=ROOT,text=True).strip() for k,c in {'head':['git','rev-parse','HEAD'],'branch':['git','branch','--show-current'],'rustc':['rustc','--version'],'cargo':['cargo','--version'],'environment':['uname','-a']}.items()}
 for k,v in actual_env.items():
  if v!=(d.get(k) if k in ['head','branch'] else d['toolchain'][k]):errors.append({'environment':k,'actual':v})
 if sha(p)!=expected:errors.append({'target_self':str(p),'actual':sha(p)})
 results.append({'target':str(p.relative_to(ROOT)),'target_sha256':sha(p),'manifest_members':len(d['manifest']),'required_absences':sum(r.get('sha256') is None for r in d['manifest']),'binaries':len(d['default_release_binaries']),'external_inventories':inventories,'environment':actual_env,'errors':errors})
registries=[]
for topic,folder,expected in [('content_authoring_minimal_effect_contract','r03_successor_004','24943fc9fda4b663e1c5811047f4a148c6312fb59b898f8c55ca33b7969f7ed6'),('content_authoring_pack_generation','r03_successor_003','08f4e68e7f2271ea718474a95c3b33dcaabb5c24e2f704edfbe8ecb612f2178a')]:
 p=BASE/topic/'execution/single_goal/audit-evidence'/folder/'AUDIT_OUTPUTS.json'; d=json.loads(p.read_text()); errors=[e for r in d['files'] if (e:=check(r))]
 if sha(p)!=expected:errors.append({'registry_hash':sha(p),'expected':expected})
 registries.append({'path':str(p.relative_to(ROOT)),'sha256':sha(p),'outputs':len(d['files']),'errors':errors})
r={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'pass':sys.argv[1],'targets':results,'predecessor_output_registries':registries,'currentness':'current / unchanged' if not any(x['errors'] for x in results+registries) else 'stale'}
(OUT/f'currentness_{sys.argv[1]}.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r,indent=2))
