# Post Policy Test Verification Notes

## Initial Notes

- This is a test verification audit. Failing or stale tests should not be fixed in this goal.
- 0-test filters must be recorded as suspicious, not passing evidence.
- The prior technical debt review found no `#[ignore]` tests, but it found repeated goal-note evidence of filters that ran 0 tests or only 1-2 tests.
- External Unity docs in `/mnt/f/unity projects/ark/docs` are canonical for DTO coverage interpretation.

## Required Policy Areas

- Damage feedback DTO
- CombatPreview threat warnings
- Abnormality attempt/retreat/re-entry
- Consumable modifier re-entry duration
- Unit overlap and blocking
- Movement backend policy and Rapier quality
- Airborne enemy mobility
- Weapon archetype and targeting
- Skill fragment compatibility
- AD/AP balance validation

## Source-Of-Truth Findings

- Runtime/test inventory is stronger than older goal notes for most areas, but historical 0-test filters are real and still reproducible.
- External Unity docs in `/mnt/f/unity projects/ark/docs` are up to date for the new DTO fields that matter here: `feedback_tags`, `threat_warnings`, `unit_deploy_costs.effective_deploy_cost`, `mobility_kind`, and skill fragment compatibility.
- `cargo check -p game_server` passing is not enough for Unity-facing DTO health because `cargo test -p game_server -- --list` fails under test configuration.
- `tests/unit_test.rs` is an empty test target. It should not be cited as verification.

## Policy Area Notes

- Damage feedback DTO has focused coverage through `feedback_tags` tests: damage tags, command timeline emission, and snake_case serialization are present.
- CombatPreview threat warnings have generator/serialization/live RON coverage, including rumor warnings and AD/AP warning tags. The hardcoded false-rumor `Disproved` lifecycle remains weak or not clearly implemented in current test coverage.
- Attempt/retreat/re-entry has two focused tests. They cover re-entry attempts and consumable modifier survival/expiry, but edge cases around all units dead before `BattleEnd`, post-`BattleEnd` command rejection, and UI command state are still thin.
- Consumable modifier duration and validation coverage is comparatively strong: percent validation, deploy cost reduction, defense mitigation, dead target rejection, alive-but-combat-unavailable target allowance, replacement without refund, and RON loading are all represented.
- Unit overlap/blocking coverage is stronger under `movement` and `fixed_defense` than under the narrow `blocking` filter recorded by older goals. Future notes should avoid using `blocking` alone as proof.
- Movement/Rapier coverage is strong for overlap allowance, static obstacle correction, board clamp, airborne terrain immunity, and backend parity. Prior exact-float brittleness was already found and addressed in the Rapier goal.
- Airborne coverage is strong: movement policy, blocking immunity, target priority, air-capable single target gating, area hit inclusion, preview warning, and live RON loading are present.
- Weapon targeting coverage is focused around targeting profiles and explicit cast/step targeting, but full end-to-end Unity/server DTO test coverage is blocked by the stale `game_server` test build.
- Skill fragment compatibility coverage is strong for static requirements, equip accept/reject, snapshot-facing compatibility data, and RON loading.
- AD/AP validation is covered through damage/resistance unit tests and live preview warning RON tests. False-rumor disproof lifecycle is still a separate coverage gap.

## Stale Or Brittle Test Notes

- `authored_protect_unit_tactical_plan_overrides_default_defense_contract` likely hardcodes a route id that is not guaranteed by the generated battlefield archetype. The stricter battlefield validation is probably correct; the fixture should be made route-aware or use an authored battlefield.
- `game_server` test fixtures lag behind current DTO/schema fields. This is stale fixture debt and also blocks server-side Unity contract tests from compiling.
- Integration test file names used as cargo filters produce 0 tests. Use `--test <name>` for those targets.
- Empty `tests/unit_test.rs` should be removed or populated in a cleanup goal.
