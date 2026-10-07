from pathlib import Path
from collections import Counter
import json,hashlib,math
R=Path('/mnt/f/work/simulator');P=R/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal';E=P/'evidence';O=P/'audit-evidence/fresh_002'
def j(p):return json.loads(p.read_text())
def nd(p):return [json.loads(l) for l in p.read_text().splitlines()]
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
out={}
# Validate actual exit/stream identities and compiler-error reasons, not merely PASS labels.
for d in ['COMBAT_EXTERNAL_003','DELIVERY_EXTERNAL_001','CLI_EXTERNAL_004','INVALID_EXTERNAL_002','API_EXTERNAL_002']:
 rows=j(E/d/'commands.json');v=[]
 for r in rows:
  if 'expected_exit' in r:assert r['exit_code']==r['expected_exit']
  for stream in ['stdout','stderr','log']:
   if r.get(stream) and r.get(stream+'_sha256'):assert h(Path(r[stream]))==r[stream+'_sha256']
  if d=='API_EXTERNAL_002':
   assert (r['exit_code']==0)==(r['kind']=='valid')
   for required in r['required_diagnostics']:assert required in Path(r['log']).read_text()
  v.append({'name':r['name'],'exit_code':r['exit_code'],'expected_exit':r.get('expected_exit'),'required_diagnostics':r.get('required_diagnostics',[])})
 out[d+'_commands']=v
combat={}
for gen in ['a','b']:
 raw=(E/'COMBAT_EXTERNAL_003'/f'{gen}_off/receipts.ndjson').read_bytes()
 for mode in ['on','gap']:assert raw==(E/'COMBAT_EXTERNAL_003'/f'{gen}_{mode}/receipts.ndjson').read_bytes()
 rows=nd(E/'COMBAT_EXTERNAL_003'/f'{gen}_on/terminal_trace.ndjson');hp=[r for r in rows if r['change']['channel']=='hp' and r.get('source') and r['source']['authored']['site']['effect_id']=='effect_01'];assert hp
 first=hp[0];assert first['change']['before']==79;assert first['change']['after']==(61 if gen=='a' else 43);docs=nd(E/'COMBAT_EXTERNAL_003'/f'{gen}_off/receipts.ndjson');assert len(docs)==15
 source=docs[0]['payload']['source_receipt'];assert len(source['files'])==25;assert all(d['payload']['source_receipt']==source for d in docs)
 combat[gen]={'first_sourced_hp':first,'receipt_documents':len(docs),'receipt_sha256':hashlib.sha256(raw).hexdigest()}
out['combat']=combat
D=E/'DELIVERY_EXTERNAL_001';raw=(D/'off/receipts.ndjson').read_bytes()
for m in ['on','gap','repeat']:assert raw==(D/m/'receipts.ndjson').read_bytes()
rows=nd(D/'on/terminal_trace.ndjson');hp=[r for r in rows if r['change']['channel']=='hp' and r.get('source')];counts=Counter(r['source']['authored']['site']['action']['action_id'] for r in hp);assert counts['delayed_repeat']==14 and counts['delayed_projectile']==7 and counts['periodic_area']==48
keys=[(r['ordinal'],r['time_ms'],r['target_unit']) for r in hp];assert len(keys)==len(set(keys));out['delivery']={'source_hp_by_action':dict(counts),'scope':'Historical execution/source/ordering/trace invariance only; old whole receipt/work is not current.'}
for gen in ['a','b']:
 D=E/'BUFF_EXTERNAL_001';raw=(D/f'{gen}_off/receipts.ndjson').read_bytes()
 for m in ['on','gap']:assert raw==(D/f'{gen}_{m}/receipts.ndjson').read_bytes()
 rows=nd(D/f'{gen}_on/terminal_trace.ndjson');sourced=[r for r in rows if r.get('source')];channels=Counter(r['change']['channel'] for r in sourced);assert channels['buff']>0 and channels['resonance']>0 and channels['hp']>0;out['buff_'+gen]={'channels':dict(channels),'first_by_channel':{c:next(r for r in sourced if r['change']['channel']==c) for c in channels}}
D=E/'INVALID_EXTERNAL_002';cases=j(D/'cases.json');out['invalid']=[]
for c in cases:
 api=j(D/'api'/(c['name']+'.json'));assert isinstance(api,dict)
 for n in [0,1]:
  actual=j(D/(c['name']+'_'+str(n)+'.stderr'));assert actual==api
 diag=api['diagnostic'];assert diag['phase']==c['phase'] and diag['code']==c['code'];assert diag['source_file']==(c['source'] if c['source'] is not None else 'selection');assert diag['pack_id']==c['pack']
 for related in c['related']:assert related in [r['source_file'] for r in diag['related_sources']]
 out['invalid'].append({'name':c['name'],'code':diag['code'],'phase':diag['phase'],'source_file':diag['source_file'],'pack_id':diag['pack_id'],'position':diag['position'],'expected':diag['expected'],'actual':diag['actual'],'related_sources':diag['related_sources']})
