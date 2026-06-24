# Experiments

## Log

| Date | Scope | Attempt | Result | Follow-up |
| --- | --- | --- | --- | --- |
| 2026-06-22 | Goal setup | Defined grant/economy/reward implementation scope. | Not started. | Inventory grant call sites before editing. |
| 2026-06-22 | Reward tags / forbidden reward removal | Removed `RewardTag` and reward `tags` from reward metadata/options. Replaced stored tags with effect-derived `RewardGrantKind` for combat/map reward policy checks. Removed `ForbiddenAbnormalityGrant` from reward effects and live reward data. | Success. `cargo check --lib`, reward policy tests, and full RON loading tests passed. | Continue canonical grant executor rollout across non-reward acquisition paths. |
| 2026-06-22 | Explicit equipment reward pools | Added `equipment_pools` to reward data with typed filter/weighted entry schema. Replaced `GrantEquipment(equipment_id: None)` live rewards with `GrantEquipmentFromPool(pool_id: "ego_gift_reward_pool")`; validation now requires pool references to produce candidates. | Success. Full RON loading and `cargo check -p game_server` passed. | Later expand tests around pool filters/weights if additional live pools are authored. |
| 2026-06-22 | Atomic reward claim / Enkephalin overflow | Changed reward session application to preflight on cloned inventory, skill fragment inventory, UUID manager, and Enkephalin, then commit only after all effects succeed. Added `Enkephalin::checked_add` and removed reward/start/headquarters/shop sell saturating/direct increment paths. | Success. Added focused partial-commit regression test; `cargo test reward_claim_failure_does_not_partially_commit_prior_effects --lib` passed. | Continue adding diffs for skill fragment/research/XP and moving other grant flows into `GrantExecutor`. |
| 2026-06-22 | Remove automatic duplicate fragment conversion | Removed `SkillFragmentStackingPolicy::ConvertAdditionalCopiesToResource` and the `NotImplemented` grant branch. Additional copies now either reject or stack according to the remaining explicit policy. | Success. Search found no remaining conversion references; focused copy stack/dismantle test and `cargo check --lib` passed. | Continue fragment dust result/diff work through canonical grant reporting. |
| 2026-06-22 | RewardGranted skill fragment/research diffs | Added `SkillFragmentGrantDiffDto` and `SkillFragmentResearchDiffDto` to grant execution results, `RewardGranted`, `CombatRewardsGranted`, and game_server behavior payloads. Reward session application now aggregates fragment/research diffs alongside inventory diff. | Success. Focused diff regression test, `cargo check --lib`, full RON loading, and `cargo check -p game_server` passed after server payload update. | XP target policy/diff remains the next reward result contract gap. |
| 2026-06-22 | Reward experience target policy / XP diffs | Changed `GrantExperience` to require explicit `ExperienceTargetPolicy`, added `GrantExecutionContext`, moved combat reward XP into `GrantExecutor`, and added `EmployeeExperienceDiffDto` to `RewardGranted`/`CombatRewardsGranted` plus server payloads. Live `experience_reward` now declares `target: AliveRoster`. | Success. Focused AliveRoster XP diff and live RON target tests passed; `cargo check --lib`, `cargo test --test ron_loading`, and `cargo check -p game_server` passed. | Continue moving non-reward acquisition flows into `GrantExecutor`. |
| 2026-06-22 | Admin grant canonical executor migration | Added public `GrantExecutor::grant_effects_with_state` and `GrantFragmentDust`. Reworked admin equipment, consumable, artifact, skill fragment, equipment material, and fragment dust grants to execute typed grant effects through clone-preflight/commit instead of direct inventory/fragment mutation. | Success. `cargo check --lib` and all `game::world::admin::tests` passed; search shows admin grant direct acquisition mutations moved into `GrantExecutor`. | Continue shop/headquarters/maintenance acquisition migration. |
| 2026-06-22 | Shop acquisition migration | Reworked shop purchase and sell to keep shop-specific validation/cost/stock/removal in the shop domain while routing final item acquisition and sell Enkephalin gain through `GrantExecutor` typed effects. | Success. `cargo check --lib` and `cargo test shop --lib` passed; search shows shop direct acquisition mutation moved into `GrantExecutor`. | Continue headquarters supplies and maintenance dismantle output migration. |
| 2026-06-22 | Headquarters emergency supplies migration | Reworked headquarters emergency Enkephalin grant to use `GrantExecutor` through `GameCore::apply_grant_effects`. | Success. Focused headquarters emergency supplies test and `cargo check --lib` passed. | Continue maintenance dismantle output migration. |
| 2026-06-22 | Maintenance dismantle output migration | Reworked equipment dismantle material yields and skill-fragment dismantle dust yields to execute as typed `RewardEffect`s through `GrantExecutor`. Split skill-fragment copy removal from dust granting. Added grant preflight for dismantle outputs before mutating equipment/fragments/loadouts. | Success. `cargo check --lib` and focused equipment/skill-fragment dismantle tests passed. | Run final grant/economy validation set before closing the subgoal. |
| 2026-06-22 | Post-battle survival XP migration | Reworked combat survival XP from direct `employee.add_experience` calls to `GrantExecutor` with `GrantExperience { target: SelectedEmployee }`. Trauma, injury, and roster-order sync remain combat resolution behavior. | Success. `cargo check --lib`, explicit XP reward test, and full combat world test module passed. | Re-scan remaining acquisition mutations before final validation. |
| 2026-06-22 | Starter loadout and equipment combination migration | Reworked starter equipment creation and maintenance equipment-combination result creation to use `GrantExecutor` equipment grants. Loadout/equipped-to assignment remains starter/maintenance domain behavior after the granted item UUID is known. | Success. `cargo check --lib`, starter-focused tests, and equipment-combination test passed. Runtime acquisition mutation scan now leaves only `GrantExecutor`, primitive resource APIs, and test fixtures. | Run broad subgoal validation. |

