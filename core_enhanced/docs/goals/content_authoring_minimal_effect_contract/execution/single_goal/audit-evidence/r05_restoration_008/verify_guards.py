import json,math,hashlib
from pathlib import Path
R=Path('/mnt/f/work/simulator');B=R/'core_enhanced/docs/goals/content_authoring_resource_lifetimes/execution/single_goal/evidence/ORDINARY_GUARDS_003';O=Path(__file__).parent
read=lambda p:json.loads(p.read_text())
p95=lambda xs:sorted(xs)[math.ceil(len(xs)*.95)-1]
rows=[]
for mode in ['plain','alloc']:
 d=read(B/('public_'+mode+'.json'))
 for k,cap,bytecap in [('run_snapshot',500000,65536),('checkpoint_export',100000,32768)]:
  x=d[k];n=p95(x['raw_elapsed_ns']);assert n==x['nearest_rank_p95_ns'] and n<cap and x['encoded_bytes']<bytecap and len(x['raw_elapsed_ns'])==101
  rows.append({'mode':mode,'seam':k,'samples':101,'p95_ns':n,'cap_ns':cap,'bytes':x['encoded_bytes'],'bytecap':bytecap})
 h=d['bounded_history_query'];small=p95(h['small_raw_elapsed_ns']);large=p95(h['large_raw_elapsed_ns']);assert small==h['small_nearest_rank_p95_ns'] and large==h['large_nearest_rank_p95_ns'] and large<=small*4+10000
 t=d['transaction_staging'];low=p95(t['low_raw_elapsed_ns']);high=p95(t['high_raw_elapsed_ns']);assert high<=math.ceil(low*3/2)+50000 and t['rejection_rollback_exact']
 if mode=='alloc':
  lo=t['low_allocation_raw_samples'];hi=t['high_allocation_raw_samples']
  for f in [lambda x:x['allocation_count']+x['reallocation_count'],lambda x:x['peak_delta_bytes'],lambda x:x['retained_delta_bytes']]:assert max(map(f,hi))<=max(map(f,lo))
 baseline=read(R/'core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json');k=read(B/('kernel_'+mode+'.json'))
 for seam,cap in [('event_heavy',25000000),('concurrent_sessions',10000000)]:
  x=k[seam];n=p95([s['elapsed_ns'] for s in x['latency_raw_samples']]);assert n==x['nearest_rank_p95_ns'] and n<cap
  obs=[]
  if mode=='alloc':
   b=[s['heap_allocation'] for s in baseline[seam]['raw_samples']];a=[s['heap_allocation'] for s in x['allocation_raw_samples']]
   for label,f in [('allocation_operations',lambda x:x['allocation_count']+x['reallocation_count']),('peak_requested_bytes',lambda x:x['peak_delta_bytes']),('retained_requested_bytes',lambda x:x['retained_delta_bytes'])]:
    want=max(map(f,b));actual=max(map(f,a));assert actual<=want;obs.append({'dimension':label,'baseline_cap':want,'current_max':actual})
  hashkey='event_log_sha256' if seam=='event_heavy' else 'combined_event_log_sha256';expected=baseline[seam]['raw_samples'][0][hashkey];assert all(s[hashkey]==expected for s in x['latency_raw_samples']+x['allocation_raw_samples'])
  rows.append({'mode':mode,'seam':seam,'p95_ns':n,'cap_ns':cap,'original_event_hash':expected,'allocation_comparisons':obs})
n=read(B/'native_budget.json');assert len(n['search_cases'])==67 and n['baseline']['passed']
bounds=[]
for b in n['boundaries']:
 assert b['exact']['passed'] and not b['one_below']['passed'];f=b['one_below']['fault'];assert f['limit']==b['minimum']-1 and f['requested']==b['minimum'];bounds.append({'dimension':b['dimension'],'minimum':b['minimum'],'fault':f})
result={'passed':True,'recomputed_original_guards':rows,'native_search_cases':67,'native_boundaries':bounds,'claims':'Original thresholds and baseline hashes unchanged. Allocator requested bytes are not RSS. No benchmark reexecution.'};(O/'GUARD_VERIFICATION.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
