# P03 Application Authority Experiments

## 2026-08-23 — GEX-00 Intake

- Run mode: full Goal Set `goal-run`.
- Selected member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P03-APPLICATION-AUTHORITY)`.
- Atomic execution node: `AG-CRAR-C2-01-APPLICATION-AUTHORITY`; `WP-CRAR-C2-02` and `WP-CRAR-C2-03` are not
  separable implementation or publication units.
- Existing exact-identity P03 execution documents: none; canonical directory
  `execution/p03_application_authority` selected.
- Production/source mutation and test execution during intake: none.

### Current Core Trace

- Read the actual public behavior router, `GameCoreState`, `GameState`, battle advance/playback, admin and active-run
  entrypoints plus the current source review for CA-003/004.
- Confirmed CA-003 commands: pause, resume, speed, begin targeting, cancel targeting, activate skill.
- Confirmed the mutation-after-fallible-projection ordering remains in playback/targeting handlers and partial mutation
  remains possible inside StartNewGame before a missing-trigger error.
- Found 23 syntactically public mutable world/state entries; after excluding value-type internals and queries, the root
  families are behavior, battle time, disconnect, admin, rollback/continue and profile. Direct test currency mutation is
  a fixture capability, not an operating route.
- Found 61 root/lifecycle assignment or transition sites. These are internal algorithm writes today; the target makes
  them draft-only and leaves one live-root commit port.
- Existing correct local transactions are reusable algorithm seams but cannot independently satisfy root ownership.

### Feasibility and N/A Recheck

- `ActiveBattleSession` and its battle kernel are cloneable; every `GameCoreState` field has a cloneable current
  representation. A complete staged authority draft is therefore feasible without rewriting gameplay algorithms.
- No durable external publish exists inside Core. All current output DTO assembly can finish on the draft before root
  replacement. A recovery-pending type can model any future diagnostic publication fault without returning `Err`.
- `ordering_concurrency` is applicable to competing mutation entries and will be covered by single-owner/commit-count
  and deterministic ordering tests. No multi-threaded shared `GameCore` owner is introduced.
- `disconnect_restart_persistence` is applicable through disconnect auto-pause and active-run continue/rollback; the
  checkpoint format remains unchanged and is not a P03 claim.
- Live data is inherited from P02's opaque validated publication and remains a prerequisite rather than a P03 rewrite.
- No external consumer, policy-meaning, node-mapping or dirty-isolation blocker was found.

### GEX-00 Result

- Result: `passed`.
- Approved next pass: separated GEX-10 implementing both atomic-group members together.

## 2026-08-23 — GEX-10 Implementation and Validation

### Application Authority Cutover

- Added a registry with exhaustive compiler-checked mapping for all 58 `ActionKind` values and ten production
  non-behavior root routes. Behavior owner is now the actual domain dispatch input; the old broad root variant match is
  removed.
- Added complete `GameCore` staging, lifecycle checks before and after draft execution, typed prepared commit token,
  one live-root replacement function and infallible accepted publication.
- Routed behavior, profile, admin, durable continue/rollback, elapsed/server/bucket battle advance and disconnect pause
  through the application root. The test currency setter is gated out of default release and uses the same root only in
  fixture builds.
- Removed the old battle-only swap/rollback wrapper because complete application staging now owns that boundary.

### Closed Lifecycle

- Replaced `GameCoreState.game_state` with private `LifecycleContext`. Every phase read is a snapshot/reference and every
  transition is internal to a staged command.
- Commit validation rejects incoherent NotStarted/pre-run/map/node/shop/reward/battle/result contexts and binds node,
  session, shop/reward and battle UUID identities.
- Two existing tests had constructed invalid internal fixtures by changing phase alone. They were corrected to use an
  actual AbandonRun route or install the matching node session; no gameplay expectation was weakened.

### Atomicity Matrix

- CA-003 playback test exercises pause, resume and speed with `u64::MAX` cursor after normal preconditions. The manual
  player-unit skill test exercises begin focus, valid activate and cancel focus the same way. Every rejection compares
  full run payload/checkpoint/profile/run-seed plus battle state/event queue/log/revision/deployment/resource/playback/
  focus/VN/UUID/lifecycle; identical retry error and identical second snapshot are required.
- CA-004 missing-trigger test compares complete nonbattle roots, profile and run seed before/after both first rejection
  and retry.
- Registry, accepted/recovery-pending and incoherent-context tests pass. Existing stale authority, revision overflow,
  late reward/event/reconciliation rollback and durable continuation tests remain green.

### Direct Validation

- `cargo fmt --all -- --check`: exit 0 after formatting.
- `cargo check -p game_core --release`: exit 0, warnings 0.
- `cargo test -p game_core --no-run`: exit 0.
- Focused P03 tests: registry/lifecycle/outcome 3/3; CA-003 playback 1/1; CA-003 targeting 1/1; CA-004 1/1.
- Final current `cargo test -p game_core`: lib 1,085 plus integrations 51 = 1,136 passed; failed/ignored 0.
- Valid application release probe: exit 0 using actual validated live publication and StartNewGame result.
- Invalid default-release probe: expected exit 101 with `E0599` (`set_enkephalin`), `E0616` (`GameCore.state`) and
  `E0603` (private state module).
- Current P01 verifier: 57 features/headings, 42 locator contents, composite
  `2547febe9ba788d82a4a1b282e2b26bed8edd3cb7af466217e3eb5ef634f4ff7`, errors 0.
- P02 actual valid/invalid release probes remain current; invalid probe retains all `E0432/E0603/E0277/E0599` guards.

### Quality and Performance Disposition

- Whole-library Clippy remains non-zero on the pre-existing denied `never_loop` at
  `src/game/data/boss_runtime_data.rs:1680`, with 79 broad warnings. P03's new `application.rs` and changed world root
  emitted no Clippy warning; existing warnings in mixed `combat.rs`/`state.rs` were not hidden or broadened.
- The full-draft correctness cutover makes the longest debug live-RON battle test exceed 60 seconds in the current
  environment. This is recorded non-blocking observation only. P08 owns like-for-like measurement, optimization and
  performance PASS; P03 introduces no threshold or optimization.
- No policy/Goal/Phase change, gameplay algorithm rewrite, external consumer work or checkpoint/history format change
  was required.

### GEX-10 Result

- Both atomic-group work packages and all P03 completion dimensions have direct current evidence.
- P01/P02 expected successor deltas are re-audited `Complete / High` on records `003`.
- Next action: write the durable P03 receipt, freeze `ATR-CRAR-C2-P03-GEX10-001`, then begin a separated read-only
  GEX-20.
