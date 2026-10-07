# DefenseRoute 전술 지휘 Phase 7 — Completion Audit

Status: Complete

Confidence: High

Goal:
[Phase 7 원자 자리 교대](PLAN.md)

Policy Authority:
[DR-14](../../decisions/defense_route_tactical_command_policy_2026-08-06.md)

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

Audit Date: 2026-08-06

## 판정

Phase 7은 `Complete / High`다. 서로 다른 두 active 배치 employee의 current position과 explicit facing을 별도
`TacticalSwapUnits` transaction 하나로 교환한다. 양쪽 source/destination을 mutation 전에 검증하고 cost 5를 한
번 지불하며 두 employee cooldown을 같은 frozen/current battle time+10000ms로 설정한다. 실패는 양쪽 state와
resource/event/revision을 전혀 바꾸지 않으며 relocation과의 fallback은 없다.

## Evidence Matrix

| ID | 완료 조건 | 정책 근거 | Runtime/data/contract evidence | Validation evidence | 판정 |
| --- | --- | --- | --- | --- | --- |
| P7-01 | Explicit command, cost-once/cooldown-both | `DR-14` | behavior/Core/world route, shared required policy | paused world transaction, cooldown gate | pass |
| P7-02 | Two-source/two-destination preflight | `DR-14` | pair-aware occupancy, source/profile gates | Attack/busy/CC/static/third/resource immutable cases | pass |
| P7-03 | Atomic spatial/engagement/tracking | `DR-14` | release-both, two helper commits, one refresh/event | pair success, identity/state, Phase 6 delivery suite | pass |
| P7-04 | Batch Ground/Platform effect | `DR-14` | common focus/block/Rest epoch helper | Any cross-tile direct swap and fresh Rest | pass |
| P7-05 | DTO/event/server/version | Master Phase 7 | result/presentation/action/server, event v30 | world projection/source ID, strict serde | pass |
| P7-06 | Canonical/legacy/dirty/broad | Master gate | four canonical docs, current-only contract | Core 898/server 55/RON 24/check/fmt/diff/search | pass |

## Runtime 경로 감사

- Entry: `PlayerBehavior::TacticalSwapUnits` → world serialized handler → `BattleLiveCommand::TacticalSwapUnits`
- Preflight: two roster/deployed employees → current positions/zone kinds → both Core source/activity/cooldown gates →
  pair-excluding valid/static/occupancy/affinity/profile → one global resource/revision capacity check
- Mutation: release both engagements → apply both body/facing/effect/deadline updates → one derived refresh/event → pay once →
  update two deployed facings → commit projection once
- Projection: existing per-unit tactical cooldown/eligibility in deployed/checkpoint/root plus one semantic presentation event

No caller destination, sequential relocation, intermediate occupancy, walking/reservation or fallback route participates.

## 검증 결과

Focused:

- `cargo test -p game_core --lib tactical_swap --quiet` — 3 passed
- live paused swap/cost/resource/revision/DTO/source-event regression — pass
- game_server strict live request serde — 1 passed
- Phase 6 relocation/occupancy/tracking suite remained green

Broad:

- `cargo test -p game_core --lib --quiet` — 850 passed
- `cargo test -p game_core --quiet` — 898 passed total
  (library 850 + integration 3 + 4 + 24 + 13 + 4)
- `cargo test -p game_server --quiet` — 55 passed
- `cargo test -p game_core --test ron_loading --quiet` — 24 passed
- Core/game_server all-target check — pass, warning 없음
- `cargo fmt --all -- --check` — pass
- `git diff --check` — pass
- Failed/ignored — 0

Strict `-D warnings` clippy는 unrelated existing tree-wide lint debt로 실패했으며 Phase 7 완료 gate로 지정되지 않았다.

## Migration과 compatibility

- Data: new live RON 없음; existing required relocation cost/cooldown only
- Command: exact two employee UUIDs + two facings; destination/path/legacy alias 없음
- DTO: action/result/presentation event added in the same release; per-unit cooldown shape reused
- Event: `UnitsTacticallySwapped`, strict battle event-log version 30
- Save: active battle is outside durable run payload; no reader/schema migration
- Core/game_server atomic repository release; Unity adoption remains approved out of scope

## Dirty와 비범위

- Phase 0~6 verified output and unrelated user dirty were preserved.
- Unity runtime/contract/UI/tests were not read or changed. Existing external protocol dirty was preserved.
- Phase 8 manual target and Phase 9 FieldWork state/placeholders were not added.

## 잔여 위험

현재 Phase 7 범위의 알려진 correctness gap은 없다. Unity consumer adoption remains out of scope. Phase 8 may add
manual target-selection command/state but must not alter swap destination/fallback/resource ownership.
