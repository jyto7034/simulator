# Server RunStart Fixture Contract Completion Review

Review date: 2026-07-12

Review procedure: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none for G04; G12 follow-up for broader startup cardinality validation

## Item Audit Table

| Item | Evidence status | Verdict | Long-term fit | Immediate correction |
| --- | --- | --- | --- | --- |
| DCA-H-08 server RunStart fixture drift | Runtime/data/fixture/tests prove closure | Complete | High | None |
| CPR-M-01 fixture-gate portion | Six blocked assertions restored | Complete | High | None |
| CPR-M-01 early supported-mode cardinality portion | Deliberately outside G04; must remain owned by G12 | Follow-up required, not claimed complete here | High ownership fit | No G04 correction |
| Phase 0 baseline evidence | Core 681/681 and server 25/25 reproducible; Unity runner and Rust serialization path identified | Complete | High | None |

## G04 — Server RunStart Fixture Contract

Master item:

Restore the intended server fixture and regression baseline without weakening assertions, while recording trustworthy Core/server/Unity evidence gates.

Subgoal:

`docs/goals/server_runstart_fixture_contract/`

Policy source:

- Master G04 and Phase 0 contract.
- Existing production Standard RunStart behavior and live trigger RON.
- No new gameplay or wire policy was introduced.

Expected behavior:

A server test fixture that represents a startable game supplies both the pre-run Visual Novel definitions and matching RunStart triggers. Standard start reaches the briefing and the six server tests reach their original reconnect, tick, resync, snapshot, and ordering assertions. Missing triggers remain an error.

Runtime evidence:

- `src/game/world.rs`: Standard start resolves the matching RunStart trigger and rejects absence with typed `InvalidStaticData`; this path was not modified.
- `src/game/data/mod.rs`: `GameDataBuilder::empty()` still produces an empty trigger database, while the public builder method accepts an explicit typed trigger database.
- `../game_server/src/game/player_game_actor/handlers.rs:440`: the server starter builder now destructures one paired Visual Novel/trigger fixture.
- `../game_server/src/game/player_game_actor/handlers.rs:450`: the paired fixture constructs both Standard and Endless definitions and their matching triggers from shared local IDs.
- `../game_server/src/game/player_game_actor/handlers.rs:491`: one typed trigger constructor fixes repeat, checkpoint, priority, and blocking semantics to the live contract.

Data evidence:

- `../game_resources/data/visual_novel_triggers/base.ron`: production has one Standard and one Endless RunStart trigger, each `OncePerRun`, `PersistAcrossCheckpointLoad`, priority 100, and blocking.
- `../game_resources/data/visual_novels/base.ron`: both referenced briefing definitions exist.
- No live RON or schema was changed by G04.

External contract evidence:

- No Unity-facing DTO, command, or WebSocket shape changed.
- Unity Test Framework 1.6.0 and `ArkClient.EditModeTests`/`ArkClient.PlayModeTests` are present.
- Project version and installed Editor both resolve to 6000.3.4f1. Unity was not run because the child changes no Unity surface and executing it would mutate the external project cache/log state.
- `player_state_snapshot_preserves_core_snapshot_root_fields` obtains a typed Rust Core snapshot, serializes it, wraps it in the typed server snapshot, serializes again, and passed in the broad server suite. This is the required real Rust serialization path, not a Unity mock-to-mock comparison.

Test evidence:

- Pre-change focused reproduction: 1 executed, 1 failed with the exact missing Standard RunStart trigger error.
- Post-change minimal server focused test: 1 passed.
- Post-change defense-derived server focused test: 1 passed.
- Full `game_server --all-targets`: 25 passed, 0 failed, including all six audit-named tests.
- Core focused trigger/start tests: 3 passed, 0 failed.
- Full `game_core --all-targets`: 681 passed, 0 failed.
- `cargo check -p game_server`: passed.
- Final `cargo fmt --all -- --check`: passed.
- Scoped `git diff --check`: passed.
- The initial exact focused command executed zero tests and is explicitly excluded from success evidence.

Legacy/fallback audit:

- The server module has one `test_hq_briefing_visual_novel_fixture` owner and one typed trigger constructor.
- The old definition-only `test_hq_briefing_visual_novel_database` path is absent from the server module.
- No runtime fallback, test-only Core bypass, serde alias, duplicated field, weakened validation, ignored test, or assertion deletion was added.
- Full production live data is not forced into every minimal actor fixture.

Long-term direction review:

- Fit: High.
- Improvement class: none for G04.
- Reason: fixture ownership is narrower and harder to drift because definitions and trigger metadata are constructed together. Production validation remains authoritative, public Core builder APIs are reused, and no cross-crate test helper or compatibility layer was introduced.

Verdict:

- Complete.

Remaining risk:

- `GameDataBase` validation currently rejects duplicate RunStart triggers and bad references, while supported-mode presence and exact RunStart policy are still enforced late by the start path rather than as a complete startup bundle invariant. This is not a G04 fixture regression and Phase 0 forbids absorbing Phase 1 policy work. G12 must retain explicit ownership, and R00 must not mark that broader CPR-M-01 aspect complete.

## Cross-Component Review

- Core runtime rejection, live RON, and the server test fixture now agree on the Standard/Endless RunStart identities and trigger policies.
- No DTO or Unity consumer changed, so there is no new cross-component contract split.
- The paired server fixture and Core's private world-test fixture are separate test-module owners but use the same public typed data contract. They are not competing production sources of truth.
- The only remaining cross-goal seam is early live-bundle cardinality validation, explicitly assigned to G12.

## Validation Classification

- Required Core/server runners: available and passed.
- Unity runner: available but not required for this fixture-only child; not executed.
- Zero-test command: invalid evidence, corrected and preserved in `EXPERIMENTS.md`.
- Baseline failures: historical 6 server failures are resolved; no new failures remain.

## Review Conclusion

G04 satisfies its user-visible and evidence completion conditions. It may be marked complete and handed to the separate Phase 0 R00 review. R00 must independently reproduce the baseline and verify the G12 ownership handoff before Phase 1 begins.
