# 직원 도메인 정책 결정 및 구현 영향 기록 — 2026-07-17/18

- 최초 논의일: 2026-07-17
- 승인·구현 감사일: 2026-07-18
- 상태: historical policy decision input / implementation master complete
- 성격: 직원 도메인 Master와 네 Phase의 승인 배경, 당시 구현 상태와 난이도 근거
- 현재 disposition: [Employee Domain Implementation Master](../goals/employee_domain_implementation_master/PLAN.md)와
  [Completion Review](../goals/employee_domain_implementation_master/COMPLETION_REVIEW.md)가 완료됐으며 현재 gameplay/runtime/data 계약은 아래
  durable canonical이 소유한다.
- 주의: 이 문서는 gameplay, live RON, DTO 또는 save 계약의 현재 source of truth가 아니다.
  초기 논의 문구와 2026-07-18 승인 결정 사이의 차이는 역사적으로 보존한다. 현재 동작을 판단할
  때는 초기 추천안이나 당시 구현 감사를 사용하지 않고 runtime/live data와 durable canonical을
  확인한다.

현재 canonical ownership:

| 영역 | 현재 canonical |
| --- | --- |
| 직원 설정, 선발·채용, condition, Psychology의 플레이 규칙 | [game_rulebook.md](../game_rulebook.md) |
| 직원 identity, recruitment, condition/result, Psychology runtime 경계 | [core_runtime_contract.md](../core_runtime_contract.md) |
| 단일 `EmployeeDefinition` catalog와 strict loading | [data_loading_contract.md](../data_loading_contract.md) |
| Unity-facing 직원/후보/condition/Psychology DTO | `/mnt/f/unity projects/ark/docs/unity_core_contract.md` |
| Phase별 구현·검토 evidence | [Master PLAN](../goals/employee_domain_implementation_master/PLAN.md), [Completion Review](../goals/employee_domain_implementation_master/COMPLETION_REVIEW.md) |

범용 `specialties` production schema, trust 기반 high-risk 거부·전투 modifier·실제 대사/반응 콘텐츠,
외부 durable save migration과 account memorial/legacy는 이 Master에서 승인된 현재 계약이 아니다.
필요해지면 이 기록을 직접 구현 지시로 사용하지 않고 별도 정책 결정과 Goal을 만든다.

후속 disposition: 2026-07-19 완료된
[Event 직원 판정 주사위 정책 결정](event_employee_check_dice_policy_2026-07-17.md)은 범용
`specialties[]` 대신 Event 판정만 소비하는 단일 `EmployeeProfile.event_aptitude`를 추가했다.
따라서 이 기록의 “전문성 보류”는 범용·다중 전문성에만 적용되며 현재 Event aptitude를 미구현으로
분류하지 않는다.

## 1. 논의 목적

현재 직원과 관련된 정보는 프로필, 생존 및 컨디션, 성장, 장비, 성격 trait, 신뢰도, 기억과 반응에 걸쳐 있다. 앞으로 Event 주사위 판정을 비롯한 여러 시스템이 직원 정보를 사용하려면, 각 개념의 의미와 source of truth를 주사위 시스템보다 먼저 정리할 필요가 있다.

이 문서의 목적은 다음과 같다.

- 성격을 신뢰도의 부속 수치나 주사위 전용 태그로 만들지 않는다.
- 전문성, 성격, 신뢰도와 기억, 컨디션의 책임을 구분한다.
- 직원 정보를 소비하는 Event, 전투, 휴식, 치료, 대사 시스템이 같은 의미를 사용하게 한다.
- Core가 직원 상태와 판단의 authority를 유지한다.
- 직원 도메인에서 먼저 정해야 할 정책과 후속 소비자 정책을 분리한다.

## 2. 이 문서의 경계

직원 도메인이 소유할 후보:

- 직원의 안정적인 정체성과 프로필
- 전문성 및 성격의 authoring과 persistence
- 생존, 출전 가능 여부와 컨디션
- 신뢰도, 기억과 최근 반응
- 직원 상태를 바꾸는 중앙 정책의 책임
- 다른 시스템에 제공할 typed employee projection

직원 도메인이 직접 소유하지 않는 것:

- Event 주사위 공식과 난이도
- Event 선택지와 check tag schema
- 장비가 제공하는 Event modifier/key의 콘텐츠 정의
- Event 화면 배치와 연출
- 개별 Event의 성공·실패 결과 콘텐츠

직원 도메인은 위 시스템에 안정적인 입력을 제공할 수 있지만, 소비자의 규칙까지 대신 결정하지 않는다.

## 3. 합의된 직원 설정 정책

2026-07-17 논의에서 다음 방향을 승인했다.

### 3.1 고정 직원풀

- 기본 채용 시스템은 무작위로 새로운 일반 직원을 계속 생성하지 않는다.
- 게임은 약 12~15명의 authored 고정 직원을 기본 풀로 가진다.
- 각 직원은 고정된 definition identity, 이름, 얕은 역할·배경 설정과 성격을 가진다.
- 모든 직원에게 긴 사전 개인 서사나 전용 퀘스트를 요구하지 않는다.
- 같은 definition의 직원이 한 런에 복수의 인스턴스로 존재하지 않는다.

### 3.2 플레이 중 형성되는 개인 서사

- 직원의 긴 이야기는 사전에 모두 작성하지 않고 출전, 신뢰도, 기억, 성장, 부상, 장비와 생존 결과를 통해 플레이 중 형성한다.
- 고정된 과거는 플레이어가 직원을 구분하고 첫 선택을 내릴 수 있을 정도로 제공한다.
- 애착의 핵심 근거는 설정문 분량보다 플레이어가 해당 직원과 함께 겪은 사건의 누적과 가시화다.

### 3.3 합류와 사망

- 전체 직원을 처음부터 roster에 지급하지 않고 시작 선발과 이후 채용·구조·파견 등을 통해 순차적으로 합류시킨다.
- 아직 합류하지 않은 고정 직원은 해당 런의 유한한 예비 인력 풀을 구성한다.
- 사망한 직원은 해당 런에서 영구적으로 제거되며 다시 채용 후보로 나타나지 않는다.
- 새로운 런에서는 같은 authored 직원풀을 다시 사용하며 trust, memories, condition, growth와 loadout 같은 run-local 상태는 초기화한다.
- 사망은 큰 손실이어야 하지만 일반적인 직원 소모가 신뢰도와 기억의 축적을 무의미하게 만들 정도로 빈번해서는 안 된다.
- 시작 선발에서 선택하지 않은 직원은 영구 탈락하지 않고 미합류 채용 풀에 남는다.
- 미합류 직원 순서는 run seed로 한 번 결정한 순환 대기열이 소유한다.
- 채용 node session은 대기열에서 정해진 수의 후보를 가져와 고정하며 반복 요청으로 후보를 reroll하지 않는다.
- 선택한 직원은 roster로 이동하고 선택하지 않은 직원은 대기열 뒤로 돌아간다.
- 후보 목록과 대기열 순서는 checkpoint에서 복원한다.

