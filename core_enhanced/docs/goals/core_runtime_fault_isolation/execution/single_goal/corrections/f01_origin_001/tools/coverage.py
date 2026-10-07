import sys
sys.path.insert(0, '/mnt/f/work/simulator/core_enhanced/docs/goals/core_runtime_fault_isolation/tools')
"""Bind frozen U03 acceptances and every inherited condition to current executions."""
import copy,hashlib,json,re
from pathlib import Path
import run
R,H=run.ROOT,run.HERE
E=H/'evidence';S=Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_runtime_fault_isolation/execution/single_goal/corrections/f01_origin_001')
current=run.identity()
sha=run.sha
suites=['f01_final_core_default002','f01_final_core_fixtures002','f01_final_core_observation002','f01_final_host_default002','f01_final_host_observation002']
passed={};counts={};bindings={}
def receipt(name, allowed=(0,)):
    p=E/(name+'.json');s=json.loads(p.read_text())
    assert s['exit_code'] in allowed and s['source_unchanged'],name
    assert s['source_after']==current,name
    assert s['log_sha256']==sha(E/(name+'.log')),name
    bindings[name]={'receipt_sha256':sha(p),'log_sha256':s['log_sha256'],'source_sha256':current['sha256'],'raw_exit_code':s['exit_code']}
    return s
for name in suites:
    receipt(name);t=(E/(name+'.log')).read_text()
    passed[name]=set(re.findall(r'^test (\S+) \.\.\. ok$',t,re.M))
    totals=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',t)
    counts[name]={key:sum(int(row[i]) for row in totals) for i,key in enumerate(['passed','failed','ignored'])}
    assert counts[name]['passed']>0 and counts[name]['failed']==0
core=passed[suites[0]] & passed[suites[1]] & passed[suites[2]]
host=passed[suites[3]] & passed[suites[4]]
observed=passed[suites[2]]|passed[suites[4]]
def select(fragments,pool):
    out=set()
    for fragment in fragments:
        matches={x for x in pool if fragment in x};assert matches,fragment;out.update(matches)
    return sorted(out)
contracts={
 'V01':(['native_fault_preserves_last_root','trusted_reconciliation_origins','trusted_boss_activation_failure'],[],[]),
 'V02':(['trusted_reconciliation_fault_at_manual','ordinary_missing_employee','ordinary_invalid_targeting_action','actual_dynamic_expression_failure','normal_refusals_queries','typed_normal_resource','actual_revision_overflow'],['first_public_battle_entry'],[]),
 'V03':(['trusted_reconciliation_origins','trusted_boss_activation_failure','interrupted_diagnostic','actual_revision_overflow'],[],[]),
 'V04':(['native_fault_preserves_last_root','interrupted_diagnostic'],['real_tick_fault_blocks'],[]),
 'V05':([],['real_tick_fault_blocks','real_manual_retreat_failure'],[]),
 'V06':([],['real_tick_fault_blocks'],[]),
 'V07':([],['real_tick_fault_blocks','fault_is_per_actor','first_public_battle_entry'],[]),
 'V08':([],['real_tick_fault_blocks','new_owner_continue_keeps'],[]),
 'V09':([],['fault_is_per_actor'],[]),
 'V10':([],[],['trusted_origin_fault_ends_observation','terminal_observation_keeps_failure','selection_switch_ring_rollover','actual_queue_batch_dedup']),
 'V11':(['authoring::trial::tests::','persistence_rollback_tests::'],['first_public_battle_entry'],[]),
 'V12':([],[],[]),
}
validations=[{'id':k,'core_selectors':select(c,core),'host_selectors':select(h,host),'observed_selectors':select(o,observed),'suite_receipts':suites,'status':'current self-verification; independent semantic audit pending'} for k,(c,h,o) in contracts.items()]
direct_seams={
 'V01':['final_fault_f01_001_none','final_fault_f01_001_off'],
 'V02':['f01_compat_public_boundary_001','f01_compat_public_api_001'],
 'V03':[], 'V04':['final_fault_f01_001_websocket'],
 'V05':['final_fault_f01_001_websocket'], 'V06':['final_fault_f01_001_websocket'],
 'V07':['final_fault_f01_001_websocket','f01_compat_prepared_public_001'],
 'V08':['final_fault_f01_001_websocket','f01_compat_public_profile_001'],
 'V09':[],
 'V10':['final_fault_f01_001_none','final_fault_f01_001_detached','final_fault_f01_001_queue_full','final_fault_f01_001_on','final_fault_f01_001_off','f01_final_heap002'],
 'V11':['f01_compat_prepared_public_001','f01_compat_prepared_boundary_001','f01_compat_public_boundary_001','f01_compat_public_api_001','f01_compat_public_oracle_001','f01_compat_public_trace_001'],
 'V12':['f01_compat_default_parity_001','f01_compat_public_cli_001','f01_compat_public_profile_001','f01_compat_pc_compare_001','f01_compat_pc_immediate_001','f01_compat_r05_compare_001'],
}
for row in validations:row['direct_receipts']=direct_seams[row['id']]
for name in ['fmt','check','event_log','clippy_core','clippy_host','heap','release']:receipt('f01_final_'+name+'002')
compat_names=['prepared_public','prepared_boundary','observed_release','default_parity','public_oracle','public_trace','public_cli','public_profile','public_boundary','public_api','pc_prepare','pc','pc_compare','pc_immediate','r05','r05_compare']
for name in compat_names:receipt('f01_compat_'+name+'_001',(0,101) if name=='r05' else (0,))
for name in ['build_on','none','detached','queue_full','on','websocket','build_off','off']:receipt('final_fault_f01_001_'+name)
fault_summary=json.loads((E/'final_fault_f01_001_SUMMARY.json').read_text());assert fault_summary['passed'] and fault_summary['source']==current
C=S/'compatibility/current_001/evidence'
for name in ['PC_IMMEDIATE_PARITY.json','PC_SUMMARY_CURRENT_002.json','R05_SUMMARY_CURRENT_002.json']:
    assert (C/name).is_file(),name
