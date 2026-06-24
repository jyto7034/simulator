# Documentation Runtime Contract Audit Plan

## Objective

Verify that the current major documentation and the actual runtime code/data/Unity-facing contracts describe the same game and transport behavior, then update stale documentation or record required policy questions.

## Source Of Truth Order

Do not trust documents first. Check facts in this order:

1. Actual runtime code.
2. Live RON/data.
3. Unity-facing snapshot/command/WebSocket contracts.
4. Latest policy documents.

If documents and code disagree, read the code and live data first. If the better correction is a policy decision rather than an implementation/documentation cleanup, stop the goal and report questions to the user.

## Primary Documents To Audit

Core policy/index documents:

- `docs/README.md`
- `docs/code_documentation_sync_guidelines.md`
- `docs/game_rulebook.md`
- `docs/skill_target_contract.md`
- `docs/refactor_preparation_plan.md`
- `docs/codex_goal_command.md`

Content/design documents:

- `docs/skills/lobotomy_content_catalog.md`
- `docs/skills/abnormality_skill_design_notes.ko.md`
- `docs/skills/skill_fragment_system.md`
- `docs/skills/skill_fragment_wiki.md`
- `docs/skills/ego_equipment_wiki.md`
- `docs/skills/tool_abnormality_wiki.md`

External Unity canonical documents:

- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

Goal documents are not canonical, but use them as recent implementation context when they explain why a policy changed:

- `docs/goals/*/PLAN.md`
- `docs/goals/*/EXPERIMENTS.md`
- `docs/goals/*/EXPERIMENT_NOTES.md`

## Runtime Areas To Compare

Focus on current live paths, not historical names.

- Player command/result DTOs:
  - `src/game/behavior.rs`
  - sibling server paths if needed: `/mnt/f/work/simulator/game_server/src/game/player_game_actor/messages.rs`
  - sibling server paths if needed: `/mnt/f/work/simulator/game_server/src/game/player_game_actor/state.rs`
  - sibling server paths if needed: `/mnt/f/work/simulator/game_server/src/game/player_game_actor/session.rs`
- Snapshot and game flow:
  - `src/game/world/snapshot.rs`
  - `src/game/world/state.rs`
  - `src/game/world/combat.rs`
  - `src/game/world/helpers.rs`
- Battle transport and timeline:
  - `src/game/battle/timeline.rs`
  - `src/game/battle/core/commands.rs`
  - `src/game/battle/core/types.rs`
  - `src/game/battle/core/sim.rs`
  - `src/game/battle/core/movement/*`
- Targeting, range, skills, effects:
  - `src/game/ability.rs`
  - `src/game/data/skill_data.rs`
  - `src/game/data/*`
  - `src/game/battle/core/targeting*`
- Live RON/data:
  - `/mnt/f/work/simulator/game_resources/data/**/*.ron`
  - local config/data mirrors if referenced by runtime.

## In Scope

- Identify stale documentation that contradicts runtime code, live RON, or external Unity contracts.
- Update documentation when the correction is already implied by runtime behavior and current policy.
- Record code issues when code fails to satisfy a current canonical policy.
- Add or update focused tests/probes only when a mismatch can be fixed safely within this audit without changing gameplay policy.
- Remove or demote stale doc wording that presents superseded contracts as canonical.
- Check that `docs/README.md` points readers to the correct canonical documents.

## Out Of Scope

- Large gameplay implementation changes.
- New Unity-facing DTO shape changes unless they are already required by current canonical docs and tests.
- Live RON schema migrations unless the audit finds a clear mismatch that can be fixed without policy risk.
- Rebalancing numbers, enemy waves, rewards, skill power, or content lists.
- Rewriting all goal documents. Goal docs are working memory, not permanent source of truth.
- Creating compatibility layers, fallback paths, or dual schemas to preserve stale docs.

## Audit Passes

