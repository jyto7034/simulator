# Phase 02 CRA-F-C01 Experiment Notes

Status: complete
Implementation Complete: true

## Activation Boundary

- Sole active candidate: CRA-F-C01, Master Phase 02 of 44.
- Direct predecessor Phase 01 is `Complete / High`; Phase 03 remains pending.
- Production mutation is limited to the current checkpoint invariant graph and its proof/canonical seams.
- Required Child documents were created before any Phase 02 production search, status inspection or test run.

## Evidence And Authority Notes

- CRA-F-C01 is the exact merged implementation identity for former CRA-P02-C02 and CRA-P03-C01.
- Core owns current V3 payload meaning, capture, validation and restore. The host store owns envelopes and monotonic
  metadata, not Core graph validity. The live tree must confirm exact paths and calls.
- `RunMap`/`MapProgression` own graph-derived selectability. Persistence should invoke or compose domain authority,
  not silently create a second progression rule.
- Current production code/data and accessible consumers override historical audit wording if facts have drifted.

## Compatibility And Policy Notes

- CRA-POL-006 is directly applicable: current V3 remains strict and shape-compatible; inconsistent documents fail
  before mutation; repair, rebinding and old-version migration are separate policy work.
- Active battle/Boss/tactical state remains intentionally outside V3.
- Unknown external save/Rust consumers prohibit field or public API removal, but do not block internal strict
  validation of invariants already guaranteed by the current writer.

## Dirty Worktree Risk

The campaign began on a heavily dirty tree and Phase 01 added test/document changes. All current Phase 02 target
changes must be fingerprinted and treated as user-owned before mutation. No reset, restore, clean, stash or unrelated
formatting is allowed. A semantically inseparable overlap is a stop condition.

## Testing Notes

- Generate negatives from a valid payload and change exactly one edge.
- Prove pre-mutation behavior with full relevant state equality, host load metadata and re-export, not only an error.
- Distinguish Continue and explicit load paths, but avoid duplicating validator rules in host callers.
- Runner counts must be nonzero; valid-path round trips alone are not evidence for negative admission safety.

## Out-Of-Scope Follow-Up Handling

Record independent defects with current code/data evidence and an owner. Do not absorb mapping refactors,
lease/conflict semantics, admin durability, repair/migration or Boss persistence into this Phase.

## Completion Boundary

- Phase 02 exited `Complete / High` with improvement class `none`; all nine completion-matrix rows pass.
- The durable seam is one strict current-V3 pre-mutation admission graph shared by Continue and explicit load with
  their intended seed contexts.
- Phase 03 is only ready. Its production discovery and mutation require separate Child initialization/read and the
  Master single-active-Phase transition.
