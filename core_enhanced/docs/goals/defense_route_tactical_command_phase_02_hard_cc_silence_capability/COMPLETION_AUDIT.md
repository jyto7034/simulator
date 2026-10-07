# DefenseRoute 전술 지휘 Phase 2 — Completion Audit

Status: Complete

Confidence: High

Goal:
[Phase 2 중앙 Hard CC/Silence Capability](PLAN.md)

Policy Authority:
[DR-13](../../decisions/defense_route_tactical_command_policy_2026-08-06.md#dr-13--hard-cc와-silence-capability)

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

Audit Date: 2026-08-06

## 판정

Phase 2는 `Complete / High`다. Active buff classification에서 중앙 Hard CC/Silence capability를 파생하고,
overlap/max expiry, basic/automatic/manual action gate와 release 전 cancellation, released delivery/non-refund,
Silence/overlap reason, damage/trigger/cooldown/block/withdraw, strict checkpoint/server projection을 direct regression과
broad suite로 닫았다.

## Evidence Matrix

| ID | 완료 조건 | Runtime/contract evidence | Validation evidence | 판정 |
| --- | --- | --- | --- | --- |
| P2-01 | Active effect-derived capability와 overlap/max expiry | `BuffKind::control_capability`, `BattleCore::unit_control_state`, exclusive replacement 제거, instance별 validator | `hard_cc_overlap_keeps_both_instances_and_uses_latest_release`, overlapping Silence, validator regression | pass |
| P2-02 | 새 basic/auto/manual gate와 unreleased cancellation | central `skill_start_disabled_reason`, basic epoch/deadline, pending cast cleanup와 `SkillCastInterrupted` | basic windup, manual focus, autocast retry focused tests | pass |
| P2-03 | Released delivery와 non-refund | Triggered resolve epoch 제외, existing `ActiveSkillCast`/projectile/area/buff runtime 보존 | released step/buff, projectile, area와 spent resonance tests | pass |
| P2-04 | Silence new-skill-only와 reason priority | Silence는 skill start만 gate, Hard CC-first typed reason | Silence 6, overlap immutable manual rejection | pass |
| P2-05 | Damage/effect/trigger/cooldown/resonance/block/withdraw | periodic damage/proc state 불변, positive resonance gate, block refresh 양쪽 control 검사, withdraw guard 없음 | control invariant regression과 Phase 1 withdraw regression | pass |
| P2-06 | Checkpoint/recovery/server strict contract | required `LiveBattleUnitControlStateDto`, max-expiry/reason world projection, generic server mapping | exact reconnect + old shape rejection, server serialization | pass |
| P2-07 | Data/version/canonical/compatibility | live RON unchanged, event log v27, canonical 2개 갱신, no default/alias | RON 23, direct-name/data diff/version searches | pass |
| P2-08 | Broad 회귀와 dirty 보존 | Phase 0/1 output와 unrelated dirty file 보존 | Core 866, server 55, fmt/diff check | pass |

## 핵심 구현 근거

- `core_enhanced/src/game/battle/buffs.rs`: status metadata의 중앙 capability classification
- `core_enhanced/src/game/battle/core/mod.rs`: derived state/max expiry, typed reason, Hard CC cancellation
- `core_enhanced/src/game/battle/core/basic_attack.rs`: auto windup epoch/deadline과 triggered release 보존
- `core_enhanced/src/game/battle/core/sim.rs`: central auto/manual gates, overlap application/expiry lifecycle
- `core_enhanced/src/game/battle/core/movement/blocking.rs`: Hard CC engagement release/reacquisition
- `core_enhanced/src/game/battle/validation/buffs.rs`: simultaneous active buff instance validation
- `core_enhanced/src/game/behavior.rs`, `core_enhanced/src/game/world/state.rs`: strict unit control checkpoint projection
- `game_server/src/game/player_game_actor/messages.rs`: typed control transport proof
- `core_enhanced/docs/game_rulebook.md`, `core_enhanced/docs/core_runtime_contract.md`: current canonical

## 검증 결과

Focused:

- `cargo test -p game_core hard_cc -- --nocapture` — 15 passed
- `cargo test -p game_core silence -- --nocapture` — 6 passed
- `cargo test -p game_core control_status -- --nocapture` — 11 passed
- server typed reason — 1 passed
- server control transport — 1 passed

Broad:

- `cargo test -p game_core` / `--quiet` — 866 passed total
- `cargo test -p game_server` — 55 passed
- `cargo test -p game_core --test ron_loading` — 23 passed
- `cargo fmt --all -- --check` — pass
- `git diff --check` — pass

## Migration과 compatibility

- Live buff RON/schema migration: 없음
- Event-log version migration: 없음, version 27 유지
- Run save/checkpoint migration: 없음; active battle은 persisted run checkpoint 대상이 아님
- Active battle DTO: required `control` field strict cutover, fallback/default/alias 없음
- Core/game_server는 같은 repository release에서 원자 배포

## 비범위 확인

- Unity runtime/contract/UI/tests는 변경·검증하지 않았다.
- Phase 3 tactical resource와 Phase 5~9 stance/relocation/swap/manual target/FieldWork state 또는 placeholder를
  추가하지 않았다.

## 잔여 위험

현재 Phase 2 범위의 알려진 미해결 correctness gap은 없다. Future stance/Rest/FieldWork/relocation consumer는
각 소유 Phase에서 이 중앙 capability를 재사용해야 하며 concrete buff 이름 검사를 다시 만들면 안 된다.

