from pathlib import Path
import json,hashlib
R=Path('/mnt/f/work/simulator');B=R/'core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal/evidence';O=R/'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal/audit-evidence/r04_correction_005';checks=[]
def j(p):return json.loads(p.read_text())
def check(n,v):checks.append({'name':n,'pass':bool(v)})
d=B/'PREDECESSOR_SOURCE_003/api';receipt=j(d/'receipt.json')
check('actual source counts',receipt['failed_replacements']==36 and receipt['successful_replacements']==3 and receipt['post_load_caller_edit_proofs']==3)
for generation in ['a','b']:
 raw=(d/'off'/(generation+'.ndjson')).read_bytes()
 for mode in ['on','gap']:check(generation+' complete receipt parity '+mode,(d/mode/(generation+'.ndjson')).read_bytes()==raw)
 for mode in ['off','on','gap']:
  x=j(d/mode/(generation+'_source.json'))
  check(generation+' actual source '+mode,bool(x) and x==j(d/'off'/(generation+'_source.json')))
r=j(B/'PREDECESSOR_REPLACEMENT_003/RESULT.json');check('actual replacement attempts count',len(r['attempts'])==54 and r['failed_attempts']==54 and r['acquisition_failures']==6 and r['replacement_failures']==48 and r['successful_replacements']==3 and r['a_continuation_replays']==3)
x={'new_gameplay_runs':0,'checks':checks,'pass':all(c['pass'] for c in checks),'source_receipt_counts':{k:v for k,v in receipt.items() if not isinstance(v,(list,dict))},'replacement_counts':{k:v for k,v in r.items() if not isinstance(v,(list,dict))}}
(O/'source_receipts.json').write_text(json.dumps(x,indent=2)+'\n');print(json.dumps(x,indent=2))