1. Document inventory.
   - List major docs and classify each as canonical, external canonical, context, goal memory, or stale/debug.
   - Confirm `docs/README.md` and `docs/code_documentation_sync_guidelines.md` agree about canonical locations.

2. Unity-facing contract pass.
   - Compare external `unity_core_contract.md` and `core_unity_battle_transport_contract.md` against typed DTOs and server message mapping.
   - Check battle setup, battle update, checkpoint, resync, command_result, error, final combat_result snapshot, range preview, unit source, HUD, movement, and timeline event fields.

3. Gameplay rulebook pass.
   - Compare `docs/game_rulebook.md` against runtime game flow, live combat mode policy, deployment/withdrawal, retreat/failure/result, battle records, blocking, movement, wave model, enemy identity, and HUD rules.

4. Skill/targeting pass.
   - Compare `docs/skill_target_contract.md` against runtime basic attack, tile range, projectile delivery, skill target modes, hostile target usefulness, `WholeFieldValidTiles`, corroded employee range presets, and ranged route attack policy.

5. Content/data pass.
   - Compare `docs/skills/*` against live RON and validation.
   - Do not invent missing content. If the document promises content not in live data, classify whether it is design-only or a runtime mismatch.

6. Refactor/process pass.
   - Check whether `docs/refactor_preparation_plan.md`, `docs/code_documentation_sync_guidelines.md`, and `docs/codex_goal_command.md` still match actual workflow and current external canonical names.

7. Report and fix pass.
   - Apply doc fixes that are unambiguous.
   - Record implementation gaps separately from documentation gaps.
   - Stop on policy decisions.

## Completion Conditions

- Major docs are classified and audited against runtime/data/Unity-facing contracts.
- Each found mismatch is categorized as:
  - doc stale and fixed,
  - code/runtime bug requiring implementation,
  - live data mismatch requiring data/schema work,
  - external Unity contract mismatch,
  - policy question requiring user decision,
  - out-of-scope follow-up.
- `EXPERIMENTS.md` records the audit commands, failed assumptions, fixes, and validation results.
- `EXPERIMENT_NOTES.md` records source-of-truth conflicts, policy questions, and follow-up candidates.
- Any documentation changed by the audit names its runtime/code/data evidence.
- If code or DTO behavior changes, relevant focused tests/probes are updated and run.
- If no mismatches are found for an area, that fact is recorded with the files checked.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

Stop and report questions instead of guessing if the audit finds:

- Unity-facing DTO shape change not already locked by current canonical docs.
- live RON schema change or migration.
- saved data migration.
- UX meaning change.
- balance standard change.
- deletion/replacement of existing content.
- reward/failure/consumption timing change.
- game mode, node flow, battle success/failure, retreat, or death policy ambiguity.
- a current document says one policy and runtime code clearly implements another, but neither side is obviously stale.

## Verification Commands

Use focused verification based on changed area. Candidate commands:

```text
rg -n "core_unity_battle_setup_snapshot_contract|core_unity_battle_update_contract|core_unity_battle_transport_contract|source of truth|canonical|supersede|deprecated" docs "/mnt/f/unity projects/ark/docs"
rg -n "battle_setup_snapshot|battle_update|battle_resync|command_result|state_snapshot|UnitSpawned|UnitDeployed|MovementSegmentStarted|BasicAttackProjectile" src /mnt/f/work/simulator/game_server/src/game/player_game_actor
cargo check -p game_core
cargo test -p game_core
cargo test -p game_server
```

Run server/probe validation only when Unity-facing JSON contract changes:

```text
ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run -p game_server
WS_PORT=8081 python3 "/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py"
```

## Completion Report Requirements

Report:

- Documents audited.
- Runtime/data/Unity contract evidence checked.
- Documentation fixes made.
- Code/data/test fixes made, if any.
- Mismatches intentionally left for follow-up.
- Policy questions, if any.
- Verification commands and results.
