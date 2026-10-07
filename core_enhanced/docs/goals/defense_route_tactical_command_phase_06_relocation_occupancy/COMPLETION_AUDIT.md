# DefenseRoute 전술 지휘 Phase 6 — Completion Audit

Status: Complete

Confidence: High

Goal:
[Phase 6 전술 이동과 점유](PLAN.md)

Policy Authority:
[DR-04, DR-05, DR-12](../../decisions/defense_route_tactical_command_policy_2026-08-06.md)

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

Audit Date: 2026-08-06

## 판정

Phase 6는 `Complete / High`다. Defense employee relocation은 같은 runtime instance를 비용 5, employee-local
10000ms cooldown으로 즉시 이동하며 logical allied occupancy를 deploy/redeploy와 공유한다. 모든 destination,
source, busy/control/resource 검증 뒤 body/facing/effect/engagement/event/revision이 commit되고 실패는 무변경이다.
Released tracking과 Ground/Platform effect 전환, Pause, checkpoint/server/live policy가 같은 strict 계약을 직접
증명한다.

## Evidence Matrix

| ID | 완료 조건 | 정책 근거 | Runtime/data/contract evidence | Validation evidence | 판정 |
| --- | --- | --- | --- | --- | --- |
| P6-01 | Defense relocation, cost/cooldown/no-op/facing | `DR-04` | Core/world command, tactical state, required 5/10000 policy | Core/world success/gates/Pause/resource/immutable | pass |
| P6-02 | Common allied logical occupancy | `DR-05` | shared Core destination validator before deploy and relocation commit | employee/object occupied, source-self, enemy overlap, redeploy smoke | pass |
| P6-03 | Same-instance atomic spatial/tracking | `DR-04` | UnitBody transaction, scoped block release/refresh, existing guidance | identity/buff/cooldown, engagement, Homing/Fixed/three area modes | pass |
| P6-04 | Ground↔Platform effect transition | `DR-12` | Any affinity, focus/block/rest epoch transaction | both directions, full interval, facing-only cadence | pass |
| P6-05 | DTO/event/server/data/version | Master Phase 6 gate | tactical eligibility, relocation event v29, server mapping, required RON | world wire/source ID, server serde, strict policy/RON | pass |
| P6-06 | Canonical/legacy/dirty/broad | Master common gate | four canonical docs, strict current shape, no future placeholder | Core 895/server 55/RON 24/check/fmt/diff/search | pass |

## Runtime 경로 감사

- Entry: `PlayerBehavior::TacticalRelocateUnit` → world serialized battle handler → `BattleLiveCommand`
- Preflight: employee condition/deployed identity → player zone kind → Core source/busy/CC/cooldown → valid/static/
  occupancy/no-op/affinity → global resource/revision
- Mutation: same `RuntimeUnit` body/facing/anchor → tactical cooldown/tile effect/Rest epoch → scoped engagement refresh →
  `UnitTacticallyRelocated` → cost/deployment facing/revision
- Projection: deployed/checkpoint/root battle update → generic game_server battle-update result

No fallible external step remains after Core mutation; world payment is an assertion over the immediately preceding
serialized preflight. No path, reservation, arrival or swap fallback participates in this route.

## 검증 결과

Focused:

- `cargo test -p game_core --lib relocation --quiet` — 8 passed
- occupied deploy/redeploy lifecycle regression — pass
- strict relocation policy regression — pass
- live paused relocation/resource/revision/DTO regression — pass
- game_server live request strict serde regression — pass

Broad:

- `cargo test -p game_core --lib --quiet` — 847 passed
- `cargo test -p game_core --quiet` — 895 passed total
  (library 847 + integration 3 + 4 + 24 + 13 + 4)
- `cargo test -p game_core --test ron_loading --quiet` — 24 passed
- `cargo test -p game_server --quiet` — 55 passed
- `cargo check -p game_core --all-targets` — pass, warning 없음
- `cargo check -p game_server --all-targets` — pass, warning 없음
- Core/server `cargo fmt --all -- --check` — pass
- `git diff --check` — pass
- Failed/ignored — 0

## Migration과 compatibility

- Live RON: required `tactical_relocation(cost, cooldown_ms)` exact shape; missing/zero/bounds/unknown rejects
- Command: exact employee/destination/facing; path/swap/focus compatibility field 없음
- DTO: cooldown/eligibility와 event/result를 같은 release에 required cutover
- Event: `UnitTacticallyRelocated`, strict battle event-log version 29
- Save: active battle은 durable run payload가 아니므로 active-run V3/setup version 유지, reader 없음
- Core/game_server는 atomic repository release; Unity adoption은 승인된 비범위

## Legacy/fallback/long-term fit

- `focus_ms` hit는 future-field rejection fixture뿐이다.
- `FreeSwap`은 기존 skill-fragment replacement 정책이며 tactical relocation/swap fallback이 아니다.
- Relocation path/reservation/pending/arrival/automatic push/swap와 Phase 8/9 state는 없다.
- Common occupancy와 same-instance body authority는 Phase 7 batch swap이 재사용할 장기 seam이며 별도 movement
  subsystem이나 중복 tactical state를 만들지 않았다.

## Dirty와 비범위

- Phase 0~5 output과 unrelated user dirty를 보존했다.
- Unity runtime/contract/UI/tests는 읽거나 변경하지 않았다. Existing external Unity protocol dirty도 보존했다.
- Phase 7 swap, Phase 8 manual targeting/focus와 Phase 9 FieldWork를 선행하지 않았다.

## 잔여 위험

현재 Phase 6 범위의 알려진 correctness gap은 없다. Phase 7은 occupied relocation fallback이 아닌 별도 swap
transaction으로 두 unit의 source gate, destination affinity/effect와 cooldown을 batch 검증해야 한다. Unity
consumer adoption은 승인된 비범위다.
