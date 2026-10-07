# Phase 16 CRA-P09-C06 Experiment Notes

Status: complete
Implementation Complete: true

## Activation Boundary

- Sole active candidate: CRA-P09-C06, Master Phase 16 of 44.
- Direct predecessor Phase 15 and supporting Phase 14 are `Complete / High`.
- Phases 17–44 remain pending. Mutation authority is limited to typed tile reach query/projection consolidation,
  characterization/integration tests and nearest canonical evidence.
- All three Child documents and the Phase 15 audit must be fully read before production/status/source inspection.

## Policy Boundary

This is a behavior-preserving refactor with no candidate-specific gameplay decision. CRA-POL-006 requires exact
public/RON/save/wire compatibility. If current callers disagree, stop rather than selecting new semantics.

## Authority Notes

- `TileRangePolicy`/`TileRangePattern` remain data geometry authority.
- The new authority may centralize query mechanics but must not merge cast reach with delivery footprint.
- Membership and materialized-cell projection remain distinct operations of one typed query.
- Valid-board clipping is included; obstacle, occupancy, line-of-sight, physics and Boss path cost are excluded.
- Phase 17 owns actual delivery-target usefulness meaning. Phase 18 owns preview footprint/union meaning.

## Dirty Worktree Risk

The workspace contains extensive user/campaign changes in targeting, skill runtime, preview, tests and canonical
paths. Fingerprint every prospective target before editing. Do not reset, restore, clean, stash, delete or broadly
format; stop if a Phase 16 hunk cannot be separated.

## Testing Notes

- Characterize before extraction. The cross-product must run nonzero cases through current behavior first.
- Cover policy/pattern, anchor inclusion, all facings, center/edge/void clipping, ordering and membership/materialized
  set equivalence.
- Keep production selector, cast, preview and area tests; a private helper-only table is insufficient.
- Preserve intentional absence of obstacle/occupancy filtering.
- Public output/snapshot values must remain exact; no Phase 17/18 expected-value correction is allowed here.

## Out-Of-Scope Follow-Up Handling

Record any current disagreement as a policy/scope blocker. Do not fix usefulness, effect preview, projectile motion,
Boss navigation or ghost range surfaces inside this Phase.

## Current Implementation Result

- `TileReachQuery` is crate-private and lives with tile-range geometry. It captures battlefield context plus either a
  validated pattern/anchor/facing or whole-field policy.
- Membership and stable materialization share valid-tile clipping. Static obstacles and occupancy remain visible to
  range queries exactly as before.
- Runtime caster projection is centralized by `BattleCore::tile_reach_query_for_unit`; selectors and explicit tile
  casts no longer reproduce geometry membership.
- Preview, action capability, area runtime and area usefulness materialize through the same authority.
- Cast reach and TileArea delivery footprints remain separate inputs. Phase 17 and Phase 18 semantics are untouched.
- The pre-existing usefulness adapter omitted clipping, but its target result was equivalent because gameplay units
  occupy valid battlefield tiles. The migrated path now shares runtime clipping without changing observed targets.
- Cross-product characterization passed before extraction and after migration; the proposed type also produced the
  intended compile-red before implementation.
