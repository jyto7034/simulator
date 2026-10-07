# Phase 13 CRA-P08-C04 Experiment Notes

Status: complete
Implementation Complete: true

## Activation Boundary

- Sole active candidate: CRA-P08-C04, Master Phase 13 of 44.
- Direct predecessor Phase 12 is `Complete / High`; Phase 14 remains pending.
- Mutation authority is limited to equipment publication reference/matrix validation, scoped tests and nearest
  canonical evidence.
- All three Child documents must be read fully before Phase 13 production/status/data inspection.

## Authority Notes

- Publication owns authoring validity across equipment, buff and skill catalogs. Runtime occurrence dispatch owns
  execution and may retain defensive lookup guards without becoming a second authoring policy.
- Phase 11's typed dispatcher and CRA-POL-002 define the executable matrix. This Phase validates that contract; it
  does not reinterpret targets, timing, retaliation or death behavior.
- Failure must occur at the existing whole-candidate/bundle atomic boundary with deterministic equipment/path/cause.

## CRA-POL-002 Notes

- OnHit CounterpartUnit means the attacker.
- OnHit Counterpart BonusDamage is owner-attributed retaliation delivered once per authored effect and does not
  re-enter OnAttack/OnHit collection.
- The matrix must distinguish this row from current-hit damage modifiers and reject combinations runtime cannot
  execute rather than silently dropping them.

## Compatibility Risk

Do not change Effect/TriggeredEffect/activation RON or public Rust shapes to make validation convenient. If current
official content conflicts with the implemented dispatcher or approved matrix, identify the exact row and stop for a
content/semantics decision. Do not normalize targets, rewrite IDs or add aliases/fallbacks.

## Dirty Worktree Risk

The workspace contains extensive user-owned Core, data loading, equipment, server, test and canonical changes from
the campaign and prior work. Fingerprint each prospective target before editing. Do not reset, restore, clean, stash
or broadly format; stop if the required hunk cannot be separated.

## Testing Notes

- Negative tests must reach the real publication authority, not only a helper, and assert exact equipment/path/cause.
- Cover missing buff and activation skill references plus invalid Permanent and occurrence rows.
- Build an explicit supported/rejected matrix from the current dispatcher, including approved retaliation.
- Prove installed/current published state is unchanged after a rejected candidate and the complete official bundle
  loads without content edits.
- Retain local weapon/profile/recipe/event metadata tests and production live occurrence integrations.
- Zero-test filters, parse-only success and catch-any-error assertions are not proof.

## Out-Of-Scope Follow-Up Handling

Keep terminal delivery under Phase 14, projectile/basic compatibility publication under Phase 15, committed-damage
consolidation under P08-C05 and public DeathHandler cleanup under P08-C06. Record independent findings without
absorbing them.

## Implemented Authority

- One Core-private classifier now owns executable equipment effect semantics for publication, Permanent application
  and occurrence dispatch. Publication does not maintain a second handwritten matrix.
- Official and programmatic builders both resolve every equipment ApplyBuff/activation edge before returning data.
- The installed official `Arc<GameDataBase>` remains unchanged after invalid source rejection; no partial candidate
  or domain publication path was introduced.
- Runtime missing-reference and invalid-row guards remain defensive only. They were neither removed nor treated as
  compatibility fallback.

## Validation Result

- Exhaustive effect matrix: 140 rows in one nonzero test; activation triggers: seven rows in the same test.
- Complete Core: 974/974. Server main: 59/59. Release matrix: 1/1. Release retaliation: 3/3.
- Official publication: 66/66. Full RON: 25/25. Production live-item behavior: 4/4.
- Check, formatter check and diff check pass; live equipment/Buff/Skill RON hashes are unchanged from activation.
- Implementation conditions P13-C01~C09 are complete. P13-C10 remains gated on the independent completion audit.
