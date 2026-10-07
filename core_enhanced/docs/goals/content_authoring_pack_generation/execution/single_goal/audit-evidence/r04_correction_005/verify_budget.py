import pathlib,json,hashlib,math
R=pathlib.Path('/mnt/f/work/simulator');D=R/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal/evidence/ORDINARY_GUARDS_004';O=R/'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal/audit-evidence/r04_correction_005';x=json.loads((D/'native_budget.json').read_text());checks=[]
def req(c,label,actual=None):checks.append({'check':label,'passed':bool(c),'actual':actual})
req(len(x['search_cases'])==67,'67 measured search cases',len(x['search_cases']))
base=x['baseline']; req(base['passed'] and base['fault'] is None and base['clock_ms']==5000,'default five-second native baseline')
for b,expected,code in zip(x['boundaries'],[843279,2600736,6090008],['clone_limit','work_limit','retained_byte_limit']):
 e=b['exact'];n=b['one_below'];req(b['minimum']==expected and e['passed'] and e['fault'] is None and e['clock_ms']==5000 and e['facts']==base['facts'] and e['log_sha256']==base['log_sha256'],b['dimension']+' exact bound full facts/log parity',b['minimum']);req(not n['passed'] and n['fault']['code']==code and n['fault']['limit']==expected-1 and n['fault']['requested']==expected,b['dimension']+' one below typed failure',n['fault'])
for name in ['public_plain.json','public_alloc.json']:
 v=json.loads((D/name).read_text());req(v['tool_source_sha256']==hashlib.sha256((R/'core_enhanced/tools/runtime_architecture_final_validation/src/public_guards.rs').read_bytes()).hexdigest(),name+' original source identity')
 for key in ['run_snapshot','checkpoint_export']:
  q=v[key];p95=sorted(q['raw_elapsed_ns'])[math.ceil(len(q['raw_elapsed_ns'])*.95)-1];req(p95==q['nearest_rank_p95_ns'],name+' '+key+' p95',p95)
  if v['latency_gate_applicable']:req(p95<q['absolute_cap_ns'],name+' '+key+' original latency cap')
  if 'encoded_bytes' in q:req(q['encoded_bytes']<=q['encoded_byte_cap'],name+' '+key+' encoded size cap')
  if 'exact_repeatable' in q:req(q['exact_repeatable'],name+' '+key+' repeatability')
 q=v['bounded_history_query']
 for size in ['small','large']:
  samples=q[size+'_raw_elapsed_ns'];req(sorted(samples)[math.ceil(len(samples)*.95)-1]==q[size+'_nearest_rank_p95_ns'],name+' history '+size+' p95')
 req(q['large_nearest_rank_p95_ns']*q['scaling_limit_denominator']<=q['small_nearest_rank_p95_ns']*q['scaling_limit_numerator'] and q['all_caps_respected'] and q['exact_repeatable'],name+' history scale/caps/parity')
 req(v['transaction_staging']['rejection_rollback_exact'],name+' transaction exact rollback')
out={'fresh_gameplay_runs' :0,'checks':checks,'passed':all(c['passed'] for c in checks)};(O/'budget_review.json').write_text(json.dumps(out,indent=2)+'\n');print('checks',len(checks),'passed',out['passed']);print([c for c in checks if not c['passed']])
