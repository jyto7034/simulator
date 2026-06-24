# Experiments

| Date | Attempt | Result | Follow-up |
|---|---|---|---|
| 2026-06-23 | Goal setup | Created documentation sync goal after large core/runtime policy implementation. | Start by reading required startup documents and building a runtime/document sync checklist. |
| 2026-06-23 | Startup document read | Read required goal docs, sync guidelines, README, refactor plan, review guide, rulebook, skill target contract, and external Unity canonical docs before code/document edits. | Proceeded to runtime/DTO evidence scan. |
| 2026-06-23 | Runtime/DTO evidence scan | Confirmed `BattleResync` has no `setup` field, checkpoint units are Active-only, event log version is 27, withdraw/death events carry positions, `SkillCastCancelled`/withdraw buff reasons exist, and redeploy HP policy is in live redeploy state. | Updated internal/external docs and created `SYNC_CHECKLIST.md`. |
| 2026-06-23 | Stale documentation search | Searched synced docs for `setup: null`, `request_battle_resync`, `known_seq`, `need_setup`, and ambiguous checkpoint wording. | Replaced old resync payload wording with `battle_response: "battle_resync"` + `request_battle_state { since_seq }`, and setup-loss with `recover_battle_setup_loss`. |
| 2026-06-23 | User-facing error text alignment | Found server error message still said `known_seq` although current request field is `since_seq`. | Updated `../game_server/src/game/player_game_actor/state.rs` and test names/messages to use `since_seq`; run focused tests next. |
| 2026-06-23 | `cargo test -p game_server player_game_actor -- --nocapture` | Passed: 24 player_game_actor focused tests. Confirmed battle resync serialization, no trailing battle snapshots, `PlayerStateSnapshotDto`, and static obstacle error mapping. | Continue core focused tests. |
| 2026-06-23 | `cargo test -p game_core withdraw -- --nocapture` | Passed: 8 focused tests. Covered withdraw projectile miss, skill projectile no-hit, buff expiry withdrawn reasons, `SkillCastCancelled`, block release, and redeploy lock. | Continue seq/error focused test. |
| 2026-06-23 | `cargo test -p game_core live_defense_battle_state_rejects_future_since_seq -- --nocapture` | Passed: 1 focused test. Confirms invalid future battle state cursor is rejected under the renamed `since_seq` test. | Run broad checks. |
| 2026-06-23 | `cargo check` | Passed. | Run broad test. |
| 2026-06-23 | `cargo test -- --test-threads=1` | Passed: game_core unit/integration/doc tests, including RON loading and skill validation suites. | Final audit and report. |
| 2026-06-23 | Final stale-term audit | `rg "setup: null|\"setup\": null|request_battle_resync|known_seq|need_setup|현재 살아있는"` across synced docs and relevant code returned no matches. | Goal can be completed. |
