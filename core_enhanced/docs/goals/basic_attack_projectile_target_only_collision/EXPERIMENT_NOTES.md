# Basic Attack Projectile Target-Only Collision Notes

## Fixed Policy

- Basic attack execution eligibility remains tile-based.
- Projectile basic attacks are locked to the original target selected at attack release.
- Projectile runtime should store `attacker_owner_at_launch` or an equivalent immutable launch-side snapshot. Hostility at impact must not depend on the attacker still being present in live `units`.
- Continuous collision is allowed only against that original target body.
- Other units on the projectile path are not collision candidates.
- Impact-time validity is narrow:
  - target exists,
  - target alive,
  - target hostile to `attacker_owner_at_launch`,
  - target satisfies non-range targetability.
- Impact-time validity must not include:
  - tile range recheck,
  - `range_units`,
  - target usefulness recheck,
  - retargeting,
  - bystander/path collision.

## Open Implementation Questions

Resolved from runtime evidence during implementation:

- Maximum travel/lifetime: use launch-time estimated travel time from existing start/aim/speed data, matching target-locked skill projectile `max_travel_ms`. No extra grace multiplier is introduced.
- Re-evaluation cadence: reuse the existing skill projectile cadence of 1ms for bounded internal event windows.
- `expected_impact_time_ms`: keep the field as the initial presentation estimate; gameplay impact is the first locked-target sweep contact within the finite travel window.
- Launch owner: `ProjectileRecord` stores `attacker_owner_at_launch` directly; this is runtime-only state and does not require a Unity-facing DTO shape change.

No unresolved gameplay policy question was needed for this slice.

## Follow-Up Candidates Outside This Goal

- Share helper code between basic attack target-only projectile sweep and skill homing projectile sweep if duplication becomes high.
- Add richer Unity projectile presentation hints if target-follow interpolation needs a clearer contract.
- Add authored projectile lifetime fields later if weapon balance needs per-weapon control.
