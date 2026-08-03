# Gameplay RON Nested Strictness Correction Notes

Status: complete

## Starting Evidence

- The reaudit production-shaped probe accepted unknown fields in the named nested types through four
  actual outer catalogs.
- The current worktree is broadly dirty, including the relevant serde modules and data canonical.
- Existing content must be treated as user-owned current state; patches will be minimal attributes
  and focused tests unless evidence requires more.

## Invariants

- Strictness is transitive, not merely an outer-wrapper property.
- Unknown fields fail at parse time with no compatibility alias.
- A green live load proves current data compatibility, not negative strictness; both are required.

## Resume Point

- Complete. `MovementDef`, `ResonanceDef`, `TileRangePattern`, `DeliveryDef`, `SkillEffectDef`,
  `StatModifier`, `MapNodePayload` and the wrapper DTO now reject unknown fields.

## Final Judgment

- Production map template loading deserializes a strict `Vec<BattlefieldTemplateDefinition>` and
  constructs `BattlefieldTemplateDatabase`; the wrapper itself is not the live file root. A direct
  wrapper regression was still added so a future direct reader cannot silently widen it.
- Five source-set tests exercise the seven demonstrated nested paths through the actual embedded
  outer catalogs and preserve source/domain error context.
- No live RON migration was required.
