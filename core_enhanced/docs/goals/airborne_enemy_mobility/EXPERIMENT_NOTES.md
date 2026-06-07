# Airborne Enemy Mobility Experiment Notes

## Findings

- `UnitTargetTrait` currently only has `Airborne`. Keep it as derived/display information for now; `mobility_kind` is the runtime source of truth.
- `AbnormalityDatabase::new` did not previously run its own runtime-contract validation. The airborne trait rejection is now enforced at database creation, not only when `GameDataBase` later calls `validate_indexes`.
- DefenseRoute generated/validated battlefields already require all defense routes to share one endpoint. In the current runtime that endpoint is the protected defense object location, so a separate "airborne route end must be in protected target range" data validation would duplicate the existing route endpoint invariant. If later multiple protected targets or off-route protected objectives are added, add explicit generated-instance validation there.
- Full live RON loading is currently blocked by weapon archetype/profile validation: live weapon equipment such as `justitia` lacks `weapon_profile`. Do not weaken that validation in the airborne goal; fix it in the Weapon Archetype and Targeting Profile goal.
- The canonical Unity implementation doc still pointed at the local `core_enhanced/docs/unity_core_contract.md`. It now points at `F:\unity projects\ark\docs\unity_core_contract.md` to avoid stale-doc drift.
