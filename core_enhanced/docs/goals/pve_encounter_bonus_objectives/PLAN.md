# Pve Encounter Bonus Objectives Plan

## Objective

Implement encounter-authored bonus objectives that grant additional abnormality response research on victory.

This goal adds the PveEncounter data contract and battle/result evaluation for `ClearWithin` and `DecisiveDamage`. It assumes run-local abnormality research state already exists. It does not implement repeat encounter weighting or boss omen chains.

## Source Of Truth Order

Verify facts in this order during implementation:

1. Runtime battle event/result stats, damage application, PvE encounter resolution, reward, and research state code.
2. Live RON/data for PvE encounters, skills, damage events, and run policy.
3. Unity-facing battle result/snapshot contracts and live WebSocket JSON.
4. Latest policy docs, especially `docs/endless_mode_research_policy_draft.md`.

Do not trust this plan over runtime behavior. If code reading shows a better long-term model, record the evidence in `EXPERIMENT_NOTES.md` and apply it unless it requires a user policy decision.

## Dependencies

- Depends on `docs/goals/endless_abnormality_research_state/`.
- Can be implemented without repeat encounter weighting.

## Current Policy To Implement

- Bonus objectives live on `PveEncounter`.
- If `bonus_objectives` is absent, no bonus-objective evaluation runs.
- There is no silent global fallback bonus objective.
- Bonus objectives are evaluated only on victory.
- Retreat, defeat, and node failure grant no bonus-objective research.
- Each bonus objective is judged independently.
- Each bonus objective id can be satisfied at most once per battle.
- Bonus objective research stacks with base victory research.
- Bonus objective reward is response research progress, not an inventory item.
- Optional presentation may say an abnormality part was obtained.
- Initial objective types:
  - `ClearWithin(time_ms)`.
  - `DecisiveDamage(minimum_damage_percent_of_max_hp)`.
- Recommended bonus research range is `20..=40`, authored per encounter.
- All bonus objective values must be live-RON configurable.
- Encounters with suppression research or bonus objectives must explicitly identify their primary abnormality target.

## In Scope

- Add `bonus_objectives` to PvE encounter live RON schema.
- Add typed objective definitions for `ClearWithin` and `DecisiveDamage`.
- Validate objective ids are unique per encounter.
- Validate objective reward amounts are non-zero.
- Validate objective target policy:
  - primary abnormality target exists;
  - multi-target encounters must explicitly define target ids before being counted.
- Track per-battle objective satisfaction state.
- Evaluate `ClearWithin` using battle start as origin.
- Evaluate `DecisiveDamage` from a single non-DoT effective damage source against the primary abnormality target.
- Exclude DoT and repeated small tick summing from `DecisiveDamage`.
- Apply bonus research only if the encounter is ultimately won.
- Include bonus objective outcomes in result DTO/logs if there is an existing result surface or a new DTO is needed.
- Add focused tests for data validation and battle-visible behavior.

## Out Of Scope

- New objective types beyond `ClearWithin` and `DecisiveDamage`.
- Spawn-relative timing objectives.
- Flat damage threshold objectives.
- Inventory items for abnormality parts.
- Unity result screen implementation.
- Repeat encounter weighting.
- Boss omen/provisional boss chains.

## Target RON Shape

Exact field names may change after reading current `PveEncounter` style, but the meaning should be:

```ron
suppression_research: (
    required: 100,
    victory_gain: 20,
    bonus_objectives: [
        (
            id: "fast_clear",
            condition: ClearWithin(time_ms: 90000),
            research_bonus: 20,
            presentation: Some("abnormality_part_obtained"),
        ),
        (
            id: "decisive_damage",
            condition: DecisiveDamage(minimum_damage_percent_of_max_hp: 10),
            research_bonus: 20,
            presentation: Some("abnormality_part_obtained"),
        ),
    ],
)
```

If implementation keeps base research on another field, document the final field layout in `EXPERIMENT_NOTES.md`.

## Evaluation Rules

`ClearWithin`:

```text
victory_battle_time_ms - battle_start_ms <= time_ms
```

- Origin is battle start.
- If a boss appears late, omit `ClearWithin`; do not silently switch to spawn time.

`DecisiveDamage`:

```text
single_source_effective_damage * 100 >= target_max_hp * minimum_damage_percent_of_max_hp
```

- Non-DoT damage only.
- Use integer percent comparison. Do not use floating point ratio thresholds for authored decisive damage requirements.
- One basic attack impact, skill step impact, projectile impact, hitscan impact, or tile area impact can count if modeled as one non-DoT damage source.
- Damage to normal enemies, corroded employees, summoned units, or facility entities does not count.
- The hit does not need to be the final hit.
- Record satisfaction during battle, but grant only on victory.

## Implementation Sketch

1. Audit current `PveEncounter` schema, combat result stats, damage event data, and reward/research mutation paths.
2. Add typed RON definitions and validation.
3. Evaluate bonus objectives from the completed `BattleEventLog` and typed `BattleResultStatsDto`.
4. Wire battle completion to grant satisfied objective research on victory.
5. Add result/snapshot DTOs for satisfied objectives.
6. Add focused tests and live RON loading tests.
7. Update docs/probes.

## Completion Criteria

- [x] Live RON can author bonus objectives.
- [x] Invalid objective data fails validation.
- [x] `ClearWithin` works from battle start time only.
- [x] `DecisiveDamage` counts only one non-DoT source against the primary abnormality target.
- [x] Defeat/retreat/node failure never grants bonus research.
- [x] Victory grants base research plus satisfied bonus research.
- [x] Result/log DTOs expose enough information for presentation.
- [x] Focused tests and broad checks pass or failures are recorded.

Implementation summary:

- Added `PveEncounter.suppression_research.bonus_objectives`.
- Added typed `ClearWithin` and `DecisiveDamage` objective conditions.
- Added authoring validation for unique objective ids, non-zero research bonus, valid condition parameters, and primary abnormality target spawn policy.
- Added completed-event-log evaluation for objective outcomes.
- Added `CombatBattleState.bonus_objectives`, battle record export, and `selected_event.bonus_objectives`.
- Wired Endless combat result completion to add satisfied objective research to base suppression victory research.
- Updated local and Unity-facing docs for the final DTO/source-of-truth policy.

## Stop Conditions

Stop and report questions instead of deciding silently if implementation discovers:

- Existing battle events lack enough source information to distinguish DoT from non-DoT.
- Existing damage result data cannot identify the primary abnormality target.
- Existing result DTO requires Unity-facing shape decisions.
- Multi-primary-target encounters already exist and need policy before validation can be strict.
- Reward/research mutation would bypass the canonical reward path.
