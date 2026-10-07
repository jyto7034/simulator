# U06 fresh independent GEX-20 re-audit

Verdict: **Complete / High** for the exact current `u06_002` target. **U06-F01 is closed. No completion-blocking finding remains.** All 20 U06 acceptance conditions, all 10 required dimensions, the 141 inherited conditions and the separately tracked 31 R06 acceptances have current direct evidence or independently validated evidence reuse. The first audit remains historical **Not Complete**; its verdict is not used as a positive completion gate.

This is a fresh pass by the same explicitly authorized implementation-nonparticipant GPT-6 Astra / High reviewer. The auditor reviewed the correction after the first audit closed and did not participate in its implementation, design a fix, change source/data/target/shared summaries, or delegate. The parent froze source and target records and kept U07 ineligible during this pass. New evidence was written only under `post_audit/u06_20261005_002`; the first audit and both frozen targets remain unchanged. The auditor did not rerun product suites: exact current executions were independently verified and the only production delta was directly reviewed.

## Authority, scope and exact target

The owner remains **GS-CBST-001 / GOAL-CBST-01 / PHASE-CBST-01** under the same P2G-30 freeze, P2G-40 plan, P2G-50 Goal Set and P2G-60 handoff. The current gex-r8 execution pipeline sections 5, 6.4, 6.7 and GEX-20 govern this pass. Accepted policy and canonical rulebook meanings are unchanged. This is a whole **single_phase** Goal Set audit, including broad regression and applicable final baseline; there is no GEX-30 that must still close the goal. The user's continuous roadmap authorization remains active.

| Identity | Audited value |
|---|---|
| Target | `core_enhanced/docs/goals/core_battle_starter_tactics/execution/single_goal/target/u06_002` |
| MANIFEST SHA-256 | `5b8f77ebe3cc5ffd16ac9f852058c652818f5ef23a6ffafa142dd8128e0b365b` |
| Product SHA-256 | `b33a8a019087f3b6766c8cf611636f46246fa3d895464b5a439f6d176694a401` |
| Combined source/data/canonical-doc SHA-256 | `1771efdf060a6f295a71e780c9d37b6730cf89cf53f876d01d03c46ba5300b2c` |
| Contextual Git HEAD | `f318414fb0156a67e04c5479000c2a2683fc0c09` |
| Membership | 3,123 distinct paths; source group392, records2593, references142, plus identified external archives and overlapping groups |
| Dirty attribution | tracked/clean37; tracked/dirty188; untracked2898 |
| Prior product | `7db70db30e290a83db502b2fc92ad08b94ef5b6a9f70dec3939dcd89e0bf976a` |
| Prior audit closure SHA-256 | `871bf0c9892e7ca0ec5b0497c36f80dc45efbed4b4b3e271217da3393b276bf6` |

[CURRENTNESS_BEGIN.json](CURRENTNESS_BEGIN.json) and [CURRENTNESS_END.json](CURRENTNESS_END.json) both have zero errors and matching identities, membership and attribution. The end check completed at `2026-10-04T19:34:46.209342+00:00`. The target is **current / unchanged** through audit closure. Git HEAD alone was not used to identify the dirty/untracked implementation.

The independent checker verified every current membership byte/size, membership-group union, product and execution-record enumeration, target archive/patch hashes, and Git attribution. It checked every member of SOURCE_BEFORE/SOURCE_AFTER/RECORDS/REFERENCES and independently performed exact forward and reverse restoration with patch fuzz zero. It also member-verified all eight external U06-001/U05 archive references: U06-001 before385/after392/records2215/references72; U05 before375/after377/records2992/references166. Current references include the closed first audit and old target metadata. These are exact historical evidence, not old source bytes falsely claimed to equal the new product.

The cumulative U06 change remains75 modified/7 added source/canonical paths from the original baseline. The correction itself changes exactly **three product files**: production `basic_heal.rs`, tests inside `targeting.rs`, and test-only `world/tests/u06_starter.rs`. The query production prefix is byte-identical. No product path was added or removed. The sole other source-group change since target001 is the ROADMAP progress preamble; its section0 onward is byte-identical. All other canonical policy documents, data/schema/layout/serializer/host sources and unrelated dirty work are preserved.

