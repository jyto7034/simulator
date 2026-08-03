# Event 직원 판정 주사위 정책 결정 — 2026-07-17/19

- 최초 논의일: 2026-07-17
- 정책 승인일: 2026-07-18
- 구현 완료·검증일: 2026-07-19
- 상태: historical policy decision input / implementation complete
- 성격: Event 직원 판정 정책의 승인 배경과 최종 구현 disposition

이 문서는 modifier 없는 초기 d20 판정을 직원 Profile, 장비와 선택지별 난이도를 사용하는
판정으로 확장하기 위해 승인한 결정을 보존한다. 현재 gameplay, runtime, live RON과 Unity wire
계약의 source of truth가 아니며, 현재 동작은 runtime/live data와 아래 durable canonical을 따른다.

구현은 [Event Employee Check Dice System Goal](../goals/event_employee_check_dice_system/PLAN.md)에서
완료됐다. 세부 실행·실패·검증 기록은 해당 Goal의
[EXPERIMENTS.md](../goals/event_employee_check_dice_system/EXPERIMENTS.md)와
[EXPERIMENT_NOTES.md](../goals/event_employee_check_dice_system/EXPERIMENT_NOTES.md)가 소유한다.

## 현재 disposition

| 영역 | 현재 canonical 또는 authority |
| --- | --- |
| Event의 플레이 규칙, aptitude·DC·장비 보정·key 의미 | [game_rulebook.md](../game_rulebook.md) |
| 판정, RNG, committed result, duplicate/checkpoint/retreat 경계 | [core_runtime_contract.md](../core_runtime_contract.md) |
| Employee/Event/Equipment RON schema와 strict validation | [data_loading_contract.md](../data_loading_contract.md) |
| 실제 구현 | `src/game/world/event_node.rs`, `src/game/employee.rs`, `src/game/data/event_data.rs`, `src/game/data/equipment_data.rs` |
| Live content | `game_resources/data/employees/definitions.ron`, `equipments/base.ron`, `events/story/base.ron`, `map/node_definitions.ron` |
| Unity-facing JSON과 소비 계약 | `/mnt/f/unity projects/ark/docs/unity_core_contract.md` |
| 실제 transport probe | [probe_registry.md](../probe_registry.md) |

초기 baseline의 scene-common `required_roll`, global employee candidate list와 modifier 없는
`raw_roll >= required_roll` 판정은 제거됐다. 호환 alias, dual schema와 Unity-side 보정 경로는
현재 계약에 없다.

## 승인한 문제와 목표

초기 Event employee check는 수행 직원을 선택해도 모든 eligible 직원의 성공 확률이 같았다.
승인한 목표는 BG3 전체 규칙을 복제하는 것이 아니라 다음 한 경계를 만드는 것이었다.

```text
선택지별 Event aptitude와 DC
-> 직원의 고정 Event aptitude와 장착 장비 평가
-> Core가 raw d20을 한 번만 확정
-> modifier와 natural 1/20을 적용해 결과 commit
-> Unity는 Core가 투영한 확률과 근거를 표시
```

Core는 판정, eligibility, RNG, 결과와 effect commit의 유일한 authority다. Unity는 선택 전 로컬
cursor와 choice/employee selection만 소유하며 DC, modifier, key, 확률과 결과를 재계산하지 않는다.

## 승인·구현된 정책

### Employee Event aptitude

각 `EmployeeDefinition`은 아래 `EventAptitude` 중 정확히 하나를 명시한다.

```text
Medical
Security
Analysis
Reconnaissance
Containment
Maintenance
```

Definition의 값은 직원 생성 시 저장된 `EmployeeProfile.event_aptitude`로 복사된다. 생성 뒤
runtime과 checkpoint는 저장된 Profile 값을 authority로 사용하며 role, background, 이름, UUID,
personality, trust 또는 현재 definition에서 aptitude를 재추론하지 않는다.

- 선택지 aptitude와 일치: `+3`
- 불일치: `+0`; ineligibility나 페널티가 아님
- `CombatAptitude`, level, Psychology, condition 수치, consumable과 skill fragment는 roll modifier가 아님
- Condition은 `EmployeeActivity::Event` eligibility에만 사용

이는 범용 `specialties[]`, 숙련 rank 또는 성장형 proficiency를 도입한 것이 아니다. Event 판정만
소비하는 단일 typed aptitude다.

### Choice와 판정

`employee_check_choices`의 각 choice는 다음 check를 소유한다.

```text
aptitude: EventAptitude
dc: 8 | 12 | 16 | 20
required_choice_key: Option<EventChoiceKeyId>
```

