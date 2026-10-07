import json,hashlib,re,datetime
from pathlib import Path
H=Path('core_enhanced/docs/goals/core_relic_numeric_variants');A=H/'post_audit/u08_current_013_pass002';T=H/'execution/single_goal/target/u08_current_013';c=json.loads((H/'evidence/finalizer_ready013/CONFIG_READY_013.json').read_text());bad=[]
def check(ok,why):
 if not ok:bad.append(why)
def load(p):return json.loads(Path(p).read_text())
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
# Independent current membership, including external archive references.
mem=load(T/'MEMBERSHIP.json');check(len({r['path'] for r in mem})==len(mem),'duplicate_membership')
for r in mem:check(Path(r['path']).stat().st_size==r['bytes'] and sha(r['path'])==r['sha256'],['membership_bytes',r['path']])
check(hashlib.sha256(json.dumps(c['source']['files'],sort_keys=True).encode()).hexdigest()==c['source']['sha256'],'product_aggregate')
paths=set()
for folder in ('core_enhanced/src','core_enhanced/tests','game_server/src','game_server/tests','game_server/config','game_server/examples','config','game_resources/data','game_resources/live_data'):
 if Path(folder).exists():paths.update(str(p) for p in Path(folder).rglob('*') if p.is_file())
paths.update(['Cargo.toml','Cargo.lock','core_enhanced/Cargo.toml','game_server/Cargo.toml','core_enhanced/docs/audit/live_skill_catalog_manifest.ron','core_enhanced/docs/skills/live_content_index.md','core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/evidence/preimplementation/active_run_v7.pre_r06.json'])
check(paths=={r['path'] for r in c['source']['files']},'current_product_membership')
# Independent non-time PC differences against immediate U07M.
p=load(c['artifacts']['pc_cost']['ref']['path']);before=load(p['immediate_u07m']['path'])['rows'];after=load(p['after']['path'])['rows'];check(len(before)==len(after)==576,'pc_count');diffs={}
def walk(x,y,path):
 if type(x)!=type(y):bad.append(['pc_type',path]);return
 if isinstance(x,dict):
  check(x.keys()==y.keys(),['pc_keys',path])
  for k in x.keys()&y.keys():walk(x[k],y[k],path+'/'+k)
 elif isinstance(x,list):
  check(len(x)==len(y),['pc_length',path])
  for i,(u,v) in enumerate(zip(x,y)):walk(u,v,path+'/'+str(i))
 elif x!=y:
  generic=re.sub(r'/\d+', '/*',path);diffs[generic]=diffs.get(generic,0)+1
  if path in ['/measurement/elapsed_ns','/setup/elapsed_ns']:return
  bad.append(['pc_other_difference',path,x,y])
for x,y in zip(before,after):walk(x,y,'')
# The copied PC source uses a different tuple serialization digest; compare actual rows.
iso=load(c['artifacts']['pc_isolate']['ref']['path']);check({p:(b,s) for p,b,s in iso['current_source']['files']}=={r['path']:(r['bytes'],r['sha256']) for r in c['source']['files']},'pc_isolate_current_source')
# Public mixed Resource Sum raw output agrees with frozen per-case oracle, not summary flags.
obs=load(c['artifacts']['resource_sum']['ref']['path']);oracle=load(H/'evidence/r06_resource_sum_probe003/ORACLE.json')
for case,row in zip(oracle['cases'],obs['cases']):
 check(case['id']==row['id'] and case['expected_exit']==row['actual_exit'],['resource_case',row['id']]);p=H/'evidence/r06_resource_sum_probe003'/ (case['id']+('.stderr' if case['expected_exit'] else '.stdout'));raw=load(p);check(raw==row['actual_document'],['resource_raw',row['id']])
 if case['expected_exit']:
  d=raw['diagnostic'];check(d['expected']==case['expected_unit'] and d['actual']==case['actual_unit'] and d['code']=='value_expr_unit_mismatch' and d['phase']=='compile',['resource_unit',row['id']])
 else:check(raw['status']=='validated',['resource_positive',row['id']])
out={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'membership_count':len(mem),'product_count':len(paths),'pc_rows':len(after),'pc_differences':diffs,'resource_cases':len(obs['cases']),'failures':bad};(A/'ADDITIONAL_INDEPENDENT.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))
