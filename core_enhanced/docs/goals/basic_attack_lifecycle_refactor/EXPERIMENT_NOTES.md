# Basic Attack Lifecycle Refactor Experiment Notes

## 2026-06-12

- The worktree is already dirty with unrelated and prior-goal changes. Do not revert them.
- Source of truth order: runtime battle core code, tests/timeline outputs, live weapon/basic attack data, then design notes.
- Do not change targeting, damage timing, projectile semantics, or Unity-facing timeline field names in this subgoal.
- Runtime inspection found the current basic attack flow:
  1. `try_start_pending_basic_attacks` scans pending auto attacks in deterministic unit id order.
  2. `BattleEvent::AttackStart` validates readiness, handles action locks, selects target, interrupts movement, records `AttackStart`, schedules `AttackResolve`, and schedules the next auto attack.
  3. `BattleEvent::AttackResolve` records `AttackResolve`, delegates to `resolve_basic_attack`, and records `AttackMiss` if resolution fails.
  4. `resolve_basic_attack` in `commands.rs` owns instant/projectile delivery, damage, resonance, and trigger behavior.
  5. Projectile launch/advance/impact helpers remain in `commands.rs`.
- First refactor step moved target selection, pending auto-attack scan, and AttackStart/AttackResolve event lifecycle into `battle/core/basic_attack.rs`.
