# P04 Battle Kernel Completion Audit

Verdict: `Complete / High`

## Audit Identity and Separation

- Handoff / Goal Set: `HA-CRAR-P2G60-002` / `GS-CRAR-P2G50-002`.
- Audited member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P04-BATTLE-KERNEL)`; Phase / node
  `PHASE-CRAR-C2-04-BATTLE-KERNEL` / `WP-CRAR-C2-05`; Goal Quality Contract `2`.
- Frozen GEX-10 target: `ATR-CRAR-C2-P04-GEX10-001` / `evidence/P04_AUDIT_TARGET.json`, SHA-256
  `0ef1bd934ee8f2a3efac48bad9c327bec914a92fcdaa09b6659abbdfc73a23bc`.
- GEX-20 began only after target publication. Its validation was read-only except for generated `target/` outputs and
  this explicitly excluded completion audit. No frozen member was changed.

## Target and Predecessor Currentness

- Repository revision/branch remains `f318414fb0156a67e04c5479000c2a2683fc0c09` /
  `cleanup/simulator-dirty-worktree-20260803`.
- Target and receipt JSON parse; declared content count is exact; all 31/31 members match their frozen SHA-256 before
  and after fresh validation.
- Core-only unrelated dirty baseline remains exact: 123 modified, 23 deleted, 724 untracked, total 870; fingerprint
  `e37c73f79689925f2d57809057b2b9942ab415f909158022fee08e76c6103942`. Generated target directories are excluded.
- P01 `ATR-CRAR-C2-P01-GEX20-004`, P02 `ATR-CRAR-C2-P02-GEX20-004` and P03
  `ATR-CRAR-C2-P03-GEX20-002` are all `Complete / High` and current. P04 strengthens P02's release boundary and changes
  only the expected P03 canonical-contract member; P03 executor/lifecycle evidence remains byte-identical.

## Completion Condition Audit

| Condition | Direct evidence | Result |
| --- | --- | --- |
| private production kernel | default release declares `pub(crate) mod core`; standalone imports fail with `E0603` | pass |
| sole production producer | actual route remains `GameCore` application draft -> private active battle; no second facade/writer added | pass |
| raw unit/kernel mutation bypass 0 | default-release callers cannot name `BattleCore`, `RuntimeUnit`, `UnitBody` or their fields | pass |
| raw event injection bypass 0 | `enqueue_event` is test/debug support and the release module cannot be named | pass |
| finite position | coordinates are crate-private; fallible construction and custom deserialization reject non-finite components | pass |
| finite radius/speed/time | typed checked values protect scenario preparation and millisecond conversion | pass |
| reject before mutation | validation precedes spawn order, placement and unit mutation; complete authority snapshot remains equal | pass |
| retry idempotency | all 15 NaN/±Inf cases produce equal first/retry errors and equal full snapshots | pass |
| deterministic finite parity | heap permutation, boss priority, movement insertion order, skill and live-data oracles pass | pass |
| policy compliance | no clamp/default/normalization/non-finite total order, duplicate state or production fixture branch | pass |
| canonical contracts | runtime and skill-target contracts describe the implemented private finite boundary | pass |

`BattleEvent` may remain a public vocabulary type without being an authority bypass: default release has no public
kernel queue port and cannot construct its kernel-private world-position payload unchecked. Internal `pub` members are
likewise not production-public when their enclosing module is crate-private.

## Validation Dimensions

| Dimension | Audited evidence / N/A basis | Result |
| --- | --- | --- |
| `success` | live embedded publication -> `GameCore` -> combat probe; full Core 1,138/1,138 | pass |
| `invalid_input` | NaN and both infinities for x/y/radius/speed/time; negative physical scalars also reject | pass |
| `failure_rollback` | units, heap/order, serialized log, battlefield, counters, seed and recording stacks unchanged | pass |
| `retry_idempotency` | identical typed first/retry error and complete snapshot for every matrix case | pass |
| `ordering_concurrency` | same-bucket permutation and insertion-order oracles; no shared concurrent writer introduced | pass |
| `boundary_empty_max` | signed zero/finite round-trip and finite `u64` millisecond conversion; existing empty/max corpus green | pass |
| `disconnect_restart_persistence` | N/A: no checkpoint, reconnect, save, history or result-lifetime change; P05/P06 own those | pass |
| `live_data` | actual embedded validated data constructs the private battle through Core ownership | pass |
| `producer_consumer` | positive live release probe plus negative standalone compile-fail probe | pass |
| `legacy_fallback_absence` | no public release kernel/event fallback, normalization or competing mutation facade | pass |

## Fresh GEX-20 Validation

- `cargo fmt --all -- --check`: exit 0.
- Target-scoped `git diff --check`: exit 0.
- `cargo check -p game_core --release`: exit 0, warnings 0.
- Valid live default-release probe: exit 0.
- Invalid kernel default-release probe: expected exit 101 with two `E0603` private-module failures.
- Finite rejection, finite deserialization, heap permutation, boss priority and movement order: 5/5 focused tests pass.
- `cargo test -p game_core`: library 1,087 plus integrations 51 = 1,138 passed; failed/ignored 0.
- Post-validation target comparison: 31/31 exact; unrelated dirty baseline exact.

## Quality, Scope and Limitations

- P04 preserved battle algorithms and current-machine deterministic behavior. It does not claim bitwise cross-platform
  replay or external physics compatibility.
- Test/debug/test-fixtures visibility is an explicit oracle capability, not default-release production authority.
- Whole-library Clippy remains blocked by the pre-existing denied `never_loop` in
  `src/game/data/boss_runtime_data.rs`; release checking is warning-free and P04 adds no lint suppression.
- P05 owns history/result lifetime, P06 checkpoint format/version, P07 transport-neutral projection and P08 measured
  optimization/performance PASS. P04 makes no completion claim for them.
- Dirty user work outside exact P04 members is preserved. `game_server`, Unity, sibling resources and external
  consumers were neither inspected nor used as evidence.
- Completion-blocking omission, unsafe overlap, policy/Goal/Phase change, external dependency or automatic stop
  condition: none.

## Verdict and Next Boundary

`GOAL-CRAR-C2-P04-BATTLE-KERNEL` is `Complete / High` on `ATR-CRAR-C2-P04-GEX10-001`, currentness
`current / unchanged`.

Strict sequence makes `GOAL-CRAR-C2-P05-HISTORY-RESULT-LIFETIME` eligible for its own GEX-00. This verdict does not
complete or pre-authorize P05-P08, and P04 requires successor-currentness review if a later phase changes its private
kernel, finite inputs, ordering tests, contracts or frozen evidence surface.
