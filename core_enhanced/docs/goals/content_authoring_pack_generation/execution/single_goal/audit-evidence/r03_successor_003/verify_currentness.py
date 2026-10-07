import json, pathlib, hashlib, subprocess, datetime, sys
ROOT=pathlib.Path('/mnt/f/work/simulator')
BASE=ROOT/'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal'
OUT=BASE/'audit-evidence/r03_successor_003'
PRED=ROOT/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal'
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def verify(m,base=ROOT):
 p=base/m['path']; expected=m.get('sha256')
 if expected is None: assert not p.exists(),str(p); return
 assert p.is_file() and sha(p)==expected,str(p)
 if 'bytes' in m: assert p.stat().st_size==m['bytes'],str(p)
def check_target(p,expected):
 assert sha(p)==expected
 d=json.loads(p.read_text())
 for m in d['manifest']: verify(m)
 n=0
 for e in d['external_actual']:
  base=ROOT/e['directory']; actual={str(q.relative_to(base)) for q in base.rglob('*') if q.is_file() or q.is_symlink()}
  assert actual=={m['path'] for m in e['files']},str(base)
  for m in e['files']:verify(m,base);n+=1
 for m in d['default_release_binaries']+d['validation_results']:verify(m)
 return d,{'target':str(p.relative_to(ROOT)),'sha256':sha(p),'manifest_members':len(d['manifest']),'absences':sum(m['sha256'] is None for m in d['manifest']),'external_directories':len(d['external_actual']),'external_files':n,'release_binaries':len(d['default_release_binaries'])}
d,summary=check_target(BASE/'AUDIT_TARGET_004.json','ccfbd726948fb4ceb2a7de99016b4526fa2972fe481ac5c3d95c25ee77724cae')
pd,ps=check_target(PRED/'AUDIT_TARGET_004.json','965e2accba9aeaca67d9aedae39a94f15438b865aa05d4e6f01575a1cfab4f13')
reg=PRED/'audit-evidence/r03_successor_004/AUDIT_OUTPUTS.json'
assert sha(reg)=='24943fc9fda4b663e1c5811047f4a148c6312fb59b898f8c55ca33b7969f7ed6'
outputs=json.loads(reg.read_text())['files']
for m in outputs:verify(m)
assert sha(PRED/'COMPLETION_AUDIT_004.md')=='750168985ace4b476954a494b983748514f4f4bf56e45b7c5592bad5a30dbfcd'
old=json.loads((BASE/'AUDIT_TARGET_003.json').read_text()); missing=[e['directory'] for e in old['external_actual']]
assert len(missing)==7 and all(not pathlib.Path(p).exists() for p in missing)
observed={k:subprocess.check_output(cmd,cwd=ROOT,text=True).strip() for k,cmd in {'head':['git','rev-parse','HEAD'],'branch':['git','branch','--show-current'],'rustc':['rustc','--version'],'cargo':['cargo','--version'],'environment':['uname','-a']}.items()}
for k in ['head','branch']: assert observed[k]==d[k]
for k in ['rustc','cargo','environment']: assert observed[k]==d['toolchain'][k]
r={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'current / unchanged','target':summary,'required_predecessor':ps,'predecessor_outputs_verified':outputs,'predecessor_registry_sha256':sha(reg),'old_external_actual_absent':missing,'environment':observed,'failures':[]}
(OUT/sys.argv[1]).write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({k:v for k,v in r.items() if k!='predecessor_outputs_verified'},indent=2))
