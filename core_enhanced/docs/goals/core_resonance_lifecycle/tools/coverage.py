"""Bind U05 and every inherited acceptance to actual current-source executions."""
import copy, json, re
from pathlib import Path
import run
from condition_basis import DELTAS, SOURCES as REQUIREMENT_SOURCES, inherited_meanings

R,H=run.ROOT,run.HERE
S,E=H/'execution/single_goal',H/'evidence'
selection=json.loads((S/'EVIDENCE_SELECTION.json').read_text())
current=run.identity(); bindings={}; sha=run.sha
def write(path,value):
    with path.open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
def receipt(name,allowed=(0,)):
    p=E/(name+'.json');v=json.loads(p.read_text())
    assert v['exit_code'] in allowed and v['source_unchanged'],name
    assert v['source_before']==current==v['source_after'],name
    assert v['log_sha256']==sha(E/(name+'.log')),name
    bindings[name]={'receipt_sha256':sha(p),'log_sha256':v['log_sha256'],'source_sha256':current['sha256'],'raw_exit_code':v['exit_code']}
    return v
suites=selection['suites'];passed={};counts={}
for name in suites:
    receipt(name);t=(E/(name+'.log')).read_text()
    passed[name]=set(re.findall(r'^test (\S+) \.\.\. ok$',t,re.M))
    totals=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',t)
    counts[name]={key:sum(int(row[i]) for row in totals) for i,key in enumerate(['passed','failed','ignored'])}
    assert counts[name]['passed']>0 and counts[name]['failed']==0,name
core=set.intersection(*(passed[n] for n in suites[:3]));host=passed[suites[3]]&passed[suites[4]]
observed=passed[suites[2]]|passed[suites[4]]
def select(fragments,pool):
    result=set()
    for fragment in fragments:
        found={s for s in pool if fragment in s};assert found,fragment;result.update(found)
    return sorted(result)
contracts={
 'V01':(['u05_selected_form_capacity_floor','u05_actual_work_relocation','consumable_initial_skill_charge'],[]),
 'V02':(['u05_employee_attack_gains','u05_actual_work_relocation','u05_all_positive_commands','u05_timed_shield','platform_rest'],['u05_paused_accepted_use']),
 'V03':(['u05_all_positive_commands','u05_timed_shield','u05_mixed_uses'],[]),
 'V04':(['u05_admission_consumes_once','u05_use_state_survives','u05_pulse_and_use_counter','r09_start_gate_uses_admission'],['u05_paused_accepted_use']),
 'V05':(['u05_last_projectile','u05_extra_attacks','u05_extra_attack_projectiles','u05_finite_modifiers','u05_selected_finite_projectile','u05_mixed_uses','u05_independent_dot','u05_independent_area_attacks','u05_repeat_branch','r11_channel','r11_hello'],[]),
 'V06':(['u05_accepted_use_ignores','u05_extra_attacks','u05_timed_shield','u05_actor_death','control_status_hard_cc','control_status_hard_cc_cancels_unreleased','r11_channel_silence'],[]),
 'V07':(['u05_direct_target_withdrawal','manual_player_unit_invalidation','manual_player_unit_tile_area_uses_acceptance_tile_after_target_moves_or_withdraws'],[]),
 'V08':(['u05_empty_tile','manual_player_selection_requires','manual_tile_cast_accepts_valid_empty_geometry','hostile_skill_targeting','cast_usefulness_uses_actual'],[]),
 'V09':(['player_unit_skill_focus_preview_cancel_and_paused_activation_are_atomic','effect_preview_matches_runtime_area_union','u05_use_state_survives'],['u05_paused_accepted_use']),
 'V10':(['manual_player_unit_tile_area_uses_acceptance_tile_after_target_moves_or_withdraws','manual_player_unit_invalidation','self_then_retargeted_enemy_skill'],[]),
 'V11':(['r09_public_ron_start_replaces_cast','r09_start_gate_uses_admission','r09_corrosion_faults','r09_auto_uses_same_actual_start','r09_world_pause_start_host_speed_terminal_employee_cost_and_save','r09_public_equipment_death_reactions'],[]),
 'V12':(['u05_employee_authoring_rejects','u05_public_descriptor','u05_selected_form_capacity_floor','live_skill_catalog_has_exact','live_skills_pin_explicit','u05_native_use_scans','u05_pulse_and_use_counter'],['u05_paused_accepted_use']),
 'V13':(['u05_use_state_survives','current_v11_','u04_continue_unfinished_battle','u04_savepoint_load','r09_v8_explicit_conversion'],['u05_paused_accepted_use','previous_core_version_protects_current_and_backup_from_all_writes']),
 'V14':(['u05_native_use_scans','u05_pulse_and_use_counter','u05_use_state_survives'],[]),
}
compat_names=['prepared_public','prepared_boundary','observed_release','default_parity','public_oracle','public_trace','public_cli','public_profile','public_boundary','public_api','pc_prepare','pc','pc_compare','pc_immediate','r05','r05_compare']
compat={name:selection.get('compatibility_overrides',{}).get(name,'compat_'+name+selection['compatibility_suffix']) for name in compat_names}
for name in selection['checks']:
    receipt(name)
    if name.startswith('integrated_heap'):
        assert re.search(r'test result: ok\. 1 passed; 0 failed;', (E/(name+'.log')).read_text()),name
