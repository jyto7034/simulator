from pathlib import Path
import json
R=Path('/mnt/f/work/simulator');E=R/'core_enhanced/docs/goals/content_authoring_resource_lifetimes/execution/single_goal/evidence';O=Path(__file__).parent;read=lambda p:json.loads(p.read_text());errors=[]
def ck(x,s):
 if not x:errors.append(s)
d=read(E/'TYPED_RESOURCE_EXTERNAL_006/RESULT.json');ck(d['passed'] and len(d['cases'])==15,'15 current cases')
for row in d['cases']:
 if 'typed_costs' in row:
  ck(bool(row['typed_costs']),'nonzero typed costs')
  for cost in row['typed_costs']:ck(cost['after']==row['expected'] and cost['resource']['typed']['lifetime']==row['case']['lifetime'] and bool(cost['source']['equipment']['persistent_copy_uuid']),'typed source/values')
a=read(E/'TYPED_RESOURCE_EXTERNAL_006/resource_256/adversarial.json');ck(a['passed'] and a['bounds_unchanged'] and a['split_replay_matches'] and a['resource_entries']==256,'256 actual results');ck(a['failures'][0]==a['failures'][1],'same repeated refusal')
for x in a['failures']:
 diag=x['error']['diagnostic'];ck(diag['code']=='battle_runtime_clone_limit' and diag['details']['requested']>diag['details']['limit'],'explicit refusal')
entries=a['split_final']['observation']['run_resources']['entries'];ck(len(entries)==256 and all(e['balance']==5 for e in entries),'256 projection');ck(a['split_final']['observation']['last_terminal']['actual_time_ms']==27450,'actual terminal');ck(a['split_final']['observation']['active_typed_resources'] is None,'terminal store gone')
b=read(E/'TYPED_RESOURCE_LIMITS_003.json');ck(b['passed'] and b['normal_replay_matches'] and len(b['cases'])==8,'8boundary cases')
for x in b['cases']:
 ck(x['preserved'] and x['refused_twice'][0]==x['refused_twice'][1],'exact refusal')
 for error in x['refused_twice']:ck(error['error']['diagnostic']['code']==x['expected'],'typed boundary code')
o=read(E/'TYPED_RESOURCE_ORDER_003/RESULT.json');ck(o['passed'] and o['pack_orders']==2 and o['reversed_declaration'] and o['atomic_shortage'],'order and shortage')
result={'passed':not errors,'errors':errors,'typed_cases':15,'original256_repeated_refusals':2,'original256_failure':'battle_runtime_clone_limit','split256_terminal_ms':27450,'split256_entries':256,'split256_balance':5,'typed_boundaries':8,'order_and_shortage':True,'new_gameplay_execution':False,'source_oracles':'authoring_typed_resource_external_probe.py; examples/typed_adversarial.rs and typed_limits.rs; original owner-produced output recomputation only'}
(O/'RESOURCE_OUTPUT_VERIFICATION.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