직원 definition의 run-local 소유 위치는 한 시점에 정확히 하나다.

```text
RecruitmentQueue
RecruitmentNodeSession
EmployeeRoster
```

- Queue는 미합류 직원, NodeSession은 현재 제안 중인 직원, Roster는 이미 합류한 직원만 소유한다.
- `is_recruited`, `is_offered`, `is_dead` 같은 동일 사실을 여러 컴포넌트에 중복 저장하지 않는다.
- 사망은 roster 안의 직원 Condition이 소유하며 채용 시스템에 별도 Dead 상태를 복제하지 않는다.

### 3.4 Profile 표현과 gameplay 속성

- `role`과 `background`는 캐릭터 설정 및 player-facing 표현을 소유한다.
- Core는 번역되거나 변경될 수 있는 role/background 문자열에서 gameplay 의미를 추론하지 않는다.
- gameplay 능력이 필요하면 별도의 typed `specialties`를 사용한다.
- 별도의 origin 또는 직업 클래스 계층은 현재 필요한 계약이 확인되기 전까지 추가하지 않는다.

### 3.5 두 축 성격

- 성격은 `EmployeeDefinition`에 직접 작성하며 런마다 변하지 않는다.
- UUID를 이용한 성격 자동 배정은 장기 정책으로 유지하지 않는다.
- 초기 성격은 위험 대응과 사회적 대응을 분리한 두 축으로 구성한다.

```text
Personality
  risk_disposition: Brave / Cautious / SurvivalInstinct
  social_disposition: Obedient / Defiant / Comradely
```

- 성격은 행동과 반응의 기준이며 모든 상황에서 자동으로 숫자 보너스를 주는 스탯이 아니다.
- 정확한 상황별 반응과 주사위 기여 여부는 각 소비 시스템의 후속 정책에서 결정한다.

### 3.6 고정 클래스 대신 기본 전투 성향

- 탱커, 힐러, 전사, 원거리 딜러처럼 장비와 역할을 고정하는 전통적 클래스는 두지 않는다.
- 각 직원은 authored 기본 전투 성향을 가지며 성향에 따라 `base_combat_profile`의 기본 스탯 경향이 달라진다.
- 실제 전투 역할은 직원의 기본 스탯, 장비와 스킬 파편의 조합으로 결정한다.
- 전투 성향은 장비 사용을 제한하지 않는다.
- 성향과 장비가 일치하지 않는다는 이유로 별도 페널티를 적용하지 않는다.
- 성향 전용 클래스 스킬이나 장비 적합성 보너스는 현재 정책에 포함하지 않는다.
- 공식 `DefenseRoute` 전투에서 플레이어 유닛은 배치 위치에 고정되며 자력으로 이동하지 않는다.
- 따라서 직원의 기본 전투 성향과 `base_combat_profile`에는 이동형/기동형 성향이나 직원별 이동 속도 차등을 두지 않는다.
- `MoveSpeedUnitsPerMs`는 적군과 범용 전투 런타임에 필요한 이동 속도 stat으로 남길 수 있지만, 직원의 authored 기본 능력이나 성향 차이를 표현하는 축으로 사용하지 않는다.
- 강제 이동, 텔레포트, 넉백 같은 명시적 특수 효과 가능성은 기본 아군 이동 능력과 구분하며, 실제 콘텐츠가 필요할 때 별도 전투 정책으로 결정한다.
- 현재 플레이어 장비 `time_dial`의 `MoveSpeedUnitsPerMs +400`처럼 고정 배치 아군에게 의미가 없는 이동 효과는 직원 도메인 Goal에서 임의로 대체하지 않고 후속 장비/밸런스 정리 대상으로 기록한다.
- 직원 기본 프로필은 HP, 공격력, 방어력, 마법 저항력과 장비 이후에도 유지되는 공격 속도 보정을 소유한다.
- 무기는 사거리, 타일 공격 범위, 피해 속성, delivery와 무기 자체의 기본 공격 주기를 소유한다.
- 최종 전투 프로필은 무기의 기본 공격 주기에 직원의 authored 공격 속도 보정을 적용한 뒤 성장, 장비 효과와 전투 modifier를 승인된 final-stat pipeline 순서로 합성한다.
- 직원의 공격 주기를 고정 밀리초 값으로 먼저 기록한 뒤 무기 장착 시 덮어쓰는 구조는 사용하지 않는다. 어떤 무기를 장착해도 직원의 기본 전투 성향이 사라지지 않아야 한다.
- 정확한 전투 성향 종류와 기본 스탯 수치는 별도 전투 밸런스 정책에서 결정한다.

### 3.7 전문성은 문서상 확장 후보로 보류

- `Medical`, `Engineering`, `Observation` 같은 typed 전문성 개념은 장기 확장 후보로 기록한다.
- 현재 구현된 시스템에서 전문성의 명확한 gameplay 소비자는 없고, 계획 중인 Event 직원 판정만 잠재 소비자다.
- 전문성을 첫 직원 도메인 Goal의 production schema나 runtime state에 추가하지 않는다.
- role/background 문자열을 임시 전문성으로 파싱하지 않는다.
- Event 외에 치료, 정비, 연구 또는 직원 작업 배정처럼 두 번째 명확한 소비 경계가 생기면 별도 정책으로 다시 검토한다.
- 다시 도입할 때는 성격과 달리 `무엇을 할 줄 아는가`만 소유하며, 행동 의사·신뢰도·전투 클래스와 결합하지 않는다.

후속 구현 구분:

- 이 절에서 보류한 것은 여러 소비자가 공유하는 범용 `specialties[]`와 rank/growth 체계다.
- 2026-07-19 후속 Dice Goal은 이 범용 체계를 도입하지 않고 Event 전용
  `EventAptitude` 하나를 직원 Profile에 구현했다.
- 현재 Event aptitude의 gameplay/runtime/data 계약은 이 역사적 절이 아니라
  [game_rulebook.md](../game_rulebook.md), [core_runtime_contract.md](../core_runtime_contract.md),
  [data_loading_contract.md](../data_loading_contract.md)가 소유한다.

