# DefenseRoute 전술 지휘 목표 정책 결정 — 2026-08-06

- 정책 승인일: 2026-08-05~06
- 상태: approved target policy / phased implementation in progress
- 정책 ID: `DR-01`~`DR-17`
- 구현 조정:
  [DefenseRoute 전술 지휘 구현 Master Goal](../goals/defense_route_tactical_command_master/PLAN.md)
- 조사·난이도 historical evidence:
  [2026-08-04 임시 조사·설계 노트](../tmp/defense_route_tactical_command_design_notes_2026-08-04.ko.md)

## 문서 책임과 적용 방법

이 문서는 `DR-01`~`DR-17`의 승인된 목표 의미와 첫 live tuning을 소유한다. 시점 runtime 조사,
구현 난이도, 초기 권고안과 code listing은 소유하지 않는다. 구현 Phase는 아래 정책을 축소하거나
과거 조사 문구로 대체하지 않는다.

이 문서는 아직 구현되지 않은 부분을 포함한 승인 목표 정책을 소유한다. 미구현 항목을
완료된 동작으로 표현하지는 않지만, Core/game_server code와 production live data는 이 문서의 승인
의미를 따라야 한다.

- 현재 구현은 Core/game_server code와 production live data에서 조사하되, 이 문서와 다른 부분은
  implementation drift다.
- 각 정책은 배정된 Phase가 code/data/server와 관련 runtime canonical을 함께 구현·동기화하고
  완료 감사를 통과해야 구현 완료가 된다.
- Goal 문서와 historical evidence는 실행·조사 기록이며 gameplay/data/wire source of truth가 아니다.
- Unity runtime, contract, UI와 tests는 승인된 구현 범위 밖이다.

## 공통 불변식

- Core가 전투 상태와 command 수락 여부의 유일한 authority다.
- 모든 command는 전체 사전 검증 뒤 원자적으로 commit한다. 실패는 resource, cooldown, state,
  event와 revision을 변경하지 않는다.
- Pause는 simulation clock만 정지하고 승인된 즉시 command/query 처리는 계속한다.
- released delivery와 unreleased action cancellation을 구분한다.
- checkpoint/reconnect는 authoritative state를 복원하고 중단 시간을 catch-up하지 않는다.
- 같은 입력, live data, seed와 command 순서는 같은 state/event 결과를 만든다.
- server와 consumer는 Core 계약을 재추론하지 않는다.
- 승인되지 않은 fallback, retarget, silent recovery, alias, dual-read/write와 compatibility는 없다.
- 미래 정책을 위한 빈 enum, placeholder state/schema와 사용하지 않는 범용 framework를 선행하지 않는다.

## 정책 결정

### DR-01 — 전투 목표

- 공식 전투 모델은 `DefenseRoute` 하나다. 보스와 생존 전투도 별도 전투 장르가 아니라 같은 모델의
  authored 변형이다.
- 기본 공격과 기본 공격 대상 선택은 자동 전투와 weapon targeting profile을 유지한다. 모든 공격을
  수동 지정하는 고 APM RTS를 목표로 하지 않는다.
- 배치·철수·태세·전술 이동·수동 스킬·현장 작업은 기존 목표를 달성하는 전술 수단이며, 사용 여부
  자체는 승패 조건이 아니다.
- 모든 필수 wave/적 처리와 보호 대상 생존을 기본 승리 조건으로 유지한다. 보호 대상 파괴와 encounter가
  명시한 scenario failure condition은 패배다.
- 생존 전투는 `DefenseRoute / Defense + survive_timer_ms`이며 timer에 도달하면 남은 적과 무관하게
  승리한다.
- Field work를 승패 조건에 포함하려면 encounter가 명시적으로 authoring해야 한다.

### DR-02 — 전술 포인트

- 플레이어 표시명은 `전술 포인트`, code/DTO 이름은 `tactical_resource`다. 새 계약에
  `stabilization` 또는 일반 `cost`를 자원 이름으로 남기지 않는다.
- 전술 포인트는 active battle의 player-side 전역 자원이다. 전투 시작 때 초기화하고 reconnect/resync는
  현재 값을 보존하며 전투 종료·Retreat에서는 폐기해 다음 전투로 이월하지 않는다.
- 최초 배치, 회수/전투불능 뒤 재배치, 전술 순간이동과 자리 교대가 하나의 current/maximum pool을
  공유한다. Field work 자체에는 별도 비용이 없다.
- current는 `0..=maximum`으로 clamp한다. 초과 회복·감소를 저장, 이월 또는 다른 값으로 변환하지 않는다.
- 비용은 전액 보유할 때만 원자적으로 지불한다. 부분 지불, debt와 음수 잔액은 없고 commit된 비용은
  이후 상황 변화로 환불하지 않는다.
