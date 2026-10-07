"""Bind each inherited condition to current successful selectors, preserving its full meaning."""
import copy,json,re
from run import ROOT,HERE,identity,sha
S=HERE/'execution/single_goal';P=ROOT/'core_enhanced/docs/goals/core_resonance_lifecycle/execution/single_goal'
selection=json.loads((S/'EVIDENCE_SELECTION.json').read_text());current=identity()
passed={}
for name in selection['suites']:
 r=json.loads((HERE/'evidence'/f'{name}.json').read_text())
 assert r['exit_code']==0 and r['source_unchanged'] and r['source_before']==current==r['source_after'],name
 assert r['log_sha256']==sha(HERE/'evidence'/f'{name}.log')
 passed[name]=set(re.findall(r'^test (\S+) \.\.\. ok$',(HERE/'evidence'/f'{name}.log').read_text(),re.M))
pool=set.union(*passed.values())
old=json.loads((P/'PREDECESSOR_IMPACT.json').read_text());coverage=json.loads((P/'COVERAGE.json').read_text())
GROUPS={
 'A08':('Basic action Damage remains default for autonomous actors; AllySingle and heal are explicit additions. Existing independent boss/projectile/source semantics remain.', ['V04','V05','V19']),
 'A09':('Corrosion remains at the accepted-use start owner; selection/settings gates do not replace the forced host clock or special death/withdraw/save rules.', ['V06','V16','V17','V18']),
 'A10':('New heal queue and selector use existing action flow and target credentials. Ordinary Defense no longer refuses a channel stance toggle; relocation remains locked by running actions. Existing flow release/settlement oracle remains.', ['V05','V07','V09','V19']),
 'A11':('CombatModifiers is an additional finite sustained profile; Instant-only heal overlay validation is added. Existing channel/aura/overlay source, lease, expiry and rollback requirements remain.', ['V04','V05','V06','V07','V19']),
 'A12':('New timed reduction and basic lifesteal use existing damage/actual HP-loss owners. Pure heal bypasses hostile procs. Rest and shield numerical contracts remain.', ['V05','V06','V08','V19']),
 'IVA':('Expression values, actual damage/error/trace oracles and selected source semantics remain. Explicit legacy sword/baseline fixtures preserve the original inputs where six new starter loadouts alter the normal defaults.', ['V04','V05','V06','V19']),
 'F':('Active loadout grant adds normal owned copies and bindings; legacy copy tests explicitly preserve originally empty active slots. Exact ownership, recipe/resource lifetime, failure and rollback rules remain.', ['V06','V18','V19']),
 'PE-CRSO':('Heal action storage, lifesteal observation source, forced timer queue, interaction and query vectors enter existing accounting. Numeric ceilings remain unchanged; current native, public, allocator and PC costs are separately reported.', ['V05','V10','V15','V18','V19']),
 'PE-CRFI':('New commands, query allocations and timer/heal production use existing candidate/sticky fault owners. Failed admission cannot publish partial TP, selection, movement or grant state; existing fault observers remain.', ['V06','V11','V15','V18','V19']),
 'PE-CRRP':('Normal starter/grant roots now include active copies; current durable checkpoint schema and unsupported-file protection remain. Added interaction is live actor state, not a new disk checkpoint.', ['V06','V16','V17','V18']),
 'X':('New typed action/selector/commands/state flow through established Core/host/authoring/clone boundaries. Re-execute exact original cross-feature selectors plus corresponding current public consumers.', ['V04','V05','V06','V15','V18','V19']),
}
def bound(selectors):
 assert set(selectors)<=pool, sorted(set(selectors)-pool)
 return {s:[n for n,v in passed.items() if s in v] for s in selectors}
def find(*fragments):
 result=set()
 for fragment in fragments:
  found={s for s in pool if fragment in s};assert found,fragment;result.update(found)
 return sorted(result)
