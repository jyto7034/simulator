# Combat Preview Threat Warnings Plan

## Current Decision

- Add typed `ThreatWarning`, `ThreatWarningTag`, `ThreatWarningStatus`, and `ThreatWarningSource` DTOs in `src/game/combat_preview.rs`.
- Add `CombatPreview.threat_warnings` and `BattlefieldInstance.threat_warnings`.
- Generate initial `Briefing` warnings from the already materialized `spawn_waves.enemy_entries` and unit metadata, avoiding a live RON schema change for now.
- Add seed-based `Rumor` warning generation with a low deterministic chance and no duplicate tag with real briefing warnings.
- Do not emit dummy warnings for tags that runtime/data cannot prove yet.

## Initial Proven Warning Sources

- `armored_enemy_possible`: enemy defense is above threshold.
- `high_magic_resist_enemy_possible`: enemy magic resist is above threshold.
- `fast_breakthrough_enemy_possible`: enemy movement speed is above threshold.

## Deferred Tags

- `air_enemy_possible`: no current air/flying source found in enemy metadata.
- `hard_to_block_enemy_possible`: no current enemy block resistance/source found in preview metadata.
- `shielded_enemy_possible`: no current shield source found in preview metadata.
- `regenerating_enemy_possible`: no current regeneration source found in preview metadata.

## Checklist

1. Add typed DTOs - done.
2. Add `threat_warnings` to `BattlefieldInstance` and `CombatPreview` - done.
3. Generate briefing warnings from concrete wave entries - done.
4. Add deterministic rumor warning selection - done.
5. Update tests and explicit `CombatPreview` constructors - done.
6. Run focused validation and record results - done.

## Completion Summary

- `CombatPreview.threat_warnings` is a typed object array.
- Each warning has `tag`, `status`, and `source`.
- `Briefing` warnings are generated from materialized enemy entries and unit metadata.
- Rumor warnings are deterministic, low chance, at most one entry, and do not duplicate real briefing tags.
- No legacy `threat_warning_tags` field was introduced.