[DELTA_REUSE_VERIFICATION.json](DELTA_REUSE_VERIFICATION.json) records these independent comparisons. The auditor recomputed the correction unified diff from archived target001 bytes and current bytes; it exactly equals PRODUCT_DELTA.patch, SHA-256 `03815f9c64928361a889adf48f14bc47794c67f83d32568f83958847b97a95a4`. The 20 accepted policy objects, 10 dimension mappings, all141 inherited requirement meanings and all31 original R06 acceptance texts/IVA links remain exact. No old selected regression selector was removed.

## U06-F01 closure and direct source review

The frozen policy requires useful HP recovery and excludes ineffective targets at action start and release. The first target admitted an ally with missing HP even when the authored/current amount was zero; repeated automatic starts and public eligible-target output violated that meaning.

Current `core_enhanced/src/game/battle/core/basic_heal.rs:18` contains one pure `effective_basic_heal_amount` helper. It extracts the original arithmetic unchanged: unsigned flat plus current spell power times percent divided by100, with the same saturating addition and signed-range clamp. It returns no usable amount for Damage or a zero result. The shared `is_basic_heal_target` now requires that positive current amount before admitting a target. `resolve_basic_heal` uses the same calculation at release. There is no new stored field, allocation, queue carrier, credential, serializer, global action gate, data coercion or cap change.

The unchanged automatic selector and public `basic_action_snapshot` both consume that authoritative predicate. Start does not create an action for a currently ineffective Heal. A valid start whose current amount becomes zero before release follows the existing miss/failure transition, clears windup and closes the action/delivery; it does not choose a new target. Positive Heal keeps its former amount, footprint and damage-free semantics. The multiplication of two u32 values fits u64; the existing saturation/clamp behavior is preserved at the upper boundary.

The auditor read the whole changed production file, its start/selection/query callers, both added native tests, and the added normal-starter public consumer test. The latter modifies authored equipment coefficients in the test data before normal grant/deployment and explicitly injects injury/current spell power; it does not replace the runtime basic action after deployment. Actual strict RON acceptance is separately exercised by the fresh release reader.

[F01_EVIDENCE_VERIFICATION.json](F01_EVIDENCE_VERIFICATION.json) independently checks the raw observations, hashes and successful executions:

| Evidence | Result and meaning |
|---|---|
| Exact prior auditor probe | Its three input files and lock are byte-identical. Both formerly ineffective cases now show0 starts/0 positive HP events/HP50. The positive control still shows4 starts/4 positive HP events/HP90. The auditor independently counted the raw events, not just the summary. |
| Native single and area tests | All-zero coefficients, percent100/SP0 and percent1/SP99 rounding-to-zero yield no candidate, query selection, queued action or start. Stat gain makes the same valid definition useful. |
| Release/current-stat/clone tests | Losing spell power during windup yields a miss and no positive HP event; a controlled clone with SP40 heals20. The locked target is preserved even when another ally becomes needier; actions and windup clear and damage loss remains0. |
| Real CoreApplication consumer | Eight normal grant/equipment/deployment/focus/reconnect/query/clock cases cover single and area Heal. Six ineffective cases remain HP50 with empty eligible/selected targets; two positive controls reachHP90. |
| Fresh public strict reader | All11 cases pass their expected outcome: four accepted packs, including zero single/area and stat-dependent Heal, and seven precise negative cases. All297 recorded input files and raw stdout/stderr hashes match. |

The fresh release `author_content` binary SHA-256 is `c86f239a7df3900126f2fd9a718068c615c97a0205ab472e5c1dd3d18c700c3b`; its bound build also builds `package_official_content`. Zero coefficients remain schema-valid, which is consistent with the frozen contract: ineffective automatic behavior is suppressed at the shared current-state owner. Strict negative integer, unknown field, unsupported delivery, invalid active references/compatibility and reduction limits retain their explicit errors.

Existing fullHP/target-loss, deterministic ratio/distance/ID selection, no retargeting, pure Heal/no damage proc, Silence, common CC/stance/work/Pause, ordering and overlay tests all reran on final009. These provide the unchanged shared-gate context; the audit does not claim a separate execution for every possible Cartesian combination. No new gap or regression was found. **F01 is closed on target002.**

## Whole-goal condition and dimension judgments

[CONDITION_JUDGMENTS.json](CONDITION_JUDGMENTS.json) includes each exact policy revision/requirement, current selector and direct-receipt locators, prior semantic-review locator, and this auditor's current judgment. It also lists all141 inherited conditions and31 separate R06 acceptances. Prior passing semantic judgments are reused only after exact delta/meaning/evidence checks. Prior F01 failures are replaced by direct current closure evidence, not by relabeling the first verdict.

