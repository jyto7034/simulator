# U07 independent GEX-20 audit

**Verdict: Not Complete. Confidence: High. Open completion findings: 1 (`U07-F01`).** The frozen target is auditable and its selected execution evidence is current, but an independent HP object's evolution or retirement can erase unrelated blocking/contact relations. Passing suites do not establish the required preservation of those relations. No dependent U07-M/U08 implementation is eligible on this verdict.

This is a whole **GS-CIOB-001 / GOAL-CIOB-01 / PHASE-CIOB-01** single-phase review, not a review limited to the final source-storage/accounting correction. The reviewer was assigned as a fresh implementation/design nonparticipant under the requested GPT-6 Astra / High role. The reviewer did not implement a fix, write product tests, build or execute product suites, or delegate. Only new audit artifacts and audit-local restoration output were written. Source, tools, policy, execution records, target and historical evidence remained frozen. There is no additional GEX-30 for this single-phase Goal Set.

## Exact target, authority and currentness

| Item | Audited identity |
| --- | --- |
| Target | `execution/single_goal/target/u07_001` |
| MANIFEST SHA-256 | `44d34de0471d0a9c722da9eb0d6316ce66c75f84f070eefc6c8ae97675240e19` |
| Product SHA-256 | `7e60689016cd81398c254e1eb61705a8ea8eaaba9ba44cfd359c89a9a9ec2deb` |
| Source/data/canonical-doc SHA-256 | `27804e5e1b8a78f533f22641216d28f20d48eb3f2d411318ff982d9c0b4c6ec6` |
| Contextual Git HEAD | `f318414fb0156a67e04c5479000c2a2683fc0c09` |
| Membership | 5,275 paths; product388, source406, records4,795, references82 with group overlaps |
| Attribution | tracked/dirty189; tracked/clean36; untracked5,050 |
| Scoped delta | 86 changed,13 added,0 deleted source/canonical paths |

The selected authority chain is HA-CIOB-001 (`f5ae7b17…`), FPF-CIOB-001 (`9e113e8d…`), PP-CIOB-001 (`b0dd7deb…`), GS-CIOB-001 (`72b48726…`) and WR-CIOB-003 (`b9f65c19…`). The exact handoff, frozen20 acceptance meanings, ten required dimensions, six execution nodes, working policy, source review, PLAN, experiments and state inventory were considered under gex-r8 §§5,6.4,6.7,8.8 and usage v2.1.3. Canonical current gameplay includes `game_rules/combat.md`; historical policy was not rewritten. The U07-M registration changes the future sequence only, not current acceptance or budgets.

[CURRENTNESS_entry.json](CURRENTNESS_entry.json) and [CURRENTNESS_exit.json](CURRENTNESS_exit.json) establish current/unchanged membership and bytes at both boundaries. The entry checker independently verified every membership hash/size, current product and record membership, target/patch hashes, every U07 archive member and all four externally referenced U06 archives. U07 archive counts are393 before/406 after/4,795 records/82 references; U06 counts385 before/392 after/2,593 records/142 references. Exact forward and reverse patch restoration with fuzz zero both returned0 and reproduced their complete file sets. External archives were streamed and referenced, not recursively copied.

Git HEAD is context, not a substitute for dirty/untracked byte identity. The target preserves identified unrelated work. The skill-authoring document supplement is explicitly a **reconstructed preimage**, obtained by reversing the recorded U07 literal addition and appended section; it is not a contemporaneous intake archive. That limitation is visible in PREIMAGE_RECORD and does not prevent exact restoration of this declared target. The final status comparison excludes only this auditor's newly created directory; it does not reset, clean or discard unrelated work.

The exact immediate predecessor is U06 `target/u06_002`, MANIFEST `5b8f77eb…`, product `b33a8a01…`, and `post_audit/u06_20261005_002` **Complete / High**, with U06-F01 closed. Its positive semantic review is reused only for unchanged meanings and owners after current impact/source/selector checks. The old Not Complete U06 audit is not a positive gate. U00–U06 were not restarted.

## U07-F01: object lifecycle erases unrelated blocking relations

**Owner:** U07 independent-object lifecycle/spatial integration, specifically `object_evolution.rs:202` and `independent_objects.rs:333` through `build.rs:615`. **Priority:** completion-blocking correctness regression. **Evidence type:** direct production-source proof; no auditor-written runtime reproducer was executed.

