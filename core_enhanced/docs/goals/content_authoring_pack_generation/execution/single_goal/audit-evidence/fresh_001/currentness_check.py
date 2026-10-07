import json,hashlib,subprocess,datetime,os,sys
from pathlib import Path
R=Path('/mnt/f/work/simulator'); B=R/'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal'; O=B/'audit-evidence/fresh_001';O.mkdir(parents=True,exist_ok=True)
def sha(p):
 p=Path(p);return hashlib.sha256(p.read_bytes()).hexdigest() if p.is_file() else None
t=json.loads((B/'AUDIT_TARGET_002.json').read_text()); errors=[]
for m in t['manifest']+t['default_release_binaries']:
 if sha(R/m['path'])!=m['sha256']:errors.append(m['path'])
for ext in t['external_actual']:
 p=Path(ext['directory']); actual=sorted(str(f.relative_to(p)) for f in p.rglob('*') if f.is_file());expected=sorted(m['path'] for m in ext['files'])
 if actual!=expected:errors.append({'inventory':str(p)})
 for m in ext['files']:
  if sha(p/m['path'])!=m['sha256']:errors.append(str(p/m['path']))
pred=json.loads((R/t['predecessor']['target']).read_text());changed=[]
for m in pred['manifest']:
 h=sha(R/m['path'])
 if h!=m['sha256']:changed.append({'path':m['path'],'before':m['sha256'],'after':h})
reuse=json.loads((B/'evidence/COMMENT_REUSE_002.json').read_text()); src=R/'core_enhanced/src/game/data/mod.rs';now=src.read_text();assert now.count(reuse['exact_new_text'])==1;old=now.replace(reuse['exact_new_text'],reuse['exact_old_text']);assert hashlib.sha256(old.encode()).hexdigest()==reuse['source_before_sha256']
vi=json.loads((B/'VALIDATION_INPUT_023.json').read_text());badvi=[p for p,h in [(m['path'],m['sha256']) for m in vi['inputs']] if sha(R/p)!=h]
oldvi=json.loads((B/'VALIDATION_INPUT_022.json').read_text());videlta=[p for p,h in [(m['path'],m['sha256']) for m in oldvi['inputs']] if {m['path']:m['sha256'] for m in vi['inputs']}.get(p)!=h]
result={'observed_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'target_sha256':sha(B/'AUDIT_TARGET_002.json'),'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip(),'direct_members':len(t['manifest']),'external_members':sum(len(x['files']) for x in t['external_actual']),'binaries':len(t['default_release_binaries']),'target_mismatches':errors,'predecessor_changes':changed,'predecessor_changes_match_claim':{m['path'] for m in changed}=={m['path'] for m in reuse['changed_predecessor_target_members']},'exact_comment_reconstruction':True,'VI023_mismatches':badvi,'VI022_to_023_delta':videlta,'toolchain':{c:subprocess.check_output([c,'--version'],text=True).strip() for c in ['rustc','cargo']},'environment':{k:os.getenv(k) for k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','CARGO_TARGET_DIR']}}
(O/(sys.argv[1]+'.json')).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2));assert not errors and not badvi
