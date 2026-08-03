# Phase 05 Targeting And Battlefield Contract Notes

Status: active

## Current Active Child

- G09 Global reach and target-centered area delivery.

## Closed Child

- G11 `enemy_range_preview_removal`: Complete / High. Opponent checkpoint actors omit preview state;
  Player deployment/deployed previews and real telegraphs remain.
- G10 `skill_targeting_authority`: Complete / High. Every skill owns an explicit selection rule;
  weapon targeting profiles remain basic-attack-only.

## Fixed Guardrails

- Remove only enemy range preview. Preserve employee deployment/facing previews and allied manual
  skill targeting previews.
- Preserve executing attacks, explicit telegraphs, projectiles/areas, hazards, and active wave
  boundaries.
- Do not reserve an empty compatibility field. A future enemy preview requires a new intentional DTO.
- Preserve G10's explicit selection authority while G09 changes reach and footprint geometry.
