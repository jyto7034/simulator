# R03 correction001 implementation evidence map

Exact member: `(GS-CATP-P2G50-001, GOAL-CATP-01)`, `PHASE-CATP-01`, `HOF-CATP-P2G60-001`.
This is the GEX-10 evidence map, not a completion audit or a replacement policy. The authoritative conditions are the
[Goal Set](../../P2G_50_GOAL_SET_001.ko.md), [Freeze](../../P2G_30_FREEZE_001.json), and
[code review](../../CODE_REVIEW_001.ko.md). Source checkpoint is [INPUT119](evidence/INPUTS_119.json).
R01 audit004 and R02 audit003 are actual separate Complete/High results on their original target004. Their current reuse is subject to the exact correction impact proof below and an independent consumer decision. This Goal's separate fresh GEX-20 remains required after initial Not Complete/F01.

## Direct integration evidence

| Seam | Implementation and actual evidence | Interpretation |
| --- | --- | --- |
| IVA01 | [query](../../../../../src/authoring/query.rs), [CLI](../../../../../src/bin/author_content.rs); [CLI004](evidence/cli_external_004.json), [full core](evidence/correction001_full_core_001.json) `authoring::tests::{describe_uses_current_support_and_immutable_source_with_exact_output_limit, explain_preserves_actual_definition_effect_origin_and_rejects_missing_sites, explain_reads_every_published_definition_and_keeps_compiled_graph_edges_as_ids}` | Same external production validation/description/explanation and finite scenario through default-release library/CLI. Explain tests cover each published definition kind, effect origin, selected Remove and actual compiled boss graph. Descriptor distinguishes editable R01 effects from broader existing runtime vocabulary. |
| IVA02 | [diagnostic](../../../../../src/authoring/diagnostic.rs), actual loader/compiler/generated producers; [invalid002](evidence/invalid_external_002.json), CLI004 | Repeated real syntax/unknown-field/unclosed RON, manifest/contract/capability, ambiguous writer, final reference, summoned/boss compile, generated policy/custom-policy/custom-boss inputs. Public API and CLI diagnostic envelopes match; invalid authority also crosses actual trial input. No invented source coordinates or fabricated diagnostics. |
| IVA03 | [observation](../../../../../src/authoring/view.rs), [battle observer](../../../../../src/game/battle/core/authoring_observation.rs), [Buff/resource external001](evidence/buff_external_001.json), [delivery001](evidence/delivery_external_001.json), [Preview004](evidence/preview_external_004.json) | Current [combat003](evidence/combat_external_003.json) and [source-byte API001](evidence/source_receipt_api_001.json)/[CLI001](evidence/source_receipt_cli_001.json) execute externally authored effects through public preparation/deploy/clock/result commands. Every initial/ready/step now carries exact immutable generation SourceReceipt. Same-semantic A/B source bytes are distinct while all other payload/usage and gameplay match. HP/Stats/Resonance/Buff observations carry authored source and actual application identity; actual delta, initial authority/seed/clock/input/content and terminal state are retained. Prototype values are not official RON changes. |
| IVA04 | [commands](../../../../../src/game/battle/core/commands.rs), [event carrier](../../../../../src/game/battle/enums.rs), [queue](../../../../../src/game/battle/core/queue.rs), [trace](../../../../../src/authoring/trace.rs); delivery001, Preview004, [focused135](evidence/focused_135.json), full core | Repeated/delayed Instant, projectile and persistent area applications; real bounded event-log eviction and correlation shifts; actual HP/time/cast/target/ordinal/source and subsequent decisions. Off/on/gap mandatory receipts and gameplay continuation agree. Source is carried across scheduling, not recovered from retained log rows. Native delivery UUID tests remain distinct from exposed authoring occurrence identity. |
| IVA05 | query, [trial inspections](../../../../../src/authoring/trial.rs); Preview004 semantic proof with [scoped reuse](evidence/correction001_source_reuse_001.json), current [23-step cost replay](evidence/cost_external_003.json), full-core `public_trial_preview_is_bounded_and_preserves_the_accepted_session` | Nine positive readonly target queries after actual reward/equip/second combat/focus. Before/after snapshot/reconnect/revision/receipt/work match and the next manual cast causes identical sourced HP change. Missing/unsupported/cancel/low-cap queries explicitly reject. No forecast is presented as an executed effect. |
| IVA06 | [TrialSession::prepare/replace/advance](../../../../../src/authoring/trial.rs); [replacement005](evidence/replacement_external_005.json), invalid002, [Weak ownership test](evidence/focused_137.json), full core | Complete B preparation precedes replacement. Repeated valid-token preparation/domain/fuel/cancel/receipt failures preserve complete A and prior committed steps; later valid B recovers. Invalid external load does not yield a B token. Failed B generations are reclaimed while their error objects remain alive; successful replacement respects explicit A pins and last-owner drop. |
| IVA07 | [control](../../../../../src/authoring/control.rs), [scenario](../../../../../src/authoring/scenario.rs), [resolve](../../../../../src/authoring/resolve.rs), [clone shape](../../../../../src/authoring/clone_shape.rs), [output](../../../../../src/authoring/output.rs); full core, replacement005, CLI004 | Inclusive byte/node/depth/step/binding/clock/output/fuel/resource limits; actual generated setup and battle startup; same-call repeats/fan-out/queue/geometry/retained-owner admission; cancellation and sticky interruption survive inner failures. Mandatory output failure rejects the candidate; optional trace overflow only records an exact gap. Native clone coverage is detailed below. |
| IVA08 | [full core1327](evidence/correction001_full_core_001.json), [all targets](evidence/correction001_all_targets_001.json), [default release check](evidence/correction001_release_check_001.json), [three release binaries](evidence/correction001_release_build_001.json), api_external002, CLI004 | Current R01 effects/source/delivery and R02 loader/pin/V6/profile/failed-candidate regressions run in the full suite. Five standalone legacy API consumers and six original forbidden-API compiler cases pass their expected oracles. Six additional TrialSession privacy/lifetime compile negatives pass. Actual prior separate predecessor verdicts are consumed with the explicit correction-only currentness evidence, not labeled unchanged. |
| IVA09 | [cost003](evidence/cost_external_003.json), [cost summary](evidence/cost_summary_003.json), [runtime guards001](evidence/runtime_guards_001.json), [kernel guards002](evidence/kernel_guards_002.json), [experiments054–055](EXPERIMENTS.md) | Actual plain and allocator binaries, separate timing, allocation/peak/end/counter samples, four independent generations and drop. Existing public guard thresholds preserved. Original raw-fixture kernel failure is retained; the documented current-token API adaptation passes original kernel parity/latency/allocation guards, while raw-to-token conversion cost is recorded separately. |