### 3.8 장기 신체 상태와 트라우마

- 기존 `EmployeeHealthState`의 Run HP는 새 장기 상태 모델과 책임이 중복되므로 제거 대상 레거시로 지정한다.
- Battle HP는 현재 전투 안에서만 존재하는 임시 체력이다.
- 전투 중 남은 Battle HP를 전투 종료 뒤 직원의 장기 신체 상태에 직접 되쓰지 않는다.
- 직원이 전투불능이 되면 신체 손상 진행도가 1 증가하고 별도 정책값만큼 트라우마가 증가한다.
- 신체 손상 진행도와 트라우마는 독립적으로 동시에 존재하고 각각 별도로 회복한다.

신체 손상 진행도:

| 진행도 | 파생 상태 |
|---:|---|
| 0 | 정상 |
| 1~2 | 경상 |
| 3~4 | 중상 |
| 5 | 치명상 |
| 6 | 사망 |

- 파생 상태 이름을 별도 mutable state로 중복 저장하지 않는다.
- 의료·치료 아이템은 authored 효과에 따라 신체 손상 진행도를 감소시킬 수 있다.
- Battle HP 회복은 신체 손상 진행도를 자동으로 감소시키지 않는다.
- 사망 뒤 일반 치료로 신체 손상 진행도를 되돌리지 않는다.
- 누적 전투불능 이력이 필요하면 치료 가능한 진행도와 별도의 통계/기록으로 보존한다.
- 기존 범용 `EmployeeInjury { id, severity }`와 `injuries[]`는 신체 손상 진행도와 역할이 중복되고 실제 효과가 없으므로 제거 대상 레거시로 지정한다.
- 출혈·감염·환상체 침식처럼 고유 효과가 실제 콘텐츠에 필요해지면 범용 injury placeholder를 보존하지 않고 별도의 typed condition 정책으로 다시 추가한다.

트라우마:

- 트라우마는 0~100의 정신적 손상이다.
- 트라우마 100은 사망이 아니라 `Breakdown`이다.
- Breakdown 직원은 일반 전투, Event 수행과 위험 행동에 참여할 수 없다.
- Breakdown 직원은 치료·안정·휴식의 대상이 될 수 있고, roster 표시와 장비 확인·해제는 유지한다.
- 트라우마가 100 아래로 감소하면 별도 mutable flag 없이 Breakdown이 해제된다.
- Breakdown의 추가 손상, 사망 연계와 위기 Event는 실제 콘텐츠와 함께 후속 정책에서 확정한다.
- 안정제, 상담, 휴식 등 승인된 효과가 트라우마를 감소시킬 수 있다.
- 신체 치료와 트라우마 치료는 기본적으로 분리하며, 둘 다 회복하는 희귀 효과는 명시적으로 authoring한다.

전투 시작 Battle HP:

```text
Battle Max HP
  = 직원 base combat profile + 장비 + 스킬 파편 등 승인된 전투 modifier

Starting Battle HP
  = Battle Max HP × (100 - Trauma) / 100
```

- 트라우마는 Battle Max HP가 아니라 전투 시작 current HP를 감소시킨다.
- 트라우마가 100 미만이고 다른 eligibility를 만족하면 최소 1 Battle HP를 보장한다.
- 트라우마 100은 Breakdown으로 출전 불가이므로 Battle HP를 생성하지 않는다.
- 신체 손상 진행도는 초기 정책에서 Starting Battle HP를 추가로 감소시키지 않는다.
- exact rounding과 전투불능 시 트라우마 증가량은 구현 전 밸런스 정책에서 확정한다.

### 3.9 목적별 eligibility와 범용 availability 제거

- 기존 `EmployeeAvailability::Available/Unavailable`은 이유 없는 중복 상태이므로 제거 대상 레거시로 지정한다.
- 직원 도메인은 하나의 범용 availability를 저장하지 않고 실제 Condition에서 목적별 eligibility를 계산한다.
- 전투 배치, Event 판정, 아이템 대상과 loadout 편집은 서로 다른 activity context를 사용한다.
- 중앙 eligibility evaluator는 boolean뿐 아니라 typed disabled reason을 반환한다.
- snapshot projection과 command validation은 같은 evaluator를 사용한다.
- Unity와 개별 world flow는 life state, 신체 손상과 trauma를 임의로 재조합해 eligibility를 추론하지 않는다.
- 필요하면 `is_available_for_combat()` 같은 편의 함수는 중앙 evaluator를 호출하는 얇은 wrapper로만 유지한다.
- 회복 중이나 스토리상 부재 같은 실제 기능이 생기면 이유가 명시된 typed restriction을 추가하되, 현재 사용처가 없는 restriction을 미리 만들지 않는다.

### 3.10 사망한 직원의 자산 처리

- 사망한 `EmployeeRunState`는 roster에 남기며 신뢰도, 기억, 성장과 출전 기록을 보존한다.
- 일반 사망에서는 장착 장비와 공유 스킬 파편을 자동 해제해 공유 inventory로 회수한다.
- 이미 소비된 아이템과 활성 소비 효과는 반환하지 않는다.
- 자산 회수 가능 여부는 Event 전용 규칙이 아니라 모든 authoritative 사망 원인이 제공할 수 있는 일반 사망 결과다.
- 기본 결과는 회수 가능이며, 사망 원인이 명시적으로 회수 불가를 지정한 경우에만 장비와 공유 파편을 상실한다.
- Event ID, node category 또는 설명 문자열에서 회수 가능 여부를 추론하지 않는다.
- 직원의 사망 상태와 자산 회수 결과는 중앙 사망 처리 경로에서 원자적으로 반영한다.
- 부분 회수와 실제 회수 불가 콘텐츠는 필요할 때 후속 정책으로 확장한다.

### 3.11 신뢰도 공식 활성화와 세부 효과 보류

- 신뢰도는 기억, 대사, 반응과 고위험 명령 판단에 사용하는 공식 직원 도메인 기능이다.
- 명시적인 고위험 행동에서는 신뢰도에 따라 경고 또는 거부가 발생할 수 있다.
- 어떤 콘텐츠가 고위험인지, 경고·거부 threshold와 성격별 차이는 실제 콘텐츠와 함께 후속 정책에서 확정한다.
- 일반 행동을 신뢰도로 빈번하게 차단하지 않는다.
- 신뢰도는 Event 주사위 수치에 직접 영향을 주지 않는다.
- 신뢰도에 따른 전투 또는 기타 추가 stat/effect 가능성은 문서상 확장 방향으로만 열어둔다.
- exact stat, 수치, 발동 조건과 지속시간은 현재 구현하지 않는다.
- 사용되지 않는 기존 modifier/feature flag 타입을 미래 가능성만으로 최종 계약처럼 보존하지 않고 실제 콘텐츠 도입 시 승인된 typed 효과로 추가한다.