| Seam | Current judgment and applicable evidence |
|---|---|
| V01 TP | Pass: common resource owner, ceiling/cost, battle-time recovery and atomic shortage; current final009 selectors. |
| V02 deployment | Pass: weaponless/condition priority, normal roster10/deployed8, ninth refusal and freed-slot reuse. |
| V03 withdrawal | Pass: CC/work eligibility, fixed redeploy HP, fresh deployment resonance and pending/released delivery lifetimes. |
| V04 profile/schema | Pass: strict current reader, authored weapon consumer, compatibility and special-gun/default-Damage meaning. |
| V05 basic Heal | Pass: F01 corrected at start/release with independent raw reproduction and single/area negative/positive/current-stat evidence. |
| V06 starters | Pass: normal six-set grant/recruit/transfer and all active outcomes rerun; official data unchanged. |
| V07 ordinary stance | Pass: new-action gating preserves previously started releases; refusal and Ground/Platform effects. |
| V08 Rest/CC | Pass: full intervals, tile-kind reset/facing preservation, pause/CC/lethal order. |
| V09 relocation | Pass: validation before commit, independent mobility gate and identity/resource/delivery preservation. |
| V10 forced Defense | Pass:1499/1500ms, stale/cancel/terminal cleanup and immediate work exceptions. |
| V11 swap | Pass: both sides before commit, one TP5, each timer, read-only preview and atomic refusals. |
| V12 work | Pass: object-owned progress, recomputed eligible contribution, one completion and fullHP/CC/Pause/exit behavior. |
| V13 blocking | Pass: swept contact and continuous position, stable prior links, capacity/range/CC/death/flying invalidation. |
| V14 block range | Pass: basic-Damage relation only, generic skill/area range unchanged, allied Heal selection separate. |
| V15 query/preview | Pass: public query now agrees with corrected Core usefulness; prior preview/revision/cost atomicity remains current. |
| V16 focus | Pass: single restore owner, manual pause and stale/terminal cleanup through real commands. |
| V17 pause/settings | Pass: layered freeze/restore, command order, host-time distinction and explicit resume. |
| V18 persistence/lifecycle | Pass: final009 clone/roundtrip/rollback/reconnect plus applicable exact public Continue/rewind/WebSocket evidence. |
| V19 evidence/cost | Pass:15 fresh and30 precisely classified reused executions, unchanged caps/oracles and current changed-consumer evidence. |
| V20 closure | Pass: normal command acceptance, all rows/dimensions current, F01 closed, no unresolved finding, fresh Complete/High issued. |

All ten dimensions remain required; none is marked N/A. Success, boundary/empty/max, live-data and producer-consumer now close the former zero-recovery gap. Invalid input retains strict typed refusals while valid ineffective data stays idle. Failure/rollback includes failed-release cleanup and current clone/transaction tests. Retry/idempotency and ordering/concurrency retain duplicate/stale/CC/work/deadline/paused input evidence. Disconnect/restart/persistence uses current native/host plus exact applicable public store evidence. Legacy/fallback absence is independently confirmed by the bounded production delta and unchanged authoritative consumers. The matrix gives each dimension's concrete current rationale.

## Execution currency, predecessors and reuse

[EVIDENCE_VERIFICATION.json](EVIDENCE_VERIFICATION.json) validates **45** selected receipt/log pairs and their command-input archives against actual source identities, and confirms nonzero successful selector occurrences for all161 U06/inherited rows. It distinguishes execution source from applicability source. **15 executions are fresh on b33…;30 remain executed on7db… and are reused only after impact review.** No old receipt was relabeled as a new run.

Fresh final009 totals are Core1794 default/1794 fixtures/1803 observation, comprising lib1737/1746 and57 integration tests; host92 default/100 observation. The observation heap test ignored by the ordinary suite is separately executed:1 pass,1,024,512 samples, held781,267 at both ends and idle766,515 within the inherited bound. Current fmt/check/event-log and both clippy checks pass with the same two existing allowances. Fresh specific public-consumer, exact reproduction, release build and schema receipts complete the affected-path evidence. The earlier focused test predating the public-test edit is historical; final009 binds the selected current selectors.

