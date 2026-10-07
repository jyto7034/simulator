# Core Enhanced Refactoring Audit Phase 05 — Experiment Notes

Status: complete

## Boundary decision

- Primary authority: manifested node/encounter/difficulty → preview → setup plan → initial active battle handoff.
- Phase 1 owns publication; Phase 2 owns run/map/difficulty; Phase 3 owns persistent roster/economy; Phase 4 owns
  Event session lifecycle.
- Phase 6 owns active battle lifecycle after commit, Phase 7 owns live tactical/spatial mutation, Phase 10 owns DTO.

## Dirty precautions

- Current dirty tree is the baseline; production files may already be modified by the user.
- Baseline hashes/status are captured before detailed source reads.
- Only Master/Phase Goal artifacts are audit-owned.

## CRA-14 policy

- Test families are read for their protected production invariant and assertion meaning.
- Fixture-only generation is not production evidence by itself.
- `remove` requires no unique contract and an equal/stronger surviving safety net.
- Runner green counts do not close static coverage gaps.

## Questions to resolve

- Whether preview cache and setup regenerate from the same manifested authority or can drift.
- Whether wave, mission, battlefield, player/enemy and reward assembly duplicate resolvers/mappings.
- Whether invalid setup state can partially commit run cache, UUID, node/session or active battle state.
- Which preview/setup tests protect public behavior versus private construction shape.

## Handoff condition

Phase 6 stays inactive until Phase 5 has complete boundary/candidate/test artifacts, no-mutation proof and an
independent `Complete / High` audit.

## Resolved findings

- `CombatPreview.enemy_stat_scale` is intentionally lossy across serde, but durable checkpoints serialize and
  restore preview caches and the resolvers trust cache hits before difficulty regeneration. Preview-cancel followed
  by a sibling SavePoint makes the defect production-reachable; C01 owns the correction.
- Publication validation stops at preview topology/threat checks. Runtime-only scenario capacity, tactical object
  and reference validation form a distinct C02 gap, not a duplicate of Phase 1 C02's difficulty-stage matrix.
- `floor_index` is a ghost input. Manifested difficulty is the sole tuning authority; cleanup belongs to C03.
- Preview conversion/world resolution and setup assembly/repeated encounter lookup are C03/C04 consolidation
  targets. No additional valid-data partial start defect was proven.
- Armed certified membership, manual-wave invariance, preview-fixed waves, FacilityEntity rejection and code-owned
  current mission policy match current canonical.

## Final handoff

- Five candidates: two correctness corrections, two behavior-preserving refactors and one cleanup.
- C01 precedes C03; Phase 1 C02's stage/seed matrix precedes Phase 5 C02, which precedes C04.
- Phase 6 may activate after the completion audit records `Complete / High`.