### 3.12 Compatibility 없이 새 계약으로 전면 교체

- 기존 employee, candidate, save와 Unity payload schema의 compatibility를 지원하지 않는다.
- dual-read, dual-write, fallback, schema alias와 임시 기본값을 추가하지 않는다.
- UUID 기반 trait, Run HP, 범용 availability와 범용 injuries를 새 계약과 병행 유지하지 않는다.
- live employee data, Core aggregate, checkpoint state, DTO, server mapping, Unity consumer, fixture와 test를 같은 승인 계약으로 일괄 migration한다.
- 구형 payload 또는 데이터가 새 필수 계약을 만족하지 않으면 조용히 보정하지 않고 명확히 거부한다.

비규범적 책임 예시:

```text
EmployeeDefinition
  employee_definition_id
  authored identity/profile
  personality
  combat_aptitude/base_combat_profile

EmployeeRunState
  employee_definition_id
  run instance UUID
  physical damage/trauma/growth/loadout
  trust/memories/reactions

RecruitmentQueue
  ordered not-joined employee_definition_ids

RecruitmentNodeSession
  currently offered employee_definition_ids
```

위 이름과 serialization shape는 아직 확정된 구현 계약이 아니다.

## 4. 현재 확인된 상태

### 4.1 직원 aggregate

현재 직원은 장기 운용 대상으로서 다음 종류의 상태를 가진다.

- 레벨과 경험치
- 생존 및 이용 가능 상태
- HP, 트라우마와 부상
- loadout
- skill fragments와 combat profile
- `EmployeeTrustState`

`docs/game_rulebook.md`도 직원을 장비, 스킬 파편, 신뢰도, HP와 트라우마를 통해 장기 관리하는 대상으로 정의한다.

### 4.2 후보 프로필과 전문성

- 현재 candidate에는 표시용 `role`과 `background`가 있다.
- 이 문자열을 runtime gameplay 규칙에서 파싱해 전문성으로 사용하지 않는다.
- 현재 논의된 의미의 typed specialization은 아직 canonical 직원 속성으로 확정되지 않았다.

### 4.3 성격 trait

- 현재 trait 후보는 `Brave`, `Cautious`, `Obedient`, `Defiant`, `Comradely`, `SurvivalInstinct`다.
- 현재 trait 배정은 authored employee identity라기보다 UUID에 기반한 결정적 배정이다.
- 이 방식을 장기 gameplay authority로 유지할지는 결정되지 않았다.
- 현재 trait가 신뢰도 상태 안에 있다는 사실만으로 성격의 의미까지 신뢰도가 소유한다고 간주하지 않는다.

### 4.4 신뢰도와 기억

- 현재 `EmployeeTrustState`는 score, traits, memories와 recent reactions를 포함한다.
- 현재 기본 정책은 `narrative_only`이며 memory/dialogue 중심 기능만 활성화한다.
- high-risk refusal, trauma, pre-battle, ally-death와 combat modifier 계열 기능은 공식 활성화되지 않았다.
- 신뢰도는 전투 수치보다 기억, 대사, 판단과 반응의 기준이라는 방향이 `docs/game_rulebook.md`에 기록되어 있다.
- 기존 정책 기록은 dormant 기능을 당장 제거하지 않되, 공식 활성화 때 별도 정책 gate를 정하도록 한다.
- Core가 dialogue cue를 생성할 수 있지만 현재 snapshot이 실제 cue payload 대신 count만 보존하는 경로가 있어 소비 계약은 완결되지 않았다.

## 5. 합의된 방향

### 5.1 개념의 의미를 분리한다

- 성격은 직원이 어떤 방식으로 행동하고 반응하는지를 나타내는 비교적 안정적인 정체성이다.
- 신뢰도는 직원이 관리자를 얼마나 믿고 명령을 받아들이는지를 나타내며 플레이에 따라 변하는 관계 상태다.
- 기억은 직원이 겪은 사건과 그 맥락을 나타낸다.
- 컨디션은 지금 행동할 수 있는지를 나타낸다.
- 성격을 신뢰도 score에 흡수하지 않는다.
- 신뢰도와 기억은 Event 주사위의 범용 numeric modifier로 직접 사용하지 않는다.

### 5.2 성격은 주사위 전용 속성이 아니다

성격을 정식 직원 속성으로 채택한다면 Event 판정만을 위해 존재해서는 안 된다. 대사와 위험 행동 반응 등 여러 직원 경험에서 같은 의미로 참조할 수 있어야 한다.

성격이 영향을 줄 수 있는 후보 영역:

- 상황별 대사와 반응 방식
- 위험 행동에 대한 경고, 수락 또는 거부
- 신뢰 변화에 대한 반응 차이
- 치료, 휴식, 강제 출전과 동료 사망 반응
- Event에서 특정 접근 방식과의 적합성

위 목록은 활용 가능성에 대한 방향이며 정확한 규칙이 승인된 것은 아니다.

### 5.3 직원 상태의 authority는 Core에 둔다

- Unity가 trait 이름이나 표시용 문자열로 직원 규칙을 재추론하지 않는다.
- 직원의 현재 상태, 적격성, 관계 판단과 상태 mutation은 Core가 소유한다.
- 다른 시스템은 typed 입력 또는 projection을 통해 직원 정보를 소비한다.
- Event나 Support 코드가 신뢰도 점수를 직접 계산하지 않고, 직원 도메인의 중앙 정책을 통해 typed 사건을 적용하는 구조를 우선 검토한다.

마지막 항목의 exact resolver와 event schema는 아직 추천안이며 확정 계약이 아니다.

## 6. 추천하는 책임 구조

다음은 책임을 설명하기 위한 비규범적 구조다. 실제 Rust 타입이나 serialization shape를 확정하지 않는다.

```text
Employee
  Profile
    identity
    role/background
    specialties? (deferred)
    personality
    combat aptitude/base combat profile

  Condition
    life state / explicit restrictions
    physical damage progress
    trauma/breakdown
    special injuries?

  Psychology
    trust
    memories
    recent reactions
```

### Profile

직원 생성 뒤에도 유지되는 안정적인 정체성과 능력 정보를 소유한다.