- maximum은 전투 시작 때 고정한다. 첫 버전은 current 증감만 허용하며 encounter별 initial/maximum/
  자연 회복 override와 maximum 변경 effect를 제공하지 않는다.
- `run/policy.ron`이 공통 initial/maximum/자연 회복 및 배치·재배치·이동·교대 비용을, 개별 skill RON이
  명시적인 즉시 증감을 소유한다. `DefenseFocus`와 `Rest`는 전술 포인트를 생산하지 않는다.
- Skill, weapon과 encounter가 명시적으로 authoring한 effect는 current만 증감할 수 있고 각 producer가
  발동 조건, 양, 주기와 중첩을 소유한다. 자연 회복 외 암묵적인 환경 감소나 기본 적 drain은 없다.
- 자연 회복은 authoritative battle time을 사용하고 Pause 중 진행하거나 Resume 뒤 catch-up하지 않는다.
- 첫 live 값은 initial `20`, maximum `99`, 자연 회복 `1/초`, 기본 배치 비용 `10`이다.

### DR-03 — 기존 회수와 재배치

- active 배치 player unit은 태세, 기본 공격 windup, skill cast/focus, Hard CC, Silence와
  `FieldWorking`에 관계없이 자발적으로 철수할 수 있다. 사망·전투불능·이미 철수한 unit은 불가하다.
- 철수 자체는 전술 포인트를 소비하지 않고 배치 비용을 환불하지 않는다. 같은 runtime instance를 즉시
  `Withdrawn`으로 만들고 제거하며 재배치 때 위치와 방향을 다시 선택한다.
- release 전 기본 공격과 skill cast/focus는 취소한다. 이미 발사된 기본/skill projectile과 생성된
  `GroundFixed` area는 독립 delivery로 기존 lifecycle을 계속한다.
- `FollowCaster`, `FollowTarget`, aura, channel/focus처럼 source runtime에 종속된 효과는 철수와 함께
  종료한다. 별도 유지 authoring option은 추가하지 않는다.
- 철수 unit이 적용한 buff/debuff는 기존 `CasterWithdrawn` 규칙으로 종료한다. 철수 instance 자신의
  buff/debuff, target, resonance, cooldown, cast/action, block과 aura state는 폐기한다. 철수 unit을 향한
  적대 projectile은 miss하고 engagement는 즉시 해제한다.
- 자발적 철수 재배치 HP는 철수 순간
  `min(max_hp, current_hp + floor(max_hp * 30 / 100))`으로 잠근다. 대기 중 추가 회복, physical damage/
  trauma 변경과 후속 max HP 변화의 소급 적용은 없다.
- 전투불능은 runtime instance마다 physical damage `+1`과 authored trauma를 한 번 반영한다. 재배치한 새
  instance가 다시 전투불능이면 새 penalty를 적용한다.
- 전투불능 뒤 최신 condition이 참가 가능할 때만 후보가 되며 시작 HP도 그 condition에서 계산한다.
- 재배치는 같은 employee identity에 새 runtime instance를 만든다. 확정 loadout/fragment, 최신
  persistent condition, 유효한 battle-scoped modifier, stats와 배치 초기 효과를 다시 구성한다.
  이전 transient state, buff, resonance, target, cooldown, cast, block과 aura는 상속하지 않으며
  `InitialSkillCharge`와 authored on-deploy effect는 매 배치 다시 적용한다.
- live 대기는 자발적 철수 `30초`, 전투불능 `90초`, 재배치 비용은 기본 배치 비용의 `150%`다. 비용은
  `ceil(base_deploy_cost * multiplier_pct / 100)`이며 배치 비용 감소는 최초/재배치 모두 적용한다.

### DR-04 — 전술 순간이동

- `TacticalRelocateUnit`은 철수·재배치를 호출하지 않는 별도 server-authoritative command다. 같은 active
  runtime instance의 position과 facing을 한 mutation에서 즉시 바꾼다.
- 보행, pathfinding, 이동 시간/segment, reservation, relocating state, 도착·취소·환불 단계를 만들지
  않는다.
- HP, buff/debuff, resonance, stance, 공격/skill cooldown과 runtime identity를 유지한다. 새 위치에서
  engagement, target/range, tile effect와 area/aura 포함 관계만 다시 계산한다.
- source는 `Defense`여야 하며 release/end 전 기본 공격·skill action과 Hard CC는 사용을 막는다.
  Silence와 `FieldWorking`은 막지 않는다. 개별 command가 상태 이름을 나열하지 않고 공통 capability를
  사용한다.
