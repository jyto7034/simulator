# DefenseRoute Single Runtime Model Notes

## Final User Policy

Confirmed policy:

- Keep DefenseRoute as one live runtime combat model.
- Remove SplitRoom mode from official runtime policy.
- Do not treat Encirclement as a separate runtime combat mode.
- Survival mode is DefenseRoute plus an explicit timer objective.
- Waves during the survival timer are decided by wave data / wave pool data.
- At `survive_timer_ms`, if the protected objective is alive, the player wins immediately even if enemies remain.
- Battlefield archetype is map shape / layout classification. It must not change combat mode by fallback.

## Current Known Bug

The latest Unity wire log showed a normal Spider Bud battle ending immediately:

```text
battle_update server_battle_time_ms: 0
BattleStart
UnitSpawned opponent
BattleEnd winner: Opponent
```

Code-reading diagnosis:

- `suppress_spider_bud` has no explicit mission variant.
- A `Surrounded` battlefield was selected.
- Current fallback maps `Defense + Surrounded` to `Encirclement`.
- Encirclement uses `SurviveUntil`.
- No player unit is deployed at battle start.
- `compute_winner(0)` treats `player_alive == false` as defeat.

This is not a Unity movement/rendering problem. The core ends the battle before movement can start.

## Design Judgement

The old split between `Defense`, `Encirclement`, and `SplitRoom` conflates two axes:

- battlefield shape/archetype,
- mission objective/win condition.

Long-term direction:

- Battlefield archetype remains data/layout.
- Objective modifiers express special mission goals.
- DefenseRoute runtime stays unified.

This keeps future options open:

- Surrounded normal Defense.
- Surrounded timed Defense.
- Corridor timed Defense.
- Future split-map Defense if UI/UX is designed later.

## Expected Policy Documentation Updates

When implementation lands, update:

- `docs/game_rulebook.md`
  - DefenseRoute single runtime model.
  - SplitRoom removal.
  - Survival timer modifier.
  - Archetype does not alter combat mode.
- External Unity canonical docs if DTO changes:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/core_unity_battle_setup_snapshot_contract.md`
  - `/mnt/f/unity projects/ark/docs/core_unity_battle_update_contract.md` only if battle update/checkpoint semantics change.

## Follow-Up Candidates Outside Goal

- Revisit whether `mission_variant` should remain in Unity-facing DTO as display metadata after runtime policy is unified.
- Design SplitRoom as a future DefenseRoute layout/UX policy if the game needs split-squad placement.
- Add richer wave pool authoring only if existing wave data cannot express survival pressure.

## Implementation Judgements

- `survive_timer_ms` is intentionally explicit encounter data. This avoids hidden behavior where battlefield archetype silently changes mission semantics.
- `survive_timer_ms` is valid only for explicit Defense encounters. Validation rejects missing node type, non-Defense node type, and zero timer values.
- `CombatMissionVariant::Encirclement` and `CombatMissionVariant::SplitRoom` were removed instead of kept as deprecated aliases. Keeping aliases would preserve a dual schema and make future live data ambiguous.
- `BattlefieldArchetype::SplitRoom` remains available as map/layout metadata. The removed policy is SplitRoom as a live mission mode, not split-room-shaped maps.
- Low-level `WinCondition::SurviveUntil` still exists in battle core as a generic primitive, but PVE authoring and mission policy no longer expose it. Official survival content must use `ProtectUnitUntil` via `survive_timer_ms`.
- The Unity-facing setup snapshot exposes `survive_timer_ms` because Unity needs it for objective UI. Battle update/checkpoint semantics did not need a new source-of-truth rule.
