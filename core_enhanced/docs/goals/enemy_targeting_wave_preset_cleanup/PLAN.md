# Enemy Targeting and Wave Preset Cleanup Plan

## Objective

Align the remaining documented combat/data contracts with runtime behavior by fixing two audited mismatches:

1. Enemy ranged basic attacks must follow the authored `targeting_profile` after blocker and persisted-target rules are satisfied.
2. Remove unused `CorrodedWavePreset.difficulty`. Floor scaling is the game's difficulty/pressure progression axis, and generated corroded wave strength is already controlled by `budget`, `count_range`, `role_mix`, and Floor policy multipliers.

This goal must make the long-term model clearer. Do not add compatibility layers, silent fallbacks, or test-only patches.

The targeting repair scope is the basic-attack target selection path used by enemy ranged units, especially fixed defense route enemies that stop, attack, reposition, and then either keep their persisted target or choose a new target. This goal must not silently redesign movement-goal acquisition helpers that still use continuous distance, such as `closest_enemy_in_attack_range`, unless the audit proves they are part of the same basic-attack target contract and can be covered by focused tests.

## Source Of Truth Order

Use sources in this order:

1. Runtime code in `src/game/battle/core/targeting.rs`, `src/game/battle/core/basic_attack.rs`, `src/game/wave_resolution.rs`, and preview/map encounter call sites.
2. Live RON in `../game_resources/data/enemies/corroded_wave_presets.ron`, encounter RON, weapon/equipment RON, and corroded employee profile RON.
3. Unity-facing DTO and snapshot contracts if a change would affect serialized output.
4. Canonical docs:
   - `docs/game_rulebook.md`
   - `docs/skill_target_contract.md`
   - `docs/skills/lobotomy_content_catalog.md`
   - external Unity docs only if DTO or JSON shape changes.

## Current Audit Findings

### Enemy Ranged Targeting

`docs/skill_target_contract.md` says ranged basic attacks use the weapon or unit `targeting_profile` as the source of truth.

Current runtime code filters valid basic attack candidates, but for non-player attackers it picks the nearest world-distance target before applying `targeting_profile`.

Expected long-term behavior:

- Blocker target remains highest priority.
- For enemy ranged route units, an existing persisted target remains valid after reposition if it is still alive, useful, and in tile range.
- If a new target must be selected, the authored `targeting_profile` sorts candidates.
- Deterministic tie-breakers remain stable.
- Basic attack eligibility remains tile-based. Do not reintroduce continuous-distance range eligibility.
- `closest_enemy_in_attack_range` is a separate movement-goal helper. If it still affects route-less enemy acquisition, either bring it under the same documented basic-attack target contract with tests or record it as a follow-up instead of mixing it into this cleanup accidentally.

### Corroded Wave Preset Difficulty Field

`PveEncounter.difficulty` has been removed from encounter selection and live RON. However `CorrodedWavePreset` still has a `difficulty: u8` field.

Expected long-term behavior:

- `CorrodedWavePreset.difficulty` is removed rather than renamed.
- Do not introduce `budget_tier`, `pressure_tier`, or a new wave tier field in this goal.
- Floor progression remains the official difficulty/pressure scaling source.
- Generated corroded wave strength remains authored through `budget`, `count_range`, `role_mix`, optional wave `budget_override`, and `floor_combat_scaling.generated_wave_budget_multiplier_percent`.

## Non-Goals

- Do not redesign all targeting profiles.
- Do not add new targeting profiles.
- Do not change player unit targeting semantics except where shared helper behavior requires it and tests prove it is intended.
- Do not change battle DTO shape unless runtime inspection proves Unity-facing data must change.
- Do not change encounter class, Floor scaling, boss omen, or research progression policies.
- Do not delete historical goal docs in this goal.

## Implementation Plan

### Phase 1: Runtime/Data Audit

