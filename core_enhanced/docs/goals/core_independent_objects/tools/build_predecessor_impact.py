import sys
from finalization_paths import paths
_, final_names = paths(sys.argv[1] if len(sys.argv) > 1 else '001')
"""Retain every predecessor requirement and bind it to current executions."""
import copy,json,re
from run import ROOT,HERE,identity,sha
S=HERE/'execution/single_goal';P=ROOT/'core_enhanced/docs/goals/core_battle_starter_tactics/execution/single_goal'
selection=json.loads((S/final_names['selection']).read_text());current=identity();passed={}
all_receipts=selection['suites']+selection['checks']+selection['public_receipts']+[selection['r05_receipt']]
for name in all_receipts:
 path=HERE/'evidence'/f'{name}.json';r=json.loads(path.read_text());log=path.with_suffix('.log')
 assert r['exit_code'] in ((0,101) if name==selection['r05_receipt'] else (0,)) and r['source_unchanged'] and r['source_before']==current==r['source_after'],name
 assert r['log_sha256']==sha(log),name
 if name in selection['suites']:passed[name]=set(re.findall(r'^test (\S+) \.\.\. ok$',log.read_text(),re.M))
pool=set.union(*passed.values())
def bound(selectors):
 assert set(selectors)<=pool,sorted(set(selectors)-pool)
 return {s:[n for n,v in passed.items() if s in v] for s in selectors}
old=json.loads((P/'PREDECESSOR_IMPACT.json').read_text());coverage=json.loads((P/'COVERAGE.json').read_text())
GROUPS={
 'A08':('Generic actors use their actual side and own action authority; traps retain a finite creation snapshot and walls use an explicit noncombatant role. Existing summoned boss/projectile contracts remain.', ['V01','V04','V08']),
 'A09':('Independent objects cannot become employees or corrosion subjects; creator withdrawal preserves only explicitly Persist objects. Existing corrosion cancellation, item and death semantics remain.', ['V01','V08','V13']),
 'A10':('Creation and evolution use the shared Spawn Kernel transaction and stable occurrence ledger. Initial CreateObject and SegmentedShield authoring excludes flow/repeat/reinput capabilities; existing flow settlement is re-executed.', ['V02','V03','V07','V14']),
 'A11':('SegmentedShield uses the existing sustained remaining-health owner; its standalone finite source releases parent action/use locks. Ordinary sustained/channel/area/overlay lifetimes remain, and evolution re-composes unrelated modifiers.', ['V05','V07','V10','V13']),
 'A12':('Trap damage uses real damage transactions and succeeds under full shield absorption. Wall destruction suppresses unit death procs; nonlethal HP cost bypasses damage mitigation. Existing reduction/lifesteal/rest/shield oracles remain.', ['V06','V10','V11','V12','V13']),
 'IVA':('New object source descriptors do not alter existing expression evaluation, error, trace or source selection. Original nonemployee workload resonance is preserved only in the isolated benchmark fixture.', ['V02','V16','V19']),
 'F':('Official fragment forms retain normal owned-copy/equipment authority. Same-copy evolution and exact equipped-weapon pool selection add consumers without changing recipe/reward ownership; no future infection producer is claimed.', ['V07','V11','V15','V18']),
 'PE-CRSO':('Object maps, counters, snapshots, epochs, queued callbacks, pool descriptors and projections enter existing work/retained/clone admission. Current numeric guards and all raw PC costs are bound without cap increases.', ['V02','V03','V16','V19']),
 'PE-CRFI':('Object create/evolve/cost runs inside the existing candidate transaction. Late retained/queue refusal preserves IDs, HP, pools, counters, ledger and source state. Existing sticky fault and observer modes remain.', ['V02','V07','V11','V19']),
 'PE-CRRP':('Object and segmented pool state is live battle state: clone and same-actor reconnect preserve it; pre-entry Continue discards the active battle through the existing persistence boundary. No live battle disk codec is added.', ['V09','V17']),
 'PE-CRSL':('Object creation releases the accepted skill action normally; independent sources and segmented storage remove parent use credentials. Form/copy resonance, preview refusal and one accepted payment keep the established lifecycle owner.', ['V07','V08','V10','V14','V15']),
 'X':('New schema, owner, DTO and public ports pass current integrated suites and exact predecessor selectors. Source/data/contract changes and restorable bytes are included in the final target.', ['V01','V15','V16','V17','V18','V19']),
}
fault=[n for n in selection['public_receipts'] if n.startswith('final_fault_')]
def historical_direct(key,group):
 if key in ('X09','X10','PE-CRSO-12'):return all_receipts
 if group=='PE-CRSO':return ['compat_public_profile_001','compat_default_parity_001','compat_public_api_001','compat_prepared_public_001','public_guards001','public_guards_allocator001']+fault+[next(n for n in selection['checks'] if n.startswith('integrated_heap'))]
 if group=='PE-CRFI':return fault+['public_guards001','public_guards_allocator001']
 if group=='PE-CRRP':return ['public_continuation_001','public_rewind_001','compat_prepared_public_001',selection['object_receipt']]
 if group in ('IVA','F'):return ['compat_public_api_001','compat_public_cli_001','compat_pc_gameplay_001',next(n for n in selection['checks'] if n.startswith('compat_pc_cost')),'public_guards001','public_guards_allocator001']
 return ['compat_public_cli_001','compat_public_oracle_001','compat_public_trace_001','public_schema_001','public_independent_area_001',selection['object_receipt']]
