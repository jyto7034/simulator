# Read-only adaptation of historical R03 audit reader, independently reviewed in this pass.
import pathlib,json,hashlib,collections,math,datetime
R=pathlib.Path('/mnt/f/work/simulator');E=R/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal/evidence';O=pathlib.Path(__file__).parent
checks=[];details={}
def ck(n,v,d=None): checks.append({'check':n,'ok':bool(v),'detail':d})
def j(p):return json.loads(p.read_text())
def nd(p):return [json.loads(l) for l in p.read_text().splitlines() if l]
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def eq(a,b):ck('equal '+str(a.relative_to(E))+' '+str(b.relative_to(E)),a.read_bytes()==b.read_bytes())
# Same current real external battle receipts, including mandatory native fields.
for dirname in ['PREDECESSOR_COMBAT_003','PREDECESSOR_RESOURCE_003','PREDECESSOR_BUFF_003']:
 p=E/dirname
 for gen in ['a','b']:
  base=p/(gen+'_off')/'receipts.ndjson'
  for mode in ['on','gap']:eq(base,p/(gen+'_'+mode)/'receipts.ndjson')
  for mode in ['off','on']:
   cli=p/('cli_'+gen+'_'+mode+'.stdout')
   if cli.exists():eq(base,cli)
  report=j(p/(gen+'_off')/'RESULT.json');ck(dirname+' '+gen+' actual complete',report['is_complete'] and report['failure'] is None and report['accepted_steps']>0)
  details[dirname+' '+gen]={'steps':report['accepted_steps'],'work':report['accepted_work'],'receipt_bytes':report['receipt_bytes'],'first_sourced':report['first_sourced'],'first_terminal':report['first_terminal']}
# Independent nonzero A/B oracle.
a=j(E/'PREDECESSOR_COMBAT_003/a_off/RESULT.json')['first_sourced'];b=j(E/'PREDECESSOR_COMBAT_003/b_off/RESULT.json')['first_sourced'];ck('literal A18 B36 actual HP',a['change']['before']==b['change']['before']==79 and a['change']['after']==61 and b['change']['after']==43 and a['time_ms']==b['time_ms']==7950);ck('same authored combat source',a['source']==b['source'])
p=E/'PREDECESSOR_DELIVERY_003'
for mode in ['on','gap','repeat']:eq(p/'off/receipts.ndjson',p/mode/'receipts.ndjson')
for mode in ['off','on']:eq(p/'off/receipts.ndjson',p/('cli_'+mode+'.stdout'))
trace=nd(p/'on/terminal_trace.ndjson');counts=collections.Counter(row['source']['authored']['site']['action']['action_id'] for row in trace if row.get('source') and row['change']['channel']=='hp');details['delivery_counts']=dict(counts)
for name,count in [('delayed_repeat',14),('delayed_projectile',7),('periodic_area',48)]:ck('actual delivery '+name,counts[name]==count)
ck('unique semantic delivery ordinals',len({r['ordinal'] for r in trace})==len(trace));ck('actual terminal 19950',j(p/'off/RESULT.json')['first_terminal']['actual_time_ms']==19950)
# Source-byte correction remains current despite new native receipt fields.
p=E/'PREDECESSOR_SOURCE_003/api';src=j(p/'receipt.json');ck('source failed/swapped counts',src['failed_replacements']==36 and src['successful_replacements']==3 and src['post_load_caller_edit_proofs']==3)
for mode in ['off','on','gap']:
 a=nd(p/mode/'a.ndjson');b=nd(p/mode/'b.ndjson');sa=j(p/mode/'a_source.json');sb=j(p/mode/'b_source.json');ck('source identities differ '+mode,sa!=sb);ck('all exact source receipts '+mode,all(x['payload']['source_receipt']==sa for x in a) and all(x['payload']['source_receipt']==sb for x in b));
 for x in a+b:del x['payload']['source_receipt']
 ck('source only difference '+mode,a==b);ck('15 source documents '+mode,len(a)==len(b)==15)
 for gen in ['a','b']:eq(p/'off'/(gen+'.ndjson'),p/mode/(gen+'.ndjson'))
 details['source_current_sizes']= [{k:v for k,v in x.items() if k in ['mode','initial_bytes','ready_bytes','ready_work','final_work','final_bytes','steps','receipts','trace_rows','trace_gap']} for x in src['observations']]
# Current replacement uses actual A/B continuation outputs, not claimed booleans alone.
p=E/'PREDECESSOR_REPLACEMENT_003';d=j(p/'RESULT.json');ck('54 attempts 3 swaps',len(d['attempts'])==d['failed_attempts']==54 and d['successful_replacements']==3);details['replacement_cases']=dict(collections.Counter(a['case'] for a in d['attempts']))
for mode in ['off','on','gap']:
 eq(p/mode/'a_continued.ndjson',E/'PREDECESSOR_COMBAT_003'/('a_'+mode)/'receipts.ndjson');eq(p/mode/'b_replaced.ndjson',E/'PREDECESSOR_COMBAT_003'/('b_'+mode)/'receipts.ndjson')
