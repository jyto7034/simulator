# Phase 03 CRA-P02-C01 Experiment Notes

Status: complete
Implementation Complete: true

## Activation Boundary

- Sole active candidate: CRA-P02-C01, Master Phase 03 of 44.
- Direct predecessor Phase 02 is `Complete / High`; Phase 04 remains pending.
- Mutation authority is limited to local active-run host monotonic publication and its proof/canonical seams.
- The three Child documents must be read completely before Phase 03 production/status inspection.

## Authority Notes

- Core owns opaque payload meaning. The host store owns envelope/checksum/generation and monotonic host metadata.
- The correction must merge host metadata into the incoming envelope, never replace incoming Core bytes with a
  previous generation's payload.
- Backup recovery is an existing approved path, not permission to redesign normal publish or add fallback.
- Current code and tests override historical wording if discovery shows drift.

## Compatibility And Policy Notes

- CRA-POL-006 remains global: envelope/Core versions, serialized fields, wire/public API and public errors are
  preserved unless separately migrated.
- No policy authorizes cross-actor lease, run identity conflict, compare-and-swap or multi-writer semantics.
- Corrupt or unsupported current files must not become silent metadata sources without existing reader authority.

## Dirty Worktree Risk

The campaign tree is heavily dirty and Phase 02 verified overlapping server/canonical changes. Capture exact current
store/test/canonical status, diff and hashes before editing. Do not reset, restore, clean, stash or broadly format;
stop if the intended store correction cannot be separated from user work.

## Testing Notes

- Use real temporary directories/files and the actual store encoder/reader, not an in-memory model.
- Give current and incoming distinct Core bytes so a metadata fix cannot accidentally select the old payload.
- Test loads-only, peak-only and both; component-wise maxima are independent.
- Preserve and test existing backup/reopen and failure paths after the focused first proof.
- Record physical test counts; zero-test bins and filtered nonmatches are not evidence.

## Out-Of-Scope Follow-Up Handling

Record independent multi-writer, migration, mapping or durability findings with an owner. Do not absorb them into
this monotonic-value correction.

## Completion Boundary

- Phase 03 exited `Complete / High` with improvement class `none`; all eight completion rows pass.
- The durable seam is one store-owned per-publish component-wise merge that keeps incoming Core bytes in current
  and previous Core bytes in backup with equal host maxima.
- Phase 04 is only ready and requires separate Child initialization/read before any production discovery.
