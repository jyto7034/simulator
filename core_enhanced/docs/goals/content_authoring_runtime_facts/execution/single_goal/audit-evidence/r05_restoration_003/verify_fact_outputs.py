import json,hashlib,datetime
from pathlib import Path
R=Path('/mnt/f/work/simulator');O=Path(__file__).parent
B=R/'core_enhanced/docs/goals/content_authoring_resource_lifetimes/execution/single_goal/evidence/PREDECESSOR_CONSUMERS_001'
F=B/'runtime_facts';S=B/'runtime_source'
read=lambda p:json.loads(p.read_text())
errors=[];checks=0

def ck(v,label):
 global checks
 checks+=1
 if not v:errors.append(label)
base=read(F/'off/RESULT.json');data=(F/'off/receipts.ndjson').read_bytes();terminal=base['first_terminal']['runtime_facts']
for mode in ['off','on','gap','repeat']:
 d=read(F/mode/'RESULT.json')
 ck(d==base,mode+' whole result parity');ck((F/mode/'receipts.ndjson').read_bytes()==data,mode+' mandatory receipt parity')
 ck(d['failure'] is None and d['is_complete'],mode+' completed');ck(d['final_observation']['active_runtime_facts'] is None,mode+' owner retired')
 ck(d['first_terminal']==d['final_observation']['last_terminal'],mode+' terminal identity')
for mode in ['off','on']:ck((F/('cli_'+mode+'.stdout')).read_bytes()==data,mode+' API CLI equal')
ck(read(F/'gap/terminal_trace.json')['gap'] is not None,'gap exercised')
summary=terminal['summary'];ck((summary['damage_strikes'],summary['damage_mutations'],summary['damage_hp_lost'],summary['deaths'])==(91,91,277,2),'actual damage death counts')
ck(summary['latest_cost']['before']==20 and summary['latest_cost']['after']==10 and summary['latest_cost']['reason']=='tactical_deploy','actual tactical cost')
ck(terminal['usage']['public_result_reservations']==0 and terminal['usage']['settled_actions']>0,'actual internal settlement')
ck(terminal['latest_settled_action']['effects_settled_at_ms'] is not None,'settled timestamp')
ck(read(B/'runtime_application.json')['terminal']==terminal,'ordinary native terminal equals Trial')
ck(read(B/'runtime_application.json')['current_owner_after_result'] is None,'ordinary result owner retired')
providers=[p['provider'] for p in terminal['providers']];witnesses={}
def walk(v):
 if isinstance(v,dict):
  s=v.get('source')
  if 'hp_before' in v and 'hp_after' in v and isinstance(s,dict) and s.get('authored'):witnesses[s['authored']['site']['action']['action_id']]=v
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
for row in data.splitlines():walk(json.loads(row))
ck({'periodic_area','delayed_projectile'}<=witnesses.keys(),'two real delayed sources')
trace=[json.loads(x) for x in (F/'on/terminal_trace.ndjson').read_text().splitlines()]
for action,v in witnesses.items():
 s=v['source'];ck(v['hp_before']>v['hp_after'],action+' actual loss');ck(s['actor']==s['cast']['caster_instance_id'],action+' actual caster');ck(s['equipment'] in providers and s['equipment']['persistent_copy_uuid'] is not None,action+' actual provider');ck(s['action_ordinal'] is not None and 'ability_event_seq' not in s['cast'],action+' log-independent observed identity');ck(read(S/(action+'.stdout'))['source']==s['authored'],action+' public authored-source query')
 ck(any(row.get('source') and row['source']['authored']==s['authored'] and row['time_ms']==v['time_ms'] and row['target_unit']==v['target'] and row['change']['before']==v['hp_before'] and row['change']['after']==v['hp_after'] for row in trace),action+' trace crosscheck')
for sub in [F,S]:
 d=read(sub/'RESULT.json')
 for name,h in d['files'].items():ck(hashlib.sha256((sub/name).read_bytes()).hexdigest()==h,'sealed actual '+str(sub.name)+'/'+name)
ck(read(F/'describe.stdout')['runtime_observation']==read(F/'explain.stdout')['runtime_observation'],'descriptor common semantics')
raw=(B.parent/'resource_core_alltargets_103.log').read_text()
required=['runtime_strike_admission_precedes_formula_limits_and_rolls_back','native_action_projectile_first_hit_is_at_300_after_accept_0_release_100','native_action_piercing_empty_final_expiry_closes_carrier_without_releasing_gate','native_action_pending_death_preserves_cast_end_cost_without_release','runtime_facts_group_effect_major_damage_by_actual_iteration_and_target','runtime_facts_keep_overheal_zero_signed_heal_and_predeath_sources_separate','runtime_facts_resource_modifiers_do_not_become_payments','runtime_facts_lethal_damage_retains_combined_hp_transition','native_runtime_facts_preserve_actual_same_definition_inventory_copies_through_player_drafts','released_fixed_area_keeps_delivery_and_chain_resources_after_controller_withdrawal','released_payload_cannot_reuse_closed_activation_and_failure_keeps_committed_withdrawal','released_projectile_completion_retains_delivery_after_motion_and_controller_end','extra_attack_child_cast_preserves_its_actual_causal_chain']
for name in required:ck(any(x.endswith('::'+name+' ... ok') for x in raw.splitlines()),'actual current nonzero '+name)
reused=[]
P=R/'core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/audit-evidence/r05_restoration_005'
for name in ['COST_VERIFICATION.json','RESOURCE_OUTPUT_VERIFICATION.json']:
 p=P/name;d=read(p);ck(d['passed'] and not d['errors'],'reused independent '+name);reused.append({'path':str(p.relative_to(R)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'bytes':p.stat().st_size})
out={'checked_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'checks':checks,'passed':not errors,'errors':errors,'summary_counts':{k:summary[k]for k in ['damage_strikes','damage_mutations','damage_hp_lost','deaths','resource_spent']},'actual_source_witnesses':witnesses,'required_current_tests':required,'reused_sealed_same_input_computations':reused,'new_gameplay_executions':0,'scope':'Recompute current raw R04 facts/source/terminal from owner107; source assertions reviewed independently; R03 numerical cost and typed output computations reused after full input/registry identity checks.'}
(O/'FACT_OUTPUT_VERIFICATION.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({'checks':checks,'passed':not errors,'errors':errors}))