- 저지 중에도 사용할 수 있다. 기존 engagement를 먼저 해제하고 이동 뒤 표준 deterministic 규칙으로
  다시 계산한다. Work cell 이탈/진입 assignment도 최종 위치에서 같은 mutation으로 reconcile한다.
- destination은 player deployment zone, current weapon affinity, valid tile, static obstacle와 allied
  occupancy 검사를 모두 통과해야 한다. 경로 도달 가능성은 검사하지 않는다.
- 다른 active Player-side actor가 destination tactical slot을 점유하면 `position_occupied`다. 자동
  밀어내기·swap fallback은 없다.
- position과 facing이 모두 같으면 no-op으로 거부한다. 같은 position에서 facing만 바꾸는 재정렬은 같은
  비용/cooldown으로 허용한다.
- source를 대상으로 시작된 delivery를 이동 자체로 취소하지 않는다. 사거리 재검증 공격은 실패할 수
  있고 homing projectile은 계속 추적하며 fixed projectile은 원래 궤도를 유지한다. `GroundFixed`는 원래
  위치, source aura/`FollowCaster`는 새 위치, `FollowTarget`은 기존 target을 따른다.
- 모든 조건, resource와 cooldown을 먼저 검증하고 성공할 때만 position/facing/resource/cooldown과 semantic
  event를 원자 commit한다.
- cost/cooldown 값은 모든 직원에 같지만 cooldown state는 employee별 battle state로 보존한다.
  `DeployCostReduction`은 적용하지 않는다.
- 첫 live 값은 비용 `5`, employee별 cooldown `10,000ms`이며 `run/policy.ron`이 소유한다.

### DR-05 — 아군 tactical tile 점유와 배치 검사

- 점유 단위는 continuous collider가 아니라 Core의 논리적 `Position` tactical tile이다. BattleCore의
  다중 점유 model은 유지하고, 1 tile 1아군은 player 배치·재배치·전술 이동 destination 입력 규칙으로만
  적용한다.
- active Player-side 직원과 방어 목표 같은 actor는 slot을 점유한다. 적의 projected overlap은 허용하며
  이후 공간/저지/공격 판정에 맡긴다.
- 빈 tile도 zone, weapon affinity, valid tile, static obstacle와 command eligibility를 별도로 통과해야
  한다. 최초 배치와 모든 재배치도 점유 tile을 `position_occupied`로 거부한다.
- 사용자에게는 하나의 `전술 이동`을 제공한다. 빈 tile은 relocation, 교대 가능한 아군은 swap preview로
  분기하지만 wire/transaction은 명시적으로 분리한다.
- preview 뒤 latest authority에서 다시 검증하며 점유 변화로 command 종류를 자동 전환하지 않는다.
  실패하면 전술 집중 모드를 유지한다.
- relocation은 source facing 하나, swap은 양쪽 facing을 받는다. Swap은 양쪽 destination 전체 검사를
  통과한 뒤 중간 중복 점유 없이 원자 commit한다.
- source 자신의 현재 tile은 자기 점유에서 제외해 facing-only 재정렬을 허용한다.
- reservation, pending occupancy, client batch, 아군 보행/pathfinding/segment와 자동 강제 이동 예외를
  만들지 않는다. 미래 non-occupying ally 예외를 선행 authoring하지 않는다.

### DR-06 — 공격/방어 태세

- authoritative stance는 `EngagementStance::Attack | Defense` 두 값뿐이다. `DefenseFocus`, `Rest`와
  `TacticalReady`를 별도 stance/boolean으로 저장하지 않는다.
- `Attack`은 기존 eligibility에 따라 자동 기본 공격, automatic/manual skill, targeting과 저지를
  허용하지만 행동을 강제로 시작하지 않는다.
- `Defense`는 새 기본 공격과 automatic/manual skill 시작을 모두 막고 전술 이동 준비 태세가 된다.
  공격/지원/self skill 예외는 없다.
- Defense 효과는 `현재 deployment tile kind + 장착 weapon profile`에서 파생한다. Ground는
  `DefenseFocus`, Platform은 `Rest` 하나만 활성화한다.
- `GroundOnly` weapon은 DefenseFocus만, `PlatformOnly`는 Rest만, `Any`는 둘 다 profile을 명시한다.
  Profile 내부 0은 유효하지만 필요한 profile 누락은 invalid live data다.
- 태세 전환은 resource를 쓰지 않는다. 양방향 전환은 같은 employee별 battle-time cooldown을 시작한다.
  Cooldown은 다음 태세 전환만 막고 이동/교대는 다른 조건 충족 시 즉시 사용할 수 있다.
- 같은 stance 요청은 `already_active`로 무변경 거부한다. transition phase, desired stance, 예약/자동
  복귀와 공격 취소 token은 없다. Stance와 파생 effect를 같은 mutation에서 즉시 바꾼다.