freeze=json.loads((H/'P2G_30_FREEZE_003.json').read_text())['artifacts'][0]
acceptances=copy.deepcopy(freeze['payload']['Observable Policy Acceptance Set'])
pe_seams={1:['V01','V02'],2:['V01','V04','V05','V10'],3:['V01','V02','V03','V07','V10','V11'],4:['V02','V03'],5:['V03','V04','V06'],6:['V01','V04','V05','V07','V11'],7:['V05','V06'],8:['V07','V08','V09'],9:['V08'],10:['V09'],11:['V10'],12:['V02','V07','V11','V12']}
for i,row in enumerate(acceptances,1):row.update(validation_refs=pe_seams[i],status='self-verified; independent audit pending')
dimensions={'success':['V01','V05','V08','V09'],'invalid_input':['V02','V04','V08'],'failure_rollback':['V01','V02','V03','V07'],'retry_idempotency':['V01','V05','V07'],'ordering_concurrency':['V05','V06','V09'],'boundary_empty_max':['V01','V02','V04','V10'],'disconnect_restart_persistence':['V06','V07','V08'],'live_data':['V01','V02','V12'],'producer_consumer':['V04','V05','V06','V07','V08','V09','V10'],'legacy_fallback_absence':['V06','V07','V11','V12']}
prior=R/'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/corrections/f01_action_flow_20261004_001/COVERAGE_CURRENT.json'
inherited=copy.deepcopy(json.loads(prior.read_text()))
conditions=inherited['conditions'];assert len(conditions)==88
sensitive={'A08-04','A08-07','A09-07','A09-09','A09-10','A10-08','A11-06','A11-09','A12-10','X05','X06','X07','X08','X09','X10','IVA-CAVER-14','IVA-CAVER-15','IVA-CAVER-17','IVA-CAVER-21','F01','F09','F10','F12'}
for row in conditions:
    assert set(row['selectors'])<=core,row['id']
    row['predecessor_receipts']=row.pop('receipts');row['receipts']=suites[:3]
    row['public_owner_impact']=row['id'] in sensitive
    row['current_semantic_boundary']='The private BattleCore/GameCore transactional owner and Trial candidate retry are retained; trusted event/boss/reconciliation origin now produces typed invariant faults, with ordinary target refusals unchanged; only a published CoreApplication retains first runtime fault outside rollback. Ordinary command refusals, accepted parent, read-only queries and successful commits retain prior meaning.'
    row['current_fault_seams']=['V01','V02','V03','V11'] if row['id'] in sensitive else ['V02','V11','V12']
    row['disposition']='exact inherited selectors rerun in all three current Core suites; independent semantic applicability pending'