for name,key in compat.items():receipt(key,(0,101) if name=='r05' else (0,))
compat_dir=S/selection['compatibility_directory']
pc_path=compat_dir/'evidence/PC_SUMMARY_CURRENT_003.json'
pc=json.loads(pc_path.read_text())
assert pc['same_gameplay_oracle_and_typed_refusal'] and pc['rows_each']==576
assert (pc['distinct_conditions'],pc['successful_conditions'],pc['typed_refusal_conditions'])==(36,31,5)
pc_raw=compat_dir/'evidence/pc_raw_current_003.json'
assert pc['current']['sha256']==sha(pc_raw)
accounted_path=compat_dir/'evidence/PC_IMMEDIATE_ACCOUNTED_U05.json'
accounted=json.loads(accounted_path.read_text())
assert accounted['passed'] and accounted['rows']==576 and accounted['after']['sha256']==sha(pc_raw)
assert accounted['before']['sha256']==sha(R/accounted['before']['path'])
assert all(sha(R/path)==digest for path,digest in accounted['source_basis'].items())
assert len(accounted['per_row_cost_changes'])==576
adaptation_path=S/selection['compatibility_adaptation']
adaptation=json.loads(adaptation_path.read_text())
assert adaptation['source_sha256']==current['sha256']
assert all(sha(R/row['copy'])==row['copy_sha256'] for row in adaptation['files'])
assert all(sha(R/row['path'])==row['sha256'] for row in adaptation['added_harness_files'])
r05_path=S/selection['r05_summary'];r05=json.loads(r05_path.read_text())
assert r05['source_digest']==current['sha256'] and not r05['other_hard_guard_failures']
assert r05['raw_exit_code']==json.loads((E/(compat['r05']+'.json')).read_text())['exit_code']
assert r05['raw_sha256']==sha(S/selection['compatibility_directory']/'evidence/r05_raw_current_002.json')
fp=selection['fault_prefix']
for name in ['build_on','none','detached','queue_full','on','websocket','build_off','off']:receipt(fp+name)
fault=json.loads((E/(fp+'SUMMARY.json')).read_text());assert fault['passed'] and fault['source']==current
for key in ('continuation','rewind','schema','independent_area'):
    receipt(selection[key]);assert json.loads((E/selection[key+'_summary']).read_text())['passed']
public=[selection['continuation'],selection['rewind']]
direct={k:[] for k in contracts}
for key in ('V04','V09','V11','V13'):direct[key]+=public
for key in ('V01','V05','V08','V10','V12'):direct[key]+=[compat['public_cli'],selection['schema']]
direct['V05'].append(selection['independent_area'])
direct['V12']+=[compat['public_profile'],compat['public_api'],compat['public_boundary']]
direct['V14']=list(compat.values())+selection['checks']+[fp+'SUMMARY.json']+public
freeze=json.loads((H/'P2G_30_FREEZE_001.json').read_text())['artifacts'][0]
acceptances=copy.deepcopy(freeze['payload']['Observable Policy Acceptance Set'])
assert len(acceptances)==14
validations=[]
for index,(key,(c,h)) in enumerate(contracts.items()):
    validations.append({'id':key,'requirement':acceptances[index]['Observable Policy Acceptance'],
        'core_selectors':select(c,core),'host_selectors':select(h,host),'suite_receipts':suites,'direct_receipts':direct[key],
        'status':'current self-verification; independent applicability audit pending'})
    acceptances[index].update(validation_refs=[key],status='self-verified; independent audit pending')

