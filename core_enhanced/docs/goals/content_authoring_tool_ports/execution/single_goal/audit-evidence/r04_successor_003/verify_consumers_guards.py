import pathlib,json,hashlib,math,datetime
R=pathlib.Path('/mnt/f/work/simulator');E=R/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal/evidence';O=pathlib.Path(__file__).parent
checks=[];details={}
def ck(n,v,d=None):checks.append({'check':n,'ok':bool(v),'detail':d})
def j(p):return json.loads(p.read_text())
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
streams=0;commands=0
for name in ['PREDECESSOR_PACK_002','PREDECESSOR_CLI_002','PREDECESSOR_API_002','PREDECESSOR_COMBAT_002','PREDECESSOR_RESOURCE_002','PREDECESSOR_BUFF_002','PREDECESSOR_INVALID_002','PREDECESSOR_DELIVERY_002','PREDECESSOR_PREVIEW_002','RUNTIME_FACT_EXTERNAL_002']:
 p=E/name/'commands.json'; rows=j(p);details[name+'_commands']=len(rows)
 for row in rows:
  commands+=1;n=name+' '+row.get('name','command')
  if 'expected_exit' in row:ck('expected exit '+n,row['exit_code']==row['expected_exit'])
  elif 'kind' in row:ck('consumer exit '+n,row['exit_code']==(0 if row['kind']=='valid' else 101))
  for key in ['stdout','stderr','log']:
   hashkey=key+'_sha256'
   if row.get(hashkey) and row.get(key):
    f=pathlib.Path(row[key]);f=f if f.is_absolute() else (E/name/f if (E/name/f).exists() else R/f);ck('retained stream '+n+' '+key,f.exists() and h(f)==row[hashkey]);streams+=1
  if row.get('required_diagnostics'):
   raw=pathlib.Path(row['log']).read_text();ck('actual compiler diagnostics '+n,all(s in raw for s in row['required_diagnostics']))
# CLI whole diagnostics equal actual API documents for each producer.
p=E/'PREDECESSOR_INVALID_002';cases=j(p/'cases.json');api=p/'api';ck('13 negative producer inputs',len(cases)==13)
for case in cases:
 name=case['name'];expected=j(api/(name+'.json'))
 for repeat in [0,1]:
  ck('actual API CLI diagnostic '+name+str(repeat),j(p/f'{name}_{repeat}.stderr')==expected and (p/f'{name}_{repeat}.stdout').stat().st_size==0)
 d=expected['diagnostic'];ck('producer code phase source '+name,d['code']==case['code'] and d['phase']==case['phase'] and d.get('pack_id')==case['pack'] and (not case['source'] or d['source_file']==case['source']));ck('related actual sources '+name,all(any(x['source_file']==path for x in d['related_sources']) for path in case['related']))
 if case.get('field'):ck('producer exact field '+name,d['field_path']==case['field'])
 ck('honest parser precision '+name,(d.get('position') is not None and d.get('parser_reason') is not None) if name == 'syntax' else d.get('position') is None and d.get('parser_reason') is None)
for repeat in [0,1]:ck('invalid authority API CLI '+str(repeat),j(p/f'invalid_authority_{repeat}.stderr')==j(api/'invalid_authority.json') and (p/f'invalid_authority_{repeat}.stdout').stat().st_size==0)
# Current ordinary guards: unchanged oracle source/input/threshold and raw arithmetic.
for file in ['ORDINARY_GUARDS_003/kernel_plain.json','CLONE_CAP_CHARACTERIZATION_001/finalalloc168.json']:
 d=j(E/file);ck('kernel original source '+file,d['tool_source_sha256']==h(R/'core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs'));ck('original baseline '+file,d['baseline']['receipt_sha256']==h(R/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json'))
 for k,cap in [('event_heavy',25000000),('concurrent_sessions',10000000)]:
  row=d[k];samples=row['latency_raw_samples'];p95=sorted(s['elapsed_ns'] for s in samples)[math.ceil(len(samples)*.95)-1];ck('plain percentile '+file+k,p95==row['nearest_rank_p95_ns'] and row['absolute_cap_ns']==cap and p95<=cap);ck('original gameplay parity '+file+k,row['repeatable'] and row['deterministic_baseline_parity'] and all(s['event_log_sha256']==samples[0]['event_log_sha256'] for s in samples) if k=='event_heavy' else row['repeatable'] and row['deterministic_baseline_parity']);
  for i,s in enumerate(row['allocation_raw_samples']):
   a=s['heap_allocation'];ck('kernel requested memory '+file+k+str(i),a['retained_delta_bytes']==a['end_live_bytes']-a['baseline_live_bytes'] and a['peak_delta_bytes']==a['peak_live_bytes']-a['baseline_live_bytes'])
  if row['allocation_raw_samples']:
   samples=row['allocation_raw_samples'];details[k+'_allocation']={'max_peak':max(s['heap_allocation']['peak_delta_bytes'] for s in samples),'max_ops':max(s['heap_allocation']['allocation_count']+s['heap_allocation']['reallocation_count'] for s in samples)};ck('unchanged allocation caps '+k,row['allocation_retained_peak_guard_pass'])
  else:details[k+'_plain_p95_ns']=p95
for file in ['public_plain.json','public_alloc.json']:
 d=j(E/'ORDINARY_GUARDS_003'/file);ck('public original guard source '+file,d['tool_source_sha256']==h(R/'core_enhanced/tools/runtime_architecture_final_validation/src/public_guards.rs'))
 for k in ['run_snapshot','checkpoint_export','bounded_history_query','transaction_staging']:
  row=d[k]
  if 'raw_elapsed_ns' in row:
   vals=row['raw_elapsed_ns'];p95=sorted(vals)[math.ceil(len(vals)*.95)-1];ck('public raw percentile '+file+k,p95==row['nearest_rank_p95_ns'] and p95<=row['absolute_cap_ns'])
  for key,v in row.items():
   if key.endswith('_pass') or key=='exact_repeatable':ck('public guard '+file+k+key,v)
result={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'passed':all(x['ok'] for x in checks),'check_count':len(checks),'retained_commands':commands,'stream_identity_checks':streams,'checks':checks,'details':details};(O/'consumer_guard_review.json').write_text(json.dumps(result,indent=2)+'\n');print('checks',len(checks),'failures',[x for x in checks if not x['ok']]);print('details',details)
