# Employee Domain Implementation Difficulty Audit Experiments

> 경로 정리: 이 감사 당시 `docs/tmp/employee_domain_policy_discussion_2026-07-17.md`였던 입력
> 문서는 이후 `docs/decisions/employee_domain_policy_2026-07-17.md`로
> 승격됐다. 아래 검증 경로는 현재 위치로 정규화했으며 당시 판정은 변경하지 않았다.

## 2026-07-18 — Employee definition, roster, recruitment and profile inventory

- 가설: 고정 직원 definition과 채용 정책은 기존 candidate schema를 확장하는 국소 변경이 아니라 identity와 run ownership migration을 요구한다.
- 수행한 조사:
  - `sed -n '1,470p' src/game/employee.rs`
  - `sed -n '1,280p' src/game/data/employee_data.rs`
  - `rg -n 'starter_employee|recruitment_employee|EmployeeData|StarterEmployeeCandidate' ...`
  - starter/recruitment live RON, world starter initialization, headquarters recruitment, node session과 관련 tests를 읽었다.
  - `UnitCombatProfile`, weapon profile, stat pipeline과 trust runtime/snapshot을 읽었다.
- 변경한 내용: Goal 실행 기록만 갱신했다. Production code/data는 변경하지 않았다.
- test runner: 실행하지 않음. 읽기 전용 코드 조사이므로 test 수/pass/fail/skip은 해당 없음.
- 실제 결과:
  - live 직원은 starter 6명과 recruitment 6명이 별도 RON/DB에 있고 cross-catalog identity uniqueness가 없다.
  - roster `Employee`에는 authored definition ID, role/background/personality가 없고 run UUID와 name만 남는다.
  - 시작 선발은 선택된 후보만 roster로 만들고 미선택 후보를 run queue에 보존하지 않는다.
  - headquarters 후보는 node seed로 전체 recruitment catalog를 매번 shuffle/truncate하며, 채용 후 definition 소유권이나 중복 채용을 추적하지 않는다.
  - `HeadquartersContactSessionState`가 후보 전체 struct를 복제해 보유하고 다른 HQ action을 선택하면 session과 함께 버린다.
  - 기본 직원 combat profile은 전 직원 공통이고 weapon 적용이 `basic_attack.interval_ms`와 `stats.attack_interval_ms`를 직접 덮어쓴다.
  - personality 후보는 trust state 안에서 run UUID modulo로 단일 trait가 자동 생성된다.
- 판단:
  - definition/profile과 recruitment ownership은 같은 definition identity cutover를 공유하므로 하나의 후속 Goal로 묶을 근거가 강하다.
  - 공격 속도 보정은 단순 RON field 추가가 아니라 weapon override와 final-stat pipeline 순서 계약 변경을 포함한다.
  - psychology migration은 profile의 authored personality에 의존하지만 trust state/DTO 자체는 별도 완결 경계로 분리 가능하다.

## 2026-07-18 — Employee condition, combat attempt and recovery inventory

- 가설: Run HP 제거는 `Employee` 필드 교체보다 훨씬 넓고, combat transaction과 eligibility를 같은 cutover에서 바꿔야 한다.
- 수행한 조사:
  - `rg -n 'EmployeeHealthState|EmployeeLifeState|EmployeeAvailability|EmployeeInjury|...' src/game ...`
  - `src/game/world/combat.rs`, `src/game/world/state.rs`, `src/game/combat_setup/player_spawns.rs`, support/event/loadout validation과 result DTO를 읽었다.
  - live `run/policy.ron`, consumable RON/schema와 관련 test 파일 목록을 확인했다.
- 변경한 내용: Goal 실행 기록만 갱신했다. Production code/data는 변경하지 않았다.
- test runner: 실행하지 않음. 읽기 전용 코드 조사이므로 test 수/pass/fail/skip은 해당 없음.
- 실제 결과:
  - active battle은 entry roster clone을 `BattleEmployeeAttemptState`로 소유하고 RuntimeUnit instance별 processed set으로 전투불능을 한 번씩 반영한다.
  - battle result는 attempt roster 전체를 staged inventory/fragment/roster/enkephalin transaction에 복사해 commit하고 retreat는 `BattleEntrySnapshot` 전체를 복원한다.
  - Run HP는 battle-start formula, post-battle result diff, admin mutation, consumable mitigation, live policy와 다수 테스트에 노출된다.
  - Event는 `life_state == Alive`, loadout은 `is_available_for_combat`, consumable은 별도 alive predicate를 사용해 eligibility가 분산돼 있다.
  - live deployment DTO는 available employee만 먼저 filter한 뒤 cost/cooldown reason을 계산하므로 condition disabled candidate를 이유와 함께 보여줄 수 없다.
  - recovery restriction은 현재 실제 Recruitment capability만 지원하며 Treatment enum은 dead code다.
  - 사망 시 장착 장비의 `equipped_to`, employee item slot과 skill fragment loadout을 원자적으로 회수하는 중앙 death path가 없다.
  - live consumables에는 TraumaMitigation, BattleHpSetup, DeathPrevent 등이 있고 schema에는 제거 대상 `RunHpLossMitigation`이 남아 있다.
