import pathlib,json,math,hashlib
R=pathlib.Path('/mnt/f/work/simulator');B=R/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal';P=B/'evidence/ORDINARY_GUARDS_004';O=pathlib.Path(__file__).parent;checks=[]
def ck(n,b):checks.append({'check':n,'pass':bool(b)})
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def p95(xs):return sorted(xs)[math.ceil(len(xs)*.95)-1]
for n in ['public_plain','public_alloc']:
 d=json.loads((P/(n+'.json')).read_text());ck(n+' source',d['tool_source_sha256']==sha(R/'core_enhanced/tools/runtime_architecture_final_validation/src/public_guards.rs'))
 ck(n+' captured full stdout',json.loads((P/(n+'.log')).read_text())==d)
 for k,cap,bytescap in [('run_snapshot',500000,65536),('checkpoint_export',100000,32768)]:
  v=d[k];ck(n+k+' count',len(v['raw_elapsed_ns'])==v['sample_count']==101);ck(n+k+' p95',p95(v['raw_elapsed_ns'])==v['nearest_rank_p95_ns']);ck(n+k+' unchanged caps',v['absolute_cap_ns']==cap and v['encoded_byte_cap']==bytescap);ck(n+k+' encoded limit',v['encoded_bytes']<bytescap)
  if d['latency_gate_applicable']:ck(n+k+' latency',p95(v['raw_elapsed_ns'])<cap)
 h=d['bounded_history_query'];t=d['transaction_staging']
 for v,prefixes in [(h,['small','large']),(t,['low','high'])]:
  for pre in prefixes:ck(n+pre+' p95',p95(v[pre+'_raw_elapsed_ns'])==v[pre+'_nearest_rank_p95_ns'] and len(v[pre+'_raw_elapsed_ns'])==101)
 ck(n+' history actual sizes',(h['small_retained_entries'],h['large_retained_entries'],h['result_entries'])==(1024,8192,10))
 if d['latency_gate_applicable']:
  ck(n+' history scale',h['large_nearest_rank_p95_ns']<=4*h['small_nearest_rank_p95_ns'])
  ck(n+' staging scale',t['high_nearest_rank_p95_ns']<=(t['low_nearest_rank_p95_ns']*3+1)//2+50000)
 if d['allocation_gate_applicable']:
  low=t['low_allocation_raw_samples'];high=t['high_allocation_raw_samples'];ck(n+' nonzero alloc samples',len(low)>0 and len(high)>0)
  for f in [lambda x:x['allocation_count']+x['reallocation_count'],lambda x:x['peak_delta_bytes'],lambda x:x['retained_delta_bytes']]:ck(n+' original staging alloc guard',max(map(f,high))<=max(map(f,low)))
  for row in low+high:ck(n+' allocation arithmetic',row['peak_live_bytes']-row['baseline_live_bytes']==row['peak_delta_bytes'] and row['end_live_bytes']-row['baseline_live_bytes']==row['retained_delta_bytes'])
for row in json.loads((P/'commands.json').read_text()):ck('actual command '+row['name'],row['exit_code']==0 and sha(P/row['log'])==row['log_sha256'])
result={'pass':all(x['pass'] for x in checks),'checks':checks,'total_checks':len(checks),'failed':[x for x in checks if not x['pass']],'new_gameplay_executions':0,'scope':'Read-only arithmetic from current original public guards; repeatability and exact rollback additionally assessed in original harness source and successful captured invocation.'}
(O/'public_guard_review.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='checks'},indent=2))
