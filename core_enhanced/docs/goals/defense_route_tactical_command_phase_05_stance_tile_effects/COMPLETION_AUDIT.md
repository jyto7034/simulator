# DefenseRoute 전술 지휘 Phase 5 — Completion Audit

Status: Complete

Confidence: High

Goal:
[Phase 5 태세와 타일 효과](PLAN.md)

Policy Authority:
[DR-06~DR-09](../../decisions/defense_route_tactical_command_policy_2026-08-06.md)

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

Audit Date: 2026-08-06

## 판정

Phase 5는 `Complete / High`다. Core는 `Attack | Defense`만 authoritative stance로 저장하고 employee별
1500ms battle-time cooldown을 적용한다. Defense의 Ground Focus와 Platform Rest는 current tile+weapon에서
파생하며 new-action admission, committed lifecycle, deterministic block shrink, additive damage reduction과
1000ms Rest cadence가 Pause/Hard CC/resonance-lock/same-time damage에 일관되게 통합됐다. Core/root/server/live
RON/canonical과 broad suite가 같은 strict 계약을 증명한다.

## Evidence Matrix

| ID | 완료 조건 | Runtime/data/contract evidence | Validation evidence | 판정 |
| --- | --- | --- | --- | --- |
| P5-01 | Attack/Defense command와 cooldown | tactical state, Core/world command, live policy 1500 | success/same/cooldown/CC/context/immutable/Pause | pass |
| P5-02 | Defense admission과 committed lifecycle | regular attack/auto/manual gates, pending resume | committed basic/manual cast, triggered attack | pass |
| P5-03 | Ground DefenseFocus | base+bonus block, additive modifier, priority shrink | block 2→5→2, 105% clamp, Hard CC, 4→2 shrink | pass |
| P5-04 | Platform Rest | epoch RestTick, damage-first priority, clamp mutation | full interval/reset/zero/no-revive/lock/CC/no-catch-up | pass |
| P5-05 | DTO/event/server/data strict cutover | tactical checkpoint/deployment, StanceChanged v28, required policy | world projection, server request/legacy reject, RON | pass |
| P5-06 | Canonical/dirty/broad | gameplay/runtime/data canonical, no Phase 6+ placeholder | Core 887, server 55, RON 24, check/fmt/diff | pass |

## 핵심 구현 근거

- `core_enhanced/src/game/battle/core/tactical_stance.rs`: stance validation/mutation, derived snapshot와 Rest scheduler
- `core_enhanced/src/game/battle/core/basic_attack.rs`, `core/mod.rs`, `core/sim.rs`: Defense admission, committed
  lifecycle와 event processing
- `core_enhanced/src/game/battle/core/movement/blocking.rs`: capacity shrink survivor ordering
- `core_enhanced/src/game/data/run_policy_data.rs`, `game_resources/data/run/policy.ron`: strict 1500/1000 authority
- `core_enhanced/src/game/behavior.rs`, `world/combat.rs`, `world/state.rs`: command/result/capability/checkpoint/event
- `game_server/src/game/player_game_actor/messages.rs`, `state.rs`: request와 battle-update transport
- `core_enhanced/docs/game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md`: current canonical

## 검증 결과

Focused:

- Tactical stance/effect Core regressions — 4 passed
- Committed manual cast Defense regression — 1 passed
- World command/cooldown/Pause/DTO/atomic rejection regression — 1 passed
- Deterministic block shrink regression — 1 passed
- game_server request/current-vs-derived stance regression — 1 passed
- Required/positive/legacy tactical policy regression — pass

Broad:

- `cargo test -p game_core --lib --quiet` — 839 passed
- `cargo test -p game_core --quiet` — 887 passed total
  (library 839 + integration 3 + 4 + 24 + 13 + 4)
- `cargo test -p game_core --test ron_loading --quiet` — 24 passed
- `cargo test -p game_server --quiet` — 55 passed
- `cargo check -p game_core --all-targets` — pass, warning 없음
- `cargo check -p game_server --all-targets` — pass, warning 없음
- `cargo fmt --all -- --check` — pass
- `git diff --check` — pass
- Failed/ignored — 0

## Migration과 compatibility

- Live RON: required `tactical_stance` exact two fields; missing/zero/unknown/legacy rejects
- Command: `set_engagement_stance` accepts stored `attack|defense` only; derived effect names reject
- DTO: deployment/checkpoint tactical projection and typed reasons added in the same release
- Event: `StanceChanged` added with strict battle event-log version 28
- Save: active battle is not durable run payload; active-run Core remains strict V3, no reader/migration added
- Core/game_server are an atomic repository release; Unity consumer adoption is approved out of scope

## Legacy 검색 분류

- `toggle_cooldown_ms` hit is rejection-test input only.
- `DefenseFocus` and `Rest` runtime enum hits are derived projection/event-effect labels, not stored stance state.
- Authoritative `UnitTacticalState.stance` type is only `EngagementStance::Attack | Defense`.
- Live policy exact values are `stance_change_cooldown_ms: 1500`, `rest_tick_interval_ms: 1000`.

## 비범위 확인

- Unity runtime/contract/UI/tests는 읽거나 변경하지 않았다.
- Phase 6 relocation/occupancy, Phase 7 swap, Phase 8 targeting, Phase 9 FieldWork state/command를 추가하지 않았다.
- Phase 0~4 output과 unrelated user dirty를 보존했다.

## 잔여 위험

현재 Phase 5 범위의 알려진 correctness gap은 없다. Phase 6 tile relocation은 stored Defense를 보존하면서
destination tile kind로 derived Focus/Rest를 원자 전환하고, Rest epoch를 invalidate/restart하며, 현재 command
cooldown과 block/engagement 의미를 유지해야 한다. Unity consumer adoption은 승인된 비범위다.