u02=R/'core_enhanced/docs/goals/core_runtime_safety_observability'
u02coverage=json.loads((u02/'execution/s2/COVERAGE.json').read_text())
for validation in u02coverage['validations']:
    assert set(validation.get('core_selectors',[]))<=passed[suites[2]],validation['id']
    assert set(validation.get('host_selectors',[]))<=passed[suites[4]],validation['id']
for validation in u02coverage['validations']:
    validation['predecessor_receipts']=validation.pop('receipts',[])
    validation['predecessor_suite_receipts']=validation.get('suite_receipts',[])
    validation['suite_receipts']=suites
    validation['predecessor_direct_receipts']=validation.get('direct_receipts',[])
    validation['direct_receipts']=[name.replace('s2compat_', 'f01_compat_') for name in validation.get('direct_receipts',[])]
    validation['current_changed_lifetime_evidence']=['f01_final_core_observation002','f01_final_host_observation002','final_fault_f01_001_SUMMARY.json','SERVICE_REUSE.json']
    validation['status']='current selectors rebound to exact U03 source; predecessor service evidence separately reused by verified input boundary'
u02_impacts=[
 'The 38 defaults, profile loader and safety/compile settings are byte-identical. Current public default/profile/CLI results establish normal behavior; only actual published runtime failure gains terminal lifetime.',
 'All strict profile schema/size/depth/duplicate/range producers are unchanged. Current negatives still reject before content publication; no fault capsule exists in an uncreated battle.',
 'Raw/effective identity, compile cache and content generation sources are unchanged. Fault state is stored on CoreApplication outside generation/hash/save; profile tests rerun and two-player test confirms isolation.',
 'First spawn and current-host restore native budgets remain unchanged. Preparation before publication retains retry; published active battle now gates all capture/restore/Continue/export ports. Current first-entry and new-owner Continue tests cover both sides.',
 'API/CLI/private Trial cancellation and admission sources remain unchanged. All current public probes rerun; published runtime fault terminal semantics are exercised through Core and stock WebSocket. Normal authority and caller-invalid skill refusals preserve the old wire.',
 'Expression parser/compiler/evaluator and read-only preview implementations are unchanged; committed reinput child absence is explicitly internal, and real divide-by-zero first cause is retained. A real legal dynamic divide-by-zero in a published mutation now terminates the application battle; query and private speculative failures retain their existing lifetimes.',
 'History storage/import/eviction/cursor and span semantics are retained; dispatch wrapper preserves normal handler outcomes and makes propagated internal shared errors typed. Terminal clear only removes observational current battle; retained root/history/first-failure summary remain. Current inherited history selectors and new fault root checks cover this boundary.',
 'Native attempt accounting is retained; trusted reconciliation/event/boss invariant producers now use typed native rejection. runtime_error latches once before diagnostic conversion, then observer end clears current without emitting a commit. Later gated calls do not add attempt work or repeat first rejection.',
 'Exporter, worker, HTTP API, dashboard, scrape config and dependencies are byte-identical and reuse the audited actual service evidence. Current five-mode real-fault parity and isolated allocator rerun cover the changed producer-to-worker lifetime.',
 'Registry/ring/histogram bounds and selection worker are unchanged. Actual selected battle fault makes current empty and selected session none while retaining the owner lease/last attempt; full queue loses observation events only.',
 'The new atomic end_battle clear is a source change and is verified directly by current Core, collector and five-mode probes. No fake successful commit, repeated attempted work or stale selected battle remains. Existing dedup/retirement tests rerun.',
 'Current full Core/host matrices, static checks, public compatibility, PC original/immediate parity and R05 raw/approved classification are bound to the final source. Original 88 conditions and 31 R06 acceptances are retained individually.'
]
u02acceptances=json.loads((u02/'execution/master/COVERAGE.json').read_text())['acceptances']
for i,row in enumerate(u02acceptances,1):
    conditions.append({'id':f'PE-CRSO-{i:02d}','policy_ref':row['policy_ref'],'requirement':row['requirement'],'predecessor_review':row['independent_judgment'],'current_impact':u02_impacts[i-1],'current_seams':['V01','V02','V07','V10','V11','V12'],'evidence_refs':suites+['f01_compat_public_profile_001','f01_compat_public_api_001','f01_compat_prepared_public_001','final_fault_f01_001_SUMMARY.json'],'disposition':'current full suites and public consumer evidence; independent U03 semantic applicability pending'})
