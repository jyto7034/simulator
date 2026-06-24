# Movement Tick Backend Refactor Experiment Notes

## 2026-06-12

- The worktree is already dirty with unrelated and prior-goal changes. Do not revert them.
- Source of truth order: runtime movement code, movement tests, core policy docs, then design notes.
- Current duplicate responsibilities:
  - unit canonicalization,
  - dead/locked/reached/no-goal stop handling,
  - target lookup,
  - steering displacement,
  - board clamping,
  - static obstacle blocked output,
  - BodyMoved output generation.
- Backend-specific responsibilities to preserve:
  - Direct: deterministic static obstacle sweep/depenetration.
  - Rapier: sync Rapier bodies/colliders, use kinematic controller for ground static obstacle correction, apply translated body positions.
- Direct removal rejected for this goal because it would remove deterministic movement semantics tests and make Rapier look like the gameplay source of truth.
