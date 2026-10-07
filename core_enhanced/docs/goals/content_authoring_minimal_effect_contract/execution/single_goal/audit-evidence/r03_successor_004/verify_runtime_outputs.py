from pathlib import Path
import json,hashlib,datetime
R=Path('/mnt/f/work/simulator');E=R/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/evidence';O=R/'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/audit-evidence/r03_successor_004'
def read(p):return json.loads(p.read_text())
def rows(p):return [json.loads(x) for x in p.read_text().splitlines()]
result={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'checks':[]}
b=E/'COMBAT_EXTERNAL_002';summaries={}
for generation in ['a','b']:
 baseline=(b/f'{generation}_off/receipts.ndjson').read_bytes()
 for mode in ['off','on','gap']:
  d=read(b/f'{generation}_{mode}/RESULT.json');summaries[generation]=d if mode=='off' else summaries[generation]
  assert d['is_complete'] and d['accepted_steps']==13 and d['failure'] is None
  assert (b/f'{generation}_{mode}/receipts.ndjson').read_bytes()==baseline
 for mode in ['off','on']:assert (b/f'cli_{generation}_{mode}.stdout').read_bytes()==baseline
witness=[summaries[x]['first_sourced'] for x in ['a','b']]
assert [x['change']['before'] for x in witness]==[79,79]
assert [x['change']['after'] for x in witness]==[61,43]
for f in ['ordinal','time_ms','source_unit','target_unit','source']:assert witness[0][f]==witness[1][f]
result['checks'].append({'combat_AB_hp':[[79,61],[79,43]],'same_occurrence_source':True,'off_on_gap_cli_receipts_equal':True})
b=E/'DELIVERY_EXTERNAL_001';baseline=(b/'off/receipts.ndjson').read_bytes()
for mode in ['on','repeat','gap']:assert (b/mode/'receipts.ndjson').read_bytes()==baseline
for mode in ['off','on']:assert (b/f'cli_{mode}.stdout').read_bytes()==baseline
trace=rows(b/'on/terminal_trace.ndjson');counts={}
for action,effect in [('delayed_repeat','repeat_hit'),('delayed_projectile','projectile_hit'),('periodic_area','area_hit')]:
 selected=[x for x in trace if x['source'] and x['source']['authored']['site']['action']['action_id']==action]
 assert selected and all(x['change']['channel']=='hp' and x['change']['before']>x['change']['after'] and x['source']['authored']['site']['effect_id']==effect for x in selected)
 explanation=read(b/f'explain_{action}.stdout');assert explanation['source']==selected[0]['source']['authored'];counts[action]=len(selected)
assert counts=={'delayed_repeat':14,'delayed_projectile':7,'periodic_area':48}
result['checks'].append({'delivery_applications':counts,'off_on_gap_repeat_cli_equal':True})
b=E/'PREVIEW_EXTERNAL_004';baseline=(b/'api/off_receipts.ndjson').read_bytes()
for mode in ['on','gap']:assert (b/f'api/{mode}_receipts.ndjson').read_bytes()==baseline
for mode in ['off','on']:assert (E/f'FINAL_EXTERNAL_001/cli_{mode}.log').read_bytes()==baseline
assert (E/'FINAL_EXTERNAL_001/cli_trace.ndjson').read_bytes()==(b/'cli_trace.ndjson').read_bytes()
result['checks'].append({'preview_current_cli_receipts_and_trace_equal':True})
result['result']='pass';(O/'runtime_outputs.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