`evolve_independent_object` assigns `self.block_state = Default::default()` for every non-trap form transition. This affects the entire battle, even when the evolved actor/wall is unrelated to an existing employee engagement or another wall contact. `retire_independent_object` likewise calls `withdraw_unit` for an active HP adapter; that helper globally resets the same state. TTL and Despawn source-policy retirement therefore introduce the same new interaction.

The canonical combat rule at lines89–90 says that new enemies do not displace existing partners. `refresh_block_state` explicitly implements that rule by first consuming `self.block_state.enemy_to_blocker` (lines139–174), retaining valid existing pairs and spending their capacity, and only then assigning new candidates. Erasing the map destroys the information that distinguishes an incumbent from a newcomer. With an unchanged capacity-one employee holding enemy A and a higher-priority unblocked B inside its valid radius, an unrelated object lifecycle transition makes the next refresh select B instead of preserving A. The old pair's target-preference cleanup is also bypassed by the direct assignment.

Wall contacts are more restrictive: `closest_available_blocker` at lines383–411 explicitly excludes `IndependentWall`. Their relationships are established by actual swept movement contact, not proximity. A subsequent generic refresh therefore cannot be assumed to restore a just-erased unrelated wall contact. Timing, basic-attack eligibility and movement can consequently differ until another appropriate contact occurs.

**Pre-existing versus new behavior:** the frozen SOURCE_BEFORE archive already contains the global reset in `withdraw_unit` at old `build.rs:614`. This audit does not misattribute that line to U07 or claim to have runtime-demonstrated every earlier employee withdrawal case. U07 adds both the direct evolution reset and the independent-object retirement caller, extending that reset into new accepted lifecycle transitions. The mandatory correction boundary is preservation/invalidation of the proper relations across those U07 transitions. If the implementation changes the common withdrawal owner, its employee and other existing callers require impact review and affected predecessor regression; if it changes the U07 bridge, that bridge must establish the same preserved semantics. This audit does not prescribe the implementation choice.

Current tests establish HP preservation/clamp, epoch/TTL, fault rollback and individual wall destruction. The segmented capacity test retains selected existing partners while reducing capacity, but does not combine unrelated object evolution/retirement with occupied employee capacity or another wall contact. Those passing tests do not refute the source defect.

**Required closure evidence:** implementation-owned normal creation followed by HP actor/wall evolution and active-object expiry/source-policy retirement, while an unrelated valid employee pair and a distinct existing wall contact are present. Observe incumbent preservation, actual basic targeting/range/contact and route behavior; validate that genuinely invalid relationships are removed. Cover same-time transition/refresh and clone/failed-transition rollback as applicable. Preserve source/failure evidence, rerun affected focused/broad/consumer checks, determine baseline impact rather than assuming reuse, freeze a new exact target, and return for independent re-audit. No policy change or new allowance is required to preserve the existing contract.

## Whole-goal judgments and source review

[CONDITION_JUDGMENTS.json](CONDITION_JUDGMENTS.json) retains every exact acceptance text and selector/direct-receipt locator, all161 predecessor conditions and31 separately tracked R06 acceptances. Each includes an independent judgment and semantic basis. Supported rows are not an overall completion award. [SOURCE_REVIEW.json](SOURCE_REVIEW.json) distinguishes new/full owner review, scoped deltas and valid predecessor reuse; it does not claim every historical source line was reread.

