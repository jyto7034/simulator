# P02 Checkpoint-Owned Floor Anchor Completion Review

Review date: 2026-07-28

Verdict: **Complete / High**

Implementation complete: true

Review basis: Child 완료 표시에 의존하지 않고 current checkpoint capture/validation/restore, Gate generation,
Core export/continue, server envelope/current-backup filesystem lifecycle, Unity consumer search, canonical과 broad
runner를 다시 대조했다.

## Audit Table

| Item | Source of truth | Verdict | Fit |
| --- | --- | --- | --- |
| Checkpoint model/anchor ownership | `RunCheckpointPayload`, node flow regressions | Complete | High |
| Deterministic cross-Floor replay | Gate replay regression | Complete | High |
| Core payload schema/rejection | `active_run.rs`, round-trip regression | Complete | High |
| Host envelope/backup merge | `active_run_store.rs`, real-filesystem tests | Complete | High |
| Unity/canonical impact | consumer search와 canonical | Complete | High |

## 1. Checkpoint Generation Ownership

Expected behavior:

- Checkpoint map/node difficulty, model version과 current Floor anchor는 한 generation으로 저장·복원된다.
- Run peak와 load count만 checkpoint rewind 밖 high-water다.

Direct evidence:

- Strict `RunCheckpointPayload`가 model version과 anchor를 직접 소유한다.
- Save captures both; load/continue restores both while retaining the current run peak and incrementing only a
  successful explicit load.
- Validation rejects stale model, zero anchor, node model/anchor mismatch와 missing live tuning profile.
- Active-run transaction rollback restores anchor/model from its snapshot and max-merges only same-model peak.

Verdict: Complete / High.

## 2. Cross-Floor Determinism

- Pre-fix regression reproduced Floor 1 map/anchor 100 loading with retained Floor 2 anchor 125.
- Post-fix load restores anchor 100 and its exact map.
- Replaying the same Gate from the same checkpoint produces anchor 125 and the same generated map, so load does
  not compound minimum rise.

Verdict: Complete / High.

## 3. Core And Host Save Schemas

- Core payload version 2 contains an explicit version and the checkpoint-owned model/anchor; export fields contain
  only payload bytes, loads and run peak.
- Host envelope version 3 monotonic metadata contains only loads and run peak.
- Backup publication preserves the previous opaque Core payload/anchor and raises only the two host high-water
  values.
- Old/missing Core version, stale model, mismatched anchor, envelope version 2 and removed monotonic anchor are
  explicitly rejected. No migration, alias, dual reader or fallback exists.

Verdict: Complete / High.

## 4. Filesystem, Failure And Unity Boundary

- Real-file tests cover publish/current/backup recovery, corruption, checksum, unsupported version, invalidation,
  single-writer lock and publish failure atomicity.
- Actor publish failure continues to restore the captured Core transaction without losing P00 profile union; the
  corrected rollback restores anchor generation while preserving peak.
- Unity script search finds no consumer for internal model/anchor/peak fields. Public checkpoint snapshot and title
  continue UX still consume only existing `loads_used`/capability fields; no wire DTO changed.
- Unity MCP execution was not applicable because neither a C# asset nor serialized Unity contract changed.

Verdict: Complete / High.

## Validation

- Pre-fix cross-Floor regression: 0 passed, 1 failed; actual anchor 125 vs expected 100.
- Post-fix focused: cross-Floor 1/1, Core round-trip/schema 1/1, server active-run 13/13, exact old-envelope 1/1.
- Core broad: 790 passed, 0 failed.
- Server broad: 54 passed, 0 failed.
- Workspace all targets: Core 790, integration groups 3/4/23/13/4, server 54; failures 0.
- Workspace check, Rust format, scoped diff/legacy search: pass.

## Final Verdict

- Primary verdict: **Complete**
- Long-term fit: **High**
- Improvement class: **none**
- Remaining risk: none material to P02. P03 must use this strict checkpoint generation when adding typed monolith
  stage difficulty and reward projection changes.
