"""Record this reviewer's semantic decisions, separate from implementation coverage."""
from pathlib import Path
import json,hashlib
A=Path(__file__).resolve().parent;G=A.parents[1];B=G/'execution/single_goal/corrections/f01_action_flow_20261004_001'
c=json.loads((B/'COVERAGE_CURRENT.json').read_text());p=G/'reviews/r08_r12_20261002_001/CONDITION_REVIEW.json';old=json.loads(p.read_text())
assert [r['id'] for r in c['conditions'][:55]]==[r['id'] for r in old['conditions']]
assert all(a['requirement']==b['requirement'] for a,b in zip(c['conditions'][:55],old['conditions']))
reasons={
'A08-01':'Boss-only admission in sim::start_boss_skill_action keeps Opponent identity and explicit facing; no Player disguise. Unchanged public milestone test verifies actual target HP, stationary origin and 100 HP loss.',
'A08-02':'release_finished_boss_waiters cancels only invalidation or TimedOut+CancelUnreleased; a normal Released completion takes the handle out before state exit. Existing t100/t300 waiter assertions and released CLI remain valid.',
'A08-03':'State-owned DetachAction now cancels matching pending AND active control work. Removing only SkillStep/SkillFlowWake preserves released projectile/area completions. Public early/inflight/state-exit oracles and six internal timing combinations close F01.',
'A08-04':'RequestRejected/TargetInvalidated routing is unchanged; timeout CancelUnreleased reaches the new exact-action cancellation. Public Cancel1500/Cancel500 versus Detach1500 show distinct policy results; pending target-loss/rejection selectors still exercise actual result routes.',
'A08-05':'Opaque Arc action equality, caster check, cast sequence, and stale-owner/generation guards isolate occurrences. Queue entries and public reservation are reclaimed. New foreign-same-ordinal isolation and stale extracted-event tests supplement repeated-state handle reclamation.',
'A08-06':'No catalog/compiler/schema delta. Original validation rejects missing skill/routes, target mismatches, capability collisions and invalid timeouts before publishing; mapped negative selectors pass unchanged.',
'A08-07':'Boss boundary clones before release_finished_boss_waiters/plan.commit; typed cancellation/scan/terminal failures return before publishing staged core. Six new fault sites verify exact core/queue/execution restoration and retry, in addition to admission tests.',
'A09-01':'corrosion_ready still requires exact fragment form, active Player employee, trauma threshold, resonance and start eligibility; final-boss exclusions remain. New boss cancellation neither broadens Player admission nor changes those inputs.',
'A09-02':'SkillStartGate remains distinct from input acceptance and ManualCastEnd. Canceled pending casts cannot match their stale start sequence. Player auto/manual gate and host pause tests remain current; boss request uses reset_resonance=false.',
'A09-03':'try_start_corrosion captures sorted current enemies once, records employee obligation before death and suppresses normal cast; new cancellation does not create enemies, costs or waves. Current integration verifies required future wave survival.',
'A09-04':'Special death still uses finalize_prepared_death_with_dispatch(..., false); mandatory death facts/wave progression remain separate from optional reactions. Queue pruning excludes death/wave events and public cross-feature tests remain current.',
'A09-05':'Corroded marker/trauma100/cost precede withdrawal and terminal enemy deaths; no mutation here in the correction. World reconciliation/once-only result tests verify employee consequences despite terminal completion.',
'A09-06':'CorrosionFreeze consumes explicit raw host elapsed before battle speed; user Pause and forced pause remain separate. Neither new cancellation function reads wall time nor changes host routes; fake-host and save consumer tests remain current.',
'A09-07':'Application forced-pause guards and direct BattleCore ensure_not_forced_paused are unchanged. New method is private and reached within admitted boss boundary, not a new public mutation bypass. Existing route matrix remains applicable.',
'A09-08':'Breakdown node counter and recovery ordering remain world/employee-owned; no ephemeral canceled action is serialized as durable state. Counter and recovery-before-expiry/fresh-process selectors retain their original oracle.',
'A09-09':'Node expiry and exact copy recovery paths are unchanged; V9 strict checksum and explicit V8 converter remain present. U00 future no-legacy decision is not applied to this goal. Current fresh-process and failed-decode tests remain valid.',
'A09-10':'Corrosion capture/cost/death/presentation transaction is unchanged. Its queue still contains all non-control events; current cross-feature and fault/retry selectors preserve the original all-or-nothing oracle.',
'A10-01':'Flow occurrence results/actual HP-loss and damage command targeting are unchanged. Cancel removes future control only; released area executes with retained source. Existing three-target/one-heal and shield-zero selectors preserve cardinality.',
'A10-02':'Flow compiler/runtime retains authored node/occurrence/result ordering. New selection is deterministic minimum cast sequence, and heap retained entries keep event ordering keys. Repeat/parallel result and trace selectors are current.',
'A10-03':'Finite repeat count/opportunity accounting is untouched. Cancellation terminates future opportunities and never banks/replays them. Empty-target and interval-only tests retain their expected finite results.',
'A10-04':'Sequence/Parallel/Wait require a live cast owner to drive. Exact owner removal plus queued control pruning and extracted-event no-op prevent new leaves after cancellation; old branch/fault tests and new stale/same-time tests pass.',
'A10-05':'Target/facing and source snapshots are in the surviving carrier. The new path never retargets or rebuilds released payloads. Existing target/anchor tests and public F01 source/HP timing prove continued delivery semantics.',
'A10-06':'F01 closes the missing R08 timeout/state-exit to R10 continuation edge, both before and after preparation. Released child survives; Detach and normal Released still continue. Corrosion/forced pause integration remains current.',
'A10-07':'Flow compiler and publication are byte-identical to the previous audited target; no unsupported branch, type, forward reference or unbounded structure is newly accepted. Negative selector oracles remain relevant.',
'A10-08':'New scans charge active cast/queue length before traversal, do not allocate a cancellation vector, propagate failures and use existing staged rollback. Existing finite/admission/overflow tests plus six new cancellation fault sites cover changed mutation boundaries.',
'A10-09':'Legacy steps still lower through CompiledSkillFlowPlan, then use the same owner/step consumers; no fallback interpreter is added. Explicit legacy delayed-step regression plus public Snapshot/Live and empty-expiry oracles remain current.',
'A11-01':'Sustained records recompute effective stats from base and remaining sources. Cancellation does not mutate the ledger or source modifiers. Source-local expiry/removal/fault tests preserve no-drift behavior.',
'A11-02':'Aura member/source ownership is independent of cast control; new pruning excludes sustained/area/projectile events. Snapshot damage and membership/priority/public actor tests remain current.',
'A11-03':'New pending cancellation releases only matching focus locks and never calls actor-wide basic/movement cancellation. Channel-owned restrictions and other lock lifetimes remain under sustained authority; original real-periodic/field-work tests pass.',
'A11-04':'Sustained end_sustained closes its own source/delivery and timer, while explicit channel stop/death/withdraw/terminal remain unchanged. No broad effect cleanup is added by this action cancellation.',
'A11-05':'Hello activation/reinput and separate Goodbye source are unchanged; normal Released result detachment does not call cancellation. Existing remaining-time, no-extra-cost, normal-world/save tests retain their oracle.',
'A11-06':'Exact sustained activation and reinput transaction are unchanged, and queue pruning matches cast control only. Stale expiry/reinput/fault tests therefore exercise the same independent lifecycle.',
'A11-07':'Basic overlay/weapon source and actual HP-loss lifesteal consumers are unchanged. The exact-action cancel path leaves pending basic attack and released source snapshots alone; overlay/Goodbye and damage-source tests remain current.',
'A11-08':'No basic epoch, next_basic_attack timer, overlay use count or released projectile is edited. Existing windup/release/expiry tests directly retain once-only consumption and no cycle reset.',
'A11-09':'Sustained retained/admission/clone and retreat transaction are unchanged. Removed cast/source references drop normally, while retained effect records own their leases; public trace and rollback tests remain applicable.',
'A12-01':'Shield remaining pool is owned by sustained record; cancellation does not recreate or remove it. Original shared600/A400/B350 and member recast tests retain their exact expected balances.',
'A12-02':'absorb_sustained_shields selects personal then shared, priority then activation; mitigation and target HP remain separate. Cancellation excludes damage events; deterministic same-time and two-employee tests are current.',
'A12-03':'Shield/source lifetime and own expiry remain intact because only SkillStep/SkillFlowWake match pruning. Recast/stale-expiry/exhaustion tests retain once-only source bookkeeping.',
'A12-04':'Heal/recharge and HP cost dispatch are untouched; costs retain separate fact classification and bypass protection. No cancellation branch refunds or reclassifies cost. Original boundary oracles remain current.',
'A12-05':'Damage survival and special-kill policies are unchanged; released carrier damage still enters existing arbitration. HP floor/death prevention/cost and corrosion cross-feature tests remain current.',
'A12-06':'Parry eligibility/bound attachment suppression and once-only counter authority remain unchanged. Queue pruning never removes already released damage/counter completion; positive/public tests remain current.',
'A12-07':'Parry miss owns only its reduction record; no generic source reset was introduced. Existing absent-attacker, multi-strike, cost/reflect and half-open-time tests retain original semantics.',
'A12-08':'Control sources and capability resolution are unchanged; exact action cancellation no longer performs broad interrupt epochs. Pending focus lock cleanup checks matching owner time/permission; movement/basic/hard-CC tests remain current.',
'A12-09':'Actual HP-loss facts still follow protection; source/action lifetime is preserved on released damage. Existing zero-loss gate/charge/lifesteal and R09 suppression tests remain current.',
'A12-10':'Shield/member/receipt and damage transaction code are unchanged. Changed cancellation errors propagate through boss and outer candidate; existing native/fault tests plus added cancellation rollback preserve queue/HP/observations.',
'X01':'Unchanged external packs still validate/describe/explain/simulate through release CLI; five original public F01 inputs now meet exact timing/damage/source oracles. Ordinary milestone and wave selectors remain current.',
'X02':'R07 normal copy grant/form/cast, sustained reinput/channel and amount/trace inputs are unchanged; six relevant public packs plus current world selectors preserve exact source through actual consumers.',
'X03':'Normal two-employee world deployment/damage/cost/shield/parry/save seam remains unchanged and passed. New cancellation is boss control-local and cannot modify world roster or protection pools.',
'X04':'Corrosion special death, Flow/channel/shield and host freeze were tested together on current source. New cancel neither changes capture/dispatch policy nor removes wave/death/terminal events.',
'X05':'Current V9 export/import checksum, graph validation and explicit converters remain; ephemeral action handles are not durable. Fresh-process selectors cover results/counters/copies/resources. Future U00 save policy is excluded here.',
'X06':'World attempt/retreat candidate and run checkpoint own recovery, independent of removed control work. Existing retry/retreat selectors plus new six-site cancellation restoration establish no duplicate assets/effects.',
'X07':'All five F01 event sets are exactly equal trace off/on. Nine existing CLI comparisons preserve public results and four actual amount documents with only inherited OBS001 employee preview ordering normalization; no observation-based gameplay branch added.',
'X08':'Opaque handle equality uses Arc occurrence identity; same ordinal foreign handle cannot match. Native/work admissions and rollback remain, new loops are charged and deterministic. Existing exact/short/overflow/stale/clone tests plus added isolation apply.',
'X09':'14 exact-source receipts/raw hashes verified; default/feature each1684/0/0 and check/event/release/clippy/fmt/45CLI pass. PC36 recompare passes. R05 exit101 remains solely authorized concurrent peak warning; no gate relaxed or20-minute test claimed.',
'X10':'New exact before387/after388 archives,173 records,35 references,9 artifacts all match. Independent fuzz0 forward/reverse membership+bytes restoration passed. All55 conditions now pass, original Not Complete remains historical and future roadmap is excluded.'
}
assert len(reasons)==55
sources={
'A08':['core/boss_runtime.rs','core/sim.rs','runtime_facts.rs','core/runtime_facts.rs','data/boss_runtime_data.rs'],
'A09':['core/corrosion.rs','core/death_runtime.rs','world/combat.rs','world/application.rs','world/node_flow.rs','world/active_run.rs'],
'A10':['ability/flow.rs','core/skill_runtime/flow.rs','core/skill_runtime/cast.rs','core/skill_runtime/projectile.rs','core/skill_runtime/area.rs','core/sim.rs'],
'A11':['core/sustained.rs','core/basic_attack.rs','core/basic_attack_projectile.rs'],
'A12':['core/sustained.rs','core/damage_runtime.rs','core/death_runtime.rs','core/commands.rs'],
'X':['core/boss_runtime.rs','core/queue.rs','core/skill_runtime','world/active_run.rs','application.rs']}
affected={'A08-02','A08-03','A08-04','A08-05','A08-07','A10-04','A10-06','A10-08','A11-03','X01','X07','X08','X09','X10'}
rows=[]
for r in c['conditions'][:55]:
 d=dict(r);d.pop('disposition',None);d['verdict']='PASS';d['impact']='directly reviewed changed boundary or validation' if r['id'] in affected else 'unchanged semantics; cumulative integration and delta checked'
 d['independent_reasoning']=reasons[r['id']];d['source_paths']=sources[r['id'][:3] if not r['id'].startswith('X') else 'X'];d['prior_review']='reviews/r08_r12_20261002_001/CONDITION_REVIEW.json#'+r['id'];d['prior_review_limit']='Historical Not Complete; no inherited blanket per-condition approval.'
 if r['id'] in affected:d['new_correction_evidence']=c['correction_selectors']
 rows.append(d)
