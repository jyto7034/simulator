# Phase 35 CRA-P10-C07 Experiment Notes

Status: complete
Implementation Complete: true

## N01 — Cleanup Boundary

- Cargo target metadata and reachable Rust modules, not a filename under `tests/`, define the test graph.
- Deletion authority requires current proof that the exact file is whitespace-only, disconnected and the only
  directory member. Historical audit evidence is not enough by itself.
- No replacement test is appropriate for an artifact with no invariant, but exact before/after metadata and test
  listing are mandatory.

## N02 — Adjacent Package Boundary

- CRA-P10-C06 owns empty main, generator harness, dependency, feature, logger/ctor and package-target cleanup and
  remains Phase 44. Do not mix any of it into this file-only Phase.
- Global CRA-POL-006 is a preservation rule here, not an external inventory gate for a proven private unreachable
  empty path.
- If any current module/build/public consumer exists, stop; do not retain a compatibility stub or infer safety from
  the old audit.

## N03 — Validation Boundary

- The decisive proof is identical normalized Cargo package/target metadata and complete Core test names/count before
  and after removal, followed by a nonzero green Core runner.
- Zero-test main/generator harnesses remain current P10-C06 facts and are excluded from behavior counts, not cleaned
  up here.
- No runtime/data/wire/Unity validation is useful unless current metadata unexpectedly connects the file.

## N04 — Dirty Worktree Precautions

- Initialization baseline is 543 expanded rows at canonical C-sort SHA-256
  `04f967cbbf1377c1d156530ffe7776d5e2b31d4f828b2efcd3f49cc0d6c07c9b`.
- The target exists and is clean; all other existing rows are user/completed-campaign work.
- These three Child files are the only expected new paths before activation. Do not reset, normalize or broadly
  format adjacent test or package files.
- The protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`.

## N05 — Initialization Boundary

- Phase 35 is initialized only. Master still marks it `ready`; no implementation Child is active.
- No target content, module edge, metadata or test list has been inspected. Sole activation, active-document reread
  and prospective fingerprints are the next permitted transition.
- Phase 36 remains pending and absent.

## N06 — Activation Boundary

- Phase 35 is now the sole active Child; Phase 36 and all later work remain pending.
- The 546-row post-initialization status and `f9f0b113...` canonical C-sort hash are the activation document
  baseline.
- Active-document/predecessor reread and prospective target/manifest fingerprints must close before target content,
  reachability, metadata or tests are inspected.

## N07 — Prospective Dirty Collision Map

- The exact target and all Cargo manifest/lock inputs are clean, so its deletion must create one explicit deleted
  status row while the three Child paths remain the only initialization additions.
- Five integration/support test paths are already modified, but Phase 35 does not need to edit any of them. Their
  current content is protected by the 18-path aggregate fingerprint and individual root hashes.
- `src/main.rs` and all five fixture bins are clean and belong to Phase 44. Preserve their hashes and zero-test
  behavior; do not opportunistically clean them here.
- Content/reachability/metadata investigation is now permitted. Capture reproducible before artifacts before the
  deletion.

## N08 — Current Deletion Authority

- The target is not semantically empty by assumption: direct bytes prove it is only CRLF, and it is the sole member
  of its directory.
- Cargo has 12 Core targets and 1,077 actual test names; neither table contains the target. Exact structured target
  and sorted-name tables are retained for after-removal equality, not just counts.
- The pre-removal debug runner passes all 1,077 tests. Removing this disconnected path requires no assertion
  transfer or replacement test.
- Apply only the one-file deletion. The existing 0-test binary/generator harnesses remain deliberately untouched for
  Phase 44.

## N09 — Exact Equality Result

- Only the clean two-byte candidate was removed. The directory was removed only after filename discovery proved it
  empty; no adjacent path was deleted.
- Raw Cargo metadata is byte-identical, the normalized 12-target table is exact, and the sorted 1,077-name test table
  is exact before/after. Counts are supporting evidence; retained tables are the decisive evidence.
- The new 547th status row is exactly the tracked deletion. Every prospective hash outside the target and the
  protected marker remains unchanged.

## N10 — Validation Result And Audit Gate

- Debug and release Core all-target runners both pass 1,077/0/0. Workspace debug/release checks, format, scoped diff
  check and normal Core Clippy also pass at the inherited 82/134 warning baseline.
- Zero-test main/generator targets remain classified evidence for Phase 44, not a failure and not Phase-35 scope.
- Implementation evidence is complete but status stays active until a separate completion audit reruns decisive
  evidence and returns `Complete / High` with no material finding.

## N11 — Audit Restart Boundary

- The initial audit readback found only an evidence-row range typo in E06: C12 was written where the PLAN ends at
  C10. The reference is corrected without changing implementation or evidence.
- The completion audit restarts after the correction. No verdict from the pre-correction readback is retained.

## N12 — Completion Boundary

- The restarted audit is `Complete / High`; all ten completion conditions pass with exact graph/list evidence and a
  fresh nonzero runner.
- Phase 35 is complete. Phase 36 is next in linear order but remains uninitialized; Master readiness is not
  implementation activation.
