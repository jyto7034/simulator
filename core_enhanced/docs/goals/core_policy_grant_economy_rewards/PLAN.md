# Grant, Economy, Reward Policy Implementation

## Objective

Make all resource, item, equipment, XP, fragment, research, and reward mutations flow through one canonical grant execution boundary, with atomic reward behavior and explicit data validation.

## Policies Covered

- `canonical grant executor`
- `atomic reward claim`
- `reward granted fragment and research diff`
- `reward experience target policy`
- `enkephalin overflow policy`
- `remove reward tags`
- `explicit equipment reward pools`
- `forbidden abnormality reward validation`
- `remove automatic duplicate fragment conversion`
- `fragment dust resource source`

## Plan

1. Read every path that grants XP, equipment, fragments, research progress, Enkephalin, abnormality rewards, dust, starter rewards, shop purchases, event rewards, and maintenance outcomes.
2. Rename and raise `RewardExecutor` or equivalent into the canonical grant executor if code confirms it is the right foundation.
3. Remove reward `tags`; replace them with typed grant semantics, explicit pool policy, display category, and validation fields.
4. Make reward claim all-or-nothing: prevalidate all grants, apply none if any grant fails.
5. Add reward event diffs for skill fragments and research progress.
6. Replace random full-DB equipment grants with explicit pools and filter policy.
7. Convert forbidden abnormality rewards into validation failures.
8. Enforce Enkephalin overflow checks without gameplay caps or saturating increments.
9. Update focused grant/reward/shop/live data tests.

## Completion Conditions

- There is one authoritative grant execution boundary for all acquisition mutations in this scope.
- Reward tags are removed from live reward data and runtime interpretation.
- Partial reward grant cannot occur on validation or application failure.
- Equipment reward pools are explicit and data-validated.
- Forbidden abnormality grants fail validation.
- Enkephalin overflow is checked explicitly.
- Tests cover user-visible grant results and event diffs.

## Completion Report

Status: complete by implementation and verification on 2026-06-22.

Implemented:

- Raised reward execution into canonical `GrantExecutor` behavior for reward claims, combat reward XP, post-battle survival XP, admin grant commands, shop acquisition/sell returns, headquarters emergency supplies, maintenance dismantle outputs, starter equipment creation, and equipment-combination result creation.
- Removed reward `tags` and replaced policy checks with effect-derived `RewardGrantKind`.
- Removed `ForbiddenAbnormalityGrant` from reward effects/live data and moved the policy to schema/validation failure.
- Replaced random full-DB equipment grants with explicit `GrantEquipmentFromPool` and typed `equipment_pools` validation.
- Made reward session claim atomic with clone-preflight/commit.
- Added Enkephalin overflow checks through `Enkephalin::checked_add`.
- Removed automatic duplicate skill-fragment conversion.
- Added `RewardGranted`/`CombatRewardsGranted` skill-fragment, research-progress, and employee-XP diffs and forwarded them through the server payload.

Removed legacy:

- Reward metadata `tags`.
- `ForbiddenAbnormalityGrant`.
- `SkillFragmentStackingPolicy::ConvertAdditionalCopiesToResource`.
- `GrantEquipment { equipment_id: None }` random full-DB behavior.
- Direct runtime acquisition mutations outside `GrantExecutor` for this subgoal scope.

Verified:

- `cargo test --test ron_loading` - 18 passed.
- `cargo check -p game_server` - passed.
- `cargo test --lib -- --test-threads=1` - 500 passed.
