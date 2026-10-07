# P02 Validated Data Current Notes

## Current Summary

- Current pass/result: GEX-20 attempt 1 `Not Complete`; correction GEX-10 exit conditions met; replacement target next.
- Exact target: `GOAL-CRAR-C2-P02-VALIDATED-DATA` / `WP-CRAR-C2-04`.
- Direct predecessor: P01 `Complete / High`, current at P02 intake.
- Scope: Core data validation/publication and construction boundary only; game_server/Unity/external admin consumers are
  excluded.
- Prior validation is not automatically reused; current invalid/valid/public API seams must execute non-zero.

## Resolved GEX-10 Questions

- Official publication is opaque `ValidatedGameData`; public Core constructors consume it.
- Bare parts/new and battle-held database authority are private; builder/fixture extraction is absent from default
  release and independently compile-failed.
- Invalid authoring returns `LiveRonError` before any token/Core authority exists; repeated failure changes no later
  publication.
- Startup-only/no-hot-reload and unchanged session/save lifetime keep both N/A rows current.

No new policy meaning or structure decision is currently identified.

## Current Technical Judgment

- Validation semantics do not need redesign: the existing loader already validates the whole candidate and fails before
  publication.
- P02 owns the missing type/API boundary: create opaque `ValidatedGameData`, make it the production construction token,
  and remove release availability of parts/builder/new/direct database authority.
- Debug/test fixture availability is an explicit capability, not a production fallback. The release API audit must prove
  it is absent without the non-default feature.
- Existing typed Core admin commands remain the only operational mutation seam; no hot reload/data setter is introduced.

## Current Completion Summary

- Actual live data preserves every parsed catalog and derived registry, and actual loader→opaque publication→Core
  construction runs in a standalone default-release probe.
- Malformed/reference/range, empty/missing/max and late candidate failures remain recoverable and fail closed; focused
  publication rollback and retry tests are non-zero.
- Default release has no public builder/parts construction capability. Explicit fixture builds remain available only
  under their capability and all gated binaries compile.
- Canonical documentation matches the actual Core-only boundary.
- Current broad regression: 1,132 passed, failed/ignored 0.
- P01 changed evidence was rebound and freshly re-audited `Complete / High` on
  `ATR-CRAR-C2-P01-GEX20-002`; predecessor continuity is restored.
- Attempt 1's public bare `CombatExecutor` helper finding is corrected. Release now lacks the helper and cannot convert
  bare `Arc<GameDataBase>` into the validated token; the expanded probe reports exact `E0277`/`E0599` in addition to
  the original builder/parts failures.
- Completion-blocking omission, technical defect, material policy question, external dependency or dirty-isolation
  problem: none.
