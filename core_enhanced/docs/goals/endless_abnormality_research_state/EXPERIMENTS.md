# Endless Abnormality Research State Experiments

## Experiment Log

| Date | Experiment | Result | Next Action |
| --- | --- | --- | --- |
| 2026-06-26 | Goal creation | Split abnormality response/research state into its own goal after `run_floor_progression_contract`. | Begin implementation by auditing runtime state, reward execution, abnormality metadata, and PvE encounter data. |
| 2026-06-26 | Phase 2 startup audit | Read `endless_mode_progression_master` and this subgoal's `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md` before code search. Audited runtime state, reward execution, abnormality metadata, PVE encounter data, and skill fragment data. `RunState` has no abnormality research state yet; reward execution already supports `GrantSkillFragment` and `GrantFragmentDust`; abnormality metadata has threat class; PVE encounters have `abnormality_id`. | Stop before implementation because the unique first-completion fragment id policy is not satisfiable from current live data without a user/content decision. |
| 2026-06-26 | User decision resumed Phase 2 | User chose explicit abnormality reward field plus temporary placeholder skill fragments for abnormalities whose final fragment design is not settled. | Proceed with implementation; reward source remains data-authored and explicit. |
| 2026-06-26 | Runtime state and policy schema implementation | Added `RunAbnormalityResearchState`, live RON abnormality research policy, response-completion fragment metadata, checkpoint persistence, snapshot DTO, and Endless combat victory reward wiring. | Validate focused state/reward/snapshot/checkpoint behavior. |
| 2026-06-26 | Focused research tests | `cargo test abnormality_research --lib`, `cargo test game::world::tests::snapshots_and_start --lib`, and `cargo test game::world::tests::combat --lib` passed. | Run broad lib and live RON checks. |
| 2026-06-26 | Full verification | `cargo test --lib`, `cargo test --test ron_loading`, `cargo check --lib`, and `cargo check -p game_server` passed. | Mark Phase 2 complete and update master goal status. |
| 2026-06-27 | Unity-facing contract sync | `/mnt/f/unity projects/ark/docs/unity_core_contract.md` documents `state_snapshot.state.abnormality_research`, including the rule that Unity displays the field but does not recompute rewards or run completion from it. | Phase 2 documentation gap closed; continue with Phase 3 bonus objectives. |

## Verification Commands

Record every focused and broad check here while implementing.

Initial planned checks:

```bash
cargo test abnormality_research --lib
cargo test combat --lib
cargo test ron_loading --test ron_loading
cargo check --lib
```

Executed checks:

```bash
cargo test abnormality_research --lib
cargo test builtin_run_policy_validates_contract --lib
cargo test --test ron_loading
cargo test game::world::tests::snapshots_and_start --lib
cargo test game::world::tests::combat --lib
cargo test game_data_base_accepts_fragment_origin_and_independent_imitation_skill --lib
cargo test game_data_base_rejects_fragment_unknown_imitation_skill --lib
cargo test --lib
cargo fmt
cargo check --lib
cargo check -p game_server
```

Post-format focused checks also passed:

```bash
cargo test abnormality_research --lib
cargo test game::world::tests::snapshots_and_start --lib
cargo check --lib
```

## Failed Attempts

No implementation attempts yet.

### 2026-06-26: Unique response-completion fragment data is incomplete

- Command:
  - `awk ... ../game_resources/data/abnormalities/base.ron ../game_resources/data/skill_fragments/base.ron`
  - `rg -n "^\\s*id: \\\"fragment_" ../game_resources/data/skill_fragments/base.ron -S`
- Failure summary: Phase 2 policy requires first response completion to grant the abnormality's data-authored unique skill fragment, but live data does not currently identify a unique abnormality-origin fragment for every suppression target.
- Evidence:
  - Live abnormality count: `30`.
  - Live `fragment_*` count: `22`.
  - Current `origin: Some(Abnormality(...))` mappings cover only 15 abnormalities.
  - Missing abnormality-origin fragment mappings:
    - `f-01-87_snow_queen`
    - `f-02-49_rudolta`
    - `d-04-108_parasite_tree`
    - `o-04-66_porcubbus`
    - `f-01-18_scarecrow`
    - `o-02-63_apocalypse_bird`
    - `o-01-73_knight_of_despair`
    - `o-01-64_king_of_greed`
    - `f-02-58_big_bad_wolf`
    - `t-06-27_moonlit_wail`
    - `t-04-50_queen_bee`
    - `o-01-15_nameless_fetus`
    - `f-02-70_black_swan`
    - `o-05-76_schadenfreude`
    - `d-01-110_clouded_monk`
- Root cause: The current skill fragment catalog has placeholder fragments for some abnormalities, but not every suppression-target abnormality has an abnormality-origin unique fragment mapping. Some existing placeholder fragments are concept-origin rather than abnormality-origin.
- Code/data/doc change made: None. This matches the subgoal stop condition: "Unique skill fragment reward ids are missing for expected reward abnormalities."
- Re-verification result: Not applicable until the user decides whether to add missing fragment content, add explicit `response_complete_skill_fragment_id` fields, narrow suppression targets, or defer first-completion unique fragment grants.

Resolution:

- User chose explicit `response_complete_skill_fragment_id` plus temporary placeholder skill fragments.
- Added the field to abnormality metadata and live RON.
- Added missing placeholder fragments and converted existing placeholder provenance where appropriate.
- Re-verified with `cargo test --test ron_loading`, `cargo test --lib`, and `cargo check --lib`.

### 2026-06-26: Full lib test exposed a fixture missing response-completion fragment id

- Command:
  - `cargo test --lib`
- Failure summary: `game::data::tests::game_data_base_accepts_fragment_origin_and_independent_imitation_skill` panicked with `abnormality 't-02-43_freischutz' must declare response_complete_skill_fragment_id`.
- Root cause: The test fixture provided a non-empty skill fragment catalog, so the new validation correctly required the abnormality fixture to declare its response-completion fragment id.
- Code/data/doc change made: Added `response_complete_skill_fragment_id: Some("fragment_freischutz_black_round")` to the relevant Freischutz test fixtures.
- Re-verification command and result:
  - `cargo test game_data_base_accepts_fragment_origin_and_independent_imitation_skill --lib` passed.
  - `cargo test game_data_base_rejects_fragment_unknown_imitation_skill --lib` passed.
  - `cargo test --lib` passed.

When a test or trial fails, record:

- command;
- failure summary;
- root cause;
- code/data/doc change made;
- re-verification command and result.