prior=R/'core_enhanced/docs/goals/core_result_persistence/execution/single_goal'
old=json.loads((prior/'PREDECESSOR_IMPACT.json').read_text())
oldcoverage=json.loads((prior/'COVERAGE_003.json').read_text())
original_meanings=inherited_meanings()
renames={
 'r12_shielded_basic_attack_grants_release_charge_but_no_damage_charge_or_lifesteal':'r12_shielded_employee_attack_grants_no_charge_and_lifesteals_only_hp_loss',
 'r09_start_gate_uses_current_eligibility_and_ignores_stale_events':'r09_start_gate_uses_admission_credential_and_ignores_stale_events',
 'r09_v8_explicit_conversion_and_v9_counter_validation_preserve_original':'r09_v8_explicit_conversion_and_current_counter_validation_preserve_original',
 'manual_player_unit_tile_area_uses_acceptance_tile_after_target_moves':'manual_player_unit_tile_area_uses_acceptance_tile_after_target_moves_or_withdraws',
 'current_v10_checkpoint_continue_then_v10_capture_is_semantically_identical':'current_v11_checkpoint_continue_then_v11_capture_is_semantically_identical',
}
def mapped(s):
    prefix,_,name=s.rpartition('::')
    return prefix+'::'+renames.get(name,name.replace('current_v10_','current_v11_')) if prefix else renames.get(name,name)
def inherited_selectors(row):
    found=[]
    def walk(value):
        if isinstance(value,dict):
            for key,child in value.items():
                if 'predecessor' in key:continue
                if 'selectors' in key and isinstance(child,list):found.extend(s for s in child if isinstance(s,str))
                else:walk(child)
        elif isinstance(value,list):
            for child in value:walk(child)
    walk(row)
    result={mapped(s) for s in found}
    assert result<=core|host|observed,result-(core|host|observed)
    return sorted(result)
