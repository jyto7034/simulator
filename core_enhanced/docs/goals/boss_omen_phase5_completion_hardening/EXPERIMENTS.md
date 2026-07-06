# Boss Omen Phase 5 Completion Hardening Experiments

## Experiment Log

| Date | Experiment | Result | Next Action |
| --- | --- | --- | --- |
| 2026-06-28 | Goal creation | Created follow-up hardening goal after Phase 5 completion review found live trigger, Event atomicity, Event combat test, and canonical doc gaps. | Re-audit runtime code and live RON before editing. |
| 2026-06-28 | Pre-goal review of skipped-chain reroll issue | User confirmed current policy can stay because WhiteNight is the only authored final-boss chain. | Keep skipped-chain pool-reset behavior out of this goal's code scope. |
| 2026-06-28 | Runtime/RON re-audit | Confirmed BossOmen eligibility depends on awakened skill fragment progress, Event choice effects were applied while iterating, Event-started combat reused live battle but result/retry paths had map-combat assumptions, and stable policy still lived in the draft. | Patch live awakenable fragment data, stage Event choice effects, and add contract tests. |
| 2026-06-28 | Boss Omen live trigger path | Added live `awakened_skill_id` values to existing active fragments that point at existing live skills, and added a live-data test proving an awakened fragment can trigger Boss Omen overlay. | Keep trigger data-authored; no code fallback. |
| 2026-06-28 | Event choice atomicity | Staged Grant/effect mutations before committing them to run state. A regression test now confirms a later StartCombat invariant panic leaves earlier grants unapplied. | Keep Event choice resolution all-or-nothing. |
| 2026-06-28 | Event-started combat hardening | Allowed Event node combat results to complete when the active content is a combat result, and allowed retry flow to reuse an explicitly stored Event combat preview. Added victory/defeat/retreat tests. | Run focused and broad verification. |
| 2026-06-28 | BossOmen step/forced boss hardening | Consumed terminal Event source steps before next-floor map generation and forced completed chains before normal overlay placement. Added tests for Event source step consumption and forced boss victory/defeat behavior. | Run focused and broad verification. |
| 2026-06-28 | Canonical documentation | Absorbed stable BossOmen/Event policy into `game_rulebook.md` and `skills/skill_fragment_system.md`; marked `boss_omen_chain_policy_draft.md` superseded and updated `README.md`. | Verify docs no longer present the draft as canonical. |

## Verification Commands

Record every focused and broad check here while implementing.

Expected focused checks:

```bash
cargo test --lib boss_omen
cargo test --lib event_node
cargo test --lib event_started_combat
cargo test --lib run_checkpoint_preserves
cargo test --lib forced_boss
cargo check -p game_server
cargo test --lib
```

Adjust exact test filters to the names created during implementation.

Actual focused checks run so far:

```bash
cargo test --lib live_awakened_fragment_can_trigger_boss_omen_overlay -- --nocapture
cargo test --lib terminal_event_boss_omen_step_consumption_forces_boss_on_next_floor -- --nocapture
cargo test --lib forced_boss_omen_defeat_immediately_fails_endless_run -- --nocapture
cargo test --lib forced_boss_omen_victory_clears_chain_and_advances_endless_floor -- --nocapture
cargo test --lib event_choice_effects_are_atomic_when_later_effect_panics -- --nocapture
cargo test --lib event_choice_starts_combat_and_retreat_preserves_committed_choice -- --nocapture
cargo test --lib event_started_combat_victory_consumes_event_node -- --nocapture
cargo test --lib event_started_combat_defeat_reenters_while_attempts_remain -- --nocapture
cargo test --lib run_checkpoint_preserves_abnormality_run_state -- --nocapture
```

All focused checks above pass. The atomicity test intentionally catches a panic from invalid fixture data, so the panic text appears in output while the test result is `ok`.

Final broad checks:

```bash
cargo test --lib boss_omen -- --nocapture
cargo test --lib event_node -- --nocapture
cargo check -p game_server
cargo test --lib
```

Results:

- `cargo test --lib boss_omen -- --nocapture`: passed, 7 tests.
- `cargo test --lib event_node -- --nocapture`: passed, 4 tests.
- `cargo check -p game_server`: passed.
- `cargo test --lib`: passed, 582 tests.

## Failed Attempts

Record failed attempts here with:

- command or approach;
- failure symptom;
- root cause;
- fix;
- re-verification command.

Do not erase failed attempts after fixing them.

| Date | Command / Approach | Failure Symptom | Root Cause | Fix | Re-verification |
| --- | --- | --- | --- | --- | --- |
| 2026-06-28 | `cargo test --lib forced_boss_omen_victory_clears_chain_and_advances_endless_floor -- --nocapture` | Test expected direct `FloorAdvanced`, but result was not that variant. | Successful combat result first returns `CombatRewardsGranted`, with actual node completion nested in `completion`. Test expectation was not matching user-visible reward envelope. | Updated test to assert `CombatRewardsGranted { completion: FloorAdvanced }`. | Same command now passes. |
| 2026-06-28 | `cargo test --lib event_started_combat_victory_consumes_event_node -- --nocapture` | Initially `InvalidAction`, then direct `FloorAdvanced` expectation failed. | `can_complete_combat_result_locally` only allowed Combat/Boss node sessions; after fixing that, test still ignored combat reward envelope. | Allowed Event sessions with combat result content to complete; updated test to assert nested `completion`. | Same command now passes. |
| 2026-06-28 | `cargo test --lib event_started_combat_defeat_reenters_while_attempts_remain -- --nocapture` | `InvalidAction` during retry preview. | Event-started combat stored an explicit combat preview for the Event node, but `combat_preview_for_node` returned `None` for Event category before checking stored previews. | Reused existing stored combat preview before applying map-node category generation guard. | Same command now passes. |
| 2026-06-28 | `cargo test --lib live_awakened_fragment_can_trigger_boss_omen_overlay event_choice_effects_are_atomic_when_later_effect_panics ...` | Cargo rejected multiple test filter arguments. | `cargo test` accepts a single test name/filter before `--`; extra names are unexpected args. | Reran focused tests individually. | Individual focused test commands pass. |
