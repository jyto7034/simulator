# DefenseRoute Single Runtime Model Plan

## Objective

Unify live DefenseRoute combat under one runtime model.

Current code treats `DefenseRoute / Encirclement` as a separate live battle variant and lets `Surrounded` battlefield archetype fallback into that variant. This made a normal Spider Bud Defense encounter start on `surrounded_medium_breach_01`, resolve as Encirclement, and end at battle time 0 because no player-side unit was deployed yet.

The target policy is:

- `DefenseRoute` is the single live runtime combat model for non-boss live defense battles.
- `SplitRoom` mode is removed from official runtime policy.
- `Encirclement` is not a separate runtime win-condition model.
- `Surrounded`, `Corridor`, `ChokePoint`, `Ambush`, and similar battlefield archetypes are map shape/classification only. They must not change the combat mode by fallback.
- Survival battles are `DefenseRoute` with an explicit `survive_timer_ms` objective modifier.
- During a survival timer battle, waves are driven by authored wave data / wave pools.
- When `survive_timer_ms` is reached and the protected objective is still alive, the battle ends immediately with player victory, even if enemies remain alive.

## Source Of Truth Order

Verify facts in this order during implementation:

1. Runtime code.
2. Live RON/data.
3. Unity-facing setup/update/snapshot command contracts.
4. Latest policy docs.

Do not trust this plan over runtime code. If implementation reveals a better long-term shape, record the evidence in `EXPERIMENT_NOTES.md` and apply it unless it requires a user policy decision.

## Current Observed Problem

Latest Unity wire log showed:

```text
confirm_enter_node ok
battle_setup_snapshot
battle_update server_battle_time_ms: 0
  BattleStart
  UnitSpawned opponent
  BattleEnd winner: Opponent
```

The selected setup was:

```text
encounter_id: suppress_spider_bud
battlefield_template_id: surrounded_medium_breach_01
mission_variant: Encirclement
```

Root cause from code reading:

- `suppress_spider_bud` has `node_type: Some(Defense)` but no explicit `mission_variant`.
- Battlefield fallback can choose `Surrounded`.
- `fallback_mission_variant_for_archetype(Defense, Surrounded)` returns `Encirclement`.
- Encirclement currently builds a `SurviveUntil(45_000)` win condition without the normal Defense protected objective.
- `compute_winner(0)` sees no player combatant before deployment and returns `Opponent`.
- The battle ends before movement ticks or deployment commands can matter.

## Target Runtime Policy

### DefenseRoute

Default live DefenseRoute win/loss:

- Protected objective destroyed -> Opponent victory.
- Required waves/groups are completed according to scenario policy -> Player victory.
- Deployment starts empty and must not be interpreted as player defeat.

### Survival Timer Modifier

Survival is not a separate `CombatMissionVariant`.

If a DefenseRoute encounter has `survive_timer_ms`:

- Protected objective destroyed before timer -> Opponent victory.
- Battle reaches `survive_timer_ms` while the objective is alive -> Player victory immediately.
- Enemy waves during the timer come from wave data / wave pool data.
- Remaining live enemies at timer expiry do not block victory.
- Wave defeat is pressure management, not the survival battle win condition.

### Battlefield Archetype

Battlefield archetype affects battlefield template selection, spawn/deployment/route layout, and presentation metadata.

It must not implicitly change:

- mission runtime model,
- win condition,
- reward policy,
- Unity command availability.

### Removed / Deprecated Policy

- Remove `SplitRoom` as an official live mission mode.
- Do not keep `CombatMissionVariant::SplitRoom` as a live-compatible variant.
- Do not use `Surrounded -> Encirclement` fallback.
- Do not preserve `Encirclement` as a separate runtime win-condition model.
- Do not introduce a compatibility layer accepting both old Encirclement semantics and new timer modifier semantics.

## Initial Code Reading Targets

Start by reading:

- `src/game/combat_preview/types.rs`
  - `CombatMissionVariant`
  - compatibility rules
- `src/game/combat_setup/mission_policy.rs`
  - `starts_as_live_battle`
  - `fallback_mission_variant_for_archetype`
  - default tactical plan / win-condition policy
- `src/game/combat_preview/mod.rs`
  - battlefield archetype and mission variant resolution
- `src/game/data/pve_data.rs`
  - encounter RON schema
- `src/game/data/validation.rs`
  - encounter validation
- `src/game/events/combat.rs`
  - preview-to-scenario conversion tests
- `src/game/battle/core/sim.rs`
  - `compute_winner`
- `src/game/world/combat.rs`
  - live battle start, setup snapshot, reward resolution
- live RON:
  - `/mnt/f/work/simulator/game_resources/data/pve/encounters.ron`
  - `/mnt/f/work/simulator/game_resources/data/map/battlefield_templates.ron`

## Implementation Plan

1. Audit current mission variant flow.
   - Find all reads/writes of `CombatMissionVariant::{Encirclement, SplitRoom}`.
   - Separate actual runtime behavior from display/reward/test leftovers.
   - Record findings in `EXPERIMENT_NOTES.md`.

2. Remove SplitRoom runtime mode.
   - Remove or deprecate `CombatMissionVariant::SplitRoom` from compatibility/live policy.
   - Ensure SplitRoom battlefield templates are not selected for live encounter fallback.
   - Update tests that expected SplitRoom compatibility.
   - If removing the enum entirely causes unnecessary serialization migration risk, stop and record the reason; prefer removal unless live data/DTO compatibility makes that unsafe.

3. Stop archetype-driven mission variant fallback.
   - `mission_variant` should default from node type: Defense -> Defense, Boss -> Boss.
   - `Surrounded` should remain a valid battlefield archetype for normal DefenseRoute.
   - Remove `Defense + Surrounded -> Encirclement` fallback.
   - Add focused tests proving a Defense encounter with Surrounded battlefield remains `mission_variant: Defense`.