# Preview snapshots and subsequent run equal across optional observations.
p=E/'PREDECESSOR_PREVIEW_003/api';d=j(p/'RESULT.json');ck('9 positive previews',d['positive_previews']==9 and d['api_replays']==4)
for mode in ['off','on','gap']:eq(p/'baseline_receipts.ndjson',p/(mode+'_receipts.ndjson'))
for s in d['summaries']:ck('manual preview actual HP '+s['mode'],s['manual_skill_hp']['change']['before']==10 and s['manual_skill_hp']['change']['after']==5)
# Current native ordinary public consumer parity and lifetime.
app=j(E/'RUNTIME_APPLICATION_003.json');facts=j(E/'RUNTIME_FACT_EXTERNAL_003/off/RESULT.json');ck('ordinary app13 / release owner',app['steps']==13 and app['current_owner_after_result'] is None)
ck('ordinary terminal equals Trial native facts',app['terminal']==facts['final_observation']['last_terminal']['runtime_facts']);details['ordinary_application']={'steps':app['steps'],'owner_after_result':app['current_owner_after_result']}
for mode in ['on','gap','repeat']:eq(E/'RUNTIME_FACT_EXTERNAL_003/off/receipts.ndjson',E/'RUNTIME_FACT_EXTERNAL_003'/mode/'receipts.ndjson')
# Every current measurement is recomputed from raw samples; no RSS inference.
plain=j(E/'RUNTIME_COST_003/plain.json');alloc=j(E/'RUNTIME_COST_003/alloc.json');ck('plain685 alloc411 repeats',len(plain['samples'])==685 and len(alloc['samples'])==411 and plain['repetitions']==5 and alloc['repetitions']==3);ck('one separate warmup',plain['warmup_suites']==alloc['warmup_suites']==1)
ck('plain uninstrumented',not plain['allocator_instrumented'] and all(s['native_requested_bytes'] is None for s in plain['samples']));ck('allocator instrumented',alloc['allocator_instrumented'])
reference=plain['results'][0]['modes'][0]['usage_by_receipt']
for label,data in [('plain',plain),('allocator',alloc)]:
 for result in data['results']:
  ck('four generations '+label+str(result['iteration']),result['coexisting_independently_loaded_generations']==4)
  for mode in result['modes']:
   ck('complete receipt/work parity '+label+str(result['iteration'])+mode['mode'],mode['usage_by_receipt']==reference and mode['accepted_steps']==23 and mode['accepted_work']==4706834 and mode['receipt_bytes']==1354821 and mode['prepared_receipt_bytes']==124036)
   ck('optional trace '+label+str(result['iteration'])+mode['mode'],mode['trace']['retained_rows']>0 if mode['mode']=='on' else mode['trace']['retained_rows']==0)
 for sample in data['samples']:ck('positive measured time '+label+sample['phase']+str(sample['iteration']),sample['elapsed_ns']>0)
for sample in alloc['samples']:
 m=sample['native_requested_bytes'];ck('allocator arithmetic '+sample['phase']+str(sample['iteration']),m['retained_delta']==m['end_live']-m['baseline_live']==m['allocated_bytes']-m['freed_bytes'] and m['peak_above_baseline']==m['peak_live']-m['baseline_live'] and m['peak_live']>=max(m['baseline_live'],m['end_live']))
queries=[s for s in alloc['samples'] if '/runtime_fact_query/' in s['phase']];ck('126 zero-allocation native queries',len(queries)==126 and all(s['native_requested_bytes']['alloc_calls']==s['native_requested_bytes']['realloc_calls']==s['native_requested_bytes']['allocated_bytes']==0 for s in queries))
pairs=[]
for iteration in [1,2,3]:
 for n in [1,2,3,4]:
  a=next(s['native_requested_bytes'] for s in alloc['samples'] if s['iteration']==iteration and s['phase']==f'coexist/load_prepare/{n}');b=next(s['native_requested_bytes'] for s in alloc['samples'] if s['iteration']==iteration and s['phase']==f'coexist/drop/{n}');ck(f'generation reclaim {iteration}/{n}',a['retained_delta']==-b['retained_delta']==1658777);pairs.append({'iteration':iteration,'generation':n,'retained':a['retained_delta'],'reclaimed':-b['retained_delta']})
details['cost']={'plain_samples':len(plain['samples']),'allocator_samples':len(alloc['samples']),'phases':len({s['phase'] for s in plain['samples']}),'zero_allocation_queries':len(queries),'generation_pairs':pairs,'sampled_high_water':plain['results'][0]['modes'][0]['runtime_facts_sampled_high_water'],'current_23step_work':4706834,'current_final_receipt_bytes':1354821}
# Current exact native limit search, including all captured binary-search cases.
budget=j(E/'ORDINARY_GUARDS_004/native_budget.json');base=budget['baseline'];ck('native 67 search cases',len(budget['search_cases'])==67 and base['passed'] and base['clock_ms']==5000);limits={'clone_slots':('max_clone_slots','clone_limit',843279),'work':('max_work','work_limit',2600736),'retained_bytes':('max_retained_bytes','retained_byte_limit',6090008)}
for row in budget['boundaries']:
 key,code,minimum=limits[row['dimension']];exact=row['exact'];low=row['one_below'];ck('exact native '+row['dimension'],row['minimum']==minimum and exact['limits'][key]==minimum and exact['passed'] and exact['facts']==base['facts'] and exact['log_sha256']==base['log_sha256']);ck('one-less native '+row['dimension'],not low['passed'] and low['limits'][key]==minimum-1 and low['fault']['code']==code)
for n,row in enumerate(budget['search_cases']):
 ck('native search case '+str(n),row['facts']==base['facts'] and row['log_sha256']==base['log_sha256'] if row['passed'] else row['fault']['code'] in [x[1] for x in limits.values()])
ck('native original harness hash',budget['kernel_source_sha256']==h(R/'core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs'));details['native_minima']={k:v[2] for k,v in limits.items()}
result={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'passed':all(c['ok'] for c in checks),'check_count':len(checks),'checks':checks,'details':details};(O/'output_review.json').write_text(json.dumps(result,indent=2)+'\n');print('checks',len(checks),'failures',[c for c in checks if not c['ok']]);print('details',json.dumps({k:v for k,v in details.items() if k in ['cost','native_minima','replacement_cases','source_current_sizes']},indent=2))