## Failed Approaches

| Date | Scope | Attempt | Failure | Correction |
| --- | --- | --- | --- | --- |
| 2026-06-22 | RewardGranted skill fragment/research diffs | Added a focused reward diff test using `fragment_freischutz_black_round`. | The `game_data_with_map_content` fixture did not include that live fragment metadata, so the executor correctly returned `InvalidStaticData`. | Switched the focused fixture to `starter_basic_attack_enhancement`, which is present in the test GameData. |
| 2026-06-22 | RewardGranted skill fragment/research diffs | Ran `cargo check -p game_server` after adding new `BehaviorResult` fields. | Server behavior-result pattern matching did not include the new diff fields. | Added `skill_fragment_diffs` and `skill_fragment_research_diffs` to the serialized `RewardGranted` and `CombatRewardsGranted` payloads. |

## Validation Commands

- `cargo check --lib` - passed after reward tag/forbidden removal.
- `cargo test reward_grant_kinds_are_inferred_from_effects --lib` - passed, 1 test.
- `cargo test reward_claim_failure_does_not_partially_commit_prior_effects --lib` - passed, 1 test.
- `cargo test live_rewards_can_grant_skill_fragments_from_ron --test ron_loading` - passed, 1 test.
- `cargo test reward_policy --lib` - passed, 4 tests.
- `cargo test --test ron_loading` - passed, 17 tests.
- `cargo check -p game_server` - passed after reward schema changes.
- `rg -n "ConvertAdditionalCopiesToResource|additional copy conversion" src tests ../game_resources/data` - no matches after removal.
- `cargo test additional_fragment_copies_stack_until_dismantled_for_dust --lib` - passed, 1 test.
- `cargo check --lib` - passed after removing duplicate fragment conversion variant.
- `cargo test reward_session_reports_skill_fragment_and_research_diffs --lib` - initially failed on missing fixture fragment id, then passed after fixture id correction, 1 test.
- `cargo check --lib` - passed after reward diff fields.
- `cargo test --test ron_loading` - passed after reward diff fields, 17 tests.
- `cargo check -p game_server` - initially failed on missing new `BehaviorResult` fields in payload serialization, then passed after payload update.
- `cargo test reward_session_grant_experience_uses_explicit_alive_roster_target_and_reports_diffs --lib` - passed, 1 test.
- `cargo test live_experience_rewards_declare_target_policy --test ron_loading` - passed, 1 test.
- `cargo check --lib` - passed after explicit XP target policy migration.
- `cargo test --test ron_loading` - passed after explicit XP target policy migration, 17 tests.
- `cargo check -p game_server` - passed after employee XP diff payload update.
- `cargo check --lib` - passed after admin grant executor migration.
- `cargo test game::world::admin::tests --lib` - passed after admin grant executor migration, 11 tests.
- `rg -n "add_item_owned|add_id_with_policy|add_fragment_dust|equipment_materials\\.add|next_owned_equipment|next_owned_consumable" src/game/world/admin src/game/reward.rs` - direct admin acquisition mutations were gone; remaining matches are inside `GrantExecutor`.
- `cargo test shop --lib` - passed after shop acquisition migration, 13 tests.
- `rg -n "add_item_owned|next_owned_equipment|next_owned_consumable|enkephalin\\.checked_add|enkephalin\\.amount \\+=|equipment_materials\\.add|add_fragment_dust" src/game/world/shop.rs src/game/world/admin src/game/reward.rs` - direct shop/admin acquisition mutations were gone; remaining matches are inside `GrantExecutor`.
- `cargo test headquarters_emergency_supplies_completes_node_without_shop_or_recruitment --lib` - passed after headquarters emergency supplies migration, 1 test.
- `cargo check --lib` - passed after headquarters emergency supplies migration.
- `cargo check --lib` - passed after maintenance dismantle output migration.
- `cargo test dismantle_equipment_is_maintenance_only_and_grants_material_stacks --lib` - passed after equipment dismantle output migration, 1 test.
- `cargo test skill_fragment_dismantle_is_maintenance_only_and_grants_dust --lib` - passed after skill-fragment dismantle output migration, 1 test.
- `cargo test skill_fragment_dismantle_unequips_equipped_fragment --lib` - passed after skill-fragment dismantle output migration, 1 test.
- `cargo test skill_fragment_dismantle_allows_last_copy --lib` - passed after skill-fragment dismantle output migration, 1 test.
- `cargo check --lib` - passed after post-battle survival XP migration.
- `cargo test reward_session_grant_experience_uses_explicit_alive_roster_target_and_reports_diffs --lib` - passed after post-battle survival XP migration, 1 test.
- `cargo test game::world::tests::combat --lib` - passed after post-battle survival XP migration, 27 tests.
- `cargo check --lib` - passed after starter loadout and equipment-combination migration.
- `cargo test start --lib` - passed after starter loadout migration, 35 tests.
- `cargo test combine --lib` - passed after equipment-combination migration, 1 test.
- `rg -n "add_item_owned|\\.add_item\\(|add_id_with_policy|add_fragment_dust|equipment_materials\\.add|next_owned_equipment|next_owned_consumable|enkephalin\\.checked_add|enkephalin\\.amount \\+=|add_experience\\(" src/game/world src/game/reward.rs src/game/resources src/game/skill_fragment.rs` - runtime acquisition mutations are centralized in `GrantExecutor`; remaining non-test matches are primitive resource APIs and tests.
- `rg -n "RewardTag|ForbiddenAbnormalityGrant|ConvertAdditionalCopiesToResource|GrantEquipment \\{\\s*equipment_id: None|GrantExperience \\{ amount: [0-9]+[),]" src tests ../game_resources/data ../game_server/src` - no matches after policy removal/migration.
