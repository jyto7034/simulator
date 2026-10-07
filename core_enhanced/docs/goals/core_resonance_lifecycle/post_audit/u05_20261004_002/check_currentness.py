from pathlib import Path
import json,hashlib,datetime,sys
R=Path('/mnt/f/work/simulator'); G=R/'core_enhanced/docs/goals/core_resonance_lifecycle'; A=G/'post_audit/u05_20261004_002'; T=G/'execution/single_goal/target/u05_002'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
m=read(T/'MANIFEST.json'); errors=[]
expected='0d31d3036b42a088bc764cb7841d36f1544324ab56a8929db4437ce03b85319c'
if sha(T/'MANIFEST.json')!=expected: errors.append('manifest identity')
for row in read(T/'MEMBERSHIP.json'):
 p=R/row['path']
 if not p.is_file() or sha(p)!=row['sha256'] or p.stat().st_size!=row['bytes']:errors.append(row['path'])
for row in m['archives_and_patches']:
 p=T/row['path']
 if not p.is_file() or sha(p)!=row['sha256'] or p.stat().st_size!=row['bytes']:errors.append('target '+row['path'])
paths=set()
for folder in ('core_enhanced/src','core_enhanced/tests','game_server/src','game_server/tests','game_server/config','game_server/examples','config','game_resources/data','game_resources/live_data'):
 paths.update(str(x.relative_to(R)) for x in (R/folder).rglob('*') if x.is_file())
paths.update(('Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml','game_server/Cargo.toml','core_enhanced/docs/audit/live_skill_catalog_manifest.ron','core_enhanced/docs/skills/live_content_index.md','core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json'))
expectedpaths={x['path'] for x in m['product_source']['files']}
if paths!=expectedpaths:errors.append({'product_added':sorted(paths-expectedpaths),'product_removed':sorted(expectedpaths-paths)})
rows=[{'path':s,'sha256':sha(R/s),'bytes':(R/s).stat().st_size} for s in sorted(paths,key=Path)]
digest=hashlib.sha256(json.dumps(rows,sort_keys=True).encode()).hexdigest()
if digest!=m['product_source']['sha256']:errors.append('product digest')
for p in ['core_enhanced/docs/game_rulebook_next.md','core_enhanced/docs/game_rules_next']:
 if (R/p).exists():errors.append('superseded current publication '+p)
out={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'manifest_sha256':sha(T/'MANIFEST.json'),'membership_count':len(read(T/'MEMBERSHIP.json')),'target_artifacts_count':len(m['archives_and_patches']),'product_membership_count':len(paths),'product_sha256':digest,'combined_source_canonical_count':len(m['source_files']),'combined_sha256':m['source_sha256'],'errors':errors,'passed':not errors,'state':'current / unchanged' if not errors else 'stale'}
(A/sys.argv[1]).write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))
