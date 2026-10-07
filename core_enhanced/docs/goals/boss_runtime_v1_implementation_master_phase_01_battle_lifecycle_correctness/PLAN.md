# Boss Runtime v1 — Phase 01 Battle Lifecycle Correctness Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — only this child scope
Master Goal: `../boss_runtime_v1_implementation_master/PLAN.md`
Master Phase: P01
Activated: 2026-08-09
Completed: 2026-08-09
Simulator Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

Boss Runtime scheduler와 transition을 추가하기 전에 기존 battle lifecycle의 세 correctness prerequisite를
test-first로 수정한다: max-time terminal authority, 모든 unequal event의 total deterministic order, 그리고
실패 시 Core/world 상태가 변하지 않는 atomic battle advance.

## Direct dependency and handoff

- P00: `../boss_runtime_v1_implementation_master_phase_00_execution_preflight/COMPLETION_AUDIT.md`
  - Verdict: `Complete / High`
  - Handoff: `ready_for_g0`
- Exact owners/tests/failure seams:
  `../boss_runtime_v1_implementation_master_phase_00_execution_preflight/P01_G0_HANDOFF.md`
- Refactor candidates: CRA-P06-C01, CRA-P06-C02, CRA-P06-C03 only

P01 uses one child and three sequential review milestones C01 → C02 → C03. Current dirty files are the user-owned
baseline; edits must be symbol/hunk scoped and must preserve unrelated tactical/Defense Route work.

## In scope

- State-owned one-shot terminal outcome shared by direct/until/by/finalize entrypoints
- Existing-priority-preserving total deterministic `BattleEvent` ordering
- One staged/rollback-safe world advance pipeline for elapsed and next-event-bucket paths
- Failing characterization and injected failure tests
- Closest canonical updates for the corrected current lifecycle contract
- Focused, Core, server and workspace regression validation proportional to the shared runtime risk

## Out of scope

- Boss catalog/schema/binding/runtime/target/movement implementation
- Live Boss RON or reference identity selection
- Public Boss DTO, server mapping or Unity presentation changes
- Tactical player-command transaction refactor CRA-P07-C02
- Independent or deferred CRA candidates
- Gameplay balance, movement/cast/effect formula changes

## Milestone P01-M1 — CRA-P06-C01

1. Add failing direct/until/by cap parity and one-shot finalization tests.
2. Make `BattleExecutionState` own the immutable terminal signal/outcome.
3. Clamp the clock, process no post-cap event and expose the satisfied winner or Draw once.
4. Finalization consumes state authority exactly once.
5. Add live-world cap-to-CombatResult coverage.

## Milestone P01-M2 — CRA-P06-C02

1. Add all-variant `cmp == Equal iff Eq`, payload-pair and permutation tests.
2. Preserve all current priority bands.
3. Add explicit stable variant order; AutoCastEnd precedes ManualCastEnd in their shared band.
4. Add stable semantic identifiers plus deterministic collision-free complete-payload fallback.
5. Retain same-time AttackResolve/BattleEnd and simultaneous cast integration behavior.

## Milestone P01-M3 — CRA-P06-C03

1. Add mutating future-cursor, revision overflow, VN failure and reconcile failure no-mutation tests.
2. Prevalidate cursor/revision before Core mutation.
3. Stage the minimum active-battle/VN/UUID state needed by advance; do not stage unrelated run state.
4. Run step/finalize/trigger/reconcile/DTO construction on staged state.
5. Publish exactly one revision and optional finished transition only after all fallible work succeeds.
6. Route elapsed-time and next-event-bucket entrypoints through one pipeline while preserving their outer semantics.

## Preserved invariants

- Max-time policy, winner semantics, same-timestamp bucket drain and exactly one `BattleEnd`
- Existing priority bands, append-only event seq/cause and deterministic RNG/IDs
- Pause/speed and raw next-bucket distinctions
- Setup snapshot only on initial advance
- Employee/tactical reconciliation and VN trigger thresholds
- Exactly +1 revision on successful advance; zero mutation on any error
- Ordinary non-Boss battle behavior and all current tactical command behavior
- Existing Core/server wire shapes

## DTO, data, save and deployment

- DTO/wire migration: none; P01 is internal correctness and preserves payload shapes.
- Data/live RON migration: none.
- Save/checkpoint migration: none; public checkpoint values must reflect corrected terminal/atomic state.
- Deployment: code-only atomic binary change. No active-session restore or cross-generation conversion is introduced.

## Focused validation

Use fully-qualified `game_core --lib -- --exact` commands for each new test. Then run:

```text
cargo test -p game_core --lib game::battle::core::tests -- --nocapture
cargo test -p game_core --lib game::battle::enums::tests -- --nocapture
cargo test -p game_core --lib game::world::tests::combat -- --nocapture
```

Record actual pass/fail/ignored/filtered counts; zero-match is failure.

## Broad validation

```text
cargo fmt --all -- --check
cargo test -p game_core --lib
cargo test -p game_server
cargo test --workspace --all-targets
```

If a pre-existing unrelated failure appears, record its exact test/error and prove whether the P01 diff can affect it.

## Completion conditions

1. Direct/until/by/next-event/server-used paths have exact terminal parity and a state-owned one-shot result.
2. Unequal live events never compare Equal; all relevant insertion permutations drain identically.
3. Future cursor, revision, VN, reconcile and projection failures leave exact Core/world/VN/resource/revision state
   unchanged.
4. Elapsed and bucket success paths use one transaction and preserve their intended differences.
5. Existing pause, stale authority, cursor, same-time attack/end and result-transition regressions pass.
6. No Boss schema/runtime/data/public Wire or independent refactor enters the Phase.
7. Corrected canonical, focused and broad validations pass with actual counts.
8. User-owned dirty changes are preserved and P01-owned hunks are auditable.
9. Independent completion audit returns `Complete / High` before P02 becomes ready.

## Stop conditions

- A comparator cross-variant choice would materially alter gameplay beyond the approved preserved priority table.
- Atomic staging requires changing public wire/save/deployment semantics.
- Current user edits change while a P01 owner is being patched and the overlapping hunk cannot be safely reconciled.
- A required failure can only be injected through a production-only backdoor rather than a real fallible boundary.

No stop condition became active. Independent completion audit: `Complete / High`, handoff `ready_for_p02`.
