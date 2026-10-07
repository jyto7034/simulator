# DefenseRoute 전술 지휘 Phase 2 — 중앙 Hard CC/Silence Capability

Status: complete

Implementation Complete: true

Goal Type: phase

Execution Mode: implement_and_verify

Production Mutation Authorized: yes — 이 Phase의 `DR-13` 범위에 한정

Parent Master: [defense_route_tactical_command_master](../defense_route_tactical_command_master/PLAN.md)

Predecessor Audit:
[Phase 1 Complete / High](../defense_route_tactical_command_phase_01_withdraw_redeploy_lifecycle/COMPLETION_AUDIT.md)

Phase: 2

Goal Name: `defense_route_tactical_command_phase_02_hard_cc_silence_capability`

Target Repository: `/mnt/f/work/simulator`

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

Audit Finding ID: 없음

## 목표

`DR-13`의 Hard CC와 Silence를 active effect에서 중앙 파생하는 Core capability로 만들고, 현재 존재하는
기본 공격·automatic/manual skill 시작과 action cancellation이 같은 판정을 사용하게 한다. 중첩/해제
시각, released delivery, cooldown/resonance, checkpoint/recovery와 typed reject reason을 결정론적으로
완결한다.

## 해결할 사용자-visible 또는 contract-visible 문제

기존 runtime은 buff kind를 표현했지만 action consumer가 status variant를 직접 검사했고 서로 다른 Hard CC를
exclusive replacement했다. Hard CC 중 pending skill completion은 취소되지 않고 release 뒤로 연기됐으며
basic windup, block engagement와 checkpoint control state도 완결되지 않았다. 이 Phase는 active buff를 단일
authority로 유지하면서 중앙 capability, release 전 cancellation과 strict reconnect projection을 완성했다.

## 승인된 정책과 근거

