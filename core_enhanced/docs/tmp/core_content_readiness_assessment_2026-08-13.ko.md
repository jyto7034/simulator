# Core 콘텐츠 제작 준비도 임시 검토

작성일: 2026-08-13
문서 상태: 임시 작업 메모
정본 여부: 비정본

이 문서는 현재 Core가 어디까지 구현됐고 콘텐츠 제작만 남은 상태인지 빠르게 판단하기 위한 임시
기록이다. Gameplay/runtime/data 정책을 새로 확정하지 않으며, 실제 권위는 runtime code, live RON과
`archive/game_rulebook_legacy_2026-10-04.md`, `core_runtime_contract.md`, `data_loading_contract.md`에 있다.

## 잠정 결론

현재 Core는 일반 콘텐츠 제작을 시작할 수 있을 만큼 넓은 수직 기반을 갖췄다. 그러나 아직
“이제부터 콘텐츠만 만들면 게임이 완성된다”는 단계는 아니다.

가장 정확한 상태는 다음과 같다.

> 콘텐츠 제작을 시작하면서 남은 공통 Core 공백과 콘텐츠 표현력의 한계를 확인하고 닫는 단계다.

기존 primitive만 사용하는 일반 조우, 스킬, 장비, 보상, 맵과 이벤트는 데이터 중심으로 제작할 수
있다. 숨겨진 보스, 새로운 전투 문법이나 현재 schema가 표현하지 못하는 고유 기믹은 추가 Core 구현이
필요할 수 있다.

## 현재 구현된 주요 기반

### 전투

- `DefenseRoute` 실시간 전투와 결정론적 simulation
- 배치, 철수, 재배치, 전술 자원, 일시정지와 배속
- 기본 공격, 수동·자동 스킬, 타겟팅과 범위 preview
- 투사체, TileArea, buff, damage, 사망과 장비 trigger
- 전투 태세, 위치 변경, 유닛 교환
- Boss Runtime v1과 하나의 live Little Red reference
- 전투 결과, 직원별 통계, condition 반영, 후퇴와 retry/failure 처리

### 런과 노드

- 고정 3-Floor Standard 튜토리얼과 Endless
- 시설 template, node assignment, Floor 생성과 navigation
- Combat, Boss, Event, Reward, Shop, SavePoint, Maintenance, HeadquartersContact
- Visual Novel과 trigger runtime
- checkpoint load, battle retreat와 run failure 사전 선택

### 성장과 경제

- 직원 roster, level/XP, physical damage와 trauma
- 장비 ownership/loadout/강화/분쇄
- 스킬 파편 획득/장착/강화/개화/분쇄
- 보상 transaction, Enkephalin, 재료, 소모품, 상점과 채용
- 도감과 영구 기록

### 장기 진행

- explicit encounter reward-source ledger
- Elite/NormalBoss/FinalBoss assignment와 cycle
- Elite 모노리스 Stage 2/3 및 숙달
- BossOmen chain/DAG/capstone
- 숙달 Elite 직접 전투/소탕과 분쇄→재획득 cycle

### 저장과 서버

- strict Active Run Core/host V4
- strict PlayerProfile V2와 bounded V1 migration
- current/backup/checksum/recovery와 Standard completion journal
- Core typed DTO/command/result와 game_server transport/persistence

### 데이터 제작 기반

현재 live RON은 직원, 환상체, 침식체, 스킬, buff, 장비, 소비 아이템, 스킬 파편, 조우, Boss Runtime,
보상, 상점, 스토리 이벤트, Visual Novel, trigger, 전장, 시설, node assignment와 RunPolicy를 다룬다.
Embedded loader와 cross-catalog validation이 ID/reference/schema 오류를 startup 전에 거부한다.

## 콘텐츠만으로 진행 가능한 범위

현재 schema와 runtime primitive 안에 머무르면 다음은 주로 RON authoring과 검증 작업이다.

- 일반 적과 환상체 actor의 stat/basic attack/기존 skill 조합
- 기존 spawn/wave 문법을 사용하는 Normal·Elite·NormalBoss·FinalBoss encounter
- 기존 skill effect/target/delivery primitive를 사용하는 스킬과 파편
- 장비, 소비 아이템, 강화·분쇄 recipe와 reward 수치
- Facility/전장/node assignment와 난이도 profile
- Event, employee check, Visual Novel과 trigger
- Reward source, repeat reward, BossOmen chain과 기존 Boss Runtime primitive 조합

새 콘텐츠가 기존 primitive로 표현되는지는 개별 콘텐츠마다 loader와 실제 전투 수직 검증으로 확인해야
한다. “RON 파일에 적을 수 있다”는 사실만으로 gameplay가 완성됐다고 보지 않는다.

## 콘텐츠 양산 전 닫아야 할 공통 Core 문제

### 1. `GrantExperience` 의미 불일치

확정 정책은 authored `GrantExperience.amount`를 대상 직원별 전액으로 정의한다. `amount: 20`이고
`AliveRoster`의 살아 있는 직원이 셋이면 각자 20 XP, 총 60 XP를 받는다. `CombatParticipants`,
`SelectedEmployee`, `AllRoster`도 각 target이 정한 대상자마다 전액을 지급하고 분배하지 않는다. 그러나
공통 `GrantExecutor::grant_experience`는 현재 대상 인원수로 amount를 나누고 remainder를 분배한다. Live
`experience_reward`도 이 경로를 사용하며 설명 역시 분배 의미로 남아 있다.