# Resolve preserved semantic owner names against actual selected fresh labels.
receipt_by_stem={re.sub(r'\d+$','',name):name for name in all_receipts}
def direct(key,group):
 return [receipt_by_stem[re.sub(r'\d+$','',name)] for name in historical_direct(key,group)]
conditions=[]
for row in old['conditions']:
 key=row['id'];group=next(g for g in GROUPS if key.startswith(g));impact,seams=GROUPS[group];selectors=row['current_selectors'];assert selectors or key in ('X09','X10'),key
 entry={'id':key,'requirement':row['requirement'],'original_meaning_basis':row.get('original_meaning_basis'),'predecessor_row':{'path':str((P/'PREDECESSOR_IMPACT.json').relative_to(ROOT)),'id':key,'sha256':sha(P/'PREDECESSOR_IMPACT.json')},'predecessor_semantic_adjustment':row.get('current_impact'),'current_impact':impact,'current_seams':seams,'current_selectors':selectors,'selector_receipts':bound(selectors),'direct_receipts':direct(key,group),'disposition':'Original requirement retained; affected consumer re-executed on current source, with only explicit U07 policy additions. Independent review pending.'}
 if key in ('PE-CRSO-09','PE-CRSO-10'):entry['unchanged_service_evidence']=final_names['service']
 if key=='X10':entry['final_artifact_refs']=[f"target/{selection['target']}/{name}.json" for name in ('MANIFEST', 'MEMBERSHIP', 'RESTORE_VERIFICATION')] + ['STATE_INVENTORY.md']
 conditions.append(entry)