assert len(conditions)==100 and len(inherited['r06_acceptances'])==31
for row in inherited['r06_acceptances']:
    assert row['iva'] and all(any(c['id']==key for c in conditions) for key in row['iva']),row
    row['status']='original acceptance unchanged; mapped IVA selectors rerun on current U03 source; private retry retained, published actual runtime failure uses V01–03 terminal lifetime; independent review pending'
impact={'source_sha256':current['sha256'],'predecessor_f01_coverage_sha256':sha(prior),'predecessor_u02_coverage_sha256':sha(u02/'execution/master/COVERAGE.json'),'conditions':conditions,'r06_acceptances':inherited['r06_acceptances'],'u02_validation_selector_binding':u02coverage['validations'],'f01_correction_selectors':inherited['correction_selectors'],'independent_audit':False}
assert set(impact['f01_correction_selectors'])<=core
impact['u03_f01_origin_correction'] = {
    'closed_predecessor_finding': 'post_audit/u03_20261004_001/FINDINGS.json#F01',
    'producer_review': 'PRODUCER_REVIEW.md',
    'actual_public_selectors': select(['trusted_reconciliation_origins', 'trusted_reconciliation_fault_at_manual', 'trusted_boss_activation_failure', 'ordinary_missing_employee', 'ordinary_invalid_targeting_action'], core),
    'observer_selectors': select(['trusted_origin_fault_ends_observation'], observed),
    'private_retry_selectors': select(['advance_rolls_back_core_and_world_when_employee_reconcile_fails', 'tactical_world_late_employee_reconciliation_failure_is_atomic', 'incoherent_parallel_context_is_rejected_before_application_staging', 'microstep_and_activation_overflow_leave_the_complete_core_unchanged'], core),
    'expectation_migration': 'Four existing private tests now require typed invariant/site at the same actual failure producer; unchanged complete-root rollback assertions retained. Raw GameCore retry still executes and rejects on the same actual invariant.',
    'implementation_independent_verdict': False,
}
result={'goal':'GOAL-CRFI-01','goal_set':'GS-CRFI-001','source_sha256':current['sha256'],'status':'GEX-10 self-verification complete; separate nonparticipant audit pending','frozen_policy_ref':freeze['ref'],'acceptances':acceptances,'validations':validations,'required_dimensions':dimensions,'suite_counts':counts,'receipt_bindings':bindings,'public_fault_summary':'../../../../evidence/final_fault_f01_001_SUMMARY.json','compatibility':'compatibility/current_001/ADAPTATION.json','unchanged_observation_service_reuse':'SERVICE_REUSE.json','limitations':['No Unity UI or remote deployment claim; actual local stock websocket server and public release Core/CLI exercised.','R05 raw exit and original hard guards are preserved; only the previously approved concurrent-peak warning may be classified separately.','20 minute test and OBS001 excluded as inherited; no new arbitrary production cap.','Independent completion verdict is not issued by this implementation.']}
for name,value in [('COVERAGE.json',result),('PREDECESSOR_IMPACT.json',impact)]:
    with (S/name).open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2)
print(json.dumps({'acceptances':len(acceptances),'seams':len(validations),'conditions':len(conditions),'r06':len(inherited['r06_acceptances']),'counts':counts}))
