import json,hashlib,statistics
from pathlib import Path
O=Path(__file__).parent;R=Path('/mnt/f/work/simulator');E=R/'core_enhanced/docs/goals/content_authoring_resource_lifetimes/execution/single_goal/evidence';read=lambda p:json.loads(p.read_text());errors=[]
def ck(v,m):
 if not v:errors.append(m)
s=read(E/'COST_SUMMARY_002.json');result={}
for name,fixture in s['fixtures'].items():
 parsed=[]
 for filename,h in fixture['inputs'].items():
  p=E/filename;ck(hashlib.sha256(p.read_bytes()).hexdigest()==h,name+'raw hash');d=read(p);ck(d['passed'] and d['warmup_suites']==1,name+'passed');parsed.append(d)
 plain,alloc=sorted(parsed,key=lambda d:d['allocator_instrumented']);ck(len(plain['samples'])==fixture['plain_samples'] and len(alloc['samples'])==fixture['allocator_samples'],name+'counts');ck(plain['repetitions']==5 and alloc['repetitions']==3,name+'repetitions')
 for phase,summary in fixture['plain_latency_ns'].items():
  v=[x['elapsed_ns'] for x in plain['samples'] if x['phase']==phase];ck(len(v)==summary['count'] and min(v)==summary['min'] and max(v)==summary['max'] and statistics.median(v)==summary['median'],name+'latency '+phase)
 reads=[x for x in alloc['samples'] if '/typed_resource_query/' in x['phase']];ck(len(reads)==fixture['zero_allocation_typed_queries'],name+'reads')
 for x in reads:ck(x['native_requested_bytes']['alloc_calls']==0 and x['native_requested_bytes']['realloc_calls']==0 and x['native_requested_bytes']['allocated_bytes']==0,name+'readonly alloc')
 pairs=[]
 for i in range(1,4):
  for g in range(1,5):
   load=next(x for x in alloc['samples'] if x['iteration']==i and x['phase']==f'coexist/load_prepare/{g}')['native_requested_bytes'];drop=next(x for x in alloc['samples'] if x['iteration']==i and x['phase']==f'coexist/drop/{g}')['native_requested_bytes'];ck(load['retained_delta']>0 and load['retained_delta']==-drop['retained_delta'] and drop['alloc_calls']==0,name+'coexist');pairs.append(load['retained_delta'])
 reference=plain['results'][0]
 for d in [plain,alloc]:
  for rep in d['results']:
   for mode in rep['modes']:
    first=reference['modes'][0]
    for field in ['accepted_steps','accepted_work','prepared_receipt_bytes','receipt_bytes','usage_by_receipt']:
     ck(mode[field]==first[field],name+'deterministic '+field)
 result[name]={'plain_samples':len(plain['samples']),'allocator_samples':len(alloc['samples']),'zero_allocation_reads':len(reads),'verified_generation_drop_pairs':len(pairs),'retained_pair_bytes':pairs,'accepted_steps':reference['modes'][0]['accepted_steps'],'accepted_work':reference['modes'][0]['accepted_work'],'receipt_bytes':reference['modes'][0]['receipt_bytes'],'plain_latency':{k:fixture['plain_latency_ns'][k] for k in ['validate','off/prepare','off/replace/success']},'limitation':'System allocator requested bytes not RSS; fact retained bound excludes full typed store; complete retains accepted receipts/output.'}
out={'passed':not errors,'errors':errors,'fixtures':result,'fresh_execution':False,'method':'Recomputed current recorded samples, phase statistics, raw receipt equality across off/on/gap/repetitions/instrumentation, zero allocation counters and exact net generation drop pairs; no new benchmark.'};(O/'COST_VERIFICATION.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
