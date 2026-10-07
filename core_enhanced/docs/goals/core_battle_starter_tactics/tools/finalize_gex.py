"""Publish GEX-10 only from complete, current-source self-verification evidence."""
import datetime,json
from run import ROOT,HERE,identity,sha
S=HERE/'execution/single_goal'
prior=ROOT/'core_enhanced/docs/goals/core_resonance_lifecycle/execution/single_goal/SERVICE_REUSE.json'
coverage=json.loads((S/'COVERAGE.json').read_text())
selection=json.loads((S/'EVIDENCE_SELECTION.json').read_text())
source=identity()['sha256'];assert coverage['source_sha256']==source
def write(name,value):
 with (S/name).open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n')
service=json.loads(prior.read_text())
for row in service['unchanged_service_inputs']:assert sha(ROOT/row['path'])==row['sha256'],row['path']
for key in ['reused_audit','reused_service_evidence','reused_master_source_review']:assert sha(ROOT/service[key])==service[key+'_sha256']
service.update(source_sha256=source,u06_predecessor_record_sha256=sha(prior),changed_producer='Pure basic healing, starter active skills, forced stance timers, swept blocking and shared tactical focus/settings time ownership use the existing Core/host observation boundary. Current host, heap, five fault modes and public recovery evidence is bound below.',current_validation=selection['suites']+selection['checks']+selection['public_receipts'])
write('SERVICE_REUSE.json',service)
C=S/'compatibility/current_001/evidence'
r05=json.loads((C/selection['r05_summary']).read_text())
fault=json.loads((HERE/selection['public_summaries']['fault']).read_text())
assert fault['passed'] and fault['source']['sha256']==source
result={'stage':'GEX-10','goal':'GOAL-CBST-01','goal_set':'GS-CBST-001','phase':'PHASE-CBST-01','status':'implementation and self-verification complete; independent GEX-20 pending','recorded_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_sha256':source,'frozen_policy':'../../P2G_30_FREEZE_001.json','phase_plan':'../../P2G_40_PHASE_PLAN_001.json','handoff':'../../P2G_60_HANDOFF_001.json','coverage':'COVERAGE.json','predecessor_impact':'PREDECESSOR_IMPACT.json','state_inventory':'STATE_INVENTORY.md','canonical_contract':'../../../../core_runtime_contract.md','canonical_rulebook':'../../../../game_rulebook.md','canonical_gameplay_detail':'../../../../game_rules/combat.md','suite_counts':coverage['suite_counts'],'frozen_acceptance_count':20,'validation_seams':20,'dimensions':10,'inherited_conditions':141,'r06_acceptances':31,'raw_r05_exit':r05['raw_exit_code'],'r05_classification':r05['classification'],'r05_concurrent_peak_excess_bytes':r05['concurrent_peak_excess_bytes'],'r05_other_hard_guard_failures':r05['other_hard_guard_failures'],'public_fault_modes':fault['mode_result_exact_equality'],'public_summaries':selection['public_summaries'],'compatibility_adaptation':'compatibility/current_001/ADAPTATION_FINAL.json','next':'Freeze exact bytes and reversible restoration; fresh implementation-nonparticipant GPT-6 Astra / High GEX-20. U07 remains ineligible until Complete / High.','self_awarded_independent_verdict':False}
write('GEX_10_RESULT.json',result)
print(json.dumps({'source':source,'status':result['status']}))
