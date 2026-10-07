import json,pathlib,hashlib
R=pathlib.Path('/mnt/f/work/simulator');H=R/'core_enhanced/docs/goals/core_independent_objects';S=H/'execution/single_goal';A=pathlib.Path(__file__).parent;O=A.parent/'u07_20261005_001'
load=lambda p:json.loads(p.read_text());sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();old=load(O/'CONDITION_JUDGMENTS.json');c=load(S/'COVERAGE_002.json');p=load(S/'PREDECESSOR_IMPACT_002.json');changed={x['path'] for x in load(A/'SOURCE_DELTA.json')}
updates={
'V07':'Same identity/counter, epoch/TTL, cancellation, overlay rebase and no-free-heal/clamp remain unchanged. The final HP form now passes preserved authoritative relations through the existing valid-incumbent reconciler. Six normal actor/wall lifecycle regressions, invalid own actor mobility/wall expiry, actual targeting/basic hit/movement and clone/late queue rollback establish U07-F01 closure.',
'V09':'Active HP retirement keeps the relation map through the existing withdrawal/field-work cleanup and always reaches final-state refresh. Valid unrelated incumbent and swept wall contact survive; expired own wall relation/target preference is released and original route resumes. Epoch/idempotent removal/queue/counter/terminal behavior remains current.',
'V12':'Actual swept hostile ground contact, airborne/same-side exclusions, damage/destruction and route behavior remain supported. Correction regressions preserve a distinct real wall contact across unrelated actor/wall evolution, expiry and Despawn; invalid own wall expiry removes the contact and permits actual route motion.',
'V15':'Shared object/cost projections remain pure and revalidate stale state. Current block-derived target/range queries now see the preserved incumbent and actual wall contact after lifecycle transitions, with genuine own invalidation reflected. Focused predicates are paired with actual attacks/HP deltas and movement.',
'V16':'Stable IDs/queue epoch/Pause/TTL ordering remain unchanged. Immediate final-form refresh precedes expiry enqueue, retains existing valid reservations and invalidates changed own mobility. Normal lifecycle cases and exact snapshot/queue/preference rollback with successful retry support integrated ordering.',
'V19':'Current final008 Core/host, separate heap, public/fault/defaults, PC005 raw strict200 and17negative controls, and R05005 inherited peak-only warning are independently bound. Focused00412 and native00158 close the prior missing lifecycle/contact interaction; raw failures and withdrawn investigation stay failures.',
'V20':'Exact target entry/exit, all member/archive bindings and fuzz-zero forward/reverse restoration are verified. Whole-goal independent judgment is Complete/High with no open findings; closure artifact supplies the final gate. Single phase requires no GEX30.'}
validations=[]
for row,prior in zip(c['validations'],old['validations']):
 v=dict(row);v.update(independent_judgment='Complete',independent_reason=updates.get(row['id'],prior['independent_reason']),reuse_basis='REUSE_VERIFICATION.json: exact acceptance/meaning; all byte-identical owners retain prior direct semantic review; cumulative changed consumers re-reviewed.',current_evidence_check='EVIDENCE_VERIFICATION.json');validations.append(v)
conditions=[]
for row,prior in zip(p['conditions'],old['predecessor_conditions']):
 reason=prior['independent_reason']
 reason=reason.replace('Specific gameplay lifecycle exception is separately reported as U07-F01 rather than hidden by broad suite success.','The prior lifecycle exception is independently closed by direct source and current normal-transition regressions.')
 reason=reason.replace('block-related exception is explicitly recorded below.','block-related preservation now has direct corrected-owner and actual consumer evidence.')
 if row['id']=='PE-CBST-13':reason='Original incumbent reservation rule is unchanged. Six normal actor/wall evolve/expiry/Despawn cases establish incumbents and separate swept contacts before the transition, then retain them against a higher-priority newcomer. Existing reconciler validity/capacity authority is preserved; own invalid relations are removed.'
 if row['id'] in ['PE-CBST-14','PE-CBST-15']:reason='The corrected authoritative relation feeds the existing target/range/query and attack/movement consumers. Focused regressions inspect preferred target and capability, actual damage after ordinary attack scheduling, stationary valid contacts and route resumption after true own invalidation. Current original U06 selectors also pass.'
 v=dict(row);v.update(independent_judgment='Complete',independent_reason=reason,reuse_basis='Exact original condition and meaning-basis equality with prior independent review; current per-condition selectors and direct receipts verified.',current_evidence_check='EVIDENCE_VERIFICATION.json',cumulative_delta_review='SOURCE_DELTA.json; common withdrawal/corrosion/summon/independent retirement/fieldwork/resonance consumers directly reviewed. No condition relies on an overall prior U07 Complete verdict.');conditions.append(v)