- 판단:
  - condition field migration, central eligibility, combat attempt/redeploy/result/retreat, treatment/recovery와 death asset transaction은 동일한 employee condition lifecycle cutover다.
  - 이를 분리하면 구형 `is_available_for_combat`/Run HP와 새 physical damage evaluator가 동시에 존재하거나 사망과 자산 상태가 불일치하는 중간 계약이 생긴다.
  - 후속 Condition Goal은 매우 높음 난이도로 평가해야 하며 단일 Goal 내부에서 단계화하는 것이 적절하다.

## 2026-07-18 — Checkpoint, wire, server and Unity consumer inventory

- 가설: server보다 checkpoint와 Unity read-model/presentation migration이 직원 정책 구현 난이도의 주요 cross-layer 비용이다.
- 수행한 조사:
  - `RunCheckpointState`, `RunCheckpointPayload`, save/load mutation과 관련 tests를 읽었다.
  - game_server의 typed request deserialize, Core snapshot serialization과 command-result forwarding을 읽었다.
  - Unity `RosterModel`, `StarterSelectionModel`, `SelectedEventModel`, presentation composers와 EditMode/PlayMode tests의 직원 필드 소비를 검색·확인했다.
  - Core/Unity canonical 문서에서 직원, 전투 condition, recruitment와 recovery 계약을 검색했다.
- 변경한 내용: Goal 실행 기록만 갱신했다. Production code/data는 변경하지 않았다.
- test runner: 실행하지 않음. 읽기 전용 코드 조사이므로 test 수/pass/fail/skip은 해당 없음.
- 실제 결과:
  - 현재 Run checkpoint는 직렬화된 디스크 save가 아니라 Core 메모리 안의 clone payload다. roster/inventory/fragments/UUID와 run state 일부를 명시 필드로 캡처·복원한다.
  - 새 RecruitmentQueue/session ownership은 checkpoint payload와 `RunState::with_persistent_run_state_from` 계열 Floor 전환 보존을 명시 갱신해야 한다.
  - game_server는 `PlayerBehavior`를 shared Core type으로 deserialize하고 `RunSnapshotDto`를 JSON으로 직렬화하는 얇은 transport라 직원 의미 재계산이 거의 없다.
  - Unity `RosterModel`은 `life_state`, `health.current_hp/max_hp`, `trauma`, `available_for_combat`를 직접 읽고 `Unknown != Dead`를 living으로 취급하는 fallback이 있다.
  - Unity loadout, item use, support, battle roster, maintenance와 preparation presentation이 HP/life/availability를 표시 또는 로컬 affordance에 사용한다.
  - Unity starter와 HQ recruitment reader는 candidate `id`를 사용하고 여러 contract/render tests가 현재 candidate/roster JSON shape를 고정한다.
  - Unity tests에는 legacy health field 무시, unknown life-state fallback 같은 compatibility expectation이 있어 승인된 no-compatibility cutover에서 함께 제거해야 한다.
- 판단:
  - server migration은 중간 이하의 추가 비용이지만 shared DTO serialization tests는 필수다.
  - Unity read model, presentation, mocks, EditMode/PlayMode tests가 definition/recruitment와 condition Goal 모두의 높은 비용 요인이다.
  - 디스크 save consumer는 현재 확인되지 않았으므로 save migration 위험은 checkpoint clone/restore에 집중된다. 배포 외부 저장 시스템 존재 여부는 확인되지 않은 위험으로 남긴다.

## 2026-07-18 — Canonical/test evidence synthesis and documentation verification

- 가설: 승인 정책 전체를 실제 근거에 연결하고 독립 source-of-truth 경계를 A~D로 나눌 수 있다.
- 수행한 조사와 검증:
  - Core/Unity canonical 문서와 완료 review에서 employee/recruitment/condition/recovery 계약을 repository-wide 검색했다.
  - 관련 Core unit/world/integration test 파일, server serialization tests와 Unity EditMode/PlayMode 소비자 tests를 검색했다.
  - 정책별 난이도 표, Goal A~D의 범위·완료 조건 후보와 A→B→C→D 순서를 대상 문서에 작성했다.
  - 다음 정확한 검증 명령을 실행했다.

