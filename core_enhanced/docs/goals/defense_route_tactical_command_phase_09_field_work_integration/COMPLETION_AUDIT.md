# DefenseRoute 전술 지휘 Phase 9 — Completion Audit

Status: Complete

Confidence: High

Goal:
[Phase 9 현장 작업과 최종 교차 통합](PLAN.md)

Policy Authority:
[DR-17](../../decisions/defense_route_tactical_command_policy_2026-08-06.md)

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

Audit Date: 2026-08-06

## 판정

Phase 9은 `Complete / High`다. Strict standalone FieldWork setup, object-owned Preserve progress, final-occupancy automatic
assignment, real FieldWorking gates, generation-safe typed completion and stable setup/checkpoint/server projection이 첫 live
Freischutz repair object까지 하나의 vertical slice로 동작한다.

## Evidence Matrix

| ID | 완료 조건 | 정책 근거 | Runtime/data/contract evidence | Validation evidence | 판정 |
| --- | --- | --- | --- | --- | --- |
| P9-01 | Strict setup/live object | `DR-17` | PvE schema → scenario definition; exact one live object | missing/zero/duplicate/wrong target/reset/unknown reject; 256 seeds | pass |
| P9-02 | Atomic assignment/activity lifecycle | `DR-17`, `DR-03/06/08/09/14` | final-occupancy reconcile and real activity | deploy/redeploy/withdraw/death/relocate/swap/Ground/Platform | pass |
| P9-03 | Battle-time progress/control | `DR-17`, `DR-13/15` | object settlement + derived contributor count | single/multi/preserve/enemy damage/Hard CC/Pause no-catch-up | pass |
| P9-04 | Exactly-once completion/effect | `DR-17` | generation/status-first typed repair | clamp/stale/full-HP/event once/underlying tile restoration | pass |
| P9-05 | Setup/checkpoint/event/server | `DR-17` | setup v2, event v31, current Core DTO mapping | live recovery + exact server JSON serialization | pass |
| P9-06 | Canonical/compatibility/dirty/broad | Master gate | rules/runtime/data canonical aligned | Core 910/server 56/RON 24/check/fmt/diff/static | pass |

## Runtime 경로 감사

- Authoring: strict PvE FieldWork data → validated `TacticalPlan.field_work_objects` → immutable setup v2
- Assignment: position/lifecycle command preflight → complete final occupancy commit → one deterministic reconcile
- Activity: assigned Player Combatant → Defense + FieldWorking → central action/skill/stance/tile-effect gates
- Time: settle elapsed × eligible contributors → generation-tagged completion deadline → stale no-op
- Completion: Active-to-Completed first → typed protected-unit repair → one semantic completion → assignment/activity restoration
- Projection: stable setup objects + checkpoint status/progress/cells/workers + unit activity/control + current server envelope

No tactical-point/static-object reuse, work command/cost, Reset state, missed-work accumulator, additional live content or durable
active-battle save participates.

## 검증 결과

Focused:

- strict/missing/invalid schema and exact live object/template reachability — pass
- deploy/same-employee redeploy/withdraw/death/relocation/two-role swap final occupancy — pass
- FieldWorking attack/skill/block/stance/DefenseFocus/Rest gates and restoration — pass
- single/multi-worker, Preserve, enemy overlap/damage, Hard CC and Pause no-catch-up — pass
- stale generation/full-HP exactly-once completion — pass
- setup/checkpoint/recovery/event/server serialized shape — pass

Broad:

- `cargo test -p game_core --quiet` — 910 passed total
  (library 862 + integration 3 + 4 + 24 + 13 + 4)
- `cargo test -p game_server --quiet` — 56 passed
- `cargo test -p game_core --test ron_loading --quiet` — 24 passed
- `cargo check --workspace --all-targets` — pass, warning 없음
- `cargo fmt --all -- --check` — pass
- `git diff --check` — pass
- production placeholder/compatibility/exact-live-object static searches — pass
- Failed/ignored — 0

## Migration과 compatibility

- Data: optional object list but strict present-object shape; no alias/default for required object fields
- Setup: strict current version 2 with separate definitions
- Event: strict version 31 for assignment/completion variants
- Checkpoint/update/resync: current runtime state only; no compatibility reader
- Save: active battle is not durable run payload, so no save version or migration
- Server: Core DTO is forwarded; no assign/cancel command or server-side rule reconstruction

## Dirty와 비범위

- Phase 0~8 verified output and unrelated user dirty were preserved.
- Existing dirty `game_server/docs/unity_bridge_protocol.md` was not edited. Unity runtime/contract/UI/tests were not read or
  changed.
- No additional content or future generic FieldWork placeholder was added.

## 잔여 위험

Phase 9 범위의 알려진 correctness gap은 없다. Unity consumer adoption remains explicitly out of scope. Parent Master must
still complete the final DR-01..DR-17 cross-phase traceability audit before declaring the overall goal complete.
