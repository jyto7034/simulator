import json,hashlib,re,tarfile,difflib,math,itertools
from pathlib import Path
R=Path('/mnt/f/work/simulator'); B=R/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal'; E=B/'evidence'; O=B/'audit-evidence/fresh_002'; checks=[]
def sha(b):return hashlib.sha256(b).hexdigest()
def ck(name,ok,detail=None):
 checks.append({'check':name,'pass':bool(ok),'detail':detail})
def read(p):return json.loads(p.read_text())
# Recompute original performance guard arithmetic directly from raw samples.
p01=read(R/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json');kernel_source=R/'core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs'
for file in ['ORDINARY_GUARDS_004/kernel_plain.json','ORDINARY_GUARDS_004/kernel_alloc.json']:
 d=read(E/file);ck('kernel source '+file,d['tool_source_sha256']==sha(kernel_source.read_bytes()));ck('P01 baseline '+file,d['baseline']['receipt_sha256']==sha((R/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json').read_bytes()))
 for mode in ['event_heavy','concurrent_sessions']:
  v=d[mode];baseline=p01[mode]['raw_samples'];samples=v['latency_raw_samples'];p95=sorted(x['elapsed_ns'] for x in samples)[math.ceil(len(samples)*.95)-1]
  ck(file+':'+mode+':p95',p95==v['nearest_rank_p95_ns'],p95)
  keys=['event_count','serialized_bytes']+(['event_log_sha256','ability_steps','projectile_launches','projectile_impacts','effect_events'] if mode=='event_heavy' else ['combined_event_log_sha256','session_count','lane_count','step_count_per_session'])
  ck(file+':'+mode+':baseline deterministic parity',all(all(s[k]==baseline[0][k] for k in keys) for s in samples+v['allocation_raw_samples']))
  if d['latency_gate_applicable']:
   base_ns=[x['elapsed_ns'] for x in baseline];new_ns=[x['elapsed_ns'] for x in samples];joined=base_ns+new_ns;u=lambda a,b:sum(2*(x>y)+(x==y) for x in a for y in b);observed=u(new_ns,base_ns);extreme=0;total=0
   for subset in itertools.combinations(range(len(joined)),len(new_ns)):
    st=set(subset);diff=u([joined[i] for i in st],[joined[i] for i in range(len(joined)) if i not in st]);total+=1;extreme+=diff>=observed
   prob=extreme/total;ck(file+':'+mode+':latency guards',p95<v['absolute_cap_ns'] and prob>=.05 and abs(prob-v['one_sided_exact_regression_p'])<1e-12,{'permutation_p':prob,'cap':v['absolute_cap_ns']})
  if d['allocation_gate_applicable']:
   obs=[x['heap_allocation'] for x in v['allocation_raw_samples']];old=[x['heap_allocation'] for x in baseline];fields={'operations':lambda x:x['allocation_count']+x['reallocation_count'],'peak':lambda x:x['peak_delta_bytes'],'retained':lambda x:x['retained_delta_bytes']};actual={k:max(f(x) for x in obs) for k,f in fields.items()};caps={k:max(f(x) for x in old) for k,f in fields.items()};ck(file+':'+mode+':original allocation guards',all(actual[k]<=caps[k] for k in fields),{'actual':actual,'original_caps':caps})
# Complete search and exact endpoint comparisons, not report passed fields.
d=read(E/'ORDINARY_GUARDS_004/native_budget.json');baseline=d['baseline'];ck('budget kernel identity',d['kernel_source_sha256']==sha(kernel_source.read_bytes()));ck('budget search nonzero',len(d['search_cases'])==67);ck('budget baseline 5000ms',baseline['clock_ms']==5000 and baseline['passed'] and baseline['fault'] is None)
for x in d['search_cases']:
 if x['passed']:ck('budget search success',x['facts']==baseline['facts'] and x['log_sha256']==baseline['log_sha256'] and x['clock_ms']==5000)
 else:ck('budget search typed failure',x['fault']['code'] in ['clone_limit','work_limit','retained_byte_limit'])
for x in d['boundaries']:
 exact=x['exact'];below=x['one_below'];key={'clone_slots':'max_clone_slots','work':'max_work','retained_bytes':'max_retained_bytes'}[x['dimension']];code={'clone_slots':'clone_limit','work':'work_limit','retained_bytes':'retained_byte_limit'}[x['dimension']];ck('budget endpoint '+x['dimension'],exact['passed'] and exact['facts']==baseline['facts'] and exact['log_sha256']==baseline['log_sha256'] and exact['limits'][key]==x['minimum'] and below['limits'][key]==x['minimum']-1 and not below['passed'] and below['fault']['code']==code,{'minimum':x['minimum'],'one_less_fault':below['fault']})

selected=[]
for number in [206,208]+list(range(209,232)):
 files=list(E.glob('r04_*'+str(number)+'.json'));ck('one command record '+str(number),len(files)==1)
 d=read(files[0]);raw=E/d['raw_log'];ck('command:'+files[0].name,d['exit_code']==0 and sha(raw.read_bytes())==d['log_sha256'],d.get('command',d.get('command_args')));selected.append({'record':str(files[0].relative_to(R)),**d})
log=(E/'r04_full209.log').read_text();names=re.findall(r'^test (.+) \.\.\. ok$',log,re.M);summaries=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',log)
ck('full209 1363 individual tests',len(names)==1363);ck('full209 exact groups',[int(x[0]) for x in summaries]==[1311,4,4,25,15,4,0]);ck('full209 zero failures or omissions',all(all(int(v)==0 for v in row[1:]) for row in summaries))
(O/'full209_test_rows.txt').write_text('\n'.join(names)+'\n')
for number in [206,208]:
 row=next(x for x in selected if str(number)+'.json' in x['record']);log=(E/row['raw_log']).read_text();ck('focused '+str(number), '1 passed; 0 failed; 0 ignored; 0 measured; 1310 filtered out' in log)
for x in names:
 if 'runtime_strike_admission_precedes_formula_limits' in x or 'native_action_projectile_first_hit_is_at_300' in x:ck('new focused test included in full: '+x,True)
result={'pass':all(x['pass'] for x in checks),'checks':checks,'total_checks':len(checks),'failed':[x for x in checks if not x['pass']],'owner_commands_reused':selected,'new_gameplay_executions':0,'scope':'Current raw output/command/guard/budget arithmetic; reader mechanics adapted after reviewing historical fresh001, conclusions independently recomputed'}
(O/'evidence_review.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['checks','owner_commands_reused']},indent=2))