- role/background는 캐릭터 설정과 표시를 소유하고 gameplay 전문성은 typed specialties로 분리한다.
- specialization은 문서상 확장 후보이며 현재 production 구현 범위에서는 제외한다.
- personality는 authored 두 축 성격으로 보존한다.
- combat aptitude는 고정 클래스로 사용하지 않고 기본 전투 스탯 경향을 설명한다.

### Condition

생존 여부, 신체 손상, 트라우마와 현재 행동 가능 여부를 소유한다. Run HP는 목표 구조에 포함하지 않는다.

- life state, 파생 신체 상태, trauma/breakdown과 특수 injury 사이의 정확한 목적별 적격성 규칙은 일부 미결정이다.
- 컨디션을 각 소비자가 임의로 조합하지 않고 직원 도메인이 목적별 적격성을 투영하는 방식을 우선 검토한다.

### Psychology

신뢰도, 기억과 최근 반응처럼 플레이 과정에서 변하는 심리·관계 상태를 소유한다.

- personality는 Psychology가 아니라 Profile 쪽의 안정적인 속성으로 보는 방향이다.
- Psychology가 personality를 참조해 반응을 계산할 수는 있지만 둘을 같은 상태로 취급하지 않는다.

## 7. 다른 시스템에 제공할 경계

직원 도메인은 소비자가 내부 필드를 재해석하지 않도록 목적에 맞는 typed 결과를 제공하는 방향을 검토한다.

비규범적 예시:

```text
EmployeeActionEligibility
  employee_uuid
  eligible
  reason?

EmployeeProfileProjection
  specialties?
  personality?

EmployeeRelationshipDecision
  accepted / warned / refused
  reaction_cues?
```

- 주사위 시스템은 판정 가능 여부와 승인된 판정 특성만 소비한다.
- Event UI는 Core가 투영한 결과를 표시하고 내부 trait 규칙을 재계산하지 않는다.
- 장비의 Event 효과는 장비 도메인이 소유하고, 직원 도메인은 현재 장착 상태의 authority만 제공한다.
- 신뢰도 또는 기억에 영향을 주는 사건은 직원 도메인의 승인된 mutation 경로를 사용한다.

## 8. 남은 후속 정책과 구현 시 확정할 세부사항

### Profile 경계

1. 고정 직원 definition과 run-local 직원 상태의 exact Rust/live RON/DTO shape.
2. 두 축 personality로 기존 12~15명 직원 콘텐츠를 실제 authoring하는 작업.
3. 이동형 성향과 이동 속도 차등을 제외한 기본 전투 성향 종류와 각 직원의 base combat stat 수치.

### Condition 경계

4. 전투불능 시 트라우마 증가량과 Starting Battle HP rounding.
5. 신체 손상 치료와 트라우마 회복 효과의 exact 수치.
6. Breakdown의 추가 패널티, 사망 연계 또는 위기 콘텐츠.
7. 실제 고유 효과가 있는 특수 condition 콘텐츠.
8. 실제 자산 회수 불가 원인과 부분 회수 콘텐츠.

### Psychology 경계

9. 실제 고위험 행동, 신뢰도 경고·거부 threshold와 성격별 반응 차이.
10. 신뢰도에 따른 추가 stat/effect의 실제 콘텐츠.
11. 기억으로 남겨야 하는 사건의 범위와 identity/dedupe/expiry 정책.
12. dialogue/reaction cue를 Core DTO로 실제 전달할지.
13. 사망한 직원의 기억과 기록을 해당 런에서 어떻게 추모·회고할지.

### 계약과 migration

14. Core aggregate와 Unity projection의 exact field naming 및 serialization shape.
15. run-local 상태 초기화와 별개로 계정 단위 추모·legacy 기록을 둘지.

## 9. 주사위 시스템과의 관계

직원 도메인을 먼저 정리하는 이유는 주사위 시스템이 성격과 전문성의 의미를 소유하지 않게 하기 위해서다.

직원 도메인이 먼저 제공해야 할 최소 결과:

- 안정적인 직원 identity
- Event 수행 가능 여부를 판단할 수 있는 authoritative 경로
- 판정에 사용하도록 승인된 typed Profile 특성
- 현재 장착 장비를 조회할 authoritative 경로
- 필요한 경우 행동 의사에 대한 trust/memory 판단 결과

주사위 시스템에서 별도로 결정할 내용:

- 어떤 직원 특성을 어떤 check tag와 비교하는지
- modifier 공식, cap과 natural 1/20 우선순위
- 장비 modifier와 choice key 평가
- roll commit, 중복 명령과 reconnect 처리
- Event DTO와 Unity presentation

## 10. 권장 Goal 순서

현재 논의 기준 권장 순서는 다음과 같다.

1. 직원 도메인의 source of truth와 소비 경계를 정리한다.
2. 정리된 직원 계약을 입력으로 Dice System 계약을 구현한다.
3. 실제 employee-check Event 콘텐츠를 authoring한다.
4. Event 화면과 판정 연출을 완성한다.

다만 1번을 하나의 거대한 Goal로 확정하지는 않는다. 조사 결과 다음 경계가 독립적으로 구현·검증 가능하다면 별도 Goal로 분리해야 한다.

- Profile의 personality/specialization authority와 migration
- Psychology의 trust/memory 활성화 및 reaction 계약
- Condition의 목적별 eligibility 계약

Dice System에 직접 필요한 최소 선행 범위는 Profile의 판정 특성과 Event 수행 eligibility다. trust/memory 공식 활성화가 독립적이라면 Dice System의 선행 Goal에 억지로 포함하지 않는다.

## 11. 현재 비범위

- 주사위 수치와 Event 난이도 확정
- 구체적인 장비 Event 효과 작성
- 최종 직원 UI와 Event UI 제작
- 모든 trust dormant 기능의 일괄 활성화
- 전투 modifier로 trust를 사용하는 정책
- 모든 기존 employee/save data의 migration 구현
- 구체적인 성격별 대사와 콘텐츠 작성
- 고정 배치 아군에게 무효인 기존 장비 이동 속도 효과의 제거 또는 대체 구현

## 12. 관련 근거

- `docs/game_rulebook.md`: 직원 장기 관리와 신뢰도 정책
- `docs/goals/core_component_refactor_master/employee_growth_trust_refactor.md`: 현재 직원/성장/신뢰도 runtime 감사
- `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`: dormant trust 기능과 공식 활성화 보류 기록
- `docs/decisions/event_employee_check_dice_policy_2026-07-17.md`: 이 직원 계약을 소비해 구현 완료된 후속 주사위 정책 결정

## 13. 실제 구현 난이도 감사

