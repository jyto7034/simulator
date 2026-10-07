# R03 trial lifecycle witness — bounded source design 021

Status: read-only design assistance under AUTORUN §8, not implementation, completion audit, or test execution. Root remains the sole production/artifact writer. This report proposes a small lifecycle producer/consumer connection for the currently private TrialState; it does not design optional trace or semantic effect occurrences. Frozen Goal C05/I03 requires actual B preparation before replacing A, and C06/I04 keeps optional trace independent of mandatory execution/admission (Goal Set:54–72,96–104).

## 1. Recommended owner and exact current boundaries

Add an optional, fixed-size private lifecycle sidecar to **GameCore**, enabled only by a new private fresh-authoring constructor. GameCore presently derives Clone and owns exactly state, game_data, run_seed; its exhaustive CloneShape lists those fields (world.rs:50–61). CoreApplication::fork_for_authoring controlled-clones that entire GameCore (application.rs:786–800). A Copy-only sidecar adds no deep allocation, definition owner, history Vec, or gameplay RNG draw. Ordinary CoreApplication::new leaves the sidecar disabled. This preserves ordinary gameplay semantics; the new field/branch still has a measurable size/copy cost, not proven zero overhead.

A GameCoreState field would naturally travel in PreparedApplicationCommit.state, but would also require modifying GameCoreState::new, its exhaustive visitor, clone_non_battle_transaction_state's complete literal, all state replacement/capture semantics, and explicit exclusions from BattleEntrySnapshot/RunCheckpointPayload. Putting observation under checkpoint/game-state ownership invites accidental time/entry rewind. GameCore ownership makes the intended separation explicit: **gameplay rewind preserves accepted authoring observation, while failed transactional candidates rewind both**.

Required GameCore propagation is small but explicit:

| Boundary | Actual current behavior | Required sidecar treatment |
|---|---|---|
| GameCore::new / private authoring new | fresh GameCoreState; no sidecar currently | ordinary None; fresh authoring Some(empty witness), before any real operation |
| derived Clone / GameCore CloneShape | complete GameCore fork | include sidecar in exhaustive field visitor; Copy-only fields/variants checked by the compiler |
| execute_application | controlled whole draft; operation; validation/polls | witness mutates only draft, and errors discard it |
| PreparedApplicationCommit / commit_application | explicitly moves state, seed, token, output, route | add witness to prepared literal, destructuring, final assignment |
| TrialState::prepare_step / advance | private application fork; final receipt; both Result layers; whole TrialState swap | consume copied witness before receipt; successful application already owns matching witness |
| with_state_transaction | captures nonbattle state, restores only state on Err | capture the fixed sidecar beside state before operation; restore it on Err too |

Sources: world/application.rs:128–134,172–220; node_flow.rs:784–803; trial.rs:95–170,173–216,291–364. The private TrialState baseline already flattens outer and inner results and replaces the complete owner only after run_with_provenance's final poll. Its missing witness is the implementation slice requested here, not evidence that the existing private step boundary is defective.

A private CoreApplication::new_for_authoring(token, seed) and copied lifecycle accessor are sufficient outward seams. They must not be a public setter that enables observation on an arbitrary imported core, a public raw core borrow, or a new fixture authority. TrialState calls the fresh constructor inside its already-installed fresh-provenance scope (trial.rs:103–114). Accessor absence means disabled/unavailable; it must not synthesize a ready/default-success witness. An internal scope assertion/Result check may enforce the sole constructor call contract without changing ordinary new.

## 2. Fixed witness facts, not inferred readiness

One compact private representation can contain: checked local next-entry ordinal; accepted cumulative actual elapsed milliseconds; and an optional latest EntryWitness. An EntryWitness holds a producer-assigned ordinal, actual battle_uuid, actual startup/last-observed execution time, optional explicit TimeZeroCompletion, optional TerminalEvidence, and optional explicit ExitReason. These are private constructor-controlled facts, not mutable caller DTOs. TerminalEvidence needs only fixed time plus the actual BattleVerdict/BattleWinner for this slice.

BattleVerdict and BattleWinner are Copy enums; BattleOutcomeId is an owned String (battle/types.rs:25–80). Do not silently clone that ID into a supposedly fixed witness. Detailed outcome/source identity can remain in the existing result or later separately admitted capsule. The fixed witness is not a replacement for that future source consumer.