- 최초 배치와 모든 재배치는 `Attack`으로 시작한다. Relocation은 `Defense`를 유지한다.
- 첫 live 전환 cooldown은 양방향 `1,500ms`이며 `run/policy.ron`이 소유한다.

### DR-07 — 태세 전환 중 행동 수명주기

- Hard CC가 아니라면 기본 공격 windup과 automatic/manual skill focus 중에도 태세를 즉시 전환한다.
- 전환 전에 이미 확정된 기본 공격/skill, delayed step, proc, 연속타와 `ExtraAttack`은 끝까지 수행한다.
  Released projectile/area/buff도 기존 lifecycle과 tracking 규칙을 유지한다.
- `Defense`는 이후의 새 정규 기본 공격과 automatic/manual skill 시작만 막는다. 아직 확정되지 않은 예약
  event는 시작하지 않는다.
- Passive/reactive `Triggered` 효과는 Defense에서도 동작한다. Trigger가 실제 공격 windup을 시작하면
  release까지 확정 행동이다.
- Relocation/swap은 기본 공격 `AttackStart..AttackResolve`, skill `CastStart..matching CastEnd` 동안
  거부한다. Release 뒤 남은 delivery만으로 막지 않지만 delayed `ExtraAttack` windup이 시작되면 그
  resolve까지 다시 막는다.
- 최초/재배치로 Attack에 진입할 때도 employee별 `1,500ms` stance cooldown을 시작한다. Relocation은
  기존 Defense를 유지하고 새 stance cooldown을 만들지 않는다.

### DR-08 — Ground DefenseFocus

- Ground tile의 active 직원이 Defense이면 weapon의 `DefenseFocus`가 파생 활성화된다. 별도 state와
  command는 없다.
- Profile은 `block_capacity_bonus`와 `incoming_damage_reduction_percent`를 독립적으로 명시하고 0을
  허용한다.
- Block bonus는 weapon base block과 합성한다. 피해 감소는 자신이 받는 피해에만 적용하고 기존 일반
  피해 감소율에 가산한다. 피해 유형별 감소까지 합친 최종 감소율은 `0..=100%`로 clamp하며 별도 곱연산
  계층을 만들지 않는다.
- 전술 포인트 회복, employee별 자원 timer와 주기 signal은 없다.
- Attack 전환/Ground 이탈 때 즉시 제거한다. Capacity가 줄면 기존 engagement를 route progress, spawn
  order, unit ID 순으로 새 capacity만큼 보존하고 나머지를 해제한다. 신규 적이 기존 engagement를
  밀어내지 않는다.
- 개별 weapon RON이 수치의 단일 authority다. 첫 live Ground weapon 피해 감소는 모두 `10%`; base block이
  1 이상인 Sword/Spear/Shield/훈련검은 block `+1`, base block 0 Bow/Gun은 `+0`이다.

### DR-09 — Platform Rest

- Platform tile의 active 직원이 Defense이면 weapon의 `Rest`가 파생 활성화된다. 별도 stance/state/
  command는 없다.
- 자신의 battle HP와 resonance만 회복한다. 전투불능 unit을 부활시키거나 전술 포인트를 회복하지 않는다.
- 공통 `rest_tick_interval_ms`는 run policy가 소유하고 weapon은 `hp_recovery_per_tick`과
  `resonance_recovery_per_tick`만 명시한다. 각 값은 0을 허용한다.
- 활성화 뒤 온전한 interval 후 첫 tick을 적용한다. 조건을 잃었다가 다시 만족하면 새 interval부터
  시작한다. Epoch/timer는 scheduling bookkeeping일 뿐 gameplay Rest state가 아니다.
- 각 회복은 자신의 maximum에서 clamp하고 초과분·max 상태의 회복량을 저장하지 않는다. 피격과 active
  wave는 중단/recently-hit timer를 만들지 않는다.
- Attack 전환/Platform 이탈 때 즉시 중단한다. Defense에서 resonance가 max여도 auto skill을 시작하지
  않으며 Attack 전환 뒤 정상 eligibility를 다시 평가한다.
- 기존 `resonance_gain_until_ms` lock 동안 해당 resonance tick은 skip하고 catch-up하지 않는다. 실제
  변화가 0이면 변화 event를 방출하지 않는다.
- 같은 timestamp의 committed damage를 Rest보다 먼저 처리한다. 그 피해로 전투불능이면 Rest가 사후
  회복·부활시키지 않는다.
- 첫 live cadence는 `1,000ms`; Staff 16개의 명시 값은 HP `3/tick`, resonance `5/tick`이다.

### DR-10 — 무기 전술 데이터 소유권