이 절은 2026-07-18 현재 dirty worktree의 실제 Core, live RON, game_server와 접근 가능한 Unity 소비자를 읽은 결과다. 정책의 최종 source of truth가 아니라 후속 구현 Goal을 안전하게 나누기 위한 영향도 기록이다.

평가는 이 문서의 초기 초안 문구보다 이후 사용자 논의에서 승인된 목표 정책을 기준으로 한다. 특히 다음 최신 결정을 전제로 한다.

- Run HP와 범용 injury/availability를 제거하고 신체 손상도를 사용한다.
- 신체 손상 파생 구간은 정상 0, 경상 1~3, 중상 4, 치명상 5, 사망 6이다.
- trauma 100은 사망이 아닌 Breakdown이다.
- 모든 직원을 하나의 authored definition catalog로 통합한다.
- `CombatAptitude`는 `Balanced`, `Defensive`, `Offensive` 기반만 먼저 제공한다.
- 전문성은 문서상 확장 후보로만 유지하고 production 계약에는 추가하지 않는다.
- trust high-risk/refusal/modifier와 실제 reaction content는 콘텐츠가 생길 때까지 보류한다.

### 13.1 난이도 기준

| 등급 | 판단 기준 |
|---|---|
| 해당 없음 | production 구현을 하지 않거나 문서상 비범위로만 유지한다. |
| 낮음 | 현재 구조가 이미 정책을 만족하며 국소 필드·validation·test 정리만 필요하다. |
| 중간 | 하나의 주 소유 계층 안에서 schema 또는 runtime을 바꾸고 제한된 DTO/test를 갱신한다. |
| 높음 | live data, runtime state, DTO 또는 Unity 소비자 중 둘 이상을 동시에 migration하거나 stat/checkpoint 계약을 바꾼다. |
| 매우 높음 | 전투 transaction, rollback/checkpoint, 여러 gameplay 소비자와 cross-layer wire를 compatibility 없이 한 번에 전환해야 한다. |

파일 수만으로 난이도를 정하지 않는다. 권위 상태 교체, 원자성, rollback, producer-consumer 동시 cutover와 기존 테스트가 보호하는 사용자-visible 계약을 기준으로 한다.

### 13.2 현재 구현 구조 요약

- 직원 static data는 `starter_candidates.ron`과 `recruitment_candidates.ron`, 그리고 두 DB로 분리돼 있다. 두 catalog 사이 definition ID uniqueness는 검증하지 않는다.
- runtime `Employee`는 run UUID와 name만 보존하고 authored definition ID, role/background, personality와 aptitude를 잃는다.
- 시작 선발에서 선택하지 않은 후보는 run state에 남지 않는다.
- HQ 후보는 `src/game/world/map_content.rs`에서 node seed로 recruitment catalog 전체를 매번 shuffle/truncate한다. run-wide queue, 유한 소유권과 이미 채용된 definition 검증은 없다.
- 기본 직원 combat profile은 전 직원 공통이다. `UnitCombatProfile::apply_weapon_profile`은 weapon interval을 `basic_attack`과 `stats`에 직접 덮어쓴다.
- personality 후보는 `EmployeeTrustState::new_for_employee`가 run UUID modulo로 단일 trait를 만든다.
- persistent condition은 `life_state + availability + Run HP + trauma + injuries[]`다. trauma 100 또는 Run HP 0이 permanent Dead 전이를 만든다.
- active battle은 entry roster clone과 RuntimeUnit-instance processed ledger를 이미 소유한다. 전투불능 상태는 attempt roster에 즉시 반영되고 result에서 원자적으로 persistent roster에 commit되며 retreat는 entry snapshot을 복원한다.
- Event, loadout, consumable과 battle deployment가 서로 다른 life/availability predicate를 사용한다. 하나의 목적별 eligibility evaluator는 없다.
- 실제 recovery capability는 recruitment뿐이다. Treatment enum은 있으나 live node/effect 경로가 없다.
- 사망과 장비 inventory reverse reference, employee item slot, skill-fragment loadout을 함께 정리하는 중앙 death transaction은 없다.
- Run checkpoint는 별도 디스크 save schema가 아니라 Core 메모리 clone payload다. queue를 추가하려면 payload와 Floor carry-forward를 명시 갱신해야 한다.
- game_server는 shared Core DTO를 deserialize/serialize하는 얇은 transport다. 직원 의미를 별도로 계산하지 않는다.
- Unity는 `RosterModel`과 여러 composer에서 `life_state`, `health`, `trauma`, `available_for_combat`를 직접 읽는다. 일부 contract test는 unknown life state와 legacy health fallback도 고정한다.

### 13.3 정책별 실제 난이도