impacts=['Object creation uses skill/cost transactions without charging deployment TP or changing battle-second TP recovery. Original deployment/cost/terminal selectors are re-executed.', 'Created objects do not consume employee deployment slots or weapon eligibility. Shared tile/domain occupancy rejects invalid objects while original eight-employee and condition-precedence oracles remain.', 'Creator withdrawal preserves only Persist objects; Despawn cleans object chains without unit death procs. Existing fixed redeployment HP, released projectile and unreleased-use cancellation remain.', 'Actor basic profiles reuse existing authored weapon/basic contracts; walls have explicit noncombatant adapters. Pure-heal Staff and special-gun profiles retain original serialization/default behavior.', 'Wall Heal and segmented RechargeShield refuse unsupported repair; ordinary actor Heal uses existing capability checks. Original pure-heal ordering, target selection, CC/Silence and no-damage-proc selectors remain.', 'All six training sets and normal copy grants stay unchanged. New official Laetitia active forms are additional consumers; standalone wall skills do not invent a grant/reward policy.', 'Ordinary employee Defense still preserves already started actions. Walls and autonomous actors do not gain employee stances; source HardCC cannot retroactively cancel independent released owners.', 'Employee Rest and hard-CC contribution rules remain. Segmented block bonuses use the existing effective capacity owner, and object HP retirement adds no rest/recovery behavior.', 'Employee relocation continues through existing eligibility and state-preserving movement. Explicit actor Path capability has its own original plan; fixed traps/walls cannot relocate.', 'Existing forced Defense timers retain epoch/lifetime checks. Object TTL/evolution adds separate ID+epoch callbacks and cannot revive employee transition timers.', 'Existing simultaneous swap retains one TP payment and rollback. Object placement uses the same domain/occupancy truth, without preview reserving an object tile.', 'Created objects cannot become employee work contributors or facility objectives. Terminal cleanup removes live owners while retaining bounded diagnostic/replay state until reset.', 'Opposite-side walls add swept actual-segment contact to the existing block relation owner. Segmented capacity decreases release excess relations; existing employee positions/capacity/air rules remain.', 'Wall basic attacks are eligible only through actual blocking contact. Trap/skill area targeting does not gain the employee blocked-basic range exception; wall Heal is separately prohibited.', 'Object/pool previews and live queries expose predicted cost, source and capability through normal command authority; repeated preview does not mutate revision/HP/occupancy.', 'Object target focus uses the same selection/playback owner. Source removal or terminal cleanup cannot resume paused battles or overwrite chosen speed.', 'Paused object casts accept cost/use once and defer release until resumed. Same-actor reconnect preserves pending and released objects without advancing paused TTL.', 'Clone and reconnect preserve object/pool live state; disk Continue restores the existing pre-entry roots, with no new live battle codec or unsupported-file bypass.', 'Current strict loader, normal World/trial, host and release CLI cover object consumers alongside all six legacy sets. New storage/accounting is measured with unchanged original numeric caps.', 'Predecessor Complete/High audit stays immutable. All 161 inherited requirements retain exact selectors and raw evidence; U07 needs a fresh independent audit before the separately registered U07-M and then U08.']
for i,row in enumerate(coverage['acceptances']):
 linked=[v for v in coverage['validations'] if v['id'] in row['validation_refs']]
 selectors=sorted({s for v in linked for k in ('core_selectors','host_selectors') for s in v[k]})
 if i in (12,13,14) and 'audit_correction' in selection:
  selectors=sorted(set(selectors)|{name for name in pool if 'u07_f01_' in name})
 conditions.append({'id':f'PE-CBST-{i+1:02d}','requirement':row['Observable Policy Acceptance'],'policy_ref':row['Policy Element Revision Ref'],'predecessor_row':{'path':str((P/'COVERAGE.json').relative_to(ROOT)),'validation_refs':row['validation_refs'],'sha256':sha(P/'COVERAGE.json')},'current_impact':impacts[i],'current_selectors':selectors,'selector_receipts':bound(selectors),'direct_receipts':[selection['object_receipt'],receipt_by_stem['public_starter_schema_']] if i<19 else [],'disposition':'Current-source predecessor selectors retained; fresh independent completion audit pending.'})
assert len(conditions)==161
r06=copy.deepcopy(old['r06_acceptances']);assert len(r06)==31
for row in r06:
 assert row['iva'] and all(any(c['id']==key for c in conditions) for key in row['iva'])
 row['status']='Exact original requirement retained; current per-condition selectors and public/PC receipts bound above; independent review pending.'
result={'source_sha256':current['sha256'],'predecessor_impact_sha256':sha(P/'PREDECESSOR_IMPACT.json'),'predecessor_coverage_sha256':sha(P/'COVERAGE.json'),'conditions':conditions,'r06_acceptances':r06,'original_requirement_sources':old['original_requirement_sources'],'independent_audit':False,'audit_correction':selection.get('audit_correction')}
with (S/final_names['impact']).open('x') as f:json.dump(result,f,ensure_ascii=False,indent=2);f.write('\n')
print('Bound 161 inherited conditions and 31 separate R06 requirements')
