# DefenseRoute 전술 지휘 Phase 1 — 회수·재배치 Action Lifecycle

Status: complete

Implementation Complete: true

Goal Type: phase

Execution Mode: implement_and_verify

Production Mutation Authorized: yes — 이 Phase의 `DR-03` 범위에 한정

Parent Master: [defense_route_tactical_command_master](../defense_route_tactical_command_master/PLAN.md)

Predecessor Audit:
[Phase 0 Complete / High](../defense_route_tactical_command_phase_00_policy_baseline/COMPLETION_AUDIT.md)

Phase: 1

Goal Name: `defense_route_tactical_command_phase_01_withdraw_redeploy_lifecycle`

Target Repository: `/mnt/f/work/simulator`

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

Audit Finding ID: 없음

## 목표

`DR-03`의 기존 자발적 철수·전투불능 재배치 의미를 Core authority에서 완결한다. 철수 시 release 전
행동만 취소하고 이미 독립한 delivery는 유지하며, 재배치 HP/condition/identity와 공통 올림 비용을
결정론적으로 적용한다.

## 해결할 사용자-visible 또는 contract-visible 문제

현재 철수/재배치 기반은 존재하지만 재배치 배율을 내림 계산하고, 아직 resolve되지 않은 instant 기본
공격을 철수 뒤에도 적용하며, skill cleanup이 이미 발사된 projectile와 생성된 `GroundFixed` area까지
삭제한다. 승인 정책과 반대인 이 세 lifecycle 경계를 정리하지 않으면 이후 Hard CC, stance와 tactical
movement가 released/unreleased 의미를 공유할 수 없다.

## 승인된 정책과 근거