| 정책 | 난이도 | 실제 구현 근거와 주된 비용 | 권장 Goal |
|---|---|---|---|
| 단일 `EmployeeDefinition` catalog | 높음 | 두 RON/DB, embedded bundle, loader/validation, starter/HQ DTO와 Unity candidate reader를 한 schema로 교체해야 한다. | A |
| definition ID와 run UUID 분리 | 높음 | roster가 definition ID를 잃고 battle/reward/loadout은 run UUID를 사용한다. aggregate, snapshot, result와 fixture를 함께 갱신해야 한다. | A |
| 두 축 authored personality | 중간 | UUID trait 생성을 제거하고 12명 live definition, profile DTO와 Unity 표시 계약을 추가한다. gameplay 계산은 아직 없다. | A |
| `CombatAptitude` 3종 기반 | 중간 | definition schema/validation/projection이 필요하지만 현재는 stat 계산이나 장비 제한을 바꾸지 않는다. | A |
| 직원별 base combat stats | 높음 | 현재 공통 `employee_default`를 live-authored profile로 교체하고 draft/stat snapshot 및 strict validation을 갱신해야 한다. | A |
| `basic_attack_speed_percent` | 높음 | weapon이 interval을 덮어쓰는 현재 순서에 새 명시 pipeline 단계를 넣고 rounding, equipment modifier 합성, battle validation/test를 고정해야 한다. | A |
| 전문성 미구현 유지 | 해당 없음 | 현재 production typed specialty가 없으므로 새 필드를 만들지 않는 것이 완료다. role/background 파싱이 없는지만 감사한다. | A의 negative validation/audit |
| RecruitmentQueue 단일 소유권 | 높음 | node-seeded 재추첨을 run-seeded persistent queue로 바꾸고 Queue→Session→Roster 이동과 비채용 후보 반환을 원자화해야 한다. | B |
| 채용 definition 중복 금지 | 높음 | roster에 definition ID가 생긴 뒤 queue/session/roster 전체 invariant와 invalid live/checkpoint state rejection이 필요하다. | B |
| 채용 session의 reconnect/checkpoint 복원 | 높음 | 현재 active session은 world state clone에는 들어가지만 SavePoint payload에는 queue가 없다. checkpoint, Floor carry-forward와 HQ 다른 선택 분기를 함께 검증해야 한다. | B |
| Run HP 제거 | 매우 높음 | battle-start formula, combat outcome diff, admin, consumable schema, canonical 문서, Core/Unity snapshot과 다수 테스트에 직접 노출된다. | C |
| 범용 injury/availability 제거 | 매우 높음 | Employee aggregate, Event/loadout/item/battle/reward predicate와 wire field를 동시에 교체해야 한다. | C |
| 신체 손상 0~6와 파생 상태 | 매우 높음 | permanent death, deployability, result projection, recovery와 death-prevent 의미의 새 source of truth가 된다. 중복 life state를 남기지 않는 cutover가 필요하다. | C |
| trauma 100 Breakdown | 높음 | 현재 trauma 100은 Dead다. support 회복, battle draft, Event/loadout eligibility와 Unity 상태 표시를 분리해야 한다. | C |
| 전투불능 instance마다 손상 +1 | 높음 | 기존 instance ledger와 attempt roster를 재사용할 수 있지만 mutation/result diff/redeploy candidate/death transaction을 새 condition에 맞춰 바꿔야 한다. | C |
| data-driven 전투불능 trauma | 낮음 | `run/policy.ron`의 `incapacitation_trauma`와 validation이 이미 존재한다. Run HP loss 정책 제거와 새 이름 정리가 남는다. | C |
| 사망 방지: 손상 6을 5로 제한 | 중간 | 기존 `DeathPrevent` 소비 경로는 재사용 가능하지만 trauma 99/Run HP 1 복구를 제거하고 physical damage transition에 연결해야 한다. | C |
| 새 Starting Battle HP 공식 | 중간 | 계산 helper와 stat-pipeline/current-HP ratio tests를 교체한다. final max HP 이후 적용 순서 검증이 핵심이다. | C |
| 목적별 중앙 eligibility와 typed reason | 매우 높음 | Event, loadout, consumable, deployment, recovery가 분산 boolean/filter를 사용하고 Unity도 일부 affordance를 추론한다. 모든 projection과 command preflight를 같은 evaluator로 전환해야 한다. | C |
| 정수 physical Treatment effect | 높음 | effect schema/validation, preview/result, target eligibility와 실제 recovery capability가 필요하다. 현재 live Treatment content는 없다. | C |
| Rest/SavePoint/Gate trauma-only 유지 | 낮음 | 기존 support 함수가 이미 trauma만 감소시킨다. life-state filter를 새 condition evaluator로 바꾸는 정도다. | C |
| 일반 사망의 장비·공유 파편 회수 | 높음 | employee slot, inventory `equipped_to`, fragment loadout과 condition transition을 staged result state에서 원자적으로 갱신하는 새 중앙 death resolver가 필요하다. | C |
| 사망 EmployeeRunState 기록 유지 | 중간 | roster는 현재 dead employee를 삭제하지 않는다. definition identity, 새 projection과 모든 action exclusion을 보강해야 한다. | C |
| trust trait 제거와 profile personality 참조 | 중간 | trust state/snapshot/Unity test에서 traits를 제거한다. authored personality 자체는 A가 소유한다. | D |
| trust score `-100..100`, 초기 0과 band | 낮음 | 현재 runtime이 이미 같은 범위, 초기값과 band를 사용한다. contract/DTO 정리만 필요하다. | D |
| 의미 단위 memory, subject/count/first/last/expiry | 높음 | 현재 memory는 kind/intensity/remaining_nodes 배열 append뿐이다. identity/dedupe/merge, checkpoint clone, DTO/Unity reader와 resolver tests를 새로 설계해야 한다. | D |
| high-risk/refusal/modifier/reaction content 보류 | 해당 없음 | dormant 내부 타입을 새 canonical DTO/save 계약으로 승격하지 않고 실제 콘텐츠를 추가하지 않는 것이 완료다. | D의 negative audit |
| compatibility 없는 cutover | 독립 난이도 없음 | 별도 기능이 아니라 A~D 모두에서 dual schema/alias/fallback을 금지해 각 Goal 난이도를 높이는 공통 제약이다. Unity legacy fallback test도 해당 Goal에서 제거한다. | A~D 공통 |

## 14. 권장 후속 Goal 묶음

### Goal A — `employee_definition_profile_contract`

난이도: **높음**

하나로 묶을 범위:

- 단일 EmployeeDefinition live catalog와 strict validation
- stable definition ID와 run employee UUID
- role/background, starter eligibility와 starter loadout
- 두 축 personality
- `CombatAptitude`
- 직원별 base combat stats
- `basic_attack_speed_percent`와 final-stat pipeline
- starter/profile/roster DTO 및 접근 가능한 Unity reader migration
- 기존 starter/recruitment candidate DB 제거에 필요한 catalog 측 정리

묶는 이유:

모든 항목이 하나의 authored definition을 생산하고 roster/battle이 같은 definition을 참조하게 만드는 Profile source-of-truth 경계다. personality나 combat stats를 별도 Goal로 떼면 live employee schema와 Unity candidate/profile DTO를 반복 migration해야 한다.

사용자-visible 완료 조건 후보:

- 시작 후보, 채용 후보와 roster가 같은 definition identity/name/profile을 표시한다.
- 각 직원은 authored personality와 aptitude를 가지며 weapon을 바꿔도 직원 공격 속도 성향이 보존된다.
- UUID, 이름 또는 role 문자열에서 profile 의미를 추론하지 않는다.

### Goal B — `employee_recruitment_queue_ownership`

난이도: **높음**

하나로 묶을 범위:

- run-seeded RecruitmentQueue
- 시작 미선발 definition의 queue 편입
- Queue/NodeSession/Roster 단일 소유권
- 후보 session 고정, 비선택/비채용 후보의 순서 보존 반환
- definition 중복 채용 금지
- checkpoint, reconnect, Floor carry-forward
- HQ command/result/snapshot과 Unity recruitment consumer

묶는 이유:

Profile catalog를 소비하지만 별도의 run ownership state machine이다. Goal A와 합치면 live data/profile migration과 map/HQ/checkpoint transaction이 한 Goal에 과도하게 겹친다. 반대로 queue, session, dedupe와 checkpoint를 서로 나누면 동일 definition이 두 위치에 존재하는 중간 상태가 생긴다.