하나의 Event에는 도달 가능한 checked scene이 최대 하나이며, 결과에서 같은 checked scene으로
돌아오는 cycle을 허용하지 않는다. 판정 없는 진행은 `advance` scene을 사용한다.

```text
employee_modifier = aptitude match ? 3 : 0
equipment_modifier = 적용 가능한 장착 장비 중 최고 한 개, 0..2
modifier_total = employee_modifier + equipment_modifier
total_roll = raw_roll + modifier_total

raw_roll == 1  -> critical_failure
raw_roll == 20 -> critical_success
total_roll >= dc -> success
otherwise -> failure
```

`success_chance_percent`는 raw d20의 20개 face를 같은 outcome 함수로 평가한 값이다. Choice,
performer와 equipment는 raw roll identity를 바꾸지 않는다.

### Equipment modifier와 choice key

장착한 owned equipment instance만 평가한다.

- `event_check_modifiers`: aptitude별 `+1` 또는 `+2`
- 같은 aptitude modifier는 합산하지 않고 최고값 하나만 적용
- 동률은 equipment instance UUID 오름차순
- `event_choice_keys`: 선택지 eligibility를 여는 비소모 key
- key는 modifier를 암묵적으로 제공하지 않음
- inventory-only 장비, 다른 직원의 장비와 equipment type 추론은 사용하지 않음

Preview와 committed result는 적용된 equipment instance/definition, aptitude, amount와 key provider를
typed provenance로 보존한다.

### Commit과 복구

유효한 첫 resolve만 raw d20을 확정하고 `committed_check`에 source scene, choice, employee,
aptitude, DC, raw/total roll, modifier components, optional key evidence와 outcome을 저장한다.

- invalid/stale/ineligible/missing-key 요청은 RNG와 상태를 변경하지 않음
- exact duplicate는 같은 committed result를 반환하고 effect를 다시 지급하지 않음
- reconnect와 committed state를 포함한 checkpoint는 저장된 결과를 재사용
- Event-started combat의 retry/retreat는 owning Event의 committed check를 보존
- success grant 또는 combat plan과 Event state는 기존 staged/atomic 경계를 사용

## 첫 Live vertical slice

`sealed_archive_recovery_01`이 첫 checked Event다.

- `decode_archive_seal`: Analysis, DC 16, `sealed_archive_access`, 성공 시 80 Enkephalin
- `stabilize_archive_power`: Maintenance, DC 12, key 없음, 성공 시 30 Enkephalin
- `archive_scanner`: Analysis `+2`와 `sealed_archive_access` 제공
- `maintenance_tech`: authored 초기 장비로 `archive_scanner` 보유
- `event_sealed_archive`: 생성 맵에서 Event를 명시적으로 참조
- `white_night_confession_01`: 기존 advance-only Event로 유지

이 vertical slice는 직원/장비 선택, 서로 다른 위험·보상, missing-key disabled reason,
success/failure/critical, duplicate/checkpoint/reconnect와 no-double-grant 경계를 검증한다.

## 승인된 비범위

다음은 구현 누락이 아니라 승인된 비범위다. 실제 도입 시 새 정책 결정과 Goal이 필요하다.

- Event당 여러 employee check와 roll history
- Advantage/Disadvantage, reroll, bonus die와 assistance
- 복수 choice key, AND/OR, key 수량·소비와 hidden discovery
- personality, trust, memory와 condition 수치의 roll modifier
- trust 기반 warning/refusal
- performer-targeted condition과 critical 전용 reward/effect
- 범용 specialties, aptitude rank와 성장형 proficiency
- 외부 durable save/database compatibility 또는 migration

실제 localized Event 문구와 illustration asset 제작은 Dice gameplay 계약이 아니라 Unity 콘텐츠·표현
범위다. Wire의 `image_id`, `text_ids`, `label_id`는 semantic resource ID이며 Core는 본문이나 이미지를
내려주지 않는다.

## 검증 disposition

완료 Goal은 strict schema, 20면 outcome/chance matrix, shared projection/resolve evaluator,
invalid no-mutation, exact duplicate/checkpoint/retreat, live RON reachability, Core-generated Unity
fixture, game_server mapping, Unity consumer와 실제 WebSocket probe를 검증했다.

이 기록의 과거 baseline이나 구현 순서를 현재 정책처럼 사용하지 않는다. 현재 구현 여부를 판단할
때는 위 canonical, runtime/live data와 최신 실행 evidence를 다시 확인한다.
