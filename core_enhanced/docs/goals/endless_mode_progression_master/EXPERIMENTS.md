# Endless Mode Progression Master Experiments

## Phase Status

| Phase | Subgoal | Status | Notes |
| --- | --- | --- | --- |
| 1 | `run_floor_progression_contract` | Complete | Completed on 2026-06-26; Floor-centric Standard/Endless progression is implemented and tested. |
| 2 | `endless_abnormality_research_state` | Complete | Completed on 2026-06-26 after user chose explicit `response_complete_skill_fragment_id` plus temporary placeholder fragments. |
| 3 | `pve_encounter_bonus_objectives` | Complete | Completed on 2026-06-27; encounter-authored bonus objective schema, validation, event-log evaluation, result DTO, and research reward integration are implemented and tested. |
| 4A | `pve_encounter_classification_floor_scaling_contract` | Complete | Completed on 2026-06-27; encounter difficulty removed, explicit encounter classes and optional primary abnormality identity added, live RON Floor scaling implemented, and validation passed. |
| 4B | `endless_repeat_encounter_weighting` | Complete | Completed on 2026-06-27; live-RON repeat weighting policy, Endless recent-Floor exclusion, response-complete weight reduction, explicit soft fallback reset, and focused tests are implemented. |
| 5 | `boss_omen_chain` | Pending | Last; policy questions remain. |

## Master Experiment Log

| Date | Experiment | Result | Next Action |
| --- | --- | --- | --- |
| 2026-06-26 | Master goal creation | Created orchestration plan for the dependent Endless progression subgoals. | Start Phase 1 with `run_floor_progression_contract`. |
| 2026-06-26 | Phase 1 completion sync | `run_floor_progression_contract` completed; master status table updated from stale Pending state. | Start Phase 2 only after reading Phase 2 docs. |
| 2026-06-26 | Phase 2 startup audit | `endless_abnormality_research_state` stopped before implementation because current live data does not provide unique first-completion fragment mappings for every suppression-target abnormality. | Ask the user how to satisfy the unique fragment reward policy before implementing Phase 2 runtime state. |
| 2026-06-26 | Phase 2 completion sync | `endless_abnormality_research_state` completed after adding run-local research state, explicit response-completion fragment metadata, placeholder fragments, snapshot DTO, reward wiring, checkpoint persistence, and tests. | Phase 3 and Phase 4 may now proceed in dependency order. |
| 2026-06-27 | Phase 3 completion sync | `pve_encounter_bonus_objectives` completed after adding typed bonus objective authoring, validation, completed event-log evaluation, selected-event/battle-record result exposure, and victory research integration. | Phase 4 repeat weighting startup audit may proceed next. |
| 2026-06-27 | Phase 4 startup audit | `endless_repeat_encounter_weighting` stopped before implementation because current map encounter assignment silently falls back when candidates are empty, and live PVE difficulty range can be exceeded by Endless Floor scaling. | Ask the user to choose explicit empty-candidate fallback/difficulty cap policy before implementing repeat weighting. |
| 2026-06-27 | Phase 4A policy decision | User chose to remove encounter difficulty, introduce explicit `Normal`/`Elite`/`NormalBoss`/`FinalBoss` classification, and implement live-RON Floor scaling stages for stats, generated wave budget, and explicit extra waves. Created `pve_encounter_classification_floor_scaling_contract` as the canonical Phase 4A prerequisite before repeat weighting. | Run Phase 4A before resuming Phase 4B `endless_repeat_encounter_weighting`. |
| 2026-06-27 | Phase 4A completion sync | `pve_encounter_classification_floor_scaling_contract` completed after replacing difficulty-window selection, adding explicit encounter classes, migrating primary abnormality identity, adding Floor scaling policy, repairing stale tests, and passing broad validation. | Phase 4B `endless_repeat_encounter_weighting` is now unblocked. |
| 2026-06-27 | Phase 4B completion sync | `endless_repeat_encounter_weighting` completed after adding live-RON repeat policy, passing research state into map encounter selection, implementing Endless-only recent/repeat weighting for primary-abnormality candidates, and testing explicit soft fallback reset. | Phase 5 `boss_omen_chain` remains pending. |

## Verification Commands

This master file records only cross-phase or final verification. Each subgoal records its own focused checks.

Planned final broad checks after all active phases:

```bash
cargo test --lib
cargo test --test ron_loading
cargo check --lib
cargo check -p game_server
```

Add probe/server commands when the final DTO shape changes are verified live.

## Failed Attempts

### 2026-06-26: Phase 2 stopped on unique fragment reward data

- Subgoal: `endless_abnormality_research_state`.
- Blocking condition: Unique skill fragment reward ids are missing for expected response-completion reward abnormalities.
- Decision made: No runtime implementation was started. The finding was recorded in the Phase 2 experiment notes.
- User-approved next action: pending.

Resolution:

- User approved explicit metadata reward ids plus temporary placeholder fragments.
- Phase 2 resumed and completed.
- Verification passed:
  - `cargo test --lib`
  - `cargo test --test ron_loading`
  - `cargo check --lib`
  - `cargo check -p game_server`

If a phase fails or is deferred, record:

- subgoal;
- blocking condition;
- decision made;
- user-approved next action.
