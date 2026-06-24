# Experiment Notes

## Policy Notes

- Grant semantics should be typed. Do not reintroduce tag-like control gates under a new name.
- If atomic claim changes a failure/consume/reward timing not already covered by policy, record `사용자와 정책 논의 필요`.
- Fragment dust is an independent skill-fragment economy resource, not equipment material DB state.
- Resolved implementation rule: reward metadata no longer stores generic `tags`; runtime policy checks use `RewardGrantKind` derived from `RewardEffect` only. This is not an authoring source and must not gain independent live data fields.
- Resolved implementation rule: `ForbiddenAbnormalityGrant` is removed from live rewards and runtime effect execution. Abnormality materialization as a reward is enforced by absence/schema/validation, not by a grant that intentionally fails.
- Resolved implementation rule: random equipment reward authoring uses `GrantEquipmentFromPool { pool_id }` and `RewardDatabase.equipment_pools` typed filters. `GrantEquipment` now requires an explicit equipment id.
- Resolved implementation rule: reward session claim is all-or-nothing at the official reward path. Implementation currently uses clone-preflight and commit; no partial state is committed if any reward/effect fails.
- Resolved implementation rule: Enkephalin increases use `Enkephalin::checked_add`; gameplay cap remains absent and arithmetic overflow fails the command.
- Resolved implementation rule: automatic duplicate skill-fragment conversion is removed. The only current stack policies are reject additional copies or stack copies; dust/resource acquisition happens through explicit dismantle/reward flows.
- Resolved implementation rule: `RewardGranted` and `CombatRewardsGranted` expose skill fragment acquisition and skill fragment research progress diffs in addition to inventory diff. The server behavior-result payload forwards those fields directly.
- Resolved implementation rule: `GrantExperience` requires explicit `ExperienceTargetPolicy`. XP mutation is handled by `GrantExecutor` with `GrantExecutionContext`, and `RewardGranted`/`CombatRewardsGranted` expose `employee_experience_diffs`.
- Resolved implementation rule: `ExperienceTargetPolicy::CombatParticipants` requires a combat context. Absence of context is command failure; an empty eligible participant set from a real combat context is a valid no-op.
- Resolved implementation rule: admin grant commands are debug wrappers over `GrantExecutor`. They use typed grant effects and clone-preflight/commit rather than direct inventory, skill fragment, material, or dust mutation.
- Resolved implementation rule: shop purchase keeps price/stock/duplicate/capacity validation in the shop domain, but final item acquisition is a typed grant effect. Shop sell keeps item removal in the shop domain, but Enkephalin gain is a typed grant effect.
- Resolved implementation rule: headquarters emergency supplies grants Enkephalin through `GrantExecutor`; headquarters node completion remains headquarters domain behavior.
- Resolved implementation rule: maintenance dismantle keeps maintenance-specific eligibility, equipment removal, copy removal, and unequip behavior in the maintenance domain, but output acquisition uses typed `GrantExecutor` effects. Dismantle validation/preflight uses the same grant execution rules as application.
- Resolved implementation rule: post-battle survival XP is an XP grant and uses `GrantExecutor` with explicit `SelectedEmployee` context. Combat trauma, injury, incapacitation, and roster-order effects remain combat resolution state changes, not grants.
- Resolved implementation rule: starter loadout and maintenance equipment-combination result item creation are equipment grants. The grant executor owns item creation/UUID assignment; starter selection and maintenance own the resulting loadout/equipped-to placement.

## Follow-Up Candidates

- None currently.