[Correction source proof](evidence/correction001_source_reuse_001.json) verifies330 archived pre-correction bytes and318 current source inputs. Six borrowed `SourceReceipt` field/initializer lines in trial.rs are the sole production delta; removing them reconstructs the prior trial file exactly. The other source delta is the new standalone public regression consumer. Earlier [source reuse002](evidence/release_source_reuse_002.json) still owns the two cfg(test) bodies/optional-feature reconstruction from source101. Buff/invalid/delivery/Preview actual execution, provenance, query and diagnostic semantics are reused only within those unchanged producer/consumer paths. Old full receipt/work/cost values are superseded for current output, not represented as byte-identical. Current source-byte API/CLI, CLI004, combat003, replacement005 and cost003 supply the changed mandatory serialization/bounds/retention evidence. New [legacy API batch002](evidence/api_external_002.json) runs all five positive/six negative original consumers. Current full-core1327 owns all core regressions, including actual preview preservation and Weak lifetime tests.

The [initial Not Complete audit](COMPLETION_AUDIT.md) and its byte-exact reproduction remain immutable history. [Correction001](CORRECTION_001.md) owns F01 implementation and validation. R01/R02 have actual historical Complete/High reports; [predecessor impact002](PREDECESSOR_IMPACT_002.md) and the [exact changed-member evidence](evidence/correction001_predecessor_currentness_001.json) support scoped consumer currentness review. No new predecessor or R03 audit verdict is supplied by this map.

## Native bounds and startup review

The production acquisition/input limits are finite before parsing; controlled validation is installed before generated
setup. The actual external-load test `controlled_external_load_uses_the_same_token_and_exact_generated_work_boundary`
checks the observed work boundary and cancellation without changing the publication. Metadata and excessive authored
spawn expansion have direct rejection tests. Trial preparation uses one attempt over actual public commands and time-zero
battle readiness; the ready output is admitted before publication. Its late aggregate receipt rejection and clock checks
are exercised by `private_trial_preparation_uses_one_scope_keeps_none_slots_and_rejects_late_output` and
`private_trial_preparation_and_execution_enforce_actual_clock_and_final_readiness`, plus external replacement005.

`CloneShape` explicitly walks owned fields before native clone; Arc pointees are shared and not charged as copied storage.
`OwnedHashMap`/`OwnedHashSet` preserve native allocation history after churn/removal/clear, including vacant-entry reservation.
Rapier native owners keep conservative insertion/removal/pair-growth history and reset it only with those native owners.
The opaque whole-world candidate reaches this preflight before copying application/battle state. The full suite includes:

- `owned_clone_shape_complete_world_preserves_entry_checkpoint_and_retreat_continuation` and `owned_clone_shape_complete_battle_admission_precedes_native_copy_and_keeps_continuation`.
- `authoring_clone_admits_retained_history_before_native_copy_and_preserves_continuation`, `clone_history_includes_removed_population_and_pair_growth_and_resets_only_with_native_owners`, and overflow rejection.
- `owned_clone_shape_queue_all_variants_preserve_native_order_and_payloads`, actual fixed-projectile/area policies, target scans, geometry projection, cast admission before identity allocation, command-buffer rejection before effect application, and same-event repeats with unchanged retry.
- Sticky fuel/resource/cancellation scope tests, prior-result binding projection limits, exact mandatory output caps, and trace-on/off/gap accepted-work/continuation tests.