- 개별 weapon RON의 `WeaponCombatProfile`이 `deployment_affinity`, base `block_capacity`, Ground
  DefenseFocus 두 수치와 Platform Rest 두 수치의 단일 authoring source다.
- Archetype, employee stat, legacy default와 effective block에서 값을 추론하거나 누락을 보충하지 않는다.
- `GroundOnly`는 DefenseFocus만, `PlatformOnly`는 Rest만, `Any`는 둘 다 명시한다. 사용할 profile은
  required이고 내부 0은 유효하며 사용할 수 없는 tile profile은 작성하지 않는다.
- PlatformOnly base block은 0이다. Any도 실제 Platform에서 effective block 0이다. Ground 가능 weapon은
  base block 0도 허용한다.
- Rest cadence와 stance cooldown은 run policy가 소유한다. 무기 없는 직원 fallback profile은 없다.
- 첫 30개 live catalog는 다음 값을 각 RON에 명시한다.
  - Staff 16: `PlatformOnly`, base block 0, `Rest(HP 3, resonance 5)`.
  - Bow 1/Gun 1: `GroundOnly`, base block 0, `DefenseFocus(block +0, damage reduction 10%)`.
  - Sword 9/Spear 1: `GroundOnly`, base block 1, `DefenseFocus(block +1, damage reduction 10%)`.
  - Shield 1: `GroundOnly`, base block 3, `DefenseFocus(block +1, damage reduction 10%)`.
  - `standard_training_sword` 1: Sword baseline, ZAYIN, Physical/DefaultForward, range 1, interval 1,500ms,
    windup 200ms, Instant, 별도 ability/trigger 없음.
- 첫 live catalog에는 `Any`가 없다. 위 표는 archetype default가 아니라 이번 RON별 migration 값이다.
- Fragment `block_capacity_min`과 certified combat power는 base block만 계속 사용한다.
- 기존 active-run Core payload V2는 보존·자동 보완하지 않고 strict version cutover한다. 구/new schema
  병행과 암묵 migration은 없다.

### DR-11 — 무기 없는 직원

- 무기 없는 직원도 roster와 전투 편성 후보 목록에 표시하며 안전지대 정보/장비 관리와 무기 장착을
  모두 허용한다.
- 전투 편성/active battle에서는 정보 표시 외 gameplay source가 될 수 없다. 최초 편성·배치, 재배치,
  이동·교대, 태세, range preview, manual skill과 field work를 모두 `weapon_required`로 거부한다.
- Server와 Core는 같은 typed reason을 사용하고 실패는 resource/cooldown/event/revision을 바꾸지 않는다.
- Employee stat, archetype과 임시 profile로 공격/affinity/block/DefenseFocus/Rest를 보충하지 않는다.
- 안전지대에서 weapon을 장착하면 그 RON으로 eligibility/profile을 다시 계산한다. 무장 가능 직원이
  없으면 기존 `배치 가능한 직원 없음` 규칙을 적용한다.
- Condition과 weapon 사유가 겹치면 사망·Critical·Breakdown 등 기존 condition 사유가 우선하고,
  condition상 가능할 때만 `weapon_required`를 표시한다.
- 신규 run 진행을 위해 starter-eligible 6명 모두에게 `standard_training_sword`를 지급하고 validation이
  starter weapon을 요구한다. 이 훈련검은 일반 equipment reward pool에서 제외한다.

### DR-12 — Ground/Platform 전술 이동

- 별도 Ground/Platform 이동 command, 비용과 cooldown은 없다. DR-04 relocation과 DR-14 swap의 destination
  affinity 및 tile-effect 전환 규칙이다.
- `GroundOnly`/`PlatformOnly`는 해당 tile만, `Any`만 양쪽을 허용한다. Ground↔Platform은 Any weapon만
  가능하다.
- 실제 current tile kind가 Ground DefenseFocus 또는 Platform Rest 하나를 선택한다. 이전 effect 제거와
  새 effect 적용은 position mutation과 원자적이며 중간/중첩 state가 없다.
- Relocation/swap은 unit-owned state를 유지한다. Any도 Platform에서는 effective block 0이고 Ground 복귀
  때 stance/profile에 따라 block/engagement를 같은 mutation에서 재계산한다.
- Affinity 실패는 gameplay mutation 전에 무변경 거부한다. DR-04/DR-14의 비용과 cooldown을 사용하며
  DR-12 고유 tuning은 없다.
- 첫 live catalog에 Any는 없지만 current tile kind/profile selector와 Rust fixture로 양방향 engine
  계약을 완성한다. 실제 Any content는 별도 승인한다.

### DR-13 — Hard CC와 Silence capability