observability_selectors={
 'PE-CRSO-01':find('original_defaults_and_no_file_profile','runtime_safety_host_file'),
 'PE-CRSO-02':find('settings_reject_schema','parser_is_bounded_before','structural_errors_identify','runtime_safety_host_file'),
 'PE-CRSO-03':find('raw_and_effective_identity','external_generation_keeps_semantics','public_content_parser_compiler'),
 'PE-CRSO-04':find('selected_native_limits_are_installed','first_public_battle_entry_uses_host_work_limit','prepared_battle_entry_drop_preserves'),
 'PE-CRSO-05':find('runtime_safety::tests::','first_public_battle_entry_uses_host_work_limit','runtime_observation_real_battle_preparation'),
 'PE-CRSO-06':find('compiler_limits_preserve','explicit_authoring_admission','public_content_parser_compiler','external_generation_keeps_semantics'),
 'PE-CRSO-07':find('history_import_and_deserialize_apply','a_short_history_span'),
 'PE-CRSO-08':find('application::runtime_observation_tests::','runtime_observation_captures_real_projectile_peak','runtime_observation_real_battle_preparation'),
 'PE-CRSO-09':find('actual_commit_and_full_or_absent_collector','stopped_scrapes_and_pinned_retirement','actual_queue_batch_dedup'),
 'PE-CRSO-10':find('selection_switch_ring_rollover','churn_registry_storage','a_contended_atomic_read','actual_queue_batch_dedup'),
 'PE-CRSO-11':find('application::runtime_observation_tests::','selection_switch_ring_rollover','actual_queue_batch_dedup','stopped_scrapes_and_pinned_retirement'),
 'PE-CRSO-12':find('runtime_observation_real_battle_preparation','application::runtime_observation_tests::'),
}
fault_receipts=[n for n in selection['public_receipts'] if n.startswith('final_fault_')]
all_receipts=selection['suites']+selection['checks']+selection['public_receipts']+[selection['r05_receipt']]
for name in all_receipts:
 r=json.loads((HERE/'evidence'/f'{name}.json').read_text())
 assert r['source_before']==current==r['source_after'] and r['source_unchanged'] and r['exit_code'] in ((0,101) if name==selection['r05_receipt'] else (0,)),name
 assert r['log_sha256']==sha(HERE/'evidence'/f'{name}.log')
def direct(key,group):
 if key in ('X09','X10','PE-CRSO-12'):return all_receipts
 if group=='PE-CRSO':
  number=int(key.rsplit('-',1)[1])
  return ['compat_public_profile_001','compat_default_parity_001','compat_public_api_001','compat_prepared_public_001'] if number<=7 else fault_receipts+['integrated_heap008']
 if group=='PE-CRFI':return fault_receipts
 if group=='PE-CRRP':return ['public_continuation_001','public_rewind_001','compat_prepared_public_001']
 if group in ('IVA','F'):return ['compat_public_api_001','compat_public_cli_001','compat_pc_gameplay_001','compat_pc_cost_002','public_guards001','public_guards_allocator001']
 return ['compat_public_cli_001','compat_public_oracle_001','compat_public_trace_001','public_schema_001','public_independent_area_001']
conditions=[]
for row in old['conditions']:
 key=row['id'];group=next(g for g in GROUPS if key.startswith(g))
 impact,seams=GROUPS[group];selectors=observability_selectors.get(key,row['current_selectors'])
 assert selectors or key in ('X09','X10'),key
 entry={'id':key,'requirement':row['requirement'],'original_meaning_basis':row.get('original_meaning_basis'),'predecessor_row':{'path':str((P/'PREDECESSOR_IMPACT.json').relative_to(ROOT)),'id':key,'sha256':sha(P/'PREDECESSOR_IMPACT.json')},'predecessor_semantic_adjustment':row.get('current_impact'),'current_impact':impact,'current_seams':seams,'current_selectors':selectors,'selector_receipts':bound(selectors),'direct_receipts':direct(key,group),'disposition':'affected consumer revalidated with unchanged original meaning except the explicit existing and U06 policy deltas; independent review pending'}
 if key in ('PE-CRSO-09','PE-CRSO-10'):entry['unchanged_service_evidence']='SERVICE_REUSE.json'
 if key=='X10':entry['final_artifact_refs']=['target/u06_001/MANIFEST.json','target/u06_001/MEMBERSHIP.json','target/u06_001/RESTORE_VERIFICATION.json','STATE_INVENTORY.md']
 conditions.append(entry)
