# Damage Feedback DTO Plan

## Current Decision

- Compute `feedback_tags` in `DamageResult` because `calculate_damage` already centralizes raw damage, final damage, damage type, critical state, and breakdown for both basic attacks and skill damage.
- Copy the computed tags into `TimelineEvent::HpChanged` when damage is applied.
- Healing and non-damage HP changes should serialize a stable empty tag list.

## Checklist

1. Add typed `DamageFeedbackTag` contract - done.
2. Add `feedback_tags` to `DamageResult` - done.
3. Add `feedback_tags` to `TimelineEvent::HpChanged` - done.
4. Update damage and HP change recording paths - done.
5. Add focused tests for `critical`, `mitigated`, `fixed_damage`, `immune`, and empty healing tags - done.
6. Run focused validation commands and record results - done.

## Completion Summary

- `DamageFeedbackTag` is a typed serde contract with snake_case JSON values.
- `DamageResult.feedback_tags` is computed centrally by `calculate_damage`.
- Damage `HpChanged` events copy `DamageResult.feedback_tags`.
- Non-damage HP changes emit an empty `feedback_tags` array.
- `piercing`, `shield`, `blocked`, and `resisted_status` remain enum-level contract candidates only; current runtime does not emit dummy tags for effects it cannot prove.