숙달 Elite 소탕과 전투 생존 XP는 직원별 `SelectedEmployee` grant로 올바르게 우회하지만, 일반 Reward와
향후 bonus XP authoring은 공통 불일치의 영향을 받는다. 대량 콘텐츠 제작 전에 runtime, live 설명과
관련 테스트를 승인 정책에 맞춰 하나로 정리해야 한다.

### 2. 숨겨진 보스 수직 경로

변질된 모노리스 획득 상태는 존재하지만 현재 consumer가 없다. 숨겨진 보스 발현, 조건 소비, 관리자
연출, 전용 capstone, 승패/Exit Gate, 보상과 도감 연결은 콘텐츠 RON만으로 완성되지 않는다. 숨겨진 보스를
현 제작 범위에 포함하면 별도 Core/live-data/server/consumer Goal이 필요하다. 제외한다면 명시적으로 후속
범위로 유지한다.

### 3. 콘텐츠 표현력 한계

- HP, 사망, wave 전멸과 지점 도달 기반 조건부 spawn은 정규 wave 계약이 아니다.
- Boss Runtime은 generic 기반이 있지만 production reference content는 하나뿐이다.
- 새 boss/fragment가 기존 effect, target query, control node와 movement primitive 밖의 동작을 요구하면
  runtime leaf/schema/validation 확장이 필요하다.
- 새로운 무기 계열, 도구 전용 시스템과 미정 gameplay는 현재 schema가 지원한다고 가정하지 않는다.

### 4. Unity 통합 미검증

현재 완료 주장은 Core/live-RON/game_server 개발 경계에 한정된다. Unity 개발 트리는 사용자 지시에 따라
검사·수정하지 않았으므로 실제 client 입력, 표시, resource와 전체 플레이 가능성을 검증한 상태가 아니다.

## 현재 placeholder와 후속 콘텐츠

- 스킬 파편 29개의 직접 `BasicAttackModifier`는 dedicated active skill 제작 전 임시 효과다.
- WhiteNight BossOmen은 generic chain 검증용 one-step bootstrap이며 최종 보스 콘텐츠가 아니다.
- Standard scripted fragment의 identity와 지급 시점은 개별 authoring이 필요하다.
- 강화/분쇄 recipe가 authoring된 live 장비 coverage는 현재 제한적이다.
- Psychology state 구조는 있지만 production mutation, 명령 거부, 전투 modifier와 dialogue cue는 없다.
- GrowthStack은 runtime 구조가 있으나 production producer가 없다.
- 최종 밸런스, pacing, tutorial polish, art/VFX/animation과 playtest는 완료 범위가 아니다.

## 현재 검증 근거

2026-08-12 Master 최종 감사 기준:

- Core: 1,074 passed; 0 failed/ignored
- game_server: 72 passed; 0 failed/ignored
- Active Run Core/host: V4
- PlayerProfile: V2
- Game Rulebook policy implementation P01~P04: 모두 `Complete / High`

이 수치는 구현된 계약의 회귀 안정성을 보여 주지만 콘텐츠 품질, Unity 통합 또는 모든 미래 기믹의
표현 가능성을 증명하지 않는다.

## 권장 다음 단계

### A. Core Closure

1. 공통 `GrantExperience`를 승인된 직원별 전액 의미로 수정하고 live 설명/테스트를 동기화한다.
2. 숨겨진 보스를 현재 milestone에 포함할지 후속으로 둘지 범위를 고정한다.
3. known placeholder와 미지원 primitive를 명시적 backlog로 분류한다.

### B. 대표 콘텐츠 수직 제작

대량 authoring 전에 다음과 같은 작은 대표 세트를 실제 품질로 제작한다.

- Normal encounter 1개
- Elite encounter 1개
- NormalBoss 또는 BossOmen/FinalBoss 1개
- dedicated fragment skill 1개
- 장비/recipe 1세트
- Event/VN 1개

각 콘텐츠를 data load, preview, 실제 전투, reward/result, checkpoint/reconnect까지 통과시키고 새 Core
primitive가 필요한지 기록한다.

### C. Schema/Primitive Freeze 후 양산

대표 세트에서 반복되는 Core 공백을 닫은 뒤 기존 primitive의 지원 범위를 명시적으로 고정하고 콘텐츠를
대량 제작한다. 새로운 고유 기믹은 콘텐츠 데이터에 억지로 우회하지 않고 별도 작은 수직 Goal로 추가한다.

## 준비도 판정

| 판정 대상 | 현재 판단 |
| --- | --- |
| Core prototype | 충분히 구현됨 |
| 일반 콘텐츠 제작 시작 | 가능 |
| 기존 primitive 기반 콘텐츠 수직 제작 | 가능 |
| 대량 콘텐츠 양산 | 공통 XP 수정과 대표 콘텐츠 검증 후 권장 |
| 독특한 보스·파편·숨겨진 보스 | 추가 Core 구현 가능성 높음 |
| Unity 포함 실제 플레이 가능 완성본 | 미검증 |
| 콘텐츠만 채우면 출시 가능 | 아님 |

## 임시 문서 종료 조건

다음 단계의 정식 Goal 또는 readiness 문서가 만들어지면 이 문서의 유효 내용을 그 산출물로 옮기고,
이 파일은 임시 참고 기록으로만 남기거나 삭제한다. 이 문서 자체를 구현 권한이나 완료 판정으로
사용하지 않는다.