The immediate U05 predecessor remains its exact independent `u05_20261004_002` Complete/High record and archived target. The first U06 audit had already independently reviewed original meanings and affected consumers; its immutable condition judgments are valid prior evidence on unaffected paths. This pass compares all original meanings and reruns their current selector bindings on final009. Original R06 texts/IVA links are also compared to the original R06 coverage, not only to the immediate U06 map. The correction touches no dimensional-expression/source-copy/aggregate owner, result/asset/store owner, resonance authority, boss/movement/delivery owner or observation/exporter layout.

[REUSE_JUDGMENTS.json](REUSE_JUDGMENTS.json) records a specific independent basis for every reused receipt. [DELTA_REUSE_VERIFICATION.json](DELTA_REUSE_VERIFICATION.json) verifies old receipt/log/summary/raw bytes against target001's frozen RECORDS manifest, all419 old input copies, all18 harness adaptations and all8 native layout/accounting sources. It confirms official basic-Heal definitions are exactly the positive flat8/4 definitions; unchanged official positive Heal outcomes are therefore preserved for unsigned current spell power. The original input suite predates BasicAttackAction Heal and authors none of it. General skill Heal is a different unchanged path.

The PC probe was read directly: automatic workload profiles remain default Damage with no weapon profile, and expression/adapter workloads do not invoke the changed basic-Heal owner. All576 original workload rows,36 conditions,31 successes/5 typed refusals, original numerical/gameplay oracle and accounted non-timing cost comparison remain valid. The earlier U06 layout growth remains exactly the previously reviewed40 bytes per RuntimeUnit bucket and80 bytes per initial ScenarioUnitSpawn element; this correction changes neither layout nor accounting. There is no new cost allowance, expected-value rewrite or performance rerun claim.

`final_performance.rs` uses Little Red and dummy Damage abnormalities, so its R05 reuse is supported by the actual unchanged branch/input/layout. The old raw R05 exit101 is preserved and is not described as a raw pass: only the inherited concurrent-peak warning remains,9,635,797 versus9,030,188, excess605,609; no other hard guard failed. Original instrumented latency is not a gate; the20-minute stress exclusion and shared-host timing limitations are unchanged.

**Reuse-description clarification:** the frozen EVIDENCE_REUSE text groups `public_guards` with the Little Red/dummy workload. Direct source inspection shows its actual measured paths are normal-starter/SavePoint Run snapshots, checkpoint exports, fixed event-log queries and rejected combat staging. Those measurements do not execute basic-Heal selection/release. Their data, state/layout, store/query/rejection code and input bytes are unchanged, so the existing public cost/allocator results remain valid for this correction. The audit uses this precise independent basis rather than the overbroad description. It is a resolved documentation imprecision, not an unverified baseline or completion blocker; no frozen record was edited.

Public CLI/schema/oracle/profile/boundary/API, independent-area, Continue/rewind/WebSocket and five fault-mode evidence retain their exact original measured source and meaning, with fresh native/host producer validation and fresh Heal consumers covering the correction. Unchanged13-input service plumbing retains the exact captured local-service evidence already independently verified in the first audit. No currently running or deployed service is claimed.

## Guardrails, limitations and closure

The current delta adds one allocation-free arithmetic helper used by the existing authoritative predicate and release. It introduces no fixture/probe-specific production branch, silent invalid-data normalization, catch-all recovery, duplicate consumer authority, speculative abstraction, arbitrary timer workaround, unexplained snapshot update, disabled assertion/test or parallel legacy path. The two query additions are cfg(test); the world addition is in the existing cfg(test) module. Positive arithmetic, skill/damage source credentials, queue and action lifetimes, implicit-Damage serialization, old-save compatibility and native CloneShape remain intact. All eight source-level implementation guardrails pass on the directly reviewed correction plus exact-applicable first-pass review.

Scope remains local Linux Core/CoreApplication, real host actor, stock WebSocket and process/file boundaries. Unity, deployed-service operation and individual final bosses remain outside U06. Shared-host elapsed observations are not new measurements. Old failures and the first Not Complete audit remain visible. Reusing exact unaffected evidence does not claim every unchanged historical implementation file was reread or every broad command rerun in this audit.

The ending target identity matches the beginning, no completion-blocking unknown remains, and all applicable whole-goal conditions pass. **Complete / High** closes U06 on **target/u06_002**. **U06-F01 is closed; open findings:0.** Dependent U07 may proceed through its normal entry under the user's existing continuous roadmap authorization. No additional GEX-30 or stage approval is required for this single_phase Goal Set.