4. Add explicit survival timer data.
   - Prefer a simple encounter field such as `survive_timer_ms: Option<u64>` if it fits current RON schema cleanly.
   - If existing schema strongly favors a structured objective field, evaluate and record the choice before editing.
   - This is a live RON schema change. Update validation and docs accordingly.

5. Convert survival runtime to DefenseRoute timer modifier.
   - Default DefenseRoute still creates/protects the defense objective.
   - If `survive_timer_ms` is present, build a DefenseRoute scenario whose win condition preserves the protected objective loss condition and adds timer victory.
   - Do not allow an empty deployment state to be a defeat condition for DefenseRoute.
   - Avoid creating a new separate battle mode or dual schema.

6. Update Unity-facing DTO/contracts if fields change.
   - `battle_setup_snapshot` or combat preview must expose survival timer info if Unity needs objective UI.
   - Update external canonical docs in `/mnt/f/unity projects/ark/docs` when DTO shape or meaning changes:
     - `unity_core_contract.md`
     - `core_unity_battle_setup_snapshot_contract.md`
     - `core_unity_battle_update_contract.md` if battle update/checkpoint meaning changes.

7. Update core docs.
   - `docs/game_rulebook.md`: DefenseRoute single runtime model, SplitRoom removal, survival timer policy.
   - `docs/README.md`: goal index.
   - Other domain docs only if code changes touch their source-of-truth area.

8. Replace tests around old policy.
   - Delete or rewrite tests that prove Encirclement as a separate runtime mode.
   - Add tests for:
     - Surrounded battlefield does not imply Encirclement.
     - Spider Bud / normal Defense encounter does not end at 0ms on Surrounded battlefield.
     - Survival timer battle wins at timer even if enemies remain.
     - Survival timer battle loses if protected objective is destroyed before timer.
     - live RON loading validates survival timer data.
     - setup snapshot / preview exposes survival objective data if exposed to Unity.

9. Run focused validation after each slice.
   - Record every failed command and fix in `EXPERIMENTS.md`.

## Expected Files To Touch

Likely:

- `src/game/combat_preview/types.rs`
- `src/game/combat_setup/mission_policy.rs`
- `src/game/combat_preview/mod.rs`
- `src/game/data/pve_data.rs`
- `src/game/data/validation.rs`
- `src/game/events/combat.rs`
- `src/game/battle/scenario.rs`
- `src/game/battle/core/sim.rs`
- `src/game/world/combat.rs`
- `src/game/behavior.rs`
- `src/game/world/state.rs`
- `src/game/world/tests/combat.rs`
- `/mnt/f/work/simulator/game_resources/data/pve/encounters.ron` if adding survival-timer live content
- `/mnt/f/unity projects/ark/docs/*` if Unity-facing DTO/contract changes
- `docs/game_rulebook.md`
- `docs/README.md`

Touch only what is required by the goal.

## Non-Goals

- Do not redesign all combat modes.
- Do not introduce a generic objective scripting framework unless simple fields prove insufficient.
- Do not implement SplitRoom.
- Do not implement new wave pool selection unless required by existing live data; survival timer should consume existing authored wave data/wave pool hooks.
- Do not change movement presentation/event transport.
- Do not change targeting, damage immunity, or skill policies.
- Do not rebalance enemies, rewards, or survival durations except where a test fixture needs explicit minimal data.

## Completion Criteria

The goal is complete when:

- `DefenseRoute / Defense` is the only official non-boss live runtime mission model.
- `SplitRoom` is removed from official live mission variant policy.
- `Surrounded` battlefield selection no longer changes a Defense encounter into Encirclement.
- Survival timer is represented as explicit encounter/objective data, not as a separate runtime mode.
- Survival timer battle ends in player victory at timer expiry if the protected objective is alive, regardless of remaining enemies.
- Protected objective destruction still causes defeat.
- A normal Defense encounter can start on `surrounded_medium_breach_01` without immediate BattleEnd at 0ms.
- Unity-facing setup/preview DTOs expose the survival objective timer if Unity needs to render it.
- Docs and tests match the implemented contract.
- No dual schema, compatibility adapter, ignored legacy test, or fallback path preserves the old Encirclement runtime behavior.

## Verification Commands

Use focused tests first, then broaden.

Suggested commands:

```text
cargo test -p game_core encirclement -- --nocapture
cargo test -p game_core mission_variant -- --nocapture
cargo test -p game_core defense_route -- --nocapture
cargo test -p game_core live_defense -- --nocapture
cargo test -p game_core --test ron_loading -- --nocapture
cargo check -p game_core
cargo test -p game_core
```

If server DTO / WebSocket message shape changes:

```text
cargo check -p game_server
cargo test -p game_server battle -- --nocapture
```

Run the exact relevant test names discovered during implementation rather than relying only on broad filters.

## Stop Conditions

Stop and report questions before coding further if any of these appear:

- Removing `CombatMissionVariant::SplitRoom` would require save migration or break live serialized data in a way not covered by current tests.
- Survival timer needs richer semantics than “timer expiry wins immediately if protected objective lives.”
- Wave pool data requires a new schema rather than using current authored wave data.
- Unity-facing DTO shape changes are larger than adding/displaying a timer field.
- Boss combat or non-Defense combat starts depending on old Encirclement/SplitRoom semantics.
- Existing content deletion/replacement is needed beyond removing unsupported SplitRoom policy.

## Reporting Requirements

On completion report:

- Change summary.
- Removed legacy behavior.
- New fixed contract.
- Tests/probes updated.
- Docs updated.
- Remaining risks.
- Verification commands run.
- Whether any user policy question remains.