```text
set -e
for path in docs/goals/employee_domain_implementation_difficulty_audit/PLAN.md docs/goals/employee_domain_implementation_difficulty_audit/EXPERIMENTS.md docs/goals/employee_domain_implementation_difficulty_audit/EXPERIMENT_NOTES.md docs/decisions/employee_domain_policy_2026-07-17.md src/game/employee.rs src/game/data/employee_data.rs src/game/world/combat.rs src/game/world/state.rs src/game/world/node_flow.rs src/game/behavior.rs ../game_server/src/game/player_game_actor/handlers.rs '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/Reading/RosterModel.cs'; do test -f "$path"; done
rg -n '^## 13\. 실제 구현 난이도 감사|^### Goal [A-D]|^## 15\. 권장 수행 순서' docs/decisions/employee_domain_policy_2026-07-17.md
if rg -n '[[:blank:]]+$' docs/goals/employee_domain_implementation_difficulty_audit docs/decisions/employee_domain_policy_2026-07-17.md; then exit 1; fi
wc -l docs/decisions/employee_domain_policy_2026-07-17.md docs/goals/employee_domain_implementation_difficulty_audit/*.md
git status --short -- docs/decisions/employee_domain_policy_2026-07-17.md docs/goals/employee_domain_implementation_difficulty_audit
```

- 변경한 내용: Goal 문서 3개를 생성·갱신하고 직원 도메인 임시 문서에 난이도 감사 절을 추가했다. Production code/data는 변경하지 않았다.
- test runner: 실행하지 않음. Markdown/읽기 전용 감사이므로 test 수/pass/fail/skip은 해당 없음.
- 실제 결과:
  - 대표 근거 경로 12개가 모두 존재했다.
  - 난이도 감사 절, Goal A~D와 수행 순서 heading이 모두 검색됐다.
  - 대상 문서에서 trailing whitespace가 검출되지 않았다.
  - 대상 문서는 691줄, Goal 문서 3개는 당시 합계 310줄이었다.
  - scoped status에는 새 Goal 디렉터리와 기존 untracked `docs/tmp` 문서만 나타났다.
- 판단:
  - 모든 승인 정책을 난이도 표에 연결했고 Core/data/checkpoint/DTO/server/Unity 영향과 Goal 결합 이유를 기록했다.
  - production 변경이 없어 Rust/server/Unity runner 검증은 완료 조건에 필요하지 않다.

### Verification correction

- 첫 최종 keyword 검증 script는 대상 문서에 문자 그대로 `A → B → C → D`가 있을 것으로 가정해 해당 pattern에서 종료했다. 문서에는 code block의 개별 `-> Goal` 행과 numbered order가 있어 내용 누락이 아니라 검증 pattern 오류였다.
- 변경한 내용: 문서를 바꾸지 않고 실제 표기인 Goal A heading과 각 `-> Goal B/C/D` 행을 검사하도록 명령을 수정했다.
- 수정 후 정확한 검증 명령:

```text
set -e
! rg -n 'discovery pending' docs/goals/employee_domain_implementation_difficulty_audit
for pattern in '단일 `EmployeeDefinition` catalog' 'RecruitmentQueue 단일 소유권' 'Run HP 제거' '목적별 중앙 eligibility' '의미 단위 memory' '### Goal A — `employee_definition_profile_contract`' '### Goal B — `employee_recruitment_queue_ownership`' '### Goal C — `employee_condition_lifecycle_migration`' '### Goal D — `employee_psychology_state_contract`' 'Goal A: EmployeeDefinition/Profile' '-> Goal B: RecruitmentQueue ownership' '-> Goal C: Condition lifecycle migration' '-> Goal D: Psychology state contract'; do rg -F -q -- "$pattern" docs/decisions/employee_domain_policy_2026-07-17.md; done
! rg -n '[[:blank:]]+$' docs/goals/employee_domain_implementation_difficulty_audit docs/decisions/employee_domain_policy_2026-07-17.md
test "$(find docs/goals/employee_domain_implementation_difficulty_audit -maxdepth 1 -type f | wc -l)" -eq 3
echo verified
```

- 실제 결과: `verified`, exit code 0.
- test runner 수치: 해당 없음.