These are source-level and actual execution bounds; a finite count of outer API calls is not the safety argument. Native
requested-byte cost is measured separately from process RSS, and fixture/live-data lifetime claims stay within the
in-memory authoring host contract. No arbitrary elapsed-time deadline or new gameplay cap is introduced by this map.

## Thirteen acceptance members

The zero-based PER locators and meanings remain the Goal Set's exact members. This table identifies evidence for the
implementation handoff; the separate reviewer must decide completion, including prerequisite currentness.

| Member | Direct evidence |
| --- | --- |
| PER-CATP-E01-REVIEWED-001[0] | IVA01 + IVA03: CLI004/default public consumer; external nonzero A/B and actual sourced runtime applications. |
| PER-CATP-E01-REVIEWED-001[1] | IVA01 + IVA02: actual support registry and explicit unsupported operations/queries and privacy rejection; common dispatch. |
| PER-CATP-E02-REVIEWED-001[0] | IVA01 + IVA02: complete API/CLI diagnostic-envelope equality on the same real invalid inputs. |
| PER-CATP-E02-REVIEWED-001[1] | IVA02 + IVA06 + IVA07: real producer source/related operands, failed load/preparation/late output, A retained and no false publication. |
| PER-CATP-E03-REVIEWED-001[0] | IVA03: external effect values affect nonzero actual HP/resource observations and result consumption. |
| PER-CATP-E03-REVIEWED-001[1] | IVA04 + IVA07: same basis/input/clock replay; off/on/gap equality; actual eviction/repeated/delayed semantics and next decision. |
| PER-CATP-E04-REVIEWED-001[0] | IVA04 + IVA05: positive and negative readonly queries; before/after state and the next actual cast/clock. |
| PER-CATP-E04-REVIEWED-001[1] | IVA01 + IVA05: exact resolved definition/effect/origin/compiled graph; actual bounded observations with explicit optional gaps. |
| PER-CATP-E05-REVIEWED-001[0] | IVA06: actual prepared B atomically replaces A and its input/provenance/receipt; source-byte API001 additionally binds all B documents to the captured B source. |
| PER-CATP-E05-REVIEWED-001[1] | IVA06 + IVA07: repeated failures/cancellation/output cap preserve complete A, followed by valid B recovery. |
| PER-CATP-E06-REVIEWED-001[0] | IVA02 + IVA06 + IVA07: empty/minimum/exact cap/cap+1/overflow/damaged inputs and sticky same-call failures without replacing A. |
| PER-CATP-E06-REVIEWED-001[1] | IVA06 + IVA08 + IVA09: actual Weak/drop and measured retention; current regression/API proofs; predecessor currentness must be independently established. |
| PER-CATP-E06-REVIEWED-001[2] | IVA03 + IVA08 + IVA09: source/fixture/binary/command/count/output records, historical failures and bounded non-impact reuse; independent fresh R03 audit still required. |

## Ten validation dimensions

All ten dimensions are required. `success` uses IVA01/03/05/06; `invalid_input` uses IVA02/07;
`failure_rollback` uses IVA02/06/07; `retry_idempotency` uses IVA04/05/06; `ordering_concurrency` uses IVA03/04/06/07;
`boundary_empty_max` uses IVA02/07/09; `disconnect_restart_persistence` uses IVA04/06/08;
`live_data` uses IVA01/02/03/06; `producer_consumer` uses all nine seams; `legacy_fallback_absence` uses IVA01/02/07/08.
Disposable in-memory trial loss at process exit, explicit same-basis replay, readonly reconnect and current V6/profile/pin
regressions are the persistence boundary. This does not claim save/restore of a live TrialSession or parallel mutation of it.

## Scope and structural disposition

There is one production loader/compiler/runtime and a common typed authoring facade. The CLI dispatches that facade.
Trial state is opaque, non-Clone and non-serde; borrowed receipt views cannot survive mutable advance. No raw fixture builder
is exposed in default release. Source lookup/trace is observation, not a second evaluator. Optional trace has an independent
quota and does not determine gameplay/RNG identity. Mandatory receipts contain actual outcomes and never turn finite input
completion into an invented battle victory. New owned state has bounded admission, cancellation/failure disposition and drop.

Official RON, personal stores, deployed server/Unity, UI, live schema migrations, later R06/R13 composition/execution-plan
features and R22 independent authoring/R24 final audit are not delivered by this R03 member. These exclusions do not remove
any of the required R03 seams or the full R00–R24 objective. Current canonical contracts are
[core runtime](../../../../core_runtime_contract.md) and [data loading](../../../../data_loading_contract.md).

Fresh predecessor input also includes [20 actual original release commands](evidence/predecessor_external_001.json) and [delta/expiry disposition](PREDECESSOR_IMPACT_002.md). Old missing `/tmp` actual is historical, not current.
