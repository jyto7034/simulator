# Battle Projection Revision Contract Completion Review

Review date: 2026-07-13

Review procedure: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

## Item Audit Table

| Item | Evidence status | Verdict | Long-term fit | Immediate correction |
| --- | --- | --- | --- | --- |
| DCA-C-01 same-seq/time projection ordering | Core commit tests, typed DTO/server ordering, actual Rust Unity fixture, and Unity revision tests prove closure | Complete | High | None |
| CPR-C-01 duplicate cursor/freshness authority | Core cursor removed, actor cursor/order proven, Unity time freshness removed, canonical docs synchronized | Complete | High | None |
| PD-01 reconnect/setup recovery | Same battle UUID/revision preserved; ordered root/setup/current-update recovery is tested | Complete | High | None |
| Required revision shape and no fallback | Root/update/resync/ack are strict; mock requires positive root revision; negative Unity test rejects missing revision | Complete | High | None |

## G01 — Battle Projection Revision Contract

Master item:

Perform one atomic Core/server/Unity `battle_state_revision` cutover so complete battle projection commits remain ordered even when event sequence and battle time do not change.

Subgoal:

`docs/goals/battle_projection_revision_contract/`

Policy source:

- Master accepted PD-01 Battle State Revision Contract.
- G01 PLAN invariants and stop conditions.
- Implemented canonical `core_unity_battle_transport_contract.md` and `unity_core_contract.md` after runtime proof.

Expected behavior:

Every successful authoritative battle projection transaction commits exactly one checked revision scoped by a fresh battle UUID. Failed, no-op, read-only, resend, and resync paths consume no revision. Unity applies ordinary updates by UUID/revision, uses event sequence only for continuity and battle time only for presentation, requests resync on a newer-revision event gap, and permits explicit same-revision resync. Reconnect/setup recovery preserves the active battle and returns immutable setup plus current revisioned state in order.

Runtime evidence:

- `src/game/world/state.rs`: `ActiveBattleSession` owns node identity separately from battle UUID, checked revision preflight/commit, and read-only explicit-cursor update projection.
- `src/game/world/combat.rs`: start/tick/playback/deploy/withdraw/skill/query/recovery paths finalize mutation before commit, treat no-ops/read paths as non-mutating, preserve failed-operation atomicity, and return immutable setup/current update for recovery.
- `src/game/world.rs` and `src/game/world/visual_novel_flow.rs`: command cursor input is explicit and battle VN transitions commit one coherent revision.
- `src/game/managers/uuid_manager.rs`: every new battle attempt receives a fresh battle-session UUID.
- `src/game/world/snapshot.rs`: root InBattle context requires the active-session invariant and exposes a required numeric revision; no connection delivery cursor is serialized.

Data evidence:

- G01 changes no live RON schema or content.
- The broad Core suite includes 18 live RON loading tests and all pass, proving the transaction/DTO cutover does not bypass or regress current live data.
- Active battle state is not a cross-version saved-run payload, so no save migration is required.

External contract evidence:

- `src/game/behavior.rs`: `LiveBattleUpdateDto.battle_state_revision` and root InBattle revision are required `u64` fields; overflow is typed.
- `../game_server/src/game/player_game_actor/messages.rs`: top-level update and nested resync serialize the typed revision.
- `../game_server/src/game/player_game_actor/state.rs`: battle command acknowledgement copies the exact resulting update revision.
- `../game_server/src/game/player_game_actor/handlers.rs`: `PlayerGameActor` owns the connection event cursor and one mailbox emission boundary; attach emits InBattle root snapshot, setup, then current update without mutating Core.
- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/BattleRuntimeState.cs`: ordinary update ordering is UUID/revision-first, sequence is continuity-only, time is presentation-only, and explicit resync has a same-revision-capable path.
- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Mock/ArkMockSnapshots.cs`: mock InBattle state explicitly authors revision 1 and the producer rejects missing/non-positive root revision instead of fabricating one.
- `../game_server/examples/generate_battle_revision_fixture.rs` exactly reproduces `/mnt/f/unity projects/ark/Assets/Tests/Fixtures/battle_revision_contract.json`; Unity consumes that checked-in typed Rust artifact.
- External canonical battle transport and Unity integration documents now describe the implemented revision, acknowledgement, cursor, reconnect, and resync semantics.

Test evidence:

- Core focused combat suite: 45 passed, including initial/increment/no-op/query/overflow/fresh UUID/failed deploy-withdraw/reconnect/VN and same-seq/time assertions.
- Server focused actor suite: 24 passed, including acknowledgement/update parity, actor-owned ordering/cursor continuity, resync non-regression, and active-battle attach recovery.
- Unity EditMode `ArkClientContractTests`: 182 passed through the open Unity 6000.3.4f1 Editor, including the actual-Rust fixture consumer, higher/equal/lower revision, gap, same-revision resync, missing revision, and UUID reset.
- Unity PlayMode `ArkClientMockRenderTests`: 49 passed through the same Editor.
- Full Core all-targets: 682 passed, 0 failed.
- Full game_server all-targets: 25 passed, 0 failed.
- Exact Rust fixture comparison, rustfmt check, Core/server diff checks, and exact G01 Unity-path diff check passed.
- Historical zero-test compile failures and the first Unity compile failure remain recorded as failed experiments and are not counted as success evidence.

Legacy/fallback audit:

- `last_pushed_event_log_seq`, `drain_battle_update_dto`, optional root revision, destructive setup-loss comment, mock `?? 1`, and time/sequence freshness comparisons are absent from current production paths.
- No serde alias, dual field, timestamp bump, fabricated event, revisionless compatibility decode, ignored relevant test, or Unity local inference was added.
- Historical completed goal documents retain old history by design; the two current canonical external documents are synchronized.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: the cutover removes two authority splits rather than layering over them. Core owns coherent projection commits and immutable event history; server connection state owns delivery cursor/order; Unity owns only last-applied revision and presentation continuity. The same seam is intentionally reusable by G06/G07 capability work without pre-implementing their policy.

Verdict:

- Complete.

Remaining risk:

- No live WebSocket-to-Editor end-to-end probe was added in G01. Current risk is bounded by actor-level exact ordering tests, typed server serialization, exact Rust fixture equality, and actual Unity consumer execution. Phase review R01 should independently cross-check this seam together with G06/G07 rather than creating a second revision authority.

## Cross-Component Review

- Core revision, server acknowledgement/update/resync mapping, and Unity last-applied revision use the same field and semantics.
- Event seq remains immutable delta continuity at every layer; no component uses it as projection freshness.
- Battle time remains simulation/presentation at every layer; no component uses it as a fallback staleness key.
- Root snapshots, command batches, scheduled ticks, reconnect, recovery, and resync no longer compete for cursor or commit ownership.
- Live RON is unaffected and broad loading remains green.
- G06/G07 retain ownership of exact capabilities and expected-revision command preconditions; G01 exposes their revision seam without inventing eligibility policy.

## Validation Classification

- Required Core, server, and Unity runners: available and passed with nonzero counts.
- Rust-to-Unity fixture: exact typed provenance and actual Unity consumption passed.
- Zero-test targets: explicitly excluded from counts.
- Initial failures: retained with cause/correction/revalidation in `EXPERIMENTS.md`.
- Compatibility/migration: none required or introduced.

## Review Conclusion

G01 satisfies every user-visible completion condition and the accepted PD-01 contract. It may be marked complete with High long-term fit and no immediate correction. G06 may begin according to the Master dependency graph; Phase 1 remains active until G06, G07, G12, and R01 complete.