r06=[]
for row,prior in zip(p['r06_acceptances'],old['r06_acceptances']):
 v=dict(row);v.update(independent_judgment='Complete',independent_reason=prior['independent_reason'],reuse_basis='Exact original acceptance/expected/IVA links independently equal; current IVA consumer selectors and PC raw values verified.');r06.append(v)
dim_reasons={
'success':'Normal public creation/combat and corrected lifecycle transitions have actual nonzero outputs.',
'invalid_input':'Strict schema and typed placement/source/cost/identity refusals remain current.',
'failure_rollback':'Existing object/pool/source rollback plus changed own-pair/preferences late queue failure restore exact candidate snapshots.',
'retry_idempotency':'Replay collision/expiry semantics and late-failure successful retry retain unchanged ownership.',
'boundary_empty_max':'Exact/one-short, empty/no-target, copy limits, real history overflow and sticky refusal include new evolution refresh.',
'ordering_concurrency':'Stable queue/epoch and same-time lifecycle refresh preserve incumbent reservations; mutable effects remain inside transaction.',
'disconnect_restart_persistence':'Current real actor pending/reconnect, clone, durable pre-entry Continue/rewind preserve established supported boundary.',
'producer_consumer':'External RON→compile/trial/World/host, actual attack/HP/contact/movement consumers verified.',
'live_data':'Official catalog/form/copy/wall constants and explicit tuned fixture distinctions unchanged and reviewed.',
'legacy_fallback_absence':'No alternate shadow/reset recovery or fixture dispatch; one authoritative relation reconciler and explicit schema refusals.'}
dimensions=[{'dimension':name,'required':True,'validation_refs':refs,'independent_judgment':'Complete','reason':dim_reasons.get(name,'Current specific linked evidence and unchanged exact dimension meaning reviewed; no N/A used.')} for name,refs in c['required_dimensions'].items()]
out={'schema':'independent-gex20-condition-judgments-v2','audit_target':'target/u07_002','source_sha256':c['source_sha256'],'overall_verdict':'Complete','confidence':'High','open_findings':0,'acceptances':c['acceptances'],'validations':validations,'dimensions':dimensions,'predecessor_conditions':conditions,'r06_acceptances':r06,'scope':'Whole GS-CIOB001 single-phase audit; cumulative first-target delta and affected consumers directly re-reviewed, valid exact unchanged condition judgments reused per gex-r8 section5. Prior U07 overall NotComplete is not a positive gate.'}
(A/'CONDITION_JUDGMENTS.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n')
review=load(O/'SOURCE_REVIEW.json')
for group in review['direct_source_review']:
 group['prior_review_scope']=group.pop('review_scope')
 for f in group['files']:f.update(prior_sha256=f['sha256'],sha256=sha(R/f['path']),current_review='directly re-reviewed cumulative change' if f['path'] in changed else 'exact bytes; prior direct semantic review reused with current affected consumers')
review['additional_direct_review']=[{'path':path,'sha256':sha(R/path),'scope':scope} for path,scope in [
('core_enhanced/src/game/battle/core/tests/u07_block_lifecycle.inc.rs','Full current12 focused regression source, normal preconditions, actual consumers, exact/short/history/sticky/clone/latefailure.'),
('core_enhanced/src/game/battle/core/field_work.rs','Full reconciliation path always ends refresh; fieldwork transitions release their own tactical pairs.'),
('core_enhanced/src/game/battle/core/corrosion.rs','Existing withdrawal bridge, captured source, special death ordering.'),
('core_enhanced/src/game/battle/core/summon_runtime.rs','Existing summon retirement withdrawal bridge and Despawn lifecycle.'),
('core_enhanced/src/game/battle/core/movement/path.rs','Actual route comparator center allocation and scan bound.'),
('core_enhanced/src/owned_collections.rs','Tombstone-aware MapCloneHistory bound and fresh map growth assumptions.'),
('core_enhanced/docs/goals/core_independent_objects/tools/bind_r05_execution.py','Full005depinfo/source/binary/environment binding; audit independently verifies output without running mutating binder.')]]
review['scope_note']='First-pass direct review remains historical. This file records exact reused bytes and new direct review separately. Prior scope strings describing F01 are historical; current closure is in this audit report/FINDINGS. Five product paths and ROADMAP change comprise the entire first→second source delta.'
(A/'SOURCE_REVIEW.json').write_text(json.dumps(review,ensure_ascii=False,indent=2)+'\n')
print('wrote20/10/161+31 Complete judgments and current source/reuse record')
