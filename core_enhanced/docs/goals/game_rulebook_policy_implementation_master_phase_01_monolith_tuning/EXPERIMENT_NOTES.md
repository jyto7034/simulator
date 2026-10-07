# Game Rulebook Policy Implementation Master Phase 01 — Notes

## Structural Judgment

- This is a Phase Goal for `GRB-WP-002`, not a new policy campaign.
- Direct dependency is completed Preflight `ready_for_wp002`; there is no preceding production Phase.
- P01 owns live stat tuning and representability validation only. P02 owns progression, cycles and V4.

## Source-Of-Truth Boundary

- Live RON owns adjustable initial values.
- Core run-policy loader owns strict publication validation.
- Runtime derived projection must consume validated values without silently saturating.
- Reward/wave/rounding and public contracts remain current and outside this Phase.

## Dirty Worktree Warning

- Both expected production target files were already modified at activation.
- Preflight observed zero target-field hunk overlap, but P01 must inspect current diffs before relying on that result.
- Existing changes are user-owned. This Phase will not restore, reset, stash, whole-file replace or broadly format.

## External Consumers And Omitted Validation

- P01 has no DTO/save/server/Unity shape change.
- Unity inspection remains omitted under the user's explicit development-tree decision; no Unity claim is made.

## Pending Technical Investigation

- The target validation/resolver/RON hunks do not overlap the existing tactical-resource changes.
- The four composed values are `u32` percentages. Their product is calculated through `u64`, then projected to
  `u32`; current projection saturates on representability failure.
- Each combat difficulty profile has exactly one validated `MonolithRemanifestation` role tuning. Loader validation
  can therefore check both stage multipliers against all four role percentages before publication.
- Resolver should still use checked projection and return `None` for a programmatically constructed invalid policy,
  while normal live publication makes that branch unreachable.
- Existing resolver and combat-preview tests directly cover target-power and enemy HP/attack/defense composition;
  they must be updated to 150/200-derived values. Reward/wave assertions remain unchanged.

## Applied Alternatives

- No arbitrary balance maximum will be introduced.
- No fallback to old 100/125 or saturation to `u32::MAX` will be retained as compatibility.
- No P02 field/schema will be added early.

## Remaining Risks

- A derived stat may use a different base/value path than the four listed fields.
- Validation at parse time may lack the real base stat needed for exact composition; investigation must find the
  actual publication authority rather than add a weak approximate cap.
- Existing user modifications may have changed the loader since Preflight.