A local entry ordinal is technically necessary if new-entry identity must survive normal rewind. commit_planned_live_battle_start obtains battle_uuid from UuidManager (combat.rs:683); BattleEntrySnapshot::restore restores the whole uuid_manager (state.rs:579–594), while checkpoint restore preserves only the VN namespace (state.rs:460–463). The deterministic battle namespace can therefore repeat after retreat/checkpoint rewind (uuid_manager.rs:38–42,90–92). Increment this ordinal with checked_add **only at the actual entry producer**, and roll it back on candidate failure. It is not a readiness counter, fabricated gameplay identity, effect-occurrence ID, or log sequence. Readiness requires the matching explicit successful TimeZeroCompletion and no subsequent exit; an immediate real terminal witness is distinct from an exit.

Keep cumulative elapsed/entry history monotonic across **accepted** retreat, checkpoint load, and new-run transitions within the same tool session. Do not reset the cumulative budget on those gameplay rewinds. A brand-new replacement B gets a fresh witness; failed B is dropped without modifying A. The most recent entry may replace the previous fixed record only after the previous accepted receipt has retained its mandatory facts; the existing bounded receipt prefix owns historical records.

## 3. Three actual producer hooks and their order

**Begin/complete entry — combat.rs:636–689.** Direct callers are handle_explicit_combat_node (423) and the successful event choice path (event_node.rs:368–383). The function assigns the real battle UUID, installs planned.active_battle, transitions to InBattle, then calls advance_active_battle_by_internal(0, true, 0). Add an entry-begin hook once that actual candidate battle identity/execution state exists, before the advance. It records the true startup time, checks expected zero for a time-zero claim, and allocates the checked local ordinal. Change the tail to retain the Result: run the actual zero-time advance; on success complete that same entry's TimeZeroCompletion; then return the actual result. Do not mark success merely at active installation, constructor success, BattleStart log emission, or setup serialization.

**Observe time — combat.rs:1454–1568.** advance_active_battle_staged reads previous_battle_time_ms before calling the real BattleCore step (1479–1486), obtains current_battle_time_ms after finalization (1504), and keeps both beyond the active borrow (1515–1522). This is the minimal arithmetic hook: validate matching live entry/battle UUID and continuity, calculate current.checked_sub(previous), then total.checked_add(delta), and update the candidate witness. An unexpected backward clock, ordinal mismatch, arithmetic overflow, or unavailable enabled witness must return an explicit failure from the existing Result seam; never saturate, default to zero, or create success. Observe each actual step exactly once. Terminal handling below must not add the same elapsed delta again.

**Terminal before active removal — combat.rs:1393–1406.** transition_finished_live_battle_to_result still has the live active battle and receives the finalized CombatBattleState. Before consuming/removing it, record the matching real terminal execution time and the actual verdict/winner. The direct production caller is advance_active_battle_staged:1566, after finalize_battle_execution:1492–1501 and all battle result/VN projection work. A strict enabled-witness hook can validate active UUID, matching last-observed time, and execution.is_finished(); it must not depend on the debug-only execution.terminal_outcome()/is_finalized() accessors (sim.rs:69–86). There is no need to alter ordinary battle result semantics or manufacture Draw.

The terminal hook may run while this entry's time-zero completion is still pending. That is legitimate: zero-time stepping can finalize/remove active_battle, return successfully, and only then let the enclosing entry function confirm time-zero completion. The fixed terminal record must survive active removal so entry completion never needs to reread a nonexistent active_battle. If anything later fails, the containing world/application draft, then the outer authoring candidate, is rejected.

Kernel startup alone is not readiness: start_battle_execution returns time=0 and battle_start_hooks_ran=false after enqueueing (sim.rs:1819–1857). Kernel step/finalize are fallible (1665–1718); finish_signal records the actual finish time/outcome (1324–1337). This production ordering explains why an “in battle” boolean or a zero-valued clock cannot stand in for completed preparation.

## 4. Complete exit, rewind, clone, and rollback map

These are required owner dispositions, not additional gameplay rules:

| Producer/owner | Proposed lifecycle disposition |
|---|---|
| Time-zero/domain/control/mandatory receipt failure | discard pending application + witness; preserve A's witness/ledger/input/receipt. No committed aborted entry is fabricated. |
| handle_retreat_battle, combat.rs:2794–2832 | capture entry identity/time before active.take and record explicit Retreated; preserve cumulative elapsed. Snapshot.restore restores gameplay/UUID but must not restore old witness facts. Hook failure must propagate through outer draft. |
| handle_complete_combat_result, 2874–2876 | close the completed entry as ResultConsumed only after actual inner completion succeeds, inside the with_state_transaction operation so failures restore both owners. Preserve the terminal evidence already produced. |
| return_to_node_confirm_for_combat_retry, 2835–2871 | explicitly close eligibility for the previous entry (RetryPrepared reason if distinguished); next actual commit_planned start creates a new ordinal, possibly with the same gameplay UUID. Do not count this map preview as another entry. |
| fail_run_with_reasons / commit_run_failure, 1259–1295 | record RunFailed exit before clearing active context; retain existing terminal evidence, do not invent terminal evidence for a run failure without a finalized battle. Their shared path must not double-count/rewrite a prior exit. |
| handle_start_new_game, world.rs:720–749 | explicitly close prior entry eligibility before resetting run/active fields; preserve tool cumulative counters. No new entry until the real combat producer. |
| handle_abandon_run, world.rs:762–772 | record Abandoned before GameCoreState replacement; preserve monotonic tool counters and prior terminal facts. Do not rely on current UI capability to make the clear unreachable. |
| restore_run_checkpoint_payload, node_flow.rs:1281–1287 | record CheckpointRestored outside the serialized checkpoint; do not restore witness/elapsed/ordinal from RunCheckpointPayload. Success has no active battle. |
| continue_from_active_run_checkpoint_draft, active_run.rs:263–291 | reaches restore_run_checkpoint_payload; ordinary constructor has observation disabled. Fresh-only TrialState does not gain an imported-core/host recovery support promise. |
| ActiveRunTransactionRollback, active_run.rs:20–25,170–216 | cfg(test) currently captures state/seed/token only; if exercising observed roots, capture and restore the fixed witness too. This is transaction rollback, unlike an accepted gameplay checkpoint rewind. |
| with_staged_tactical_world_mutation, combat.rs:1822–1854 | clones/replaces only ActiveBattleSession and may restore original on VN projection failure. The new GameCore lifecycle hooks do not run inside its ActiveBattleSession-only callback; preserve witness unchanged. Any future producer added there must join this rollback explicitly. |
| mastered sweep pristine state replacement, mastered_elite_sweep.rs:211–329 | reward/node staging is not a battle entry/time producer. Preserve the GameCore-side witness unchanged across self.state = pristine; do not mark sweep readiness from its combat-shaped result. |

with_state_transaction is especially important: ConfirmEnterNode enters it from **nonbattle** state and can start/time-zero a battle before a later error (node_flow.rs:604–658). State-only rollback would otherwise leave an entry witness for a battle that no longer exists. Copy the fixed witness before invoking the operation and restore on the Err branch. This is required even though the current public outer transaction also rejects the draft; it preserves the real inner owner's stated rollback semantics and future error composition.

BattleEntrySnapshot and RunCheckpointPayload should intentionally contain **no** witness (state.rs:388–470,541–595). Their native capture/restore fields remain gameplay owners, while with_state_transaction pairs a separate sidecar snapshot with its state snapshot. Ordinary controlled Clone remains a complete candidate copy, not a new observation session. The sidecar has no serialization into V6/profile and no generation/DB Arc, so it introduces no durable migration or retained content owner. Later source capsules will change that ownership analysis and must be separately admitted.

No new runtime “abort outcome” is required. Cancellation/failed trial preparation is an unaccepted candidate drop; an explicit accepted gameplay exit uses its actual producer reason. ResetActiveRun and ContinueRun PlayerBehavior currently return InvalidAction in core's map/run dispatcher (world.rs:429–432); do not invent supported reset/continue behavior in TrialState.

## 5. Actual elapsed admission and the private TrialState consumer

ScenarioLimits bounds requested raw delta and requested sum; NextEventBucket contributes zero to that static sum (scenario.rs:520–534). That is insufficient as actual elapsed proof. Host tick applies playback ratio/remainder and may yield no advance for pause/fractional time (state.rs:827–840); blocking VN returns None earlier (combat.rs:1598–1616). A None tick must not add time or erase the prior witness. Clock requests can finish before their target, and NextEventBucket can jump directly to its next real event (sim.rs:1868–1883,1969–2003). Count observed execution delta, not requested time, result JSON time, wall time, or an assumed multiplier.