- 개별 command가 Stun/Freeze 이름을 나열하지 않고 active effect에서 중앙 파생한
  `hard_cc_active`/`silenced` capability만 사용한다. 같은 분류가 중첩되면 하나라도 남은 동안 유지한다.
  Hard CC의 다음 해제 시각은 남은 active Hard CC 가운데 최대 expiry가 소유한다.
- Hard CC는 새 기본 공격/automatic/manual skill 시작을 막고 적용 순간 release 전 windup/cast/focus를
  취소한다. Released attack/projectile/delayed step/area/buff와 확정 파생 effect는 계속한다. 소비 resonance와
  시작 cooldown은 환불하지 않는다.
- Hard CC 중 stance toggle, relocation/swap과 swap 양쪽 역할을 금지하고 현재 stance는 보존한다.
- Ground DefenseFocus damage reduction은 유지하지만 effective block은 0으로 만들고 engagement를 모두
  해제한다. Platform Rest, resonance gain과 field work contribution은 중단하되 cadence/assignment/progress는
  보존한다. 종료 뒤 현재 state에서 재개하고 CC 시간을 catch-up하지 않는다.
- Hard CC 자체는 battle time을 멈추지 않아 cooldown은 흐른다. 피해, 기존 buff/debuff와 passive/reactive
  Triggered effect는 유지하고 자발적 철수도 허용한다.
- Silence는 새 automatic/manual skill 시작만 막는다. 시작된 skill은 완료하며 기본 공격, block, damage
  reduction, Rest, resonance gain, field work, stance, relocation/swap과 철수를 막지 않는다.
- Silence 중 max resonance는 보존하고 종료 뒤 정상 eligibility를 평가한다.
- Hard CC 거부는 `hard_crowd_control`, Silence skill 거부는 `silenced`를 쓴다. 둘이 겹치면 Hard CC 제한이
  우선하며 별도 비용/tuning은 없다.

### DR-14 — 자리 교대

- 하나의 전술 이동 UX에서 점유 아군을 선택하면 명시적 swap preview/command로 분기한다. Server가
  relocation과 swap을 상호 fallback하지 않는다.
- 서로 다른 두 active Player employee의 logical/world position을 한 command/revision에서 원자 교환한다.
  보행, pathfinding, 중간 중복 점유와 reservation은 없다.
- 둘 다 Defense이고 employee별 relocation cooldown이 준비되어야 한다. Hard CC, unreleased action과
  이동 금지 exclusive activity는 양쪽을 막는다. Silence와 FieldWorking은 막지 않는다.
- 두 destination의 zone, weapon affinity, valid/static tile와 work eligibility를 각각 검사한다. 점유
  검사에서는 swap pair만 제외하고 방어 목표/제3 Player actor가 있으면 전체 거부한다.
- Preview에서 양쪽 새 facing을 지정한다. 기본은 각자의 기존 facing이며 자동 복사/교환하지 않는다.
- Assignment는 employee를 따라가지 않는다. 양쪽 remove/add를 final occupancy 기준으로 batch 처리한 뒤
  object interruption을 한 번 평가해 순간적인 worker 0/progress reset을 만들지 않는다.
- 양쪽 runtime identity, HP, buff/debuff, resonance, stance, 공격/skill cooldown과 unit-owned state를
  유지한다. Engagement를 먼저 해제하고 final 위치에서 block/target/range/area와 tile/work effect를
  deterministic하게 다시 계산한다. Released delivery는 계속한다.
- 성공은 전술 포인트 `5`를 command당 한 번 소비하고 양쪽 employee에 `10,000ms` relocation cooldown을
  시작한다. Relocation과 같은 policy fields를 사용하고 swap 전용 tuning을 중복하지 않는다.
- 전체 사전 검증 실패는 양쪽 position/assignment/resource/cooldown/event/revision을 바꾸지 않는다.

### DR-15 — Pause 중 명령

- Pause는 authoritative battle clock만 정지한다. Server command/query 처리는 계속하며 각 command는
  frozen latest state에서 평상시와 같은 eligibility를 검증한다.
- Gameplay mutation은 한 번에 하나씩 제출·처리한다. 앞 response와 최신 revision을 받은 뒤 다음
  mutation을 보내며 local plan/batch는 없다.
- 각 mutation은 독립 원자 transaction이다. 성공은 즉시 state/event를 commit하고 revision을 1 증가,
  실패는 완전 무변경이다. 뒤 실패가 앞 성공을 rollback하지 않으며 명시적 Resume까지 Pause를 유지한다.
- Resource, position/facing, stance, 배치/철수, cast start와 cooldown start는 frozen time에도 즉시 보인다.
  Queued simulation, 이동/공격/cast 진행, field work, HP/resonance/전술 포인트 회복, buff duration과
  cooldown 시간 경과는 정지하고 catch-up하지 않는다.
