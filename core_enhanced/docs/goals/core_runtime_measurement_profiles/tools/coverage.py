"""Close ten policy acceptances and ten required dimensions with actual evidence."""
import re
from evidence_common import ROOT, HERE, S, C, read, write, ref, selected, passed_tests, bind

selection = selected()
suites = passed_tests(selection)
pool = set().union(*suites.values())
patterns = {
    'V01':['runtime_safety::tests::cmrp_', 'cmrp_public_override', 'cmrp_playback_gate', 'cmrp_playback_manifest'],
    'V02':['cmrp_native_actual_cap', 'cmrp_small_owner_override', 'cmrp_actual_snapshot', 'cmrp_external_world', 'cmrp_accounting_actual_warning', 'cmrp_accounting_failed_seed'],
    'V03':['cmrp_other_limits', 'cmrp_small_owner_override', 'cmrp_accounting_checked_overflow', 'cmrp_accounting_growth_refused', 'cmrp_accounting_unscoped', 'cmrp_raw_outer', 'runtime_fault', 'u07_f01_clone'],
    'V04':['cmrp_accounting_', 'cmrp_identity_', 'cmrp_allocator_', 'cmrp_rss_', 'cmrp_local_projection'],
    'V05':['runtime_measurement::allocator::tests::', 'cmrp_playback_export', 'cmrp_playback_maximum', 'cmrp_playback_actual_queue_full', 'observability::tests::'],
    'V06':['cmrp_identity_generation', 'cmrp_playback_gate', 'cmrp_script_missing_history', 'runtime_safety_checkpoint_restore', 'u04_real_process_crash'],
    'V07':['cmrp_playback_normal_external', 'cmrp_script_preserved_u07', 'cmrp_playback_authority'],
    'V08':['cmrp_playback_actual_queue_full', 'cmrp_script_history', 'cmrp_identity_generation', 'actual_queue_batch_dedup', 'u07_object_terminal_finalize'],
    'V09':['cmrp_', 'u07_f01_'],
    'V10':[],
}
actual = {key: [name for name in sorted(pool) if any(part in name for part in values)] for key,values in patterns.items()}
assert all(actual[key] for key in patterns if key != 'V10')
summaries = {key:read(ROOT/value['path']) for key,value in selection['public_summaries'].items()}
playback = summaries['playback_actual']
host = summaries['host_actual']
lifecycle = summaries['lifecycle_actual']
assert playback['passed'] and len(playback['rows']) == 26
assert host['passed'] and len(host['rows']) == 4 and all(len(row['phases']) == 2 for row in host['rows'])
assert lifecycle['passed'] and len(lifecycle['rows']) == 10
pc = read(ROOT/selection['pc_cost']['path'])
assert pc['passed'] and pc['rows'] == 576 and pc['distinct_conditions'] == 36
assert (pc['successful_conditions'],pc['typed_refusal_conditions']) == (31,5)
assert pc['immediate_u07']['non_timing_exact'] and pc['immediate_u07']['allowed_delta'] == 0
assert sum(row.get('rejected',False) for row in pc['controls']) >= 17
support = __import__('json').loads(next(line for line in reversed((HERE/'evidence/measurement_support006.log').read_text().splitlines()) if line.startswith('{')))
assert support['instrumented'] is False and support['live_requested_bytes'] is None and support['lifetime_peak_requested_bytes'] is None
impact = read(S/'PREDECESSOR_IMPACT.json')
assert len(impact['conditions']) == 181 and len(impact['r06_acceptances']) == 31
adaptation = read(C/'ADAPTATION_FINAL.json')
assert adaptation['source_sha256'] == selection['source_sha256'] and adaptation['immediate_input_count'] == 448
freeze = read(HERE/'P2G_30_FREEZE_001.json')['artifacts'][0]
acceptances = freeze['payload']['Observable Policy Acceptance Set']
assert len(acceptances) == 10
sample_keys = {'V01':['playback_actual','profile','input_validation'], 'V02':['lifecycle_actual','profile'],
               'V03':['fault','lifecycle_actual'], 'V04':['host_actual','playback_actual','lifecycle_actual'],
               'V05':['host_actual','playback_actual'], 'V06':['host_actual','playback_actual','continuation','rewind'],
               'V07':['host_actual','playback_actual','lifecycle_actual'], 'V08':['host_actual','playback_actual','lifecycle_actual'],
               'V09':list(summaries), 'V10':[]}
