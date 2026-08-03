# Placeholder Content Policy Canonicalization Notes

Status: complete

## Starting Evidence

- The user explicitly chose to leave temporary fragment and range data untouched.
- WhiteNight design remains undecided and is deferred.
- Maintenance must be per-item recipe-gated; only `standard_armor` currently has equipment recipes.
- G04 has already finalized weapon-only block wording; G05 must not reopen or duplicate that authority.

## Resume Point

- Child complete. Resume from Master G07.

## Final Policy Boundaries

- Formal fragment: active skill or active-skill result. Current exact placeholder observation:
  abnormality-derived 28 plus concept starter 1 direct `BasicAttackModifier`; two abnormality-derived
  fragments are `ActiveSkill`.
- Formal range: Boss/Elite/special authored range/skill. Current exact placeholder observation: 30
  live abnormalities use generic missing-pattern behavior.
- WhiteNight: detailed chain future/undecided. Current exact placeholder observation: one-step live
  encounter plus response-research acquisition of a trace modifier.
- Maintenance: per-definition authored dismantle/enhancement recipe only. Missing recipe projects
  `can_execute: false` with existing operation disabled reason. Current live coverage:
  `standard_armor` only.

## Validation Summary

- Focused formal/current/future/recipe vocabulary search found every required distinction.
- Core and external Markdown `git diff --check` passed.
- No runtime tests were run because G05 changed documentation only and relied on the already green
  G04 runtime baseline.