| Validation | Independent judgment |
| --- | --- |
| V01 side/capability/result isolation | Supported: actual-side adapters; non-unit traps; independent sources excluded from required authored victory groups and employee reward/death authority. |
| V02 atomic creation/replay | Supported: common candidate owns all placements/costs/IDs/forms/queue/settlements; collision, expiry replay and late failure/retry are exercised. |
| V03 placement/counters | Supported: every offset/reach/ground/reserved/work/occupancy check, copy/family limits and one-slot evolution share the current owner. |
| V04 actor | Supported: own attack/cast actions and both-side explicit motion, with no employee command or parent use authority. |
| V05 field | Supported: existing finite area owner, normal external periodic consumer, Pause/TTL and parent-use separation. |
| V06 friends | Supported: official base/awakened source copies, snapshot trap damage, eligible target/empty/air/consumption cases and actual post-withdraw output. |
| V07 evolution | **Incomplete F01**; local ID/epoch/TTL/cost/no-free-heal tests pass, integrated contact preservation does not. |
| V08 source removal/provenance | Supported for explicit Persist/Despawn identity and use unlock; lifecycle contact side effect is tracked under V09/F01. |
| V09 cleanup | **Incomplete F01** despite idempotent epoch/removal, queue/counter/terminal cleanup evidence. |
| V10 segmented pool | Supported: one remaining balance, derived stacks/top/block bonus, exact source selection, single absorption and atomic detachment. |
| V11 HP fallback | Supported: positive nonlethal cost, protection bypass as cost, actual wall HP and late rollback/retry. |
| V12 wall | **Incomplete F01**; individual swept contact/attack/destruction/route tests pass, unrelated lifecycle contacts are lost. |
| V13 maxHP/death | Supported: explicit clamp/finalization order and no duplicate ordinary death/proc for objects; inherited special death/cost/protection regressions remain current. |
| V14 acceptance/cancellation | Supported: preflight and release recheck distinguish accepted resonance from later creation cost; pending/released lifecycle gates remain separate. |
| V15 preview/query | Supported for pure current object/cost projections; block-derived inherited query behavior remains affected by F01. |
| V16 ordering/time | **Incomplete F01** for lifecycle/spatial interaction; stable queue/ID/epoch, Pause and TTL evidence remains valid in its tested scope. |
| V17 clone/reconnect/persistence | Supported: live copy isolation and actor reconnect; fresh Continue uses pre-entry, with no new live battle disk codec claim. |
| V18 external authoring/consumer | Supported: strict RON through real compile/trial/CLI/CoreApplication and actual host serializer. |
| V19 final evidence/guardrails | Current execution/cost evidence verified; **whole integration incomplete F01**. |
| V20 independent closure | **Not Complete**: exact restoration/currentness pass; independent Complete/High and zero findings do not. |

All ten dimensions remain required. Invalid input, failure/rollback, retry/idempotency, boundary/empty/max, disconnect/restart/persistence and live-data have specific current nonzero evidence. Success, ordering/concurrency, producer-consumer and complete absence of an unauthorized reset/recovery path cannot be closed for the lifecycle/contact combination. No dimension is hidden behind N/A.

For predecessor conditions, **PE-CBST-13 fails in the U07 interaction**; PE-CBST-14/15 require affected block-target/range/query revalidation. Other exact meanings have supported current applicability and selectors, with no separate detected regression. This does not retroactively change the historical U06 verdict. The original31 R06 texts and linked IVA meanings remain exact; expression/aggregate/stat/copy ownership is not replaced by independent-object identity.

The six nodes were reviewed at their actual boundaries: DEFINITIONS strict schema/compile and normal catalog input; CREATION common transaction/ledger/source lifetime; SPATIAL actor/trap/wall execution (F01 open); SEGMENTS single shield/HP cost owner; PUBLIC shared projections/normal application/host/restore; VERIFY current evidence, guardrails and exact target (whole completion withheld). No remaining unreviewed execution node is concealed by stopping after the first finding.

## Source storage, accounting and public evidence

The new source representation keeps the default inline RuntimeFactSource at136 bytes and moves optional origin/reinput observations into a shared wrapper. The manual borrowed serializer preserves legacy field order/omission and handles all1,024 optional combinations. `Arc` CloneShape meters a shared handle; retained/admission paths separately price the actual wrapper, nested source strings and reinput payload. Origin stripping removes parent action/use/delivery/resource/finite/flow/reaction/reinput credentials while retaining the finite creation observation. Own actor actions validate their bound origin/cast rather than treating creation provenance as authority. Actual late released-reinput wrapper failure restores activation/cast/queue/source and retries successfully.

The reviewer followed the shared authoring charge forwarding to native, the historical19-site correction, the later seven duplicate pairs and partial preview-sort correction, including native-only geometry pricing. Exact/one-short tests cover object/pool scans, retained request/reference scans, form/expiry pruning, overlay validation and preview count/sort/copy before materialization. The current source scan reports no remaining U07-added adjacent duplicate candidates; it is supporting static evidence, not a proof of every possible runtime cost.

[EVIDENCE_VERIFICATION.json](EVIDENCE_VERIFICATION.json) independently checks50 selected receipt/log pairs, source_before/source_after identities, raw exits, archived runner and command inputs, actual nonzero selector lines, suite totals and462 adapted file copies. All181 U07/predecessor validation rows have their recorded nonzero selector/direct receipt bindings. No failed006 or PC001 result is relabeled as current success.

Final007 totals are Core1,853 default/1,853 fixtures/1,862 observation; host93 default/101 observation, with the one ordinarily ignored heap test separately run and passed. Fmt/check/event-log/clippy, release, focused native, public guards and five fault modes are selected current evidence. The audit reused those executions and did not run redundant suites merely because a new reviewer arrived.

