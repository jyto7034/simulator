# DefenseRoute 전술 지휘 확장 임시 조사·설계 노트

작성일: 2026-08-04  
최종 갱신일: 2026-08-06  
상태: historical 조사·난이도 evidence — 2026-08-06 동결  
승인 상태: 1~17번 정책 확정·코드 대조·난이도 책정 완료,
17번 첫 live content와 tuning 확정 완료,
전체 dependency·안전한 vertical slice·권장 구현 순서 종합 완료,
구현 미착수

목표 정책 source:
[DefenseRoute 전술 지휘 목표 정책 결정 — 2026-08-06](../decisions/defense_route_tactical_command_policy_2026-08-06.md)

이 문서는 위 decision record에 의해 목표 정책 source로서 superseded되었다. 이 시점의 runtime 조사,
초기 권고안, 난이도와 검증 evidence를 역사적으로 보존하며 이후 구현 상태나 canonical 계약에 맞춰
갱신하지 않는다. 정책 의미를 판단할 때 번호별 과거 초안이나 아래 `부분 승인된 정책` 대신 위
decision record를 사용한다.

## 번호별 정책 재검토 상태

아래 번호는 2026-08-05에 정리한 간략 정책 목록을 기준으로 한다.

1. `전투 목표`: 정책 확정·코드 대조 완료 — 구현 난이도 1/5(매우 낮음)
2. `전술 포인트`: 정책 확정·코드 대조 완료 — 단일 battle-local pool, 구현 난이도 3/5(중간)
3. `기존 회수/재배치`: 정책 재확정·코드 대조 완료 — 자율 전달체만 유지, 구현 난이도 2/5(낮음)
4. `전술 순간이동`: 정책 확정·코드 대조 완료 — 비용 5, 직원별 cooldown 10,000ms, 구현 난이도 4/5(높음)
5. `아군 타일 점유와 배치 검사`: 정책 확정·코드 대조 완료 — 구현 난이도 2/5(낮음), 4·14번 선행 후 증분 1/5
6. `공격/방어 태세`: 정책 확정·코드 대조 완료 — `Attack/Defense` 2상태, 대칭 전환 cooldown
   1,500ms, 구현 난이도 4/5(높음), 4번 공통 seam 선행 후 증분 3/5
7. `태세 전환 중 행동 처리`: 정책·코드 대조 완료 — 즉시 전환, 확정된 행동과 파생 효과 완료,
   현재 tree 구현 난이도 4/5(높음), 4·6·14번 선행 후 증분 2/5(낮음)
8. `Ground 방어 집중`: 축소 정책·코드 대조 완료 — 현재 tree 구현 난이도 4/5(높음), 6번 선행 후
   3/5(중간), 6·10번 선행 후 증분 2/5(낮음)
9. `Platform 휴식`: 정책·코드 대조 완료 — 공통 cadence 1,000ms, 첫 weapon 기준 HP 3/tick·공명
   5/tick, 현재 tree 구현 난이도 4/5(높음), 6번 선행 후 3/5(중간), 6·10번 선행 후 증분 2/5(낮음)
10. `무기 데이터 소유권`: 정책·코드 대조 완료 — strict active-run version cutover, 현재 tree
    구현 난이도 3/5(중간), 6·12번 선행 후 증분 2/5(낮음)
11. `무기 없는 직원`: 정책·코드 대조 완료 — 안전지대 관리는 허용하고 전투 gameplay만
    `weapon_required`로 비활성, starter 훈련검 보장, 구현 난이도 2/5(낮음)
12. `Ground/Platform 순간이동`: 정책·코드 대조 완료 — 별도 command 없이 전술 순간이동·자리 교대의
    affinity 검사와 원자적 tile effect 전환으로 처리, 현재 tree 4/5(높음), 4·10번 후 3/5,
    4·6·8·9·10·14번 후 증분 2/5
13. `Hard CC와 Silence`: 정책·코드 대조·난이도 책정 완료 — 중앙 capability, Hard CC는 release
    전 행동 취소·전술 기능 중단, Silence는 새 skill 시작만 제한, 현재 tree 3/5(중간),
    4·6·8·9·12·14·17번 선행 후 증분 2/5(낮음)
14. `자리 교대`: 정책·코드 대조·난이도·tuning 확정 완료 — 하나의 전술 이동 기능에서 점유 아군
    선택 시 분기, FieldWorking 교대·원자적 작업자 인계 허용, 비용 5/양쪽 직원 cooldown 10,000ms,
    현재 tree 4/5(높음), 선행 정책 후 3/5(중간), 17번까지 선행 후 증분 2/5(낮음)
15. `Pause 중 명령`: 정책·코드 대조·난이도 책정 완료 — battle clock만 정지하고 최신 revision에서
    mutation을 한 번에 하나씩 즉시 처리, client plan/batch 없음, Pause 중 stored speed 변경 허용,
    구현 난이도 1/5(매우 낮음)
16. `수동 지정 스킬`: 정책·코드 대조·난이도·첫 live content 확정 완료 — 현재 tree 4/5(높음),
    4·6·13번 선행 후 3/5(중간), Black Round progression `PlayerUnit`, 별도 skill cooldown과 첫
    `PlayerTile` live content 없음
17. `현장 작업`: 정책·코드 대조·난이도·첫 live content 확정 완료 — 별도 object/assignment,
    자동 assignment, Preserve-only object progress, 파생 suspension, exclusive battle-time 작업,
    현재 tree 4/5·선행 정책 후 3/5, Freischutz 긴급 수리 10,000ms/100 HP

## 순차 정책 확정 및 구현 난이도 기록

이 절은 1번부터 17번까지 각 항목을 사용자와 다시 검토한 뒤 확정한 정책, 실제 code/live data 대조와
구현 난이도를 즉시 누적한다. 여기서 `정책 확정`은 이 임시 설계 범위의 합의를 뜻하며 durable
canonical 승격이나 구현 시작을 뜻하지 않는다. 난이도는 현재 code/data/wire/Unity 상태에서 해당
항목을 구현할 때의 상대 등급이며, 인접 항목과 하나의 vertical slice로 합치면 다시 조정할 수 있다.
4번부터는 난이도 판정에 사용한 핵심 reference code/live data와 각 파일의 관련 책임을 같은 번호 기록에
함께 남긴다. 1~6번 조사에는 당시 확인한 외부 Unity reference도 보존하되, 사용자 결정에 따라 7번부터는
구현된 기반이 없는 외부 Unity를 조사·난이도 책정 범위에서 제외한다.

### 1. 전투 목표 — 정책 확정 / 난이도 1/5

확정일: 2026-08-05  
구현 난이도: `1/5 — 매우 낮음`

확정 정책:

- 공식 전투 모델은 계속 `DefenseRoute` 하나다.
- 기본 공격과 기본 공격 대상 선택은 기존 자동 전투와 weapon targeting profile을 유지한다.
- 배치·철수·태세·전술 이동·수동 스킬·현장 작업은 기존 전투 목표를 달성하기 위한 전술 수단이다.
  해당 기능을 사용하지 않았다는 사실 자체는 승리 또는 패배 조건이 아니다.
- 모든 필수 wave와 적을 처리하고 보호 대상을 생존시키는 기존 기본 승리 조건을 유지한다. 보호 대상
  파괴나 encounter가 명시한 scenario failure condition은 패배를 만든다.
- 생존 전투는 기존 `DefenseRoute / Defense + survive_timer_ms`를 유지하며 timer 도달 시 남은 적과
  무관하게 승리한다.
- 높은 APM이나 모든 유닛의 공격을 수동으로 지정하는 RTS를 목표로 하지 않는다.
- 현장 작업이 승리·실패 조건에 관여하려면 encounter가 이를 명시적으로 authoring해야 한다. 허용할
  구체 형태와 첫 content는 17번에서 결정한다.

실제 코드 대조:

- `src/game/battle/scenario.rs`의 `WinCondition`은 필수 적 전멸, 특정 unit 격파, 보호, 제한 시간 보호와
  시간 생존을 전술 명령과 독립적으로 소유한다.
- `src/game/battle/core/sim.rs::compute_winner`는 `WinCondition`, 실제 unit lifecycle과
  `required_for_victory` group만으로 결과를 계산하고 특정 전술 기능 사용 여부를 검사하지 않는다.
- `src/game/combat_setup/mission_policy.rs`는 Defense/Boss의 기본 보호 대상과 `survive_timer_ms` 기반
  `ProtectUnitUntil`을 이미 구성한다.
- `src/game/events/combat.rs`는 authored win condition을 우선하고 없으면 위 기본 정책을 사용한다.
- `src/game/world/combat.rs`는 공식 live 진입을 Defense/Defense와 Boss/Boss 조합으로 제한한다.
- `game_resources/data/pve/encounters.ron`의 현재 live encounter도 기본 보호/필수 wave 계약과
  `suppress_clouded_monk`의 45초 생존 계약을 사용한다.

난이도 판정:

- 합의 정책은 현재 runtime과 일치하므로 gameplay runtime 변경은 필요하지 않다.
- 구현 작업은 이 합의를 `archive/game_rulebook_legacy_2026-10-04.md`에 승격하고 인접 canonical과 표현을 동기화하는 문서 작업이
  중심이다.
- 필수 field work objective를 실제로 추가하는 비용은 이 항목에 포함하지 않고 17번에서 책정한다.

검증 결과:

- `cargo test -p game_core protect_unit_until`: 2 passed
- `cargo test -p game_core battle_setup_snapshot_exposes_survival_timer_without_encirclement_variant`:
  1 passed

### 2. 전술 포인트 — 정책 확정 / 난이도 3/5

확정일: 2026-08-05  
구현 난이도: `3/5 — 중간`

확정 정책:

- 플레이어 표시명은 `전술 포인트`, code/DTO domain 개념명은 `tactical_resource`다. 새 계약에서
  `stabilization`이나 일반적인 `cost`를 자원 이름으로 유지하지 않는다.
- 전술 포인트는 active battle에 속한 player-side 전역 자원이다. 전투 시작 때 live policy의 initial로
  초기화하고 reconnect/resync에서는 현재 값을 보존하며 전투 종료 때 폐기한다. Retreat는 기존
  battle-entry rollback과 함께 이 battle-local 상태를 버리고 다음 전투에 잔량을 이월하지 않는다.
- 최초 배치, 기존 회수/전투불능 뒤 재배치, 전술 순간이동과 자리 교대가 하나의 current/maximum pool을
  공유한다. Field work 자체에는 별도 작업 비용이 없다.
- current는 항상 `0..maximum`이다. 모든 회복은 maximum에서, 명시적인 감소 효과는 0에서 clamp하고
  초과분을 저장·이월·변환하지 않는다.
- 비용은 전액을 보유한 경우에만 원자적으로 지불한다. 부분 지불, 음수 잔액과 자원 debt는 없으며
  validation 실패는 resource/state/event/revision을 변경하지 않는다. 성공해 commit된 비용은 이후
  상황 변화로 환불하지 않는다.
- 첫 버전의 maximum은 전투 시작 때 고정한다. skill, weapon과 encounter effect는 current만 증감하며
  maximum 변경 효과는 제공하지 않는다.
- 공통 initial, maximum, 기본 자연 회복과 배치·재배치·순간이동·교대의 공통 비용은
  `run/policy.ron`이 소유한다. Skill의 즉시 증감은 해당 skill RON이 소유한다. Ground
  `DefenseFocus`와 Platform `Rest`는 전술 포인트를 회복하지 않는다. 같은 수치를 여러 source에 중복
  authoring하지 않는다.
- 첫 버전에는 encounter별 전술 포인트 initial/maximum/자연 회복 override가 없다.
- 명시적으로 authored한 effect는 current를 증가시키거나 감소시킬 수 있다. 자연 회복 외의 암묵적인
  환경 감소나 적의 기본 drain은 없고 각 producer가 발동 조건, 양, 주기와 중첩 규칙을 소유한다.
- 모든 시간 기반 회복은 authoritative battle time을 사용한다. Pause 중에는 진행하지 않고 해제 뒤
  소급 지급하지 않는다.
- 현재 live baseline `initial 20 / maximum 99 / 자연 회복 초당 1`은 이 기능 때문에 변경하지 않는다.
  별도 balance 결정이 생기기 전 첫 구현도 같은 값을 유지한다. 순간이동·교대·태세 효과의 신규 수치는
  해당 번호에서 별도로 확정한다.

실제 코드 대조:

- `src/game/world/state.rs::LiveBattleDeploymentState`가 현재 `current_cost`, `max_cost`,
  `cost_per_second`, 마지막 갱신 시각과 배치/redeploy 상태를 함께 소유한다. 전투별 초기화와 수명은 이미
  `ActiveBattleSession` 경계에 있다.
- 같은 파일의 자연 회복은 authoritative battle time의 정수 초를 사용하고 maximum에서 clamp한다.
  `BattleLiveSignal::StabilizationDelta` 적용도 양수는 maximum, 음수는 0에서 clamp한다.
- `src/game/world/combat.rs::handle_deploy_unit`은 잔액 eligibility를 먼저 검사하고 성공한 Core 배치와
  함께 비용을 차감한다. 기존 focused test는 부족한 비용 요청이 state, salt와 revision을 변경하지
  않는다는 것을 확인한다.
- `src/game/ability.rs`와 `src/game/battle/core/sim.rs`에는 signed `ModifyStabilization` effect와
  `StabilizationDelta` signal seam이 이미 있다. 현재 live skill RON은 이 effect를 사용하지 않는다.
- `src/game/data/run_policy_data.rs`와 `game_resources/data/run/policy.ron`은 현재 값을
  `live_deployment` policy 하나에 저장하고 encounter override를 제공하지 않는다.
- 현재 wire는 `battle_update.checkpoint.deployment`와 root `in_battle.deployment` 안에 자원과 배치
  projection을 결합한다. 외부 Unity의 `DeploymentModel`도 같은 결합 shape를 직접 읽는다.

필요한 구현 범위:

- `LiveBattleDeploymentState`에서 자원 책임을 `TacticalResourceState`로 분리하고 배치 mapping,
  redeploy lock과 instance salt는 별도 deployment state에 둔다.
- run policy를 tactical resource policy와 deployment policy로 분리하고 구/new RON field를 병행하지
  않는다.
- `ModifyStabilization`/`StabilizationDelta`와 관련 테스트·표시를 tactical resource domain 이름으로
  교체한다.
- checkpoint와 root in-battle projection에 별도 `tactical_resource` DTO를 두고 deployment DTO에서는
  current/maximum/자연 회복 필드를 제거한다.
- game_server serialization, 외부 Unity reader/HUD/mock/contract test와 두 Unity canonical 문서를 같은
  breaking migration에서 갱신한다.
- 자연 회복의 Pause 정지, signed delta clamp, reconnect/resync 보존, 종료·Retreat 폐기와 새 DTO
  casing/nullability를 focused test로 추가한다.

난이도 판정:

- 자원 산술, battle-local 수명, clamp, 비용 부족 원자성과 signed skill delta 기반은 이미 존재하므로
  새로운 알고리즘 위험은 낮다.
- 반면 자원과 deployment ownership을 분리하고 compatibility alias 없이 Core data/state, wire DTO,
  server test와 실제 Unity 소비 코드를 동시에 바꾸는 breaking migration이 필요하다.
- 따라서 Core 계산만 보면 낮음에 가깝지만 전체 vertical slice 기준 난이도는 `3/5 — 중간`이다.

검증 결과:

- `cargo test -p game_core live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock`:
  1 passed
- `cargo test -p game_core manual_fragment_can_restore_stabilization_and_enable_extra_deployment`:
  1 passed

### 3. 기존 회수/재배치 — 정책 재확정 / 난이도 2/5

확정일: 2026-08-05  
재확정일: 2026-08-05  
구현 난이도: `2/5 — 낮음`

확정 정책:

- 자발적 철수는 전장에 active 상태로 배치된 player unit이면 현재 태세와 무관하게 사용할 수 있다.
  기본 공격 선딜, skill cast/focus, Hard CC, Silence 또는 FieldWorking 중에도 철수할 수 있다. 사망,
  전투불능 또는 이미 철수한 unit에는 사용할 수 없다.
- 자발적 철수 자체는 전술 포인트를 소비하지 않으며 최초 배치 비용을 환불하지 않는다. 성공하면 기존
  runtime instance를 즉시 `Withdrawn`으로 만들고 전장에서 제거한다. 재배치할 때 새 위치와 방향을 다시
  선택한다.
- 아직 release되지 않은 기본 공격과 skill cast/focus는 철수 시 취소한다. instant 기본 공격은 실제
  damage resolve가 release 경계이므로 그 전에 철수하면 취소한다.
- 철수 전에 이미 발사된 기본 공격·skill 투사체와 이미 선언·생성된 `GroundFixed` area는 시전자와
  분리된 자율 delivery로 보고 각자의 기존 lifecycle을 계속한다. 발사·생성된 뒤에는 시전 취소나
  시전자 철수 때문에 제거하지 않는다.
- `FollowCaster`, `FollowTarget`, aura, channel/focus처럼 계속 unit이나 source runtime에 종속되는 효과는
  자율 delivery가 아니며 철수와 함께 종료한다. 첫 버전에는 이를 철수 뒤 유지시키는 별도 authoring
  option을 추가하지 않는다.
- 철수한 instance가 다른 unit에 이미 적용한 buff/debuff도 기존 `CasterWithdrawn` 규칙대로 종료한다.
  실제 content에서 출처 소멸 뒤에도 유지할 필요가 생기기 전에는
  `persists_after_source_exit` 같은 새 lifecycle policy를 도입하지 않는다.
- 철수한 instance 자신의 buff/debuff, target, resonance, 기본 공격·skill cooldown과 cast/action 상태는
  기존 instance와 함께 폐기한다. 철수한 unit을 표적으로 비행 중인 적대 투사체는 빗나가며, 해당
  unit의 저지와 교전 관계는 즉시 해제한다.
- 자발적 철수 뒤 재배치 HP는 철수 성공 시점에
  `min(철수 시 최대 HP, 철수 시 현재 HP + floor(철수 시 최대 HP * 30 / 100))`로 고정한다. 대기 시간에
  따른 추가 회복은 없고 physical damage와 trauma를 변경하지 않으며, 이후 최대 HP 변화도 이미 잠근
  재배치 HP를 소급 변경하지 않는다.
- 전투불능은 각 incapacitated runtime instance마다 physical damage `+1`과 authored trauma를 정확히
  한 번 반영한다. 같은 직원이 새 instance로 재배치된 뒤 다시 전투불능이 되면 새 penalty를 다시
  적용한다.
- 전투불능 뒤에는 penalty가 반영된 직원의 최신 condition이 전투 참가 가능할 때만 재배치 후보가 된다.
  `Critical`, `Breakdown`, `Dead`처럼 참가 불가능한 condition은 후보에서 제외하며, 재배치 시작 HP는
  고정 비율이 아니라 이 최신 condition에서 계산한다.
- 재배치는 같은 employee identity로 새 runtime instance를 만든다. 전투 진입 시 확정된
  loadout/skill fragment, 최신 persistent condition, 아직 유효한 battle-scoped consumable modifier,
  장비·fragment 기반 stat과 배치 초기 효과를 다시 구성한다.
- 새 instance는 이전 instance의 transient state, buff/debuff, resonance, 기본 공격·skill cooldown,
  cast, target, block과 aura state를 상속하지 않는다. 철수 전에 독립한 자율 delivery도 새 instance가
  다시 소유하거나 제어하지 않는다. `InitialSkillCharge`와 명시적인 `on deploy` 효과는 새 배치마다 다시
  적용한다. 향후 진짜 once-per-battle 효과가 필요하면 별도 battle ledger로 명시한다.
- 현재 live baseline인 자발적 철수 재배치 대기 `30초`, 전투불능 재배치 대기 `90초`, 재배치 비용
  `최초 배치 비용의 150%`를 유지한다. 시간은 authoritative battle time을 사용하므로 Pause 중에는
  진행하지 않고 reconnect/resync에서 보존한다.
- 재배치 비용은 `ceil(base_deploy_cost * multiplier_pct / 100)`으로 계산한다. 비용 감소 consumable은
  최초 배치와 재배치에 모두 적용한다. invalid 요청은 state/resource/event/revision을 변경하지 않으며
  성공한 최초 배치 비용은 철수 뒤 환불하지 않는다.

실제 코드 대조:

- `src/game/battle/core/build.rs::withdraw_unit`은 active lifecycle만 요구하고 stance, action lock, Hard CC나
  Silence를 금지 조건으로 검사하지 않는다. 성공 시 lifecycle, action/cast/target/movement/block 상태와
  소유 item을 정리하므로 현재 구현 골격은 즉시 철수 의미와 맞는다. `FieldWorking`은 아직 runtime에
  없으므로 17번 구현 때 같은 허용 규칙을 연결해야 한다.
- `src/game/battle/core/sim.rs::BattleDeployCurrentHpPolicy::withdraw_redeploy`는 현재 HP에 최대 HP의 30%를
  정수 내림으로 더하고 최대 HP에서 clamp한다. `src/game/world/combat.rs::handle_withdraw_unit`이 이 값을
  철수 직후 redeploy lock에 저장하므로 대기 중 회복이나 소급 재계산이 없다.
- `src/game/world/state.rs::reconcile_live_deployment_with_battle_state`와
  `src/game/world/combat.rs::reconcile_employee_attempt`은 전투불능 instance를 배치 목록에서 제거하고
  processed instance 집합으로 penalty를 한 번만 적용한다. 새 배치는 새 instance id를 받으므로 반복
  전투불능은 다시 반영된다.
- 직원 condition의 combat eligibility가 재배치 candidate 노출에도 적용된다. 전투불능 뒤 새 draft와
  시작 HP는 attempt roster의 최신 condition에서 다시 만들어진다.
- `src/game/battle/stat_pipeline.rs`와 `src/game/combat_setup/player_spawns.rs`는 재배치마다 현재
  battle-entry employee/loadout/fragment와 유효한 consumable modifier로 새 profile을 구성한다. 새
  instance는 이전 resonance/cooldown/target/cast를 상속하지 않고 `InitialSkillCharge`를 다시 받는다.
- 이미 발사된 기본 공격 투사체는 별도 projectile runtime에 남고 철수한 표적을 inactive로 판정해
  miss한다. buff cleanup은 target이 철수했거나 caster가 철수한 두 경우를 구분해 기존 효과를 종료한다.
  이 두 동작은 재확정 정책과 일치한다.

현재 정책과 다른 코드:

- `src/game/world/combat.rs::handle_withdraw_unit`과
  `src/game/world/state.rs::reconcile_live_deployment_with_battle_state`는 재배치 비용을 현재
  `base * pct / 100`으로 내림 계산한다. 확정한 올림 계산 helper를 두 경로가 공유해야 한다.
- 철수는 unit의 pending 기본 공격 상태를 지우지만, 이미 event queue에 commit된 instant 기본 공격은
  `src/game/battle/core/commands.rs::resolve_committed_basic_attack`에서 공격자가 inactive여도 snapshot
  damage를 적용한다. 철수한 공격자의 아직 해결되지 않은 instant 공격을 취소하도록 보강해야 한다.
- `cancel_active_skill_runtime_for_withdrawn_unit`은 시전과 함께 같은 cast의 `active_projectiles`와
  `active_areas`를 모두 삭제한다. 이미 발사된 skill projectile과 `GroundFixed` area만 유지하고
  `FollowCaster`/`FollowTarget` area는 기존처럼 종료하도록 선택적으로 분리해야 한다.
- active skill projectile은 이미 source snapshot을 보유해 시전자 없이 impact를 처리할 기반이 있다.
  `GroundFixed` area도 고정 geometry를 보유하지만 damage tick은 현재 active caster에서 source snapshot을
  다시 만들기 때문에, 생성 시점의 독립 damage/effect context를 보존하도록 작은 runtime 보강이 필요하다.

필요한 구현 범위:

- 재배치 배율의 공통 올림 계산 helper를 만들고 자발적 철수·전투불능 lock 생성과 관련 DTO/test가 같은
  값을 사용하게 한다.
- 철수 시 미해결 instant 기본 공격은 취소하되 이미 발사된 기본 공격 투사체는 유지하는 회귀 테스트를
  추가한다.
- 기존 `TargetWithdrawn`/`CasterWithdrawn` buff cleanup은 유지하고 회귀 테스트로 계약을 고정한다.
- pending/active skill cast는 `Withdrawn` 취소 event를 남긴다. 같은 cast에서 이미 발사된 skill
  projectile과 생성된 `GroundFixed` area만 독립 delivery로 유지하고 source-dependent area는 제거한다.
  GroundFixed tick에 필요한 생성 시점 source context와 checkpoint/resync 결정론을 함께 보강한다.
- Hard CC, Silence, action/cast 중 철수 허용, 비용 무환불, 새 instance의 transient state 초기화,
  `InitialSkillCharge` 재적용, 참가 불가능 condition 제외와 홀수 base 비용의 올림 사례를 focused test로
  보강한다.

난이도 판정:

- 철수 명령, 30% HP 잠금, 새 runtime instance, 두 종류의 redeploy lock, 최신 condition 기반 HP,
  once-per-instance penalty와 참가 가능성 filtering은 이미 구현되어 있다. wire/Unity 명령 shape도 새로
  바꿀 필요가 없다.
- 남은 변경은 재배치 비용 올림, release 전 instant 공격 취소, skill cast cleanup에서 이미 독립한
  projectile/GroundFixed area를 선택적으로 보존하는 데 집중된다. 투사체 snapshot과 area 고정 geometry가
  이미 있어 전면적인 delivery architecture 변경은 필요하지 않다.
- wire/Unity 명령 shape 변경 없이 Core의 제한된 lifecycle 보강과 focused regression test로 마칠 수
  있으므로 전체 vertical slice 난이도는 `2/5 — 낮음`이다.

검증 결과:

- `cargo test -p game_core live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock`:
  1 passed
- `cargo test -p game_core live_deployment_reconciles_defeated_player_unit_during_authoritative_advance`:
  1 passed
- `cargo test -p game_core withdraw_`: 4 passed

### 4. 전술 순간이동 — 정책 확정 / 난이도 4/5

확정일: 2026-08-05  
난이도·초기 tuning 확정일: 2026-08-05  
구현 난이도: `4/5 — 높음`  
초기 live tuning: `전술 포인트 5 / 직원별 cooldown 10,000ms`

확정 정책:

- `TacticalRelocateUnit`은 기존 철수·재배치를 호출하지 않는 별도 server-authoritative 명령이다. 같은
  active player runtime instance의 logical/world position과 facing을 한 mutation에서 즉시 변경한다.
- 전술 순간이동은 보행이 아니다. pathfinding, 이동 시간, movement segment, source/destination 예약,
  relocating 중간 상태, 도착·취소와 환불 단계를 만들지 않는다.
- 성공 뒤에도 HP, buff/debuff, resonance, stance, 기본 공격·skill cooldown과 runtime identity를
  유지한다. 새 위치 때문에 달라지는 저지, target/range eligibility, tile effect와 area/aura 포함 관계만
  authoritative하게 다시 계산한다.
- source는 `Defense` stance여야 한다. 아직 resolve/end되지 않은 기본 공격이나 skill action 중에는
  사용할 수 없다. Silence와 `FieldWorking`은 사용을 막지 않고 Hard CC는 막는다.
- tactical relocation 가능 여부는 Hard CC 이름을 여러 command handler가 직접 나열하지 않고 공통
  capability의 `blocks_tactical_relocation` 의미로 판정한다. 첫 버전에서 Hard CC와 아직 끝나지 않은
  기본 공격·skill action은 차단하고 Silence와 FieldWorking은 허용한다. 향후 exclusive activity도 이
  capability를 명시한다.
- 저지 중인 source도 사용할 수 있다. 성공 mutation 안에서 기존 engagement를 먼저 해제하고 위치를
  변경한 뒤 새 위치의 blocker/engagement를 표준 deterministic 규칙으로 다시 계산한다.
- `FieldWorking` source가 기존 work cell을 벗어나면 같은 mutation에서 기존 assignment를 종료한다. 새
  work cell로 이동하면 즉시 새 assignment를 시작한다. 작업 전환에는 이동 command의 원래 비용 외 별도
  field work 비용을 부과하지 않는다. 구체 assignment 계약은 17번 확정 정책을 따른다.
- destination은 최초 배치와 같은 deployment zone, 현재 weapon deployment affinity, valid tile과 static
  obstacle 검사를 통과해야 한다. 보행 traversal과 경로 도달 가능성은 검사하지 않는다.
- 다른 active player-side allied unit이 destination tactical slot을 점유하면 `position_occupied`로
  거부한다. 점유된 두 아군 slot의 교환은 14번 자리 교대만 사용한다. 적의 projected tile 겹침은 이
  아군 slot 배타 규칙으로 금지하지 않고 새 위치의 표준 spatial/blocking 판정에 맡긴다.
- destination과 현재 position이 같고 facing도 같으면 no-op invalid 요청으로 거부한다. position은 같고
  facing만 다르면 별도 command를 추가하지 않고 같은 `TacticalRelocateUnit`의 전술 재정렬로 허용하며
  일반 전술 순간이동과 같은 비용·쿨타임을 적용한다.
- source를 대상으로 이미 시작되거나 생성된 적의 공격, skill과 projectile을 순간이동 자체가 삭제하거나
  취소하지 않는다. 같은 runtime identity를 대상으로 각 delivery가 이동 뒤 authoritative state에서 기존
  resolver 규칙대로 처리된다. 사거리 재검증 공격은 실패할 수 있고 유도 투사체는 계속 추적하며 고정
  궤도 투사체는 원래 경로를 유지한다.
- source가 만든 `GroundFixed` 효과는 원래 위치에 남는다. source-bound aura와 `FollowCaster`는 같은
  instance의 새 위치를 따르고, `FollowTarget`은 원래 target을 계속 따른다.
- command는 destination/facing, stance/action/capability, affinity/field validity, allied occupancy,
  tactical resource와 직원별 cooldown을 모두 먼저 검증한다. 성공할 때만 위치·facing·전술 포인트와
  cooldown을 원자적으로 commit하고 semantic relocation event 하나를 기록한다. invalid 요청은
  state/resource/event/revision을 변경하지 않는다.
- 모든 직원은 첫 버전에서 같은 fixed tactical resource cost와 같은 base cooldown을 사용하지만 cooldown
  state는 runtime instance가 아니라 employee별로 독립 보존한다. 같은 타일의 facing-only 재정렬도 같은
  cost/cooldown을 사용하고 거리별 비용은 없다.
- 공통 cost와 base cooldown은 `run/policy.ron`이 소유한다. `DeployCostReduction`은 전술 순간이동에
  적용하지 않는다. 향후 modifier가 필요하면 relocation 전용 효과로 명시한다.
- cooldown은 authoritative battle time을 사용하고 Pause 중 흐르지 않으며 reconnect/resync에서
  `ready_at_ms`를 보존한다. Pause 중 command 접수 여부는 15번에서 확정한다.
- 초기 live cost는 전술 포인트 `5`, 직원별 base cooldown은 `10,000ms`로 확정한다. 두 값은
  `run/policy.ron`에서 조정 가능해야 하며 gameplay 의미를 바꾸지 않는 balance tuning 값이다.

현재 구현된 기반:

- `src/game/behavior.rs`에는 battle authority가 필요한 기존 `DeployUnit`/`WithdrawUnit`/
  `ActivateSkill`/playback action, deployment DTO와 typed payload eligibility가 있다. 새 relocation도
  같은 command/action/projection 경계에 추가할 수 있다.
- `src/game/world.rs`와 `src/game/world/combat.rs`에는 mutation 전 battle UUID/revision preflight,
  비용·placement 검증 뒤 `BattleCore` command를 적용하고 semantic event, checkpoint와 revision을 한
  번 commit하는 transaction 패턴이 있다.
- `src/game/world/state.rs::LiveBattleDeploymentState`는 battle-local current/max 자원, employee와
  runtime unit mapping, battle-time `ready_at_ms`를 쓰는 redeploy lock과 checkpoint projection을 이미
  소유한다. 직원별 relocation cooldown은 없지만 시간 authority와 projection 선례는 존재한다.
- `src/game/battle/core/types.rs::RuntimeUnit`은 continuous world position, projected tile, facing과
  `set_world_position` primitive를 가진다. HP, buff, resonance, runtime identity를 새 instance 없이
  유지하며 위치만 바꿀 물리적 기반이 있다.
- `src/game/battle/core/movement/blocking.rs`는 stable unit ordering으로 sticky engagement를 유지하고
  blocker state를 다시 구성한다. Relocation 전 명시 해제 seam은 없지만 새 위치에서 deterministic하게
  다시 저지할 계산 기반은 있다.
- skill `GroundFixed` area는 생성 위치를 보존하고 `FollowCaster`/`FollowTarget`은 runtime에서 현재
  caster/target geometry를 다시 구한다. Skill/basic attack projectile도 homing과 fixed guidance 및
  독립 source snapshot을 보유하므로 확정한 relocation 뒤 delivery 의미와 대체로 맞는다.
- `run/policy.ron`의 live 경제는 `initial 20 / maximum 99 / 초당 1 / 최초 배치 10`, authoritative
  최대 전투 시간은 60초다. Live encounter에는 45초 생존전이 있어 비용과 cooldown의 실제 시간 비율을
  대조할 수 있다.
- game server는 `PlayerBehavior`를 직접 deserialize하고 `BehaviorResult`의 battle update를
  `CommandAccepted` ack 뒤 side message로 전달한다. Unity도 battle authority, deployment checkpoint,
  pending 배치 preview와 위치·방향 선택 interaction을 이미 소비한다.

정책과 일치하는 부분:

- 기존 withdraw/redeploy와 live Core command가 분리되어 있어 relocation을 새 command로 추가하고 같은
  runtime instance를 유지할 수 있다. 기존 철수 path를 조합할 필요가 없다.
- 위치 변경 primitive는 HP, stat, resonance, buff/debuff와 cooldown을 초기화하지 않는다. 이는 확정한
  unit-owned state 보존 의미와 맞는다.
- area/projectile의 tracking 분류는 `GroundFixed` 유지, `FollowCaster` 추적, `FollowTarget` 유지와
  homing/fixed projectile 의미를 이미 구분한다.
- 비용 부족과 stale authority는 현재 deploy/playback handler에서 mutation 전에 거부되며, 성공한
  mutation은 projection revision을 정확히 한 번 증가시키고 command event에 `source_command_id`를
  연결할 수 있다.
- 전술 자원 자연 회복과 existing cooldown 선례는 authoritative battle time을 사용하므로 Pause 중
  진행하지 않고 reconnect projection에서 absolute `ready_at_ms`를 보존하는 정책과 맞는다.

현재 정책과 다른 코드:

- `PlayerBehavior`, `ActionKind`, `BattleLiveCommand`, `BehaviorResult`, battle semantic event와 server
  result mapping에 tactical relocation surface가 전혀 없다. 현재 공식 player battle command에는
  deploy, withdraw, skill, retreat와 playback만 있다.
- `EngagementStance::Defense`, relocation eligibility, exclusive activity와 직원별 relocation cooldown
  state가 없다. `ActionState`는 `Idle | Dead`뿐이고 기존 action lock, pending attack/skill과 Hard CC를
  하나의 `blocks_tactical_relocation` capability로 합성하지 않는다.
- `set_world_position`은 low-level primitive일 뿐이다. Position/facing, deployment mapping, 저지,
  range/target/aura derived state, resource와 cooldown을 하나의 rollback-safe mutation으로 commit하는
  relocation transaction은 없다.
- Sticky engagement refresh는 기존 유효 pair를 거리와 무관하게 먼저 보존한다. 확정 정책처럼 저지
  중 relocation을 허용하려면 source가 맺은 engagement를 mutation 안에서 명시적으로 해제한 뒤 새
  위치에서 표준 ordering으로 재구성해야 한다.
- Core deploy validator는 zone·affinity·static placement를 확인하지만 destination의 다른 active allied
  unit 점유를 검사하지 않는다. `PositionOccupied` error는 존재하나 deploy path에서 사용하지 않는다.
- Checkpoint의 deployment unit은 position/facing, withdraw와 skill readiness만 보유한다. Stance,
  relocation `ready_at_ms`, relocation eligibility와 tactical focus state는 없다.
- `run/policy.ron`과 `RunPolicyData`에는 relocation cost/cooldown field와 validation이 없다.
- Unity `GameActionNames`, `GameCommandBus`, battle gateway, deployment read model, selected-unit action,
  ghost/facing interaction과 presentation event에는 전술 이동 경로가 없다. 현재 local occupancy는 배치
  highlight hint일 뿐 Core 권위가 아니며 canonical도 `move_unit` 요청이 공식 계약에 없다고 명시한다.

필요한 구현 범위:

- Core command/wire에 `TacticalRelocateUnit`, action kind, typed disabled reason, behavior result와
  `UnitTacticallyRelocated` semantic event를 추가한다.
- World transaction에서 현재 authority, source mapping, stance/action/Hard CC capability, zone·affinity·
  valid tile·static obstacle·allied occupancy, resource와 employee cooldown을 모두 먼저 검사한다.
- Battle runtime에 relocation 전 engagement 해제, position/facing 원자 변경, 새 위치의 block/target/range/
  aura·area derived state 재계산과 no-op 거부를 구현한다.
- Employee별 relocation `ready_at_ms`, stance/capability와 relocation eligibility를 checkpoint/resync에
  투영하고 Retreat/전투 종료의 battle-local 수명을 검증한다.
- `run/policy.ron`에 fixed cost `5`와 base cooldown `10,000ms`를 strict field로 추가하고 구 field나
  `DeployCostReduction` fallback 없이 검증한다.
- Game server deserialize, error/result/update mapping과 command ack 뒤 battle update 순서를 갱신한다.
- 외부 Unity의 action 목록, command bus/gateway, read model, selected-unit action, 전술 집중 source/tile/
  facing ghost flow, semantic event presentation과 checkpoint reconcile을 같은 breaking migration에서
  구현하고 두 canonical 문서를 동기화한다.
- Invalid destination/occupancy/stance/activity/Hard CC/resource/cooldown/stale authority의 무변경,
  same-position facing-only 재정렬, unit-owned state 보존, block/area/projectile 연동, Pause/reconnect를
  Core/server/Unity focused test로 고정한다.

난이도 판정:

- 위치 primitive, battle-time authority, resource 산술, semantic event와 revision commit, projectile/area
  tracking은 존재하므로 simulation 구조를 새로 설계하는 `5/5` 범위는 아니다.
- 그러나 확정 정책 자체는 구현되어 있지 않다. 새 command/state/capability/cooldown과 rollback-safe
  spatial mutation을 Core 여러 계층에 만들고 server wire, checkpoint/resync, 실제 Unity interaction과
  canonical까지 완성해야 한다.
- 따라서 현재 tree에서 완성된 vertical slice 비용은 `4/5 — 높음`이다.
- 2번 전술 자원 분리만 먼저 구현되어도 stance, occupancy, activity/cooldown과 Unity 전술 집중이 남아
  현재 등급은 `4/5`로 유지된다.
- 2번과 5·6·7·13·15번의 자원, 점유, 태세/action, Hard CC capability와 Pause 직렬 기반이 먼저
  구현되면 4번 증분 범위는 relocation mutation/event/checkpoint/Unity 연결로 좁아져 `3/5 — 중간`으로
  내려간다. 16번에서 공통 전술 집중 mode를 먼저 완성한 경우에도 Unity/modal 부분은 줄지만 Core와
  wire를 가로지르는 command vertical slice가 남으므로 `3/5`로 본다.

신규 정책의 runtime 변경 가치:

- 4번만 고립해서 구현하면 기존 withdraw/redeploy가 더 단순하고 일관된 위치 변경 수단이므로
  `4/5` 비용에 비해 가치가 약할 수 있다.
- 반면 14번 자리 교대와 17번 현장 작업까지 진행하면 공통 occupancy, placement, 위치 mutation과 전술
  집중 UX의 기반이 되며, HP·buff·cooldown을 보존하는 즉시 재배치는 기존 회수와 구별되는 핵심 전술
  선택이 된다. 이 확장 묶음에서는 신규 runtime을 도입할 가치가 있다.

초기 tuning 판정:

- 비용 `5`는 최초 배치 비용 `10`의 절반이다. Initial `20`으로 두 직원을 먼저 배치하면 즉시 이동할
  잔액이 없고, 이동을 먼저 쓰면 후속 배치가 늦어져 배치와 재정렬 사이에 실제 trade-off가 생긴다.
- 직원별 `10,000ms` cooldown은 45초 생존전과 60초 runtime 상한에서 한 직원의 반복 이동을 제한하되
  기능을 사실상 일회성으로 만들지 않는다. 전역 자원 회복이 초당 1이므로 resource와 per-employee
  cooldown이 서로 다른 spam 제한으로 작동한다.
- 두 값은 확정한 첫 live tuning이며 의미 정책이 아니다. 이후 playtest에서 `run/policy.ron` 값만
  조정할 수 있다.

검증 결과:

- `cargo test -p game_core apply_live_command_deploys_and_withdraws_player_unit`: 1 passed
- `cargo test -p game_core fixed_defense_preserves_existing_block_before_new_route_priority`:
  1 passed
- `cargo test -p game_core live_defense_playback_pause_freezes_server_tick_and_resume_advances`:
  1 passed
- `cargo test -p game_core supplied_stale_battle_authority_rejects_before_playback_mutation`:
  1 passed
- `cargo test -p game_core pending_deployment_range_preview_returns_all_facings_without_mutating_battle`:
  1 passed
- `cargo test -p game_server deserializes_live_battle_requests`: 1 passed
- `cargo test -p game_server stale_battle_authority_emits_error_then_current_resync_without_mutation`:
  1 passed

이 결과는 relocation에 재사용할 기존 command, block, Pause, authority, preview와 server transport seam이
안정적이라는 증거다. 전술 순간이동 자체가 구현됐거나 신규 계약 테스트가 존재한다는 증거가 아니다.

아직 사용자가 확정해야 할 tuning 또는 선택:

- 4번 자체에는 남은 tuning이 없다. 태세 전환 cooldown은 6번에서 `1,500ms`로 확정했으며 자리 교대
  비용과 weapon별 Defense/Rest 수치는 각각 해당 후속 번호에서 별도로 확정한다.

난이도 책정 reference 파일:

- Core command/wire:
  - `src/game/behavior.rs`: `PlayerBehavior`, `ActionKind`, disabled reason, deployment DTO,
    presentation event와 `BehaviorResult` wire surface
  - `src/game/battle/event_log.rs`: authoritative battle semantic event 정의
- Core world transaction/state:
  - `src/game/world.rs`: battle behavior dispatch와 authority preflight 경계
  - `src/game/world/combat.rs`: deploy/withdraw/skill/playback transaction, zone·affinity validation,
    resource 차감과 revision commit 패턴
  - `src/game/world/state.rs`: `LiveBattleDeploymentState`, employee↔runtime mapping, battle-time
    `ready_at_ms`와 checkpoint deployment projection
  - `src/game/world/battle_capability.rs`: action-family capability projection과 command preflight
- Battle runtime:
  - `src/game/battle/core/sim.rs`: `BattleLiveCommand`, manual action eligibility와 live command 적용 경계
  - `src/game/battle/core/types.rs`: `RuntimeUnit`, `ActionLocks`, body position/facing과
    `set_world_position`
  - `src/game/battle/core/movement/blocking.rs`: sticky engagement, explicit release 시 target 정리와
    deterministic blocker 재구성
  - `src/game/battle/core/basic_attack.rs`: windup/action lock과 예약된 attack resolve
  - `src/game/battle/core/skill_runtime/area.rs`: `GroundFixed`/`FollowCaster`/`FollowTarget` geometry 추적
  - `src/game/battle/core/skill_runtime/projectile.rs` 및
    `src/game/battle/core/basic_attack_projectile.rs`: homing/fixed projectile의 runtime 위치 추적
- Policy/live data:
  - `src/game/data/run_policy_data.rs`: 현재 live deployment policy schema와 validation
  - `../game_resources/data/run/policy.ron`: `initial 20 / max 99 / 초당 1 / 최초 배치 10`, 최대 전투
    시간 60초의 live authority
  - `../game_resources/data/pve/encounters.ron`: 현재 wave timing과 45초 생존 encounter
- Server transport:
  - `../game_server/src/game/player_game_actor/messages.rs`: `PlayerBehavior` JSON command와 battle
    authority transport
  - `../game_server/src/game/player_game_actor/state.rs`: `BehaviorResult`에서 battle update를 추출하는
    server response 경계
- 외부 Unity runtime/contract:
  - `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameActionNames.cs`: 허용 action과 battle
    mutation action 목록
  - `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameCommandBus.cs`: 실제 command JSON 생성
  - `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/Reading/DeploymentModel.cs`: revisioned
    deployment/deployed-unit authority reader
  - `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Battlefield/BattlefieldDeploymentModel.cs`:
    deployment cell, affinity와 Unity-local allied occupancy preview
  - `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Views/BattleActionHandler.cs`,
    `IBattleCommandGateway.cs`, `GameCommandBusBattleGateway.cs`: tile/facing 선택과 battle command dispatch
  - `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Presentation/BattleScreenComposer.cs`: 선택된
    deployed unit의 action/capability 표시
  - `/mnt/f/unity projects/ark/Assets/Tests/EditMode/ArkClientContractTests.cs`: command, read model,
    occupancy와 selected-unit UI 계약 회귀 테스트
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md` 및
    `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`: 외부 Unity canonical

### 5. 아군 타일 점유와 배치 검사 — 정책 확정 / 난이도 2/5

확정일: 2026-08-05  
난이도 확정일: 2026-08-05  
구현 난이도: `2/5 — 낮음`  
4번·14번 선행 구현 후 증분 난이도: `1/5 — 매우 낮음`

확정 정책:

- 점유 단위는 continuous world collider가 아니라 Core의 논리적 `Position` tactical tile이다.
- BattleCore의 runtime과 position model은 한 tactical tile에 여러 unit이 존재할 수 있는 다중 점유
  구조를 유지한다. 1타일 1아군은 전역 simulation 불변식이 아니라 플레이어가 내리는 배치·재배치·전술
  이동 command의 목적지 선택 규칙이다. 해당 command가 목적지를 확정할 때 직원뿐 아니라 방어 목표처럼
  active Player-side actor가 있는 타일도 점유된 것으로 본다.
- 여기서 빈 타일은 다른 active player-side allied unit이 tactical slot을 점유하지 않았다는 뜻일 뿐,
  배치 가능성을 뜻하지 않는다. 모든 목적지는 player deployment zone, 해당 유닛의 현재 weapon
  deployment affinity, battlefield valid tile, static obstacle과 command eligibility 검사를 별도로 모두
  통과해야 한다.
- 적 유닛은 아군 tactical slot을 점유하지 않는다. 적의 projected tile과 아군 tile이 겹치는 것은
  허용하며 이후 저지·공격·공간 판정은 기존 authoritative battle runtime에 맡긴다.
- 최초 배치와 자발적 철수·전투불능 뒤 재배치는 빈 tile에만 가능하다. 다른 active allied unit이
  점유하면 `position_occupied`로 거부한다.
- 플레이어에게는 순간이동과 자리 교대의 독립 버튼을 두지 않고 `전술 이동` 기능 하나만 제공한다.
  Source를 고른 뒤 target tile이 비어 있으면 일반 전술 순간이동, 자리 교대 가능한 아군이 점유하면
  자리 교대 preview로 분기한다.
- 점유 아군이 태세, 진행 중 행동, Hard CC, cooldown, affinity 또는 placement 조건을 만족하지 못하면
  target 선택을 거부하되 전술 집중 모드는 유지한다.
- 일반 이동은 source facing 하나, 자리 교대는 source와 target 양쪽 facing을 선택한 뒤 확정한다. 내부
  wire와 transaction은 `RelocateUnit`과 `SwapUnitPositions`로 분리하지만 사용자에게는 하나의 전술 이동
  기능이다.
- 빈 tile relocation preview 뒤 최종 확정 전에 다른 아군이 그 tile을 점유하면 최신 authority의
  relocation command를 `position_occupied`로 거부하고 전술 집중 모드를 유지한다. 교대 대상 선택과
  양쪽 facing이 없는 command를 Server가 자동 swap으로 바꾸지 않는다.
- 자리 교대는 source가 target의 현재 tile에, target이 source의 현재 tile에 배치 가능한지 각각 전체
  placement 검사를 수행한다. 둘 중 하나라도 실패하면 전체를 거부한다. 성공 시 두 logical/world
  position과 facing을 하나의 mutation에서 교환하며 중간 중복 점유 상태를 만들지 않는다.
- 같은 source의 현재 tile은 자기 자신 때문에 점유된 것으로 보지 않는다. 따라서 4번의 같은 tile
  facing-only 재정렬은 다른 전체 조건을 만족하면 허용한다.
- Command는 최신 battle authority에서 즉시 검증하므로 source/destination reservation, pending occupancy,
  client-local command batch를 만들지 않는다. Unity의 occupancy 표시와 분기 preview는 편의 기능이며
  최종 권위는 Core다.
- 아군 보행, pathfinding, 이동 segment와 이동 중 state는 만들지 않는다. 현재 아군 knockback·강제
  이동도 도입하지 않는다. 향후 system-driven spawn·강제 이동이 추가될 경우의 동타일 중첩은 그 기능의
  별도 정책으로 결정하며, 5번이 BattleCore의 다중 점유 능력을 금지하지 않는다.
- 배치·재배치·전술 이동 command는 중복 목적지를 자동으로 밀어내거나 compatibility state로 만들지
  않는다. 이 command 경로가 이미 점유된 목적지에 새 아군을 놓는 것만 거부한다.
- 향후 공중 소환물처럼 tactical slot을 점유하지 않아야 하는 실제 아군 content가 생기면 그때 명시적인
  occupancy capability를 검토한다. 현재 존재하지 않는 예외를 위해 첫 버전에 범용 field나 fallback을
  추가하지 않는다.
- 이 항목에는 balance tuning 수치가 없다.

현재 구현된 기반:

- `src/game/behavior.rs`에는 `GameError::PositionOccupied`와 기존 배치·방향별 range preview command/DTO가
  이미 있다. `game_server`도 이 오류를 `position_occupied`로 변환하므로 새 error wire shape는 필요하지
  않다.
- active unit의 실제 위치 authority는 `BattleCore.units[*].body`이며 `projected_tile()`로 논리적
  `Position`을 얻을 수 있다. `src/game/world/state.rs`의 checkpoint는 직원뿐 아니라 방어 목표를 포함한
  모든 active runtime unit의 `owner`, `role`, `position`을 이미 투영한다.
- `src/game/world/combat.rs::validate_employee_deployment_cell`은 player deployment zone과 직원의 현재
  weapon deployment affinity를 검사한다. 실제 Core spawn은
  `src/game/battle/core/build.rs::place_scenario_spawn`에서 valid tile과 static obstacle을 다시 검사한다.
- battlefield template validation은 deployment cell이 valid tile이어야 하고 obstacle 또는 spawn zone과
  겹치지 않도록 보장한다. 따라서 현재 live data의 빈 배치 타일에 대한 zone/valid/static 기반은 이미
  존재한다.
- 외부 Unity `BattlefieldDeploymentModel`은 checkpoint deployment의 직원 위치를 로컬 occupied cell로
  만들어 배치 highlight에서 제외한다. `BattleActionHandler`는 최종 방향 선택 전에 Core range preview를
  요청하고 Core 오류를 표시하는 preflight 흐름을 이미 가진다.
- 외부 Unity canonical 두 문서는 아군 deployment cell 중복 금지와 `position_occupied` 오류를 이미
  명시한다.

정책과 일치하는 부분:

- 점유를 물리 collider가 아니라 runtime world position에서 파생한 tactical tile로 판단할 수 있는 source가
  있다. Rapier movement는 unit collider를 이동 보정 장애물로 쓰지 않으므로 이 규칙을 player tactical
  slot에만 한정해도 기존 continuous combat overlap과 충돌하지 않는다.
- 배치 zone, affinity, valid tile과 static obstacle은 서로 다른 검사 의미로 이미 분리돼 있다.
  `StaticObstacleBlocked`와 `PositionOccupied`도 Core/server에서 별도 오류다.
- checkpoint에 모든 active actor의 owner와 position이 있으므로 방어 목표까지 포함한 Unity hint를 새 DTO
  없이 만들 수 있다. 최종 command authority를 Core에 두는 현재 transport 원칙도 확정 정책과 같다.
- 적은 player deployment state에 들어가지 않고 continuous movement가 unit collider overlap을 허용하므로,
  적과 projected tile이 같다는 이유로 player tactical slot을 막지 않는 정책을 유지할 수 있다.

정책과 다른 부분:

- 실제 gameplay code에는 `GameError::PositionOccupied`를 생산하는 call site가 없다. 현재 배치 preview와
  deploy path 모두 다른 active Player-side runtime unit의 projected tile을 검사하지 않는다.
- 기존 `live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock` test는 두 직원을 같은 위치에
  배치하는 성공을 명시적으로 기대한다. 즉 “기반은 존재하지만 확정 정책은 미구현”인 정도를 넘어 현재
  회귀 계약이 확정 정책과 반대다.
- `LiveBattleDeploymentState.deployed_units`는 employee UUID에서 instance로만 매핑하며 position index나
  전 Player-side actor 목록을 소유하지 않는다. 이 구조만 조회하면 방어 목표 점유를 놓친다.
- `BattleCore::spawn_scenario_group`과 `build_runtime_field`는 valid/static obstacle만 검사하고 모든
  side의 동타일 unit을 허용한다. 이 다중 점유 runtime은 유지한다. 다만 live deploy도 scenario group을
  먼저 추가한 뒤 spawn하므로 player command의 새 점유 검사는 mutation 전 preflight에 두어 실패한
  command가 내부 scenario/event/resource/revision을 남기지 않게 해야 한다.
- 현재 live `game_resources/data/map/battlefield_templates.ron`에서 Defense route의 끝 `P` marker는 실제
  ground deployment cell이다. 방어 목표는 그 route 끝에 Player-side actor로 생성되므로 정책 적용 뒤
  현재 각 Defense 맵의 해당 배치 타일 하나는 의도적으로 사용할 수 없게 된다.
- Unity `BattlefieldDeploymentModel`의 occupied set은
  `checkpoint.deployment.deployed_units`만 읽는다. 직원끼리의 로컬 중복은 숨기지만
  `checkpoint.units`에만 나타나는 방어 목표는 놓친다.
- Unity의 `BattleActionHandler_OccupiedDropUsesCorePreviewRejection` test는 fake gateway가
  `position_occupied`를 반환하는 경우만 검증한다. 현재 실제 Core가 그 오류를 반환한다는 증거는 아니다.
- 전술 순간이동과 자리 교대 command/UI는 아직 없으므로, 빈 목적지와 교대 가능 아군을 하나의 `전술 이동`
  UX에서 분기하는 부분은 4번·14번 구현 범위에 남아 있다.

필요한 구현 범위:

- player 배치·재배치·전술 이동 command가 사용할 수 있도록 active `Side::Player` unit의
  `body.projected_tile()`에서 목적지 점유자를 파생하는 공통 query와 destination validator를 둔다. 별도
  authoritative position index나 battlefield single-occupant storage는 만들지 않는다.
- validator는 제외할 instance 집합을 받아 최초/재배치는 제외 없음, 같은 tile facing-only 이동은 source
  제외, 자리 교대는 두 source를 제외한 뒤 양쪽 목적지를 검사할 수 있게 한다.
- live deploy와 range preview가 같은 validator를 사용하게 하고, 점유 실패를 Core mutation 전에
  `PositionOccupied`로 반환한다. 실패 시 tactical resource, instance salt, scenario group, event log와
  battle revision이 모두 그대로인지 검증한다.
- scenario/system spawn의 전역 동타일 제한은 추가하지 않는다. player command preflight가 통과한 뒤 기존
  spawn primitive를 호출하며, Opponent 및 system-driven runtime overlap 의미는 그대로 둔다.
- 기존 player command의 동타일 배치 성공 test를 거부·무변경 test로 바꾸고, 직원-직원, 방어 목표-직원,
  inactive withdrawn/dead unit 무시, 적 overlap 허용과 valid/static/affinity 선행 검사를 focused test로
  고정한다. 별도의 low-level runtime test에서는 다중 점유 자체가 계속 가능함을 보존한다.
- Unity occupied hint는 runtime checkpoint의 active `owner == Player` unit position을 사용하도록 보강하되,
  employee 배치 여부는 계속 deployment DTO에서 읽는다. 방어 목표 tile 제외와 stale hint 뒤 Core 거부를
  contract test로 추가한다.
- server는 기존 error mapping과 command envelope을 재사용하므로 DTO 변경은 없다. 실제 Core
  `position_occupied` 전달과 거부 시 update/revision 무변경 test만 보강하면 된다.
- live RON schema나 balance 값은 변경하지 않는다. 방어 목표가 차지하는 기존 `P` 한 칸을 유지하는 것이
  확정 정책의 직접 결과이며, 이를 피하려고 목표를 자동 이동시키거나 예외 처리하지 않는다.

신규 정책의 runtime 변경 가치:

- 유지할 가치가 있다. 배치와 전술 이동은 플레이어가 명시적으로 tile을 선택하는 기능이므로, 그 입력이
  같은 아군 slot에 새 unit을 겹치게 만들면 선택·방향·교대 대상의 의미가 모호해진다. 4번 순간이동과
  14번 자리 교대의 결정론적 destination 검증에도 이 command-level 규칙이 필요하다.
- 기존 continuous/dynamic overlap 구조 자체는 더 단순하고 전투 이동에 적합하므로 되돌리지 않는다.
  player-side tactical command 입력에만 파생 검사를 추가하면 적 이동, system spawn, 저지, 공격과
  physics 의미를 그대로 보존하면서 필요한 UX 규칙만 얻는다.

난이도 판정:

- 현재 tree 기준 제안은 `2/5 — 낮음`이다. runtime 위치·owner source, valid/static/affinity 검사,
  `PositionOccupied` server mapping, checkpoint의 전 Player-side actor projection과 Unity occupancy model이
  모두 존재한다.
- 핵심 변경은 새 state나 wire DTO가 아니라 공통 derived query/preflight, 기존 deploy/preview 연결과
  focused test다. Core world/runtime과 Unity를 함께 만지지만 각 seam이 좁고 data migration도 없다.
- 다만 기존 test가 동타일 배치 성공을 의도적으로 고정하므로 단순 문서 수정인 `1/5`는 아니다. command
  실패의 완전한 무변경성과 방어 목표 점유까지 회귀 테스트해야 한다.
- 4번 전술 순간이동과 14번 자리 교대가 먼저 공통 destination validator와 Unity의 전술 이동 분기를
  완성했다면 5번의 증분 난이도는 `1/5 — 매우 낮음`까지 내려갈 수 있다. 그 경우 최초/재배치 path와
  방어 목표 Unity hint에 같은 helper를 연결하고 기존 stacking test를 교체하는 일이 중심이다.
- 자연스러운 구현 순서는 반대로 5번 validator를 먼저 만드는 것이다. 그러면 4번·14번은 같은 source of
  truth를 재사용할 수 있지만, 그 절감분을 현재 5번 난이도에서 미리 차감하지 않는다.

난이도 책정 reference 파일:

- Core command/wire
  - `core_enhanced/src/game/behavior.rs`: 기존 deploy/range-preview command와 deployment/checkpoint DTO,
    `GameError::PositionOccupied` 정의를 소유한다.
- Core world transaction/state
  - `core_enhanced/src/game/world/combat.rs`: zone·affinity·eligibility preflight, range preview와 deploy의
    resource/instance/revision commit 순서를 소유한다.
  - `core_enhanced/src/game/world/state.rs`: employee deployment mapping과 모든 active runtime actor의
    owner/role/projected position checkpoint를 소유한다.
- Battle runtime
  - `core_enhanced/src/game/battle/core/build.rs`: 다중 점유를 허용하는 공통 scenario/player spawn과
    valid/static placement를 수행하며, player deploy command가 preflight 뒤 호출하는 실제 mutation
    경로다.
  - `core_enhanced/src/game/battle/battlefield/field.rs`: valid tile과 static obstacle 검사를 서로 다른
    오류로 제공한다.
  - `core_enhanced/src/game/battle/core/movement/rapier_backend.rs`: unit collider overlap을 이동 장애물로
    취급하지 않는 continuous movement 정책과 그 test를 소유한다.
- Policy/live data
  - `core_enhanced/src/game/combat_preview/template.rs`: `P`를 ground deployment cell과 route endpoint
    marker로 해석하는 template parser를 소유한다.
  - `core_enhanced/src/game/combat_preview/validation.rs`: deployment zone의 valid/obstacle/spawn-zone
    교차 불변식과 Defense route endpoint 검증을 소유한다.
  - `core_enhanced/src/game/combat_setup/defense_object.rs`: 보호 대상을 route 마지막 cell에
    `Side::Player` actor로 생성한다.
  - `game_resources/data/map/battlefield_templates.ron`: 현재 Defense route 끝 `P`와 실제 ground deployment
    cell 중첩을 authoring한 live battlefield data다.
- Game server transport
  - `game_server/src/game/player_game_actor/messages.rs`: 기존 live deploy/range-preview behavior를 그대로
    deserialize하는 command envelope을 소유한다.
  - `game_server/src/game/player_game_actor/state.rs`: `PositionOccupied`를 Unity-facing
    `position_occupied`로 변환한다.
- 외부 Unity runtime/contract/tests
  - `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Battlefield/BattlefieldDeploymentModel.cs`:
    affinity별 deployment highlight와 현재 직원-only occupied cell hint를 계산한다.
  - `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Views/BattleActionHandler.cs`: tile/facing pending
    배치와 Core range-preview preflight 오류 처리를 소유한다.
  - `/mnt/f/unity projects/ark/Assets/Tests/EditMode/ArkClientContractTests.cs`: affinity/직원 점유 highlight와
    fake `position_occupied` preflight 동작을 검증한다.
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`: 아군 deployment cell 중복 금지와
    `position_occupied` 외부 계약을 명시한다.
  - `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`: deploy command 거부, checkpoint
    actor authority와 pending placement가 transport/event 흐름에 결합되는 방식을 명시한다.

focused test 결과:

- `cargo test -p game_core live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock`:
  1 passed. 다중 점유 runtime 자체가 아니라 현재 player deploy command도 동타일 아군 배치를 허용하고
  있음을 고정한 test이므로, 확정한 command 입력 정책과 충돌한다.
- `cargo test -p game_core pending_deployment_range_preview_returns_all_facings_without_mutating_battle`:
  1 passed. 현재 preview의 무변경성과 방향별 범위 기반만 검증하며 점유 거부 구현 증거는 아니다.
- `cargo test -p game_core defense_node_type_builds_default_black_box_defense_objective`:
  1 passed. 방어 목표가 route 끝에 생성되는 기존 기반을 확인한다.
- `cargo test -p game_core battlefield_template_exposes_ground_and_platform_deployment_zones`:
  1 passed. live template의 두 deployment kind 파싱 기반을 확인한다.
- `cargo test -p game_core static_obstacles_block_placement`: 1 passed.
- `cargo test -p game_core non_rectangular_valid_tiles_block_void_placement`: 1 passed.
- `cargo test -p game_core rapier_engine_ignores_unit_colliders_when_correcting_translation`:
  1 passed. continuous movement overlap 기반을 확인하며 tactical occupancy 구현 증거는 아니다.
- `cargo test -p game_server deserializes_live_battle_requests`: 1 passed. 기존 deploy/preview transport가
  안정적이라는 증거이며 신규 점유 검증 구현 증거는 아니다.
- 외부 Unity test runner는 이번 조사에서 실행하지 않았다. 위 Unity test source는 읽었지만 fake
  `position_occupied` 응답만 사용하므로 실제 Core 연동 증거로 세지 않는다.

난이도·tuning 확정 결과:

- 현재 tree 기준 난이도는 `2/5 — 낮음`으로 확정했다.
- 4번·14번이 공통 destination validator와 Unity 전술 이동 분기를 먼저 구현한 뒤의 증분 난이도는
  `1/5 — 매우 낮음`으로 확정했다.
- 이 항목에는 balance tuning 수치가 없다.

### 6. 공격/방어 태세 — 정책·코드 대조·난이도·tuning 확정 완료

확정일: 2026-08-05  
구현 난이도: `4/5 — 높음`  
선행 정책 구현 후 증분 난이도: 4번의 공통 stance/checkpoint/Unity seam이 실제 구현된 경우
`3/5 — 중간`  
초기 tuning: 직원별 대칭 전환 cooldown `1,500ms`

확정 정책:

- active 배치 직원이 저장하는 authoritative 교전 태세는 `EngagementStance::Attack`과
  `EngagementStance::Defense` 두 값뿐이다. `DefenseFocus`, `Rest`, `TacticalReady`를 별도 stance나
  boolean state로 저장하지 않는다.
- `Attack`은 현재 자동 기본 공격, automatic/manual skill 시작, 자동 targeting과 저지 규칙을 각 기능의
  기존 eligibility에 따라 허용한다. `Attack` 자체가 공격이나 skill을 강제로 시작하지는 않는다.
- `Defense`는 새로운 기본 공격과 automatic/manual skill 시작을 모두 막고 전술 순간이동과 자리 교대의
  준비 태세가 된다. 공격/지원/self skill을 분류해 Defense에서만 허용하는 예외는 첫 버전에 두지 않는다.
- `Defense`의 실제 효과는 저장 태세를 늘리지 않고 `현재 deployment tile kind + 장착 weapon RON
  profile`에서 파생한다. Ground에서는 해당 weapon의 `DefenseFocus`, Platform에서는 해당 weapon의
  `Rest` 하나만 활성화한다.
- weapon RON은 `deployment_affinity`와 사용할 defense profile의 존재·수치를 직접 소유한다.
  `GroundOnly`는 `DefenseFocus` 필수·`Rest` 없음, `PlatformOnly`는 `Rest` 필수·`DefenseFocus` 없음,
  `Any`는 두 profile 모두 필수로 strict validation한다.
- profile 안의 수치 `0`은 해당 효과를 만들지 않는다는 명시적 authoring이다. 필수 profile 자체의 누락은
  invalid live data다. Weapon archetype, 직원 기초 stat 또는 현재/effective `block_capacity`에서 profile
  종류나 존재를 추론하지 않는다.
- 태세 전환은 전술 포인트를 소비하지 않는다. 성공한 `Attack -> Defense`와 `Defense -> Attack`은 모두
  같은 직원별 battle-time `stance_ready_at_ms` cooldown을 시작하고, ready 전에는 다음 태세 전환을
  거부한다.
- stance cooldown은 태세 전환만 제한한다. `Attack -> Defense` 성공 뒤 다른 action/activity 조건을 모두
  만족하면 cooldown 만료를 기다리지 않고 전술 순간이동과 자리 교대를 사용할 수 있다.
- 같은 현재 태세를 다시 요청하면 `already_active` 의미로 거부하며 stance, cooldown, event, resource와
  battle revision을 변경하지 않는다.
- authoritative transition phase, `desired_stance`, 예약 태세, 자동 복귀와 공격 취소 token은 만들지
  않는다. 성공한 command는 태세와 새 tile/weapon 파생 효과를 같은 mutation에서 즉시 바꾼다.
- 최초 배치와 자발적 철수·전투불능 뒤 재배치는 `Attack`으로 시작한다. 전술 순간이동은 기존
  `Defense`를 유지한다. Work cell 직접 배치와 `FieldWorking`의 시작·종료 태세 의미는 17번에서
  최종 검토한다.
- 전환 전에 이미 시작된 기본 공격·skill의 완료 여부는 7번, Hard CC와 Silence의 전환·효과 제한은
  13번, Ground `DefenseFocus`와 Platform `Rest`의 구체 효과와 수치는 8~10번 및 12번에서 각각
  검토한다.
- stance cooldown은 authoritative battle time을 사용한다. Pause 중 시간 진행 여부와 직렬 command
  의미는 15번 정책을 따른다.
- 이 단계에서는 직원별 공통 stance cooldown의 의미만 확정하며 실제 milliseconds tuning은 code/live
  전투 시간과 전술 이동·공격 주기를 대조한 뒤 정한다.

현재 구현된 기반:

- Core의 live battle command는 `PlayerBehavior -> GameCore world transaction -> BattleLiveCommand ->
  BehaviorResult/battle_update` 경로로 연결되어 있다. 성공 mutation은 battle projection revision을 한 번
  commit하고 `source_command_id`를 semantic event에 연결할 수 있다.
- `RuntimeUnit`은 lifecycle, action lock, 기본 공격 준비 시각, pending 기본 공격, skill activation mode,
  pending cast, 공명과 저지 수를 이미 보유한다. 태세와 태세 전환 cooldown은 없지만 새 시작을 판정할
  런타임 정보는 한곳에 모여 있다.
- 새 기본 공격은 `try_start_pending_basic_attacks`와 `handle_basic_attack_start_event`, automatic skill은
  `can_start_autocast`와 `schedule_pending_autocast_for`, manual skill은
  `validate_manual_skill_activation`에서 각각 검증한다. 이미 commit된 공격 resolve와 진행 중 skill cast는
  시작 경로와 분리되어 있어 7번의 "새 시작만 막고 시작된 행동은 완료"를 구현할 seam이 있다.
- FixedDefense 저지는 active Player combatant의 `block_capacity`를 읽어 `refresh_block_state`에서
  재구성한다. 태세·tile profile을 반영하지는 않지만 effective block capacity를 연결할 재계산 경계는
  존재한다.
- Live checkpoint는 active runtime unit과 배치 직원별 withdraw/skill eligibility를 매 update에
  투영한다. Battle context action capability와 payload별 typed disabled reason도 이미 사용한다.
- weapon RON의 `WeaponCombatProfile`은 `deny_unknown_fields`이고 모든 Weapon에 profile을 요구하며
  runtime validation을 수행한다. 새 affinity와 defense profile의 존재 조합을 strict validation할 기반은
  있다.
- Authoritative battle time과 Pause가 이미 있다. Live policy는 50ms movement tick, 60,000ms battle
  상한을 사용하며 Pause 동안 server tick이 진행되지 않는다.
- Game server는 Core의 `PlayerBehavior`를 직접 deserialize하고 live mutation을 `CommandAccepted` 뒤
  revisioned battle update로 전송한다. Unity는 action capability, checkpoint 배치 상태와 선택 직원의
  withdraw/skill eligibility를 이미 소비한다.

정책과 일치하는 부분:

- 현재 공격·skill runtime은 "시작 eligibility"와 "이미 시작된 resolve/cast"를 분리한다. 따라서
  `Defense` 진입 때 공격 취소 token이나 별도 transition phase를 만들 필요가 없다.
- 자동 skill과 수동 skill의 시작 검사가 구분되어 있으면서도 둘 다 Core 권위 경계에 있으므로
  `Defense`에서 두 경로를 같은 의미로 막을 수 있다.
- 기존 deploy는 매번 새 runtime unit을 만들고 기본 공격 state를 초기화한다. 새 `Attack` 기본값을 이
  생성점에 명시하면 최초 배치와 기존 재배치의 정책을 자연스럽게 만족한다.
- cooldown과 checkpoint는 기존 재배치의 absolute battle-time `ready_at_ms`, Pause와 resync 패턴을
  재사용할 수 있다. 전술 포인트를 차감하지 않는 태세 command도 기존 transaction에서 원자적으로
  처리할 수 있다.
- weapon profile의 strict loader/validator는 archetype이나 block 수에서 profile을 추론하지 않는 확정
  방향과 맞는다.

현재 정책과 다른 코드:

- `EngagementStance`, `stance_ready_at_ms`, 태세 변경 command/action/result/event가 Core command,
  world, battle runtime과 wire 어디에도 없다. 현재 `ActionState`도 `Idle | Dead`뿐이며 교전 태세를
  표현하지 않는다.
- 기본 공격, automatic skill과 manual skill 시작 검사는 태세를 보지 않는다. 현재 active combatant는
  다른 lock/CC/target 조건만 맞으면 항상 새 공격·skill을 시작할 수 있다.
- 현재 저지는 태세와 현재 tile kind에 관계없이 raw runtime `block_capacity`가 양수이면 작동한다.
  `DefenseFocus`/`Rest` selector나 active defense profile 개념이 없다.
- `WeaponCombatProfile`에는 `deployment_affinity`, `DefenseFocus`, `Rest`가 없다. 직원 combat profile은
  기본 `GroundOnly`이고 `apply_weapon_profile`도 weapon에서 affinity를 복사하지 않으므로 현재 live
  weapon이 배치 적성을 직접 소유한다는 확정 정책과 다르다.
- `RunPolicyData`와 `run/policy.ron`에는 stance cooldown이 없다. 현재 weapon live data는 명시적인
  기본 공격 interval/windup과 block 수만 보유한다.
- Checkpoint unit/deployed-unit DTO에는 stance, `stance_ready_at_ms`, 다음 태세 전환 eligibility가 없고,
  action capability에도 태세 변경 family가 없다.
- Game server의 generic command envelope는 재사용할 수 있지만 새 `BehaviorResult`를 battle update로
  추출하는 exhaustive match와 command deserialize contract test를 갱신해야 한다.
- 외부 Unity canonical은 공식 live command를 deploy/withdraw/skill/playback 중심으로만 정의하며
  stance command/event/checkpoint shape가 없다. Runtime도 action 이름, command bus/gateway, read model,
  선택 유닛 태세 표시·전환 버튼과 event reconcile을 구현하지 않았다.

필요한 구현 범위:

- Core shared type에 `EngagementStance::{Attack, Defense}`를 추가하고 `SetEngagementStance`
  `PlayerBehavior`/`ActionKind`, battle authority 요구, typed `stance_cooldown`·태세 관련 disabled reason,
  result와 `UnitEngagementStanceChanged` semantic event를 정의한다.
- `RuntimeUnit`에 현재 stance와 absolute `stance_ready_at_ms`를 추가한다. Player employee deploy 생성점은
  명시적으로 `Attack`으로 초기화하고, same-stance no-op와 cooldown을 mutation 전에 검증한 뒤 성공 시
  stance와 ready time을 한 번에 변경한다.
- 새 기본 공격의 두 시작 진입점, automatic skill scheduling과 manual skill activation 검증에
  `Attack` gate를 넣는다. 진행 중 resolve/cast에는 이 gate를 다시 적용하지 않아야 하며 세부 interrupt
  의미는 7번 test에서 고정한다.
- 현재 tile kind와 weapon profile에서 active defense profile kind를 계산하는 단일 helper를 둔다.
  `deployment_affinity`와 profile 존재 조합의 RON schema/strict validation은 추가하되, 구체 회복·저지
  수치 field와 effect 적용은 8~10번·12번 확정과 함께 한 번에 migration한다. 의미가 정해지지 않은
  범용 effect bag이나 archetype fallback은 만들지 않는다.
- 태세 변경 뒤 effective block/target/skill readiness와 향후 defense effect가 같은 authoritative
  projection에서 다시 계산되도록 derived-state refresh 경계를 연결한다.
- Checkpoint에 stance, `stance_ready_at_ms`와 직원별 다음 전환 eligibility를 투영하고 reconnect/resync,
  Pause, same-stance 거부와 revision 무변경을 검증한다. 전역 action capability는 command family 존재를,
  직원별 payload eligibility는 실제 전환 가능 여부를 소유하게 한다.
- `run/policy.ron`에 하나의 공통 대칭 stance cooldown을 strict field로 추가한다. Weapon별 cooldown이나
  Attack/Defense 방향별 중복 field는 만들지 않는다.
- Game server의 request deserialize, result/update mapping과 ack/update 순서를 갱신한다.
- 외부 Unity 두 canonical 문서와 `GameActionNames`, command bus/gateway, deployment/checkpoint read model,
  selected-unit status/action, handler와 contract test를 같은 breaking migration에서 갱신한다. Unity는
  cooldown을 로컬로 계산해 권위를 만들지 않고 checkpoint eligibility와 battle time을 표시 source로
  사용한다.

신규 정책의 runtime 변경 가치:

- 도입 가치가 있다. 4번 전술 순간이동과 14번 자리 교대가 요구하는 준비 조건을 scattered
  `can_move_tactically` boolean이나 command별 공격 중지 부작용으로 복제하지 않고 하나의 명시적 상태로
  통합한다.
- Ground/Platform 효과도 `Attack/Defense` 두 상태에 tile+weapon 파생 profile만 얹으므로 저장 상태를
  늘리지 않는다. 현재 runtime을 유지하면 relocation 때 공격을 어떻게 멈추고 언제 다시 허용하는지
  command마다 암묵적으로 달라져 장기적으로 더 복잡해진다.
- 반대로 `DefenseFocus`, `Rest`, `TacticalReady`를 각각 저장 state/command로 추가하는 확장은 가치가
  없다. 확정한 2상태+파생 profile이 구현 범위와 wire 상태를 가장 작게 유지한다.

난이도 판정:

- 태세 값과 cooldown 비교 자체는 작고 공격·skill 시작, strict RON, battle-time checkpoint seam도
  존재한다. 핵심 simulation 구조나 광범위한 새 데이터 모델이 필요한 `5/5`는 아니다.
- 그러나 현재 tree에는 확정 정책이 구현되어 있지 않다. 새 command/state/event를 Core 여러 계층에
  추가하고 공격·automatic/manual skill의 서로 다른 시작 경로, checkpoint/capability, server와 실제
  Unity 선택 유닛 UX까지 완성해야 한다.
- 따라서 현재 tree에서 완성된 vertical slice의 난이도는 `4/5 — 높음`으로 확정했다.
- 4번 구현 과정에서 공통 `EngagementStance`, 직원별 capability/checkpoint와 Unity 전술 집중/선택 유닛
  seam까지 먼저 실제로 완성한 경우 6번의 증분 범위는 태세 전환 command, 세 공격·skill gate와
  weapon profile selector 중심이 되어 `3/5 — 중간`으로 내려간다. 4번이 단지 `Defense`를 미래 조건으로
  참조하는 문서나 placeholder만 둔 경우에는 기반 구현으로 세지 않으므로 여전히 `4/5`다.
- 8~10번의 concrete profile schema/effect까지 함께 구현하는 묶음에서는 RON migration을 한 번만 할 수
  있지만, 6번의 독립 command/server/Unity vertical slice가 남으므로 현재 등급 자체는 `4/5`로 본다.

초기 tuning 판정:

- live player weapon의 authored 기본 공격 interval은 주로 `1,350~1,800ms`, windup은
  `160~280ms`다. Weapon/accessory의 영구 감소를 함께 적용하면 빠른 조합은 약 `1,070ms`까지 내려갈 수
  있고, live 침식 직원은 `1,200~1,900ms`, 환상체는 주로 `1,750~2,150ms`를 사용한다.
- `1,000ms` 이하는 빠른 공격 한 주기보다 짧아 태세 commitment보다 입력 debounce에 가까워진다.
  `1,500ms`는 대부분의 실제 공격 주기와 같은 규모이고, relocation 직후 Attack 복귀 때 대략 한 번의
  공격 기회를 포기하게 하면서 60초 전투에서 태세를 사실상 장기 고정하지는 않는다.
- 따라서 첫 live 공통값으로 직원별 대칭 cooldown `1,500ms`를 확정했다. 이는 의미 정책이 아니라
  `run/policy.ron`에서 조정할 balance tuning이다. 전술 순간이동 cooldown `10,000ms`와 독립하며,
  Defense 진입 직후 relocation 자체를 기다리게 하지 않는다.

검증 결과:

- `cargo test -p game_core pending_basic_attack_retargets_when_persisted_target_is_out_of_range`:
  1 passed
- `cargo test -p game_core silence_blocks_new_casts_but_does_not_interrupt_pending_casts`:
  1 passed
- `cargo test -p game_core fixed_defense_block_capacity_prioritizes_route_progress_and_allows_excess_to_pass`:
  1 passed
- `cargo test -p game_core live_defense_playback_pause_freezes_server_tick_and_resume_advances`:
  1 passed
- `cargo test -p game_core --test ron_loading live_weapon_block_capacity_is_explicit_and_matches_approved_guideline`:
  1 passed
- `cargo test -p game_server deserializes_live_battle_requests`: 1 passed

이 결과는 공격 시작/진행 분리, 저지 재계산, Pause battle time, strict weapon RON과 server transport의
현재 기반이 안정적이라는 증거다. 공격/방어 태세, 새 command와 Unity 계약이 구현됐다는 증거가 아니다.
외부 Unity EditMode test는 이번 read-only 조사에서 실행하지 않았고 관련 기존 contract test를 읽어
필요한 추가 범위만 확인했다.

난이도·tuning 확정 결과:

- 현재 tree 기준 난이도는 `4/5 — 높음`으로 확정했다.
- 4번의 공통 stance/checkpoint/Unity seam이 실제 선행 구현된 경우의 증분 난이도는
  `3/5 — 중간`으로 확정했다. 문서나 placeholder만 존재하는 경우에는 적용하지 않는다.
- 직원별 공통 대칭 stance cooldown 초기값은 `1,500ms`로 확정했다.
- `DefenseFocus`/`Rest`의 구체 field와 weapon별 수치는 6번 tuning이 아니라 8~10번·12번에서 별도로
  확정한다.

난이도 책정 reference 파일:

- Core command/wire:
  - `core_enhanced/src/game/behavior.rs`: 현재 `ActionKind`, typed disabled reason, `PlayerBehavior`, live
    checkpoint/deployment DTO, presentation event와 `BehaviorResult` surface를 소유한다.
  - `core_enhanced/src/game/battle/event_log.rs`: command와 연결할 authoritative battle semantic event를
    정의한다.
- Core world transaction/state:
  - `core_enhanced/src/game/world.rs`: battle behavior dispatch와 battle authority preflight 경계다.
  - `core_enhanced/src/game/world/combat.rs`: deploy/withdraw/manual skill/playback의 선검증, runtime command,
    revision commit과 battle update transaction 패턴을 소유한다.
  - `core_enhanced/src/game/world/state.rs`: battle-time state, employee↔runtime mapping과 매 update checkpoint
    projection을 소유한다.
  - `core_enhanced/src/game/world/battle_capability.rs`: battle command-family capability projection과
    preflight를 소유한다.
- Battle runtime:
  - `core_enhanced/src/game/battle/core/types.rs`: `RuntimeUnit`, lifecycle, action lock, pending 공격/skill과
    raw block capacity state를 소유한다.
  - `core_enhanced/src/game/battle/core/build.rs`: runtime unit 생성과 deploy 시 공격/cooldown 초기화 지점이다.
  - `core_enhanced/src/game/battle/core/basic_attack.rs`: 새 기본 공격 scheduling/start와 commit된 resolve의
    분리 경계다.
  - `core_enhanced/src/game/battle/core/mod.rs`: automatic skill의 pending cast와 새 autocast 시작 gate를
    소유한다.
  - `core_enhanced/src/game/battle/core/sim.rs`: manual skill 검증과 `BattleLiveCommand` 적용 경계다.
  - `core_enhanced/src/game/battle/core/movement/blocking.rs`: active blocker 선정과 block state 재계산을
    소유한다.
- Policy/live data:
  - `core_enhanced/src/game/data/equipment_data.rs`: strict `WeaponCombatProfile` schema와 모든 Weapon profile
    validation을 소유한다.
  - `core_enhanced/src/game/battle/types.rs`: employee 기본 `deployment_affinity`와 weapon profile을 runtime
    combat profile에 적용하는 경계다.
  - `core_enhanced/src/game/data/run_policy_data.rs`: battle runtime/live deployment tuning의 typed policy
    schema이며 stance cooldown field는 아직 없다.
  - `game_resources/data/equipments/base.ron`: live weapon의 block 수, 기본 공격 interval/windup과 영구
    interval modifier를 직접 확인한 source다.
  - `game_resources/data/run/policy.ron`: 50ms tick, 60초 battle 상한과 현재 live deployment 경제를
    authoring한다.
  - `game_resources/data/abnormalities/base.ron`: live 환상체 기본 공격 interval/windup 비교 source다.
  - `game_resources/data/enemies/corroded_employees.ron`: live 침식 직원 기본 공격 interval/windup 비교
    source다.
- Game server transport:
  - `game_server/src/game/player_game_actor/messages.rs`: Core `PlayerBehavior` command envelope deserialize와
    battle update server message 변환을 소유한다.
  - `game_server/src/game/player_game_actor/state.rs`: `BehaviorResult`에서 live battle update를 추출하고
    `CommandAccepted`/side message로 매핑한다.
  - `game_server/src/game/player_game_actor/handlers.rs`: mutation 실행, active-run rollback/persistence와
    실제 side message 방출 순서를 소유한다.
- 외부 Unity runtime/contract/tests:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`: 공식 live command request와 selected Core
    eligibility 계약을 소유하며 현재 stance command가 없다.
  - `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`: revisioned update, semantic event와
    checkpoint reconcile 계약을 소유하며 현재 unit stance shape가 없다.
  - `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameActionNames.cs`: 알려진 Core action과
    battle authority가 필요한 mutation action 목록이다.
  - `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameCommandBus.cs`: live battle command JSON과
    authority envelope 전송 경계다.
  - `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/Reading/DeploymentModel.cs`: checkpoint의
    deployed unit, withdraw와 skill eligibility read model이다.
  - `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Views/IBattleCommandGateway.cs`와
    `GameCommandBusBattleGateway.cs`: battle presentation에서 command bus로 이어지는 interface/adapter다.
  - `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Presentation/BattleScreenComposer.cs`: 선택 직원의
    현재 status, withdraw와 manual skill action을 구성하는 UI 경계다.
  - `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Views/BattleActionHandler.cs`와
    `BodyPanelActionHandler.cs`: 선택 유닛 action의 latest capability 재검사와 실제 dispatch를 소유한다.
  - `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Battlefield/BattleSelectedUnitStatusModel.cs`:
    presented checkpoint에서 선택 직원 상태를 읽는 경계다.
  - `/mnt/f/unity projects/ark/Assets/Tests/EditMode/ArkClientContractTests.cs`: deployment read model과 선택 유닛
    withdraw/skill action의 기존 EditMode contract test를 소유한다.

### 7. 태세 전환 중 행동 처리 — 정책·코드 대조·난이도 확정 완료

확정일: 2026-08-05  
구현 난이도: 현재 tree 기준 `4/5 — 높음`  
선행 정책 구현 후 증분 난이도: 4·6·14번 command/state seam이 구현된 경우 `2/5 — 낮음`

확정 정책:

- 행동 불가 Hard CC 상태가 아니라면 기본 공격 windup 또는 automatic/manual skill focus 중에도 태세를
  즉시 전환할 수 있다.
- 태세와 새 태세의 효과는 command 성공 시점부터 즉시 적용한다. 별도의 전환 시간이나 완료 대기 상태는
  없다.
- 태세 전환 전에 이미 확정된 기본 공격과 skill은 취소하지 않고 끝까지 수행한다.
- 확정된 행동에 포함된 delayed step, proc, 연속타와 `ExtraAttack`도 해당 행동의 일부로서 정상
  수행한다. 태세 전환이 이미 확정된 행동의 결과를 중간에서 잘라내지 않는다.
- 이미 발사된 projectile, 생성된 area와 적용된 buff/debuff는 기존 lifecycle과 authored tracking 규칙을
  유지한다. 고정 지점 기반 효과는 기존 지점에 남고, 시전자 추적·재탐색 효과는 전술 이동 후의 새
  위치를 반영할 수 있다.
- `Defense`는 전환 이후의 새로운 정규 기본 공격과 automatic/manual skill 시작을 막는다. 이미 예약된
  주기 event라도 아직 공격이나 cast가 확정되지 않았다면 시작하지 않는다.
- 장비·상태에서 발생하는 passive/reactive `Triggered` 효과는 `Defense`에서도 작동한다. `Defense`는
  Silence나 passive 무효화 상태가 아니라 정규 공격·skill 시작을 중단하는 태세다.
- `Triggered` 효과가 공격 windup을 실제로 시작하면 그 공격은 확정된 행동으로 취급하여 정상적으로
  방출한다.
- 전술 순간이동과 자리 교대는 해당 직원에게 아직 방출되지 않은 행동이 있는 동안 거부한다. 정규·triggered
  기본 공격은 `AttackStart`부터 `AttackResolve`까지, automatic/manual skill은 CastStart부터 matching
  CastEnd까지 진행 중이다.
- 기본 공격 projectile이 발사되거나 skill이 CastEnd에서 방출된 뒤에는 남아 있는 projectile, delayed
  delivery, area 또는 buff/debuff만을 이유로 전술 순간이동이나 자리 교대를 막지 않는다. 이후 delayed
  `ExtraAttack`이 실제 windup을 시작하면 그 시점부터 해당 AttackResolve까지 다시 진행 중 행동으로 본다.
- 태세 전환을 위해 `desired_stance`, 공격 취소 token, 행동 예약, 취소·환불 상태를 만들지 않는다.
- 최초 배치와 기존 회수 후 재배치로 `Attack` 태세에 진입할 때도 직원별 `1,500ms` 태세 전환 cooldown을
  시작한다. 배치 직후 첫 공격만 확정하고 즉시 `Defense`로 전환하는 무위험 정답 조작을 방지한다.
- 전술 순간이동은 기존 `Defense` 태세를 유지하며 새로운 태세 전환 cooldown을 시작하지 않는다.
- 6번의 “`Defense`가 새로운 기본 공격을 막는다”는 새로운 정규 기본 공격은 막되, 확정된 행동의 파생
  공격과 passive/reactive `Triggered` 효과는 유지한다는 의미로 구체화한다.

현재 구현된 기반:

- 기본 공격은 `AttackStart`에서 대상과 source damage snapshot을 확정하고 windup 뒤
  `AttackResolve`를 예약한다. resolve는 새 공격 시작 경로와 분리되어 있어 시작 뒤 공격을 끝까지
  수행하는 기반이 이미 있다.
- automatic/manual skill은 CastStart에서 `RuntimeUnit.pending_skill_cast`를 저장하고 matching CastEnd에서
  이를 꺼낸 뒤 `invoke_ability`로 skill을 방출한다. 따라서 focus 중인지 판별하는 정확한 runtime state가
  이미 존재한다.
- `invoke_ability` 뒤의 delayed step, projectile와 area는 `ActiveSkillCast`와 별도 delivery runtime에서
  진행된다. `GroundFixed`, `FollowCaster`, cast target과 impact context 등 authored anchor/tracking 의미도
  이미 분리되어 있다.
- 장비의 `TriggerProc`은 automatic/manual cast를 거치지 않고 proc/ICD를 판정한 뒤 ability를 직접
  invoke한다. `ExtraAttack`은 `schedule_next: false`인 one-shot `AttackStart`를 만들고 event log에서는
  `AttackKind::Triggered`로 구분된다.
- live skill `little_red_hunt_the_prey`는 첫 projectile이 피해를 준 뒤 160ms delayed step에서
  `ExtraAttack` 두 번을 생성한다. 확정된 skill의 파생 공격까지 유지한다는 정책이 실제 live content에
  필요한 사례다.
- live 장비에는 `OnAttack` ability proc과 `OnHit` 반응 효과가 실제로 존재한다. passive/reactive trigger를
  Defense에서 유지할 정책 대상이 현재 content에도 존재한다.
- World의 live command transaction은 employee UUID를 runtime unit으로 해석하고 Core command를 적용한 뒤
  semantic event, 단일 revision commit과 checkpoint/update를 만드는 구조다. Game server는
  `PlayerBehavior`를 직접 deserialize하고 generic `CommandAccepted`와 revisioned battle update를
  전송한다.

정책과 일치하는 부분:

- 현재 runtime은 새 공격 시작과 이미 commit된 resolve를 분리하므로 태세 전환이 공격 취소 token을
  만들지 않아도 기존 공격을 완료할 수 있다.
- pending cast의 stale CastEnd를 무시하고 명시 interrupt/withdraw에서만 pending cast를 취소한다. 태세
  전환 경로에서 이 취소 함수를 호출하지 않으면 확정 정책처럼 focus를 그대로 완료할 수 있다.
- projectile은 launch snapshot과 독립 identity를 사용하고, skill cast state는 delivery가 terminal이 될
  때까지 유지된다. 시전자 태세 변경만으로 이미 방출된 projectile/area/buff를 없앨 이유가 없다.
- `TriggerAbility`와 triggered effect는 automatic/manual cast start gate와 별도 경로다. Defense가 정규
  행동 시작만 막고 passive/reactive trigger는 유지한다는 확정 의미와 구조적으로 맞는다.
- `little_red_hunt_the_prey`의 delayed `ExtraAttack`과 그 cause chain을 현재 runtime과 test가 보존한다.
  확정된 skill의 파생 결과를 잘라내지 않는 정책은 기존 skill 의미도 유지한다.
- server transport는 command 종류별 별도 protocol을 만들지 않고 Core의 typed behavior/result를 generic
  envelope로 전달한다. 선행 태세·이동 command가 추가되면 같은 accepted/update 경계를 재사용할 수 있다.

현재 정책과 다른 코드:

- 현재 tree에는 `EngagementStance`, 태세 전환 command, stance cooldown과 Defense 시작 gate가 없다. 따라서
  windup/focus 중 즉시 태세를 바꾸거나 Defense에서 정규 공격·automatic/manual skill만 막는 정책은 아직
  구현되지 않았다.
- `RuntimeUnit.pending_basic_attack`은 target을 기다리는 미확정 정규 공격을 뜻하며 commit된 windup 상태가
  아니다. `ActionState`도 `Idle | Dead`뿐이다. commit된 공격은 event queue의 `AttackResolve`와 action lock
  timestamp에만 존재하므로 전술 명령이 이를 정확히 질의할 전용 상태가 없다.
- `action_locks.movement_until_ms`는 공격과 skill을 포함한 여러 원인을 합친 capability lock이다. 이를
  그대로 사용하면 원인을 구분할 수 없고, 향후 `focus_permissions.allows_move`인 skill focus도 정책상
  막아야 한다는 의미를 놓친다.
- 반대로 `active_skill_casts`는 delayed step, projectile와 persistent area가 모두 끝날 때까지 유지된다.
  이를 busy flag로 사용하면 skill 방출 뒤에도 전술 순간이동·교대를 오래 막아 확정 정책과 어긋난다.
- 향후 Defense gate를 모든 `AttackStart`에 동일하게 추가하면 `schedule_next: false`인 `ExtraAttack`까지
  차단하게 된다. 정규 periodic start와 triggered one-shot start를 명시적으로 구분해야 한다.
- 최초 배치와 withdraw 후 재배치는 새 runtime unit을 만들지만 초기 `Attack` 태세의
  `stance_ready_at_ms = deploy_time + 1,500ms` 의미가 없다. 현 상태에서는 배치 직후 첫 공격을 확정한 뒤
  즉시 Defense로 바꾸는 조작을 제한할 기반도 없다.
- 4번 전술 순간이동과 14번 자리 교대 command 자체가 아직 없으므로 진행 중 행동을 이유로 해당 command를
  `action_locked`로 거부하는 preflight/DTO도 없다.
- checkpoint deployed-unit DTO에는 stance/cooldown 및 전술 순간이동·교대 eligibility가 없고,
  `BattleLogEvent`에도 stance 전환 semantic event가 없다. 이 큰 wire 공백은 주로 4·6·14번의 구현
  범위지만 현재 tree에서 7번을 완성할 때는 함께 필요하다.

필요한 구현 범위:

- 6번 구현에서 정규 periodic 기본 공격과 automatic/manual cast start만 Defense로 gate한다. Defense에서
  거부된 정규 공격·autocast는 Attack 복귀 뒤 기존 scheduler가 다시 시작할 수 있도록 pending readiness와
  cause를 보존한다.
- `schedule_next: false` triggered `AttackStart`, `TriggerAbility`와 기존 trigger effect에는 Defense start
  gate를 적용하지 않는다. 이들이 시작한 attack windup과 파생 delivery는 기존 cause chain대로 완료한다.
- `RuntimeUnit`에 commit된 기본 공격의 release 경계를 나타내는 작은 battle-time 상태와
  `has_unreleased_unit_action(now_ms)` 같은 단일 질의 seam을 추가한다. overlapping triggered attack도
  안전하게 다루도록 마지막 미방출 resolve 시각 또는 동등한 deterministic 표현을 사용한다.
- 위 질의는 commit된 기본 공격 release 상태와 `pending_skill_cast.is_some()`만 행동 방출 대기로 본다.
  `pending_basic_attack`, 범용 movement lock, `active_skill_casts`, active projectile/area는 이 판정에 사용하지
  않는다.
- 4번 순간이동과 14번 교대의 Core/world preflight가 위 질의를 사용하고 진행 중이면 typed
  `ActionLocked`로 거부한다. 거부 시 위치, 자원, cooldown, event와 revision을 변경하지 않는다.
- deploy와 기존 withdraw/redeploy의 runtime unit 생성 시 최초 `Attack`과 함께
  `stance_ready_at_ms = deploy_time + 1,500ms`를 초기화한다. 전술 순간이동은 기존 Defense와 ready time을
  그대로 보존한다.
- stance transition 자체는 위 action-busy seam으로 막지 않는다. Hard CC 여부만 13번 정책에 따라 별도로
  검사하고, 성공한 transition은 기존 pending attack/cast와 delivery state를 취소하지 않는다.
- checkpoint에는 선행 4·6·14번이 소유하는 stance/cooldown/command eligibility를 투영한다. 내부
  `ActiveSkillCast`나 projectile/area 전체를 command busy 상태로 노출하지 않는다.
- focused runtime test로 attack windup과 manual/auto focus 중 즉시 transition, 기존 resolve/cast 완료,
  Defense의 새 정규 start 억제, triggered proc/ExtraAttack 유지, release 전 이동 거부와 release 뒤 허용,
  projectile/area 비차단, 최초 배치 cooldown을 고정한다. 동일 battle timestamp에서 event와 command가
  만나는 최종 직렬화 순서는 15번 Pause 정책과 함께 고정한다.
- Game server는 선행 command/result variant의 exhaustive mapping과 deserialize/update contract test를
  갱신한다. 7번만을 위한 별도 transport envelope이나 server-owned action state는 만들지 않는다.

신규 정책의 runtime 변경 가치:

- 이미 시작된 공격·skill을 태세 전환 때 취소하면 manual skill 비용·target·환불과 stale event 처리까지
  새 정책이 필요하고 live 연속타 의미도 달라진다. 현재 확정안은 commit된 행동의 기존 의미를 보존하므로
  취소 runtime을 새로 만드는 것보다 단순하고 가치가 높다.
- 다만 기존 movement lock이나 장기 `active_skill_casts`를 그대로 재사용하는 축소 구현은 정책을 정확히
  표현하지 못한다. 기본 공격 release를 나타내는 작은 전용 seam은 추가할 가치가 있다.
- 파생 `ExtraAttack`을 Defense로 차단하는 구현상 간소화도 채택하지 않는다. 실제 live skill을 부분
  실행시키고 플레이어가 본 확정 행동의 결과를 바꾸므로 비용 절감보다 의미 손실이 크다.

난이도 판단:

- 현재 tree 기준 제안은 `4/5 — 높음`이다. 7번만의 상태는 작지만 현재 tree에는 관찰 가능한 태세 command,
  이동·교대 command, checkpoint eligibility와 server result variant가 모두 없다. 완성된 vertical slice는
  4·6·14번의 command/state/runtime/server 기반과 이 정책의 action-release semantics를 함께 통과해야 한다.
- 6번만 먼저 구현되면 태세 전환 중 기존 행동을 유지하고 정규 start만 막는 Core 증분은 `2/5` 수준이다.
  그러나 4·14번 command가 없으므로 “release 전 이동·교대 거부”까지 포함한 7번 전체 slice는 그 시점에
  독립적으로 완료할 수 없다.
- 4·6·14번의 command, DTO와 transaction seam이 먼저 구현된 뒤의 7번 증분 난이도는
  `2/5 — 낮음`이다. 그 경우 남는 핵심은 기본 공격 release marker, 기존 `pending_skill_cast`를 사용한 공통
  busy helper, triggered 예외와 focused deterministic test다.
- Unity는 사용자 결정에 따라 7번부터 조사와 난이도에서 제외했다. 위 등급에는 Unity 구현 비용이 없다.

난이도 책정 reference 파일:

- Core command/wire:
  - `core_enhanced/src/game/behavior.rs`: 현재 live action kind, typed disabled reason, player command,
    checkpoint/deployed-unit DTO와 exhaustive `BehaviorResult` surface를 소유하며 stance/action-busy shape는
    아직 없다.
  - `core_enhanced/src/game/battle/event_log.rs`: `AttackStart/AttackResolve`, cast/ability와 deploy/withdraw
    semantic event를 소유하며 stance transition event는 아직 없다.
- Core world transaction/state:
  - `core_enhanced/src/game/world.rs`: authority 검사 뒤 battle behavior를 live command handler로 dispatch하는
    경계다.
  - `core_enhanced/src/game/world/combat.rs`: deploy/withdraw/manual skill의 preflight, Core command 적용,
    단일 revision commit과 battle update transaction을 소유한다.
  - `core_enhanced/src/game/world/state.rs`: employee↔runtime unit mapping, absolute battle time과 deployed-unit
    eligibility/checkpoint projection을 소유한다.
  - `core_enhanced/src/game/world/battle_capability.rs`: battle command-family capability와 typed preflight의
    공통 경계다.
- Battle runtime:
  - `core_enhanced/src/game/ability.rs`: `Manual/Auto/Triggered` activation mode와 focus permission, delivery 및
    area tracking schema를 정의한다.
  - `core_enhanced/src/game/battle/core/types.rs`: `RuntimeUnit`, action lock, pending basic attack,
    `pending_skill_cast`와 장기 `ActiveSkillCast` state의 서로 다른 책임을 소유한다.
  - `core_enhanced/src/game/battle/core/build.rs`: deploy 시 runtime unit과 공격/cast timing state를 초기화하는
    지점으로 최초 stance cooldown도 이 lifecycle에 맞춰야 한다.
  - `core_enhanced/src/game/battle/core/basic_attack.rs`: periodic/triggered `AttackStart`, commit snapshot과
    windup 뒤 `AttackResolve` release 경계를 소유한다.
  - `core_enhanced/src/game/battle/core/mod.rs`: autocast scheduling, pending cast와 cast 취소/완료 test 기반을
    소유한다.
  - `core_enhanced/src/game/battle/core/sim.rs`: manual cast start, automatic CastStart/CastEnd,
    `invoke_ability`, `ExtraAttack` command 생성과 `BattleLiveCommand` 적용 경계다.
  - `core_enhanced/src/game/battle/core/commands.rs`: triggered ability proc의 직접 invoke, one-shot attack
    scheduling과 projectile launch snapshot 처리를 소유한다.
  - `core_enhanced/src/game/battle/core/triggers.rs`: live equipment의 passive/reactive trigger와 ability binding
    수집 경계다.
  - `core_enhanced/src/game/battle/core/skill_runtime/cast.rs`: delayed step progress와 active cast cleanup을
    소유하여 장기 cast state를 unit busy로 쓰면 안 되는 이유를 보여 준다.
  - `core_enhanced/src/game/battle/core/skill_runtime/projectile.rs`: skill projectile이 launch 뒤 독립 delivery로
    진행되고 impact context를 남기는 경계다.
  - `core_enhanced/src/game/battle/core/skill_runtime/area.rs`: `GroundFixed/FollowCaster/FollowTarget` authored
    tracking과 persistent area lifecycle을 소유한다.
  - `core_enhanced/src/game/battle/enums.rs`: 같은 timestamp의 cast end, skill step, attack start/resolve 우선순위와
    deterministic queue order를 소유한다.
- Policy/live data:
  - `game_resources/data/skills/base.ron`: live focus time, delayed delivery와 `little_red_hunt_the_prey`의 실제
    delayed `ExtraAttack` 두 번을 확인한 source다.
  - `game_resources/data/equipments/base.ron`: live `OnAttack` ability proc, `OnHit` 반응 효과와 ICD를 확인한
    source다.
  - `core_enhanced/src/game/data/run_policy_data.rs`: battle runtime/live deployment tuning의 typed schema이며
    stance cooldown은 아직 없다.
  - `game_resources/data/run/policy.ron`: 50ms movement tick, 60초 battle 상한과 현재 live deployment 정책을
    authoring하며 stance cooldown field는 아직 없다.
- Game server transport:
  - `game_server/src/game/player_game_actor/messages.rs`: Core `PlayerBehavior`를 직접 deserialize하고 battle
    authority와 update/resync side-message 요청을 구성한다.
  - `game_server/src/game/player_game_actor/state.rs`: `BehaviorResult`의 battle update contract를 generic
    `CommandAccepted`와 revisioned side message로 변환하는 exhaustive 경계다.
  - `game_server/src/game/player_game_actor/handlers.rs`: command 실행, active-run persistence 시도, rollback과
    ordered side-message 방출을 소유한다.

실행한 focused test와 결과:

- `cargo test -p game_core committed_instant_basic_attack_snapshot_survives_same_timestamp_source_death` — 통과.
  commit된 기본 공격 resolve가 start 뒤 source lifecycle과 분리되는 기존 기반을 확인했다.
- `cargo test -p game_core live_manual_skill_command_starts_cast_and_reuses_skill_runtime` — 통과. live manual
  command가 공통 pending cast/runtime을 사용하는 기반을 확인했다.
- `cargo test -p game_core manual_pending_cast_can_be_interrupted_and_stale_end_is_ignored` — 통과. pending cast와
  matching CastEnd identity가 분리되어 있음을 확인했다.
- `cargo test -p game_core skill_projectile_impact_uses_launch_source_snapshot_after_caster_death` — 통과. 방출된
  skill projectile이 시전자 이후 상태와 독립적으로 resolve되는 기반을 확인했다.
- `cargo test -p game_core item_activation_proc_respects_internal_cooldown` — 통과. triggered ability proc과 ICD
  기반을 확인했다.
- `cargo test -p game_core withdraw_cancels_active_skill_cast_with_cancelled_event` — 통과. withdraw는 active
  delivery를 명시적으로 취소하므로 전술 이동이 이 lifecycle을 재사용하면 안 됨을 확인했다.
- `cargo test -p game_core little_red_follows_marked_shot_with_two_finish_attacks_on_the_same_prey` — 통과. live
  delayed step이 triggered `ExtraAttack` 두 번을 cause chain으로 생성하는 기반을 확인했다.
- `cargo test -p game_core conditional_followup_waits_for_projectile_damage_resolution` — 통과. projectile impact
  뒤 conditional delayed step이 이어지는 기존 delivery lifecycle을 확인했다.
- `cargo test -p game_core battle_event_heap_orders_by_time_then_priority_then_ids` — 통과. 기존 동일 timestamp
  event 순서가 deterministic함을 확인했다.
- `cargo test -p game_core silence_blocks_new_casts_but_does_not_interrupt_pending_casts` — 통과. 새 시작 gate와
  이미 pending인 cast 완료를 분리할 수 있는 기존 패턴을 확인했다.
- `cargo test -p game_server deserializes_live_battle_requests` — 통과. 현재 generic live command deserialize
  기반을 확인했다.
- `cargo test -p game_server behavior_result_to_command_result_uses_battle_update_side_message` — 통과. accepted
  command와 revisioned update mapping 기반을 확인했다.

이 test 통과는 7번 신규 기능이 구현됐다는 증거가 아니라, 공격 commit, cast/delivery, trigger와 server
transport의 기존 기반이 안정적이라는 증거다. 현재 stance와 action-busy 기능 test는 존재하지 않는다.

후속 번호에서 확정할 항목:

- 7번의 독립 balance tuning은 없다. 최초 배치 cooldown은 6번에서 확정한 대칭 `1,500ms`를 그대로
  사용하며 별도 수치를 만들지 않는다.
- Hard CC와 Silence의 최종 상호작용은 13번, 같은 battle timestamp에서 Pause command와 event가 만나는
  직렬화 의미는 15번에서 확정한다.
- 7번의 현재 tree 난이도 `4/5`, 4·6·14번 선행 뒤 증분 난이도 `2/5`와 독립 tuning 없음은
  2026-08-05에 확정했다.

### 8. Ground 방어 집중 — 축소 정책·코드 대조·난이도 확정 완료

확정일: 2026-08-05  
구현 난이도: 현재 tree `4/5 — 높음`, 6번 선행 후 `3/5 — 중간`, 6·10번 선행 후 증분
`2/5 — 낮음`

확정 정책:

- active 배치 직원이 Ground tile에서 `Defense` 태세이면 장착 weapon의 `DefenseFocus` profile 하나가
  활성화된다.
- `DefenseFocus`를 별도 stance, boolean state 또는 command로 저장하지 않는다. 현재 deployment tile이
  Ground이고 authoritative stance가 `Defense`인지로 파생한다.
- 각 weapon RON의 `DefenseFocus` profile은 `block_capacity_bonus`와
  `incoming_damage_reduction_percent` 두 수치를 직접 소유한다.
- 두 수치는 weapon별로 독립적으로 authoring하며 `0`을 명시적으로 허용한다. profile이 존재한다고 두
  효과가 모두 양수여야 하는 것은 아니다.
- `block_capacity_bonus`는 해당 weapon의 기본 block capacity에 합성하여 active unit의 effective block
  capacity를 만든다.
- `incoming_damage_reduction_percent`는 해당 직원이 받는 피해에만 합성한다. 다른 직원에게 aura처럼
  전파하지 않는다.
- `incoming_damage_reduction_percent`는 기존 `DamageModifiers`의 일반 피해 감소율에 가산한다. 피해
  유형별 감소율까지 합친 최종 감소율은 기존 규칙대로 `0..=100%`로 clamp하며, 별도의 곱연산 방어
  계층을 만들지 않는다.
- Ground `DefenseFocus`는 전술 포인트 회복을 만들지 않는다. 직원별 회복 timer나 BattleCore→World
  주기 signal도 8번 구현 범위에 추가하지 않는다. 9번에서 확정한 Platform `Rest`도 전술 포인트를
  회복하지 않는다.
- `Attack` 전환 또는 Ground 이탈로 활성 조건이 사라지면 두 효과를 즉시 제거한다. 별도 해제 지연이나
  transition tick은 없다.
- effective block capacity 감소로 현재 engagement 수가 새 capacity를 초과해도 교전 시작 순서를 새로
  저장하지 않는다. 기존 engagement만을 정규 저지 우선순위인 경로 진행도, spawn 순서, unit ID 순으로
  다시 판정하여 새 capacity만큼 보존하고 나머지를 해제한다. 아직 engagement되지 않은 적이 기존
  engagement를 밀어내지는 않는다.
- 같은 수치를 공통 run policy나 weapon archetype에 중복 authoring하지 않는다. 구체 수치의 authority는
  개별 weapon RON의 `DefenseFocus` profile 하나다.
- Hard CC 중 효과 처리와 Ground↔Platform 이동에서의 profile 교체 의미는 각각 13번과 12번에서
  확정한다.
- 구체적인 weapon별 두 수치는 10번에서 live weapon의 배치 적성과 profile 구성을 확정한 뒤 tuning한다.

현재 구현된 기반:

- `RuntimeUnit`은 현재 정적인 `block_capacity`, `block_radius_units`와
  `incoming_damage_modifiers`를 보유한다. 직원 생성 시 weapon combat profile의 기본 저지력과 피해
  modifier가 복사되므로 두 효과가 들어갈 runtime stat 경계는 이미 있다.
- FixedDefense의 `refresh_block_state`는 active Player combatant의 저지력과 적의 route 진행도, spawn
  순서, unit ID를 이용해 block 관계를 결정론적으로 다시 만든다. 기존 engagement를 신규 적보다 먼저
  보존하고, 적 사망·blocker 철수 뒤 빈 capacity를 다시 채우는 기반도 있다.
- `DamageModifiers::merge`는 일반 받는 피해 감소율을 다른 modifier와 가산 합성하고, 최종 일반+피해
  유형별 감소율을 `0..=100`으로 clamp한 뒤 피해에 적용한다. `DefenseFocus` 전용 피해 공식을 만들
  필요가 없다.
- 모든 Weapon은 `deny_unknown_fields`인 `WeaponCombatProfile`을 가져야 하고 live loader가 필수 field와
  combat profile을 검증한다. 개별 weapon RON에 strict `DefenseFocus` profile을 추가할 기반이 있다.
- World에는 전술 자원 자연 회복과 skill의 즉시 `StabilizationDelta`를 적용하는 경계가 있지만, 직원별
  stance/profile 기반의 주기 회복 source는 없다. 축소 정책은 이 World 경계를 변경하지 않는다.
- Game server는 Core의 battle behavior 결과를 generic `CommandAccepted`와 revisioned battle update로
  변환한다. 8번은 독립 command나 side message를 만들지 않으므로 6번의 stance update 경계를 그대로
  사용한다.

정책과 일치하는 부분:

- Ground+`Defense`에서 profile을 파생하고 별도 `DefenseFocus` state나 command를 저장하지 않는 정책은
  6번의 확정된 `EngagementStance`와 defense profile selector 방향에 맞는다.
- 저지 관계를 derived state로 매번 재구성하는 현재 구조는 태세나 tile 변경 즉시 effective capacity를
  다시 반영한다는 정책과 맞는다.
- 기존 damage modifier merge와 final clamp는 weapon-authored self 피해 감소를 다른 buff/debuff와
  일관되게 합성할 수 있다.
- 기존 engagement를 아직 저지되지 않은 적보다 먼저 처리하는 현재 규칙은 신규 적이 기존 engagement를
  밀어내지 않는 축소 정책과 일치한다.
- Ground 전술 포인트 회복을 제거함으로써 battle stance를 소유한 BattleCore와 global resource를 소유한
  World 사이에 새 per-unit timer/signal을 만들 필요가 없어졌다.

현재 정책과 다른 코드:

- 현재 tree에는 `EngagementStance`, 현재 deployment tile kind 기반 defense profile selector와
  `DefenseFocus` profile이 아직 없다. active 직원은 태세·tile과 무관하게 raw `block_capacity`를
  사용한다.
- `WeaponCombatProfile`에는 `deployment_affinity`, `DefenseFocus`, `block_capacity_bonus`,
  `incoming_damage_reduction_percent`가 없고 live weapon RON도 이를 authoring하지 않는다.
- `RuntimeUnit`은 base와 effective block capacity를 구분하지 않는다. 현재 `refresh_block_state`는 raw
  field를 직접 읽는다.
- 기존 engagement 보존 단계는 blocker ID와 enemy ID로 정렬한다. 축소 정책의 경로 진행도→spawn
  순서→unit ID 우선순위를 capacity 감소 시 기존 engagement 사이에도 적용하려면 이 정렬을 명시적으로
  바꿔야 한다. 다만 교전 시각·순번 state는 필요하지 않다.
- incoming modifier를 읽는 damage snapshot 경로들은 정적인 `RuntimeUnit.incoming_damage_modifiers`를
  직접 사용한다. Ground `Defense` 동안만 weapon 수치를 합성하는 단일 effective helper가 없다.
- checkpoint/server에는 8번 전용 field나 command가 없다. 수치 효과 자체에는 새 transport가 필요하지
  않지만, 활성 조건인 stance와 tile/profile 계약은 6번·10번 구현에 의존한다.

필요한 구현 범위:

- weapon data에 strict `DefenseFocusProfile { block_capacity_bonus,
  incoming_damage_reduction_percent }`를 추가한다. percentage는 `0..=100`을 검증하고 block 합성은 정수
  overflow가 없도록 checked 또는 saturating 합성을 사용한다.
- 6번의 authoritative stance와 10번의 weapon deployment/profile 구조를 이용해 `Ground && Defense`일
  때만 활성 profile을 반환하는 helper를 둔다. profile을 별도 mutable state로 복제하지 않는다.
- base block capacity와 현재 stance/tile을 합성하는 effective block helper를 만들고
  `refresh_block_state`가 이를 사용하게 한다. 태세 전환·순간이동·교대 뒤 같은 authoritative mutation
  안에서 block state를 다시 계산한다.
- capacity 감소 시 기존 engagement만 정규 저지 우선순위로 정렬해 새 capacity만큼 유지한 뒤, 남은
  capacity에만 신규 적을 배정한다. engagement timestamp/sequence와 복원 history는 추가하지 않는다.
- base incoming modifier와 활성 profile의 일반 피해 감소를 합성하는 effective helper를 두고 기본
  공격, projectile와 skill damage snapshot이 같은 값을 사용하도록 연결한다. 태세가 바뀌기 전에 이미
  생성된 projectile의 source lifecycle은 유지하되, 피격 시점 target modifier를 읽는 기존 damage 의미는
  바꾸지 않는다.
- Ground-capable live weapon에 두 수치를 명시하고 affinity/profile 조합 및 누락·범위 오류를 strict live
  data test로 고정한다. 구체 값은 10번의 전체 weapon 배치 적성과 함께 정한다.
- 8번 전용 World resource timer, `BattleLiveSignal`, command, server message와 별도 Unity 계약은 만들지
  않는다.

신규 정책의 runtime 변경 가치:

- 저지력과 self 피해 감소는 공격을 포기한 Ground 직원에게 명확한 생존·전선 유지 보상을 주므로 태세
  선택의 가치가 있다. 기존 block과 damage modifier seam을 재사용하므로 효과 자체도 runtime과 잘 맞는다.
- 반면 제거한 Ground 전술 포인트 회복은 정수 최소 `1/s`만으로도 현재 자연 회복 `1/s`, 순간이동 비용
  `5`, cooldown `10초` 경제에 비해 지나치게 크고 per-unit 주기 state와 Core→World signal을 새로
  요구했다. 8번의 방어 정체성에 필수적이지 않아 축소 가치가 더 크다.
- 교전 시작 순서를 새로 보존하는 것도 플레이어가 관찰하기 어려운 capacity 감소 edge case를 위해 새
  state lifecycle을 만드는 비용이 컸다. 기존 정규 저지 우선순위를 재사용하는 쪽이 더 단순하고
  결정론적이다.

난이도 판정:

- **현재 tree 기준 `4/5 — 높음`**이다. 두 효과만 보면 집중된 Core/data 변경이지만 활성 조건인
  6번 stance command/state/checkpoint와 10번 weapon profile/affinity가 현재 구현되어 있지 않다. 현재
  tree에서 완성된 vertical slice를 만들려면 이 선행 end-to-end 경계도 함께 필요하다.
- 6번의 stance와 defense profile selector만 먼저 실제 구현된 경우 8번 증분 난이도는
  `3/5 — 중간`이다. strict profile field/live migration, effective block/damage 합성과 focused test가
  남는다.
- 6번과 10번의 stance·weapon schema/live migration이 모두 먼저 실제 구현된 경우 증분 난이도는
  `2/5 — 낮음`이다. 두 effective stat 합성, block retention 정렬과 focused test가 중심이 된다.
- 앞서 의미 축소 직후 제시한 현재 tree `3/5`는 효과 자체의 범위만 본 잠정치였다. complete vertical
  slice가 아직 없는 6번 command/state/server 경계를 다시 포함하면 이 문서의 난이도 기준상 `4/5`가
  일관된 값이다.

난이도 책정 reference 파일:

- Core command/wire:
  - `core_enhanced/src/game/behavior.rs`: 현재 battle `ActionKind`, `PlayerBehavior`, authority 요구와 live
    update DTO를 소유하며, 8번 전용 command는 없고 6번 stance command가 아직 필요함을 확인했다.
- Core world transaction/state:
  - `core_enhanced/src/game/world/combat.rs`: battle command의 선검증, BattleCore mutation과 revision commit
    경계로, stance 변경 뒤 derived effect를 같은 transaction에서 반영해야 하는 지점이다.
  - `core_enhanced/src/game/world/state.rs`: battle checkpoint와 현재 자원 자연 회복·live signal 적용을
    소유하며, 축소한 8번이 새 per-unit 자원 recovery state를 요구하지 않음을 확인했다.
- Battle runtime:
  - `core_enhanced/src/game/battle/core/types.rs`: `RuntimeUnit`의 정적 block capacity와 incoming damage
    modifier를 소유하므로 base/effective 합성의 중심이다.
  - `core_enhanced/src/game/battle/core/build.rs`: combat profile에서 runtime block/damage stat을 만드는
    생성 경계다.
  - `core_enhanced/src/game/battle/core/mod.rs`: 현재 block 관계를 두 방향 map만으로 저장하며 engagement
    chronology가 없음을 확인했다.
  - `core_enhanced/src/game/battle/core/movement/blocking.rs`: effective block 사용, 기존 engagement 보존,
    정규 적 우선순위와 capacity 초과 해제를 구현할 authoritative 경계다.
  - `core_enhanced/src/game/battle/damage.rs`: damage modifier 가산 합성, 감소율 clamp와 최종 피해 계산을
    소유한다.
  - `core_enhanced/src/game/battle/core/damage_runtime.rs`: runtime target incoming modifier를 damage
    snapshot으로 전달하므로 활성 `DefenseFocus` 피해 감소를 연결할 경계다.
  - `core_enhanced/src/game/battle/core/commands.rs`: instant 기본 공격 resolve 시점의 target modifier
    snapshot을 만들므로 effective incoming modifier를 적용해야 하는 경계다.
  - `core_enhanced/src/game/battle/core/basic_attack_projectile.rs`: projectile impact 시점의 live target
    modifier를 읽으므로 태세 전환 뒤 활성 피해 감소를 반영해야 하는 경계다.
  - `core_enhanced/src/game/battle/types.rs`: employee combat profile에 weapon block capacity를 적용하는
    현재 base stat 조립 경계다.
- Policy/live data:
  - `core_enhanced/src/game/data/equipment_data.rs`: strict `WeaponCombatProfile` schema와 validation 대상
    field를 소유한다.
  - `game_resources/data/equipments/base.ron`: 모든 live weapon의 현재 block capacity를 명시하며 새
    `DefenseFocus` 두 수치의 실제 authoring 대상이다.
  - `core_enhanced/src/game/data/run_policy_data.rs`: 현재 global 자연 회복과 전투 상한 policy schema를
    소유하며 축소한 두 효과에는 새 공통 tuning field가 필요하지 않음을 확인했다.
  - `game_resources/data/run/policy.ron`: live `initial 20 / maximum 99 / 자연 회복 1/s`, 60초 전투 상한을
    명시해 Ground 자원 회복 제거 판단의 경제 기준이 됐다.
- Game server transport:
  - `game_server/src/game/player_game_actor/state.rs`: Core battle result를 `CommandAccepted`와 revisioned
    side message로 변환하는 generic mapping을 소유한다.
  - `game_server/src/game/player_game_actor/handlers.rs`: command 실행, persistence/rollback과 side-message
    방출 transaction을 소유한다. 8번 자체에는 새 server 분기가 필요하지 않는다.
  - `game_server/src/game/player_game_actor/messages.rs`: Core live checkpoint/update를 server message로
    투영하며 8번 수치 효과를 위한 별도 message가 필요하지 않음을 확인했다.
- 외부 Unity runtime/contract/tests:
  - 사용자 결정에 따라 7번 이후 조사 범위에서 제외했다. 8번 난이도에도 Unity 구현 비용을 별도로
    더하지 않았다.

실행한 focused test와 결과:

- `cargo test -p game_core fixed_defense_preserves_existing_block_before_new_route_priority` — 통과. 기존
  engagement를 신규 적보다 먼저 보존하는 기반을 확인했다.
- `cargo test -p game_core fixed_defense_block_capacity_prioritizes_route_progress_and_allows_excess_to_pass`
  — 통과. 현재 정규 적 저지 우선순위와 capacity 초과 통과 기반을 확인했다.
- `cargo test -p game_core fixed_defense_releases_block_when_enemy_dies_and_fills_capacity` — 통과. block
  관계 해제와 빈 capacity 재배정 기반을 확인했다.
- `cargo test -p game_core target_incoming_modifiers_reduce_all_incoming_damage` — 통과. 일반 incoming 피해
  감소가 모든 피해에 합성되는 기반을 확인했다.
- `cargo test -p game_core post_mitigation_modifiers_apply_after_resistance` — 통과. 기존 피해 감소 적용
  순서와 clamp 기반을 확인했다.
- `cargo test -p game_core live_weapon_block_capacity_is_explicit_and_matches_approved_guideline` — 통과.
  현재 live weapon별 저지력 명시 기반을 확인했다.
- `cargo test -p game_core source_set_rejects_weapon_profile_missing_block_capacity` — 통과. strict weapon
  profile 누락 거부 기반을 확인했다.
- `cargo test -p game_core manual_fragment_can_restore_stabilization_and_enable_extra_deployment` — 통과.
  현재 World의 즉시 전술 자원 증감 seam을 확인했으며, Ground 주기 회복이 이미 구현됐다는 증거는 아니다.
- `cargo test -p game_server behavior_result_to_command_result_uses_battle_update_side_message` — 통과. 현재
  generic server battle-update mapping 기반을 확인했다.

이 test 통과는 8번 신규 효과가 구현됐다는 증거가 아니라 저지, 피해 modifier, strict weapon data와
generic server transport의 기존 기반이 안정적이라는 증거다. 현재 stance/profile/effective stat을
결합한 8번 test는 없다.

난이도·tuning 확정 결과:

- 현재 tree 난이도 `4/5`, 6번만 선행 시 `3/5`, 6·10번 선행 시 증분 `2/5`로 확정했다.
- 8번에는 공통 balance tuning을 두지 않는다. weapon별 `block_capacity_bonus`와
  `incoming_damage_reduction_percent` 값은 10번에서 base blocker만 block +1, Ground weapon 공통 피해
  감소 10%로 확정했다. Ground 전술 포인트 회복 수치와 cadence는 축소 정책에서 제거했으므로 tuning
  대상이 아니다.
- 피해 감소는 기존 일반 피해 감소율에 가산하고 피해 유형별 감소까지 합친 최종값을 `0..=100%`로
  clamp하며 별도 곱연산 계층을 만들지 않는 것으로 확정했다.

### 9. Platform 휴식 — 정책·코드 대조·난이도·tuning 확정 완료

확정일: 2026-08-06  
구현 난이도: 현재 tree `4/5 — 높음`, 6번 선행 후 `3/5 — 중간`, 6·10번 선행 후 증분
`2/5 — 낮음`

확정 정책:

- active 배치 직원이 Platform tile에서 `Defense` 태세이면 장착 weapon의 `Rest` profile 하나가
  활성화된다.
- `Rest`를 별도 stance, boolean state 또는 command로 저장하지 않는다. 현재 deployment tile이
  Platform이고 authoritative stance가 `Defense`인지로 파생한다.
- `Rest`는 해당 active 직원의 battle HP와 공명만 authoritative battle time에 따라 회복한다. 전투불능
  직원을 부활시키지 않으며 전술 포인트를 회복하지 않는다.
- HP 회복과 공명 회복은 weapon별로 독립적으로 authoring하며 각각 `0`을 명시적으로 허용한다. profile이
  존재한다고 두 회복이 모두 양수여야 하는 것은 아니다.
- 모든 `Rest`는 run policy의 공통 `rest_tick_interval_ms` 하나를 사용하며 첫 live 값은 `1,000ms`다.
  무기별로 서로 다른 cadence를 두지 않고 개별 weapon RON은 `hp_recovery_per_tick`과
  `resonance_recovery_per_tick` 두 회복량만 소유한다.
- `Rest` 활성화 뒤 온전한 interval 하나가 지난 시점에 첫 tick을 적용한다. 활성 조건을 잃었다가 다시
  만족하면 이전 주기를 이어받지 않고 새 interval을 처음부터 시작한다. stale tick 중복을 막는 내부
  timer/epoch는 scheduling bookkeeping이며 별도 gameplay `Rest` state가 아니다.
- 각 회복 적용은 `min(current + recovery, maximum)`으로 자신의 상한에서 clamp한다. 상한을 넘은 양을
  저장·이월·변환하지 않고, 상한에 머문 동안의 회복량도 비축하지 않는다. 이후 피해를 받거나 공명을
  소비하면 다음 정상 회복 시점부터 다시 적용한다.
- 피격 여부와 active wave 유무는 `Rest`를 중단하거나 회복 지연을 새로 시작하지 않는다. `recently_hit`,
  휴식 재개 대기시간이나 별도 interruption state를 만들지 않는다.
- `Attack` 전환 또는 Platform 이탈로 활성 조건이 사라지면 HP·공명 회복을 즉시 중단한다. 별도 해제
  지연이나 transition tick은 없다.
- `Defense` 중 공명이 maximum에 도달해도 automatic skill을 시작하지 않고 현재 공명을 보존한다. 이후
  `Attack`으로 전환하면 그 시점의 일반 automatic skill eligibility를 검사해 필요한 pending cast를
  만들며 즉시 시전을 보장하지 않는다.
- `Rest` 공명 회복은 기존 `resonance_gain_until_ms`를 우회하지 않는다. 시전/focus 뒤 공명 gain lock이
  active이면 해당 tick의 공명 회복은 적용하지 않고 나중에 소급 지급하거나 비축하지 않는다.
- HP나 공명이 이미 maximum이라 실제 변화가 없으면 0변화 `HpChanged`/`ResonanceChanged` event를
  방출하지 않는다.
- 같은 timestamp에는 이미 commit된 projectile, skill step, buff damage와 basic attack resolve 등 피해를
  Rest보다 먼저 처리한다. 그 피해로 unit이 전투불능이면 같은 timestamp의 Rest가 사후 회복시키거나
  부활시키지 않는다.
- 공통 run policy는 cadence만 소유한다. weapon archetype에서 회복량을 추론하지 않고 구체 HP·공명
  per-tick 양의 authority는 개별 weapon RON의 `Rest` profile 하나다.
- Hard CC 중 회복 처리와 Pause 중 battle-time 진행은 각각 13번과 15번, Ground↔Platform 이동에서의
  profile 교체 의미는 12번에서 확정한다.
- 구체적인 weapon별 HP·공명 per-tick 수치는 실제 HP·공명 runtime과 live 전투 주기를 대조한 뒤
  tuning한다.

현재 구현된 기반:

- `RuntimeUnit`은 active lifecycle, `UnitStats.current_health/max_health`,
  `resonance_current/max`와 `resonance_gain_until_ms`를 이미 한곳에 보유한다.
- `apply_hp_delta_and_record`는 active unit만 변경하고 양수 delta를 maximum에서 clamp한다. 이미 죽거나
  withdraw된 unit을 회복하지 않으므로 Rest가 부활을 만들지 않는 기반이 있다.
- `add_resonance`는 active/hard CC/gain lock을 확인하고 maximum에서 clamp하며, 공명이 가득 찰 때 auto
  cast를 예약할지 호출자가 선택할 수 있다. Rest tick은 `allow_autocast_when_full: false`를 재사용할 수
  있다.
- HP는 `HpChanged`, 공명은 `ResonanceChanged` semantic event와 live checkpoint HUD/stat에 이미
  투영된다. Rest 전용 외부 message 없이 기존 event/checkpoint로 실제 수치 변화를 전달할 수 있다.
- `BattleEvent` heap은 battle time, event priority와 stable ID로 결정론적 순서를 제공한다. Buff와 skill
  area에는 periodic tick 예가 있지만 derived Rest profile을 위한 tick event는 없다.
- World의 deploy validation은 Ground/Platform deployment zone을 구분하고 affinity를 검사한다.
  BattleCore 자체는 현재 unit의 실제 deployment zone kind를 보존하거나 조회하지 않는다.
- Game server는 Core의 generic battle update와 checkpoint를 그대로 투영한다. 9번은 독립 player
  command나 global resource mutation을 만들지 않는다.

정책과 일치하는 부분:

- 기존 HP helper는 active 검사, maximum clamp와 event 기록을 제공하므로 HP Rest에 별도 생명주기나
  healing formula가 필요하지 않다.
- 기존 공명 helper는 gain lock, maximum clamp와 `allow_autocast_when_full`을 분리한다. Defense에서
  공명을 채우되 auto cast를 예약하지 않는 정책과 정확히 맞는다.
- live checkpoint는 매 update마다 현재 HP와 공명을 authoritative하게 다시 투영하므로 reconnect/resync를
  위한 Rest 전용 누적 DTO가 필요하지 않다.
- Pause는 World가 simulation delta 자체를 만들지 않아 battle event time이 진행되지 않는다. Rest를
  battle event로 구현하면 별도 wall-clock 정지 로직 없이 15번 방향과 맞는다.
- 전술 포인트 회복을 제거했으므로 9번은 BattleCore 밖의 global resource pool이나 Core→World signal을
  변경하지 않는다.

현재 정책과 다른 코드:

- 현재 tree에는 `EngagementStance`, `RestProfile`, 실제 current tile kind selector와 Rest activation
  helper가 없다. 모든 직원 combat profile의 기본 affinity도 `GroundOnly`다.
- `WeaponCombatProfile`과 live weapon RON에는 `Rest`, `hp_recovery_per_tick`,
  `resonance_recovery_per_tick`가 없다. 6번에서 PlatformOnly로 정한 Staff도 아직 affinity/profile을
  authoring하지 않는다.
- run policy에는 `rest_tick_interval_ms`가 없고 `BattleEvent`에도 Rest tick이 없다. 현재 periodic buff
  tick은 active buff instance와 validator를 전제로 하므로 파생 profile인 Rest를 buff로 위장해 재사용할
  수 없다.
- Rest 이탈·재진입 뒤 stale event를 구분할 per-unit scheduling epoch가 없다. 무조건 event만 다시
  예약하면 중복 회복 가능성이 생긴다.
- `apply_hp_delta_and_record`는 요청 delta가 양수이면 maximum 상태에서도 applied delta 0인 `HpChanged`를
  기록할 수 있다. Rest caller가 실제 부족 HP를 먼저 확인해야 한다.
- 공명이 Defense에서 maximum이 되어도 pending cast를 만들지 않는 것은 기존 helper로 가능하지만,
  이후 Attack 전환 시 full auto unit을 일반 eligibility로 다시 enqueue하는 6번 통합 경계는 아직 없다.

필요한 구현 범위:

- run policy `battle_runtime`에 positive `rest_tick_interval_ms`를 추가하고 live 값을 `1,000`으로
  authoring한다. 무기별 interval이나 legacy fallback은 만들지 않는다.
- strict weapon data에 `RestProfile { hp_recovery_per_tick, resonance_recovery_per_tick }`를 추가한다.
  두 값은 `u32` 범위에서 0을 허용하고 Platform-capable weapon/profile 조합 validation은 10번과 함께
  고정한다.
- 6번 stance와 10번 affinity/profile 구조를 이용해 `Platform && Defense`인 active employee의 Rest
  profile을 반환하는 단일 derived helper를 둔다.
- `RuntimeUnit`에 Rest scheduling epoch 또는 동등한 내부 식별자를 두고 activation 뒤
  `now + rest_tick_interval_ms`에 unit-scoped `RestTick`을 예약한다. tick은 epoch와 현재 derived
  eligibility를 다시 검증하고 유효할 때만 다음 tick을 예약한다. 이탈·재진입은 epoch를 바꿔 stale
  event를 무효화하고 새 full interval을 시작한다.
- Rest tick에서 부족 HP가 있을 때만 self-source HP 양수 delta를 적용하고, 공명은 기존 gain lock을
  존중하면서 `allow_autocast_when_full: false`로 더한다. 두 수치가 maximum이거나 profile 값이 0이면
  변화 event를 만들지 않는다.
- Attack 전환 성공 시 Auto mode이면서 공명이 full인 unit은 그 시점의 hard CC/Silence/action/target
  조건을 사용하는 기존 pending-autocast 경로로 연결한다. Manual mode는 공명을 보존하고 player command를
  기다린다.
- 같은 timestamp의 Rest와 피해 순서를 새 event priority와 heap test로 고정한다. 권고는 projectile,
  skill step, buff damage와 basic attack resolve 등 이미 commit된 피해를 먼저 처리하고 그 결과 unit이
  active일 때만 Rest를 적용하는 것이다. Rest가 같은 timestamp의 치명타를 사후에 되돌리지는 않는다.
- HP·공명 변화는 기존 semantic event/checkpoint를 사용한다. Rest 전용 player command, World
  transaction, server message와 전술 자원 signal은 추가하지 않는다.

신규 정책의 runtime 변경 가치:

- Platform에서 공격을 포기하는 대가를 개인 생존과 다음 skill 준비로 돌려주는 효과라 Ground
  DefenseFocus와 역할이 명확히 다르며 도입 가치가 있다.
- HP·공명과 event/checkpoint 기반이 이미 있어 새로운 자원 모델은 필요하지 않다. Rest를 별도 stance나
  buff로 만들지 않고 tile+stance+weapon profile로 파생하는 현재 정책이 가장 작다.
- 무기별 cadence는 평균 회복량이 같은데도 burst timing과 stale event 조합만 늘린다. 공통 1초 cadence와
  weapon별 per-tick 양으로 축소한 편이 player 예측성, live tuning과 test 범위를 함께 줄인다.
- 피격 interruption/recently-hit timer도 Platform 회복의 핵심 가치보다 상태·wire 비용이 크므로 추가하지
  않는 현재 정책이 타당하다.

난이도 판정:

- **현재 tree 기준 `4/5 — 높음`**이다. Rest effect 자체는 집중된 data/runtime 기능이지만 활성
  조건인 6번 stance command/state/checkpoint와 10번 weapon affinity/profile이 현재 구현되어 있지 않아
  complete vertical slice에는 그 end-to-end 기반도 필요하다.
- 6번 stance와 defense profile selector만 먼저 실제 구현된 경우 `3/5 — 중간`이다. weapon/live schema,
  Rest event/epoch, HP·공명 적용과 event-order test가 남는다.
- 6번과 10번의 stance·weapon schema/live migration이 모두 먼저 실제 구현된 경우 증분 난이도는
  `2/5 — 낮음`이다. 공통 cadence event, 작은 scheduling epoch와 회복/전환 focused test가 중심이다.

초기 tuning 판정:

- 공통 `rest_tick_interval_ms: 1,000`은 의미 보완과 함께 확정했다.
- 첫 실제 Platform weapon의 기준값으로 `hp_recovery_per_tick: 3`,
  `resonance_recovery_per_tick: 5`를 확정했다. 모든 weapon을 강제로 같은 값으로 만드는 공통 default가
  아니라 10번 per-weapon authoring의 첫 balance guideline이다.
- starter 직원은 base HP 100에 `standard_armor`의 +80 HP로 보통 180 HP에서 시작한다. 초당 3 HP는
  10초 Rest에 30 HP를 회복해 의미가 있지만 짧은 휴식으로 전부 회복시키지는 않는다.
- live 일반 corroded enemy는 attack 8~15, interval 약 1,200~1,900ms다. 3 HP/s는 일반 피해를 일부
  상쇄하지만 집중 공격을 무효화하지 않으며 boss급 58 이상의 공격을 버티게 하는 주 방어 수단도 아니다.
- 공명 maximum은 100이고 기본 공격 release만으로도 10을 얻는다. live weapon의 약 1,350~1,800ms 공격
  주기를 기준으로 공격 태세는 대략 13.5~18초에 기본 release 공명 100을 만들 수 있다. Rest 5/s는
  20초가 필요하므로 공격보다 약간 느리되 HP 회복을 함께 얻는 trade-off가 된다.
- placeholder weapon을 포함한 실제 per-weapon 값과 `0` 사용 여부는 10번 affinity/profile migration에서
  확정한다.

난이도 책정 reference 파일:

- Core command/wire:
  - `core_enhanced/src/game/behavior.rs`: live unit stats/HUD, HP·공명 presentation event와 deployed-unit skill
    readiness DTO를 소유해 Rest 결과가 기존 checkpoint/update로 전달됨을 확인했다.
  - `core_enhanced/src/game/battle/event_log.rs`: `HpChanged`, `ResonanceChanged`와 HP change reason을
    소유하는 authoritative semantic event 경계다.
- Core world transaction/state:
  - `core_enhanced/src/game/world/combat.rs`: Ground/Platform deployment zone lookup, affinity validation과
    battle command/revision transaction을 소유한다.
  - `core_enhanced/src/game/world/state.rs`: Pause 기반 battle time과 매 update HP·공명 checkpoint
    projection을 소유한다.
- Battle runtime:
  - `core_enhanced/src/game/battle/core/types.rs`: `RuntimeUnit` HP·공명과 resonance gain lock을 소유하며
    Rest scheduling epoch가 들어갈 unit runtime 경계다.
  - `core_enhanced/src/game/battle/core/build.rs`: combat profile에서 runtime HP·공명과 unit state를 만드는
    생성 경계다.
  - `core_enhanced/src/game/battle/core/mod.rs`: 공명 clamp, hard CC/gain-lock 검사와 optional auto-cast
    scheduling을 구현한 기존 helper를 소유한다.
  - `core_enhanced/src/game/battle/core/damage_runtime.rs`: active-only HP delta, maximum clamp, death와
    `HpChanged` 기록을 소유한다.
  - `core_enhanced/src/game/battle/core/commands.rs`: skill/trigger heal과 resonance command가 기존 HP·공명
    helper로 들어가는 공통 mutation 경계다.
  - `core_enhanced/src/game/battle/core/sim.rs`: battle event scheduling, cast 종료의 공명 reset/lock과
    pending autocast lifecycle을 소유한다.
  - `core_enhanced/src/game/battle/enums.rs`: event time/priority/stable ordering을 소유하며 새 Rest tick의
    동일 timestamp 의미를 고정할 경계다.
- Policy/live data:
  - `core_enhanced/src/game/data/equipment_data.rs`: strict `WeaponCombatProfile` schema와 새 `RestProfile`
    field/validation 대상이다.
  - `core_enhanced/src/game/data/run_policy_data.rs`: `battle_runtime` cadence와 positive validation을 추가할
    policy schema다.
  - `game_resources/data/run/policy.ron`: movement 50ms, battle 상한 60,000ms를 명시해 1초 Rest cadence의
    live 시간 기준이 됐다.
  - `game_resources/data/equipments/base.ron`: Staff를 포함한 모든 live weapon profile과 표준 방어구 +80
    HP를 authoring하며 Rest per-tick 값 migration의 대상이다.
  - `game_resources/data/employees/definitions.ron`: 현재 직원 base HP가 100임을 확인한 tuning 기준이다.
  - `game_resources/data/enemies/corroded_employees.ron`: 일반 적 attack 8~15와 1,200~1,900ms 공격 주기를
    명시해 HP 회복량의 live 비교 기준이 됐다.
  - `game_resources/data/abnormalities/base.ron`: boss급 HP/attack과 공명 100 baseline을 확인해 Rest가
    boss 피해를 무효화하지 않아야 한다는 상한 판단에 사용했다.
- Game server transport:
  - `game_server/src/game/player_game_actor/state.rs`: Core battle result를 `CommandAccepted`와 revisioned
    update로 변환하는 generic mapping이다. 9번 전용 server branch는 필요하지 않다.
  - `game_server/src/game/player_game_actor/messages.rs`: Core unit stats/HUD와 semantic event를 server
    battle update로 투영한다.
- 외부 Unity runtime/contract/tests:
  - 사용자 결정에 따라 조사하지 않았고 9번 난이도에도 별도 Unity 구현 비용을 더하지 않았다.

실행한 focused test와 결과:

- `cargo test -p game_core add_resonance_clamps_and_sets_pending_cast_only_when_allowed` — 통과. 공명 maximum
  clamp, gain lock과 auto-cast 예약 선택 기반을 확인했다.
- `cargo test -p game_core manual_activation_mode_does_not_schedule_autocast_when_resonance_fills` — 통과.
  공명이 가득 차도 activation mode에 따라 자동 시전을 만들지 않는 기반을 확인했다.
- `cargo test -p game_core --test skill_test_suite white_night_heals_and_buffs_allies_before_judging_enemies` —
  통과. 기존 live skill HP 회복과 event/runtime 적용 기반을 확인했다.
- `cargo test -p game_core live_defense_playback_pause_freezes_server_tick_and_resume_advances` — 통과. Pause가
  authoritative battle time을 정지하는 기반을 확인했다.
- `cargo test -p game_core battlefield_template_exposes_ground_and_platform_deployment_zones` — 통과. live
  battlefield의 Ground/Platform 구분 기반을 확인했다.
- `cargo test -p game_core platform_only_spawned_unit_cannot_block_even_if_profile_has_capacity` — 통과. 현재
  PlatformOnly runtime profile의 block 0 기반을 확인했다.
- `cargo test -p game_core battle_event_heap_orders_by_time_then_priority_then_ids` — 통과. 새 periodic event가
  따라야 할 현재 결정론적 ordering 기반을 확인했다.
- `cargo test -p game_core live_weapon_block_capacity_is_explicit_and_matches_approved_guideline` — 통과. strict
  live weapon profile migration 기반을 확인했다.
- `cargo test -p game_server behavior_result_to_command_result_uses_battle_update_side_message` — 통과. 기존
  generic server update mapping 기반을 확인했다.

이 test 통과는 9번 Rest가 구현됐다는 증거가 아니라 HP·공명, Pause, Platform data, deterministic event와
server transport의 기존 기반이 안정적이라는 증거다. Rest profile/tick/stance 결합 test는 현재 없다.

난이도·tuning 확정 결과:

- 현재 tree `4/5`, 6번 선행 후 `3/5`, 6·10번 선행 후 증분 `2/5`로 확정했다.
- 같은 timestamp에는 commit된 피해를 Rest보다 먼저 처리하고, 그 피해로 전투불능이면 Rest가 사후
  회복시키지 않는 event ordering으로 확정했다.
- 첫 per-weapon guideline은 `HP 3/tick / 공명 5/tick`으로 확정했고, 10번에서 현재 Staff weapon
  16개 전부의 명시 live 값으로 확정했다.

### 10. 무기 데이터 소유권 — 정책·코드 대조·난이도·tuning 확정 완료

확정일: 2026-08-06  
구현 난이도: 현재 tree `3/5 — 중간`, 6·12번 선행 후 증분 `2/5 — 낮음`

확정 정책:

- 개별 weapon RON의 `WeaponCombatProfile`이 다음 전투 배치·태세 속성의 단일 authoring source다.
  - `deployment_affinity`: `GroundOnly | PlatformOnly | Any`
  - 기본 `block_capacity`
  - Ground `DefenseFocus`: `block_capacity_bonus`, `incoming_damage_reduction_percent`
  - Platform `Rest`: `hp_recovery_per_tick`, `resonance_recovery_per_tick`
- weapon archetype, 직원 기초 능력치, legacy 기본값이나 현재 effective block에서 위 값을 추론하거나
  누락값을 보충하는 fallback은 두지 않는다.
- `GroundOnly` weapon은 `DefenseFocus`를 명시하고 `Rest`는 갖지 않는다. `PlatformOnly` weapon은
  `Rest`를 명시하고 `DefenseFocus`는 갖지 않는다. `Any` weapon은 두 profile을 모두 명시한다.
- affinity상 사용할 수 있는 profile은 존재해야 하지만 profile 내부 개별 수치는 독립적으로 `0`을
  허용한다. 사용할 수 없는 tile kind의 profile은 명시하지 않는다.
- `PlatformOnly` weapon의 기본 `block_capacity`는 `0`이어야 한다. `Any` weapon도 실제 Platform tile에
  배치된 동안에는 effective block capacity `0`을 사용한다.
- Ground 배치 가능 여부와 저지 역할을 결합하지 않는다. `GroundOnly`와 `Any` weapon도 명시적인 기본
  `block_capacity: 0`을 가질 수 있다. 따라서 Ground 비저지 원거리·지원 weapon, Attack에서는 막지 않고
  `DefenseFocus` bonus로만 저지력을 얻는 weapon, 피해 감소만 제공하는 `DefenseFocus`를 허용한다.
- weapon profile은 배치 capability와 가능한 효과 수치를 정의하고, 실제 current tile kind가
  `DefenseFocus` 또는 `Rest` 중 활성 profile을 결정한다.
- Rest 공통 tick 주기와 태세 전환 cooldown은 weapon 데이터가 아니라 run policy가 소유한다.
- 무기 없는 직원의 eligibility와 표시 의미는 11번에서 별도로 확정한다. 누락 weapon을 위한 기본
  tactical profile은 10번에서 만들지 않는다.
- 구체적인 live weapon별 affinity와 `DefenseFocus`/`Rest` 수치는 실제 data/runtime을 대조한 뒤 10번
  tuning으로 별도 확정한다.

현재 구현된 기반:

- `DeploymentAffinity`에는 이미 `GroundOnly`, `PlatformOnly`, `Any`가 있고 `DeploymentZoneKind`는
  Ground/Platform cell이 해당 affinity를 지원하는지 판정한다.
- World의 최초 배치와 전투 중 배치 handler는 직원의 effective combat profile을 만든 뒤 destination
  zone kind와 `deployment_affinity`를 비교한다. weapon profile이 affinity를 공급하기만 하면 기존
  deployment validator를 그대로 이용할 수 있다.
- 모든 Weapon equipment는 `deny_unknown_fields`인 `WeaponCombatProfile`을 반드시 가져야 하며 loader는
  weapon의 profile 누락과 non-weapon의 profile 보유를 이미 거부한다. `block_capacity`도 기본값 없이
  필수 authoring한다.
- 장착 weapon의 `WeaponCombatProfile`은 직원의 effective basic attack과 base `block_capacity`로 이미
  복사되고, inventory/roster snapshot에는 원본 및 effective weapon profile이 그대로 직렬화된다.
- BattleCore의 `RuntimeUnit`은 정적 `block_capacity`와 incoming damage modifier를 소유하며,
  `PlatformOnly` profile은 생성 시 block capacity를 0으로 만드는 제한된 기반이 있다.
- Game server는 Core의 typed snapshot/update를 별도 weapon schema로 재선언하지 않고 generic JSON과
  battle side message로 전달한다. 새 command나 10번 전용 server message는 필요하지 않다.

정책과 일치하는 부분:

- 현재 기본 저지력은 이미 직원 stat이나 armor가 아니라 개별 weapon profile이 소유한다. live weapon
  29개 모두 `block_capacity`를 명시한다.
- 현재 live 기준은 Sword 9개와 Spear 1개가 1, Shield 1개가 3, Bow 1개·Gun 1개·Staff 16개가 0이다.
  따라서 Ground 배치 가능 weapon의 base block 0을 허용하는 확정 정책과 기존 Bow/Gun 의미를 그대로
  유지할 수 있다.
- `WeaponCombatProfile` 전체가 inventory와 roster DTO에 이미 노출되므로 새 profile field도 같은
  projection 경계를 이용할 수 있다.
- skill fragment의 `block_capacity_min`과 certified combat power는 현재 effective stance 수치가 아니라
  직원 combat profile의 base `block_capacity`를 읽는다. 이를 base weapon eligibility/power로 유지하면
  Defense 전환 여부에 따라 안전지대 장착 가능성과 인증 전투력이 변하는 불안정성을 만들지 않는다.

현재 정책과 다른 코드·data:

- `WeaponCombatProfile`에는 `deployment_affinity`, `DefenseFocus`, `Rest`가 없다. 현재 live RON 29개도
  이 값을 authoring하지 않는다.
- 장착 weapon을 직원 profile에 적용하는 `apply_weapon_profile`은 base block과 공격 속성만 복사하며
  `deployment_affinity`는 복사하지 않는다. 따라서 현재 모든 employee는 weapon과 무관하게 기본
  `GroundOnly`로 남고 Platform에 배치 가능한 live 직원이 없다.
- RuntimeUnit은 weapon profile, deployment affinity와 current deployment tile kind를 보존하지 않는다.
  생성 helper는 `PlatformOnly`인지로만 block 0을 결정하므로 `Any` weapon을 실제 Platform에 배치했을 때
  effective block 0을 보장할 수 없다. 실제 tile kind와 profile 활성 선택은 12번 runtime seam이 필요하다.
- `DefenseFocus`와 `Rest` 수치를 BattleCore까지 전달하거나 활성 profile을 선택하는 경로가 없다. 수치
  적용 자체는 각각 8번과 9번 구현 범위다.
- 현재 live test는 block 값을 개별 weapon 계약이 아니라 archetype별 고정 guideline으로 단정한다. 새
  정책에서는 값이 각 weapon의 명시 authority이므로 affinity/profile 조합과 weapon별 승인값을 검증하는
  exact catalog test로 교체해야 한다.
- `OwnedEquipment`는 definition ID만 저장하지 않고 weapon metadata 전체를 active-run checkpoint에
  직렬화한다. 새 필드를 strict required field로 추가하면 기존 `ActiveRunCorePayloadV2`는 역직렬화되지
  않는다. 현재 Core와 server persistence는 미지원 version이나 누락 required field를 자동 migration하지
  않고 명시적으로 거부한다.

필요한 구현 범위:

- weapon data schema에 required `deployment_affinity`, strict `DefenseFocusProfile`과 `RestProfile`을
  추가하고 affinity/profile 존재 조합, `PlatformOnly + block_capacity: 0`, 피해 감소율 `0..=100`, block
  합성 overflow를 검증한다. production RON에는 serde default나 archetype fallback을 두지 않는다.
- `UnitCombatProfile::apply_weapon_profile`이 weapon affinity와 두 profile을 effective combat profile에
  전달하게 한다. test fixture의 Rust `Default`는 유효한 명시 조합을 만들되 production RON 누락 허용과
  혼동하지 않는다.
- 기존 29개 live weapon과 11번에서 확정한 ZAYIN `standard_training_sword`를 함께 migration하고 exact
  catalog test로 30개 개수, ID별 affinity, base block, profile 존재와 수치를 고정한다. reward pool이
  equipment type과 rarity로 전체 catalog를 선택하므로 placeholder weapon 22개도 생략할 수 없다.
- inventory/roster snapshot의 확장된 weapon profile shape와 effective affinity를 Core serialization test로
  고정한다. server는 generic mapping을 유지하되 state snapshot/battle update가 확장 shape를 손실하지
  않는 focused contract test를 둔다.
- Battle runtime에는 8·9번이 소비할 immutable weapon tactical profile을 전달한다. actual tile kind 보존과
  `Any`의 Platform effective block/profile 전환은 12번에서 구현하며, 10번 첫 live data에는 `Any`를
  authoring하지 않아 미구현 조합을 live로 노출하지 않는다.
- `block_capacity_min` fragment compatibility와 certified combat power는 첫 구현에서 base
  `block_capacity`만 계속 사용한다. 태세별 수치를 단순 가산하는 별도 power weight는 만들지 않는다.
- strict cutover를 택하면 active-run core payload version을 올리고 이전 V2 continue를 명시적으로
  거부한다. 기존 run 보존을 요구하면 별도 V2→신규 version migration에서 저장된 weapon metadata를
  current definition ID와 승인된 live profile로 재결합하고 migration/rollback test를 추가해야 한다.

신규 정책의 runtime 변경 가치:

- affinity와 두 defense profile을 weapon RON 한 곳에 모으면 장비 교체가 배치 capability와 방어 태세
  의미를 함께 바꾸며 snapshot에도 같은 source가 노출된다. 직원 stat·archetype·runtime fallback을
  병렬 authority로 두는 것보다 content authoring과 검증이 명확하다.
- 현재 world deployment validator, effective combat profile과 snapshot projection을 재사용할 수 있어
  schema 확장 이상의 별도 command/server 기능은 필요하지 않다.
- 반면 첫 release에 `Any` weapon을 억지로 추가하거나 rarity별 DefenseFocus/Rest 배율, 태세 효과용
  certified-power 공식을 함께 도입하는 것은 검증 가능한 live 근거보다 변경 폭이 크다. 첫 migration은
  기존 affinity 의미를 보존하는 단순 catalog와 공통 초기 수치로 제한하는 편이 낫다.

live affinity와 tuning 확정 결과:

- Staff 16개: `PlatformOnly`, base block 0, `Rest(HP 3/tick, 공명 5/tick)`. Paradise Lost와 reward
  pool의 Staff placeholder 15개를 모두 같은 첫 baseline으로 둔다.
- Bow 1개와 Gun 1개: `GroundOnly`, base block 0, `DefenseFocus(block +0, 받는 피해 감소 10%)`. Defense
  중에도 적을 막지 않는 Ground 원거리 역할을 보존한다.
- Sword 9개와 Spear 1개: `GroundOnly`, base block 1,
  `DefenseFocus(block +1, 받는 피해 감소 10%)`.
- 11번 starter 진행 보장용 ZAYIN `standard_training_sword` 1개도 같은 Sword baseline인 `GroundOnly`,
  base block 1, `DefenseFocus(block +1, 받는 피해 감소 10%)`를 사용한다. Physical/DefaultForward, range 1,
  interval 1,500ms, windup 200ms, Instant이며 별도 ability/triggered effect는 두지 않는다.
- Shield 1개: `GroundOnly`, base block 3,
  `DefenseFocus(block +1, 받는 피해 감소 10%)`.
- 첫 live catalog에는 `Any` weapon을 두지 않는다. 12번의 actual tile-kind runtime이 구현되고 실제 dual
  placement content가 필요할 때 개별 weapon을 추가 또는 전환한다.
- 위 표는 archetype fallback 규칙이 아니라 이번 30개 RON에 각 값을 반복해서 명시하는 migration
  guideline이다. 이후 weapon별 값은 독립적으로 조정할 수 있다.
- 공통 10% 피해 감소는 현재 일반 corroded enemy의 attack 8~15와 1,200~1,900ms 공격 간격에서 과도한
  무적 계층을 만들지 않는 보수적 시작점이다. block +1은 기존 blocker에게만 한 명의 추가 저지를
  제공한다. Staff의 3 HP/s는 starter HP 180 기준 약 1.7%/s이고 공명 5/s는 0에서 20초에 채우므로 9번
  확정 guideline과 현재 전투 시간에 맞는다.
- rarity별 배율은 두지 않는다. 현재 rarity 차이는 공격 stat과 triggered effect에 이미 반영되고,
  placeholder를 포함한 전 catalog에 새 배율까지 넣을 live 근거가 없다.

난이도 판정:

- **기존 active-run을 보존하지 않는 strict version cutover 기준 현재 tree `3/5 — 중간`**이다. 새
  command는 없지만 Core data schema, effective profile, Battle runtime carrier, inventory/roster DTO,
  active-run version과 30개 live RON/test를 함께 바꾸는 집중된 vertical slice다.
- 기존 V2 active-run을 계속할 수 있어야 한다면 **현재 tree `4/5 — 높음`**이다. 암묵적 serde default가
  아니라 저장된 metadata를 current definition과 재결합하는 명시 migration, server persistence 복구와
  rollback test가 추가되어 Core/server 저장 경계를 가로지른다.
- 6번 stance와 12번 actual tile-kind state가 먼저 구현되어 weapon profile을 보존·선택하는 runtime seam이
  이미 있다면 strict cutover 기준 10번 증분은 `2/5 — 낮음`이다. schema/validation, live migration과
  projection test가 중심이 된다.
- 8·9번 효과 적용 비용은 각 번호에 이미 책정했으므로 10번 난이도에 중복 계산하지 않는다. 반대로
  10번을 먼저 구현하면 문서에 기록된 8·9번의 선행 후 증분 난이도가 낮아진다.

난이도 책정 reference 파일:

- Core command/wire:
  - `core_enhanced/src/game/behavior.rs`: roster와 equipped item DTO가 `WeaponCombatProfile` 및 effective
    affinity를 직접 직렬화하는 현재 wire 경계이며 10번 전용 command가 필요 없음을 확인했다.
- Core world transaction/state:
  - `core_enhanced/src/game/world/combat.rs`: effective affinity와 destination deployment zone kind를
    대조하는 현재 deploy validation 경계다.
  - `core_enhanced/src/game/world/snapshot.rs`: 장착 weapon profile과 effective combat profile을 roster
    snapshot에 투영한다.
  - `core_enhanced/src/game/world/active_run.rs`: strict versioned active-run core payload export/continue를
    소유해 weapon metadata schema 변경의 저장 version 비용을 결정한다.
  - `core_enhanced/src/game/resources/inventory.rs`: `OwnedEquipment`가 전체 metadata를 저장하고 inventory
    DTO가 weapon profile을 노출하므로 save와 wire가 모두 새 field의 소비자임을 확인했다.
- Battle runtime:
  - `core_enhanced/src/game/battle/types.rs`: employee 기본 affinity와 장착 weapon의 attack/base block을
    effective `UnitCombatProfile`로 조립하는 중심이다.
  - `core_enhanced/src/game/combat_setup/player_spawns.rs`: roster/loadout을 battle draft로 바꾸는 경계로,
    effective weapon profile이 scenario spawn으로 진입하는 지점이다.
  - `core_enhanced/src/game/battle/core/build.rs`: combat profile을 `RuntimeUnit`으로 복사하며 현재는
    `PlatformOnly`만 보고 runtime block을 0으로 만든다.
  - `core_enhanced/src/game/battle/core/types.rs`: 현재 RuntimeUnit에 weapon profile, affinity와 tile kind가
    없어 8·9·12번이 사용할 carrier가 필요함을 확인했다.
- Policy/live data:
  - `core_enhanced/src/game/data/equipment_data.rs`: strict `WeaponCombatProfile`, weapon/non-weapon 조합과
    runtime profile validation을 소유한다.
  - `core_enhanced/src/game/data/live_ron.rs`: embedded equipment RON의 parse/typed domain validation과
    required block field 누락 거부 test를 소유한다.
  - `game_resources/data/equipments/base.ron`: 현재 실제 29개 weapon과 11번에서 추가하기로 확정한 starter
    weapon의 archetype, base block, 공격 수치와 triggered effect가 있는 migration 대상이다.
  - `game_resources/data/events/rewards/base.ron`: 빈 explicit entry 대신 equipment type/rarity filter로
    전체 catalog를 reward pool에 포함하므로 placeholder weapon도 live 획득 가능함을 확인했다.
  - `game_resources/data/enemies/corroded_employees.ron`: 현재 일반 적의 attack 8~15와 공격 간격
    1,200~1,900ms를 명시해 10% 피해 감소 제안의 전투 시간 기준이 됐다.
  - `core_enhanced/src/game/data/skill_fragment_data.rs`: fragment의 weapon archetype 및 base block
    compatibility가 새 profile의 간접 소비자다.
  - `core_enhanced/src/game/combat_power.rs`: certified power가 현재 base block을 가중하며 태세별 profile을
    단순 가산하지 않는다는 범위 판단 근거다.
- Game server transport:
  - `game_server/src/game/player_game_actor/state.rs`: Core result와 snapshot payload를 generic server
    response/side message로 변환해 10번 전용 분기가 필요 없음을 확인했다.
  - `game_server/src/game/player_game_actor/messages.rs`: Core battle setup/update DTO를 server message로
    투영하며 별도 weapon DTO를 재선언하지 않는다.
  - `game_server/src/persistence/active_run_store.rs`: outer active-run envelope의 strict version/checksum,
    backup과 복구를 소유하며 미지원 version을 자동 migration하지 않는다.
- 외부 Unity runtime/contract/tests:
  - 사용자 결정에 따라 조사하지 않았고 10번 난이도에도 Unity 구현 비용을 별도로 더하지 않았다.

실행한 focused test와 결과:

- `cargo test -p game_core weapon` — 12 passed. strict weapon profile 누락 거부, 장착 weapon의 base block
  authority, snapshot/skill-fragment의 weapon profile 소비와 현재 live block guideline 기반을 확인했다.
- `cargo test -p game_core platform_only_spawned_unit_cannot_block_even_if_profile_has_capacity` — 1 passed.
  현재 `PlatformOnly` runtime block 0 기반을 확인했다.
- `cargo test -p game_core battlefield_template_exposes_ground_and_platform_deployment_zones` — 1 passed.
  deployment zone kind 구분 기반을 확인했다.
- `cargo test -p game_core employee_roster_snapshot_exposes_gameplay_status_and_loadout_without_display_slot` —
  1 passed. effective deployment affinity의 기존 roster projection 기반을 확인했다.
- `cargo test -p game_core typed_monolith_stage_round_trips_checkpoint_continue_and_transaction_rollback` —
  1 passed. active-run payload가 strict required field와 versioned round-trip을 사용하고 누락 field를 자동
  보완하지 않는 기반을 확인했다.
- `cargo test -p game_server behavior_result_to_command_result_uses_battle_update_side_message` — 1 passed.
  generic server update mapping 기반을 확인했다.
- `cargo test -p game_server unsupported_envelope_version_is_not_silently_migrated` — 1 passed. server
  persistence가 미지원 저장 version을 암묵적으로 변환하지 않는 기반을 확인했다.

이 test 통과는 10번 affinity/profile schema나 live migration이 구현됐다는 증거가 아니라 현재 strict
weapon data, 배치 zone, profile projection, runtime block과 저장/transport 기반이 안정적이라는 증거다.
새 affinity/profile 조합, 30개 승인값과 저장 version cutover test는 현재 없다.

난이도·tuning 확정 결과:

- 구현 대상 live catalog는 Staff 16개 `PlatformOnly + Rest 3/5`, starter 훈련검을 포함한 나머지 14개
  `GroundOnly`, base blocker만
  `DefenseFocus block +1`, Ground weapon 공통 피해 감소 10%, 첫 `Any` 없음으로 확정했다.
- fragment `block_capacity_min`과 certified combat power는 base block만 계속 사용하고 태세 profile 수치를
  첫 구현 power model에 넣지 않는 범위 축소안으로 확정했다.
- 기존 `ActiveRunCorePayloadV2`는 보존하거나 암묵적으로 보완하지 않고 strict version cutover한다. 이에
  따라 현재 tree 난이도 `3/5`, 6·12번 선행 후 증분 `2/5`로 확정했다. 기존 V2를 명시 migration하는
  `4/5` 대안은 채택하지 않았다.

### 11. 무기 없는 직원 — 정책·코드 대조·난이도 책정 완료

확정일: 2026-08-06  
구현 난이도: 현재 tree `2/5 — 낮음` 확정

확정 정책:

- 무기 없는 직원도 직원 roster와 전투 편성 후보 목록에는 계속 표시한다.
- 안전지대에서는 정보 확인, 장비 관리와 무기 장착 등 일반 직원 상호작용을 모두 허용한다.
- 전투 편성 및 active battle에서는 정보 표시를 제외한 gameplay 상호작용을 허용하지 않는다. 최초
  편성·배치, 철수 후 재배치, 전술 이동·자리 교대, 태세 전환, 배치·공격 범위 preview, 수동 스킬과
  현장 작업의 source가 될 수 없다.
- 전투 UI와 Core의 전투 eligibility에는 `weapon_required` 사유를 사용한다. Server도 관련 전투 요청을
  같은 typed reason으로 거부하며 자원·cooldown·event·revision을 변경하지 않는다.
- 직원 기본 능력치, 무기 archetype 추론이나 임시 weapon profile로 공격, deployment affinity, 저지력,
  `DefenseFocus` 또는 `Rest`를 보충하지 않는다.
- 안전지대에서 무기를 장착하면 해당 weapon RON을 기준으로 전투 eligibility와 profile을 다시 계산한다.
- 배치 가능한 무장 직원이 한 명도 없다면 기존 `배치 가능한 직원 없음` 규칙을 적용한다.
- 직원 자체를 비활성화하는 정책이 아니다. 무기를 장착하기 전까지 전투 gameplay에만 참여할 수 없는
  상태이며, 안전지대 관리와 정보 확인은 복구 가능한 정상 흐름으로 유지한다.

코드·live data 대조 결과:

1. 확정 정책
   - 위 정책처럼 직원 roster와 안전지대 상호작용은 유지하고, 무기 없는 직원의 전투 gameplay만
     `weapon_required`로 비활성화한다. 임시 기본 공격이나 weapon profile fallback은 두지 않는다.
2. 현재 구현된 기반
   - `EmployeeCondition::eligibility`와 `EmployeeRoster::combat_eligible_employee_ids`가 combat eligibility의
     중심 seam이다. 현재는 사망·치명상·breakdown만 검사하며 employee의 item slot은 이미 장착품의
     `EquipmentType`을 보유한다.
   - live deployment DTO에는 직원별 `BattlePayloadEligibilityDto`가 있고 Core/server에는
     `ActionUnavailable`의 typed disabled reason 전달 경로가 있다.
   - 안전지대 장비 장착·해제는 기존 loadout context와 staged transaction을 사용한다. 배치 가능한 직원이
     없을 때의 `NoDeployableEmployees` recovery/failure 흐름과 certified combat power의 combat-eligible
     filter도 이미 존재한다.
3. 정책과 일치하는 부분
   - 전체 employee roster와 별도의 전투 가능 ID/배치 후보 projection이 있어 직원을 roster에서 제거하지
     않고 전투 후보만 disabled로 표현할 수 있다.
   - 장비 변경은 active battle 밖의 안전지대 context에서만 허용되므로, 무기 장착으로 eligibility를
     복구하는 정책과 일치한다.
   - server는 `ActionDisabledReasonCode::as_str()`을 일반적으로 전달하므로 새 reason에 별도 전송 분기를
     만들 필요가 없다. 기존 no-deployable 판정도 중앙 combat eligibility를 재사용할 수 있다.
4. 정책과 다른 부분
   - 현재 `EmployeeIneligibilityReason`과 `ActionDisabledReasonCode`에는 `weapon_required`가 없다.
     combat eligibility는 장착 무기를 검사하지 않는다.
   - `UnitCombatProfile::for_employee`가 무기 없는 직원에게도 GroundOnly, 근접 기본 공격, block capacity 0인
     완전한 fallback profile을 만들며 기존 test가 이 동작을 성공 조건으로 고정한다.
   - deploy와 deployment range preview는 condition eligibility만 통과하면 그 fallback draft/profile을
     사용한다. live deployment 후보도 condition-eligible 직원만 filter한 뒤 배치 여부·cooldown·비용만
     검사하므로 unarmed disabled 상태를 표현하지 않는다.
   - certified combat power와 map readiness/no-deployable 판정도 현재는 무기 없는 직원을 전투 가능 인원으로
     계산한다.
   - 가장 큰 live data 충돌은 starter-eligible 직원 6명의 초기 장비가 모두 `standard_armor`뿐이라는
     점이다. 시작 시 3명을 선택하지만 어느 조합도 무기가 없으므로, 이 정책만 구현하면 신규 run은 첫
     전투부터 배치 가능한 직원이 0명이 된다. 현재 starter validation은 장비가 하나 이상인지만 검사하고
     weapon을 요구하지 않으며 HQ/unknown shop도 weapon을 보장하지 않는다.
5. 필요한 구현 범위
   - `EmployeeIneligibilityReason`과 `ActionDisabledReasonCode`에 명시적 `weapon_required`를 추가한다.
     combat eligibility는 기존 condition 사유를 먼저 판정한 뒤 item slot에 weapon이 없으면 이 사유를
     반환한다. Loadout/Event 등 비전투 eligibility는 바꾸지 않는다.
   - roster combat IDs, map readiness, no-deployable 판정과 certified combat power가 이 중앙 eligibility를
     재사용하게 한다. 장비 reference가 stale/missing인 invalid data는 unarmed로 완화하지 않고 기존 data
     error로 유지한다.
   - live deployment DTO는 condition상 전투 가능한 unarmed 직원도 후보 목록에 남기되 eligibility만
     `weapon_required`로 만든다. deploy와 range preview handler도 draft/profile 생성 전에 같은 검사를
     수행해 typed reason으로 원자적으로 거부한다. 4·6·14·16·17번의 신규 전투 command도 같은 source
     eligibility helper를 재사용해야 한다.
   - BattleCore에는 unarmed unit을 spawn하지 않으므로 별도 unarmed runtime state나 기본 공격을 추가하지
     않는다. gameplay mutation 전에 거부하므로 battle resource, cooldown, event, revision도 변하지 않는다.
   - 신규 run의 진행 가능성을 보장하려면 starter-eligible 정의 모두에 weapon을 지급하고 live validation도
     `starter_eligible` loadout에 weapon이 있음을 강제해야 한다. 권고안은 보상 pool의 TETH~ALEPH 추첨에
     섞이지 않는 ZAYIN `standard_training_sword` 하나를 추가해 6명 모두에게 지급하는 것이다.
6. 현재 tree 기준 난이도와 이유
   - 제안 난이도는 `2/5 — 낮음`이다. 기존 eligibility, item slot, 후보별 disabled reason, safe-zone loadout,
     no-deployable과 server error seam을 그대로 사용할 수 있다. Core eligibility/DTO/두 handler와 live
     starter data·validation, focused test를 함께 바꾸는 제한된 vertical slice이며 BattleCore의 combat
     state machine이나 server transport protocol을 새로 설계하지 않는다.
7. 선행 정책 구현 후 난이도 차이
   - 10번이 먼저 구현돼도 11번 전체는 `2/5`로 본다. weapon profile의 단일 소유권은 더 선명해지지만
     eligibility/DTO/starter data 변경은 여전히 필요하기 때문이다.
   - 4·6·14·16·17번이 공통 전투 source validator를 먼저 만들었다면 각 신규 command에 무기 gate를 넣는
     증분은 `1/5`에 가까워진다. 그러나 starter 보장과 기존 deploy/preview/snapshot을 포함한 11번 완성
     vertical slice의 등급은 `2/5`에서 내려가지 않는다.
8. 난이도 책정 reference 파일과 책임
   - Core command/wire
     - `core_enhanced/src/game/behavior.rs`: action disabled reason, employee eligibility snapshot, roster와 live
       deployment candidate DTO의 wire shape를 소유한다.
   - Core world transaction/state
     - `core_enhanced/src/game/world/combat.rs`: deploy·range preview preflight, no-deployable run 판정과 실패 시
       mutation 경계를 소유한다.
     - `core_enhanced/src/game/world/state.rs`: live deployment 후보와 직원별 비용/cooldown/resource eligibility
       projection을 만든다.
     - `core_enhanced/src/game/world/snapshot.rs`: employee roster와 active battle deployment snapshot을 외부
       DTO로 투영한다.
     - `core_enhanced/src/game/world/battle_capability.rs`: deployment DTO를 바탕으로 deploy/preview action
       capability를 계산한다.
     - `core_enhanced/src/game/world/helpers.rs`: 안전지대 loadout 검증과 장비 변경 허용 context를 소유한다.
     - `core_enhanced/src/game/world/maintenance.rs`: maintenance node의 장비 변경을 staged transaction으로
       처리한다.
     - `core_enhanced/src/game/world/node_flow.rs`: certified combat power와 combat node 진입 준비도를 연결한다.
   - Battle runtime
     - `core_enhanced/src/game/combat_setup/player_spawns.rs`: employee loadout에서 BattleUnitDraft와 effective
       combat profile을 구성하고 현재 condition-only combat eligibility를 검사한다.
     - `core_enhanced/src/game/battle/types.rs`: 무기가 없을 때도 만들어지는 employee fallback profile과
       장착 weapon profile의 적용 경계를 소유한다.
   - Policy/live data
     - `core_enhanced/src/game/employee.rs`: employee activity eligibility, item slot을 포함한 loadout과 combat
       eligible roster ID를 소유한다.
     - `core_enhanced/src/game/combat_power.rs`: combat-eligible 직원의 certified power 합산 대상을 결정한다.
     - `core_enhanced/src/game/data/validation.rs`: starter loadout reference와 slot 충돌을 검증하지만 현재
       starter weapon 존재는 검사하지 않는다.
     - `game_resources/data/employees/definitions.ron`: starter-eligible 6명은 현재 armor만, HQ recruit 6명은
       빈 loadout으로 authoring돼 있다.
     - `game_resources/data/equipments/base.ron`: live weapon catalog와 새 starter weapon이 들어갈 장비
       authoring source다.
     - `game_resources/data/events/shops/base.ron`: 현재 안전지대 상점이 weapon을 보장하지 않는다는 것을
       확인한 shop pool이다.
     - `game_resources/data/events/rewards/base.ron`: equipment reward가 TETH~ALEPH만 추첨하므로 권고 ZAYIN
       starter weapon을 일반 보상 pool에서 분리할 수 있다.
     - `game_resources/data/run/policy.ron`: 신규 run이 starter 6명 중 3명을 선택한다는 setup 수치를
       소유한다.
   - Game server transport
     - `game_server/src/game/player_game_actor/state.rs`: `GameError::ActionUnavailable`의 typed reason을 별도
       command 분기 없이 server error code로 보존한다.
   - 외부 Unity runtime/contract/tests
     - 사용자 결정에 따라 7번부터 Unity 조사를 제외했으며 11번 난이도에도 Unity 구현 비용을 포함하지
       않았다.
9. 실행한 focused test와 결과
   - `cargo test -p game_core start_new_game_creates_starter_employee_roster_and_equipment` — 통과(1개). 현재
     starter roster와 armor-only 초기 장비 baseline을 확인했다.
   - `cargo test -p game_core employee_default_profile_is_ground_blockable_but_unarmed_capacity_is_zero` —
     통과(1개). 현재 unarmed fallback combat profile이 의도적으로 존재함을 확인했다.
   - `cargo test -p game_core equipped_weapon_authors_block_capacity_for_every_current_archetype` — 통과(1개).
     장착 weapon profile이 현재 block capacity를 덮어쓰는 seam을 확인했다.
   - `cargo test -p game_core employee_roster_snapshot_exposes_gameplay_status_and_loadout_without_display_slot` —
     통과(1개). unarmed starter가 현재 combat eligible로 투영되고 armor만 노출되는 baseline을 확인했다.
   - `cargo test -p game_core combat_map_node_fails_run_when_no_employee_can_be_deployed` — 통과(1개). 기존
     no-deployable recovery/failure 흐름을 확인했다.
   - `cargo test -p game_core unequip_item_allows_unbound_equipment_in_safezone` — 통과(1개). 안전지대 loadout
     복구 seam을 확인했다.
   - `cargo test -p game_core certification_is_deterministic_and_ignores_eligible_condition_magnitudes_and_current_hp`
     — 통과(1개). certified power가 현재 condition-only eligibility와 unarmed profile을 사용함을 확인했다.
   - `cargo test -p game_server action_unavailable_preserves_typed_disabled_reason_code` — 통과(1개). server가
     typed disabled reason을 일반 경로로 보존함을 확인했다.
   - 이 통과 결과는 기존 기반과 충돌 지점을 안정적으로 재현한 증거일 뿐, 11번이 구현됐다는 증거가
     아니다.
10. 난이도·tuning과 선택 확정 결과
    - 신규 run softlock을 막기 위해 starter-eligible 6명 모두에게 동일한 ZAYIN
      `standard_training_sword`를 지급하고 starter validation이 weapon을 요구하게 한다. 이에 맞춰 10번의
      구현 대상 live weapon catalog도 29개에서 30개로 갱신했다.
    - 첫 profile은 10번의 Sword baseline에 맞춘 `GroundOnly`, Sword, base block 1,
      `DefenseFocus block +1 / incoming damage reduction 10%`, Physical/DefaultForward, range 1,
      interval 1,500ms, windup 200ms, Instant다. 이 무기는 starter 진행 보장용이므로 별도 능력과 triggered
      effect는 두지 않고 ZAYIN으로 일반 equipment reward pool에서 제외한다. 이름과 가격은 live content
      tuning이며 전투 의미에는 영향을 주지 않는다.
    - 동시에 여러 비활성 사유가 있으면 기존 condition 사유(사망·치명상·breakdown)를 먼저 보여주고,
      condition상 전투 가능할 때만 `weapon_required`를 보여준다.
    - 현재 tree 난이도는 `2/5 — 낮음`으로 확정했다. 별도 비용·cooldown 같은 battle-time tuning은
      11번에 없다.

### 12. Ground/Platform 순간이동 — 정책·코드 대조·난이도 책정 완료

확정일: 2026-08-06  
구현 난이도: 현재 tree `4/5 — 높음`, 4·10번 후 `3/5 — 중간`,
4·6·8·9·10·14번 후 증분 `2/5 — 낮음` 확정

확정 정책:

- 별도의 `Ground/Platform 순간이동` 버튼, command, 비용 또는 cooldown을 만들지 않는다. 12번은 4번
  전술 순간이동과 14번 자리 교대에 적용되는 destination affinity 및 tile effect 전환 규칙이다.
- 개별 weapon RON의 `deployment_affinity`가 배치 가능한 destination을 결정한다. `GroundOnly`는 Ground,
  `PlatformOnly`는 Platform만 허용하고 `Any`만 두 tile kind를 모두 허용한다. 따라서 Ground↔Platform
  전환은 `Any` weapon을 장착한 직원만 가능하다.
- 실제 current deployment tile kind가 `Defense`의 활성 profile을 선택한다. Ground에서는 해당 weapon의
  `DefenseFocus`, Platform에서는 해당 weapon의 `Rest`만 활성화하며 둘을 동시에 적용하지 않는다.
- Ground↔Platform 전환은 하나의 authoritative mutation 안에서 이전 tile effect를 제거하고 새 tile
  effect를 즉시 적용한다. 중간 state, 효과 중첩 또는 효과 없는 별도 transition tick은 만들지 않는다.
- 전술 순간이동의 태세·HP·공명·buff/debuff·공격/skill cooldown·runtime identity 보존은 4번 정책을
  그대로 따른다. 자리 교대도 양쪽 unit-owned state를 보존하고 위치·방향 및 spatial derived state만
  원자적으로 다시 계산한다.
- 실제 Platform tile에서는 `Any` weapon의 authored base block도 effective block capacity 0으로 처리한다.
  Ground로 전환하면 현재 태세와 weapon profile에 맞춰 effective block과 engagement를 같은 mutation에서
  다시 판정한다.
- affinity가 맞지 않는 destination은 gameplay mutation 전에 거부해 resource, cooldown, event와 revision을
  변경하지 않는다. 점유, obstacle, deployment zone, stance와 진행 중 행동 검사는 4·5·7·14번의 공통
  규칙을 그대로 사용한다.
- 전술 순간이동은 4번의 비용과 직원별 cooldown을 사용하고 자리 교대는 14번에서 확정할 비용·cooldown
  규칙을 사용한다. 12번은 독립된 balance tuning을 소유하지 않는다.
- 첫 live catalog에는 `Any` weapon을 두지 않는다. 첫 구현에서 실제 Ground↔Platform player content는
  없지만 Ground 직원과 Platform 직원의 current tile kind 및 활성 defense profile selector는 구현한다.
  향후 개별 `Any` weapon을 추가할 때 runtime 구조를 다시 설계하지 않고 같은 규칙을 사용한다.
- weapon archetype, 직원 능력치나 누락 profile에서 affinity 또는 tile effect를 추론하는 fallback은 두지
  않는다.

코드·live data 대조 결과:

1. 확정 정책
   - 12번은 독립 command가 아니다. 4번 전술 순간이동과 14번 자리 교대가 destination을 바꿀 때 weapon
     affinity를 검사하고, 실제 Ground/Platform tile에 맞는 defense profile과 effective block을 같은
     mutation에서 전환하는 공통 runtime 규칙이다.
2. 현재 구현된 기반
   - `DeploymentAffinity`에는 이미 `GroundOnly | PlatformOnly | Any`와 Ground/Platform 허용 helper가 있다.
     `DeploymentZoneKind::supports_affinity`와 World의 `validate_employee_deployment_cell`도 최초 배치
     destination cell의 zone kind와 employee combat profile을 비교한다.
   - combat preview와 live battlefield template은 Ground/Platform deployment zone 및 각 tile kind를
     명시한다. ActiveBattleSession은 원본 combat preview를 계속 보존하고 battle setup transport도 zone을
     노출한다.
   - RuntimeUnit은 world position, projected tile, facing과 effective block/damage modifier를 갖고 있으며
     blocking refresh는 stable ordering으로 engagement를 다시 만들 수 있다. Server는 Core setup/update를
     별도 zone DTO 재선언 없이 전달한다.
3. 정책과 일치하는 부분
   - 최초 deploy의 zone/affinity preflight는 이미 확정 정책과 같은 방향이다. `Any`도 enum과
     `supports_affinity` 수준에서는 두 zone을 허용한다.
   - BattleCore의 low-level position mutation은 runtime identity와 HP·공명·buff/cooldown을 새 instance로
     교체하지 않으므로 4번 state 보존 의미에 재사용할 수 있다.
   - current position은 RuntimeUnit body가 authority이고 checkpoint도 body에서 position을 투영하므로,
     원자적 relocation 뒤 spatial projection을 한 곳에서 갱신할 기반이 있다.
4. 정책과 다른 부분
   - World의 Ground/Platform 정보는 BattleCore 진입 전에 사라진다. `BattleStartPlan`과
     `BattlefieldLayout`은 dimensions, valid tiles와 obstacle만 보존하고 deployment zone/tile kind는
     전달하지 않는다.
   - `WeaponCombatProfile`은 아직 affinity와 DefenseFocus/Rest profile을 갖지 않으며
     `apply_weapon_profile`도 affinity를 복사하지 않는다. 따라서 모든 live employee profile은 현재
     fallback `GroundOnly`이고 Platform에 배치 가능한 live 직원이나 `Any` weapon이 없다.
   - RuntimeUnit은 weapon tactical profile, authored base block, deployment affinity, current deployment
     tile kind와 stance를 보존하지 않는다. 현재 spawn helper는 실제 tile이 아니라
     `deployment_affinity == PlatformOnly`인지만 보고 block을 0으로 만든다. 따라서 `Any` profile을
     Platform cell에 놓으면 authored block이 그대로 남는 정책 불일치가 생긴다.
   - 공식 `BattleLiveCommand`에는 deploy/withdraw/manual skill만 있으며 4번 relocation과 14번 swap이 없다.
     Ground↔Platform 전환과 양쪽 destination kind를 한 mutation에서 적용할 command seam도 아직 없다.
   - checkpoint unit/deployment DTO는 position과 facing만 보존하고 current tile kind, stance와 active
     DefenseFocus/Rest를 노출하지 않는다. 8·9번 효과 runtime도 아직 구현되지 않았다.
   - blocking refresh는 capacity가 0이면 기존 engagement를 해제할 수 있지만, 위치가 바뀐 blocker의 기존
     engagement를 거리와 무관하게 먼저 보존한다. 4번에서 확정한 relocation 전 명시 release와 이후
     refresh가 함께 필요하다.
5. 필요한 구현 범위
   - combat domain에 Ground/Platform만 표현하는 공통 deployment tile kind를 두고, World가 combat preview의
     destination position에서 kind를 결정한다. World는 position↔zone kind를 검증하고 BattleCore command는
     전달받은 kind가 runtime unit의 authored affinity에 맞는지 다시 검사한다.
   - 전체 `BattlefieldLayout`에 deployment zone map을 복제하지 않는다. active Player combatant의
     RuntimeUnit에 `current_deployment_tile_kind`와 10번의 immutable weapon tactical profile/base block만
     보존하는 것이 권고하는 최소 carrier다. enemy continuous traversal에 Ground/Platform 의미를 새로
     부여하지 않는다.
   - 최초 deploy가 validated kind를 BattleCore에 전달하고, 4번 relocation은 destination kind 하나를,
     14번 swap은 양쪽 destination kind를 같은 command에 전달한다. BattleCore는 affinity와 양쪽 kind를
     모두 검증한 뒤 위치·방향·kind를 함께 commit한다.
   - `effective_defense_profile(stance, current_tile_kind, weapon_profile)` 같은 단일 selector를 두고 8번
     DefenseFocus, 9번 Rest와 Platform effective block 0을 여기서 파생한다. 별도 Ground/Platform stance,
     transition state나 archetype fallback은 만들지 않는다.
   - relocation/swap mutation은 기존 engagement를 먼저 release하고 위치·kind와 effective block/damage/
     recovery profile을 교체한 뒤 deterministic blocking/target/range/area derived state를 재계산한다.
     resource/cooldown/event/revision commit은 4·14번 transaction이 소유한다.
   - checkpoint/resync에는 최소한 player unit의 current deployment tile kind와 6·8·9번이 요구하는 stance/
     active defense profile projection을 보존한다. Battle setup의 zone map은 계속 destination UI source로
     사용한다. Server에는 12번 전용 message나 결과 type을 만들지 않고 기존 setup/update mapping을
     확장한다.
   - 첫 live catalog에는 `Any`를 추가하지 않는다. Rust fixture로 Ground→Platform→Ground와 swap 양방향을
     검증하고, 실제 content 추가는 별도 live tuning으로 남긴다.
6. 현재 tree 기준 난이도와 이유
   - 제안 난이도는 `4/5 — 높음`이다. 12번이 독립 command를 만들지는 않지만 현재 tree에는 그 규칙이
     소비할 4번 relocation, 6번 stance, 8·9번 defense effect, 10번 weapon tactical profile과 14번 swap이
     아직 없다. 완성된 vertical slice는 World destination validation, BattleCore runtime carrier와 원자적
     spatial mutation, effective profile/block 전환, checkpoint/server projection을 가로지른다.
   - 다만 새로운 navigation, 이동 segment, 별도 자원 또는 gameplay scripting 구조는 필요하지 않으므로
     핵심 simulation 재설계인 5/5 범위는 아니다.
7. 선행 정책 구현 후 난이도 차이
   - 4번 relocation과 10번 weapon affinity/profile carrier만 먼저 구현되면 12번 증분은
     `3/5 — 중간`이다. current tile kind와 Platform block은 연결할 수 있지만 stance와 두 defense effect,
     swap 양쪽 원자 전환이 남는다.
   - 4·6·8·9·10·14번이 먼저 구현되어 공통 spatial mutation, stance, weapon profile, 효과 적용과 swap
     transaction이 존재하면 12번 증분은 `2/5 — 낮음`이다. current tile kind carrier, 단일 selector,
     양방향 affinity/effect 전환과 checkpoint focused test가 중심이 된다.
   - 인접 정책 구현 시 이 selector를 각 번호에서 따로 만들지 말고 12번 seam을 공동 구현해야 중복
     비용을 피할 수 있다.
8. 난이도 책정 reference 파일과 책임
   - Core command/wire
     - `core_enhanced/src/game/behavior.rs`: battle setup의 deployment zones, unit/deployment checkpoint와
       현재 deploy command/result wire를 소유하며 tile kind/stance projection이 아직 없음을 확인했다.
   - Core world transaction/state
     - `core_enhanced/src/game/world.rs`: 현재 player behavior dispatch에는 relocation/swap command가 없다.
     - `core_enhanced/src/game/world/combat.rs`: destination cell의 zone kind 조회, employee affinity 검증,
       deploy/preview와 BattleCore transaction 경계를 소유한다.
     - `core_enhanced/src/game/world/state.rs`: ActiveBattleSession의 combat preview와 employee↔runtime mapping,
       body position 기반 deployment/checkpoint projection을 소유한다.
   - Battle runtime
     - `core_enhanced/src/game/battle/types.rs`: `DeploymentAffinity`, employee `UnitCombatProfile`과 weapon
       적용 경계를 소유하지만 weapon affinity/tile profile 전달은 아직 없다.
     - `core_enhanced/src/game/combat_preview/types.rs`: Ground/Platform tile·deployment zone과
       `supports_affinity`를 정의한다.
     - `core_enhanced/src/game/combat_setup/battlefield_plan.rs`: preview를 BattleScenario로 바꿀 때 zone/tile
       kind를 버리고 valid/static 정보만 전달하는 경계다.
     - `core_enhanced/src/game/battle/battlefield/mod.rs`와
       `core_enhanced/src/game/battle/battlefield/field.rs`: BattleCore의 static layout이 valid tile과 obstacle만
       소유해 deployment kind를 조회할 수 없음을 확인했다.
     - `core_enhanced/src/game/battle/core/types.rs`: RuntimeUnit의 body/facing/effective block과 누락된
       affinity/current tile/profile carrier를 확인한 중심 state다.
     - `core_enhanced/src/game/battle/core/build.rs`: combat profile을 RuntimeUnit으로 만들며 현재 실제 tile
       대신 `PlatformOnly` affinity만 보고 runtime block을 0으로 만드는 spawn 경계다.
     - `core_enhanced/src/game/battle/core/sim.rs`: 현재 deploy/withdraw/skill만 있는 live command 적용
       경계로 relocation/swap과 destination kind가 없다.
     - `core_enhanced/src/game/battle/core/movement/blocking.rs`: effective block 변경 뒤 deterministic
       engagement 재구성과 relocation 시 explicit release가 필요한 sticky ordering을 소유한다.
   - Policy/live data
     - `core_enhanced/src/game/data/equipment_data.rs`: 현재 weapon profile schema가 attack/base block만
       소유하고 10번 affinity/DefenseFocus/Rest field가 아직 없는 source다.
     - `game_resources/data/equipments/base.ron`: 현재 29개 weapon에는 affinity/profile authoring이 없고
       확정된 30개 migration 및 첫 `Any` 없음의 대상이다.
     - `game_resources/data/map/battlefield_templates.ron`: `P` Ground와 `T` Platform deployment cell을 실제
       live 전장에 authoring한다.
   - Game server transport
     - `game_server/src/game/player_game_actor/messages.rs`: Core battle setup zone과 update checkpoint를
       top-level/resync message로 전달하며 12번 전용 server DTO가 필요 없음을 확인했다.
     - `game_server/src/game/player_game_actor/state.rs`: Core battle update와 typed error를 일반 server
       경로로 보존한다.
   - 외부 Unity runtime/contract/tests
     - 사용자 결정에 따라 7번부터 Unity 조사를 제외했으며 12번 난이도에도 Unity 구현 비용을 포함하지
       않았다.
9. 실행한 focused test와 결과
   - `cargo test -p game_core battlefield_template_exposes_ground_and_platform_deployment_zones` — 통과(1개).
     preview/live template parser가 두 deployment zone과 tile kind를 구분하는 기반을 확인했다.
   - `cargo test -p game_core platform_only_spawned_unit_cannot_block_even_if_profile_has_capacity` — 통과(1개).
     현재 affinity-only Platform block 0 seam과 실제 tile 미사용 한계를 확인했다.
   - `cargo test -p game_core employee_roster_snapshot_exposes_gameplay_status_and_loadout_without_display_slot` —
     통과(1개). effective deployment affinity가 roster projection에 존재하는 기반을 확인했다.
   - `cargo test -p game_core pending_deployment_range_preview_returns_all_facings_without_mutating_battle` —
     통과(1개). zone/affinity preview preflight가 mutation을 만들지 않는 현재 transaction 기반을 확인했다.
   - `cargo test -p game_server behavior_result_to_command_result_uses_battle_update_side_message` — 통과(1개).
     Core checkpoint를 전달하는 generic server update seam을 확인했다.
   - 이 통과 결과는 현재 zone/affinity/position/transport 기반이 안정적이라는 증거일 뿐, 실제 tile kind
     carrier, `Any` 전환이나 12번 효과 교체가 구현됐다는 증거가 아니다.
10. 난이도·tuning과 구현 축소안 확정 결과
    - 별도 balance tuning은 없다. 전술 순간이동과 자리 교대의 비용/cooldown은 각각 4번과 14번이
      소유하며 첫 `Any` weapon content도 이번 번호에서 추가하지 않는다.
    - 현재 tree 난이도 `4/5`, 4·10번 선행 후 `3/5`, 4·6·8·9·10·14번 선행 후 증분 `2/5`로 확정했다.
    - 구현 시 전체 BattlefieldLayout에 zone map을 복제하지 않고 validated current tile kind를 player
      RuntimeUnit에만 보존하는 축소안을 권고한다. 이는 enemy traversal semantics와 대규모 scenario schema
      변경을 12번 범위에 끌어들이지 않는다.
    - 위 RuntimeUnit 축소안을 확정했다. 첫 `Any` live content가 없는 동안 12번을 고립된 기능으로 먼저
      구현하지 않고 4·6·8·9·10·14번의 공통 runtime seam과 함께 구현한다.

### 13. Hard CC와 Silence — 정책·코드 대조·난이도 책정 완료

확정일: 2026-08-06  
난이도 확정일: 2026-08-06  
구현 난이도: 현재 tree `3/5 — 중간`, 4·6·8·9·12·14·17번 선행 후 증분 `2/5 — 낮음`

확정 정책:

- 개별 command가 `Stun`, `Freeze` 같은 상태 이름을 직접 나열하지 않는다. 활성 효과에서 중앙 파생한
  `hard_cc_active`와 `silenced` capability만 사용한다. 같은 분류의 효과가 여러 개 중첩되면 하나라도
  남아 있는 동안 해당 capability를 유지한다.
- Hard CC는 새로운 기본 공격과 automatic/manual skill 시작을 막는다. Hard CC 적용 시 아직 release되지
  않은 기본 공격 windup과 skill cast/focus는 취소한다. 이미 release된 공격, projectile, delayed step,
  area, buff/debuff와 확정된 파생 효과는 기존 lifecycle을 계속한다.
- Hard CC로 취소된 행동이 이미 소비한 공명이나 시작한 cooldown은 환불·rollback하지 않는다.
- Hard CC 중에는 태세 전환, 전술 순간이동과 자리 교대를 사용할 수 없고 자리 교대의 source나 target도
  될 수 없다. 현재 `Attack/Defense` 태세는 그대로 보존한다.
- Hard CC 중 Ground `DefenseFocus`의 incoming damage 감소는 유지하지만 effective block capacity는 0이
  되고 기존 engagement를 모두 해제한다. Platform `Rest` 회복, 공명 획득과 field work 진행은 중단한다.
  Rest cadence, field work assignment와 누적 progress는 보존한다.
- Hard CC가 끝나면 현재 위치·태세에서 effective block과 자동 행동 eligibility를 다시 계산한다. 이전
  engagement identity나 취소된 행동은 복원하지 않는다. Rest와 field work는 남은 cadence·진행 지점부터
  재개하고 CC 시간을 소급하거나 비축하지 않는다.
- Hard CC는 battle time을 멈추지 않는다. 태세·이동·기본 공격·skill cooldown은 계속 흐르며 전체 battle
  `Pause`만 battle time을 정지한다.
- Hard CC 중에도 피해를 받고 이미 존재하는 buff/debuff 및 passive/reactive `Triggered` effect는
  유지한다. generic passive suppression은 추가하지 않으며 필요한 개별 content만 별도로 authoring한다.
- 3번에서 확정한 자발적 철수는 Hard CC 중에도 허용한다.
- Silence는 automatic/manual을 포함한 새로운 skill 시작만 막는다. 이미 시작된 skill action은 중단하거나
  환불하지 않고 기존 lifecycle대로 완료한다.
- Silence는 기본 공격, 저지, incoming damage 감소, Rest, 공명 획득, field work, 태세 전환, 전술
  순간이동, 자리 교대와 자발적 철수를 막지 않는다. 각 행동의 다른 stance/activity/resource/cooldown/
  destination 조건은 그대로 검증한다.
- Silence 중 공명이 가득 차도 auto skill은 시작하지 않고 현재 공명을 보존한다. Silence가 끝나면 그
  시점의 정상 eligibility를 다시 평가한다.
- Hard CC로 command를 거부하면 `hard_crowd_control`, Silence로 skill 시작을 거부하면 `silenced` typed
  reason을 사용한다. 거부된 command는 resource, cooldown, event와 revision을 변경하지 않는다.
- Hard CC와 Silence가 동시에 적용되면 Hard CC의 전체 행동 제한을 먼저 적용한다. Hard CC가 끝난 뒤
  Silence가 남아 있으면 skill 시작 제한만 계속한다.
- 13번은 별도 자원 비용이나 balance tuning을 소유하지 않는다.

코드·live data 대조 결과:

1. 확정 정책
   - 활성 buff에서 중앙 파생한 Hard CC/Silence control capability가 기존 공격·skill과 신규 전술 command의
     단일 authority가 된다. Hard CC는 release 전 공격·cast를 취소하고 active control을 중단하며,
     Silence는 새 skill 시작만 막는다.
2. 현재 구현된 기반
   - live buff schema는 `BuffKind::Stun | Freeze | Silence`를 typed enum으로 구분하고 각 control buff의
     `max_stacks == 1`을 검증한다. Runtime buff key는 caster·target·buff ID를 포함하므로 여러 source를
     독립적으로 보존할 자료구조도 이미 있다.
   - BattleCore에는 active Hard CC의 최대 expiry/release time, Silence 존재 검사, action lock, pending basic
     attack/skill cast, manual/auto cast gate와 resonance gain gate가 있다.
   - 새 manual skill은 Hard CC를 `hard_crowd_control`, Silence를 `silenced` typed reason으로 이미 거부한다.
     Silence 중 pending autocast는 공명을 보존하고 expiry 뒤 재시도하며, 이미 pending인 cast는 Silence가
     적용되어도 완료한다.
   - withdraw는 active lifecycle만 요구하고 pending cast와 active cast를 명시적 cancel event와 함께
     정리하므로 Hard CC/Silence 중에도 사용할 수 있는 3번 정책 및 취소 처리 선례와 맞는다.
3. 정책과 일치하는 부분
   - Hard CC는 새 기본 공격 시작, 이동, 새 manual/auto skill과 resonance gain을 막고 target lock을
     해제한다. battle time 기반 absolute action/cooldown deadline은 CC 중에도 흐른다.
   - Silence는 기본 공격·이동·resonance gain을 막지 않고 새 skill만 차단한다. 이미 시작된 manual cast가
     Silence 중 완료되는 focused test도 있다.
   - `ActionDisabledReasonCode::HardCrowdControl/Silenced`와 server의 generic typed error mapping이 이미
     있어 새 error vocabulary나 전용 server response가 필요하지 않다.
   - 이미 만들어진 projectile/area와 active buff 및 reactive effect를 Hard CC/Silence가 일괄 삭제하는
     suppression layer는 현재 없으므로 확정한 보존 의미와 맞는다.
4. 정책과 다른 부분
   - Hard CC 적용은 아직 release되지 않은 기본 공격 windup을 취소하지 않는다. `AttackResolve`가 이미
     source snapshot과 함께 queue에 들어가며 Hard CC 여부나 action identity를 다시 확인하지 않아 CC 중에도
     그대로 release된다.
   - pending auto/manual skill cast도 취소하지 않는다. CastEnd를 Hard CC release time까지 재예약한 뒤
     CC가 끝나면 원래 skill을 발동하는 기존 test가 현재 성공 조건이다. `SkillCastCancelReason`도
     `Withdrawn`만 지원한다.
   - Stun과 Freeze는 target별 exclusive Hard CC로 취급된다. 다른 Hard CC가 들어오면 기존 효과를
     `Replaced`로 즉시 만료하므로, 더 짧은 새 CC가 더 긴 기존 CC를 단축할 수 있다. event-log validator도
     이 replacement를 강제한다.
   - Hard CC 판정이 `is_hard_cc_kind`, manual cast, autocast, basic attack, movement와 resonance 경로에
     나뉘어 있다. 현재는 두 variant뿐이라 동작하지만 신규 stance/relocation/swap/work가 각자 status를
     다시 해석하면 의미가 쉽게 갈라진다.
   - blocking validator는 RuntimeUnit의 static `block_capacity`만 보고 Hard CC를 검사하지 않으므로 Hard CC
     중에도 기존 blocker engagement가 유지된다. DefenseFocus/Rest/field work/stance 및 tactical command는
     아직 없어 해당 확정 상호작용도 구현되지 않았다.
   - checkpoint에는 active buff나 derived unit control capability가 없다. deployed unit의 manual skill
     readiness만 Hard CC/Silence reason을 간접 노출하므로 신규 전술 command와 resync가 공통 control state를
     소비할 projection이 부족하다.
5. 필요한 구현 범위
   - RON에 임의 capability boolean이나 범용 status scripting을 추가하지 않는다. 기존 typed
     `BuffKind::Stun/Freeze/Silence`가 `UnitControlCapabilities { hard_cc_active, silenced, release_at_ms }`를
     중앙 파생하고 모든 runtime/command validator가 이를 사용하게 하는 축소안을 권고한다.
   - Hard CC exclusive replacement를 제거해 caster/buff별 active instance를 각자 expiry까지 유지한다.
     capability는 하나라도 남아 있으면 active이고 release time은 남은 Hard CC들의 최대 expiry에서
     파생한다. buff event-log validator와 replacement 성공 test를 중첩/개별 expiry test로 교체한다.
   - Hard CC 적용 mutation에서 아직 release되지 않은 pending skill cast를 제거하고
     `SkillCastCancelled(reason: HardCrowdControl)`을 기록한다. 기존 CastEnd는 start seq가 더 이상 pending이
     아니므로 stale no-op이 된다. 현재 resonance는 CastEnd에서 소비되므로 release 전 취소에는 애초에
     소비된 resource가 없고, 향후 start 시점 소비가 생기면 확정 정책대로 환불하지 않는다.
   - basic attack windup에는 action generation/token을 부여한다. Hard CC 적용 시 generation을 무효화하고
     `AttackResolve`가 token 불일치면 release하지 않게 한다. projectile/delayed effect를 이미 만든
     resolve 이후에는 token을 보지 않는다. 범용 event-queue 삭제나 공격 전체 rollback은 만들지 않는다.
   - 취소된 action의 pending state는 제거하되 이미 기록된 absolute cooldown/recovery deadline은 앞으로
     당기지 않는다. Hard CC 종료 뒤 현재 deadline과 stance/control state를 함께 재평가해 새 행동을
     시작하며 취소된 action은 복원하지 않는다.
   - 8·9·12번 effective defense selector가 Hard CC capability를 소비해 effective block만 0으로 만들고
     Rest를 suspend한다. incoming damage 감소와 stance는 보존한다. block state를 refresh해 engagement를
     모두 해제하고 마지막 Hard CC expiry 뒤 현재 위치에서 새로 판정한다.
   - 4·6·14·17번 command/work validator도 같은 capability를 사용한다. Withdraw는 예외적으로 계속 enabled를
     유지한다. checkpoint/resync에는 최소 derived control state와 release time을 투영하고 server는 기존
     generic battle update mapping을 유지한다.
6. 현재 tree 기준 난이도와 이유
   - 제안 난이도는 `3/5 — 중간`이다. Hard CC/Silence buff lifetime, cast/resonance gates, typed reason과
     server mapping은 이미 있다. 주된 변경은 BattleCore의 중앙 capability, Hard CC 중첩 lifetime,
     skill/basic windup cancellation identity, block gate, checkpoint projection과 focused validator/test다.
   - 새 command, navigation, 자원 경제나 범용 effect engine을 만들지 않으므로 4/5의 신규 end-to-end
     command/state 기능까지는 아니다. 반면 기존 success test 두 개의 의미를 바꾸고 attack/cast event
     lifecycle 및 event-log validator를 함께 수정하므로 2/5로 보기는 어렵다.
7. 선행 정책 구현 후 난이도 차이
   - 4·6·8·9·12·14·17번이 먼저 구현되어 stance, defense effect, relocation/swap과 field work consumer가
     모두 존재하면 13번 증분은 `2/5 — 낮음`이다. 중앙 capability를 각 validator/selector에 연결하고
     suspension/cancellation focused test를 추가하는 범위로 좁아진다.
   - 반대로 13번을 먼저 구현하면 이후 정책은 Hard CC/Silence 이름이나 buff table을 직접 검사하지 않고
     capability 하나를 재사용할 수 있어 각 command의 상태 연동 비용이 줄어든다. 이 경우에도 13번 자체의
     attack/cast cancellation과 중첩 lifetime 비용은 남아 현재 tree `3/5`를 유지한다.
8. 난이도 책정 reference 파일과 책임
   - Core command/wire
     - `core_enhanced/src/game/behavior.rs`: 기존 `hard_crowd_control`/`silenced` disabled reason, skill cast
       cancellation presentation event와 unit/deployment checkpoint shape를 소유한다.
     - `core_enhanced/src/game/battle/event_log.rs`: buff expiry reason과 현재 `Withdrawn`만 있는 skill cast
       cancel reason 및 authoritative semantic event를 정의한다.
   - Core world transaction/state
     - `core_enhanced/src/game/world/combat.rs`: manual skill/withdraw battle transaction과 향후 전술 command
       gate가 결합될 경계다.
     - `core_enhanced/src/game/world/state.rs`: deployed unit skill readiness와 battle checkpoint/resync
       projection을 만들지만 현재 unit control state는 투영하지 않는다.
   - Battle runtime
     - `core_enhanced/src/game/battle/buffs.rs`: typed Stun/Freeze/Silence metadata와 control buff validation을
       소유한다.
     - `core_enhanced/src/game/battle/core/mod.rs`: active Hard CC/Silence 조회, resonance/autocast gate와
       runtime buff state의 중심이다.
     - `core_enhanced/src/game/battle/core/sim.rs`: manual/auto cast gate, Hard CC exclusive replacement,
       CastEnd 지연과 buff expiry 재평가를 구현한다.
     - `core_enhanced/src/game/battle/core/basic_attack.rs`: Hard CC 중 새 AttackStart는 지연하지만 이미 queue된
       AttackResolve를 취소할 identity가 없는 attack lifecycle이다.
     - `core_enhanced/src/game/battle/core/types.rs`: max-deadline `ActionLocks`, pending basic/skill state와 새
       attack cancellation generation이 들어갈 RuntimeUnit state를 소유한다.
     - `core_enhanced/src/game/battle/core/build.rs`: Hard CC/Silence 중에도 허용할 withdraw 및 pending/active
       skill cancellation의 기존 선례다.
     - `core_enhanced/src/game/battle/core/movement/blocking.rs`: blocker validity가 현재 Hard CC를 보지 않아
       effective block 0과 engagement release를 연결해야 하는 경계다.
     - `core_enhanced/src/game/battle/validation/buffs.rs`: event log에서 exclusive Hard CC replacement를
       강제하므로 중첩 lifetime 정책에 맞춰 바꿀 validator다.
   - Policy/live data
     - `game_resources/data/buffs/base.ron`: 현재 poison, stun, freeze와 silence 4개 typed buff를 authoring한다.
     - `game_resources/data/skills/base.ron`: 현재 live skill에 Freeze 1회, Stun 3회, Silence 6회가 있어 변경된
       control 의미가 실제 전투 content에 적용된다.
     - `game_resources/data/equipments/base.ron`: 현재 weapon triggered effect에 Freeze 2회, Stun 1회,
       Silence 2회가 있어 passive/reactive 보존과 중첩 동작의 실제 source다.
   - Game server transport
     - `game_server/src/game/player_game_actor/state.rs`: `ActionUnavailable`의 typed reason을 server code로
       보존하고 Core battle update를 일반 경로로 전달한다.
     - `game_server/src/game/player_game_actor/messages.rs`: Core checkpoint/event delta를 top-level update와
       resync에 그대로 운반하므로 13번 전용 server message가 필요 없다.
   - 외부 Unity runtime/contract/tests
     - 사용자 결정에 따라 7번부터 Unity 조사를 제외했으며 13번 난이도에도 Unity 구현 비용을 포함하지
       않았다.
9. 실행한 focused test와 결과
   - `cargo test -p game_core hard_cc_delays_autocast_completion_until_lock_release` — 통과(1개). Hard CC가
     pending cast를 취소하지 않고 expiry 뒤 발동시키는 현재 정책 불일치를 확인했다.
   - `cargo test -p game_core hard_cc_replacement_records_explicit_buff_expired_reason` — 통과(1개). Stun/Freeze
     exclusive replacement가 기존 성공 조건임을 확인했다.
   - `cargo test -p game_core hard_cc_clears_locked_target_and_reacquires_after_release` — 통과(1개). target clear와
     expiry 뒤 현재 상태 재평가 기반을 확인했다.
   - `cargo test -p game_core silence_blocks_new_casts_but_does_not_interrupt_pending_casts` — 통과(1개).
     Silence의 신규 cast 제한/기존 cast 완료 의미가 확정 정책과 일치함을 확인했다.
   - `cargo test -p game_core silence_blocks_pending_autocast_until_expire` — 통과(1개). full resonance를
     보존하고 Silence expiry 뒤 autocast를 재평가하는 기반을 확인했다.
   - `cargo test -p game_server action_unavailable_preserves_typed_disabled_reason_code` — 통과(1개). server의
     typed reason 전달 seam을 확인했다.
   - 이 통과 결과는 현재 Hard CC/Silence 기반과 명시적 충돌을 재현한 증거일 뿐, 확정한 cancellation,
     overlapping lifetime, block/Rest/work suspension이나 신규 전술 command 연동이 구현됐다는 증거가
     아니다.
10. 아직 사용자가 확정해야 할 tuning 또는 선택
    - 없음. 별도 balance tuning은 두지 않고 기존 live Stun/Freeze/Silence duration과 각 content 발동
      확률은 이번 의미 정책에서 변경하지 않는 것으로 확정했다.
    - 현재 tree 난이도 `3/5`, 4·6·8·9·12·14·17번 선행 후 증분 `2/5`로 확정했다.
    - 기존 typed BuffKind에서 capability를 파생하고 Hard CC용 attack generation만 추가하는 축소안으로
      확정했다. 범용 status capability RON, passive suppression 시스템 또는 전체 event queue rollback은
      만들지 않는다.

### 14. 자리 교대 — 정책·코드 대조·난이도·tuning 확정 완료

확정일: 2026-08-06  
난이도·초기 tuning 확정일: 2026-08-06  
구현 난이도: 현재 tree `4/5 — 높음`, 4·5·6·7·8·9·12·13번 선행 후 `3/5 — 중간`,
17번까지 선행 후 증분 `2/5 — 낮음`  
초기 live tuning: 전술 포인트 `5`를 command당 1회 / 양쪽 직원 cooldown `10,000ms`

확정 정책:

- 플레이어에게 일반 순간이동과 자리 교대의 독립 버튼을 두지 않는다. 하나의 `전술 이동`에서 source를
  선택한 뒤 빈 target tile을 선택하면 4번 전술 순간이동, 다른 active 아군 직원을 선택하면 14번 자리
  교대 preview로 분기한다. 내부 wire와 transaction은 명시적인 relocation/swap command로 구분한다.
- 자리 교대는 서로 다른 두 active Player-side 직원의 logical/world position을 한 command와 revision에서
  원자적으로 맞바꾼다. 보행, pathfinding, 이동 시간, 중간 중복 점유나 reservation state는 만들지 않는다.
- 두 직원 모두 `Defense` 태세이고 employee별 tactical relocation cooldown이 준비되어야 한다. Hard CC,
  아직 release/end되지 않은 기본 공격 또는 skill action과 이동을 금지하는 exclusive activity가 있으면
  source나 target이 될 수 없다. Silence와 `FieldWorking`은 자리 교대를 막지 않는다.
- 각 직원은 상대 tactical tile에 대해 player deployment zone, 현재 weapon deployment affinity, valid
  tile, static obstacle과 work-cell eligibility를 각각 통과해야 한다. 점유 검사에서는 교대하는 두
  runtime unit만 제외하며, 방어 목표나 제3의 active Player-side actor가 어느 destination을 점유하면
  전체를 거부한다. 적의 projected overlap은 자리 교대를 막지 않는다.
- 교대 preview에서 두 직원의 새 facing을 각각 지정한다. 기본값은 각 직원의 교대 전 facing이며, 서로의
  facing을 자동 복사하거나 교환하지 않는다.
- `FieldWorking` 직원도 source 또는 target이 될 수 있다. Work assignment는 직원을 따라 이동하지 않는다.
  기존 work cell을 벗어난 직원은 같은 mutation에서 assignment를 종료하고, authored work cell에 들어온
  적격 `Defense` 직원은 같은 mutation에서 새 assignment를 시작한다. 일반 tile로 이동한 직원은 일반
  `Defense`와 해당 tile-kind effect로 즉시 복귀한다.
- 교대하는 두 직원의 assignment 제거·추가를 모두 적용한 뒤 field work object의 interruption policy를
  한 번 평가한다. 누적 progress는 worker가 아니라 object가 소유하므로 같은 교대로 새 작업자가 work
  cell에 들어오면 중간 무작업 상태를 만들거나 progress를 초기화하지 않는다. 구체적인 worker eligibility,
  progress와 completion 의미는 17번 확정 정책을 따른다.
- 성공 뒤에도 양쪽의 runtime identity, HP, buff/debuff, resonance, stance, 기본 공격·skill cooldown과
  그 밖의 unit-owned state를 유지한다. 양쪽 위치·facing과 work/tile effect만 같은 mutation에서 전환한다.
- 성공 mutation에서 양쪽의 기존 engagement를 먼저 해제하고 새 위치의 effective block, target/range,
  aura/area 포함 관계와 `DefenseFocus/Rest/FieldWorking`을 deterministic하게 다시 계산한다. 이미 release된
  공격, projectile, delayed step, area, buff/debuff와 확정된 파생 효과는 기존 lifecycle을 계속한다.
- 전술 포인트는 자리 교대 command당 fixed cost를 한 번만 소비하고, 성공 시 양쪽 직원의 tactical
  relocation cooldown을 모두 시작한다. Field work assignment 전환에는 별도 비용을 부과하지 않는다.
  실제 swap cost와 cooldown tuning은 현재 live 전투 시간·자원 경제와 기존 relocation policy를 코드/data로
  대조한 뒤 확정한다.
- command는 두 unit의 identity/lifecycle, stance/action/control/activity, 양방향 placement/work eligibility,
  occupancy, 전술 포인트와 양쪽 cooldown을 모두 먼저 검증한다. 하나라도 실패하면 양쪽 위치·assignment,
  resource, cooldown, event와 revision을 전혀 변경하지 않는다.
- Preview 뒤 occupancy나 eligibility가 바뀌면 최신 authority에서 전체를 다시 검증한다. 점유 상태가
  달라졌다는 이유로 server가 swap을 일반 relocation으로 바꾸거나 반대 방향으로 fallback하지 않으며,
  실패 뒤 전술 집중 모드를 유지한다.

코드·live data 대조 결과:

1. 확정 정책
   - 하나의 전술 이동 UX에서 점유 아군을 선택했을 때 명시적 swap command로 분기하고, 두 active
     `Defense` 직원의 위치·새 facing·tile/work effect를 한 revision에서 원자적으로 맞바꾼다.
   - `FieldWorking`도 source/target으로 허용한다. Assignment는 직원을 따라가지 않고 최종 work-cell
     점유에서 다시 만들며, object progress interruption은 양쪽 assignment 전환 뒤 한 번만 평가한다.
2. 현재 구현된 기반
   - `PlayerBehavior`의 deploy/withdraw/skill 명령, battle authority와 `source_command_id`,
     `BattleLiveCommand`/outcome 및 semantic event→presentation DTO 변환 경계가 이미 있다.
   - World deploy transaction은 zone·affinity·resource eligibility를 먼저 보고 BattleCore command,
     deployment mapping, 자원, event/checkpoint와 revision을 한 번 commit하는 vertical transaction 선례다.
   - `RuntimeUnit`은 stable instance identity, world position, projected tile과 facing을 보유하고
     `set_world_position`으로 HP·buff·resonance·cooldown을 초기화하지 않은 채 위치만 바꿀 수 있다.
   - Battlefield는 valid tile과 static obstacle 검사를 분리한다. FixedDefense block runtime은 stable ID와
     기존 engagement를 사용해 결정론적으로 저지 관계를 다시 만들고 해제된 target preference를 정리한다.
   - battle-local deployment resource의 current/max, battle time 자연 회복과 server-authoritative
     checkpoint/revision은 이미 있다. `PositionOccupied`도 server에서 `position_occupied`로 변환한다.
3. 정책과 일치하는 부분
   - BattleCore 위치 model은 player/enemy를 포함한 동타일 다중 점유를 허용하므로 원자적 swap을 위해
     simulation 전역의 1-unit-per-tile 불변식이나 임시 reservation을 만들 필요가 없다.
   - Unit 위치와 facing이 checkpoint에 authoritative하게 투영되고 range/targeting은 현재 position/facing을
     읽으므로, unit-owned state를 보존한 채 최종 공간 상태를 다시 계산하는 방향과 맞는다.
   - Deploy가 사용하는 deployment zone affinity, valid tile와 static obstacle primitive를 두 destination에
     재사용할 수 있다. Existing world transaction과 server envelope도 성공 1 revision/실패 무변경의
     기반을 제공한다.
   - 현재 전술 자원은 battle time에서 초당 1 회복하고 기존 확정 relocation은 비용 5/직원 cooldown
     10,000ms이므로 swap의 resource/cooldown tuning을 같은 시간축과 policy에 둘 근거가 있다.
4. 정책과 다른 부분
   - `SwapUnitPositions`에 해당하는 ActionKind, PlayerBehavior, BattleLiveCommand/outcome, world handler,
     semantic/presentation event와 result DTO는 전혀 없다. 일반 relocation도 아직 구현되지 않았다.
   - `LiveBattleDeploymentState`는 deploy/redeploy cost와 redeploy lock만 소유한다. 직원별 tactical relocation
     cooldown, swap cost, stance/activity/control eligibility와 swap readiness projection이 없다.
   - 현재 employee combat profile은 `DeploymentAffinity::GroundOnly` default를 유지하며 weapon RON의
     `WeaponCombatProfile`에는 deployment affinity와 Defense/Rest profile이 없다. 따라서 양방향 affinity와
     tile effect 전환은 10·12번까지 구현되어야 실제 검증할 수 있다.
   - Player deploy command는 같은 tile에 두 아군 직원을 성공적으로 배치하며 existing test도 이를 성공
     조건으로 고정한다. 교대 pair만 제외하고 방어 목표·제3 Player actor를 거부할 공통 occupancy query는
     아직 없다.
   - stance, Hard CC 중앙 capability, unreleased action eligibility, `DefenseFocus/Rest`와 `FieldWorking`이
     구현되지 않았다. Setup의 `static_objects`는 항상 빈 배열이고 runtime/checkpoint에는 field work
     object, assignment나 accumulated progress가 없다.
   - Block refresh는 simulation movement/targeting tick에서 호출되지만 player relocation/swap 직후 두
     직원의 기존 engagement를 명시적으로 모두 해제하고 최종 위치에서 한 번 재계산하는 command seam은
     없다.
   - 현재 typed disabled reason에는 hard CC, action lock과 resource 기반은 있으나 stance, relocation
     cooldown, activity/work eligibility를 구분할 vocabulary와 swap payload eligibility DTO는 없다.
5. 필요한 구현 범위
   - Core wire에 두 employee UUID와 두 facing을 가진 명시적 `SwapUnitPositions`, ActionKind, result와
     `UnitPositionsSwapped` semantic/presentation event를 추가한다. 일반 relocation과 payload를 합치거나
     server-side 자동 fallback을 만들지 않는다.
   - World handler가 두 employee→runtime mapping, 서로 다른 active Player combatant, stance/action/control/
     activity, 양쪽 cooldown, resource, 양방향 zone/affinity/valid/static/work eligibility와 pair-excluding
     occupancy를 모두 읽기 전용으로 검증해 commit plan을 만든다. 검증 중 BattleCore를 일부 변경했다가
     rollback하는 방식은 사용하지 않는다.
   - 검증 완료 뒤 BattleCore가 두 final position/facing을 한 infallible mutation에서 적용한다. 양쪽 기존
     engagement를 해제하고 final state에서 block/target/range/area 및 tile effect를 한 번 다시 계산한다.
     Runtime identity와 unit-owned combat state는 보존한다.
   - 5번 occupancy validator는 제외 unit set을 받아 swap pair만 제외하고 나머지 active Player-side actor를
     검사하게 한다. 12번 affinity/tile-effect selector는 두 destination을 각각 미리 계산한 뒤 한꺼번에
     적용한다.
   - 17번 field work runtime이 있으면 assignment remove/add를 final cell 점유에서 한 batch로 계산하고,
     object별 interruption policy를 batch 뒤 한 번 적용한다. 작업자를 교체하는 과정의 순간적인 worker 0
     상태를 외부 state/event로 만들지 않는다.
   - Battle deployment state/checkpoint에 employee별 relocation `ready_at_ms`와 양쪽 readiness를 투영한다.
     성공 시 전술 포인트를 한 번 차감하고 두 직원의 ready time을 함께 기록하며 실패 시 resource,
     cooldown, event와 revision을 바꾸지 않는다.
   - Game server는 Core의 새 behavior/result/event를 기존 generic command/update envelope로 운반한다.
     새 server 전용 transaction이나 두 command batch는 만들 필요가 없다.
6. 현재 tree 기준 난이도와 이유
   - 제안 난이도는 `4/5 — 높음`이다. 현재 tree에는 relocation, stance, command-level occupancy,
     weapon-owned affinity/tile effect, Hard CC capability와 field work가 모두 없어 완성된 14번 vertical
     slice가 command/wire, world state/transaction, BattleCore, policy/checkpoint와 server contract를 지난다.
   - 다만 RuntimeUnit 위치 primitive, deployment authority, resource와 revision, placement 및 deterministic
     block 기반은 존재한다. 핵심 simulation이나 범용 effect/data model을 처음부터 다시 설계할 필요는
     없으므로 `5/5 — 매우 높음`은 아니다.
7. 선행 정책 구현 후 난이도 차이
   - 4·5·6·7·8·9·12·13번이 먼저 구현되면 일반 relocation command/wire, employee cooldown/resource,
     stance/action/control gate, occupancy와 tile effect 전환을 재사용할 수 있어 증분은 `3/5 — 중간`이다.
     이때도 두 유닛 validation/commit과 FieldWorking handoff가 남는다.
   - 위 선행 정책과 17번 field work object/assignment batch seam까지 구현되어 있으면 14번 증분은
     `2/5 — 낮음`이다. Pair validation, 두 position/facing commit, 양쪽 cooldown과 focused atomicity test가
     중심이 된다.
   - 따라서 14번을 현재 tree에서 단독 구현하기보다 4·5번 공통 relocation transaction과 17번 assignment
     reconcile seam을 설계한 뒤 연결하는 편이 자연스럽다. 이는 현재 tree 난이도를 미리 낮춰 책정한다는
     뜻은 아니다.
8. 난이도 책정 reference 파일과 책임
   - Core command/wire
     - `core_enhanced/src/game/behavior.rs`: 기존 battle ActionKind/PlayerBehavior, deployed-unit checkpoint,
       typed disabled/error와 event/result wire를 소유하며 swap command/readiness/event가 추가될 경계다.
     - `core_enhanced/src/game/battle/event_log.rs`: `source_command_id`를 가진 append-only semantic event와
       deploy/withdraw event를 정의하며 원자적 swap 연출 event가 추가될 경계다.
   - Core world transaction/state
     - `core_enhanced/src/game/world.rs`: PlayerBehavior를 battle domain handler로 routing하는 단일 경계다.
     - `core_enhanced/src/game/world/combat.rs`: deployment zone/affinity preflight와 resource→BattleCore→mapping→
       revision commit 순서를 소유하며 양쪽 swap plan을 검증할 transaction 경계다.
     - `core_enhanced/src/game/world/state.rs`: battle-local resource, employee↔runtime mapping, redeploy lock과
       checkpoint/setup projection을 소유하지만 relocation cooldown과 field work state는 없다.
   - Battle runtime
     - `core_enhanced/src/game/battle/core/sim.rs`: 현재 세 종류뿐인 `BattleLiveCommand`/outcome과
       `source_command_id` recording wrapper를 소유한다.
     - `core_enhanced/src/game/battle/core/build.rs`: player deploy/withdraw mutation, facing 설정, walkable tile
       검사와 event 기록 선례를 제공한다.
     - `core_enhanced/src/game/battle/core/types.rs`: stable RuntimeUnit identity, world position/facing,
       action lock과 pending basic/skill state를 소유한다.
     - `core_enhanced/src/game/battle/core/movement/blocking.rs`: stable ordering, sticky engagement, release된
       target 정리와 deterministic block refresh를 소유한다.
     - `core_enhanced/src/game/battle/battlefield/field.rs`: valid tile과 static obstacle placement 검사를
       분리해 제공한다.
     - `core_enhanced/src/game/battle/types.rs`: `GroundOnly/PlatformOnly/Any` affinity와 현재 employee profile의
       GroundOnly default를 소유한다.
   - Policy/live data
     - `core_enhanced/src/game/data/run_policy_data.rs`: 현재 deploy/redeploy economy schema와 validation을
       소유하며 공통 tactical relocation cost/cooldown field가 추가될 경계다.
     - `core_enhanced/src/game/data/equipment_data.rs`: weapon combat RON schema를 소유하지만 현재 affinity와
       Defense/Rest authoring은 없다.
     - `game_resources/data/run/policy.ron`: 초기 20, 최대 99, 초당 1 회복, 기본 deploy 비용 10의 현재 live
       경제를 authoring하며 아직 relocation/swap field는 없다.
     - `game_resources/data/equipments/base.ron`: 현재 live weapon profile은 공격과 block 값만 authoring하고
       destination affinity/tile effect는 authoring하지 않는다.
     - `game_resources/data/pve/encounters.ron`: 현재 Defense encounter의 wave spawn time을 authoring하며
       전투 종료 시간은 적 처치·목표 상태에 따라 runtime에서 결정된다.
   - Game server transport
     - `game_server/src/game/player_game_actor/messages.rs`: Core PlayerBehavior를 battle authority가 포함된
       generic command envelope로 deserialize하고 기존 live battle request contract test를 소유한다.
     - `game_server/src/game/player_game_actor/state.rs`: Core typed error와 `PositionOccupied`를 안정된 server
       error code로 변환한다.
   - 외부 Unity runtime/contract/tests
     - 사용자 결정에 따라 7번부터 Unity 조사를 제외했으며 14번 난이도에도 Unity 구현 비용을 포함하지
       않았다.
9. 실행한 focused test와 결과
   - `cargo test -p game_core apply_live_command_deploys_and_withdraws_player_unit` — 통과(1개). Runtime
     identity, live command/outcome와 deploy/withdraw semantic event 기반을 확인했다.
   - `cargo test -p game_core live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock` — 통과(1개).
     World resource/mapping/revision transaction 기반과 현재 player command가 아군 동타일 배치를 성공
     조건으로 유지하는 정책 불일치를 확인했다.
   - `cargo test -p game_core fixed_defense_preserves_existing_block_before_new_route_priority` — 통과(1개).
     Stable order와 기존 engagement를 사용하는 현재 deterministic block refresh 기반을 확인했다.
   - `cargo test -p game_core static_obstacles_block_placement` — 통과(1개). Valid/static destination 검사의
     기존 low-level 기반을 확인했다.
   - `cargo test -p game_server deserializes_live_battle_requests` — 통과(1개). Battle authority command
     envelope과 기존 deploy/withdraw/skill transport 기반을 확인했다.
   - 이 통과 결과는 기존 seam이 안정적이라는 증거일 뿐, swap command, 원자적 두 유닛 mutation,
     양방향 affinity/occupancy, cooldown 또는 FieldWorking handoff가 구현됐다는 증거가 아니다.
10. 아직 사용자가 확정해야 할 tuning 또는 선택
    - 없음. 현재 tree 난이도 `4/5`, 4·5·6·7·8·9·12·13번 선행 후 `3/5`, 여기에 17번까지 선행한
      증분 `2/5`로 확정했다.
    - 일반 전술 순간이동과 같은 전술 포인트 `5`를 command당 한 번 소비하고, 양쪽 직원에 같은
      `10,000ms` tactical relocation cooldown을 시작하는 것으로 확정했다. 비용과 cooldown은 각각 기존
      relocation policy field 하나를 공유하고 swap 전용 중복 tuning field를 만들지 않는다.
    - Swap은 두 직원을 움직이지만 아군 배치 수를 늘리지 않고 양쪽 cooldown을 동시에 잠근다. 비용까지
      10으로 두 배로 만들면 현재 초당 1 자연 회복에서 작업자 구조가 과도하게 무거워진다. 비용 5는
      자리 교대를 같은 전술 이동 기능의 파생으로 유지하면서 양쪽 cooldown으로 추가 가치를 제약한다.
    - `FieldWorking` source/target 허용과 원자적 worker handoff도 확정됐다. 첫 object의 구체 progress/
      completion content는 17번에서 확정하되 14번의 별도 balance 수치로 만들지 않는다.

### 15. Pause 중 명령 — 정책·코드 대조·난이도 책정 완료

확정일: 2026-08-06  
난이도·세부 선택 확정일: 2026-08-06  
구현 난이도: 현재 tree와 선행 정책 구현 후 모두 `1/5 — 매우 낮음`

확정 정책:

- `Pause`는 authoritative battle clock만 정지한다. Server의 command와 read-only query 처리는 계속한다.
  Pause 자체가 command eligibility를 완화하지 않으며 각 명령은 frozen battle time의 최신 state에서
  평상시와 같은 authority, resource, cooldown, stance, activity, target과 destination 조건을 검증한다.
- Gameplay mutation은 한 번에 하나씩 제출·처리한다. Client는 앞 mutation의 authoritative response와
  최신 battle revision을 받은 뒤 다음 mutation을 보낸다. 여러 gameplay command를 local plan으로
  모으거나 한 batch로 제출하지 않는다.
- 각 mutation은 독립된 원자 transaction이다. 성공하면 해당 command의 즉시 state/event를 commit하고
  battle revision을 정확히 1 증가시킨다. 실패하면 state, resource, cooldown, event와 revision을 전혀
  변경하지 않는다. 성공·실패 뒤에도 명시적 `ResumeBattle` 전까지 Pause를 유지한다.
- 앞 command의 성공은 뒤 command가 실패해도 유지한다. 여러 command sequence의 all-or-nothing
  transaction이나 앞 성공을 되돌리는 rollback은 제공하지 않는다.
- 성공한 자원 소비, 위치·facing, 태세, 배치·철수, skill cast 시작, cooldown 시작과 그 밖의 즉시
  mutation은 frozen battle time에도 바로 authoritative state에 반영한다.
- Pause 중에는 queued simulation event를 진행하지 않는다. 적 이동·공격, pending cast와 field work 진행,
  HP·공명·전술 포인트 회복, buff duration과 battle-time cooldown 감소는 모두 정지하며 해제 뒤 Pause
  시간을 소급하거나 비축하지 않는다.
- 예를 들어 Pause 중 태세 전환이 성공하면 태세와 cooldown 시작 시각은 즉시 commit되지만 battle time이
  흐르지 않으므로 같은 직원의 즉시 재전환은 cooldown으로 거부한다. 다른 command도 자신의 즉시 결과는
  보이되 time-based 후속 단계는 Resume 뒤에만 진행한다.
- 같은 frozen battle time에 여러 command가 성공하면 authoritative 순서는 server 처리 순서, battle
  revision과 event sequence가 소유한다. `source_command_id`는 요청과 semantic event의 correlation만
  담당하며 정렬·우선순위 authority로 사용하지 않는다.
- Battle state/resync, 유닛 상태, HP/공명, buff/debuff, 태세, cooldown, 장비·skill 정보와 range/target/
  deployment preview 같은 read-only query는 최신 Pause state를 즉시 읽는다. 성공한 query는 revision을
  증가시키지 않으며 잘못된 query도 gameplay state를 바꾸지 않고 typed error로 거부한다.
- `ResumeBattle`도 직렬 command 경계를 따른다. Client는 진행 중인 mutation response를 받은 뒤 최신
  revision으로 재개를 요청하고, 성공 뒤에만 battle clock이 다시 흐른다.
- `RetreatBattle`은 Pause 중에도 같은 직렬 경계에서 즉시 검증·실행할 수 있다. 성공하면 active battle과
  Pause state를 함께 종료한다.
- Pause 중 `SetBattleSpeed`를 허용한다. Frozen battle clock은 진행하지 않고 Resume 뒤 사용할 stored
  speed만 즉시 변경하며 revision을 정확히 1 증가시킨다. 별도 resume-speed command나 Pause 전용 speed
  state는 만들지 않는다.
- Pause 전용 command runtime, client-local pending gameplay queue, command batch와
  `ResumeBattleWithPlan`은 만들지 않는다. Reconnect/resync는 이미 commit된 최신 Pause state만 복구하며
  아직 제출하지 않은 local 선택이나 계획은 복구하지 않는다.

코드·live data 대조 결과:

1. 확정 정책
   - Pause는 authoritative playback clock만 정지하고 기존 battle mutation/query pipeline은 계속
     사용한다. 각 mutation은 최신 revision에서 독립적으로 즉시 commit/reject하며 local plan, batch나
     sequence rollback을 만들지 않는다.
   - 같은 frozen battle time의 순서는 server 처리 순서와 revision/event seq가 소유하고
     `source_command_id`는 correlation에만 사용한다.
2. 현재 구현된 기반
   - `BattlePlaybackState { paused, speed }`와 Pause/Resume/SetBattleSpeed behavior, payload eligibility,
     checkpoint projection이 이미 있다.
   - Production server tick은 `simulation_delta_for_tick`을 통과하며 paused면 `None`을 반환해 BattleCore를
     전혀 step하지 않는다. Resume 뒤에만 battle time이 다시 증가한다.
   - Active battle mutation은 domain handler보다 먼저 exact battle UUID/revision authority를 검사한다.
     각 성공 handler는 commit 가능 여부를 먼저 확인하고 revision을 한 번 증가시키며, stale/missing
     authority는 현재 resync와 함께 무변경으로 거부된다.
   - Deploy/withdraw/skill과 action capability는 playback paused를 금지 조건으로 사용하지 않는다. 따라서
     현재 명령도 frozen battle time에서 평상시 payload eligibility로 즉시 처리할 구조다.
   - `RequestBattleState`, setup-loss recovery와 deployment preview는 authority가 필요 없는 read-only
     경로이며 checkpoint를 만들 뿐 revision을 commit하지 않는다.
   - Game server의 `PlayerGameActor` command handler는 synchronous actor mailbox에서 GameCore mutation과
     persistence를 한 요청씩 처리한다. 성공 battle command는 request-scoped ack 뒤 battle update를
     보내고 stale/missing authority는 error 뒤 current battle resync를 보낸다.
3. 정책과 일치하는 부분
   - Pause/Resume 성공은 playback state를 즉시 바꾸고 revision을 정확히 1 증가시킨다. 반복 Pause/Resume와
     revision overflow는 state/event/time/revision을 바꾸지 않고 typed reason으로 실패한다.
   - Paused server tick은 BattleCore event queue를 진행하지 않으므로 movement/attack/cast, buff tick/expiry,
     area/projectile, cooldown과 회복이 모두 같은 battle-time authority에서 함께 정지한다.
   - 전술 자원 자연 회복도 active battle advance 결과에서만 `last_event_time_ms`로 계산하므로 Pause 중
     wall time을 소급하지 않는다. Redeploy lock과 향후 tactical cooldown도 absolute battle
     `ready_at_ms` 비교를 재사용할 수 있다.
   - Checkpoint와 setup-loss recovery가 paused flag, 같은 battle time/revision과 event cursor를 보존한다.
     별도 pending plan 복구 state는 현재 없으며 확정 정책에도 필요하지 않다.
   - Server actor가 command를 한 번에 하나씩 처리하고 revision이 stale request를 걸러내므로 별도 Pause
     mutex, transaction queue 또는 `ResumeBattleWithPlan` 없이 결정론적 순서를 유지한다.
   - `BattleEventLogEntry`는 동일 time의 사건을 monotonic seq로 구분하고 `source_command_id`를 별도
     optional correlation field로 저장한다. 확정한 authority 분리와 일치한다.
4. 정책과 다른 부분
   - 현재 focused test는 Pause clock, query, resume, authority와 transport 순서를 검증하지만 paused 상태에서
     deploy/withdraw/manual skill을 실제 commit한 뒤 battle time 고정과 즉시 state 변화를 함께 검증하지
     않는다.
   - 같은 frozen battle time에서 최신 revision으로 첫 mutation이 성공한 뒤 두 번째가 실패해도 첫 결과를
     유지하는 sequence test, 두 요청이 같은 revision을 보냈을 때 첫 성공 뒤 둘째가 stale로 거부되는
     actor test가 없다.
   - 아직 구현되지 않은 stance, relocation/swap, targeted skill과 field work에는 Pause 연동 test가 없다.
     이는 현재 Pause 기반의 결함이라기보다 해당 정책 구현 시 추가해야 할 consumer coverage다.
   - `advance_active_battle_by`와 `advance_active_battle_to_next_event_bucket`은 playback을 거치지 않는 public
     deterministic driver primitive다. Production server는 pause-aware tick만 사용하지만 새 production
     caller가 raw driver를 잘못 호출하면 Pause를 우회할 수 있어 책임을 문서·visibility로 명확히 해야 한다.
   - `SetBattleSpeed`는 paused 중에도 현재 허용되며 speed와 revision을 즉시 바꾼다. Clock은 그대로
     정지하고 새 speed는 Resume 뒤 적용되지만, 이 세부 의미가 확정 정책에 명시적으로 적혀 있지는 않다.
5. 필요한 구현 범위
   - Pause 전용 runtime이나 DTO는 추가하지 않는다. 기존 playback flag, production server tick,
     BattleCore command handler와 revision transaction을 그대로 source of truth로 유지한다.
   - Core focused test에 Pause→deploy/withdraw/manual skill→query를 추가해 같은 battle time, command별
     revision 1 증가, 즉시 resource/state/event 반영과 queued action 미진행을 검증한다.
   - 앞 mutation 성공 뒤 다음 invalid mutation 실패 시 앞 state/revision이 보존되는 test와 Resume 뒤에만
     pending action/cooldown/time이 진행하는 test를 추가한다.
   - 각 4·6·14·16·17번 command가 구현될 때 capability가 paused만으로 비활성화되지 않는지, 최신 revision
     commit, time-based 후속 단계 정지와 실패 무변경을 해당 command의 focused test에 포함한다. 15번용
     통합 command batch를 만들지 않는다.
   - Server test에는 동일 revision mutation 두 건의 처리 결과를 추가한다. 첫 요청이 성공하면 둘째는
     stale error→current resync가 되어야 한다. Client의 one-in-flight는 contract discipline이며 server에
     별도 in-flight token state를 추가하지 않는다.
   - Raw deterministic advancement API는 production playback 경로가 아님을 명시하고 가능하면 visibility를
     test/internal 용도로 좁힌다. BattleCore 자체에 paused flag를 넣어 simulation과 playback을 결합하지
     않는다.
   - Paused `SetBattleSpeed` 허용을 확정하면 현재 동작을 유지하고 “Resume 시 적용될 stored speed 변경”
     focused test와 canonical 설명만 추가한다.
6. 현재 tree 기준 난이도와 이유
   - 제안 난이도는 `1/5 — 매우 낮음`이다. Clock freeze, playback checkpoint, revision authority,
     mutation/query 분리, actor serialization, ack/update 및 error/resync 순서까지 이미 확정 정책과
     일치한다.
   - 남은 일은 Pause 중 기존 mutation sequence focused test, raw driver 책임 명시와 paused speed test
     중심이다. 새 command, runtime state, data schema나 server transport를 추가하지 않으므로 2/5의 제한된
     runtime 기능 변경에도 미치지 않는다.
7. 선행 정책 구현 후 난이도 차이
   - 현재 tree와 선행 정책 구현 후 모두 `1/5 — 매우 낮음`이다. 4·6·14·16·17번이 구현되면 각 command에
     Pause consumer test가 늘어나지만 공통 Pause runtime을 변경하지 않고 같은 authority/clock seam을
     재사용한다.
   - 해당 테스트 비용은 각 신규 command vertical slice에도 포함되며 15번에서 중복 infrastructure를
     만들지 않는다.
8. 난이도 책정 reference 파일과 책임
   - Core command/wire
     - `core_enhanced/src/game/behavior.rs`: playback state/speed, Pause/Resume/SetSpeed command, battle authority
       요구 여부, playback eligibility와 checkpoint wire를 소유한다.
     - `core_enhanced/src/game/battle/event_log.rs`: 같은 battle time을 seq로 구분하고
       `source_command_id`를 correlation field로 보존하는 semantic event entry를 정의한다.
   - Core world transaction/state
     - `core_enhanced/src/game/world.rs`: battle mutation authority를 domain 실행 전에 검증하고 mutation과
       read-only behavior를 battle handler로 routing한다.
     - `core_enhanced/src/game/world/battle_capability.rs`: Pause 중에도 deploy/withdraw/skill/query capability를
       유지하며 Pause/Resume만 playback state로 전환한다.
     - `core_enhanced/src/game/world/combat.rs`: pause-aware production tick, Pause/Resume/SetSpeed transaction,
       read-only state/setup recovery와 일반 battle mutation handlers를 소유한다.
     - `core_enhanced/src/game/world/state.rs`: playback flag, raw tick→simulation delta, battle-local resource
       회복, revision commit과 current checkpoint/resync projection을 소유한다.
   - Battle runtime
     - `core_enhanced/src/game/battle/core/sim.rs`: playback을 모른 채 전달받은 target battle time까지만 event
       queue를 진행하고 live command를 현재 battle time에 적용하는 simulation 경계다.
   - Policy/live data
     - `game_resources/data/run/policy.ron`: 현재 자연 회복 초당 1과 redeploy battle-time 수치를 authoring하며
       Pause 전용 wall-clock/tuning field는 없다.
     - `game_resources/data/buffs/base.ron`: poison cadence와 typed control buff를 authoring하지만 실제 tick과
       expiry 진행 시점은 BattleCore의 authoritative battle time이 소유한다.
   - Game server transport
     - `game_server/src/game/player_game_actor/handlers.rs`: synchronous actor command 처리, pause-aware live tick,
       ack→update와 stale/missing error→resync 전송 순서를 소유한다.
     - `game_server/src/game/player_game_actor/messages.rs`: request ID, optional battle authority와 battle
       update/resync envelope를 정의한다.
     - `game_server/src/game/player_game_actor/state.rs`: Core battle result를 accepted battle time/revision ack와
       ordered side message로 변환한다.
     - `game_server/docs/unity_bridge_protocol.md`: mutation의 exact revision 의무, query 예외, ack/update 및
       stale error/resync 순서와 event seq가 revision 대체물이 아니라는 server canonical을 명시한다.
   - 외부 Unity runtime/contract/tests
     - 사용자 결정에 따라 7번부터 Unity 조사를 제외했으며 15번 난이도에도 Unity 구현 비용을 포함하지
       않았다.
9. 실행한 focused test와 결과
   - `cargo test -p game_core live_defense_playback_pause_freezes_server_tick_and_resume_advances` — 통과(1개).
     Pause/repeat Pause/query/resume/repeat Resume, frozen server tick과 command별 revision 기반을 확인했다.
   - `cargo test -p game_core supplied_stale_battle_authority_rejects_before_playback_mutation` — 통과(1개).
     Missing/stale authority가 playback/revision을 바꾸지 않고 current authority만 성공하는 기반을 확인했다.
   - `cargo test -p game_core battle_revision_overflow_rejects_mutation_without_changing_playback` — 통과(1개).
     Revision commit 불가를 state mutation 전에 거부하는 기반을 확인했다.
   - `cargo test -p game_core live_defense_setup_loss_recovery_preserves_battle_revision_and_state` — 통과(1개).
     Setup-loss recovery가 paused state와 current revision/event cursor를 보존하는 기반을 확인했다.
   - `cargo test -p game_server paused_live_battle_tick_does_not_push_delta_until_resumed` — 통과(1개).
     Paused actor tick 무출력, ack→paused update 순서와 Resume 뒤 delta 재개를 확인했다.
   - `cargo test -p game_server stale_battle_authority_emits_error_then_current_resync_without_mutation` —
     통과(1개). Stale mutation의 request-scoped error→current resync 순서를 확인했다.
   - 이 통과 결과는 기존 Pause/authority/transport 기반이 안정적이라는 증거다. 아직 Pause 중 일반
     gameplay mutation sequence와 향후 stance/relocation/swap/targeted skill/work가 검증됐다는 증거는
     아니다.
10. 아직 사용자가 확정해야 할 tuning 또는 선택
    - 없음. 별도 balance tuning은 두지 않고 현재 tree와 선행 정책 구현 후 난이도 모두
      `1/5 — 매우 낮음`으로 확정했다.
    - 현재처럼 Pause 중 `SetBattleSpeed`를 허용해 frozen clock은 유지하되 Resume 뒤 사용할 stored speed를
      즉시 변경하고 revision을 1 증가시키는 의미로 확정했다. 별도 resume-speed command나 Pause 전용
      speed state는 만들지 않는다.

### 16. 수동 지정 스킬 — 정책·코드 대조·난이도·첫 live content 확정 완료

확정일: 2026-08-06  
난이도·첫 live content 확정일: 2026-08-06  
구현 난이도: 현재 tree `4/5 — 높음`, 4·6·13번 선행 후 증분 `3/5 — 중간`

확정 정책:

- 스킬의 기준 대상 종류와 effect 의미는 기존 `SkillTarget`이 소유하고, 누가 cast target을 선택하는지는
  별도 `SkillCastSelection::Self | AutomaticUnit | PlayerUnit | PlayerTile`이 소유한다. 모든 live skill은
  selection을 명시적으로 작성하며 숨은 default나 weapon archetype 기반 추론은 두지 않는다.
- `Self`는 caster 자신을 대상으로 하고 별도 선택을 요구하지 않는다. `AutomaticUnit`은 플레이어가
  manual skill 발동만 결정하고 Core가 기존 authored targeting rule로 Unit을 고른다.
- `PlayerUnit`은 explicit `Unit { unit_instance_id }`, `PlayerTile`은 explicit `Tile { position }` target이
  필수다. Target이 없거나 selection과 payload 종류가 다르면 자동 target fallback 없이 typed reason으로
  거부한다.
- 최종 command acceptance에서 caster의 active Player identity, manual activation mode, `Attack` stance,
  action/activity, Hard CC/Silence, 공명·cooldown과 target을 모두 먼저 검증한다. 하나라도 실패하면 skill,
  resource/resonance, cooldown, event와 revision을 전혀 변경하지 않고 manual targeting focus를 유지한다.
- `PlayerUnit`은 authored target rule, active/lifecycle, ally/hostility, air eligibility, target usefulness와
  final cast reach/range를 검사한다. `PlayerTile`은 battlefield valid tile과 final cast reach/range를
  검사하되 현재 occupant나 유효 Unit target 존재를 요구하지 않는다.
- 성공하면 server가 command acceptance 시점의 Unit identity 또는 Tile을 authoritative cast target으로
  캡처하고 skill cast/focus를 시작한다. Unity click time의 위치나 preview state는 authority가 아니다.
- `PlayerUnit` direct effect는 실행 시 같은 Unit identity를 사용한다. 실행 시 target이 죽거나 authored
  eligibility를 잃었다면 해당 effect는 no-op으로 끝내고 다른 Unit을 찾지 않는다.
- `PlayerUnit`을 기준으로 하는 `TileArea`는 command acceptance 시 target의 authoritative tile을 anchor로
  캡처한다. 이후 Unit이 이동·사망·철수해도 area는 그 tile에서 실행한다. `PlayerTile`도 acceptance한
  authoritative tile을 끝까지 유지한다.
- Command acceptance 뒤 target이 무효화돼도 자동 retarget하거나 이미 반영된 resonance/resource/cooldown을
  환불·rollback하지 않는다. Hard CC cast 취소, Silence와 자발적 철수의 상호작용은 각각 13번과 3번
  정책을 그대로 따른다.
- 첫 버전에서는 `PlayerUnit/PlayerTile + RetargetOnStep` 조합을 strict live validation으로 금지한다.
  범용 retarget/refund system, target reservation과 별도 target-follow state는 만들지 않는다.
- `PlayerUnit/PlayerTile` 선택은 전술 이동과 공통 server-authoritative `전술 집중 모드`를 사용한다.
  실행 중 전투에서는 run policy의 공통 `tactical_focus_speed: X0_5`를 적용하고 이미 Pause라면 frozen
  playback을 그대로 유지한다.
- Focus state는 `purpose: ManualSkillTargeting`, caster와 skill ID, 복원할 이전 playback state만 소유한다.
  Hover와 아직 제출하지 않은 selected target은 Unity-local preview이며 Core state/revision을 바꾸지 않는다.
- Core의 read-only target preview는 현재 revision 기준 candidate Unit IDs 또는 final Tile cells를 반환한다.
  최종 `ActivateSkill`은 같은 조건을 다시 원자적으로 검증한다. Preview는 command acceptance나 target
  reservation이 아니다.
- 성공하면 skill 시작, 이전 playback 복원과 focus 종료를 같은 authoritative commit으로 처리한다.
  실패하면 gameplay state/revision과 focus를 바꾸지 않는다. 명시적 cancel은 비용 없이 local preview를
  버리고 이전 playback을 복원한다.
- Reconnect/resync는 focus purpose, caster/skill과 복원할 playback state를 보존하지만 제출되지 않은 local
  target selection은 복구하지 않는다. Pause에서 진입한 focus는 성공·실패·취소 뒤에도 Pause를 유지한다.
- Target이 필요 없는 `Self` 또는 `AutomaticUnit` manual skill은 targeting focus 없이 완성된 command를
  즉시 보낸다. 기본 공격은 계속 자동 targeting을 사용하며 16번이 수동 기본 공격을 도입하지 않는다.
- 첫 실제 `PlayerUnit/PlayerTile` live skill과 그 content 수치는 코드·live data 대조 뒤 별도로 확정한다.

코드·live data 대조 결과:

1. 확정 정책
   - Cast effect의 대상 의미와 선택 주체를 분리해 모든 skill이 `Self/AutomaticUnit/PlayerUnit/PlayerTile`
     중 하나를 명시한다. Player 대상은 server acceptance에서 완전 검증·고정하고 누락, 잘못된 payload,
     실행 시 무효화에 fallback·retarget·refund를 제공하지 않는다.
   - `PlayerUnit/PlayerTile`만 공통 server-authoritative 전술 집중 모드와 read-only candidate preview를
     사용한다. 최종 `ActivateSkill` 성공만 skill 시작, playback 복원, focus 종료와 revision 증가를 한
     commit으로 만든다.
2. 현재 구현된 기반
   - `PlayerBehavior::ActivateSkill`과 `BattleLiveCommand::ActivateSkill`은 이미 optional
     `SkillCastTarget::Unit/Tile`을 Core까지 전달한다. World는 current battle authority를 먼저 검사하고,
     skill preflight와 revision overflow 사전 검사를 통과한 뒤 BattleCore command를 적용하고 projection을
     한 번 commit한다.
   - BattleCore는 Player active unit, `Manual` activation mode, Hard CC, Silence, action lock, 일치하는
     skill ID, full resonance와 target을 검사한다. `SelfUnit`은 self를, `EnemySingle`은 authored rule로
     자동 선택하며 explicit enemy Unit은 active hostility, air eligibility와 cast reach를 검사한다.
   - `PendingSkillCast`가 선택한 Unit identity 또는 Tile을 cast end까지 보존한다. `ReuseCastTarget` direct
     enemy step은 같은 Unit이 실행 시 무효하면 대상 없이 끝내고 다른 Unit을 찾지 않는다.
   - Unit target의 tile-area용 위치 snapshot과 world 위치를 `ActiveSkillCast`에 보존하는 seam이 있다.
     현재 focused test는 target이 사라진 뒤에도 그 snapshot의 area가 실행되고, direct target이 죽으면
     다른 target을 재획득하지 않음을 고정한다.
   - Checkpoint에는 player unit별 skill readiness와 static final range preview가 있고 root skill catalog는
     cast target/reach/range와 step delivery를 투영한다. `BattlePlaybackSpeed::X0_5`, Pause와 stored speed,
     revisioned reconnect/resync 기반도 이미 있다.
   - Game server는 Core `PlayerBehavior`를 직접 deserialize하고 exact battle authority를 Core로 넘기며,
     accepted update와 stale error→current resync를 generic transport로 전달한다.
3. 정책과 일치하는 부분
   - Unit/Tile wire identity, manual activation path, typed Hard CC/Silence/readiness reason, server-authoritative
     validation과 revision commit 순서는 신규 수동 지정 명령이 사용할 수 있는 실제 기반이다.
   - Explicit enemy Unit의 hostility, lifecycle, air eligibility와 final cast reach 검사가 이미 acceptance
     preflight에 있고 잘못된 explicit target은 `no_valid_target`으로 skill 시작 전에 거부된다.
   - `ReuseCastTarget`은 direct Unit target에 execution-time range를 다시 적용하거나 다른 Unit을 찾지
     않는다. target이 죽으면 no-op이고 tile area에는 captured anchor fallback이 있어 no-retarget 정책과
     큰 방향이 맞는다.
   - Static tile range 계산은 facing을 적용하고 battlefield valid tile로 clip한다. 기존 preview DTO와
     skill catalog가 있으므로 Unity가 RON range를 재해석하게 만들 필요가 없다.
   - 현재 live manual active skill은 fragment 장착으로 `SkillActivationMode::Manual`이 되고 기존
     end-to-end test에서 full resonance, readiness, command, cast event와 resonance reset까지 연결된다.
4. 정책과 다른 부분
   - `SkillCastSelection`이 없다. 현재 selection은 `SkillTarget`에서 암묵적으로 추론되어 `SelfUnit`은 Self,
     `EnemySingle`은 항상 automatic Unit, `CastTarget`은 사실상 manual Tile로 취급된다. 모든 live RON과
     skill catalog에도 선택 주체 field가 없다.
   - `PlayerUnit`/`PlayerTile` 필수 payload 의미가 없다. 특히 cast-level `CastTarget`은 target이 없어도
     manual cast validation을 통과하고, explicit Tile은 battlefield valid 여부만 검사해 authored
     pattern/global reach를 적용하지 않는다.
   - Explicit Unit validation은 target usefulness를 검사하지 않는다. Automatic target selector만 damage,
     hostile buff, heal 등 step usefulness를 사용하므로 같은 skill의 자동 선택과 explicit 선택 eligibility가
     갈릴 수 있다.
   - Acceptance 때 `PendingSkillCast`에는 Unit identity만 저장한다. Unit의 tile/world anchor는
     `focus_time_ms`가 끝나 `invoke_ability`가 실행되는 시점에 캡처하므로, target이 cast 중 이동하면 확정
     정책의 command-acceptance tile이 아니라 이동 후 tile에서 area가 실행된다.
   - Current manual cast는 acceptance에서 resonance를 소비하지 않고 정상 `ManualCastEnd`에서 0으로 만든다.
     중간 interruption/withdraw는 pending cast만 제거하므로 이미 accepted action의 공명을 환불하지 않는
     13·16번 정책과 다르다. 별도 active-skill cooldown model은 없고 full resonance와 `next_action_time`이
     현재 readiness를 맡는다.
   - `RetargetOnStep` 자체는 구현되어 있고 live step 4개가 사용한다. Selection과의 조합 validation이 없어
     향후 Player 선택 skill에 그대로 authoring하면 acceptance target과 무관한 재탐색이 일어난다.
   - 동적 candidate Unit ID query는 없다. Checkpoint range preview는 static tile cell만 계산하고
     `EnemySingle` preview는 `requires_runtime_target`으로 비활성 표시한다. Tile preview도 전용 query가
     아니라 checkpoint에 포함된 현재 unit 위치/facing 기준 projection이다.
   - `ActiveBattleSession`에는 tactical focus state가 없고 run policy에도 `tactical_focus_speed`가 없다.
     Begin/cancel command, focus purpose/caster/skill/restore playback DTO, modal capability gate, reconnect 보존,
     Hard CC/death/withdraw 때 focus 정리와 성공 시 원자적 playback 복원이 모두 미구현이다.
   - `Attack/Defense` stance도 아직 없어 manual skill acceptance가 `Attack`을 검사하지 못한다. Hard CC는
     pending cast를 취소하지 않고 cast end를 CC 종료까지 미루는 현재 동작이라 13번의 release 전 취소
     정책과도 아직 다르다.
5. 필요한 구현 범위
   - `SkillCastTargetingDef::Explicit`의 strict field로 `SkillCastSelection`을 추가하고 raw RON loader,
     validator와 skill catalog version을 함께 갱신한다. `Self↔SelfUnit`, `AutomaticUnit/PlayerUnit↔Unit target`,
     `PlayerTile↔CastTarget` 조합과 `PlayerUnit/PlayerTile + RetargetOnStep` 금지를 data load 때 검증한다.
   - 25개 live skill을 숨은 default 없이 migration한다. 현재 cast-level 구성은 `EnemySingle` 21개,
     `SelfUnit` 4개이고 실제 `CastTarget`은 0개이므로 첫 migration은 기존 의미를 보존하는
     `AutomaticUnit` 21개와 `Self` 4개로 쓸 수 있다. live `RetargetOnStep` 4개는 AutomaticUnit에만 남긴다.
   - Manual preflight를 selection별로 분기해 required target 누락과 target kind mismatch를 stable typed
     reason으로 거부한다. PlayerUnit은 authored relation/air/usefulness/reach를, PlayerTile은 valid tile과
     동일한 pattern/global reach를 검사하며 어떠한 automatic fallback도 호출하지 않는다.
   - Acceptance snapshot에 Unit identity, 선택 당시 authoritative tile과 필요하면 world anchor를 함께
     보존한다. Direct Unit step은 identity를, Unit-anchor TileArea와 PlayerTile은 이 acceptance anchor를
     사용하도록 `PendingSkillCast`→`ActiveSkillCast` 전달을 바꾼다.
   - Manual skill의 공명 소비와 향후 authored cooldown 시작 경계를 acceptance commit으로 옮겨 Hard CC,
     withdraw와 execution-time target invalidation 뒤 환불하지 않는다. 별도 skill cooldown은 16번만을
     위해 새로 만들지 않고 현재 resonance/action eligibility를 사용하며, 독립 cooldown을 원한다면 별도
     skill balance/data 정책으로 추가한다.
   - 4번과 공유하는 tactical focus state, `BeginTacticalFocus`/`CancelTacticalFocus`, modal action gate,
     run-policy speed, checkpoint/resync projection과 previous playback 복원을 구현한다. Player target 성공은
     BattleCore cast 시작과 focus 종료를 하나의 World transaction/revision으로 commit하고 실패는 focus를
     유지한다.
   - Read-only `RequestSkillTargetPreview` 계열 query/response를 추가해 current revision, caster/skill과
     sorted candidate Unit IDs 또는 final Tile cells를 반환한다. Query는 reservation이 아니며 기존
     `ActivateSkill` preflight를 final authority로 유지한다.
   - 6번 stance와 13번 central capability를 연결하고 focus source가 Hard CC/death/withdraw 등으로
     무효해질 때 local selection을 버릴 수 있는 authoritative focus projection을 갱신한다. 전술 집중 중
     허용 command는 이미 승인된 modal allowlist로 중앙 gate한다.
   - Game server에는 별도 targeting 판단을 넣지 않고 새 Core command/query와 DTO를 direct transport한다.
     strict JSON, missing/stale authority, failed acceptance 무변경, focus reconnect/resync와 ordered update를
     focused contract test로 고정한다.
6. 현재 tree 기준 난이도와 이유
   - **제안은 `4/5 — 높음`**이다. Unit/Tile wire와 skill runtime seam은 있어 핵심 simulation을 새로
     설계하는 5/5는 아니지만, 완성된 vertical slice는 strict skill data migration, target validator와
     acceptance anchor, resonance 소비 경계, read-only preview, tactical focus/playback state, World
     transaction/checkpoint, server transport를 함께 바꾼다.
   - 특히 PlayerUnit/PlayerTile validator만 추가하는 것은 작은 변경이지만 그것만으로는 확정 정책의
     0.5x focus, reconnect, modal authority와 atomic restore를 충족하지 못한다. 따라서 “기반 존재”를
     “기능 구현”으로 보아 2~3/5로 낮출 수 없다.
7. 선행 정책 구현 후 난이도 차이
   - 4번의 공통 tactical focus/playback/checkpoint/modal command 기반, 6번의 stance state/capability,
     13번의 중앙 Hard CC/Silence 취소·eligibility가 먼저 실제 구현되면 **증분 `3/5 — 중간`**으로
     내려간다.
   - 그 뒤에도 selection schema와 25개 live migration, PlayerUnit/PlayerTile final validator, acceptance
     anchor, dynamic preview와 server contract test는 16번 고유 작업이라 `2/5`까지 내려가지는 않는다.
8. 난이도 책정 reference 파일
   - Core command/wire
     - `core_enhanced/src/game/behavior.rs`: `ActivateSkill`, Unit/Tile target import, playback, skill readiness,
       range preview, skill catalog와 action/disabled reason DTO를 소유해 신규 focus/query wire의 주 경계다.
     - `core_enhanced/src/game/battle/event_log.rs`: 기존 `SkillCastTarget::Unit/Tile`과 manual cast/ability/area
       semantic event를 소유해 acceptance target이 runtime과 presentation으로 전달되는 기반이다.
   - Core world transaction/state
     - `core_enhanced/src/game/world.rs`: battle authority 선검사, action gate와 PlayerBehavior routing을
       소유해 focus modal allowlist와 query/mutation 구분이 들어갈 경계다.
     - `core_enhanced/src/game/world/combat.rs`: manual skill preflight→BattleCore command→projection commit의
       현재 transaction을 소유하며 성공 시 focus restore/close를 함께 묶어야 한다.
     - `core_enhanced/src/game/world/state.rs`: `ActiveBattleSession` playback/revision/checkpoint와 Pause tick
       scaling을 소유하며 tactical focus와 restore playback의 authoritative 저장 위치다.
     - `core_enhanced/src/game/world/snapshot.rs`: skill catalog version 2와 cast target/reach/range projection을
       소유해 selection field의 strict catalog migration 대상이다.
   - Battle runtime
     - `core_enhanced/src/game/ability.rs`: `SkillTarget`, `SkillCastTargetingDef`, `StepTargetingMode`와 skill
       focus/cast data를 소유하며 새 selection enum 및 금지 조합의 domain schema 경계다.
     - `core_enhanced/src/game/battle/core/types.rs`: `PendingSkillCast`와 `ActiveSkillCast`가 현재 target/anchor를
       보존해 command-acceptance anchor snapshot을 추가할 내부 state 경계다.
     - `core_enhanced/src/game/battle/core/sim.rs`: manual validation, automatic/explicit target resolution,
       cast start/end, anchor capture와 `ReuseCastTarget/RetargetOnStep` 실행 의미를 소유한다.
     - `core_enhanced/src/game/battle/core/skill_runtime/cast.rs`: direct step의 execution-time active/hostility/
       usefulness 검사와 invalid Unit no-op 동작을 소유한다.
     - `core_enhanced/src/game/battle/core/skill_runtime/area.rs`: Unit/Tile area anchor를 current 위치 또는 stored
       cast anchor로 해석해 acceptance snapshot 사용 여부를 바꿀 경계다.
     - `core_enhanced/src/game/battle/tile_range.rs`: facing 기반 pattern과 final tile cell 계산의 공통 자료형을
       소유한다.
     - `core_enhanced/src/game/range_preview.rs`: 현재 Self/AutoUnit/ManualTile 추론과 static final tile preview를
       구현해 explicit selection 및 dynamic candidate query와 의미를 맞춰야 한다.
   - Policy/live data
     - `core_enhanced/src/game/data/skill_data.rs`: strict raw RON loader와 target/reach/delivery contract를
       소유해 selection 필수값과 Player selection/retarget 금지 validation을 추가할 곳이다.
     - `game_resources/data/skills/base.ron`: live skill 25개, cast-level EnemySingle 21/SelfUnit 4와
       RetargetOnStep 4개를 실제 authoring하며 selection migration 및 첫 PlayerUnit content의 대상이다.
     - `core_enhanced/src/game/data/run_policy_data.rs`: 현재 battle runtime/live deployment policy만 있고
       tactical focus speed가 없어 공통 focus policy field와 validation을 추가할 schema다.
     - `game_resources/data/run/policy.ron`: live `battle_runtime` 값만 있고 `tactical_focus_speed`가 없어
       확정된 `X0_5`를 authoring할 source다.
     - `core_enhanced/src/game/skill_fragment.rs`: active fragment를 employee combat profile의 Manual skill로
       연결해 실제 PlayerUnit content가 live player command로 들어오는 기반이다.
     - `game_resources/data/skill_fragments/base.ron`: Black Round fragment의 imitation/awakened skill 연결과
       Gun compatibility를 소유해 첫 live PlayerUnit 후보 progression을 확인한 source다.
   - Game server transport
     - `game_server/src/game/player_game_actor/messages.rs`: Core PlayerBehavior의 direct JSON transport,
       battle authority와 update/resync envelope를 소유해 신규 focus/query wire의 strict contract 대상이다.
     - `game_server/src/game/player_game_actor/handlers.rs`: Core 실행 전후 rollback/persistence와 authority
       rejection의 error→resync 순서를 소유하며 failed target acceptance 무변경을 검증할 경계다.
     - `game_server/src/game/player_game_actor/state.rs`: Core typed error와 battle result를 server error/update로
       변환해 신규 stable target reason을 전달할 generic mapping이다.
     - `game_server/docs/unity_bridge_protocol.md`: exact revision, Core-owned payload eligibility, atomic
       projection과 stale recovery를 명시한 현재 server canonical이며 focus/query 계약 갱신 대상이다.
   - 외부 Unity runtime/contract/tests
     - 사용자 결정에 따라 조사하지 않았고 16번 난이도에도 별도 Unity 구현 비용을 포함하지 않았다.
9. 실행한 focused test와 결과
   - `cargo test -p game_core live_manual_skill_command_starts_cast_and_reuses_skill_runtime` — 통과(1개).
     기존 Unit-independent manual command가 cast runtime과 resonance reset까지 연결되는 기반을 확인했다.
   - `cargo test -p game_core activate_skill_uses_equipped_manual_fragment_in_live_defense` — 통과(1개).
     Live employee fragment의 Manual readiness, World command, event/update 연결을 확인했다.
   - `cargo test -p game_core manual_fragment_readiness_disables_payload_when_target_is_missing` — 통과(1개).
     Automatic EnemySingle에 유효 target이 없을 때 readiness와 command가 typed reason으로 거부되는 현재
     preflight 기반을 확인했다.
   - `cargo test -p game_core active_skill_preview_clips_pattern_to_valid_tiles` — 통과(1개). 기존
     CastTarget static preview가 facing pattern을 battlefield valid cells로 clip하는 기반을 확인했다.
   - `cargo test -p game_core global_target_area_uses_captured_tile_after_target_disappearance_without_facing` —
     통과(1개). Ability 실행 시점에 만들어진 Unit anchor snapshot이 target 소멸 뒤 area에 쓰이는 기반을
     확인했다.
   - `cargo test -p game_core targeted_execute_does_not_reacquire_when_locked_target_dies` — 통과(1개).
     Reused direct Unit이 죽으면 다른 Unit을 재탐색하지 않는 기반을 확인했다.
   - `cargo test -p game_core manual_pending_cast_can_be_interrupted_and_stale_end_is_ignored` — 통과(1개).
     Pending manual cast identity와 stale CastEnd 무효화 기반을 확인했다.
   - `cargo test -p game_core --test live_skill_catalog_audit` — 통과(4개). 현재 25개 live skill data의
     delivery/reference ownership이 안정적임을 확인했다.
   - `cargo test -p game_server deserializes_live_battle_requests` — 통과(1개). 기존 target optional
     `ActivateSkill`과 battle authority의 direct server transport 기반을 확인했다.
   - 이 통과 결과는 기존 manual skill, target identity, static range, no-retarget와 server wire 기반이
     안정적이라는 증거다. `SkillCastSelection`, PlayerUnit/PlayerTile final validation, acceptance anchor,
     target preview query와 tactical focus가 구현됐다는 증거는 아니다.
10. 난이도·tuning·첫 live content 확정 결과
    - 현재 tree `4/5`, 4·6·13번 선행 후 증분 `3/5`로 확정했다. 현재 tree에서도 정책 전체를 구현할 수
      있으며 이 등급은 simulation 재설계 위험이 아니라 data/runtime/focus/world/server vertical slice의
      통합량을 반영한다.
    - 공통 `tactical_focus_speed: X0_5`를 유지하며 16번 고유 추가 수치는 두지 않는다.
    - 별도 active-skill cooldown model은 추가하지 않는다. Full resonance와 기존 action/cast lock을
      readiness로 유지하고, 독립 cooldown이 실제로 필요해지면 combat cadence와 skill power를 조사하는
      별도 balance/data 정책으로 분리한다.
    - 첫 live `PlayerUnit`은 Gun-compatible `fragment_freischutz_black_round`과 awakened
      `freischutz_magic_bullet` progression을 함께 전환한다. 두 skill의 기존 front 3x2, airborne 불가,
      focus time, damage, projectile와 self effect 수치는 바꾸지 않고 선택 주체만 `PlayerUnit`으로 바꾼다.
    - 첫 `PlayerTile` live skill은 만들지 않는다. Runtime/data validation과 fixture contract는 완성하되,
      자연스러운 area content가 별도로 설계될 때 명시적으로 authoring한다.
    - 16번에는 더 확정할 tuning이나 선택이 없다.

### 17. 현장 작업 — 정책·코드 대조·난이도·첫 live content 확정 완료

확정일: 2026-08-06  
난이도·첫 live content 확정일: 2026-08-06  
구현 난이도: 현재 tree `4/5 — 높음`, 4·5·6·8·9·11·12·13·14번 선행 후
`3/5 — 중간`

확정 정책:

- `tactical_points`나 기존 `static_objects`에 작업 상태를 덧붙이지 않고 setup에 별도
  `FieldWorkObjectDef`를 둔다. 각 object는 stable ID, presentation/object kind, position, 하나 이상의
  work cell, positive `required_work_ms`와 첫 vertical slice에 필요한 typed completion effect를 명시한다.
- 첫 버전의 progress interruption 의미는 `Preserve` 하나다. 사용하지 않는 `Reset/Preserve` enum과
  Reset 분기·초기화 규칙은 미리 만들지 않는다. 모든 worker가 이탈해도 object의
  `accumulated_work_ms`는 보존하고 다음 유효 worker가 남은 지점부터 이어받는다.
- 별도 `AssignFieldWork`/`CancelFieldWork` command는 두지 않는다. Active Player unit이 authored work
  cell을 점유하면 assignment를 자동 시작하고 cell을 벗어나거나 death/withdraw하면 자동 종료한다.
  최초 배치와 기존 회수/전투불능 뒤 재배치로 work cell에 직접 들어가는 것도 허용하며 배치 commit과
  같은 mutation에서 `Defense + FieldWorking`이 된다.
- Field work 자체에는 전술 포인트 비용이 없다. Work cell 진입에 사용한 배치·재배치·전술 순간이동·자리
  교대 command가 소유한 기존 비용과 cooldown만 적용한다.
- 빈 work cell로 전술 순간이동한 `Defense` source는 이동 commit과 같은 mutation에서 worker가 된다.
  자리 교대로 work cell에 들어간 직원은 worker가 되고, cell에서 밀려난 기존 worker는 assignment를
  종료한 뒤 최종 tile의 일반 `Defense`와 tile-kind effect로 즉시 복귀한다. Assignment는 직원을 따라
  이동하지 않으며 모든 위치 mutation 뒤 최종 점유를 기준으로 원자적으로 reconcile한다.
- `FieldWorking` worker는 새 basic attack과 automatic/manual skill, block, stance toggle을 할 수 없고
  Ground `DefenseFocus`와 Platform `Rest`도 받지 않는다. 다만 자발적 철수와 전술 순간이동·자리 교대의
  source/target이 될 수 있으며 해당 mutation으로 work cell을 떠날 수 있다.
- Worker는 계속 피해를 받는다. 적이 같은 tile에 겹치거나 worker를 공격하는 사실 자체는 assignment를
  종료하거나 progress를 중단하지 않는다. Enemy occupancy는 player-side work-cell assignment 자격이나
  `1 tile = 1 allied unit` 입력 규칙을 바꾸지 않는다.
- 각 work cell은 player command의 `1 tile = 1 active allied unit` 규칙을 따른다. 현재 유효한 worker는
  authoritative battle time `1ms`당 object의 `accumulated_work_ms`를 `1ms` 증가시키며 여러 work cell이
  동시에 점유되면 worker별 기여를 합산한다. 누적값은 `required_work_ms`에서 clamp한다.
- Hard CC는 assignment와 object progress를 보존한 채 해당 worker의 기여만 일시정지한다. Assignment에
  별도 authoritative `suspended` bool이나 suspension lifecycle을 저장하지 않고 현재 중앙
  `hard_cc_active` capability에서 기여 가능 여부를 파생한다. 필요하면 checkpoint가 표시용 derived
  `suspended`를 투영할 수 있다. Hard CC가 끝나면 남은 지점부터 재개하며 CC 시간을 소급하지 않는다.
- Pause는 assignment suspension이 아니라 global battle clock 정지다. Pause 중 cell 진입·이탈 mutation은
  15번의 즉시 직렬 규칙으로 처리하지만 progress는 흐르지 않고 Resume 뒤 정지 시간을 소급하지 않는다.
- 최초로 `accumulated_work_ms >= required_work_ms`가 된 authoritative update에서 completion을 정확히 한
  번 처리한다. Object를 `Completed`로 바꾸고 work cell을 일반 underlying tile로 되돌리며, 남아 있는
  worker assignment를 같은 mutation에서 끝내고 각 unit을 일반 `Defense`와 현재 tile-kind effect로
  복귀시킨다.
- Assignment start/stop과 object completion만 semantic event로 보낸다. Current object progress,
  completion과 work-cell worker mapping은 checkpoint/resync가 소유하며 stable object/work-cell/unit
  ordering을 사용한다. Event seq나 HashMap iteration은 progress 또는 completion authority가 아니다.
- 현장 작업이 승리·실패 조건에 관여하려면 encounter가 이를 명시적으로 authoring한다. 범용 scripting
  engine이나 skill effect 전체 재사용을 만들지 않고 첫 vertical slice에 필요한 typed effect만 구현한 뒤
  두 번째 실제 content에서 공통화를 재검토한다.
- 첫 live content는 `suppress_freischutz`의 1회성 `black_box_emergency_repair`다. Presentation kind는
  `EmergencyRepairTerminal`, object position과 단일 work cell은 세 Ambush/Medium template에 공통인 Ground
  deployment cell `(2, 6)`, `required_work_ms`는 `10,000`으로 확정한다.
- 첫 completion effect는 `RepairProtectedUnit { flat_hp: 100 }` 하나다. 보호 대상이 이미 최대 HP이면 기존
  HP clamp에 따라 실제 회복량 0으로 완료·소모하고 반복 사용하지 않는다.

코드·live data 대조 결과:

1. 확정 정책
   - Field work는 전술 지점이나 기존 static object에 상태를 얹지 않는 별도 authored object이며,
     work-cell 점유에서 assignment를 자동 파생한다. Object가 Preserve-only 누적 progress와 완료를
     소유하고 worker는 exclusive `FieldWorking` activity가 된다.
   - Hard CC 동안 assignment와 progress는 보존하되 worker 기여만 중앙 `hard_cc_active` capability에서
     파생해 정지한다. 별도 suspension state, Assign/Cancel command, Reset 분기와 범용 completion script는
     만들지 않는다.
2. 현재 구현된 기반
   - `BattleScenario`는 battlefield, stable scenario unit ref, tactical plan, event와 win condition을
     검증하고 `BattleCore`는 scenario와 runtime state를 함께 소유한다. Work object는 없지만 새
     scenario-owned state를 둘 수 있는 경계는 있다.
   - Battle execution은 authoritative battle time의 event bucket을 stable priority와 ID tie-break로
     처리한다. Pause는 World에서 tick delta 자체를 만들지 않으므로 battle time이 멈춘다.
   - Protect mission은 route endpoint에 HP 350의 synthetic `DefenseObject`를 stable scenario unit ref로
     만들고 enemy target 및 win condition과 연결한다. Active unit HP를 maximum에서 clamp하며 변경 event를
     기록하는 공통 mutation도 있다.
   - Deployment는 authored zone와 weapon affinity를 검증하고 Core deploy, battle-local state, checkpoint와
     revision을 한 transaction에서 commit한다. Setup snapshot, event delta, checkpoint/resync와 server의
     generic forwarding 경계도 이미 있다.
3. 정책과 일치하는 부분
   - 현재 battlefield tile은 Ground/Platform/Obstacle만 소유하므로 work cell을 새 terrain kind로 만들지
     않고 underlying deployment tile 위의 별도 overlay로 두는 확정 방향과 맞는다.
   - BattleCore는 같은 tile의 enemy overlap을 허용하고 player deployment의 점유 제한은 별도 입력 정책으로
     추가하는 구조다. 따라서 enemy가 worker tile에 겹쳐도 assignment를 끊지 않는 의미를 simulation 전역
     점유 불변식으로 만들 필요가 없다.
   - Hard CC의 active expiry 조회와 Pause의 battle-clock freeze가 있어 suspension bool이나 wall-clock
     보정 없이 worker 기여 eligibility를 계산할 수 있다.
   - `suppress_freischutz`는 `black_box_device` 보호 계약을 명시하고 Ambush/Medium의 세 seeded template 모두
     `(2, 6)`을 Ground deployment cell로 제공한다. 첫 work cell을 해당 좌표에 두면 현재 template 선택
     방식에서도 항상 유효하다.
4. 정책과 다른 부분
   - Core, live RON과 server에 `FieldWorkObjectDef`, object runtime state, assignment, progress, completion
     effect, semantic/presentation event와 setup/checkpoint DTO가 전혀 없다. 현재 `static_objects`도 setup에서
     빈 배열이며 defense object는 unit checkpoint로만 나타난다.
   - `PveEncounter`와 `TacticalPlan`은 tactical point와 objective만 운반한다. Work object의 stable ID,
     위치/work cell, required work와 effect를 deserialize·cross-validate하거나 scenario로 복사하지 않는다.
   - BattleCore의 basic attack, skill, blocking과 Hard CC gate에는 `FieldWorking` activity가 없다. Deploy,
     death와 withdraw 뒤 assignment reconcile도 없고 전술 이동·교대·태세·Ground/Platform 효과 자체가 아직
     미구현이다.
   - Checkpoint는 unit/deployment만 소유하고 field object progress·completion·worker mapping이 없다.
     Event log와 presentation DTO에도 assignment start/stop/completion variant가 없으며 setup version은 1이다.
   - 서버 tick이 끝난 뒤 World에서만 progress를 더하면 같은 tick 안의 Hard CC, death와 위치 변경 경계를
     잃는다. `BattleCore::step_battle_execution_until`은 다음 event가 target보다 늦으면 time만 바로 전진할
     수 있으므로 현재 후처리 seam만으로는 정확한 completion 시점도 만들 수 없다.
5. 필요한 구현 범위
   - PVE data와 scenario에 typed `FieldWorkObjectDef`/ID/effect를 추가하고 positive work, unique ID/cell,
     valid non-obstacle tile, player deployment zone 및 첫 content target을 cross-validation한다. Tactical point,
     static object나 BattlefieldTileKind를 재사용하지 않는다.
   - BattleCore에 object별 `accumulated_work_ms/completed`, work-cell별 worker identity와 마지막 정산 battle
     time을 둔다. Object definition과 runtime state는 stable object/work-cell 순서로 투영한다. Presentation
     object는 collider나 unit occupancy를 만들지 않는 overlay다.
   - 모든 event-time 경계 전에 직전 구간의 eligible worker 수로 progress를 정산한다. 현재 기여율에서
     예상 완료 시간을 전용 internal completion event로 예약하고 assignment/Hard CC/death 변화 때 generation을
     갱신해 stale event를 no-op 처리한다. 같은 시각의 combat event와 completion priority도 명시적으로
     고정한다.
   - Deploy·withdraw·death 및 4·14번 위치 mutation 뒤 최종 occupancy에서 assignment를 reconcile한다.
     `FieldWorking`을 attack/skill/block/stance와 DefenseFocus/Rest gate에 연결하되 withdraw와 relocation/swap은
     허용한다. Enemy overlap과 단순 damage에는 reconcile을 호출하지 않는다.
   - 첫 effect는 좁은 `RepairProtectedUnit { flat_hp }` variant만 추가해 기존 protected-unit ref와 HP clamp/
     HpChanged 경로를 재사용한다. 일반 `RepairUnit`, 임의 skill effect, scripting engine과 win-condition
     mutation은 첫 slice에서 만들지 않는 것을 권고한다.
   - Setup snapshot에 authored object definition을, checkpoint/resync에 current progress/completion/worker
     mapping을, event delta에 assignment start/stop/completion을 추가하고 setup version을 올린다. Server는
     새 명령 없이 Core DTO를 기존 battle side-message 순서로 전달한다.
   - Data validation, exact completion time, multi-worker 합산, Preserve, Hard CC/Pause no-catch-up,
     death/withdraw/relocation/swap handoff, completion exactly-once, full-HP clamp와 reconnect/resync를 focused
     test로 고정한다.
6. 현재 tree 기준 난이도와 이유
   - 확정 난이도는 `4/5 — 높음`이다. Field work 전용 구현은 전혀 없어 PVE/scenario schema, BattleCore
     event-time state, activity 상호작용, event log, setup/checkpoint와 server transport 및 첫 live RON을
     가로지르는 신규 end-to-end vertical slice가 필요하다.
   - 다만 stable scenario identity, deterministic event queue, Pause battle clock, protected unit lookup, HP
     mutation과 generic server forwarding을 재사용할 수 있다. Navigation이나 simulation architecture,
     범용 effect/data model을 새로 설계할 필요는 없으므로 `5/5 — 매우 높음`은 아니다.
   - Runtime 변경 가치는 첫 content를 보호 대상 긴급 수리로 제한할 때 충분하다. 한 직원을 공격에서 빼
     objective 생존력으로 전환하는 선택은 기존 자동 전투와 구별되고 4·14번 위치 기능에도 목적을 준다.
     반대로 첫 effect가 전술 포인트 지급뿐이면 기존 자연 회복과 비용 경제를 순환시키는 데 그쳐 이
     `4/5` 변경을 정당화하기 어렵기 때문에 권고하지 않는다.
7. 선행 정책 구현 후 난이도 차이
   - 4·5·6·8·9·11·12·13·14번이 먼저 구현되어 공통 player occupancy/placement, stance/activity gate,
     Ground/Platform selector, armed eligibility, 중앙 Hard CC capability와 relocation/swap의 원자적 위치
     mutation이 존재하면 17번 증분은 `3/5 — 중간`이다.
   - 이 경우에도 field object schema, event-time progress/completion, setup/checkpoint/event DTO와 첫 live
     content는 새로 필요하므로 `2/5`까지 낮아지지는 않는다. 반대로 17번이 먼저 assignment reconcile
     seam을 만들면 14번 swap의 worker handoff 구현 비용이 줄어든다.
8. 난이도 책정 reference 파일과 책임
   - Core command/wire
     - `core_enhanced/src/game/behavior.rs`: setup version/object definition, assignment semantic presentation
       event와 checkpoint의 progress/worker projection이 추가될 wire owner다.
   - Core world transaction/state
     - `core_enhanced/src/game/world/state.rs`: ActiveBattleSession, Pause clock, setup snapshot 및
       checkpoint/resync를 소유하지만 현재 field work state는 없다.
     - `core_enhanced/src/game/world/combat.rs`: battle-time advance와 deploy/withdraw/향후 relocation mutation,
       revision commit 및 server-facing battle update를 한 transaction으로 묶는 경계다.
   - Battle runtime
     - `core_enhanced/src/game/battle/scenario.rs`: stable scenario ID, battlefield position과 cross-reference
       validation에 field object definition이 추가될 곳이다.
     - `core_enhanced/src/game/battle/core/mod.rs`: scenario, runtime maps, protected-unit ref와 중앙 Hard CC
       조회를 소유해 object/assignment runtime의 중심이 된다.
     - `core_enhanced/src/game/battle/core/sim.rs`: event bucket별 authoritative time advance와 live command를
       처리하므로 progress 정산 및 exact completion scheduling이 들어갈 핵심 경계다.
     - `core_enhanced/src/game/battle/enums.rs`: same-time event priority와 stable tie-break를 정의하므로 내부
       field completion event의 결정론적 순서를 고정해야 한다.
     - `core_enhanced/src/game/battle/event_log.rs`: assignment start/stop과 completion의 authoritative semantic
       event 및 event-log version을 소유한다.
     - `core_enhanced/src/game/battle/core/damage_runtime.rs`: 첫 completion effect가 재사용할 active-unit HP
       clamp와 `HpChanged` 기록 경로다.
     - `core_enhanced/src/game/battle/core/movement/blocking.rs`: protect win condition의 scenario ref에서 현재
       defense-object runtime ID를 찾는 기존 seam이다.
     - `core_enhanced/src/game/combat_setup/defense_object.rs`: 첫 수리 대상인 route-end 보호 object의 생성,
       HP 350과 profile을 정의한다.
   - Policy/live data
     - `core_enhanced/src/game/data/pve_data.rs`: encounter RON schema, tactical plan 적용과 data validation에
       field object authoring을 추가할 owner다.
     - `core_enhanced/src/game/events/combat.rs`: PVE encounter와 preview를 runtime BattleScenario로 조립한다.
     - `core_enhanced/src/game/combat_preview/template.rs`: tile marker에서 valid terrain과 Ground/Platform
       deployment zone을 파생해 work-cell eligibility의 source가 된다.
     - `core_enhanced/src/game/combat_preview/mod.rs`: archetype/size/seed로 실제 template을 선택하므로 authored
       work cell이 모든 가능한 live template에서 유효한지 판단하는 근거다.
     - `game_resources/data/pve/encounters.ron`: `suppress_freischutz`의 ProtectUnit 계약, Ambush/Medium 선택과
       5초 wave 구성을 소유한다.
     - `game_resources/data/map/battlefield_templates.ron`: 세 Ambush/Medium variant의 실제 terrain,
       deployment cell과 route endpoint를 소유한다.
     - `game_resources/data/abnormalities/base.ron`: Freischutz의 HP 650, 공격 70, 공격 간격 1,900ms를 소유해
       첫 수리량과 작업 시간의 live pressure 근거가 된다.
     - `game_resources/data/enemies/corroded_employees.ron`: 같은 encounter의 marksman HP/공격/공격 간격을
       소유해 boss 외 동반 wave 압력을 확인하는 근거다.
   - Game server transport
     - `game_server/src/game/player_game_actor/messages.rs`: Core setup/checkpoint/event DTO를 top-level
       `battle_setup_snapshot`과 battle update/resync로 그대로 serialize한다.
     - `game_server/src/game/player_game_actor/state.rs`: Core battle result를 command response 뒤 ordered side
       message로 변환하는 generic transport 경계다.
     - `game_server/docs/unity_bridge_protocol.md`: battle setup/update/resync projection과 revision/event seq의
       authority 분리를 설명하는 현재 server contract다.
   - 외부 Unity runtime/contract/tests
     - 사용자 결정에 따라 7번부터 Unity 조사를 제외했으며 17번 난이도에도 Unity 구현 비용을 포함하지
       않았다.
9. 실행한 focused test와 결과
   - `cargo test -p game_core authored_protect_unit_tactical_plan_overrides_default_defense_contract` — 통과
     (1개). Authored protect ref가 runtime scenario와 defense object를 연결하는 기반을 확인했다.
   - `cargo test -p game_core fixed_defense_route_enemy_targets_defense_object_at_route_end` — 통과(1개).
     Enemy가 route endpoint의 보호 object를 실제 target으로 삼는 기존 runtime을 확인했다.
   - `cargo test -p game_core live_defense_playback_pause_freezes_server_tick_and_resume_advances` — 통과(1개).
     Pause가 battle time을 멈추고 Resume 뒤에만 진행하는 기반을 확인했다.
   - `cargo test -p game_core battle_setup_snapshot_live_defense_state_request_returns_battle_update_without_advancing_cursor`
     — 통과(1개). Setup 분리와 state request/checkpoint cursor 보존 기반을 확인했다.
   - `cargo test -p game_core hard_cc_clears_locked_target_and_reacquires_after_release` — 통과(1개). 중앙화할
     Hard CC active/release-time seam의 현재 동작을 확인했다.
   - `cargo test -p game_core battlefield_template_database_selects_seeded_variants` — 통과(1개). 같은
     archetype/size에서 seed별 template 선택이 실제 계약임을 확인했다.
   - `cargo test -p game_core live_pve_scenario_authoring_contracts_drive_preview_data` — 통과(1개). Live PVE RON
     authoring과 preview/scenario reference가 유효함을 확인했다.
   - `cargo test -p game_server serializes_battle_setup_snapshot_as_top_level_transport_message` — 통과(1개).
     Server의 generic setup serialization seam을 확인했다.
   - 최초 `--exact` 필터 실행은 module-qualified test name과 맞지 않아 0개가 실행됐고 검증 근거에서
     제외했다. 위 결과는 기존 기반의 안정성만 증명하며 field work 기능이 구현됐다는 증거가 아니다.
10. 아직 사용자가 확정해야 할 tuning 또는 선택
    - 없음. 첫 live content는 `suppress_freischutz`의 1회성 `black_box_emergency_repair`, presentation kind
      `EmergencyRepairTerminal`, object position/단일 Ground work cell `(2, 6)`으로 확정했다.
    - `required_work_ms: 10,000`, completion은 좁은
      `RepairProtectedUnit { flat_hp: 100 }`로 확정했다. Full HP에서는 기존 clamp에 따라 실제 회복 0으로
      완료·소모하며 반복 사용하지 않는다. Objective 최대 HP 350의 약 28.6%이고 Freischutz의 raw attack
      70은 defense 8 대상에게 기본 physical damage 64가 되므로 약 1.5회 공격분이다. 한 worker를 10초 동안
      전투에서 빼는 대가와 비교한 첫 live tuning이다.
    - `suppress_nameless_fetus`는 OpenHall/Small을 요청하지만 exact template가 없어 현재 OpenHall 기본
      Medium으로 fallback한다. 첫 field work를 이 암묵적 fallback에 결합하지 않는 편이 안전하다.
    - 현재 tree 난이도 `4/5`, 선행 정책 후 `3/5`, 위 첫 object/좌표, `10,000ms / 100 HP`와 full-HP
      no-op consumption까지 모두 확정했다.

## 전체 구현 종합 — 안전한 vertical slice와 권장 순서

종합일: 2026-08-06  
상태: 구현 순서 권고안, gameplay 구현 미착수

### 적용 범위와 원칙

- 이 순서는 정책 번호순이 아니라 현재 tree의 실제 dependency와 불완전한 중간 gameplay 노출 위험을
  기준으로 한다. 한 정책을 여러 slice에 흩어 임시 의미로 공개하지 않고, 사용자가 관찰할 수 있는 기능은
  해당 의미·state·wire·server·focused test가 모두 닫힌 뒤에만 capability로 노출한다.
- 각 slice는 Core/runtime, 필요한 live data, setup/checkpoint/event DTO, game server serialization과
  canonical 문서를 함께 끝내는 별도 implementation goal/PR 단위다. 단순히 type이나 seam만 추가한 상태를
  vertical slice 완료로 세지 않는다.
- 사용자 결정에 따라 Unity runtime/contract 구현은 이 순서와 난이도에서 제외한다. Core/server slice가
  완료돼도 외부 Unity consumer가 별도 구현되기 전에는 실제 client gameplay가 완성됐다고 표현하지 않는다.
- 아직 쓰지 않는 `FieldWorking`, Reset policy, generic status/effect/script, movement reservation 또는
  compatibility alias를 앞 slice에서 미리 만들지 않는다. 실제 첫 consumer가 들어오는 slice에서 가장 작은
  typed contract만 추가한다.
- 1번 전투 목표와 15번 Pause 의미는 현재 runtime 기반과 거의 일치하므로 별도 gameplay feature slice가
  아니라 사전 계약 게이트와 이후 모든 command의 공통 acceptance test로 처리한다.

### Dependency graph

```text
G0 canonical 계약 고정
├─ S1 기존 철수/재배치 lifecycle 정리 (#3) ─→ S2 중앙 control capability (#13)
├─ S3 전술 자원 분리 (#2)
└─ S4 무기 전술 계약 + 무기 없는 직원 (#10, #11)

S2 + S4 ─→ S5 완성된 Attack/Defense + Ground/Platform 효과 (#6, #7, #8, #9)
S2 + S3 + S5 ─→ S6 전술 순간이동 + 점유 + tile 전환 (#4, #5, #12)
S6 ─→ S7 자리 교대 (#14)
S2 + S6 ─→ S8 수동 지정 스킬 (#16)
S1 + S2 + S5 + S6 + S7 + S8 ─→ S9 현장 작업 (#17)
```

S3과 S4는 논리적으로 병렬 가능하지만 active-run/data/wire version 변경과 공통 fixture 충돌을 피하기 위해
권장 선형 순서는 `G0 → S1 → S2 → S3 → S4 → S5 → S6 → S7 → S8 → S9`다. S7과 S8도 S6 뒤에는
서로 독립적이지만 둘 다 tactical focus와 behavior/server contract를 수정하므로 같은 작업 tree에서는
S7을 먼저 닫고 S8을 적용하는 편이 안전하다.

### 종합 난이도 reference

각 slice의 증분 난이도는 새 정책 난이도를 다시 책정한 값이 아니라 1~17번 각 절의 8번 reference와
선행 구현 후 난이도를 실제 순서에 맞게 합성한 것이다. 종합에 사용한 주요 cross-slice 파일은 다음과
같다.

- Core command/wire
  - `core_enhanced/src/game/behavior.rs`: player battle command, typed reason, setup/checkpoint/event DTO와
    신규 capability가 최종적으로 만나는 공통 wire owner다.
- Core world transaction/state
  - `core_enhanced/src/game/world.rs`: behavior dispatch와 battle authority 검증의 최상위 경계다.
  - `core_enhanced/src/game/world/combat.rs`: deploy/withdraw/skill/playback mutation, preflight, Core command,
    resource/state 변경과 revision commit을 묶는 transaction owner다.
  - `core_enhanced/src/game/world/state.rs`: ActiveBattleSession, deployment/resource, playback, setup/checkpoint와
    resync projection을 소유한다.
  - `core_enhanced/src/game/world/active_run.rs`: strict cutover 대상인 Core active-run payload version과
    continue rejection을 소유한다.
- Battle runtime
  - `core_enhanced/src/game/battle/core/sim.rs`: BattleLiveCommand, authoritative event-time advance와 exact
    effect/completion 처리의 중심이다.
  - `core_enhanced/src/game/battle/core/types.rs`: RuntimeUnit의 action/cast/cooldown/위치 state와 향후 stance/
    activity carrier가 들어갈 곳이다.
  - `core_enhanced/src/game/battle/core/build.rs`: deploy/withdraw lifecycle과 placement validation을 소유한다.
  - `core_enhanced/src/game/battle/core/basic_attack.rs`: release 전후 공격 lifecycle과 Hard CC/stance gate가
    결합되는 경계다.
  - `core_enhanced/src/game/battle/core/movement/blocking.rs`: effective block, engagement release와 보호 대상
    runtime ref를 소유한다.
  - `core_enhanced/src/game/battle/scenario.rs`: field object를 포함한 stable scenario definition과 위치/
    cross-reference validation owner다.
  - `core_enhanced/src/game/battle/event_log.rs`: semantic event shape, cause/source command와 event-log version을
    소유한다.
- Policy/live data
  - `core_enhanced/src/game/data/run_policy_data.rs`와 `game_resources/data/run/policy.ron`: tactical resource,
    deployment, focus speed와 cadence 수치의 policy owner다.
  - `core_enhanced/src/game/data/equipment_data.rs`와 `game_resources/data/equipments/base.ron`: 30개 weapon의
    affinity/block/Defense profile strict schema와 live authoring source다.
  - `core_enhanced/src/game/data/skill_data.rs`와 `game_resources/data/skills/base.ron`: cast selection,
    target/effect와 첫 PlayerUnit live content의 owner다.
  - `core_enhanced/src/game/data/pve_data.rs`와 `game_resources/data/pve/encounters.ron`: field work object와
    Freischutz 첫 live authoring의 owner다.
- Game server transport
  - `game_server/src/game/player_game_actor/messages.rs`: Core setup/update/resync DTO의 top-level serialization
    owner다.
  - `game_server/src/game/player_game_actor/state.rs`: command result와 ordered battle side-message mapping을
    소유한다.
  - `game_server/src/game/player_game_actor/handlers.rs`: battle authority error/resync와 active-run continue의
    request 경계다.
  - `game_server/src/persistence/active_run_store.rs`: host envelope version과 strict local active-run 저장/
    load lifecycle을 소유한다.
  - `game_server/docs/unity_bridge_protocol.md`: revisioned setup/update/resync authority와 event/checkpoint 책임을
    설명하는 현재 server canonical이다.
- 외부 Unity runtime/contract/tests
  - 사용자 결정에 따라 조사와 난이도에서 제외했다. Core/server contract 고정 뒤 별도 consumer goal로
    다룬다.

### G0. Canonical 계약과 baseline 고정

포함 정책: `1`, `15`, 2~17번의 확정 의미 문서 승격  
성격: 사전 구현 게이트, runtime 변경 없음

작업 범위:

- 이 임시 문서의 확정 정책만 `archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md`,
  `skill_target_contract.md`, `data_loading_contract.md`와 server canonical에 책임별로 승격한다.
- 전투 목표/승패, Pause battle-clock freeze, revision/event/checkpoint 책임과 strict cutover 원칙을 먼저
  canonical로 고정한다. 임시 권고나 제외한 범용 기능은 canonical에 섞지 않는다.
- 현재 DefenseRoute smoke, protect/survive, Pause/resume, setup/update/resync와 active-run version baseline을
  focused test로 다시 고정한다.

완료 게이트:

- canonical끼리 용어와 ownership이 충돌하지 않고, 임시 문서만 읽어야 알 수 있는 필수 정책이 남지 않는다.
- runtime과 live data는 바꾸지 않으며 기존 focused test가 실제 1개 이상 실행돼 통과한다.

### S1. 기존 철수/재배치 action lifecycle 정리

포함 정책: `3`  
예상 증분 난이도: `2/5 — 낮음`

작업 범위:

- 재배치 비용을 공통 `ceil(base * pct / 100)` helper로 통일한다.
- 철수 시 아직 release되지 않은 instant 기본 공격과 pending cast는 취소하고, 이미 발사된 기본 공격/skill
  projectile 및 생성된 `GroundFixed` area는 독립 lifecycle을 유지한다.
- `FollowCaster`, `FollowTarget`, aura/channel과 caster-owned buff는 기존 source-exit 의미대로 종료한다.
- 자발적 철수 HP snapshot, 30초/90초 lock, 최신 condition 기반 전투불능 재배치를 보존한다.

완료 게이트:

- withdraw 전후 release boundary, projectile/area 생존, buff cleanup, 올림 비용, repeated incapacitation을
  focused test로 고정한다.
- 기존 deploy/withdraw wire를 유지하며 새 전술 상태나 미래 `FieldWorking` 분기를 추가하지 않는다.

이유:

- 다음 S2의 Hard CC action cancellation이 같은 attack/cast identity와 자율 delivery 경계를 재사용한다.
  이 경계를 먼저 정리하면 두 정책이 서로 다른 취소 규칙을 만들 위험이 줄어든다.

### S2. 중앙 Hard CC/Silence capability

포함 정책: `13`  
예상 증분 난이도: `3/5 — 중간`

작업 범위:

- typed Stun/Freeze/Silence에서 중앙 `UnitControlCapabilities`를 파생하고 기존 attack, skill, movement,
  resonance와 신규 command가 같은 query를 사용하게 한다.
- Hard CC instance를 source별 expiry까지 보존하고 최대 release time을 파생한다. 더 짧은 새 CC가 긴 기존
  CC를 대체해 단축하는 현재 exclusive replacement를 제거한다.
- Hard CC 적용 시 unreleased basic attack generation과 pending skill cast를 취소하되 이미 release된
  delivery/effect는 유지한다. Silence는 새 skill 시작만 막는다.
- checkpoint/resync에 필요한 derived control state와 release time, typed cancellation/rejection reason을
  투영한다.

완료 게이트:

- overlapping Hard CC, 개별 expiry, attack/cast cancellation, Silence 보존, withdraw 예외와 server typed
  reason을 focused test로 고정한다.
- 이후 slice가 `BuffKind`를 직접 검사하지 않고 capability 하나만 소비할 수 있어야 한다.

### S3. 전술 자원 authority 분리

포함 정책: `2`  
예상 증분 난이도: `3/5 — 중간`

작업 범위:

- `LiveBattleDeploymentState`에서 current/max/recovery를 `TacticalResourceState`로 분리하고 deployment
  mapping, redeploy lock과 instance salt는 별도 state로 유지한다.
- `stabilization` 이름과 effect/signal/DTO를 `tactical_resource` domain으로 strict 교체한다.
- `run/policy.ron`에서 전술 자원과 deployment policy ownership을 분리하되 현재 live 기준
  `initial 20 / maximum 99 / 초당 1 / 최초 배치 10`은 바꾸지 않는다.
- checkpoint/root projection과 server serialization을 새 단일 shape로 교체하고 구/new field 병행이나
  compatibility alias를 두지 않는다.

완료 게이트:

- 자연 회복의 Pause 정지, signed delta clamp, 비용 부족 무변경, reconnect 보존, battle 종료/Retreat 폐기,
  deploy/redeploy 공유 pool을 focused test로 고정한다.
- 아직 존재하지 않는 relocation/swap/field-work producer를 가짜 command나 빈 DTO로 만들지 않는다.

### S4. 무기 전술 계약과 weapon-required gate

포함 정책: `10`, `11`  
예상 결합 증분 난이도: `3/5 — 중간`

작업 범위:

- starter 훈련검을 먼저 실제 catalog에 포함해 첫 strict catalog를 30개 weapon 기준으로 만든다.
- 모든 weapon RON이 deployment affinity, block capacity와 Ground/Platform Defense profile 소유권을
  명시하게 하고 archetype/직원 기초 스탯 fallback을 제거한다.
- 무기 없는 직원은 roster/safe-zone 관리는 유지하되 battle roster의 deploy/preview/stance/movement/
  skill/work source를 `weapon_required`로 원자적으로 거부한다.
- active-run Core payload version을 한 번 올리고 기존 V2를 암묵 보완하거나 migration하지 않는 strict
  cutover를 적용한다. Equipment snapshot, preview와 server fixture를 같은 변경에서 갱신한다.

완료 게이트:

- 30개 live weapon의 필수 field와 상호 validation, starter 훈련검 보장, unarmed 모든 battle mutation
  거부, old active-run version reject를 focused test로 고정한다.
- S5가 소비할 profile은 완성하되 stance command나 효과 runtime을 이 slice에서 미리 노출하지 않는다.

### S5. 완성된 Attack/Defense와 weapon-driven Ground/Platform 효과

포함 정책: `6`, `7`, `8`, `9`  
예상 결합 증분 난이도: `4/5 — 높음`

이 네 정책은 내부 sub-commit으로 나눌 수 있지만 하나의 release slice로 닫는다. `Defense` state만 먼저
노출하면 Ground/Platform 효과가 없는 임시 의미가 되고, 효과만 먼저 넣으면 stance authority가 분산되기
때문이다.

작업 범위:

- `Attack/Defense` stance, 직원별 대칭 `1,500ms` cooldown, command/wire/event/checkpoint와 eligibility를
  구현한다. 배치·재배치는 Attack과 같은 cooldown으로 시작한다.
- 전환 전에 commit된 basic attack/skill과 파생 delivery는 끝까지 유지하고 전환 뒤 새 정규 action만
  stance로 막는다. Hard CC는 전환을 거부하고 Silence는 막지 않는다.
- current projected tile과 S4 weapon profile을 읽는 단일 effective defense selector를 만든다.
- Ground DefenseFocus는 authored block 증가/incoming damage 감소, Platform Rest는 1,000ms cadence의
  HP 3/공명 5 회복을 적용한다. 두 효과는 전술 자원을 회복하지 않는다.
- Hard CC 동안 Ground damage reduction은 유지하되 effective block은 0, Rest는 정지하고 해제 뒤
  no-catch-up으로 재평가한다.

완료 게이트:

- stance 전환 중 committed action 완료, 후속 action 차단, cooldown/Pause, block engagement release,
  Rest cadence/clamp/Hard CC suspension, initial Ground/Platform deploy를 focused test로 고정한다.
- Attack/Defense capability는 Ground와 Platform 양쪽 profile 및 checkpoint가 모두 완성된 뒤에만 노출한다.
- 미래 `FieldWorking` enum이나 work suspension state는 아직 추가하지 않는다.

### S6. 전술 순간이동, player occupancy와 tile effect 원자 전환

포함 정책: `4`, `5`, `12`  
예상 결합 증분 난이도: `3/5 — 중간`

세 정책을 같은 slice로 묶는다. 4번만 먼저 공개하면 같은 타일에 아군이 겹칠 수 있고, 12번을 늦추면
Platform 이동 직후 Rest/DefenseFocus state가 틀리는 중간 runtime이 생긴다.

작업 범위:

- 공통 destination validator가 valid tile, static obstacle, player deployment zone, S4 weapon affinity,
  active Player-side tactical slot occupancy를 단일 authority로 검사하게 한다. 기존 deploy도 같은 occupancy
  query로 `position_occupied`를 반환한다.
- `Defense` 직원의 원자적 relocation command, 전술 포인트 `5`, 직원별 battle-time cooldown `10,000ms`,
  facing, semantic event, checkpoint readiness와 server result를 구현한다.
- runtime identity/HP/buff/resonance/cooldown을 보존하고 기존 engagement를 해제한 뒤 target/range/block,
  aura/area 포함과 S5 Ground/Platform effect를 최종 위치에서 한 번 재계산한다.
- server-authoritative tactical focus state와 `0.5x`/Pause 보존, cancel/success/failure의 playback 복원,
  source invalidation과 modal command gate를 구현한다. 실제 ghost는 client-local presentation이므로 Core
  state에 넣지 않는다.

완료 게이트:

- invalid destination/affinity/occupancy/resource/cooldown/stance/action/control 요청이 state, event,
  resource와 revision을 바꾸지 않고, 성공은 revision을 정확히 한 번 증가시킨다.
- Ground↔Platform 이동의 효과 제거/적용, Pause 중 commit, reconnect focus recovery와 이전 delivery 유지가
  focused test로 고정돼야 capability를 노출한다.
- 보행/pathfinding, reservation, relocation pending state와 swap fallback을 추가하지 않는다.

### S7. 자리 교대

포함 정책: `14`  
예상 증분 난이도: `3/5 — 중간`

작업 범위:

- 전술 이동 UX의 점유 아군 분기를 명시적 `SwapUnitPositions` transaction으로 구현한다. Server가
  relocation과 swap 사이를 자동 fallback하지 않게 한다.
- 두 직원의 identity, Defense/action/control, 양방향 zone/affinity/occupancy와 cooldown을 모두 먼저
  검증한 뒤 위치·facing을 한 revision에서 원자적으로 교환한다.
- 전술 포인트는 command당 `5`를 한 번 지불하고 양쪽 직원 cooldown `10,000ms`를 시작한다.
- 양쪽 engagement와 S5 tile effect를 final position 기준으로 한 batch에서 재계산한다. S9가 같은
  post-position reconciliation 경계에 work assignment를 연결할 수 있게 하되 가짜 FieldWorking state나
  빈 assignment collection은 만들지 않는다.

완료 게이트:

- 양쪽 validation 중 하나라도 실패할 때 완전 무변경, 성공 시 unit-owned state 보존, 양방향 affinity,
  단일 비용/양쪽 cooldown, focus 실패 유지와 Pause commit을 focused test로 고정한다.
- 이 slice 시점에는 field work live content가 없으므로 S9에서 FieldWorking source/target과 atomic handoff
  integration test를 추가해야 14↔17 교차 계약까지 최종 폐쇄된다.

### S8. 수동 지정 스킬과 공통 tactical focus 재사용

포함 정책: `16`  
예상 증분 난이도: `3/5 — 중간`

작업 범위:

- explicit `SkillCastSelection::Self | AutomaticUnit | PlayerUnit | PlayerTile`을 data/runtime/wire에 추가하고
  모든 live skill이 selection을 명시하도록 strict validation한다.
- read-only target preview와 acceptance-time target/range/hostility/air/usefulness validation을 S6 tactical
  focus에 연결한다. PlayerUnit/PlayerTile target 누락, stale target과 invalidation에 fallback/retarget/refund를
  만들지 않는다.
- 첫 live PlayerUnit으로 `fragment_freischutz_black_round`와 awakened
  `freischutz_magic_bullet`을 함께 전환하고 기존 damage/focus/delivery 수치는 유지한다.
- PlayerTile은 engine/fixture까지만 완성하고 첫 live content나 별도 active-skill cooldown을 만들지 않는다.

완료 게이트:

- Self/Automatic 기존 동작, PlayerUnit 필수 target, PlayerTile 필수 tile, acceptance 후 target 소멸의
  direct no-op/captured tile, Pause/focus/resync와 server serialization을 focused test로 고정한다.
- Black Round 두 progression 외 live target ownership을 임의로 바꾸지 않는다.

### S9. 현장 작업과 최종 교차 통합

포함 정책: `17` 및 `3·6·8·9·13·14·16`의 FieldWorking 교차 계약  
예상 증분 난이도: `3/5 — 중간`

작업 범위:

- PVE/scenario/setup에 별도 `FieldWorkObjectDef`, stable object/work-cell ID, positive work와 좁은
  `RepairProtectedUnit { flat_hp }` effect를 추가한다. Tactical point/static object/BattlefieldTileKind를
  재사용하지 않는다.
- 이 slice에서 처음 실제 consumer와 함께 `Ready/FieldWorking` exclusive activity를 추가한다. Assignment는
  command가 아니라 final work-cell occupancy에서 자동 reconcile한다.
- BattleCore event-time 경계에서 eligible worker 기여를 정산하고 exact completion internal event를
  generation으로 예약해 stale completion을 no-op 처리한다. Hard CC는 별도 suspension state 없이 S2
  capability에서 기여만 정지하고 Pause/CC 시간을 소급하지 않는다.
- deploy/redeploy/withdraw/death/S6 relocation/S7 swap에 assignment start/stop과 atomic handoff를 연결한다.
  FieldWorking은 attack/automatic·manual skill/block/stance/DefenseFocus/Rest를 막지만 withdraw와
  relocation/swap source/target은 허용한다.
- setup/checkpoint/resync/event와 server contract를 완성하고 `suppress_freischutz`에
  `black_box_emergency_repair`, work cell `(2, 6)`, `10,000ms`, 보호 대상 HP `100` 수리를 authoring한다.

완료 게이트:

- multi-worker 합산, Preserve-only 이탈/재개, enemy overlap/damage 지속, Hard CC/Pause no-catch-up,
  death/withdraw, relocation/swap handoff, full-HP no-op consumption과 completion exactly-once를 BattleCore
  event time에서 focused test로 고정한다.
- setup loss recovery와 mid-work checkpoint/resync가 object progress/completion/worker mapping을 stable
  ordering으로 복원하고 event seq/HashMap order에 의존하지 않아야 한다.
- 이 slice가 끝나야 3·13·14·16번의 FieldWorking 관련 교차 조건까지 구현 완료로 간주한다.

### Contract/version 전략

- Schema나 wire field를 미래 slice 때문에 미리 추가하지 않는다. 실제 shape가 바뀌는 slice가 해당
  setup/checkpoint/event-log/data version을 올리고 구/new shape를 병행하지 않는다.
- S3과 S4를 같은 production release에 싣는다면 active-run/data cutover를 S4의 새 Core payload version
  하나로 합친다. S3 audit에서 active-run serialized shape 변경이 발견되더라도 별도 임시 migration을
  만들지 않고 같은 strict cutover에 포함한다.
- S6과 S9를 별도 release하면 각 release의 setup shape 변경에 맞춰 `setup_version`을 각각 올린다. 하나의
  미공개 feature branch에서 연속 구현한 뒤 함께 배포한다면 최종 wire shape 기준으로 한 번만 올린다.
- Event-log variant/validator 의미가 바뀌는 release는 `BATTLE_EVENT_LOG_VERSION`과 fixture를 같이 갱신한다.
  내부 commit마다 무의미하게 version을 올리거나 과거 version reader를 추가하지 않는다.
- 외부 Unity는 별도 consumer gate다. Core/server contract version과 fixture가 고정된 뒤 Unity 전용 goal에서
  구현하며, 이 문서의 Core 난이도에 소급 합산하지 않는다.

### Merge/release 검증 게이트

각 slice PR:

- 변경 계약과 직접 관련된 focused Core/data/server test를 실행하고 실제 실행 수가 1개 이상인지 확인한다.
- invalid command가 state/event/resource/cooldown/revision을 바꾸지 않고 success가 revision을 한 번만
  증가시키는 transaction test를 신규 command마다 둔다.
- Setup은 immutable definition, event delta는 timing, checkpoint/resync는 current state라는 책임 분리를
  serialization test로 유지한다.
- 동일 seed/input/command sequence의 deterministic 결과와 stable ordering을 확인한다.
- 해당 slice의 canonical 및 goal `PLAN/EXPERIMENTS/EXPERIMENT_NOTES`를 구현과 함께 갱신한다.

Production release candidate:

- slice별 focused test 외에 `game_core`, `game_server` package test와 live embedded RON strict-load/audit를
  실행한다.
- 새 active-run/setup/event-log version의 success/reject fixture와 battle setup→update→resync server message
  order를 확인한다.
- 기존 DefenseRoute의 protect, survive, deploy/withdraw, Pause/resume와 전투 종료 smoke가 유지되는지
  회귀 검증한다.
- Unity가 아직 없다면 Core/server release 완료와 실제 playable client 완료를 별도 상태로 보고한다.

### 의도적으로 만들지 않는 중간·범용 기능

- player 보행/pathfinding, movement reservation, swap fallback
- generic control capability RON 또는 passive suppression system
- desired stance, stance transition 예약 state, 태세 전환 자원 비용
- 별도 active-skill cooldown과 첫 PlayerTile live content
- field work Assign/Cancel command, Reset policy, stored suspension state와 generic scripting engine
- client gameplay command batch/plan 및 Pause 중 소급 time 진행
- compatibility alias, 구/new DTO 병행과 legacy active-run 자동 migration

이 경계를 벗어나는 요구가 구현 중 발견되면 현재 slice를 조용히 확장하지 않고 정책 재검토 대상으로
되돌린다.

## 문서 지위

이 문서는 `DefenseRoute`를 정적인 배치 후 자동 전투에서 플레이어가 실시간으로 개입하는
전술 지휘 전투로 확장하기 위해 수행한 2026-08-04~06 구조 조사와 설계 논의를 동결 보존한다.

- 이 문서는 gameplay/runtime/data/Unity wire canonical이 아니다.
- 승인된 목표 의미는
  [DefenseRoute 전술 지휘 목표 정책 결정](../decisions/defense_route_tactical_command_policy_2026-08-06.md)이
  소유한다.
- 아래 `부분 승인된 정책`과 번호별 기록은 승인 과정의 historical evidence이며 목표 정책 source가
  아니다.
- 그 밖의 `권고`는 정책 제안이며 승인된 규칙이 아니다.
- 이 문서만을 근거로 runtime, live RON, DTO 또는 Unity 계약 구현을 시작하지 않는다.
- 문서와 구현이 충돌하면 현재 runtime code와 live RON/data를 우선한다.
- 구현된 정책만 각 Phase에서 가장 가까운 canonical 문서로 옮긴다.
- 승인 전에는 레거시 compatibility, dual schema 또는 임의 fallback을 만들지 않는다.

## 상위 방향

현재 합의된 상위 방향은 다음과 같다.

- 기존 배치 코스트를 배치뿐 아니라 이동과 작전에도 사용하는 범용 전술 자원으로 확장한다.
- 직원은 범용 자원과 유닛별 전용 쿨타임을 사용하는 `전술 순간이동`으로 다른 배치 타일에 즉시 재배치할 수 있다.
- 두 직원의 위치와 방향을 원자적으로 다시 정하는 `자리 교대`를 지원한다.
- 공격 중지와 전술 명령 준비 상태는 `공격 태세 / 방어 태세` 형태의 교전 태세 전환으로 통합한다.
- 직원을 전장 오브젝트에 배정하는 `현장 작업`을 도입한다.
- 기본 공격 대상은 플레이어가 지정하지 않는다. 기존 자동 타겟팅과 무기별 targeting profile을 유지한다.
- 일부 수동 스킬은 플레이어가 대상 유닛 또는 시전 타일을 직접 지정한다.
- 모든 자동 공격을 수동화하거나 높은 APM을 요구하는 RTS를 목표로 하지 않는다.
- 결정론, server-authoritative simulation, checkpoint/resync와 Unity transport 경계를 유지한다.

전술 순간이동의 첫 live tuning은 비용 `5`, 직원별 cooldown `10,000ms`로, 태세 전환의 첫 live tuning은
직원별 대칭 cooldown `1,500ms`로 확정했다. 자리 교대는 command당 비용 `5`를 한 번 지불하고 양쪽
직원의 전술 이동 cooldown `10,000ms`를 시작하는 것으로 확정했다. 첫 수동 지정 skill은 Black Round
progression의 `PlayerUnit` 전환으로 확정했다. 첫 field work content는 `suppress_freischutz`의
`black_box_emergency_repair`, 단일 Ground work cell `(2, 6)`, 작업량 `10,000ms`, 보호 대상 HP `100`
수리로 확정했다.

## 부분 승인된 정책

2026-08-04 논의에서 다음을 승인했다. 이 승인 내용은 이 임시 문서 안에서 기존 보행 이동 권고보다
우선하지만, 아직 canonical 문서나 runtime/data/transport 구현 승인은 아니다.

### 범용 전술 자원

- 기존 배치 코스트를 배치뿐 아니라 전술 순간이동, 자리 교대와 작전에도 사용하는 범용 전술 자원으로
  확장한다.
- 플레이어 표시명은 `전술 포인트`로 확정한다. 코드·DTO의 domain 개념명은 `tactical_resource`를
  사용한다.
- 최초 배치, 기존 회수/전투불능 뒤 재배치, 전술 순간이동과 자리 교대는 같은
  current/maximum을 가진 하나의 전술 포인트 pool에서 소비한다. 별도의 배치 포인트와 전술 포인트
  pool로 분리하지 않는다. Field work 자체에는 별도 작업 비용이 없다.
- 전술 자원 회복은 한 가지 전역 규칙에만 한정하지 않는다. 스킬 파편의 자원 생성 효과 등 명시적으로
  승인된 gameplay effect가 전술 자원을 증감할 수 있다. Ground `DefenseFocus`와 Platform `Rest`는
  전술 자원을 회복하지 않는다.
- 각 회복 수단의 발동 조건, 회복량, 주기와 중첩 규칙은 해당 skill/effect 또는 공통 전투 policy가
  명시하며 Core가 authoritative하게 적용한다.
- Pause 중에는 battle time 기반의 지속 회복이 진행되지 않는다. Pause 해제 시 한꺼번에 소급 계산하지
  않는다.
- 현재의 battle-time 기본 자연 회복도 유지한다. 여기에 스킬 파편 등 별도로 승인한 효과가 전술 자원을
  증감할 수 있다.

### 기존 회수/재배치와 전술 순간이동 분리

- 기존 `WithdrawUnit`과 재배치 기능의 HP 회복, 버프 제거, 스킬 취소, 재배치 lock/cost와 새 runtime
  instance 생성 의미는 그대로 유지한다.
- 자발적 철수 뒤 재배치 HP는
  `min(최대 HP, 철수 당시 HP + floor(최대 HP * 0.30))`로 철수 시점에 확정한다. 재배치 대기 시간에
  따른 추가 회복은 없다.
- 전투불능 뒤 재배치는 별도 고정 HP 비율을 사용하지 않고, 전투불능 결과가 반영된 직원의 최신
  condition/trauma에서 새 runtime unit의 시작 HP를 다시 계산한다.
- 두 재배치 모두 새 runtime instance를 만들며 이전 instance의 buff/debuff, cast, target과 combat
  cooldown state를 상속하지 않는다.
- 전술 순간이동은 기존 회수/재배치를 조합하지 않는 별도 명령이다.
- 전술 순간이동은 범용 전술 자원을 소비하고 직원별 전용 쿨타임을 시작한다.
- 같은 runtime unit의 위치와 방향만 원자적으로 변경한다.
- HP, 버프·디버프, 공명, 공격/스킬 쿨타임, 태세, runtime unit identity 등 unit-owned state를
  초기화하거나 변경하지 않는다.
- 새 위치 때문에 달라지는 저지, 자동 target eligibility, range, aura/area 포함 관계 등 spatial derived
  state는 authoritative Core가 다시 계산한다.
- 경로, 보행, 이동 도중 상태, 목적지 예약, 도착 또는 이동 취소 단계는 없다.
- BattleCore의 logical/world position은 side와 무관하게 동타일 다중 점유를 계속 허용한다. 플레이어가
  내리는 deploy·재배치·전술 이동 command만 다른 active Player-side actor가 있는 destination에 새 아군을
  놓지 못한다.
- 기존 deploy와 전술 순간이동은 다른 active allied unit이 점유한 destination을 `position_occupied`로
  거부하고, 아군이 점유한 두 타일의 교환은 자리 교대만 사용한다. 현재 runtime은 동일한 아군 배치
  타일을 허용하므로 구현 단계에서 이 검사를 새로 추가해야 한다.
- 순간이동 destination에는 최초 배치와 같은 deployment zone, affinity, valid tile과 static obstacle
  검사를 수행한다. 보행 traversal은 적용하지 않는다.
- 최초 배치를 제외하면 active allied unit의 위치가 바뀌는 수단은 전술 순간이동, 자리 교대와 기존
  회수 후 재배치뿐이다. 아군은 넉백 또는 강제 이동의 대상이 되지 않는다.
- 아군 보행, 이동 경로, 경로 탐색, 이동 segment, 목적지 예약과 이동 중 상태는 만들지 않는다.
- 적 스킬에는 넉백이 없다. 향후 system-driven 강제 이동이나 spawn이 생길 경우의 동타일 중첩은 해당
  기능에서 별도로 결정하며, 이 player command 목적지 규칙에서 자동으로 추론하지 않는다.
- 첫 버전은 모든 직원에게 같은 fixed cost `5`와 같은 base cooldown `10,000ms`를 사용하되 cooldown
  state는 직원별로 독립한다. 두 값은 live policy에서 authoring하고 이후 balance tuning으로 조정할 수
  있다.

### 방어 태세와 전술 명령 준비

- 전술 순간이동은 `Defense` 태세에서만 사용할 수 있다.
- 자리 교대는 두 직원 모두 `Defense` 태세일 때만 사용할 수 있다.
- `Attack`은 기존 자동 기본 공격을 사용하고 `Defense`는 새 정규 기본 공격을 시작하지 않는다.
- 최초 배치와 기존 withdraw 뒤 재배치는 일반 tile에서 `Attack`으로 시작한다. Work cell에
  직접 배치하는 경우는 예외적으로 `Defense + FieldWorking`으로 시작한다. 전술 순간이동은
  기존 `Defense`를 유지한다. 일반 배치·재배치의 최초 `Attack` 진입도 `1,500ms` 태세 전환 cooldown을
  시작한다.
- 행동 불가 hard CC 중이 아니라면 기본 공격 windup 또는 skill cast/focus 중에도 태세와 새 태세의
  효과를 즉시 전환한다.
- 전환 전에 이미 확정된 기본 공격과 skill action은 delayed step, proc, 연속타와 `ExtraAttack`을 포함해
  끝까지 수행한다. 이미 생성된 projectile/area와 적용된 buff/debuff도 기존 lifecycle과 authored
  tracking을 유지한다.
- `Defense`는 전환 뒤 새로운 정규 기본 공격과 automatic/manual skill action의 시작을 막는다. 장비·상태의
  passive/reactive `Triggered` 효과는 유지한다.
- 아직 방출되지 않은 공격 windup 또는 skill focus가 있으면 전술 순간이동과 자리 교대를 사용할 수 없다.
  attack resolve 또는 CastEnd 뒤 남은 projectile/delayed delivery/area/buff만으로는 전술 명령을 막지
  않는다.
- 태세 전환을 위해 `desired_stance`, 공격 취소 token 또는 별도 예약 상태를 만들지 않는다.
- 행동 불가 hard CC 중에는 태세 전환도 사용할 수 없지만 Silence는 태세 전환을 막지 않는다.
- 태세 전환 자체는 전술 자원을 소비하지 않는다.
- 성공한 태세 전환은 즉시 직원별 대칭 `1,500ms` 태세 전환 cooldown을 시작한다. ready 전에는 다음
  태세 전환을 할 수 없다.
- 태세 전환 및 전술 순간이동 쿨타임은 authoritative battle time을 사용하고 Pause 중 흐르지 않는다.
- 순간이동 뒤 직원은 `Defense` 태세를 유지한다.
- `Defense`에서는 현재 배치 tile kind와 무관하게 정규 기본 공격과 automatic/manual skill을 새로 시작할
  수 없다.
- 현재 Ground tile에 배치된 직원의 `Defense`는 `DefenseFocus` 의미를 가지며 해당 weapon RON이
  authoring한 block capacity 증가와 incoming damage 감소 효과를 받는다. 전술 자원은 회복하지 않는다.
- 현재 Platform tile에 배치된 직원의 `Defense`는 `Rest` 의미를 가지며 battle time 동안 HP,
  공명(마나)을 천천히 회복하고 전술 자원은 회복하지 않는다.
- `Rest` 회복은 Defense가 active이고 hard CC가 아닌 실행 battle time 동안 피격 여부나 active wave
  유무와 관계없이 적용한다. 각 회복 tick은 `min(current + recovery, max)`로 HP는 해당 unit의
  `max_health`, 공명은 해당 unit의 `resonance_max`에 명시적으로 clamp한다. 상한을 넘은 회복량은 버리며
  저장·이월하거나 다른 자원으로 변환하지 않는다. 상한에 도달한 동안의 tick도 비축하지 않고, 이후
  공명을 소비하거나 피해를 받은 뒤 다음 정상 tick부터 다시 회복한다. Pause 중에는 진행하지 않는다.
- 각 개별 weapon RON의 combat profile이 GroundOnly/PlatformOnly/Any 배치 적성과 block_capacity를
  함께 명시한다. Weapon archetype은 이 runtime 전투 방식을 추론하는 authority가 아니다.
- Ground를 허용하는 weapon은 양수 block_capacity를 가져야 하며 PlatformOnly weapon은 block_capacity
  0이어야 한다. 이 교차 규칙을 strict live validation으로 강제한다.
- 실제 Platform tile에서는 `Any` weapon의 authored block_capacity도 effective 0으로 처리한다.
- 현재 Staff weapon은 모두 PlatformOnly로 authoring한다.
- 무기를 장착하지 않은 직원은 전투 배치 로스터에는 계속 표시하되 disabled 상태로 투영한다. Unity는
  해당 항목의 선택, 드래그, 배치 위치·방향 preview와 deploy command 생성을 모두 막는다.
- Core는 해당 직원에게 battle deploy capability를 제공하지 않고, 변조·stale client가 직접 보낸 배치
  요청도 `weapon_required` typed reason으로 거부한다. 기존 unarmed deploy 동작을 compatibility
  fallback으로 보존하지 않는다.
- 이 비활성 규칙은 전투 배치 상호작용에 한정한다. 전투 밖 로스터 열람과 무기 장착·교체 기능은 계속
  사용할 수 있어야 한다.
- 실제 current tile kind가 `DefenseFocus/Rest` 효과를 선택한다.
- `Any` 직원은 순간이동/교대로 지형을 바꾸면 같은 command 안에서 방어 효과를 즉시 전환한다.
- Weapon RON의 `deployment_affinity`는 destination tile kind에 대한 배치 capability를 소유한다.
  `GroundOnly`는 Ground, `PlatformOnly`는 Platform만 허용하며 `Any`만 두 종류 사이를 전술
  순간이동하거나 자리 교대할 수 있다.
- `Any` weapon RON은 Ground `DefenseFocus`와 Platform `Rest` profile을 모두 소유하지만 동시에 둘을
  활성화하지 않는다. 실제 current tile kind가 활성 profile 하나를 선택한다.
- Ground/Platform 사이의 전술 순간이동·교대는 같은 authoritative mutation 안에서 이전 tile 효과를
  제거하고 새 tile 효과를 적용한다. 중간 state, 두 효과의 중첩 또는 효과 없는 별도 transition tick은
  만들지 않는다.
- 태세 전환 전에 이미 생성된 projectile/area와 적용된 buff/debuff는 기존 lifecycle을 유지한다.
- `Defense`에서 공명이 가득 차도 auto skill을 시작하지 않고 현재 공명을 보존한다.
- `DefenseFocus` 해제로 effective block capacity가 감소하면 기존 engagement를 정규 저지 우선순위로
  다시 판정하여 새 capacity만큼 보존하고 나머지를 해제한다. 교전 시작 순서는 별도 state로 저장하지
  않고, 아직 engagement되지 않은 적이 기존 engagement를 밀어내지 않는다.
- 이미 Defense인 직원이 hard CC에 걸려도 stance와 incoming damage 감소는 유지한다. 다만 effective
  block capacity는 0이 되어 현재 engagement를 모두 해제하고, Rest HP/공명 회복과 태세
  전환·순간이동·교대 명령은 CC 동안 중단한다.
- Hard CC가 끝나면 현재 위치와 상태에서 effective block을 복구하고 저지 대상을 새로 판정한다. 이전
  engagement identity를 복원하지 않는다. 각 회복 source는 남은 회복 주기부터 재개하며 CC 시간의
  회복량을 소급 지급하거나 비축하지 않는다.
- Hard CC는 battle time 자체를 멈추지 않으므로 이미 시작된 태세 전환·순간이동 cooldown은 계속 흐른다.
  전체 battle `Pause`만 battle-time cooldown을 정지한다.
- Silence는 automatic/manual을 포함한 새로운 skill 시작만 제한한다. 이미 시작된 skill action은 기존
  lifecycle대로 완료한다.
- Silence는 기본 공격, 저지, 방어 태세 효과, 회복, 태세 전환, 전술 순간이동과 자리 교대를 막지 않는다.
  Silence 중에도 각 명령의 stance/activity/resource/cooldown/destination 조건은 그대로 검증한다.
- 구체적인 per-weapon affinity 값과 회복/방어 효과 수치는 live RON에서 authoring한다.
- 각 weapon RON의 Ground `DefenseFocus` profile이 `block_capacity_bonus`,
  `incoming_damage_reduction_percent`를 직접 소유한다. Runtime은 Ground `Defense` 동안 두 값을 기존
  effective block capacity와 incoming `DamageModifiers`에 합성하고, 태세 또는 tile kind가 바뀌면 두
  효과를 모두 제거한다. 같은 수치를 공통 tactical policy에 중복 authoring하지 않는다.
- Ground `DefenseFocus`에는 `tactical_resource_recovery_per_second`나 동등한 회복 field를 두지 않는다.
- run policy의 공통 `rest_tick_interval_ms: 1,000`이 모든 Platform `Rest` cadence를 소유한다. 각 weapon
  RON의 `Rest` profile은 `hp_recovery_per_tick`과 `resonance_recovery_per_tick`만 직접 소유하고 각 값의
  `0`은 해당 자원을 회복하지 않는다는 명시적 authoring이다. 무기별 cadence나 전술 자원 회복 field는
  두지 않는다.
- `GroundOnly` weapon은 `DefenseFocus` profile을, `PlatformOnly` weapon은 `Rest` profile을 명시한다.
  `Any` weapon은 두 profile을 모두 명시하고 실제 배치 tile kind에 해당하는 하나만 활성화한다.
- weapon archetype, 직원 기초 스탯 또는 legacy default에서 배치 적성·저지력·태세 효과를 추론하는
  fallback은 두지 않는다.

### 자리 교대

- 실제 동시 보행이 아니라 한 command/revision 안에서 두 직원의 위치를 원자적으로 교환한다.
- 두 직원의 새 facing을 command에서 각각 다시 지정한다.
- 양쪽 모두 `Defense`이고 진행 중인 기본 공격·skill 또는 hard CC 상태가 아니어야 한다. `FieldWorking`
  중인 직원도 source 또는 target으로 자리 교대할 수 있으며, 그 밖의 이동을 금지하는 exclusive
  activity는 eligibility를 실패한다.
- 양쪽의 tactical relocation cooldown이 모두 준비되어야 하며, 각 직원이 상대 destination에 대해 최초
  배치와 같은 zone/affinity/valid tile/static obstacle 검사를 통과해야 한다.
- 한쪽이라도 승인된 eligibility를 실패하면 자원, cooldown, state와 revision을 전혀 변경하지 않는다.
- 교대 뒤 양쪽의 spatial derived state를 새 위치와 방향에서 다시 계산한다.
- 교대 비용은 command당 fixed cost 한 번 지불하고 성공 시 양쪽 직원의 tactical relocation cooldown을
  모두 시작한다.
- 한쪽이라도 resource/cooldown/stance/activity/placement validation을 실패하면 전체를 거부하고 아무
  state도 변경하지 않는다.

### Pause 중 즉시 직렬 명령

- `Pause`는 battle simulation clock만 정지한다. 게임과 authoritative command 처리는 멈추지 않는다.
- Unity는 Pause 중 gameplay mutation을 한 번에 하나만 server에 보낸다. 앞 command의 authoritative
  response와 새 revision을 받은 뒤에만 다음 mutation을 보낸다. 여러 gameplay command를 client-local
  plan으로 모으거나 한 batch로 제출하지 않는다.
- Server는 각 mutation을 frozen battle time의 최신 state/revision에서 즉시 검증한다. 성공하면 해당
  command만 원자적으로 commit하고 revision을 정확히 1 증가시키며, 실패하면 즉시 typed error로
  거부하고 state/event/revision을 전혀 바꾸지 않는다. 성공·실패 뒤에도 명시적 `ResumeBattle` 전까지
  Pause를 유지한다.
- 앞 command의 성공은 뒤 command가 실패해도 rollback하지 않는다. 각 command의 atomicity만 보장하며
  여러 command sequence의 all-or-nothing transaction은 제공하지 않는다.
- 성공한 자원 소비, 위치·facing, 태세, 배치·회수, skill 시작, cooldown 시작과 기타 즉시 state 변화는
  바로 authoritative state에 반영한다. 같은 frozen battle time에 여러 command가 성공하면 revision,
  event sequence와 `source_command_id` 순서로 결정론적으로 구분한다.
- Battle time은 정지되어 있으므로 적 이동·공격, cast/work 진행, HP/공명/전술 자원 회복, buff duration과
  cooldown 감소는 진행하지 않는다. 예를 들어 Pause 중 `Attack -> Defense`가 성공해 태세 cooldown을
  시작하면 battle time이 흐르지 않으므로 즉시 `Defense -> Attack` 요청은 cooldown으로 거부한다.
- 유닛 상태, HP/공명, buff/debuff, 태세, cooldown, 장비·skill 정보, range/target/deployment preview와
  battle state/resync 같은 read-only query도 즉시 처리한다. 성공한 query는 최신 authoritative Pause
  state를 읽고 revision을 증가시키지 않으며, 잘못된 query는 state를 바꾸지 않고 즉시 typed error로
  거부한다.
- `ResumeBattle`도 직렬 command 경계를 따른다. Unity는 진행 중인 mutation response를 받은 뒤 현재
  revision으로 재개를 요청하며 성공 뒤에만 battle clock이 다시 흐른다.
- `RetreatBattle`은 같은 직렬 경계에서 즉시 검증·실행하고 성공하면 active battle과 Pause를 종료한다.
- Client-local pending gameplay queue와 `ResumeBattleWithPlan` command는 만들지 않는다. Reconnect/resync는
  이미 commit된 최신 Pause state만 복구하면 된다.

## 확인한 source

### 내부 문서

- [README.md](../README.md)
- [archive/game_rulebook_legacy_2026-10-04.md](../archive/game_rulebook_legacy_2026-10-04.md)
- [skill_target_contract.md](../skill_target_contract.md)
- [core_runtime_contract.md](../core_runtime_contract.md)
- [data_loading_contract.md](../data_loading_contract.md)
- [abnormality_skill_design_notes.ko.md](../skills/abnormality_skill_design_notes.ko.md)
- [engineering_workflow.md](../engineering_workflow.md)

### 외부 Unity canonical

- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`

### 주요 runtime/data seam

- `src/game/behavior.rs`: `PlayerBehavior`, `ActionKind`, battle DTO와 presentation event wire shape
- `src/game/world.rs`: battle authority 검증과 behavior dispatch
- `src/game/world/combat.rs`: deploy/withdraw/skill/playback handler와 projection commit
- `src/game/world/state.rs`: `ActiveBattleSession`, live deployment 상태, checkpoint/setup projection
- `src/game/battle/core/sim.rs`: `BattleLiveCommand`, manual skill validation과 simulation event 처리
- `src/game/battle/core/types.rs`: `RuntimeUnit`, action lock, projectile/area runtime
- `src/game/battle/core/movement/*`: continuous movement planner/backend/blocking/event
- `src/game/battle/core/basic_attack.rs`: 자동 기본 공격 시작/해결과 이동 중단
- `src/game/battle/core/build.rs`: unit spawn/deploy와 battlefield placement validation
- `src/game/range_preview.rs`: 기본 공격/스킬 최종 cell preview와 targeting kind
- `src/game/data/pve_data.rs`: tactical point live authoring schema
- `game_resources/data/run/policy.ron`: live deployment resource와 battle tick 수치
- `game_resources/data/skills/base.ron`: 현재 live skill targeting
- `game_resources/data/pve/encounters.ron`: 현재 tactical point authoring
- `game_resources/data/map/battlefield_templates.ron`: Ground/Platform/Obstacle/route 타일 source

## 현재 runtime 구조

### 공식 전투와 명령 표면

현재 공식 live 전투 모델은 `DefenseRoute` 하나다. 일반 전투와 Boss는 같은 live battle transport,
battlefield, protection target과 wave 모델을 사용한다.

현재 player battle command는 다음뿐이다.

- `RequestBattleState`
- `RecoverBattleSetupLoss`
- `RequestDeploymentRangePreview`
- `DeployUnit`
- `WithdrawUnit`
- `ActivateSkill`
- `RetreatBattle`
- `PauseBattle`
- `ResumeBattle`
- `SetBattleSpeed`

`MoveUnit`, `SwapUnitPositions`, stance, field work command는 없다.

Gameplay mutation command는 top-level `battle_authority { battle_uuid, battle_state_revision }`를
요구한다. Authority 누락/불일치는 mutation 전에 거부되고 server actor가 current battle resync를
뒤이어 보낸다. 성공한 command는 projection revision을 정확히 한 번 증가시키고 command가 만든
event에는 `source_command_id`를 기록할 수 있다. 이 경계는 새 전술 명령에서도 유지할 수 있다.

### 현재 runtime 공간 안정화치 — 전술 포인트 migration source

현재 자원은 `ActiveBattleSession.live_deployment: LiveBattleDeploymentState`가 소유한다.

- `current_cost`, `max_cost`, `cost_per_second`
- `base_deploy_cost`
- 철수/전투불능 재배치 쿨타임과 재배치 비용 배율
- 배치된 직원과 runtime unit mapping
- redeploy lock
- 다음 runtime instance salt

현재 live 수치는 다음과 같다.

| 항목 | 값 |
| --- | ---: |
| initial cost | 20 |
| max cost | 99 |
| recovery | 전투 시간 1초당 1 |
| first deployment | 10 |
| withdraw redeploy cooldown | 30초 |
| defeat redeploy cooldown | 90초 |
| redeploy cost multiplier | 150% |

회복은 wall time이 아니라 authoritative battle time을 따른다. Pause 중에는 battle time이 멈춰
회복도 멈추고, 배속은 wall time 대비 회복 속도에도 반영된다.

`SkillEffectDef::ModifyStabilization`과 `BattleLiveSignal::StabilizationDelta` seam이 이미 존재하지만
현재 live skill RON은 이 효과를 사용하지 않는다. Core가 signal을 배출하고 `ActiveBattleSession`이
이를 deployment cost에 반영한다.

### 배치와 점유

배치 handler는 다음을 검증한다.

- 선택한 타일이 player deployment zone에 속하는가.
- 해당 zone kind가 직원 `DeploymentAffinity`와 호환되는가.
- 직원이 현재 배치 가능하고 비용/재배치 쿨타임을 만족하는가.
- Core placement에서 타일이 valid이며 static obstacle이 아닌가.

그러나 active player unit이 같은 타일을 이미 점유하는지는 검사하지 않는다. `GameError::PositionOccupied`
variant와 server error mapping은 존재하지만 실제 deploy 경로에서 사용되지 않는다.

Continuous movement runtime도 unit collider를 gameplay obstacle로 사용하지 않으므로 unit world position
겹침이 가능하다. 현재는 player tactical slot이라는 별도 권위 상태가 없다.

### 기존 연속 이동 기반

이미 존재하는 기반은 다음과 같다.

- 연속 world 좌표와 tile projection
- `MovementGoal::AttackUnit`과 `MovementGoal::MoveToPoint`
- unit body radius와 move speed
- Rapier 기반 static obstacle/void 충돌
- 50ms movement tick
- `MovementSegmentStarted`와 `MovementStopped`
- checkpoint active unit `position`과 `world_position`
- movement/cast/basic attack action lock

현재 player movement plan은 `PlayerMovementPlan::FixedDefense` 하나다. Planner는 매 tick player가
사거리 안에서 자동 공격할 target만 찾고 접근 이동은 만들지 않는다. 반면 route enemy는 authored
route cell의 다음 waypoint로 이동한다.

현재 `MoveToPoint`는 목적지를 향해 직선으로 진행하며 static obstacle에 막히면 정지한다. 임의의
player destination을 위한 grid/nav pathfinding은 없다. `RuntimeUnit.tactical_anchor`는 spawn 시
설정되지만 현재 player relocation authority로 사용되지 않는다.

`ActionState`도 `Idle | Dead`뿐이어서 기본 공격 windup, 스킬 실행과 전술 명령 가능 여부를 이 enum
하나로 판별할 수 없다. 승인된 전술 순간이동은 연속 이동 기반을 사용하지 않지만 active lifecycle,
action lock, active skill cast, hard CC와 exclusive activity를 함께 보는 별도 tactical command
capability가 필요하다.

전술 이동은 보행이 아닌 원자적 순간이동으로 승인되었으므로 player pathfinding, `MoveToPoint`,
movement segment와 이동 도중 recovery는 이 기능의 변경 범위가 아니다.

### 자동 기본 공격과 저지

기본 공격은 계속 Core가 자동으로 target을 선택한다.

- 근거리 player basic attack은 blocker-first다.
- 나머지는 weapon `targeting_profile`을 사용한다.
- skill은 명시 `UnitTargetRule`을 사용하며 weapon profile을 상속하지 않는다.
- deterministic expected damage 또는 실제 hostile target-applied effect가 있어야 유효 target이다.

Basic attack 시작은 현재 이동을 중지하고 windup 종료까지 movement/basic attack lock을 건다.
이미 `AttackStart`가 기록된 공격은 `AttackResolve` 예약 event로 처리되며 별도 attack cancel epoch은 없다.

저지는 sticky engagement다. 문서와 runtime 모두 거리 이탈만으로 기존 engagement를 해제하지 않는다.
사망, 철수, unblockable 전환 또는 강제 이동 같은 명시 조건이 필요하다. Player relocation이 저지 중
허용된다면 이동 시작 시 engagement를 해제할지, 이동을 거부할지를 반드시 새 정책으로 정해야 한다.

### 수동 스킬

`ActivateSkill` request의 `target`은 이미 다음 wire shape를 지원한다.

- `Unit { unit_instance_id }`
- `Tile { position }`

장착 active skill은 employee combat profile에서 `SkillActivationMode::Manual`로 전환된다. 그러나 현재
live skill 25개의 cast-level target은 모두 `EnemySingle` 또는 `SelfUnit`이며 cast-level
`CastTarget`을 사용하는 실제 manual tile skill은 없다.

현재 동작은 다음과 같다.

- `SelfUnit`: target 생략 시 자신을 자동 선택한다.
- `EnemySingle`: target 생략 시 skill rule로 자동 선택한다.
- 명시 Unit을 보내면 hostility/air/reach를 검사해 해당 Unit을 사용할 수 있다.
- range preview는 `EnemySingle`을 `AutoUnit`, `CastTarget`을 `ManualTile`로 분류한다.
- `AutomaticUnit`과 `PlayerUnit`을 구분하는 live authoring field는 없다.
- 모든 live skill이 `focus_permissions`를 생략하므로 focus 중 이동/기본 공격은 기본적으로 금지된다.

기존 `ReuseCastTarget` 계약은 Unit direct effect와 Unit-anchor TileArea를 다르게 처리한다.

- direct Unit effect는 실행 시 대상이 죽거나 적대성/air/usefulness를 잃으면 대상 없이 끝난다.
- TileArea는 선택 시 캡처한 target tile을 사용하므로 대상이 죽거나 철수해도 같은 타일에서 실행된다.
- `RetargetOnStep`은 기존 cast target을 고치는 기능이 아니라 step 실행 시 같은 cast reach/range/air
  규칙으로 새 target을 자동 선택하는 기능이다.

### 전장 오브젝트와 tactical point

현재 PVE tactical point live schema는 `id + position`만 가진다. Live data에는 Freischütz encounter의
`sniper_nest` 한 건이 있다. Runtime behavior, assignment, capacity, duration 또는 effect는 없다.

`battle_setup_snapshot.static_objects`와 `initial_units`는 현재 빈 배열이다. 외부 Unity 계약상
`static_objects`는 비동적 scene prop 전용이며 HP/상태 actor의 source가 아니다. 상태와 작업자를
가지는 field work object를 이 배열에 재사용하면 기존 계약과 충돌한다.

### checkpoint, event와 recovery

기존 경계는 다음과 같다.

- `battle_setup_snapshot`: immutable scene construction source
- `battle_update.events_delta`: 연출 timing source
- `battle_update.checkpoint`: 현재 상태와 reconcile/resync source
- movement presentation: `MovementSegmentStarted` / `MovementStopped`
- checkpoint world position: movement animation source가 아니라 reconcile target
- active battle resync: current revisioned projection
- run SavePoint checkpoint: active battle resync와 별개
- `RetreatBattle`: battle entry 전에 캡처한 snapshot으로 정확히 rollback

전술 명령도 event만으로 상태를 재구성하게 만들면 안 된다. 이동 intent, cooldown, stance, field work
assignment와 진행 상태는 checkpoint에 포함해야 한다.

## 확인된 문서와 runtime의 충돌·공백

### 1. 아군 deployment tile 중복

외부 Unity canonical은 아군 deployment cell 중복을 허용하지 않고 `position_occupied`로 거부한다고
명시한다. 실제 runtime deploy validator와 `BattleCore` placement는 dynamic player occupancy를
검사하지 않는다.

현재 동작을 기준으로 보면 문서가 runtime보다 앞서 있다. 이동/자리 교대 구현 전에 다음 중 하나를
새 canonical로 승인해야 한다.

- player tactical slot 독점 점유
- player unit 동일 타일 중복 허용

권고: player tactical slot을 독점 점유로 확정하고 첫 이동 vertical slice에서 기존 deploy 중복도
같이 수정한다.

### 2. 명시 manual tile target의 range 검증

Canonical은 skill cast 후보를 `SkillCastTargetingDef.reach + target_range`로 제한한다. 실제
`explicit_skill_cast_target_is_valid`는 `SkillTarget::CastTarget + Tile`에 대해 battlefield valid tile만
검사하고 cast reach/range를 검사하지 않는다.

현재 live manual tile skill이 없어 발현하지 않는 dormant gap이지만 `PlayerTile` 도입 전에 수정해야 한다.

### 3. manual tile target 생략

현재 manual activation validator는 `CastTarget` skill의 target이 생략되어 target resolution이
`None`이어도 요청을 허용할 수 있다. `PlayerTile` mode를 도입하면 target 생략을 typed
`NoValidTarget`으로 거부하고 자동 fallback을 금지해야 한다.

### 4. Unit target control authority 부재

Runtime은 명시 Unit target을 받을 수 있지만 live data와 DTO metadata에는 `AutomaticUnit`과
`PlayerUnit`의 authoring 구분이 없다. 현재 `EnemySingle`은 모두 `AutoUnit`으로 표시되고 target을
생략하면 자동 선택된다. Player-selected Unit skill을 도입하려면 targeting semantics와 selection
authority를 분리해야 한다.

### 5. 임의 player path 부재 — 전술 순간이동 범위에서 해소

Continuous movement와 obstacle collision은 존재하지만 arbitrary destination의 deterministic path를
계산하는 navigation layer는 없다. 전술 이동을 원자적 순간이동으로 승인했으므로 이 navigation
layer는 해당 기능에 필요하지 않으며 새 player path를 추론해 추가하지 않는다.

### 6. Platform traversal 의미 부재 — 보행 정책은 범위 밖

Battlefield live data는 Ground와 Platform deployment tile을 구분하지만 `BattleCore` movement layout은
valid/obstacle만 알고 tile kind를 traversal policy로 사용하지 않는다. 전술 순간이동에는 traversal이
없으므로 이 공백은 보행 정책으로 남겨 두고, 순간이동 destination에는 별도로 승인할 deployment
affinity/zone validation만 적용한다.

### 7. Platform 배치 가능한 live 직원 부재와 저지 authority 차이 — 정책 해소, 구현 필요

현재 모든 employee `UnitCombatProfile`은 기본 `deployment_affinity: GroundOnly`로 생성되고 live employee
RON은 affinity를 authoring하지 않는다. `WeaponCombatProfile`에도 deployment affinity field가 없어서
현재 live 장비만으로 Platform 배치 직원을 만들 수 없다.

또한 직원 block capacity는 실제 배치 tile kind가 아니라 장착 무기 RON이 단독으로 정한다. 현재 기준은
Shield 3, Sword/Spear 1, Bow/Gun/Staff와 무기 없음 0이다. Runtime은 `PlatformOnly` profile의 block을
0으로 만들지만 실제 tile kind를 보지 않으므로 향후 `Any` 직원이 Platform에 배치될 때 같은 보장이 없다.

승인된 해소 정책은 다음과 같다.

- 각 weapon RON이 `deployment_affinity`, 기본 `block_capacity`, Ground `DefenseFocus`와 Platform `Rest`
  profile을 직접 명시한다.
- Ground를 허용하는 `GroundOnly`와 `Any` weapon도 명시적인 기본 `block_capacity: 0`을 가질 수 있다.
  배치 capability와 저지 역할은 결합하지 않으며, 누락값을 archetype이나 기본값으로 보충하지 않는다.
- `PlatformOnly` weapon은 기본 `block_capacity: 0`이어야 하며, 실제 Platform tile에서는 `Any` weapon도
  effective block capacity 0을 사용한다.
- 실제 current tile kind가 활성 `DefenseFocus/Rest` profile을 선택한다. Weapon archetype이나 직원
  기초 스탯에서 이를 추론하지 않는다.
- 현재 모든 Staff weapon은 `PlatformOnly`로 authoring한다.

## 최소 일관 설계 권고안

### 상태 ownership

현재 `LiveBattleDeploymentState`를 범용 상태로 계속 키우지 않고 다음 책임으로 분리한다.

```text
ActiveBattleSession
├─ TacticalResourceState
│  ├─ current
│  ├─ max
│  ├─ recovery_per_second
│  └─ last_update_ms
├─ DeploymentState
│  ├─ base/redeploy cost policy
│  ├─ deployed employee ↔ runtime unit mapping
│  ├─ redeploy locks
│  └─ instance salt
└─ BattleCore
   └─ RuntimeUnitTacticalState
      ├─ stance
      ├─ tactical relocation cooldown
      ├─ stance transition cooldown
      └─ exclusive activity
```

승인된 exclusive activity:

```text
Ready
FieldWorking
```

Lifecycle, stance와 exclusive activity는 서로 다른 축으로 둔다. 순간이동은 원자적 mutation이므로
pending/relocating activity를 checkpoint에 보존하지 않는다.

DTO에서는 `checkpoint.deployment`에 들어 있던 자원 필드를 제거하고
`checkpoint.tactical_resource`를 별도 authority로 두는 방향을 권고한다. 구/신 field를 동시에
운반하는 compatibility shape는 만들지 않는다.

### 단일 유닛 전술 순간이동

승인된 의미를 반영한 command 후보:

```text
TacticalRelocateUnit {
    employee_uuid,
    destination,
    facing,
}
```

승인된 의미:

- 기존 withdraw/redeploy command와 lifecycle을 호출하지 않는다.
- 직원은 `Defense` 태세여야 한다.
- 스킬 실행, hard CC 또는 이동을 금지하는 exclusive activity 중이면 거부한다. `FieldWorking`은
  basic/skill/stance effect를 막지만 전술 순간이동 source eligibility는 막지 않는다.
- destination validation, 자원과 직원별 전용 cooldown을 모두 검사한 뒤 한 mutation으로 위치와 facing을
  변경하고 비용/cooldown을 적용한다.
- HP, buff/debuff, resonance, attack/skill cooldown, stance와 runtime identity를 보존한다.
- 저지, target eligibility, range와 aura/area 포함 관계는 새 위치에서 다시 계산한다.
- 경로, body 이동 segment, source/destination reservation, mid-move checkpoint와 cancel/refund 단계가 없다.

필요 event의 최소 후보:

- `UnitTacticallyRelocated`

한 semantic event가 이전/새 logical position, world position, facing과 `source_command_id`를 전달한다.
Unity는 이 event로 회수/재배치 또는 순간이동 연출을 하고 checkpoint를 최종 reconcile source로 사용한다.

Checkpoint에는 최소한 다음이 필요하다.

- 최종 position/world position/facing
- stance와 stance transition `ready_at_ms`
- tactical relocation `ready_at_ms`
- tactical command eligibility/capability에 필요한 current state

### 목적지 점유

승인된 점유 모델:

- player unit은 active tactical slot 하나를 소유한다.
- deploy, relocation과 swap은 같은 occupancy validator를 사용한다.
- 순간이동은 원자적이므로 reservation state를 만들지 않는다.
- 빈 destination으로는 단일 순간이동, 점유된 두 slot 사이에는 자리 교대만 허용한다.
- 기존 deploy도 점유된 player slot을 `position_occupied`로 거부한다.

### 자리 교대

권고 command:

```text
SwapUnitPositions {
    first_employee_uuid,
    second_employee_uuid,
    first_facing,
    second_facing,
}
```

승인된 의미는 실제 동시 보행이 아니라 한 revision 안의 원자적 logical/world position 교환이다.

- 두 직원 모두 active combatant다.
- 둘 다 `Defense` 태세이며 진행 중인 기본 공격·skill 또는 hard CC 상태가 아니다. `FieldWorking`인
  직원도 source 또는 target이 될 수 있고, 그 밖의 이동 금지 exclusive activity는 거부한다.
- 각 직원은 상대 slot에 대해 최초 배치와 같은 zone/affinity/valid tile 검사를 통과해야 한다.
- 양쪽 tactical relocation cooldown이 모두 준비되어야 한다.
- 양쪽 위치와 새 facing을 한 mutation으로 적용한다.
- 양쪽 spatial derived state를 다시 계산한다.
- `UnitPositionsSwapped` 같은 전용 event를 연출 source로 사용한다.
- 교대 비용은 command당 fixed cost `5`를 한 번 지불하고 양쪽 tactical relocation cooldown
  `10,000ms`를 모두 시작한다.

### 전술 선택 Unity interaction과 전술 집중 모드 — 승인

- `전술 집중 모드`는 전술 이동과 PlayerUnit/PlayerTile 수동 skill targeting이 공유하는
  server-authoritative modal selection mode다. Server projection은 active 여부, 목적
  (`Relocation`/`ManualSkillTargeting`)과 복원할 이전 playback state를 명시한다.
- 플레이어에게는 일반 순간이동과 자리 교대를 하나의 `전술 이동` 메뉴로 제공한다.
- Unity는 한 interaction으로 보여 주지만 wire command는 `RelocateUnit`과 `SwapUnitPositions`처럼
  명시적으로 분리한다. Occupancy가 달라졌다고 server가 두 명령 사이를 자동 fallback하지 않는다.
- source와 swap target의 facing 기본값은 각각 그 unit이 이동 전에 사용하던 기존 facing이다.

승인된 흐름:

1. 전술 메뉴에서 `전술 이동`을 누르면 server-authoritative `전술 집중 모드`에 진입한다. 실행 중인
   전투에서는 Server가 진입 전 사용자 배속을 저장하고 실제 battle simulation 배속을 `policy.ron`의
   `tactical_focus_speed: X0_5`로 낮춘다. 현재 Core가 지원하는 speed enum 중 하나를 policy가 선택하며
   첫 live 값은 `0.5x`다. 이미 Pause라면 paused playback을 그대로 유지해 battle time을 진행시키지
   않는다.
2. 행동 가능한 `Defense` source unit을 선택한 뒤 target tile을 선택한다. 여기서 행동 가능은 hard CC가
   없다는 조건뿐 아니라 진행 중인 기본 공격·skill 부재, relocation cooldown과 command eligibility를
   포함해야 한다. `FieldWorking`은 전술 이동을 막지 않으며 이동과 같은 mutation에서 기존 assignment를
   종료하고 destination work cell의 새 assignment를 판정한다.
3. Target에 active allied unit이 없으면 일반 전술 순간이동 ghost preview를 만든다. 적의 projected
   overlap은 자리 교대 분기 조건이 아니다.
4. Target에 다른 active allied unit이 있으면 자리 교대 ghost preview를 만든다. Target이 `Defense`가
   아니거나 hard CC/진행 중 행동/cooldown/placement eligibility를 실패하면 클릭을 거부하되 전술 집중
   모드는 유지한다.
5. Ghost는 Unity-local presentation일 뿐 Core position, facing, block/effect, resource, cooldown, event와
   revision을 바꾸지 않는다.
6. 일반 이동 preview에서는 source facing을, 자리 교대 preview에서는 source facing 다음 target facing을
   선택한다. 각 기본값은 해당 unit의 이동 전 facing이다.
7. 필요한 마지막 facing 선택 시 Unity가 현재 authority와 함께 완성된 relocation 또는 swap command를
   보낸다. Server는 position, 양쪽 facing, resource, cooldown, stance/activity, occupancy와 placement를
   다시 검증하고 전부를 한 mutation으로 적용하거나 전부 거부한다.
8. 성공하면 같은 authoritative commit에서 진입 전 playback state를 복원하고 전술 집중 모드를
   종료한다. 실행 중 진입했다면 이전 배속으로 돌아가고, Pause 중 진입했다면 Pause를 그대로 유지한다.
   실패하면 gameplay state를 바꾸지 않고 전술 집중 모드를 유지한다.

추가 승인 규칙:

- 전술 집중은 언제든 취소할 수 있다. Commit 전 ghost와 local 선택을 버리고 비용·cooldown 없이 진입 전
  배속을 복원한다. 이미 성공한 이동을 취소로 되돌리지는 않는다.
- Source가 선택 뒤 hard CC/death/withdraw/진행 중 exclusive activity 등으로 무효화되면 local 선택과
  ghost를 버리고 전술 집중 모드의 source 선택 단계로 돌아간다.
- 전술 집중 중 일반 배속 변경 UI와 command는 비활성화한다.
- 전술 집중은 modal gameplay 상태다. 집중 중에는 `CancelTacticalFocus`, 현재 purpose에 맞게 완성된
  `RelocateUnit`/`SwapUnitPositions` 또는 targeted `ActivateSkill`, 읽기 전용 state/preview query와 즉시
  `RetreatBattle`만 허용한다. 회수, 태세 전환, 배치, 다른 skill·전술 명령과 일반 playback mutation
  capability는 비활성화하고 server도 직접 요청을 같은 eligibility 규칙으로 거부한다.
- Tactical focus active 여부, purpose와 복원할 이전 playback state는 checkpoint/resync projection
  대상이다. 재접속하면 전술 집중 모드는 유지하되 Unity-local source/target/facing 선택은 버린다.
  `Relocation`은 source 선택부터, `ManualSkillTargeting`은 해당 caster/skill의 target 선택부터 재개한다.
- Pause 중에도 전술 집중 모드에 진입할 수 있다. 이 경우 `0.5x`로 재생하지 않고 frozen battle time에서
  ghost·방향 또는 skill target을 선택하며, 완성된 relocation/swap/targeted skill command만 즉시
  authoritative commit한다. 성공·취소 뒤에도 Pause를 유지한다.

PlayerUnit/PlayerTile 수동 skill 흐름:

1. `Attack` 태세의 actionable caster가 player-targeted skill을 선택하면
   `ManualSkillTargeting { caster, skill_id }` 전술 집중 모드에 진입한다. Target을 요구하지 않는 Self 또는
   automatic manual skill은 전술 집중 없이 즉시 command를 보낸다.
2. 실행 중 전투에서는 policy의 `tactical_focus_speed`를 적용하고, Pause 중이면 frozen playback을
   유지한다.
3. Unity는 Core의 read-only target preview를 사용해 candidate Unit 또는 Tile과 예상 범위를 표시한다.
   선택은 local preview일 뿐 gameplay state를 바꾸지 않는다.
4. 최종 target 선택 시 현재 authority를 포함한 `ActivateSkill`을 보내고 server가 caster/stance/activity,
   resource/resonance/cooldown, target identity/tile, hostility/air/usefulness/range를 다시 검증한다.
5. 성공하면 skill 시작과 playback 복원·전술 집중 종료를 같은 authoritative commit으로 처리한다.
   실패하면 skill/resource/cooldown/revision을 바꾸지 않고 targeting 집중 모드를 유지한다.
6. 취소하면 target preview를 버리고 비용 없이 진입 전 playback state로 복원한다.

### 공격/방어 태세

권고 상태:

```text
EngagementStance::Attack
EngagementStance::Defense
```

승인된 공통 의미:

- `Attack`: 현재 자동 기본 공격과 저지 규칙을 사용한다.
- `Defense`: 새 정규 기본 공격과 automatic/manual skill을 시작하지 않으며 전술 순간이동/교대의 준비
  상태가 된다.
- hard CC가 아니면 기본 공격 windup과 skill cast/focus 중에도 태세와 효과를 즉시 전환한다.
- 전환 전에 확정된 기본 공격과 skill은 delayed step, proc, 연속타와 `ExtraAttack`까지 취소하지 않고
  완료하며, 이미 발사된 projectile과 생성된 area/buff도 authored lifecycle을 유지한다.
- `Defense`는 후속 정규 기본 공격과 automatic/manual skill 시작만 막고 passive/reactive `Triggered`
  효과는 유지한다.
- 아직 방출되지 않은 공격 windup·skill focus가 끝날 때까지 순간이동과 교대는 거부한다. 방출 뒤 남은
  projectile, delayed delivery, area와 buff/debuff는 전술 명령을 막지 않는다.
- 별도 `desired_stance`, 공격 취소 token 또는 태세 전환 예약 상태는 만들지 않는다.
- 태세 전환은 tactical resource를 소비하지 않지만 직원별 대칭 `1,500ms` battle-time cooldown을
  시작한다. 최초 배치·재배치의 `Attack` 진입도 같은 cooldown을 시작한다.
- 직원은 순간이동 뒤에도 `Defense`를 유지한다.
- `Defense` effect profile은 weapon archetype이 아니라 현재 deployment tile kind가 선택한다.
- Ground `Defense/DefenseFocus`는 새 공격·skill을 금지하고 weapon-authored block capacity 증가,
  incoming damage 감소 효과를 받으며 전술 자원을 회복하지 않는다.
- Platform `Defense/Rest`는 새 공격·skill을 금지하고 weapon-authored HP와 resonance 회복 효과를 받으며
  전술 자원을 회복하지 않는다.
- 배치 적성과 두 태세 effect profile의 authoring source는 개별 weapon RON이고, 실제 current tile
  kind가 활성 profile 하나를 선택한다. `Any` 직원의 Ground/Platform 순간이동·교대에서는 같은 mutation
  안에서 두 효과를 원자적으로 전환한다.

### 수동 스킬 대상 지정

`SkillTarget`은 기준 대상의 종류를 계속 소유하고, player control 여부는 별도 authoring field가
소유한다.

```text
SkillCastSelection
├─ Self
├─ AutomaticUnit
├─ PlayerUnit
└─ PlayerTile
```

승인된 규칙:

- 모든 live skill은 selection을 명시적으로 작성한다.
- `AutomaticUnit`은 현재 `EnemySingle.rule` 자동 선택을 유지한다.
- `PlayerUnit`은 explicit Unit target이 필수이며 생략 시 자동 fallback하지 않는다.
- `PlayerTile`은 explicit Tile target이 필수이며 생략 시 자동 fallback하지 않는다.
- command acceptance 때 hostility, air eligibility, usefulness와 final cast range를 검사한다. 이 최초
  검사를 실패하면 skill을 시작하지 않고 resource/resonance/cooldown을 소비하지 않는다.
- focus 시작 시 선택 Unit identity 또는 Tile을 캡처한다.
- PlayerUnit direct effect는 실행 시 target이 무효면 no-op으로 끝난다.
- PlayerUnit 기준 TileArea는 선택 당시 target tile에서 실행한다.
- PlayerTile은 선택 tile을 유지한다.
- Skill acceptance 뒤 target invalidation에는 자동 retarget이나 resource/resonance/cooldown 환불을 하지
  않는다.
- 첫 버전에서는 `PlayerUnit/PlayerTile + RetargetOnStep` 조합을 live validation으로 금지한다.
- 실행 중 전투의 PlayerUnit/PlayerTile target 선택은 공통 전술 집중 모드에서 policy-authored `0.5x`를
  사용한다. Pause 중에는 frozen playback을 유지한다. 최종 target 선택 전 preview는 Unity-local이며,
  성공한 `ActivateSkill`만 Core state를 변경한다.

PlayerUnit 선택 후보를 Unity가 local rule로 재계산해서는 안 된다. 동적인 exact target list가 필요하면
read-only `RequestSkillTargetPreview` query가 현재 revision 기준 Unit IDs 또는 final tile cells를
반환하고 실제 `ActivateSkill`이 다시 원자적으로 검증한다.

### 현장 작업

`tactical_points` 또는 기존 `static_objects`에 상태를 덧붙이지 않고 별도 field work object 계약을
사용한다.

Setup authoring:

```text
FieldWorkObjectDef
├─ id
├─ presentation/object kind
├─ position
├─ work_cells
├─ required_work_ms
└─ typed completion effect
```

Runtime/checkpoint:

- object state
- authoritative `accumulated_work_ms`
- work cell별 current worker assignment

승인된 의미:

- 별도 `AssignFieldWork`/`CancelFieldWork` command를 두지 않는다. Active allied unit이 authored work
  cell을 점유하면 해당 object의 field work assignment를 자동으로 시작하며, cell을 벗어나면 자동으로
  종료한다. 최초 배치와 기존 회수/전투불능 뒤 재배치로 work cell을 직접 점유하는 것도
  허용하며 배치 commit과 같은 mutation에서 `Defense + FieldWorking` worker가 된다.
- Field work는 별도 작업 비용을 소비하지 않는다. Work cell 진입을 위해 수행한
  배치·전술 순간이동·자리 교대에 원래 비용이 있다면 그 command의 비용만 적용한다.
- 전술 순간이동으로 빈 work cell에 들어간 `Defense` source는 이동 commit과 같은 mutation에서 worker가
  된다.
- 자리 교대로 점유된 work cell에 들어간 source는 worker가 되고, work cell에서 밀려나온 기존 target
  worker는 assignment를 종료한 뒤 새 tile의 일반 `Defense` 상태와 tile-kind effect로 즉시 복귀한다.
- Work cell의 worker는 작업 중 basic attack, skill, block과 stance toggle을 할 수 없으며 Ground
  `DefenseFocus`와 Platform `Rest` 효과도 모두 중단한다. 다만 전술 순간이동·자리 교대의 source 또는
  target이 될 수 있고 기존 withdraw 정책으로 회수할 수도 있다.
- 작업자는 계속 damage를 받을 수 있다.
- 적이 같은 tile에 겹치거나 worker를 공격하는 사실 자체는 assignment를 종료하거나 progress를 중단하지
  않는다. Enemy occupancy는 player-side work-cell assignment 자격을 바꾸지 않는다.
- 각 work cell은 전역 `1 tile = 1 allied unit` 규칙을 따른다. 작업 자격이 유효한 각 worker는
  battle time 1ms당 object의 `accumulated_work_ms`를 1ms씩 증가시킨다. 여러 work cell이 동시에
  점유되면 worker별 기여를 합산한다. 예를 들어 두 worker가 각각 3초를 유효하게 작업하면
  총 6,000ms의 작업량이 누적된다.
- 누적 작업량은 `required_work_ms`까지 clamp하고, 최초로
  `accumulated_work_ms >= required_work_ms`가 된 authoritative battle update에서 완료를 한 번만 처리한다.
- Hard CC 동안 assignment와 누적 progress/남은 시간을 보존한 채 작업 진행만 일시정지하고, 해제 뒤
  남은 지점부터 재개한다. CC 시간을 소급 진행하지 않는다. Assignment별 authoritative suspension
  state를 저장하지 않고 현재 `hard_cc_active` capability에서 작업 기여 가능 여부를 파생한다.
- Work cell 이탈, source/target 자리 교대, death 또는 withdraw는 해당 worker assignment를
  해제한다. 누적 progress는 worker가 아니라 object state가 소유한다.
- 하나의 object에 여러 work cell과 worker assignment가 있을 수 있다. 특정 worker가 이탈해도
  다른 assignment가 남아 있으면 object의 작업은 중단된 것으로 보지 않는다.
- 첫 버전은 `Preserve` 의미만 지원한다. 모든 assignment가 없어져도 object progress를 보존해 다음
  worker가 남은 지점부터 이어받게 하며 사용하지 않는 Reset enum과 초기화 분기는 만들지 않는다.
- 요구 작업량이 채워지면 object state를 `Completed`로 바꾸고 work cell을 일반 underlying tile로
  되돌린다. 그 cell에 남아 있는 worker는 같은 mutation에서 작업 assignment를 끝내고 일반
  `Defense` 태세와 해당 tile-kind effect로 복귀한다.
- Pause 중 work cell 진입·이탈 mutation은 15번의 즉시 직렬 규칙으로 처리하지만 battle time이 정지되어
  작업 progress는 진행하지 않는다.
- worker assignment start/stop과 object completion만 semantic event로 보내며 reconnect 진행 상태는
  checkpoint가 소유한다.
- field work completion은 battle time과 stable object/assignment identity만 사용하고 event seq나 HashMap
  순서에 의존하지 않는다.

첫 실제 object는 `suppress_freischutz`의 1회성 `black_box_emergency_repair`다. Presentation kind는
`EmergencyRepairTerminal`, object position과 단일 Ground work cell은 `(2, 6)`, required work는
`10,000ms`, completion은 `RepairProtectedUnit { flat_hp: 100 }`으로 확정했다. 보호 대상이 full HP여도
완료·소모한다. 범용 scripting engine이나 스킬 effect 전체를 재사용하지 않고 첫 vertical slice에 필요한
이 typed effect만 만든 뒤 두 번째 실제 콘텐츠에서 공통화를 재검토한다.

## 결정론과 transport 불변식

모든 단계에서 다음을 유지한다.

- 새 gameplay RNG가 필요하면 event seq가 아니라 stable gameplay identity에서 파생한다.
- candidate iteration은 고정된 tile/Unit ordering을 사용한다.
- command validator는 비용/slot/stance/activity/cooldown을 모두 확인한 뒤 원자적으로 mutation한다.
- 실패와 stale authority는 state, event, resource, cooldown과 revision을 바꾸지 않는다.
- 성공한 command는 projection revision을 정확히 한 번 증가시킨다.
- command가 만든 semantic event는 `source_command_id`를 보존한다.
- events delta는 timing, checkpoint는 current state/reconcile source다.
- Pause 중 허용된 command는 같은 battle time에서 revision 순서로 직렬화한다.
- 실제 진행 시간과 cooldown은 authoritative battle time을 사용한다.
- setup shape가 바뀌면 `setup_version`을 올린다.
- active battle resync와 run SavePoint checkpoint를 합치지 않는다.
- retreat는 기존 battle-entry rollback 의미를 유지한다.

## 남은 정책·콘텐츠 질문

없음. 1~17번의 의미 정책과 각 번호에서 요구한 첫 live content를 모두 확정했다.

## Code/live data 재대조 뒤 정할 수치

없음. 이번 1~17번 검토 범위의 의미·난이도·초기 tuning은 모두 확정했다. 구현 뒤 실제 playtest에서
balance 조정이 필요하면 의미 정책을 바꾸지 않는 별도 tuning 작업으로 다룬다.

## 예상 변경 범위

정책 승인 뒤 실제 code/live data 대조로 정한 구현 범위에서 다음을 함께 수정해야 한다.

- Gameplay canonical: `docs/archive/game_rulebook_legacy_2026-10-04.md`
- Targeting canonical: `docs/skill_target_contract.md`
- Runtime canonical: `docs/core_runtime_contract.md`
- Data canonical: `docs/data_loading_contract.md`
- 작업 goal: `docs/goals/<approved_goal>/PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
- Core command/DTO: `src/game/behavior.rs`
- World/session ownership와 handler: `src/game/world.rs`, `src/game/world/combat.rs`, `src/game/world/state.rs`
- Battle runtime: `src/game/battle/core/*`
- Battlefield/PVE/skill/run policy schema와 live RON
- Server deserialize/result/message-order mapping tests
- 외부 Unity canonical 두 문서
- 필요 시 Unity implementation goal과 WebSocket probe

Compatibility alias, 구/new DTO 병행, legacy fallback은 사용자 승인 없이 추가하지 않는다.

## 필요한 테스트 범위

### Core/runtime

- 같은 입력/seed/command sequence의 deterministic 순간이동/교대 결과
- invalid destination/affinity/occupancy/stance/activity 거부 원자성
- unarmed 직원은 전투 roster에 disabled로 남고 Unity 선택·drag·preview·command가 없으며 deploy
  capability와 server deployment가 `weapon_required`로 거부됨
- 비용 부족과 cooldown eligibility
- 순간이동의 단일 acceptance/commit 시점과 unit-owned state 보존
- 기본 공격 windup/skill 중 태세와 효과의 즉시 전환
- 태세 전환 전에 시작된 `AttackResolve`/skill 완료와 기존 projectile/area 유지
- 진행 중 행동 완료 전 순간이동·교대 거부 및 `Defense` 전환 뒤 후속 공격·skill 시작 금지
- 새 위치의 block/target/range/aura 재계산
- skill/work/hard CC/death/withdraw 연동
- swap 양쪽 검증과 단일 revision commit
- stance attack gate, transition cooldown과 tile-kind effect
- field work progress/pause/cancel/complete
- PlayerUnit/PlayerTile target range와 invalidation

### World/server/transport

- stale/missing authority가 event/resource/cooldown/revision을 바꾸지 않음
- 성공 command가 revision을 정확히 한 번 증가
- `source_command_id` correlation
- checkpoint post-teleport/mid-work current state
- reconnect/setup-loss recovery와 resync
- Pause 중 단일 in-flight mutation의 즉시 success/reject, revision 직렬화와 frozen battle-time 적용
- Pause 중 연속 command에서 앞 성공 보존, 뒤 실패 무변경, cooldown 미진행에 따른 재전환 거부
- DTO casing/nullability/shape serialization
- command ack 뒤 battle update 순서

### Live data

- strict embedded RON loading
- missing/unknown/stale tactical field 거부
- 모든 weapon의 명시 deployment_affinity와 block_capacity 교차 validation
- invalid tactical policy/field-work object/cross-reference 거부
- 모든 live skill의 selection 명시 여부
- live skill catalog audit와 manifest 동기화

## 조사 시점 검증

조사 중 다음 기존 테스트를 실행했고 통과했다.

- `live_ron_defense_route_playable_path_runs_to_combat_result`
- `manual_fragment_can_restore_stabilization_and_enable_extra_deployment`

이 결과는 현재 baseline이 유효하다는 근거일 뿐, 위 전술 지휘 확장이 구현되었거나 정책이 승인됐다는
의미가 아니다.

## 다음 단계

1~17번 정책 검토, 현재 tree 기준 코드·live data·server 대조와 전체 dependency/vertical slice 종합이
끝났다. 사용자가 이 권고 순서를 승인하고 구현을 요청하면 `G0`부터 slice별 별도 goal과
`PLAN/EXPERIMENTS/EXPERIMENT_NOTES`를 만든다. 명시적인 구현 요청 전에는 gameplay runtime, live RON과
transport를 변경하지 않는다.