impacts={
 'A08':('Autonomous boss source, release/settlement and cancellation remain authoritative; player employee use adds a separate admission/lease owner. Re-run boss tests and both independent-area and combo last-release cases.',['V05','V06','V14']),
 'A09':('Corrosion now consumes at accepted employee input and uses that credential at the existing actual start. Preserve forced pause, special death, trauma, withdrawal, no second cost and saved employee outcomes.',['V04','V11','V13']),
 'A10':('Use lifetime is distinct from Flow Released/EffectsSettled. Preserve authored control, expression and payload order; final projectile can outlive use, future combo/branch/repeat cannot. Manual anchors are fixed and direct target loss cancels future control.',['V05','V06','V07','V08','V09','V10']),
 'A11':('Actual finite beneficial leases own employee use through expiry/early consumption/channel exit. Independent area, hostile status and permanent effects keep their own owners. Action/resource settlement remains distinct.',['V03','V05','V06','V12']),
 'A12':('Preserve shield scaling, HP-cost/lifesteal/death and Parry outcomes. Employee charge gain during active use is intentionally refused; old shield recast uses actual shield exhaustion before recharge.',['V03','V05','V06','V11']),
 'IVA':('Typed arithmetic, bindings, filters and read-only queries retain their requirements. Public trial readiness is migrated explicitly to prototype max25 and ordinary23s charge; original amount/trace/error oracles and performance hard guards remain. Changed saved schemas retain explicit original protection and current restoration evidence.',['V01','V08','V09','V12','V13','V14']),
 'F':('Actual selected fragment copy/form now owns resonance_max; preserve recipes, exact copy/source identities, resource lifetime and rollback. Durable employee authoring is strict version11; explicit historical conversion outputs version12.',['V01','V04','V12','V13']),
 'PE-CRSO':('Native and authoring budgets account for new use slots, source strings, queue pulses and lifecycle scans. Keep numeric limits, public first-fault/observer parity and service plumbing; rerun changed producers and original public guards.',['V04','V05','V12','V13','V14']),
 'PE-CRFI':('Admission and automatic pulses use existing candidate rollback and sticky-fault boundaries. Faults cannot publish cost/use/queue changes; normal refusals remain distinct and same-actor reconnect preserves accepted state.',['V04','V06','V12','V13','V14']),
 'X':('Cross-feature consumers now receive explicit lifetime/capacity/schema and immediate employee cost. Re-run exact inherited selector bindings plus public RON/CLI/host/native cases; preserve original independent owners and refusal oracles.',['V01','V04','V05','V11','V12','V13','V14']),
}
conditions=[]
for row in old['conditions']:
    key=row['id'];group=next((g for g in ('PE-CRSO','PE-CRFI','IVA','A08','A09','A10','A11','A12','F','X') if key.startswith(g)),None)
    assert group,key
    meaning,seams=impacts[group]
    if key in DELTAS:meaning+=' '+DELTAS[key]
    basis=original_meanings.get(key)
    requirement=basis['requirement'] if basis else row.get('requirement',row.get('expected'))
    assert requirement,key
    conditions.append({'id':key,'requirement':requirement,
        'original_meaning_basis':basis or {'path':str((prior/'PREDECESSOR_IMPACT.json').relative_to(R)),'id':key,'file_sha256':sha(prior/'PREDECESSOR_IMPACT.json')},
        'predecessor_row':{'path':str((prior/'PREDECESSOR_IMPACT.json').relative_to(R)),'id':key,'file_sha256':sha(prior/'PREDECESSOR_IMPACT.json')},
        'current_selectors':inherited_selectors(row),'current_impact':meaning,'current_seams':seams,
        'receipts':suites,'public_receipts':sorted(set(name for seam in seams for name in direct[seam])),
        'disposition':'re-executed current selectors and explicit U05 impact; independent review pending'})
u04_meanings=[
 'Terminal result commits continue to collect employee outcomes once; the new live use is absent from the confirmed receipt.',
 'Reward choice/claim and copy grants retain their paired publication and stale-actor refusal.',
 'Retreat restores the pre-entry employee/fragment state and never carries a spent live use into the new deployment.',
 'Fresh process Continue restores confirmed/pre-entry roots; same actor reconnect separately retains paused accepted use.',
 'Before-first-savepoint and post-result Continue retain current assets; version11 carries strict employee base authoring.',
 'Independent SavePoint rewinds retain condition/load/copy owners; redeployment applies the selected form capacity and legal entry effects.',
 'Trauma/employee attempt/resource settlement remains independent of resonance admission and cancellation.',
 'Native failures during new pulse/use processing cannot overwrite either durable aggregate or the first fault.',
 'Prepared host publication remains paired and crash recoverable with the version11 payload.',
 'Claim idempotency/CAS and unrelated actor isolation remain unchanged by live use state.',
 'Current schema11 rejects10; conversion-only12 stays separate. Strict reader/writer and real-file original protection are rebound.',
 'Core remains the authoritative command/query owner; host projects the additive use and gain-block DTO through actual reconnect.',
 'Every public consumer and baseline hard guard is rerun; migration copies preserve original failed and historical evidence.',
 'Full current suites, predecessor conditions and exact reversible target restoration remain completion obligations.',
 'New use/queue fields and scans participate in clone/extent/work accounting; observer/fault/allocator guards retain their existing ceilings.',
]
for index,row in enumerate(oldcoverage['acceptances'],1):
    linked=[v for v in oldcoverage['validations'] if v['id'] in row['validation_refs']]
    selectors=sorted(set(s for v in linked for s in inherited_selectors(v)))
    conditions.append({'id':f'PE-CRRP-{index:02d}','policy_ref':row['Policy Element Revision Ref'],
        'requirement':row['Observable Policy Acceptance'],'predecessor_validation_refs':row['validation_refs'],
        'current_selectors':selectors,'current_impact':u04_meanings[index-1],'current_seams':['V13','V14'],
        'receipts':suites,'public_receipts':public+[fp+'SUMMARY.json'],
        'disposition':'current source execution; independent applicability review pending'})
