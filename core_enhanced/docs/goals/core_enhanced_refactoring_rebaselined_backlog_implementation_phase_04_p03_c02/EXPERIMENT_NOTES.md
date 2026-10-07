# Phase 04 CRA-P03-C02 Experiment Notes

Status: complete
Implementation Complete: true

## Activation Boundary

- Sole active candidate: CRA-P03-C02, Master Phase 04 of 44.
- Direct predecessor Phase 03 is `Complete / High`; Phase 05 remains pending.
- Mutation authority is limited to enhancement material aggregation/atomicity and its tests/canonical seam.
- All three Child documents must be read fully before Phase 04 production/status inspection.

## Authority Notes

- Authored recipe rows define cost input; run-local material inventory and owned equipment are authoritative state.
- Validation and execution must use one checked aggregate meaning. A row-wise preflight plus incremental live consume
  is not an atomic authority.
- Boss setup and other equipment consumers observe the committed enhancement only and own no cost mutation.
- Current code/data/tests override historical wording if discovery has drifted.

## Compatibility And Policy Notes

- Exact-aggregate duplicate rows must succeed, so wholesale duplicate rejection is outside this approved target.
- CRA-POL-006 keeps RON/schema, checkpoint, wire and public API shape unchanged.
- No recipe balance/content adjustment or generic economy redesign is authorized.

## Dirty Worktree Risk

Phase 02 changed inventory validation and the shared world fixture; other equipment/data/test changes are user-owned.
Fingerprint current enhancement source, tests, recipe/live data and canonical before editing. Do not reset, restore,
clean, stash or broadly format; stop if overlap cannot be separated.

## Testing Notes

- Compare the complete relevant material/equipment state, not only the returned error.
- An insufficient duplicate total must not produce a success diff or consume the first row.
- Exact total must consume the aggregate and raise the level exactly once.
- Exercise checked sum overflow if the serialized amount type permits it without an invalid fixture.
- Retain current single-row/live RON tests with explicit nonzero counts.
- Maintenance preview is part of admission authority: aggregate cost display, `can_execute`, common action capability
  and target handler must not disagree.
- Post-implementation focused proof passes: duplicate runtime 2/2 and aggregate-overflow publication 1/1.

## Exit Notes

- Final broad evidence: equipment world 30/30, equipment data 7/7, live RON 25/25 and full Core 986/986; Core/server
  all-target checks and static checks pass.
- Completion audit verdict: `Complete / High`; improvement class `none`; no policy question or immediate correction.
- Phase 05 may become `ready` but owns no production mutation until its required documents exist and are read fully.

## Out-Of-Scope Follow-Up Handling

Record two-ledger ownership, generic grant aggregation, balance or consumer findings under their assigned candidates;
do not absorb them into this correction.
