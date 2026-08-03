# Employee Definition Profile Contract Completion Review

Status: complete

Reviewed: 2026-07-18

## 결론

Master Phase 1 `employee_definition_profile_contract`는 현재 worktree에서 완료됐다. unified authored catalog, stable definition identity/Profile, attack speed pipeline과 Core→server→Unity complete cutover가 모두 구현·검증돼 Phase 2의 안정된 선행 계약으로 사용할 수 있다.

## 완료 조건별 증거

- 동일 authored source: `game_resources/data/employees/definitions.ron` 하나가 starter 6명과 non-starter 6명을 소유하며 Core `EmployeeDefinitionDatabase`가 이를 publication한다.
- strict startup rejection: duplicate/nonempty identity, required Profile, starter count, max HP, positive attack speed와 initial loadout reference tests가 통과한다.
- identity 분리: candidate/command는 `employee_definition_id(s)`, roster는 run-local `uuid`와 `employee_definition_id`를 함께 사용한다.
- authored Profile: role/background, 두 personality 축, `CombatAptitude`, base stats와 attack speed가 candidate/recruitment/roster에 같은 shape로 투영된다.
- stat pipeline: default 및 weapon interval에 `ceil(base * 100 / speed_percent)`를 적용하고 125%/1001ms의 801ms ceiling case를 test가 고정한다.
- Unity cutover: current reader/command/presentation이 authored identity/Profile을 사용하고 legacy candidate alias negative tests가 통과한다.
- legacy 제거: old two candidate DB/RON, UUID-derived `EmployeeTrait`/`new_for_employee`/`trust.traits`와 specialties production contract가 없다.
- canonical sync: gameplay, runtime, data-loading, server bridge와 Unity wire 문서가 current contract를 기술한다.

## 현재 검증

- Core: 759 passed, 0 failed, 0 ignored
- game_server: 30 passed, 0 failed, 0 ignored
- Unity focused EditMode: 4 passed, 0 failed, 0 skipped
- Unity full EditMode: 240 passed, 0 failed, 0 skipped
- Unity full PlayMode: 57 passed, 0 failed, 4 explicit server-dependent skipped
- Rust-generated Unity fixture comparison: exact match

## Phase 2에 제공하는 안정 계약

- Queue key는 `employee_definition_id`다.
- roster membership/instance key는 `employee.uuid`다.
- starter pool은 unified catalog의 `starter_eligible == true`, recruitment pool은 `false` definition이다.
- Queue/Session/Roster ownership은 Phase 2가 추가하되 Phase 1 Profile shape와 no-compatibility 정책을 바꾸지 않는다.

## 남은 위험과 비범위

- workspace 밖 durable save/DTO consumer는 확인하지 못했다.
- 직원별 최종 balance/personality/aptitude authoring은 후속 콘텐츠다.
- 현 node-seeded recruitment reroll/dedupe/checkpoint 문제는 Phase 2가 소유한다.
- Phase 1 기록은 재개 중 한 차례 덮어써졌으나 이전 session patch log로 정확히 복구했고 사고와 재검증을 실행 기록에 남겼다.