[PUBLIC_VERIFICATION.json](PUBLIC_VERIFICATION.json) records1,509 checks of hash-addressed public input paths and independently parses raw outcomes. Six normal external creation trials yield one object each, correct exact copy, small-to-big epoch2, segment-derived wall HP50/pool100 and positive fallback HP. Separate raw combat follows normal cast then creator withdrawal: actor BasicAttack10, trap Ability raw24/final25 once then consumption, wall HP3→2→1→0 once without UnitDied and enemy route resumption. The wall HP3/defense0 and readiness/reward/unlocked-form inputs are explicit test fixtures; official wall HP120/defense10 is separately exercised by the real actor test. No unavailable official reward/recipe is claimed.

Strict current public readers exercise object10 cases (nine negatives), starter11 (seven negatives) and general schema11 (ten negatives), preserving precise diagnostics. CoreApplication tests use normal grant/equipment/copy/form/deploy/focus/preview/cast/withdraw; the real actor test accepts a paused cast, changes sessions without a duplicate cost, resumes to create the official wall, then reconnects/withdraws with Persist. Durable Continue/SavePoint/rewind and original-file protection use the established pre-entry boundary and current host/process evidence. Unchanged13-input service plumbing is valid predecessor reuse; there is no deployed service or Unity assertion.

## Final baseline and retained exceptions

The original PC12-source×500-enemy/8MiB input remains present. The strict comparator was read and independently invoked against raw predecessor/current data:576 rows,36 conditions,31 successes/five original typed refusals. It excludes only elapsed timing and permits exactly the measured152-byte empty object state plus48-byte optional placement-domain cost, totaling200 retained/admission bytes. The five original refusal requested-byte values receive that same exact200; code/site/cap, work, clone, allocation, evaluator, facts/gameplay/log fields remain exact. Seventeen negative controls, including the actual failed PC001 source12/enemy500 output, are rejected. This is a stated layout-accounting adjustment, not a raised8MiB ceiling or discarded workload.

[PERFORMANCE_VERIFICATION.json](PERFORMANCE_VERIFICATION.json) independently matches the original R05 tool members to their historical archive, verifies the literal one-line `object_placement: None` migration, checks original baseline equivalence except its already accepted log-hash migration, raw workload identity, unchanged guard implementation, and fresh004 depinfo. The isolated harness compiles the exact current final-performance source while using historical preservation bytes solely for its include provenance sentinel. Cargo differs only by the path binding to current Core; lock and product are exact. Failed002/003 preflight attempts and stale shared-cache include evidence remain historical; fresh004 produces measurements.

R05 raw004 **exits101**, not0. Event-heavy maxima operations44,194/retained265,846/peak271,415 remain within original70,671/276,174/276,174. Concurrent maxima370,928/4,196,928/9,660,264 retain operations and retained caps389,240/4,196,928, but peak exceeds9,030,188 by630,076. The sole panic is the unchanged concurrent allocation guard, with no other hard failure and deterministic parity intact. Only the inherited concurrent-peak warning applies. Instrumented latency has `latency_gate_applicable=false`; shared-host timings and the existing20-minute stress exclusion are not new exemptions. Nothing in F01 depends on accepting another cost allowance.

## Implementation guardrails and return boundary

Direct source review found no production dispatch on test/fixture identity, catch-all invalid-input normalization, duplicated presentation authority, unused future-only registry, arbitrary sequence/delay workaround, unexplained golden replacement/disabled assertion, or parallel superseded schema/runtime path. Test-only readiness/grants/data edits are explicitly confined and normal public consumers execute the production contract. Added durable documentation accurately states current ports and unsupported live persistence/reward/infection scope. The whole preservation/ordering requirement still fails at F01; guardrail compliance is not substituted for that correctness requirement or inferred merely from PASS summaries.

Limits of this pass are explicit: no new product runtime reproduction of F01, no exhaustive Cartesian runtime execution and no reread of every unchanged historical source line. These limits do not make the finding Blocked: the relevant source and contract are available, and the direct unconditional erase is established. The remaining uncertainty is the full observable regression envelope and the implementation-owned correction/verification, not whether all relations are erased.

Return to GEX-10 for U07-F01 correction and appropriately current validation, then freeze a new exact target for independent GEX-20 re-audit. Historical evidence and this **Not Complete** audit remain immutable. Do not start U07-M/U08 based on this target. The reviewer closes this read-only pass without modifying product or directing a particular fix design.
