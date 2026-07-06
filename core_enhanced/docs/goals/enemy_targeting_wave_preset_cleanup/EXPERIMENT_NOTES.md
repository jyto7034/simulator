# Enemy Targeting and Wave Preset Cleanup Notes

## Initial Policy Notes

- This goal should not restore continuous-coordinate basic attack eligibility. Basic attack target eligibility remains tile-based.
- Enemy ranged attackers are allowed to have authored targeting profiles. Once blocker and persisted-target policy are satisfied, new target selection should honor that profile.
- Blocker priority is stronger than ranged targeting profile.
- Existing valid persisted target after ranged reposition is stronger than searching for a new target.
- The first repair target is `choose_attack_target_in_range`, which currently applies a nearest-distance branch for non-player attackers after tile/usefulness filtering.
- `closest_enemy_in_attack_range` is a separate movement-goal helper that still sorts by continuous world distance. Do not accidentally fold it into this cleanup without focused tests. If route-less enemy acquisition remains on that helper, record it as an explicit follow-up risk.
- `CorrodedWavePreset.difficulty` is unused in runtime wave generation and should be removed, not renamed.
- Floor progression is the game's difficulty/pressure progression axis.
- Generated corroded wave strength is already represented by `budget`, `count_range`, `role_mix`, optional wave `budget_override`, and `floor_combat_scaling.generated_wave_budget_multiplier_percent`.

## Rejected Alternatives

- `budget_tier`: rejected for now because no wave tier concept exists in runtime.
- `pressure_tier`: rejected for now because `pressure` already exists as authoring metadata and Floor policy controls scaling.
- Keeping `difficulty` as deprecated metadata: rejected because it preserves an obsolete difficulty axis and creates confusion with Floor scaling.

Do not keep `difficulty` as a deprecated alias and do not add a replacement field unless a future balance policy explicitly introduces wave tiers.

## Known Runtime Touchpoints

- `src/game/battle/core/targeting.rs`
- `src/game/battle/core/basic_attack.rs`
- `src/game/data/corroded_wave_data.rs`
- `src/game/wave_resolution.rs`
- `src/game/combat_preview/mod.rs`
- `../game_resources/data/enemies/corroded_wave_presets.ron`

## Implementation Notes

- No Unity-facing JSON DTO shape changed in this goal.
- `CorrodedWavePreset.difficulty` was a RON/schema authoring field only. Removing it does not change spawn wave DTOs or battle transport DTOs.
- A negative schema test intentionally contains a stale `difficulty` field to prove that old wave preset data fails deserialization. Positive schema, live RON, and fixtures no longer carry the field.
- `closest_enemy_in_attack_range` remains a continuous-distance movement-goal helper. This goal repaired the documented basic-attack target selection path in `choose_attack_target_in_range`; route-less/non-fixed enemy acquisition cleanup remains a follow-up candidate if design requires all such acquisition to become tile/profile-based.

## Follow-Up Candidates Outside This Goal

- Additional targeting profiles such as `BossFirst`, `EliteFirst`, or `ClosestFirst`.
- Enemy archetype-specific AI beyond the existing targeting profile policy.
- Route-less/non-fixed enemy movement-goal acquisition cleanup if `closest_enemy_in_attack_range` should also become tile/profile-based.
- Full content rebalance of generated wave presets.
- Historical draft document deletion/archive cleanup.

## Current Open Questions

No user policy questions are open at goal creation.

The `difficulty` field removal policy is settled for this goal. New wave tier policy would be a separate future decision.