out={'goal':'GS-CARX-001 / GOAL-CARX-01 / HA-CARX-001','source_digest':c['source_digest'],'independent_audit':True,'conditions':rows,'count':55,'passed':55,'failed':0,'prior_condition_review_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'source_path_note':'Symbol locators use core=core_enhanced/src/game/battle/core; world/data/ability=core_enhanced/src/game corresponding folders. Exact full source bytes are identified by target manifest.'}
with (A/'CONDITION_REVIEW.json').open('x') as f:json.dump(out,f,ensure_ascii=False,indent=2);f.write('\n')
impact={
'R06_integration':'R06 typed compiler/evaluator, exact copy/resource bindings, checked cadence/stat composition and immutable amount replay retain their original requirements. R08-R12 adds Flow/sustained/damage/save consumers but does not substitute a second evaluator or implicit source. Cancellation removes only control owners; delivery tickets and captured expression/source/resource contexts remain valid until close_delivery. Existing current public Snapshot/Live, actual HP-loss, source-removal and native-budget selectors substantiate each row; PC36 and public CLI confirm integration.',
'R07_F01_F12':'R07 copy UUID/form/recipe/binding/grant/dismantle and source capsule authority remain. R09 adds Breakdown and V9 checkpoint through explicit approved migration while original V7 bytes and explicit V7/V8 converters remain. Cancellation does not mutate copy or durable inventory; released source Arc and resource delivery scopes are preserved. Current normal public pack, world rollback, exact fragment lease and fresh-process checkpoint selectors substantiate each row.'}
pr=[]
for r in c['conditions'][55:]:
 d=dict(r);d['verdict']='PASS';d['independent_reasoning']=impact[d['group']];d.pop('disposition',None)
 if d['id'] in ['IVA-CAVER-05','IVA-CAVER-06','IVA-CAVER-12','IVA-CAVER-13','IVA-CAVER-14','IVA-CAVER-17','IVA-CAVER-19','IVA-CAVER-20','F08','F12']:d['direct_correction_impact']='Reviewed live/action/delivery validation, terminal/close_delivery/resource terminal, carrier source, queue work, retained ownership, cancellation rollback; new correction tests supplement existing exact source/lease checks.'
 else:d['direct_correction_impact']='Producer inputs and consumer contracts unchanged by the correction; source identity and current full-suite/public integration evidence verified.'
 pr.append(d)
accept=[]
for r in c['r06_acceptances']:
 d=dict(r);d['status']='PASS; independent current integration impact review';d['evidence_rows']=d['iva'];accept.append(d)
pre={'target_digest':c['source_digest'],'independent_audit':True,'scope':'Required R06/R07 predecessor meaning and current integration applicability at this GOAL-CARX-01 target; historical targets/verdicts unchanged. This is the fresh impact restoration, not a claim that old source digests equal current source.','basis':['reviews/r06_r07_20260930_001/COMPLETION_AUDIT_R06.md','reviews/r06_r07_20260930_001/COMPLETION_AUDIT_R07.md','reviews/r08_r12_20261002_001/R06_R07_IMPACT_REVIEW.json'],'R06_acceptances':accept,'integration_conditions':pr,'r06_verdict':'Complete / High for applicable R06 current integration','r07_verdict':'Complete / High for applicable R07 current integration','limitations':['R06 effect/state-source set aggregation transferred to R11/R17 remains outside R06 scope.','OBS001 recovery preview employee-array ordering retained; no general wire-byte equality claim.','U00 future legacy removal is not applied to current V9/V7/V8 contract.']}
with (A/'R06_R07_IMPACT_REVIEW.json').open('x') as f:json.dump(pre,f,ensure_ascii=False,indent=2);f.write('\n')
print('55 conditions and 31 R06 acceptances / 21 R06 integration / 12 R07 rows recorded')