사용자-visible 완료 조건 후보:

- 같은 run/seed/checkpoint에서 후보 순서가 안정적이고 반복 요청으로 reroll되지 않는다.
- 선택하지 않은 후보가 순환하며 같은 definition을 두 번 채용할 수 없다.
- 다른 HQ 행동과 checkpoint load 뒤에도 queue 소유권이 정확히 복원된다.

### Goal C — `employee_condition_lifecycle_migration`

난이도: **매우 높음**

하나로 묶을 범위:

- Run HP, generic injury, mutable availability/life-state 중복 제거
- physical damage/trauma/Breakdown source of truth
- Starting Battle HP
- instance-idempotent incapacitation과 same-attempt redeploy
- death prevent와 central death/asset recovery transaction
- physical Treatment와 기존 trauma recovery
- 목적별 central eligibility, typed reasons와 recovery/failure 판단
- battle result diff, retreat, checkpoint, admin와 consumable migration
- Core DTO, server serialization, Unity roster/loadout/item/support/battle/maintenance consumer 및 tests

묶는 이유:

이 항목들은 모두 `EmployeeCondition`이 어떤 활동을 허용하고 전투불능·사망·치료·rollback에서 어떻게 바뀌는지 정의한다. 기존 Run HP를 제거한 뒤 eligibility나 death asset transaction을 후속 Goal로 미루면 새 condition과 구형 boolean/자산 상태가 동시에 권위를 갖는다. 범위는 크지만 하나의 완결된 lifecycle cutover로 다뤄야 한다.

사용자-visible 완료 조건 후보:

- physical damage와 trauma가 승인된 구간과 eligibility를 일관되게 표시·검증한다.
- 반복 전투불능, 재배치, 결과 commit과 retreat가 같은 working condition을 사용한다.
- 사망 방지, 사망 자산 회수, 치료와 no-deployable recovery가 부분 적용 없이 동작한다.
- Unity가 HP/legacy availability를 재추론하지 않고 Core typed eligibility를 따른다.

### Goal D — `employee_psychology_state_contract`

난이도: **높음**

하나로 묶을 범위:

- trust trait/UUID personality 제거와 Goal A profile 참조
- score/band persistence
- 의미 단위 memory identity, merge/dedupe와 expiry
- recent reaction과 durable memory 분리
- death record/checkpoint/snapshot/Unity psychology reader
- dormant high-risk/refusal/combat modifier/cue content의 비활성·비계약 상태 감사

묶는 이유:

Profile personality를 소비하지만 trust와 memory는 독립적인 run-local Psychology aggregate다. 실제 high-risk/dialogue 콘텐츠와 분리해도 score/memory persistence와 wire 계약은 독립적으로 검증할 수 있다.

사용자-visible 완료 조건 후보:

- 직원 성격은 definition에서, 신뢰도와 기억은 run-local Psychology에서 일관되게 보인다.
- 같은 authoritative 사건의 중복 command/reconnect가 기억을 중복 생성하지 않는다.
- 반복 사건은 승인된 의미 key로 병합되고 사망 직원의 run 기록에도 남는다.
- 보류한 기능이 활성 gameplay나 public 계약처럼 노출되지 않는다.

## 15. 권장 수행 순서와 의존성

```text
Goal A: EmployeeDefinition/Profile
  -> Goal B: RecruitmentQueue ownership
  -> Goal C: Condition lifecycle migration
  -> Goal D: Psychology state contract
```

1. **Goal A를 먼저 수행한다.** B는 definition identity가 없으면 queue dedupe를 구현할 수 없고, D는 authored personality source가 필요하다. C도 최종 EmployeeRunState가 어떤 definition을 가리키는지 먼저 고정하면 DTO와 test fixture를 다시 바꾸는 비용이 줄어든다.
2. **Goal B를 다음에 수행한다.** A의 catalog를 run ownership으로 연결하는 비교적 독립적인 flow이며, 매우 높은 위험의 C 전에 checkpoint/identity 기반을 안정화할 수 있다.
3. **Goal C를 세 번째로 수행한다.** 가장 넓고 위험한 migration이다. A/B가 안정된 뒤 EmployeeRunState와 checkpoint를 한 번에 condition 기준으로 전환한다.
4. **Goal D를 마지막에 수행한다.** A의 personality를 참조하고 C의 death/checkpoint lifecycle 위에서 memory persistence를 고정한다. 보류된 콘텐츠는 포함하지 않는다.

Dice System과 employee-check Event 후속 구현은 최소한 Goal A와 Goal C가 완료된 뒤 시작할 수 있다. Goal D의 high-risk/dialogue 콘텐츠는 Dice System의 필수 선행 조건이 아니다.

## 16. 검증 비용과 주요 위험

후속 Goal의 focused 검증 후보:

- A: employee data strict loading, profile/stat pipeline, starter snapshot, 실제 Rust JSON을 읽는 Unity starter/roster contract tests
- B: queue ownership unit test, HQ session/reconnect/checkpoint/Floor integration, 중복 채용 negative test, Unity recruitment command/render tests
- C: employee condition unit test, repeated incapacitation/redeploy/result/retreat integration, death asset atomicity, recovery restriction, live RON negative validation, server DTO와 Unity roster/loadout/item/battle tests
- D: trust score/band unit test, semantic memory merge/dedupe/expiry, checkpoint/death persistence, DTO/Unity contract tests

주요 위험:

- 현재 Core와 Unity 모두 매우 큰 dirty worktree다. 후속 Goal 시작 시 관련 파일의 existing diff를 다시 검토해야 한다.
- Unity에는 승인된 no-compatibility 정책과 충돌하는 legacy/unknown fallback test가 있다. 각 DTO Goal에서 production reader와 test를 함께 제거해야 한다.
- current Core checkpoint는 in-memory payload뿐이다. 별도 배포 save/database가 존재한다면 이번 조사에서 확인하지 못한 외부 consumer다.
- Goal C는 범위가 크지만 임의로 나누면 Run HP와 physical damage, 또는 death와 asset recovery 사이에 dual authority가 생긴다. 내부 구현 단계와 focused test 단위를 세분화하되 완료 Goal은 하나로 유지하는 편이 안전하다.
- 이 문서 앞부분의 초기 정책 구간과 2026-07-18 승인 결정 사이 문구 차이는 별도 policy canonicalization에서 정리해야 한다. 이번 난이도 감사는 기존 문단을 최신 canonical 계약처럼 다시 쓰지 않는다.