After any real public command/clock in prepare_step, obtain the copied witness and compare its cumulative total with the scenario's checked max_total_clock_ms before final receipt. Compare candidate/baseline totals with checked arithmetic; every public step may be the actual entry/clock producer, so do not restrict this check to the Clock syntax variant. The same facts should be a borrowed/fixed receipt projection before encode_receipt. Failed cap admission rejects the outer authoring candidate even if the internal public application already committed its local draft. Arithmetic overflow inside the world hook must also be fallible and discard that candidate.

Per-request max_clock_delta_ms remains the existing raw-request grammar bound unless the owner explicitly connects an additional actual-per-step check. This report does not silently change its unit/meaning. The cumulative actual bound is distinct and necessary. Post-execution actual-time rejection is atomic admission, not a guarantee that zero work was spent: existing opt-in work fuel remains responsible for promptly bounding the step's internal work. Do not forecast a battle's actual terminal time to avoid measuring it.

The preparation boundary consumes explicit time-zero/terminal/exit facts, not game-state matching. A successful immediate terminal is provable; a later Retreated/ResultConsumed/Abandoned/CheckpointRestored entry is historical evidence and cannot masquerade as a currently prepared entry. The receipt/history can still explain that past trial. A never-entered or failed-time-zero B must not replace A.

## 6. Mandatory future capsule constraints and focused oracles

This witness does not satisfy full R03 source observation by itself. finish_battle adds the final BattleEnd record, and only debug/test/fixture configurations clone its event log into BattleResult (sim.rs:1299–1321); CombatBattleState likewise retains the log only under those flags (combat.rs:1380–1381). BattleEventLogEntry.skill_effect_source is serde(skip) (event_log.rs:820–832). A future source capsule must therefore consume actual retained sources while active battle still exists, after final event production but before removal; it cannot reconstruct them from the serialized result or this fixed terminal witness. Earlier eviction requires producer-time retention/gap handling, outside this slice.

That capsule must follow the same whole candidate clone/rollback/commit and generation lifetime, without mutable Arc sharing between A/B. Optional trace overflow remains gap-only and must not affect this mandatory witness, its admission/work/receipt, or gameplay/RNG. Entry ordinal/log correlation must not become a substitute for actual effect-occurrence identity.

Proposed tests, **not executed here**: ordinary constructor disabled versus private fresh initialization; real time-zero running and immediate terminal; entry followed by late VN/projection/receipt/control failure preserves A; with_state_transaction rollback removes the candidate witness; terminal survives active removal; retreat/reentry reuses gameplay UUID but advances true producer ordinal; checkpoint rewind preserves accepted elapsed; result consumption/reset clears readiness without erasing history; paused/blocking/fractional ticks add zero, scaled ticks and bucket jumps add actual time; terminal-before-target adds only consumed time; checked arithmetic/cap overflow rejects the whole candidate; clone and PreparedApplicationCommit preserve exact witness; trace-on/off share mandatory witness and continuation. Native copies, normal-path overhead, and cost limits still require root's actual measurement.

## 7. Source method, hashes, and drift

Read method: rg for bounded symbol discovery; Python Path.read_bytes + numbered UTF-8 slices; SHA-256 calculated from the exact complete bytes furnishing each slice. EOF-clipped ranges are normalized in JSON. Searches that named nonexistent run_recovery.rs/old Goal paths returned errors and were replaced by actual path discovery; they are not evidence of those files' existence. No repository writes, Git commands, builds/tests, runtime commands, or completion verdict were made. Root-reported test outcomes were not this helper's executions.

29 source-read snapshots, 20 distinct files. Close rehash 2026-09-10T11:20:22.119655+00:00. Only trial.rs drifted: inspected 2603eb5c… → close 13762396… (same byte count). The close variant was rehashed only and is not reviewed here; all TrialState claims cite its inspected baseline. Other listed hashes were unchanged through close.

