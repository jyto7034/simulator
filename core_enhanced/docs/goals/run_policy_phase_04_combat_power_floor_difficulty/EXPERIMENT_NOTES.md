# Certified Combat Power And Floor Difficulty Experiment Notes

Status: complete notes

Implementation complete: true

Created: 2026-07-27

## 1. Source-of-truth decisions

- Policy coefficients and band/role/Floor schedules will be live strict data. Runtime hardcoded defaults are not
  a fallback.
- Facility assignment keeps category responsibility; difficulty reference extends that profile and node
  difficulty is fixed only after complete category/kind assignment.
- Node difficulty stores the complete generation result needed by combat/reward consumers. Consumers do not
  recompute it from current roster or Floor number.
- Unity projection exposes qualitative band/threat/readiness only; internal certified/target power remains Core.

## 2. Certified profile seam

- Reuse the same employee/fragment/equipment/enhancement source data as battle entry, but expose an explicit
  persistent-only calculation before active consumable and Trauma-derived current HP.
- Include equipped fragment effects even when they primarily select a skill; the provisional model may use a
  live fixed skill contribution or typed effect contribution. Do not inspect arbitrary skill behavior strings.
- Use checked/saturating integer arithmetic only where the live contract defines caps. Silent wrap or float-based
  platform drift is not acceptable.

## 3. Persistence seam

- Current `RunCheckpointState.loads_used` demonstrates a non-payload monotonic field, but run peak also needs
  active-run current/backup metadata merge so restart and backup recovery cannot lower it.
- Floor anchor/current node difficulty remain checkpoint-owned because loading a prior valid SavePoint must restore
  that Floor's exact generated state. Import merges non-rewind peak separately.
- Intermediate phase build compatibility is not approved; new required fields should fail old payloads rather than
  use `serde(default)`.

## 4. Old scaling migration

- Current `floor_combat_scaling` applies one multiplier to all nodes on a Floor and clamps beyond Floor 8. It is a
  competing source and must be removed after every preview/runtime consumer uses stored node difficulty.
- Manual waves currently ignore generated-wave budget scaling. P04 must explicitly define whether stat scaling
  alone carries manual wave target variance and test that the stored contract is honored; it must not silently
  pretend manual counts were budget-generated.

## 5. Out-of-scope follow-up

- Final formula calibration against representative headless battles and live playtests remains post-provisional
  balance work.
- P05 supplies monolith/remanifestation/hidden-boss role inputs; P04 only authors and validates generic role
  profiles for those consumers.

## 6. Final technical decisions

- Level is an explicit required `level_flat` certified-power weight. The existing employee draft tier did not
  modify employee stats, so relying on it would have violated the approved level-sensitivity requirement.
- Temporary readiness is deliberately separated from certification: living Breakdown/Critical employees remain
  in raw persistent power while `NoDeployableEmployees`, `TraumaBreakdown` and `CriticalPhysicalDamage` describe
  present usability. Permanent death removes the employee from current certification; the historical run peak
  still cannot decrease.
- Only quantity-bearing reward effects are percentage-scaled. Duplicating a singular equipment/pool roll/skill
  fragment would introduce an unapproved drop-identity and rarity rule. Hard/role budget strength is enforced in
  live policy; later rarity-weight authoring can be added as an explicit typed contract rather than inferred.
- Event-authored combat is not an ordinary Combat/Boss manifestation and retains an explicit Standard
  non-manifested stat/reward contract. Every manifested Combat/Boss path now rejects missing authority before
  preview or execution; validation fixtures use the same explicit non-manifested Standard seam.
- Exact node authority remains checkpoint-owned with `RunMap`; peak/anchor/model remain outside the rewindable
  payload in the signed host envelope. Same-version backup merge takes maxima and version mismatch cannot continue.

## 7. Final legacy audit

- Production has no `floor_combat_scaling`, `FloorCombatScaling`, last-stage clamp or Floor-number stat/wave
  consumer. The removed name remains only in negative compatibility-rejection tests.
- Public map DTO and Unity models contain no `target_power`, `floor_power_anchor`, `power_model_version`,
  `difficulty_profile_id` or multiplier field.
- P05 may consume `MonolithRemanifestation`; it must not add a second power formula, ordinary quota path or Unity
  raw-power calculation.
