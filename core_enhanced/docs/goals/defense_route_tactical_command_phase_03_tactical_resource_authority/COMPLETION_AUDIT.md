# DefenseRoute 전술 지휘 Phase 3 — Completion Audit

Status: Complete

Confidence: High

Goal:
[Phase 3 전술 자원 Authority](PLAN.md)

Policy Authority:
[DR-02](../../decisions/defense_route_tactical_command_policy_2026-08-06.md#dr-02--전술-포인트)

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

Audit Date: 2026-08-06

## 판정

Phase 3은 `Complete / High`다. Deployment mapping에 결합됐던 legacy cost/stabilization authority를
battle-global `TacticalResourceState`, strict run policy/RON, explicit skill signal과 required
checkpoint/root/server DTO로 분리했다. Initial 20, maximum 99, battle-time 1/초 recovery, deploy 10과 locked
redeploy price가 하나의 clamped pool을 원자적으로 사용하며 Pause/reconnect/Retreat/battle-end 수명을 direct
regression과 broad suite로 닫았다.

## Evidence Matrix

| ID | 완료 조건 | Runtime/data/contract evidence | Validation evidence | 판정 |
| --- | --- | --- | --- | --- |
| P3-01 | Battle-local global authority와 deployment 분리 | `ActiveBattleSession::tactical_resource`, private current/max/cursor, deployment mapping/cooldown-only state | state arithmetic + deploy integration | pass |
| P3-02 | Fixed initial/max, overflow-safe recovery/clamp | `TacticalResourceState::new/recover_until/apply_delta`, live policy 20/99/1 | cadence, cap banking, `u64/u32` overflow tests | pass |
| P3-03 | Deploy/redeploy full atomic payment와 withdraw no-refund | shared `can_pay/pay`, locked ceiling redeploy price, post-Core single payment | deploy/redeploy, insufficient immutability, consumable, paused withdrawal | pass |
| P3-04 | Explicit signed skill delta only | `ModifyTacticalResource` → `TacticalResourceDelta`, current-only world application | positive upper clamp, negative `i32::MIN` lower clamp, legacy effect rejection | pass |
| P3-05 | Pause/reconnect/Retreat/battle-end lifecycle | authoritative battle clock, same in-memory session, `active_battle = None` discard | 50s paused no-catch-up, exact reconnect/root, retreat reinitialize, battle-end broad | pass |
| P3-06 | Strict RON/DTO/server cutover | separate `tactical_resource` policy, required DTO in checkpoint/root, deployment resource fields removed | old policy ownership/name rejection, old DTO rejection, RON 23, server transport | pass |
| P3-07 | Canonical/version/compatibility | gameplay/runtime/data canonical, no event/save version change, no alias/default/dual state | static searches, active-run payload inspection, fmt/diff | pass |
| P3-08 | Broad regression와 dirty preservation | Phase 0~2 output and unrelated user dirty preserved | Core 872, server 55 | pass |

## 핵심 구현 근거

- `core_enhanced/src/game/world/state.rs`: `TacticalResourceState`, recovery/payment/delta와 separate DTO
- `core_enhanced/src/game/world/combat.rs`: battle initialization, deploy/redeploy payment와 skill signal drain
- `core_enhanced/src/game/data/run_policy_data.rs`, `game_resources/data/run/policy.ron`: strict policy split와
  first live tuning
- `core_enhanced/src/game/ability.rs`, `core_enhanced/src/game/battle/core/sim.rs`: explicit authored effect/signal
- `core_enhanced/src/game/behavior.rs`, `core_enhanced/src/game/world/snapshot.rs`: required checkpoint/root DTO
- `game_server/src/game/player_game_actor/messages.rs`, `state.rs`: transport serialization/mapping proof
- `core_enhanced/docs/game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md`: current canonical

## 검증 결과

Focused:

- `cargo test -p game_core tactical_resource` — 8 passed
- Pause/no-catch-up, deploy/redeploy, paused withdraw, reconnect/strict shape, retreat — 각 1 passed
- `cargo test -p game_core deploy_cost` — 2 passed (tactical filter와 1개 중복)
- Legacy stabilization effect rejection — 1 passed
- `cargo test -p game_core --test ron_loading` — 23 passed
- Server battle-update serialization/mapping — 각 1 passed

Broad:

- `cargo test -p game_core` — 872 passed total
  (library 825 + integration 3 + 4 + 23 + 13 + 4)
- `cargo test -p game_server` — 55 passed
- `cargo fmt --all -- --check` — pass
- `git diff --check` — pass

## Migration과 compatibility

- Live RON: required top-level `tactical_resource`로 strict cutover; deployment old resource fields와 alias 없음
- Skill RON: `ModifyTacticalResource` strict name; `ModifyStabilization` rejected
- DTO: checkpoint/root required `tactical_resource`; deployment old resource fields rejected
- Save/checkpoint: active battle은 `ActiveRunCorePayloadV2`/`RunCheckpointPayload`에 없으므로 migration/version
  change 없음
- Event log: 새 semantic event가 없어 version change 없음
- Core/game_server는 같은 repository release에서 원자 배포

## 비범위 확인

- Unity runtime/contract/UI/tests는 읽거나 변경하지 않았다.
- Phase 4~9 weapon/stance/relocation/swap/manual-target/FieldWork state/command/DTO placeholder를 추가하지 않았다.
- Encounter resource override, maximum mutation, debt/partial payment, implicit drain과 DefenseFocus/Rest producer를
  추가하지 않았다.

## Legacy 검색 분류

- `ModifyStabilization`, `initial_cost/max_cost/cost_per_second`는 rejection regression 안에만 남는다.
- `current_cost` production hit는 Shop refresh quote domain이며 tactical resource가 아니다.
- Deployment DTO/resource policy/live RON에는 legacy resource 이름이 없다.

## 잔여 위험

현재 Phase 3 범위의 알려진 correctness gap은 없다. Phase 6/7은 relocation/swap 비용 consumer를 추가할 때
이 pool과 full-payment transaction을 재사용해야 하며 별도 resource authority를 만들면 안 된다. Unity
consumer adoption은 승인된 비범위다.