u05_impacts=[
 'Selected form capacity100 on new six training copies obeys the same floor(max/5) and normal readiness gates.',
 'Work exit and forced1500ms transitions update eligibility through the existing global pulse; original charge/time numbers are preserved.',
 'Timed CombatModifiers and buffs use explicit ActionAndEffects references; unrelated independent carriers retain their own lifetime.',
 'New healing/manual targets and selection menus preserve one accepted charge payment and typed failure rollback.',
 'New CombatModifiers source snapshots and Heal Instant overlay consumption close the actual finite use owner without retaining it for unrelated delivery.',
 'Ordinary stance toggle preserves running channel/action; actual HardCC/withdraw/death still apply prior use cancellation rules.',
 'AllySingle uses fixed mandatory direct target validation; no new implicit target retargeting.',
 'Manual area heal preserves legal empty PlayerTile acceptance and its accepted anchor semantics.',
 'General/deployment/skill selection now share one playback restore owner, while existing skill target preview and cost rollback remain.',
 'Ally area uses accepted tile anchor through the same delivery owner; unit movement cannot retarget accepted areas.',
 'Settings/selection gate ordinary input while explicit corrosion forced host clock and terminal outcomes remain.',
 'Six new skills/fragments explicitly author use lifetime and capacity; strict loader/CLI preserve publication rejection boundaries.',
 'New normal copies persist in existing owned roots; live focus/settings/forced timer/heal readiness use clone and same-actor reconnect only.',
 'New action/timer/interaction/source retained sizes and clone visits are accounted; original PC and R05 numeric gates remain separate.',
]
for i,row in enumerate(coverage['acceptances']):
 linked=[v for v in coverage['validations'] if v['id'] in row['validation_refs']]
 selectors=sorted({s for v in linked for k in ('core_selectors','host_selectors') for s in v[k]})
 conditions.append({'id':f'PE-CRSL-{i+1:02d}','requirement':row['Observable Policy Acceptance'],'policy_ref':row['Policy Element Revision Ref'],'predecessor_row':{'path':str((P/'COVERAGE.json').relative_to(ROOT)),'validation_refs':row['validation_refs'],'sha256':sha(P/'COVERAGE.json')},'current_impact':u05_impacts[i],'current_selectors':selectors,'selector_receipts':bound(selectors),'disposition':'current source nonzero execution; independent review pending'})
assert len(conditions)==141
r06=copy.deepcopy(old['r06_acceptances']);assert len(r06)==31
for row in r06:
 assert row['iva'] and all(any(c['id']==key for c in conditions) for key in row['iva'])
 row['status']='Original exact requirement retained; current per-condition selector receipts above. Public and PC receipts are in COVERAGE, independent review pending.'
result={'source_sha256':current['sha256'],'predecessor_impact_sha256':sha(P/'PREDECESSOR_IMPACT.json'),'predecessor_coverage_sha256':sha(P/'COVERAGE.json'),'conditions':conditions,'r06_acceptances':r06,'original_requirement_sources':old['original_requirement_sources'],'independent_audit':False}
with (S/'PREDECESSOR_IMPACT.json').open('x') as f:json.dump(result,f,ensure_ascii=False,indent=2);f.write('\n')
print('Bound141 inherited conditions and31 separate R06 requirements')