assert len(conditions)==127 and len(old['r06_acceptances'])==31
impact={'source_sha256':current['sha256'],'predecessor_impact_sha256':sha(prior/'PREDECESSOR_IMPACT.json'),
        'predecessor_coverage_sha256':sha(prior/'COVERAGE_003.json'),'conditions':conditions,
        'original_requirement_sources':[{'path':str(p.relative_to(R)),'sha256':sha(p)} for p in REQUIREMENT_SOURCES],
        'r06_acceptances':copy.deepcopy(old['r06_acceptances']),'renamed_selectors':renames,
        'version_selector_migration':'current_v10_* -> current_v11_*','independent_audit':False}
for row in impact['r06_acceptances']:
    assert row['iva'] and all(any(c['id']==key for c in conditions) for key in row['iva'])
    row['status']='current mapped IVA selectors and public/PC oracles re-executed; independent review pending'
service=copy.deepcopy(json.loads((prior/'SERVICE_REUSE.json').read_text()))
for row in service['unchanged_service_inputs']:assert sha(R/row['path'])==row['sha256'],row['path']
for key in ('reused_audit','reused_service_evidence','reused_master_source_review'):assert sha(R/service[key])==service[key+'_sha256']
service.update(u05_predecessor_record_sha256=sha(prior/'SERVICE_REUSE.json'),
    changed_producer='Employee accepted-use, periodic charge and additive readiness flow through the existing Core/host observation boundary. All five fault modes, current host and heap checks rerun.',
    current_validation=suites+[fp+'SUMMARY.json',selection['continuation']],
    reuse_limit='Reuse only exact unchanged service plumbing; no deployed service or Unity claim.')
dimensions={'success':['V01','V02','V04','V05'],'invalid_input':['V07','V08','V10','V12'],
    'failure_rollback':['V04','V06','V09','V13','V14'],'retry_idempotency':['V04','V11','V13'],
    'ordering_concurrency':['V04','V05','V06','V13'],'boundary_empty_max':['V01','V03','V08','V12','V14'],
    'disconnect_restart_persistence':['V04','V11','V13'],'live_data':['V01','V08','V10','V12'],
    'producer_consumer':['V02','V04','V05','V12','V13'],'legacy_fallback_absence':['V01','V02','V10','V12','V13']}
result={'goal':'GOAL-CRSL-01','goal_set':'GS-CRSL-001','source_sha256':current['sha256'],
    'status':'GEX-10 self-verification complete; independent GEX-20 pending','frozen_policy_ref':freeze['ref'],
    'acceptances':acceptances,'validations':validations,'required_dimensions':dimensions,'suite_counts':counts,
    'receipt_bindings':bindings,'r05_summary':{'path':selection['r05_summary'],'sha256':sha(r05_path)},
    'pc_comparison':{'gameplay_summary_sha256':sha(pc_path),'accounted_cost_summary_sha256':sha(accounted_path),
        'raw_sha256':sha(pc_raw),'cost_claim':accounted['claim'],
        'strict_unchanged_cost_failure_retained':'evidence/compat_pc_immediate_002.json'},
    'compatibility_adaptation':{'path':selection['compatibility_adaptation'],'sha256':sha(adaptation_path)},
    'state_inventory':'STATE_INVENTORY.md','unchanged_observation_service_reuse':'SERVICE_REUSE.json',
    'limitations':['Actual local Linux Core, stock WebSocket and file process boundaries; no deployed service or Unity implementation claim.',
        'Summons are currently produced by independent boss runtime, not a public SkillEffectDef variant; preserved summon lifecycle tests remain inherited evidence, not a new employee summon API.',
        'Only the existing approved R05 concurrent-peak warning and inherited20minute stress exclusion apply; raw failures are retained.',
        'Independent completion verdict belongs solely to the implementation-nonparticipant audit.']}
write(S/'SERVICE_REUSE.json',service);write(S/'PREDECESSOR_IMPACT.json',impact);write(S/'COVERAGE.json',result)
print(json.dumps({'source':current['sha256'],'acceptances':14,'inherited':127,'r06':31,'counts':counts}))
