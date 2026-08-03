# P00 Compendium Transaction Rollback Integrity Completion Review

Review date: 2026-07-28

Verdict: **Complete / High**

Implementation complete: true

Review basis: Child 완료 표시에 의존하지 않고 현재 Core transaction restore, active-run rollback,
game_server AttachSession/persistence lifecycle, serialized snapshot, canonical과 현재 broad runner를 직접
재검토했다.

## Audit Table

| Item | Source of truth | Verdict | Fit |
| --- | --- | --- | --- |
| Common transaction profile lifetime | `world/node_flow.rs`, Core regression | Complete | High |
| Active-run publish rollback union | `world/active_run.rs`, actor rollback path/tests | Complete | High |
| AttachSession-before-snapshot sync | player actor handler, real profile store + snapshot test | Complete | High |
| Profile failure/dirty/reset UX | profile persistence state and server broad tests | Complete | High |
| Canonical/broad convergence | runtime/server canonical, workspace all-targets | Complete | High |

## 1. Common Transaction Rollback

Expected behavior:

- A failed run-local/non-battle transaction rolls back its run state but cannot delete profile knowledge that
  existed before the operation or committed while it was in flight.

Direct evidence:

- `GameCoreState::clone_non_battle_transaction_state()` still excludes the profile from rollback ownership.
- `GameCore::with_state_transaction()` clones the live profile only on failure, restores the run-local snapshot,
  then applies the Core domain idempotent union.
- `failed_common_transaction_preserves_profile_knowledge_from_before_and_during_operation` failed with an empty
  set before the correction and passes with both expected IDs afterward.

Legacy/fallback audit:

- No profile copy was added to checkpoint/active-run payloads and no replacement load or compatibility branch was
  introduced.

Verdict: Complete / High.

## 2. Active-Run And Host Transaction Separation

Expected behavior:

- Active-run publish failure restores run state, keeps the newest compendium union, and attempts independent
  profile publication.

Direct evidence:

- Existing `restore_active_run_transaction_rollback()` uses the same restore-then-union lifetime.
- Player actor persistence failure path restores active-run state and separately calls full-union profile publish.
- Core active-run rollback regression and server
  `active_run_write_failure_does_not_rollback_or_skip_compendium_publish` both pass.
- Server broad 53/53 includes compendium write failure, dirty retry, current/backup, checksum, lock and reset
  regressions.

Verdict: Complete / High.

## 3. AttachSession Projection And Recovery UX

Expected behavior:

- Every readable durable singleton profile is merged before a reconnect root snapshot. Fatal persistence still
  exposes the blocking title Retry/Reset UX and is not replaced by an empty durable profile.

Direct evidence:

- `AttachSession` now performs readable-profile sync before `build_state_snapshot()`.
- `attach_session_merges_durable_compendium_before_building_snapshot` opens a real profile file, starts a Core with
  an empty profile and proves the first attach snapshot projects the stored discovery.
- `fatal_compendium_bootstrap_gates_title_to_retry_and_confirmed_reset` passes after the ordering change.
- Snapshot field names and profile persistence DTO are unchanged; no Unity reader/presentation change was needed.

Verdict: Complete / High.

## Cross-Component Findings

- Common transaction, active-run rollback and AttachSession now share one monotonic set-union meaning.
- File lifecycle remains host-owned and independent from Core checkpoint/run rollback.
- Core/server canonical now state ordinary rollback and attach ordering explicitly.
- No material schema, DTO, Unity consumer, compatibility or profile-identity finding remains.

## Validation

- Focused new regressions: Core 1/1, server 1/1.
- Focused existing boundaries: active-run Core 1/1, active-run publish server 1/1, fatal bootstrap server 1/1,
  Core compendium 5/5, server compendium 5/5.
- `cargo fmt --all -- --check`: pass after formatting only the reported P00 lines.
- `cargo check -p game_core`, `cargo check -p game_server`: pass.
- Core broad: 788 passed, 0 failed.
- Server broad: 53 passed, 0 failed.
- Workspace all targets: Core 788, integration groups 3/4/23/13/4, server 53; failures 0.
- Scoped `git diff --check` and Goal trailing-whitespace audit: pass.

## Final Verdict

- Primary verdict: **Complete**
- Long-term fit: **High**
- Improvement class: **none**
- Remaining risk: none material to P00. Later correction phases must preserve this profile lifetime when changing
  certified power, checkpoint schema and monolith transaction consumers.