- 같은 frozen time의 순서는 server 처리 순서, battle revision과 event seq가 소유한다.
  `source_command_id`는 correlation일 뿐 ordering authority가 아니다.
- State/resync와 모든 preview/query는 최신 Pause state를 읽고 revision을 증가시키지 않는다.
- Resume와 Retreat도 같은 직렬 command다. Retreat 성공은 active battle과 Pause state를 함께 종료한다.
- Pause 중 `SetBattleSpeed`는 frozen clock을 유지하면서 Resume 후 쓸 stored speed를 즉시 변경하고
  revision을 1 증가시킨다.
- Pause 전용 runtime, client pending gameplay queue, batch, `ResumeBattleWithPlan`과 미제출 local 선택
  recovery는 없다.

### DR-16 — 수동 지정 스킬

- Effect 대상 의미는 기존 `SkillTarget`, 선택 주체는 별도
  `SkillCastSelection::Self | AutomaticUnit | PlayerUnit | PlayerTile`이 소유한다. 모든 live skill은 selection을
  명시하며 default/archetype 추론은 없다.
- Self는 caster, AutomaticUnit은 Core의 authored targeting, PlayerUnit/PlayerTile은 각각 explicit Unit/Tile
  payload를 요구한다. 누락/종류 불일치는 fallback 없이 typed reason으로 거부한다.
- Acceptance는 caster identity, manual mode, Attack stance, action/activity, Hard CC/Silence,
  resonance/cooldown과 target을 모두 먼저 검증한다. 실패는 skill/resource/resonance/cooldown/event/revision을
  바꾸지 않고 targeting focus를 유지한다.
- PlayerUnit은 target rule, active/lifecycle, side, air, usefulness와 final reach/range를 검증한다.
  PlayerTile은 valid battlefield tile과 reach/range를 검사하며 occupant를 요구하지 않는다.
- Acceptance 시 authoritative Unit identity/Tile을 캡처한다. Direct PlayerUnit effect는 실행 때 그 Unit이
  무효면 no-op하고 retarget하지 않는다. Unit 기반 TileArea는 acceptance 시 target tile, PlayerTile은
  accepted tile을 고정한다.
- Acceptance 뒤 무효화에도 retarget/refund/rollback하지 않는다. 첫 버전은
  `PlayerUnit/PlayerTile + RetargetOnStep`을 strict validation으로 금지한다.
- PlayerUnit/PlayerTile은 relocation과 공통 server-authoritative tactical focus를 사용한다. 실행 중
  `tactical_focus_speed: X0_5`, 이미 Pause면 frozen 상태를 유지한다.
- Focus는 purpose/caster/skill/restore playback만 소유한다. Hover와 미제출 target은 local preview다.
  Read-only preview는 current revision candidate를 반환하고 final ActivateSkill이 다시 원자 검증한다.
- 성공은 skill start/playback restore/focus end를 한 commit으로, 실패는 focus 포함 완전 무변경으로
  처리한다. Cancel은 비용 없이 local preview를 버리고 playback을 복원한다.
- Reconnect는 focus purpose/caster/skill/restore playback만 보존한다. Pause에서 시작한 focus는 종료 뒤에도
  Pause다. Self/AutomaticUnit은 focus 없이 즉시 command하며 수동 기본 공격은 추가하지 않는다.
- 별도 active-skill cooldown은 없다. 첫 PlayerUnit은 `fragment_freischutz_black_round`와 awakened
  `freischutz_magic_bullet` progression이며 기존 range/focus/damage/delivery 수치는 유지한다. PlayerTile
  engine/fixture는 구현하지만 첫 live PlayerTile skill은 없다.

### DR-17 — 현장 작업

- Setup에 별도 `FieldWorkObjectDef`를 둔다. Stable ID, presentation/object kind, position, 하나 이상의
  work cell, positive `required_work_ms`와 필요한 typed completion effect를 명시한다. `tactical_points`와
  `static_objects`에 상태를 덧붙이지 않는다.
- Progress interruption은 Preserve 하나뿐이다. Reset enum/분기 없이 worker 0에서도 object의
  `accumulated_work_ms`를 보존한다.
- Assign/Cancel command는 없다. Active Player unit의 authored work-cell 점유로 assignment를 자동
  시작하고 이탈/death/withdraw로 종료한다. 최초/재배치로 직접 진입하면 같은 mutation에서
  `Defense + FieldWorking`이 된다.
- Field work 비용은 없다. 진입에 쓴 배치/재배치/relocation/swap 비용과 cooldown만 적용한다.
- 모든 위치 mutation은 final occupancy에서 assignment를 원자 reconcile한다. Work cell 이탈 worker는
  일반 Defense와 tile effect로 복귀하고 진입 employee는 worker가 된다.