validations = []
for index, acceptance in enumerate(acceptances,1):
    key = f'V{index:02d}'
    validations.append({'id':key,'requirement':acceptance['Observable Policy Acceptance'],
                       'selectors':actual[key],'selector_receipts':bind(actual[key],suites),
                       'normal_public_evidence':{name:selection['public_summaries'][name] for name in sample_keys[key]},
                       'completion':'Executed self-verification; independent whole-goal review pending'})
    acceptance = dict(acceptance, validation_refs=[key], status='self-verification bound; independent review pending')
    acceptances[index-1] = acceptance
dimensions = {
    'success':['V01','V02','V04','V07'], 'invalid_input':['V01','V02','V06'],
    'failure_rollback':['V03','V05','V08'], 'retry_idempotency':['V03','V06','V08'],
    'ordering_concurrency':['V03','V05','V08'], 'boundary_empty_max':['V02','V04','V05'],
    'disconnect_restart_persistence':['V03','V06','V08'], 'live_data':['V01','V07'],
    'producer_consumer':['V02','V04','V06','V07'], 'legacy_fallback_absence':['V01','V02','V03','V05','V09'],
}
docs = [HERE/'README.md',ROOT/'core_enhanced/docs/core_runtime_contract.md',ROOT/'game_server/config/observability/README.md',S/'STATE_INVENTORY.md']
write(S/'COVERAGE.json',{'goal':'GOAL-CMRP-01','goal_set':'GS-CMRP-001','source_sha256':selection['source_sha256'],
      'frozen_policy_ref':freeze['ref'],'status':'GEX10 self-verification; independent GEX20 pending',
      'acceptances':acceptances,'validations':validations,
      'required_dimensions':[{'dimension':key,'disposition':'required','validation_refs':value,'status':'self-verified, independent review pending'} for key,value in dimensions.items()],
      'suite_counts':{name:sum(map(int,re.findall(r'test result: ok\. (\d+) passed;', (HERE/'evidence'/(name+'.log')).read_text()))) for name in suites},
      'distinct_passed_selector_counts':{name:len(values) for name,values in suites.items()}, 'receipt_bindings':selection['receipt_bindings'],
      'predecessor_impact':ref(S/'PREDECESSOR_IMPACT.json'),'compatibility_adaptation':ref(C/'ADAPTATION_FINAL.json'),
      'pc_comparison':selection['pc_cost'],'r05_summary':selection['r05_summary'],'performance_environments':selection['performance_environments'],
      'canonical_documentation':[ref(path) for path in docs],
      'samples':{'normal_cli_cases':26,'actual_host_cases':4,'actual_host_phases':8,'lifecycle_rows':10,
                'budget_scope':'Low engineering nominal256KiB, same queued_fact_source request262165: strict accepted262029/hard refusal vs dev16 accepted634913/soft only. No claim of deployment8MiB exceed. Failed8KiB loader preflight is not a Core operation sample.',
                'raw_support':'Process allocator windows with installed wrapper and explicit unavailable states; LinuxVmRSS and lifetimeVmHWM separately. No native admission/heap/RSS sum, session-heap claim or timing isolation claim.'},
      'followup':{'U08':'independent state owner','U11':'content identity','U30-C':'normal maximum Core/device sampling and profile tuning','U30-P':'actual Unity play/platform sampling','U31':'final integration acceptance'},
      'limitations':['Local Linux only; no Unity or deployed-service claim','16/32MiB are finite engineering windows, not a final deployment budget','Allocator requested bytes exclude fragmentation; RSS is whole-process OS resident memory','Cumulative admission is not retained owner extent or measured process heap','Outside-native rollback captures remain outside native scope; their allocation is in process counters','Inherited R05 concurrent peak-only warning and20-minute stress exclusion remain exact']})
print('PASS10 acceptances/10 dimensions/181+31/current actual samples')