- Confirm all uses of `CorrodedWavePreset.difficulty` are schema/live RON/test fixture references and not runtime behavior.
- Read enemy ranged attack selection flow:
  - blocker priority
  - persisted target validation
  - `ranged_reposition_until_ms`
  - `choose_attack_target_in_range`
  - `closest_enemy_in_attack_range` call sites, to confirm whether they are in or out of this goal's scope
  - targeting profile comparators
- Record findings in `EXPERIMENTS.md`.

### Phase 2: Enemy Targeting Repair

- Refactor candidate selection so non-player ranged target selection uses the authored `targeting_profile` when selecting a new target.
- Preserve blocker priority and persisted-target behavior.
- Keep basic attack range validation tile-based.
- Prefer a narrow repair in `choose_attack_target_in_range` if that is sufficient for fixed defense route ranged enemies.
- Do not leave route-less enemy basic attack acquisition on a contradictory continuous-distance policy without explicitly recording why it is out of scope.
- Add focused tests proving:
  - enemy ranged attacker with `LowDefenseFirst` chooses the lower-defense in-range target over a nearer higher-defense target.
  - enemy ranged attacker with `LowMagicResistFirst` chooses the lower-magic-resist target.
  - blocker still overrides targeting profile.
  - persisted target remains selected during the valid post-reposition flow.
  - out-of-range targets are still excluded by tile range before profile sorting.
  - route-less/non-fixed enemy acquisition is either covered by the same policy or documented as a follow-up if it remains on `closest_enemy_in_attack_range`.

### Phase 3: Wave Preset Schema Cleanup

- Remove `CorrodedWavePreset.difficulty`.
- Update live RON and tests/fixtures.
- Do not add a replacement wave tier field.
- Add/adjust validation so stale `difficulty` fails rather than being silently accepted.
- Add focused loading tests proving live RON parses with the new schema.

### Phase 4: Docs And Contract Sync

- Update canonical docs if they mention the old wave preset field or enemy ranged targeting behavior.
- If Unity-facing JSON DTO does not change, state that explicitly in `EXPERIMENT_NOTES.md`.
- Do not update external Unity docs unless serialized DTO shape changes.

### Phase 5: Verification

Run focused tests after each phase, then a broader check:

- Focused targeting tests.
- Focused wave/RON loading tests.
- `cargo check` or the repository's equivalent broad validation command.
- If there is a live probe covering battle startup or wave preview, run it if practical and record result.

## Completion Criteria

- Enemy ranged basic attacks follow `targeting_profile` for new target selection without losing blocker/persisted-target policy.
- Basic attack eligibility remains tile-based.
- Any remaining continuous-distance enemy acquisition path is explicitly tested as intended behavior or documented as a follow-up risk, not left as an accidental contradiction.
- `CorrodedWavePreset.difficulty` no longer exists in runtime schema, live RON, or positive test fixtures. A negative stale-data test may mention it only to prove deserialization failure.
- No replacement wave tier field is added.
- Stale `difficulty` in wave preset RON fails validation/deserialization rather than being silently accepted.
- Focused behavior tests cover the changed contracts.
- Live RON/data loading is verified.
- Canonical docs match runtime behavior.
- Final report includes:
  - changed behavior,
  - renamed/removed data field,
  - tests added/updated,
  - removed legacy,
  - remaining risks,
  - exact verification commands.

## Stop Conditions

Stop and ask the user if any of these appear:

- Applying `targeting_profile` to enemy ranged units conflicts with a documented enemy archetype behavior.
- Removing `CorrodedWavePreset.difficulty` reveals hidden runtime behavior that depends on it.
- The cleanup would require Unity-facing DTO shape changes.
- Live RON contains content that cannot be migrated without deciding new balance values.
- `closest_enemy_in_attack_range` proves to be user-visible enemy basic attack behavior that cannot be reconciled with the tile/profile contract in this goal's scope.
- A broad refactor outside targeting or wave preset schema becomes necessary.