- Primary policy: `DR-03`
- Target source:
  [DefenseRoute 전술 지휘 목표 정책 결정](../../decisions/defense_route_tactical_command_policy_2026-08-06.md#dr-03--기존-회수와-재배치)
- Historical code/difficulty evidence:
  [임시 조사 노트의 3번](../../tmp/defense_route_tactical_command_design_notes_2026-08-04.ko.md)
- Master boundary:
  [Phase 1](../defense_route_tactical_command_master/PLAN.md#phase-1--회수재배치-action-lifecycle)

## 현재 runtime 및 live data 근거

Current authority 조사 결과는 다음과 같다.

- `BattleCore::withdraw_unit`이 active lifecycle, transient action/target/block, buff와 cast cleanup을 소유한다.
- Basic `AttackResolve`와 `projectiles`, skill `active_skill_casts`/`active_projectiles`/`active_areas`가 release
  경계를 소유한다. 기존 cast cleanup은 detached delivery도 cast 단위로 삭제했고 queued `SkillStep`은
  active cast 제거 뒤에도 실행될 수 있었다.
- `ActiveBattleSession::reconcile_live_deployment_with_battle_state`와 world withdraw handler가 각각 같은
  redeploy cost 내림식을 중복했다.
- Deployment checkpoint는 active unit과 redeploy lock/candidate를 투영하고 game_server는 typed battle update를
  generic mapping으로 전달한다. 새 serialized shape는 필요하지 않았다.
- Production `run/policy.ron`은 기존 30초/90초/150%를 이미 소유했다.

## Source Of Truth와 소유 계층

- 목표 의미: target decision record `DR-03`
- 현재/구현 runtime: BattleCore withdraw/action/delivery와 World deployment/redeploy transaction
- Live tuning: `game_resources/data/run/policy.ron`의 30초/90초/150%
- Current projection/transport: Core battle update/checkpoint와 game_server generic mapping
- 구현 뒤 gameplay/runtime canonical: `game_rulebook.md`, `core_runtime_contract.md`
- 실행 evidence: 이 Phase Goal과 completion audit

## 유지할 불변식

- Active Player runtime instance만 자발적 철수한다.
- 철수는 비용을 소비하거나 과거 배치 비용을 환불하지 않는다.
- Unreleased action은 취소하고 released autonomous delivery는 유지한다.
- Source-owned delivery/effect와 caster-owned transient state는 철수와 함께 종료한다.
- 재배치는 같은 employee identity에 새 runtime instance를 만들고 persistent condition/loadout만 다시
  구성한다.
- Invalid request는 resource, lock, state, event와 revision을 변경하지 않는다.
- 대기와 cooldown은 authoritative battle time을 사용해 Pause 중 진행/catch-up하지 않는다.
- 승인되지 않은 lifecycle option, compatibility와 future Phase placeholder를 추가하지 않는다.
- Phase 0 변경과 기존 사용자 dirty 파일을 보존한다.

## 범위

- 자발적 철수의 basic attack/skill released-unreleased cleanup
- 독립 projectile와 GroundFixed area의 source-exit lifecycle/context
- Source-bound area/aura/follow/channel 및 buff cleanup 회귀 보호
- 재배치 비용의 공통 ceiling 계산과 모든 lock/projection 경로 일치
- 자발적 철수 HP lock, 전투불능 condition/penalty와 새 instance state 회귀 보호
- Core checkpoint/event와 접근 가능한 game_server mapping 검증
- 관련 current canonical 갱신
- Phase exit `Complete / High` 감사

## 명시적 비범위

- Hard CC/Silence 중앙 capability (`DR-13`, Phase 2)
- Tactical resource 이름/DTO 분리 (`DR-02`, Phase 3)
- Weapon tactical profile, stance, relocation, swap, manual target와 FieldWork
- 새 delivery lifecycle authoring option 또는 once-per-battle effect ledger
- Unity runtime/contract/UI/tests
- 승인되지 않은 DTO/save/wire compatibility

## 미결정 정책

현재 알려진 미결정 항목은 없다. Released `GroundFixed`가 필요한 source snapshot을 기존 contract로
보존할 수 없거나 새 authoring/schema 선택이 필요하면 영향 변경을 중단하고
`policy_decision_required`로 전환한다.

## 구현 범위

1. Withdraw command부터 Core cleanup, world redeploy lock/reconcile와 checkpoint까지 current path를 읽는다.
2. 관련 focused baseline과 수정 대상 diff를 확보한다.
3. 공통 ceiling redeploy cost helper를 두 lock 생성 경로와 projection이 공유하게 한다.
4. Release 전 instant basic attack은 취소하고 released basic projectile은 유지한다.
5. Released skill projectile/GroundFixed는 유지하고 source-bound area/runtime은 종료한다.
6. 생성 시점 source context, checkpoint/resync와 stale event가 결정론적으로 유지되는지 검증한다.
7. HP/condition/identity/transient reset 및 Pause behavior를 regression으로 보호한다.
8. Canonical, legacy/fallback, broad validation과 completion audit을 수행한다.

## 완료 조건

- `DR-03` 각 철수/재배치 의미가 누락·축소 없이 runtime과 canonical에 반영된다.
- Release 전 instant attack/cast는 취소되고 released basic/skill projectile와 GroundFixed는 유지된다.
- Source-bound effect, caster buff ownership, target/engagement와 caster transient state는 종료된다.
- 자발적 철수 HP lock과 전투불능 최신 condition/penalty가 기존 의미를 유지한다.
- 재배치 비용이 모든 경로에서 `ceil(base * pct / 100)`을 사용한다.
- 새 runtime instance가 persistent loadout/condition을 재구성하고 이전 transient state를 상속하지 않는다.
- Pause/reconnect와 invalid request 무변경이 focused test로 증명된다.
- Core/server focused+broad, production RON validation과 legacy search가 통과한다.
- Phase exit audit가 `Complete / High`이고 Master가 Phase 2만 ready로 전환한다.

## DTO migration

없음. Existing command/update/checkpoint shape와 game_server mapping을 그대로 사용했다.

## Data/live RON migration

없음. Existing 30초/90초/150% tuning을 유지했고 production RON을 수정하지 않았다.

## Save/checkpoint migration

없음. Released GroundFixed context는 serialized checkpoint가 아닌 private in-memory `AreaRuntime`에
source snapshot/facing을 보존했다. Official checkpoint와 active-run save version은 변경하지 않았다.

## 배포 단위와 producer-consumer rollout 가정

Core와 game_server는 같은 repository release에서 원자 배포한다. 예상대로 serialized shape가 바뀌지
않으면 strict migration은 없다. 구/new behavior compatibility는 제공하지 않는다.

## 필수 focused 검증

- Active vs dead/incapacitated/already-withdrawn request와 실패 무변경
- Windup instant basic attack 취소와 released basic projectile 유지
- Pending skill cast 취소 event와 released skill projectile 유지
- GroundFixed 유지, FollowCaster/FollowTarget/source-owned effect 종료
- Caster/target buff cleanup과 engagement release
- 30% HP lock, 30/90초 Pause freeze와 reconnect preservation
- 150% odd base cost ceiling과 DeployCostReduction 적용
- 최신 condition eligibility, once-per-instance penalty와 new-instance transient reset
- Core event/checkpoint exact shape와 game_server generic mapping

최종 focused regression:

- `cargo test -p game_core withdraw`: 16 passed
- `redeploy_cost_uses_ceiling_for_odd_base_and_saturates`: 1 passed
- `deploy_cost_reduction_consumable_reduces_live_deployment_cost_for_employee`: 1 passed
- `live_deployment_reconciles_defeated_player_unit_during_authoritative_advance`: 1 passed

중복을 제외한 Phase 1 focused test는 19개다. Baseline `withdraw` filter는 변경 전 9개가 통과했다.

## 필수 broad 검증

- `cargo test -p game_core`
- `cargo test -p game_server`
- `cargo test -p game_core --test ron_loading`
- `cargo fmt --all -- --check`

영향 조사에서 더 작은 충분한 broad 범위가 입증되면 이유를 기록하되 compile 성공만으로 대체하지 않는다.

## 생략할 검증과 근거

- Unity runtime/contract/tests: 사용자 승인 비범위
- Phase 2~9 behavior: 선행 implementation 금지
- 관련 없는 외부 repository: 발견되지 않는 한 접근하지 않음

## 중단 조건

- Delivery lifecycle이 독립적인 새 authoring/source-of-truth 경계로 분리되어야 함
- 승인되지 않은 save/checkpoint/wire migration 또는 compatibility가 필요함
- 필수 Core/server source, live data나 test runner에 접근할 수 없음
- Phase 0 완료 evidence가 current tree 변화로 무효화됨
- 기존 사용자/Phase 0 변경과 안전하게 병합할 수 없음

## Compatibility

승인된 compatibility는 없다. Existing command/DTO shape 안에서 strict behavior correction을 수행하고
fallback, dual lifecycle과 schema alias를 추가하지 않는다.

## 완료 감사 Evidence Matrix

| ID | 완료 조건 또는 불변식 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| P1-01 | Withdraw eligibility와 실패 무변경 | `DR-03` | `build.rs::withdraw_unit`, world transaction | status/invalid/revision tests | pass |
| P1-02 | Unreleased basic/skill cancellation | `DR-03` | `basic_attack.rs`, active-cast gate in `sim.rs` | instant attack, pending/active/queued skill tests | pass |
| P1-03 | Released autonomous delivery 유지/source-bound 종료 | `DR-03` | projectile records, snapshot/facing `AreaRuntime`, tracking cleanup | basic/skill projectile와 3 tracking regression | pass |
| P1-04 | HP/condition/identity/transient lifecycle | `DR-03` | HP policy, attempt ledger, redeploy draft | withdraw/death redeploy tests | pass |
| P1-05 | 공통 ceiling 재배치 비용과 Pause/reconnect | `DR-03` | `LiveBattleDeploymentState::redeploy_cost`, lock/checkpoint | odd/overflow/reduction/Pause/resync tests | pass |
| P1-06 | DTO/data/save/server/canonical 일치 | Master Phase 1 gate | shape/version/RON 무변경, 두 current canonical | server 54, RON 23, canonical audit | pass |
| P1-07 | Legacy/compatibility/dirty 보존과 broad 검증 | Master 공통 gate | old floor/delete pattern 0, scoped diff | Core 855/server 54/fmt/diff audit | pass |

## 현재 진행 상태

- Phase 문서 초기화와 Master activation: 완료
- Current runtime/diff/baseline 조사: 완료; 변경 전 9 passed
- Production 구현과 canonical 동기화: 완료
- Focused/broad validation: focused 19, Core 855/server 54/RON 23 모두 통과
- Completion audit: [Complete / High](COMPLETION_AUDIT.md)