D=E/'PREVIEW_EXTERNAL_004/api';raw=(D/'baseline_receipts.ndjson').read_bytes()
for m in ['off','on','gap']:
 assert raw==(D/(m+'_receipts.ndjson')).read_bytes();queries=j(D/(m+'_previews.json'));assert len(queries)==3 and all(q==queries[0] for q in queries);assert queries[0]['disposition']=='read_only';assert queries[0]['payload']['eligibility']['enabled']
out['preview']={'queries':9,'same_full_historical_receipts':True,'manual_hp':j(D/'RESULT.json')['summaries'][0]['manual_skill_hp'],'scope':'Readonly query/gameplay retained; old receipt/work values superseded.'}
D=E/'REPLACEMENT_EXTERNAL_005';attempts=j(D/'attempts.json');assert len(attempts)==54
for m in ['off','on','gap']:
 assert (D/m/'a_continued.ndjson').read_bytes()==(E/'COMBAT_EXTERNAL_003'/f'a_{m}/receipts.ndjson').read_bytes();assert (D/m/'b_replaced.ndjson').read_bytes()==(E/'COMBAT_EXTERNAL_003'/f'b_{m}/receipts.ndjson').read_bytes()
out['replacement']={'attempt_count':len(attempts),'cases':dict(Counter(r['case'] for r in attempts)),'modes':3,'current_full_a_b_continuations_match':True}
# Recompute sample counts, requested-byte arithmetic, percentile and cross-build results.
D=E/'COST_EXTERNAL_003';plain=j(D/'plain.json');alloc=j(D/'alloc.json');base=plain['results'][0];base.pop('iteration');cost={}
for name,a,n in [('plain',plain,5),('allocator',alloc,3)]:
 assert a['repetitions']==n and len(a['samples'])==95*n and a['warmup_suites']==1
 phases=Counter(s['phase'] for s in a['samples']);assert len(phases)==95 and set(phases.values())=={n}
 for result in a['results']:
  result.pop('iteration',None);assert result==base
  modes=result['modes'];assert all(m['usage_by_receipt']==modes[0]['usage_by_receipt'] for m in modes);assert all(m['accepted_steps']==23 and m['receipt_bytes']==624152 and m['accepted_work']==3815638 for m in modes)
 for s in a['samples']:
  mem=s['native_requested_bytes'];assert (mem is not None)==(name=='allocator')
  if mem:
   assert mem['end_live']-mem['baseline_live']==mem['retained_delta']==mem['allocated_bytes']-mem['freed_bytes'];assert mem['peak_live']>=max(mem['end_live'],mem['baseline_live'])
 selected={}
 for phase in ['validate','off/prepare','on/prepare','off/step/22','off/replace/success','coexist/load_prepare/4','coexist/drop/1']:
  samples=[s for s in a['samples'] if s['phase']==phase];times=sorted(s['elapsed_ns'] for s in samples);selected[phase]={'count':len(samples),'p95_ns':times[math.ceil(.95*len(times))-1],'requested_bytes':[s['native_requested_bytes'] for s in samples]}
 cost[name]={'samples':len(a['samples']),'phases':len(phases),'selected':selected}
out['cost']=cost;out['cost']['cross_binary_results_equal']=True;out['cost']['modes']=[{k:v for k,v in m.items() if k!='usage_by_receipt'} for m in base['modes']]
# Current 23-step receipt gameplay matches old Preview after removing only explicitly changed byte/work metadata.
old=nd(E/'PREVIEW_EXTERNAL_004/api/baseline_receipts.ndjson');new=base['modes'][0]['usage_by_receipt'];assert len(old)==len(new)
for a,b in zip(old,new):
 a=a['payload'].copy();b=b['payload'].copy();b.pop('source_receipt');a.pop('prior_accepted_work',None);b.pop('prior_accepted_work',None);assert a==b
out['current_preview_gameplay_equal']=True
(O/'outputs_check.json').write_text(json.dumps(out,indent=2)+'\n');print('PASS',list(out));print('delivery',out['delivery']);print('current replacement',out['replacement']);print('cost',out['cost']['modes'])