1. `core_enhanced/src/game/world.rs` — spans 45–115, 695–751; 759–779, 269–385; 418–464; SHA-256 `496b24ed05866ffc88baebaa0f7076ad46569af7b3386b7727d43bc922b72a91`.
2. `core_enhanced/src/game/world/application.rs` — spans 1–220; SHA-256 `a13098c116e6db6f2e19f390306b8b8e8915d9b3d03e4d7a425675cef404731b`.
3. `core_enhanced/src/game/world/combat.rs` — spans 400–432, 625–693, 1210–1420, 1421–1602, 2780–2960; 1598–1648; 1780–1861; SHA-256 `c8e2c36f80a7c1f3a407fb661cc42a535b61512a1997e21d0de078a142c8d46b`.
4. `core_enhanced/src/game/world/state.rs` — spans 132–229, 386–478, 521–615, 1810–1940; 815–842; SHA-256 `05c7ce0fbd02258e4f34f611e059f0bb8eb6d738947baeced9b425f8f9a11336`.
5. `core_enhanced/src/authoring/trial.rs` — spans 1–325; 317–382; SHA-256 `2603eb5c2a05b141529f2c570f309b50bfa089771f430a3be959c1c62d474faa`.
6. `core_enhanced/src/application.rs` — spans 775–803, 829–890; SHA-256 `6e77a86f121fd092c83b68a6021e86a99c2d24d657fd8a7d18247f5c81e54f8e`.
7. `core_enhanced/src/game/world/active_run.rs` — spans 1–44, 165–260, 260–293; SHA-256 `e5ad247acbcf9d4f9939712f426c1305acc87989ad3a11e9c53dc0dc32aeadcc`.
8. `core_enhanced/src/game/world/node_flow.rs` — spans 775–812, 1240–1310; 600–661; SHA-256 `7857bb8a1363f928b37a8e168274ee724537fef12b69d8265e5d5a5069075445`.
9. `core_enhanced/src/game/world/event_node.rs` — spans 329–390; SHA-256 `9ad3df59233e9f30989a14c29ac6a42edc35d1a39024cdd4938475c148f03f7a`.
10. `core_enhanced/src/game/battle/core/sim.rs` — spans 38–99, 1660–1736, 1815–1888, 1958–2020; 1295–1347; SHA-256 `9e3a3ef70179d302be10b8d6f0ca7873885cd448ac629e893505f6856b35f5b4`.
11. `core_enhanced/docs/roadmaps/content_authoring/AUTORUN_REQUEST.ko.md` — spans 110–140; SHA-256 `3e05b131d445cd10ecf478ef959b80f55ce718c41c8a1820e5a5b7d969f9a252`.
12. `core_enhanced/docs/goals/content_authoring_tool_ports/P2G_50_GOAL_SET_001.ko.md` — spans 54–72, 96–104; SHA-256 `bb0b69cfebceb17fd13374826e1dbb6ba6e38e711cb574acc29ca61ca2745f98`.
13. `core_enhanced/src/game/managers/uuid_manager.rs` — spans 1–103; SHA-256 `de6c39ffbd6232ab223e21a782a2495e1775b85498a23a237b6781ac4807cc9d`.
14. `core_enhanced/src/game/battle/types.rs` — spans 18–91; SHA-256 `4e8c235f337de94ad59c10f6927d1769e58d6416e8d8d63b785ffaddf328fbea`.
15. `core_enhanced/src/game/world/action_capability.rs` — spans 136–169, 249–267; SHA-256 `10dab9e64a4e5d93a0a0e7d79b9f5421678bdc1985df812ed3f6f8f6981bf2dd`.
16. `core_enhanced/src/authoring/control.rs` — spans 193–255, 308–359; SHA-256 `1a15e8bdd7618572950ddc69ad25b5cda674eebfecd0f544010abb3477b2fed8`.
17. `core_enhanced/src/authoring/scenario.rs` — spans 15–55, 437–455, 520–545; SHA-256 `0152fd9a1f33ad5d061102da34b5bfc13f8e282b697f4e0030dcb741320da859`.
18. `core_enhanced/src/game/battle/event_log.rs` — spans 814–832; SHA-256 `49ddbf7545b8939ab4ae2b3cd42a87e1b42fbe97c7dbdfec67201daf33b63619`.
19. `core_enhanced/src/game/world/helpers.rs` — spans 544–580; SHA-256 `ead4e7e10d7dbf4471d16d2bae5888d3004bfc4a28fc9d5eb6634db8efe4dc22`.
20. `core_enhanced/src/game/world/mastered_elite_sweep.rs` — spans 285–332; 211–285; SHA-256 `68b5c5417f68a93c3812b16525c9da7eb2ec6af8ab4f1055e14caa5ceefabed6`.
