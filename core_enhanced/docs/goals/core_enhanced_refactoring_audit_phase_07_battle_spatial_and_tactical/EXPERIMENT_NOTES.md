# Core Enhanced Refactoring Audit Phase 07 — Experiment Notes

Status: complete

## Boundary decision

- Primary authority: battlefield/static topology → runtime occupancy/position → movement and tactical command
  mutation → active-session resource/revision projection.
- Phase 5 owns setup input; Phase 6 owns clock/advance; Phases 8~9 own effect/target semantics; Phase 10 owns DTO.
- Shared large files are read only at spatial/tactical mutation slices.

## Dirty precautions

- Current dirty tree is the baseline; production files may already be modified by the user.
- Baseline hashes/status must be captured before detailed Phase 7 reads.
- Only Master/Phase Goal artifacts are audit-owned.

## CRA-14 policy

- Geometry/movement tests count only when assertions observe authoritative tiles/positions/path/time or atomic state.
- Mechanics tests are not evidence for effect/target correctness owned by later Phases.
- `remove` requires no unique contract and an equal/stronger surviving safety net.
- Runner green counts do not close static gaps.

## Questions to resolve

- Whether field, spatial map, movement backend and projection expose competing position/occupancy authorities.
- Whether deploy/withdraw/stance/relocation/swap/field-work costs and locks are prevalidated and atomic.
- Whether Pause, fixed tick, route/path generation and stale event generations remain deterministic.
- Which broad movement/tactical tests protect behavior versus backend/private construction shape.

## Handoff condition

Phase 8 stays inactive until Phase 7 has complete boundary/candidate/test artifacts, no-mutation proof and an
independent `Complete / High` audit.

## Resolved findings

- Battlefield valid/static topology, runtime body position, derived player occupancy and deterministic block state
  have one current authority. Active movement segments are interpolation caches, not a second position source.
- FieldWorking cross-kind relocation/swap applies tile effects from tile alone, causing duplicate/missing Focus and
  invalid Rest deadlines. C01 owns one full-state tactical-effect transition; current live RON lacks Any weapons but
  typed runtime/canonical already approve the capability.
- Five tactical world handlers can reject a future cursor after Core/resource/mapping/revision mutation. C02 owns
  domain plan/commit atomicity and coordinates its shared cursor seam with Phase 6 C03.
- Production always uses Rapier, while Direct physics, backend selection, Rapier board walls and 19 related test
  bodies maintain non-production paths. C03 transfers unique assertions then removes/depublishes those paths.
- The movement tick/body/output model carries unread and never-produced state. C04 minimizes it only after C03 and
  external Rust consumer inventory.
- Tactical resource, current Pause/cooldown semantics, ordinary relocation/swap, field-work progress/generation and
  live Ground/Platform policy remain retained authorities.
- Deployment-zone overlap is a Phase 5 publication handoff. Invalid TestFixture/reused-salt Core deploy remains
  unverified hardening because current world/live input prevents the partial path.

## Final handoff

- Four candidates: two correctness corrections and two cleanups.
- C01 and C02 are test-first. C02 shares a final transaction mechanism decision with Phase 6 C03; C03 precedes C04.
- Phase 8 may activate only after the independent completion audit records `Complete / High`.
