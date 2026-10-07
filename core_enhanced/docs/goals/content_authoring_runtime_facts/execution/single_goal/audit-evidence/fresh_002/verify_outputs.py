# Reader mechanics adapted after reviewing historical fresh_001 script; all inputs and comparisons below are current target003.
import json,hashlib,collections
from pathlib import Path
R=Path('/mnt/f/work/simulator');B=R/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal';E=B/'evidence';O=B/'audit-evidence/fresh_002';checks=[]
def ck(n,v,d=None):checks.append({'check':n,'pass':bool(v),'detail':d})
def j(p):return json.loads(p.read_text())
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
command_counts={}
for directory in ['PREDECESSOR_PACK_003','PREDECESSOR_CLI_003','PREDECESSOR_API_003','PREDECESSOR_COMBAT_003','PREDECESSOR_RESOURCE_003','PREDECESSOR_BUFF_003','PREDECESSOR_INVALID_003','PREDECESSOR_DELIVERY_003','PREDECESSOR_PREVIEW_003','RUNTIME_FACT_EXTERNAL_003']:
 b=E/directory;commands=j(b/'commands.json');command_counts[directory]=len(commands)
 for c in commands:
  for field in ['stdout','stderr']:
   if field+'_sha256' in c:
    p=Path(c[field]);p=p if p.is_absolute() else b/p;ck(directory+':'+c['name']+':'+field,h(p)==c[field+'_sha256'])
# Full mandatory output equality in each actual mode/generation.
for directory,groups in [('RUNTIME_FACT_EXTERNAL_003',[['off','on','gap','repeat']]),('PREDECESSOR_COMBAT_003',[['a_off','a_on','a_gap'],['b_off','b_on','b_gap']]),('PREDECESSOR_RESOURCE_003',[['a_off','a_on','a_gap'],['b_off','b_on','b_gap']]),('PREDECESSOR_BUFF_003',[['a_off','a_on','a_gap'],['b_off','b_on','b_gap']]),('PREDECESSOR_DELIVERY_003',[['off','on','gap','repeat']])]:
 b=E/directory
 for group in groups:
  first=b/group[0];report=j(first/'RESULT.json');ck(directory+group[0]+':complete',report['failure'] is None and report['is_complete'])
  for mode in group[1:]:
   ck(directory+mode+':whole result parity',j(b/mode/'RESULT.json')==report)
   ck(directory+mode+':whole receipt bytes parity',(b/mode/'receipts.ndjson').read_bytes()==(first/'receipts.ndjson').read_bytes())
  # Match each captured simulate stdout to the corresponding whole reference.
  for c in j(b/'commands.json'):
   if 'simulate' in c['command']:
    name=c['name'];belongs=(group[0]=='off' or name.startswith('cli_'+group[0][0]))
    if belongs:ck(directory+name+':CLI parity',(b/(name+'.stdout')).read_bytes()==(first/'receipts.ndjson').read_bytes())
b=E/'RUNTIME_FACT_EXTERNAL_003';report=j(b/'off/RESULT.json');facts=report['first_terminal']['runtime_facts'];s=facts['summary'];ck('actual native totals',[s[k] for k in ['damage_mutations','damage_strikes','damage_hp_lost','deaths','cost_mutations','resource_spent']]==[91,91,277,2,1,10]);ck('native terminal/result lifetime',report['final_observation']['active_runtime_facts'] is None and report['final_observation']['last_terminal']==report['first_terminal']);ck('actual provider copies',any(x['provider']['persistent_copy_uuid'] for x in facts['providers']));ck('describe/explain same support',j(b/'describe.stdout')['runtime_observation']==j(b/'explain.stdout')['runtime_observation'])
a=j(E/'RUNTIME_APPLICATION_003.json');ck('ordinary application actual13/exact terminal/released owner',a['steps']==13 and a['terminal']==facts and a['current_owner_after_result'] is None)
source=j(E/'RUNTIME_FACT_SOURCE_003/RESULT.json');ck('source input exact',source['input_result_sha256']==h(b/'RESULT.json'))
for c in source['comparisons']:
 w=c['native_witness'];t=c['trace_witness'];ex=j(E/'RUNTIME_FACT_SOURCE_003'/(c['action']+'.stdout'));ck('native delayed source:'+c['action'],c['exit_code']==0 and w['source']['authored']==ex['source'] and w['source']['authored']==t['source']['authored'] and w['target']==t['target_unit'] and w['time_ms']==t['time_ms'] and w['hp_before']==t['change']['before'] and w['hp_after']==t['change']['after'] and w['hp_before']>w['hp_after'] and w['source']['equipment']['persistent_copy_uuid'] is not None)
# Recompute every raw cost sample's bookkeeping and all actual drop/query samples.
cost=j(E/'RUNTIME_COST_003/RESULT.json');reference=None;query_count=0;pairs=[];high=collections.defaultdict(int)
for name,count,repeats,instrumented in [('plain.json',685,5,False),('alloc.json',411,3,True)]:
 d=j(E/'RUNTIME_COST_003'/name);ck(name+':raw identity',h(E/'RUNTIME_COST_003'/name)==cost['raw'][name]);ck(name+':count/config',len(d['samples'])==count and d['repetitions']==repeats and d['allocator_instrumented']==instrumented and d['warmup_suites']==1)
 for result in d['results']:
  ck(name+':four logical generations',result['coexisting_independently_loaded_generations']==4)
  for mode in result['modes']:
   witness={k:mode[k] for k in ['accepted_steps','accepted_work','prepared_receipt_bytes','receipt_bytes','usage_by_receipt']}
   if reference is None:reference=witness
   ck(name+':whole mode receipt/work parity',witness==reference)
   for k,v in mode['runtime_facts_sampled_high_water'].items():high[k]=max(high[k],v)
 if instrumented:
  for sample in d['samples']:
   a=sample['native_requested_bytes'];ck('allocator arithmetic:'+sample['phase'],a['end_live']-a['baseline_live']==a['retained_delta'] and a['peak_live']-a['baseline_live']==a['peak_above_baseline'] and a['allocated_bytes']-a['freed_bytes']==a['retained_delta'])
   if '/runtime_fact_query/' in sample['phase']:
    query_count+=1;ck('borrowed query no allocation',a['alloc_calls']==0 and a['realloc_calls']==0 and a['retained_delta']==0)
  for it in range(1,repeats+1):
   samples={s['phase']:s for s in d['samples'] if s['iteration']==it}
   for g in range(1,5):
    retained=samples[f'coexist/load_prepare/{g}']['native_requested_bytes']['retained_delta'];reclaimed=-samples[f'coexist/drop/{g}']['native_requested_bytes']['retained_delta'];pairs.append({'iteration':it,'generation':g,'retained_bytes':retained,'reclaimed_bytes':reclaimed});ck('generation release',retained==reclaimed==1658777)
ck('cost query count',query_count==cost['zero_allocation_fact_queries']==126);ck('cost exact reclamation',pairs==cost['reclamation_pairs']);ck('cost sampled high water',dict(high)==cost['sampled_runtime_high_water'],dict(high))
result={'pass':all(c['pass'] for c in checks),'checks':checks,'total_checks':len(checks),'failed':[c for c in checks if not c['pass']],'commands_by_inventory':command_counts,'new_gameplay_executions':0,'scope':'read-only exact captured output comparisons; no implementation/runtime execution'}
(O/'output_review.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='checks'},indent=2))