- Primary policy: `DR-13`
- Target source:
  [DefenseRoute 전술 지휘 목표 정책 결정](../../decisions/defense_route_tactical_command_policy_2026-08-06.md#dr-13--hard-cc와-silence-capability)
- Master boundary:
  [Phase 2](../defense_route_tactical_command_master/PLAN.md#phase-2--중앙-hard-ccsilence-capability)
- Predecessor release contract:
  [Phase 1 Complete / High](../defense_route_tactical_command_phase_01_withdraw_redeploy_lifecycle/COMPLETION_AUDIT.md)

## 현재 runtime 및 live data 근거

- Live `buffs/base.ron`과 `BuffKind`가 `Stun`, `Freeze`, `Silence`를 이미 표현하며 active authority는
  `BattleCore::buffs`의 `(caster, target, buff)` instance와 `expires_at_ms`다.
- `BuffKind::control_capability()`와 `BattleCore::unit_control_state()`가 중앙 derived state를 소유한다.
- Basic attack, automatic/manual skill, movement, block와 positive resonance gain이 이 query를 소비한다.
- Pending cast는 `PendingSkillCast`, basic windup은 epoch와 owned deadline, released skill/projectile/area는
  각각 기존 detached runtime이 소유한다.
- Official active-battle checkpoint/reconnect는 `LiveBattleUnitControlStateDto`를 필수 unit projection으로
  사용하고 game_server는 Core DTO를 그대로 전달한다. Run checkpoint는 active battle을 저장하지 않는다.

## Source Of Truth와 소유 계층

- 목표 의미: target decision record `DR-13`
- Current/implementation authority: `BuffKind::control_capability`, active `BattleCore::buffs`,
  `BattleCore::unit_control_state`
- Live authoring/tuning: `game_resources/data/buffs/base.ron`
- Projection/transport: `LiveBattleUnitControlStateDto`와 generic game_server battle update mapping
- 구현 뒤 gameplay/runtime canonical: `game_rulebook.md`, `core_runtime_contract.md`
- 실행 evidence: 이 Phase Goal과 completion audit

## 유지할 불변식

- Capability는 개별 command의 Stun/Freeze 이름 나열이 아니라 active effect 분류에서 파생한다.
- 같은 분류가 중첩되면 하나라도 남은 동안 capability를 유지한다.
- Hard CC next release time은 남은 active Hard CC의 최대 expiry다.
- Hard CC는 새 기본 공격과 automatic/manual skill 시작을 막고 적용 순간 unreleased action만 취소한다.
- Released attack/projectile/delayed step/area/buff와 확정 파생 effect는 기존 lifecycle을 유지한다.
- 이미 소비한 resonance와 시작한 cooldown은 환불하지 않는다.
- Silence는 새 automatic/manual skill 시작만 막고 시작된 skill과 기본 공격을 방해하지 않는다.
- Hard CC 자체는 battle time을 멈추지 않으며 damage, 기존 effect와 passive/reactive trigger를 막지 않는다.
- Hard CC와 Silence가 겹치면 Hard CC 제한/reason이 우선한다.
- 자발적 철수는 Hard CC/Silence와 무관하게 계속 허용한다.
- Invalid request는 state, resource, cooldown, event와 revision을 변경하지 않는다.
- 기존 active effect가 checkpoint/recovery에서 보존하는 time/state를 duplicate stored capability 없이 사용한다.
- 승인되지 않은 future stance/movement/work state, compatibility와 placeholder를 추가하지 않는다.
- Phase 0/1 변경과 기존 사용자 dirty 파일을 보존한다.

## 범위

- Hard CC/Silence active-effect 분류와 중앙 derived capability/query
- 중첩 capability와 최대 Hard CC expiry
- 현재 기본 공격·automatic/manual skill eligibility의 공통 capability 소비
- Hard CC 적용 순간 basic windup과 skill cast/focus cancellation
- Released delivery, confirmed derived effect와 resource/cooldown non-refund 회귀 보호
- Silence의 new-skill-only gate와 overlap reason precedence
- Hard CC 중 damage/effect/trigger/cooldown, voluntary withdraw와 battle-time behavior 회귀 보호
- Active effect checkpoint/recovery와 접근 가능한 game_server consumer 검증
- 관련 current canonical 갱신
- Phase exit `Complete / High` 감사

## 명시적 비범위

- Phase 3 tactical resource authority와 새 resonance contract
- Phase 4 weapon tactical profile
- Phase 5 stance, DefenseFocus, effective block, Platform Rest
- Phase 6/7 relocation과 swap
- Phase 8 새 manual target selection shape
- Phase 9 FieldWork assignment/progress
- 위 future feature를 위한 empty state, DTO field나 command placeholder
- Unity runtime/contract/UI/tests
- 승인되지 않은 DTO/save/wire compatibility

## 미결정 정책

현재 알려진 사용자-visible 미결정 항목은 없다. 기존 active effect authoring이 Hard CC/Silence 분류를
표현할 수 없어 새 schema 선택이 필요하거나 save/checkpoint/wire migration에 둘 이상의 유효한 선택이
발견되면 영향 변경을 중단하고 `policy_decision_required`로 전환한다.

## 구현 계획

1. Effect authoring→active runtime→expiry/removal→action gate→checkpoint/server 경로를 읽는다.
2. 수정 대상 diff와 관련 focused baseline을 production 변경 전에 확보한다.
3. Active effect 분류에서 중앙 capability와 최대 Hard CC expiry를 파생한다.
4. Basic/automatic/manual skill start consumer를 공통 판정에 연결한다.
5. Hard CC 적용 edge에서 unreleased action을 취소하고 Phase 1 detached delivery를 보존한다.
6. Silence와 overlap reason, non-refund, battle-time/checkpoint/recovery를 regression으로 보호한다.
7. 필요한 production RON/schema migration이 있으면 strict cutover와 version 책임을 같은 Phase에서 완결한다.
8. Canonical, legacy/fallback, broad validation과 completion audit을 수행한다.

## 완료 조건

- `DR-13`의 현재 구현 가능한 모든 의미가 누락·축소 없이 runtime과 canonical에 반영된다.
- Hard CC/Silence가 active effect에서 중앙 파생되고 중첩/expiry가 direct regression으로 증명된다.
- 기본 공격·automatic/manual skill consumer가 같은 capability를 사용한다.
- Hard CC 적용은 unreleased action만 취소하고 released delivery와 확정 효과를 보존한다.
- Silence는 새 skill 시작만 막고 시작된 skill/기본 공격을 유지한다.
- Non-refund, damage/effect/trigger/cooldown/withdraw와 reason precedence가 증명된다.
- Checkpoint/recovery와 Core/server consumer가 같은 state/time/reason contract를 사용한다.
- 필요한 data/DTO/save/version migration이 실제 live fixture와 strict rollout까지 완결되거나 없음이 증명된다.
- Core/server focused+broad, production RON validation과 legacy search가 통과한다.
- Phase exit audit가 `Complete / High`이고 Master가 Phase 3만 ready로 전환한다.

## DTO migration

`LiveBattleUnitCheckpointDto.control`을 strict required field로 추가했다. Field는 `hard_cc_active`, 남은 Hard
CC 최대 expiry인 `hard_cc_release_time_ms`, `silenced`, 우선순위를 반영한
`skill_start_disabled_reason`을 가진다. Default/alias/dual shape는 없으며 Core와 game_server를 같은 release에서
원자 cutover한다.

## Data/live RON migration

없음. 기존 live `Stun`/`Freeze`/`Silence` kind와 `max_stacks: 1`이 필요한 classification을 표현하며 23개
production RON test가 통과했다.

## Save/checkpoint migration

없음. Derived capability는 active buff와 checkpoint battle time에서 매번 계산하며 run checkpoint/save에
중복 저장하지 않는다. Active battle reconnect는 같은 in-memory battle authority를 projection한다.

## 배포 단위와 producer-consumer rollout 가정

Core와 game_server는 같은 repository release에서 원자 배포한다. Shape/version 변경이 필요하면 fallback,
alias와 dual-read/write 없이 strict cutover하고 모든 workspace producer/consumer와 fixture를 같은 Phase에서
갱신한다.

## 필수 focused 검증

- `cargo test -p game_core hard_cc -- --nocapture`: 15 passed
- `cargo test -p game_core silence -- --nocapture`: 6 passed
- `cargo test -p game_core control_status -- --nocapture`: 11 passed
- `cargo test -p game_server action_unavailable_preserves_typed_disabled_reason_code -- --nocapture`:
  1 passed
- `cargo test -p game_server serializes_control_status_checkpoint_contract_with_typed_reason -- --nocapture`:
  1 passed

위 filter는 overlap/max expiry, basic/auto/manual cancellation, released step/projectile/area/buff, non-refund,
Silence, reason priority, damage/proc cooldown/resonance, block, withdraw와 strict reconnect projection을 직접
포함한다.

## 필수 broad 검증

- `cargo test -p game_core`
- `cargo test -p game_server`
- `cargo test -p game_core --test ron_loading`
- `cargo fmt --all -- --check`

영향 조사에서 추가 package/data validator가 발견되면 포함하고 실제 수를 기록한다.

## 생략할 검증과 근거

- Unity runtime/contract/tests: 사용자 승인 비범위
- Phase 3~9 future behavior: 선행 implementation 금지
- 관련 없는 외부 repository: 발견되지 않는 한 접근하지 않음

## 중단 조건

- Active effect 분류에 사용자-visible schema/policy 선택이 새로 필요함
- 승인되지 않은 save/checkpoint/wire compatibility가 필요함
- 필수 Core/server source, live data나 test runner에 접근할 수 없음
- Phase 1 release contract가 current tree 변화로 무효화됨
- 기존 사용자/선행 Phase 변경과 안전하게 병합할 수 없음

## Compatibility

승인된 compatibility는 없다. Current authority 안에서 strict behavior correction을 수행하고 fallback,
dual capability, status-name alias와 schema alias를 추가하지 않는다.

## 완료 감사 Evidence Matrix

| ID | 완료 조건 또는 불변식 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| P2-01 | 중앙 derived capability, overlap와 최대 expiry | `DR-13` | `buffs.rs`, `core/mod.rs`, validator overlap model | Hard CC 15, Silence 6 | pass |
| P2-02 | Hard CC action gate와 unreleased cancellation | `DR-13` | `basic_attack.rs`, `sim.rs`, owned epoch/deadline와 pending cast cleanup | control status cancellation regressions | pass |
| P2-03 | Released delivery/non-refund 유지 | `DR-13`, Phase 1 | existing detached projectile/area/skill runtime 보존 | released step/projectile/area/buff와 resonance regression | pass |
| P2-04 | Silence new-skill-only와 reason precedence | `DR-13` | central `skill_start_disabled_reason` | Silence 6, immutable overlap reason regression | pass |
| P2-05 | Damage/effect/trigger/time/withdraw 불변식 | `DR-13` | damage/proc state 불변, resonance/block/movement consumer | periodic damage/proc cooldown/block/withdraw regressions | pass |
| P2-06 | Data/DTO/checkpoint/server/canonical 일치 | Master Phase 2 gate | strict control DTO, world projection, canonical 2개 | checkpoint 1, server 55, RON 23 | pass |
| P2-07 | Legacy/compatibility/dirty 보존과 broad 검증 | Master 공통 gate | direct-name/compat/data diff search, predecessor 보존 | Core 866, server 55, fmt/diff check | pass |

## 현재 진행 상태

- Predecessor audit와 child path/dirty 확인: 완료
- Phase 문서 초기화: 완료
- Production runtime/data discovery: 완료
- Production 변경과 strict checkpoint cutover: 완료
- Canonical 갱신: 완료
- Focused/broad validation: 완료
- Completion audit: [Complete / High](COMPLETION_AUDIT.md)
