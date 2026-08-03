# Placeholder Content Policy Canonicalization

Status: complete

Implementation complete: true

Started: 2026-07-18

Master: `docs/goals/documentation_code_alignment_corrections/PLAN.md` G05

Findings: `DCA2-GAME-002`, `DCA2-BATTLE-001`, `DCA2-CONTENT-001`, `DCA2-GAME-003`

Policies: Master P02–P05

## Objective

Make durable current/future/formal/placeholder wording exact for skill fragments, abnormality ranges,
WhiteNight and Maintenance without redesigning or migrating current placeholder content.

## Approved Policies

- Formal skill fragments are active-skill-centered; current `BasicAttackModifier` entries are
  temporary placeholders and remain unchanged.
- Formal Boss/Elite/special abnormalities require authored attack range; current generic-range
  entries are placeholders and remain unchanged.
- WhiteNight detailed mechanics are future/undecided; the current one-step/research-based shape is a
  temporary placeholder and remains unchanged.
- Equipment Maintenance actions require an explicit per-item RON recipe. Missing recipe means typed
  disabled, never a generic fallback. Current valid live coverage is only `standard_armor`.

## In Scope

- Durable gameplay, targeting, content and live-index wording nearest each policy.
- Exact separation of formal design rules from current placeholder observations.
- Maintenance recipe-gate and current coverage wording.
- Static searches proving no durable document upgrades placeholder mechanics to completed formal
  design.

## Out Of Scope

- Production code, live RON, tests, balance values or new content.
- Fragment active-skill migration.
- Authored ranges for the current 30 abnormalities.
- WhiteNight mechanic/reward implementation.
- New equipment recipes.

## Implementation Plan

1. [x] Inventory current durable claims and exact live/runtime evidence for the four policies.
2. [x] Correct nearest canonical documents with separate formal/current-placeholder/future sections.
3. [x] Preserve exact current live observations in derived indexes with explicit placeholder labels.
4. [x] Run contradiction/current-future vocabulary searches and diff checks.

## Completion Conditions

- Formal policy is not inferred from current placeholder data.
- Current placeholder behavior is not presented as completed formal design.
- WhiteNight future design is not a current implementation promise.
- Maintenance explicitly requires authored recipes and reports only current `standard_armor` coverage.
- No production artifact changes under this child.

All completion conditions satisfied on 2026-07-18.

## Result

- Formal fragment policy and 29 current direct-modifier placeholders are now stated separately.
- Formal special-abnormality range authoring and the 30 current generic-range placeholders are
  stated separately.
- WhiteNight's detailed chain is explicitly future/undecided; its current one-step/research trace is
  explicitly a placeholder.
- Maintenance is explicit-recipe-only, missing-recipe-disabled, no-fallback, with current equipment
  coverage limited to `standard_armor`.
- No code, test or live RON changed under G05.

## Stop Conditions

- Any supposedly placeholder mechanic is shown to be shipped formal content.
- Accurate wording would require changing current runtime behavior.
