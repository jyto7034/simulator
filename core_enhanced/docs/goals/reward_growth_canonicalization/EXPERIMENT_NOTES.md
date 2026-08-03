# Reward And Growth Canonicalization Notes

Goal: G23

Status: active

## Fixed Guardrails

- Current behavior is the subject; no balance or content invention.
- G24 owns inventory admission and quantity semantics.
- G29 owns the future game-global encounter compendium.
- G21 owns authoritative Safe Zone research delivery timing.

## Resolved Findings

- Generic `RewardEffect` capability is broader than current live PVE authoring. Canonical text now
  distinguishes representable effects, fixed encounter effects, and Reward-node selection.
- Authored XP is a sorted/deduplicated total-pool split. Automatic survival XP is separately granted
  per employee-level survivor on both committed victory and defeat.
- XP thresholds are an authored vector; the last entry repeats and one grant may cross levels.
- Employee level is stored. Battle tier is derived from level and projected but is not currently a
  stat multiplier.
- GrowthStack is separate from level. KillStack applies flat attack before equipment modifiers; no
  production gameplay producer currently exists.

## Follow-Up Candidates

- Unity has no dedicated XP diff presentation reader. This is not a G23 contract defect because the
  root roster snapshot remains authoritative and this goal does not add UI scope; a future result-UX
  goal may consume the existing typed diffs/outcome changes without changing Core progression.