- FieldWorking은 새 기본 공격/skill, block, stance toggle, DefenseFocus와 Rest를 금지한다. 자발적 철수와
  relocation/swap 양쪽 역할은 허용한다.
- Worker는 계속 피해를 받는다. 적 overlap/공격은 assignment/progress를 끊지 않고 enemy occupancy는
  player work-cell slot 규칙을 바꾸지 않는다.
- 각 active eligible worker는 battle time 1ms당 object progress 1ms를 기여하고 여러 work cell은 합산한다.
  Progress는 required_work_ms에서 clamp한다.
- Hard CC는 assignment/progress를 보존하고 해당 worker contribution만 중앙 capability에서 파생 정지한다.
  Stored suspension state는 없고 종료 뒤 catch-up 없이 재개한다. Pause는 global clock 정지이며 Pause 중
  위치 mutation은 즉시 처리하되 progress는 흐르지 않는다.
- 최초로 required에 도달한 update에서 한 번만 Completed로 전환하고 work cell을 underlying tile로
  되돌린다. Assignment 종료와 unit의 일반 Defense/tile effect 복귀를 같은 mutation에서 처리한다.
- Assignment start/stop과 completion만 semantic event다. Current progress/completion/worker mapping은
  checkpoint/resync가 stable object/work-cell/unit ordering으로 소유한다.
- 승패 관여는 encounter가 명시한다. 범용 scripting 대신 첫 slice의 typed effect만 제공한다.
- 첫 live object는 `suppress_freischutz`의 1회성 `black_box_emergency_repair`, presentation
  `EmergencyRepairTerminal`, object/단일 Ground work cell `(2, 6)`, `required_work_ms: 10,000`,
  `RepairProtectedUnit { flat_hp: 100 }`이다. Full HP에서도 실제 회복 0으로 완료·소모하고 반복하지 않는다.

## Phase coverage와 canonical owner

각 정책 ID는 아래 한 행에만 나타난다. Cross-policy dependency는 구현 Phase가 참조하되 primary
implementation owner를 복제하지 않는다.

| 정책 | Primary Phase | 구현 뒤 gameplay/runtime canonical |
| --- | --- | --- |
| `DR-01` | Phase 0 | `archive/game_rulebook_legacy_2026-10-04.md` |
| `DR-02` | Phase 3 | `archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md`, `data_loading_contract.md` |
| `DR-03` | Phase 1 | `archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md` |
| `DR-04` | Phase 6 | `archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md`, server transport canonical |
| `DR-05` | Phase 6 | `archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md` |
| `DR-06` | Phase 5 | `archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md` |
| `DR-07` | Phase 5 | `core_runtime_contract.md` |
| `DR-08` | Phase 5 | `archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md`, `data_loading_contract.md` |
| `DR-09` | Phase 5 | `archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md`, `data_loading_contract.md` |
| `DR-10` | Phase 4 | `archive/game_rulebook_legacy_2026-10-04.md`, `data_loading_contract.md` |
| `DR-11` | Phase 4 | `archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md`, `data_loading_contract.md` |
| `DR-12` | Phase 6 | `archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md` |
| `DR-13` | Phase 2 | `archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md` |
| `DR-14` | Phase 7 | `archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md`, server transport canonical |
| `DR-15` | Phase 0 + every command Phase audit | `archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md`, server transport canonical |
| `DR-16` | Phase 8 | `archive/game_rulebook_legacy_2026-10-04.md`, `skill_target_contract.md`, `data_loading_contract.md`, server transport canonical |
| `DR-17` | Phase 9 | `archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md`, `data_loading_contract.md`, server transport canonical |

## 명시적 비범위

- Unity runtime/contract/UI/tests
- 임의 player pathfinding, reservation, pending movement와 swap fallback
- 전술 이동과 독립된 보행/Platform traversal
- 범용 FieldWork scripting/object framework와 추가 live 작업
- 첫 PlayerTile live skill, 수동 기본 공격과 별도 active-skill cooldown
- 승인되지 않은 save/wire/data compatibility와 schema alias
- 확정 목록 밖 gameplay, content와 balance 변경

## Compatibility와 migration 원칙

Compatibility는 승인되지 않았다. 각 Phase는 strict cutover를 사용한다. 구 payload 지원이 필요하면
구현 전에 대상 배포, 기간, 제거 조건과 전용 test를 별도로 승인받는다. 미래 Phase field를 앞 schema에
미리 넣지 않고, 같은 미배포 release의 연속 version change만 최종 shape에서 합칠 수 있다.
