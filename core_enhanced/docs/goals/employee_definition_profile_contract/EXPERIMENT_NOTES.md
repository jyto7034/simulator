# Employee Definition Profile Contract Experiment Notes

## 초기화 기록

- goal_name: `employee_definition_profile_contract`
- 초기화 날짜: 2026-07-18
- 생성 경로: `docs/goals/employee_definition_profile_contract/`
- 경로 충돌: 없음. 필수 문서 3개가 모두 존재하지 않아 신규 Goal로 초기화했다.
- 관련 저장소 루트: Core, game_resources, game_server와 Unity 경로가 모두 존재했다.
- 초기 dirty 상태: Core/인접 simulator repository와 Unity repository 모두 광범위한 modified/deleted/untracked 상태였다.
- Goal 초기화 전 허용 범위에서는 개별 dirty diff의 소유자나 의미를 추정하지 않았다.

## 구조적 판단

- 현재 Child는 Master Phase 1이며 authored EmployeeDefinition/Profile이라는 하나의 계약 경계를 소유한다.
- RecruitmentQueue ownership, condition lifecycle와 Psychology는 각각 후속 Phase의 독립 경계로 유지한다.
- Master는 read-only 직접 의존 기록이며 현재 Child 수행 중 Master 상태를 함께 수정하지 않는다.
- typed 계약은 `EmployeeDefinition`을 authored source로 두고 starter와 recruitment가 같은 definition 값을 투영한다. roster `Employee`는 run-local `uuid`와 합류 당시 definition/Profile snapshot을 함께 보존해 checkpoint/retreat clone과 전투 초기화가 catalog 재추론 없이 같은 계약을 사용하게 한다.
- public candidate와 roster DTO는 공통 typed `EmployeeProfile` projection을 사용한다. candidate authored identity field는 `employee_definition_id`, roster는 `uuid`와 `employee_definition_id`를 모두 제공한다. 선택/채용 command와 result도 authored identity를 뜻하는 필드를 `employee_definition_id(s)`로 complete cutover한다.
- employee speed percent는 employee combat profile에 보존하고 weapon 적용 지점에서 weapon base interval에 공식을 적용한다. non-employee profile은 100% 중립을 사용해 기존 enemy semantics를 유지한다.
- Phase 2 전 recruitment selection의 node-seeded shuffle/truncate 동작은 유지하고, unified catalog에서 `starter_eligible == false`인 definition만 입력으로 사용한다.

## Source-of-truth 충돌

- 현재 authored source는 starter/recruitment 두 RON/DB이고 목표는 하나의 EmployeeDefinition catalog다.
- 현재 roster `Employee`는 run UUID/name만 보존하고 목표 stable definition ID/Profile을 잃는다.
- 현재 personality authority는 trust 내부 UUID modulo 단일 trait지만 목표는 definition의 authored risk/social 두 축이다.
- 현재 weapon 적용은 interval을 덮어쓰지만 목표는 weapon base interval 뒤 definition speed percent를 적용하는 것이다.
- 현재 data/core/Unity canonical 문서는 구형 두 file/candidate `id`/legacy roster shape를 기술한다.
- Master Phase 1 완료 게이트는 UUID trait 생성 제거를 요구하므로 Phase 4의 broader Psychology refactor를 기다리지 않고 UUID trait authority/projection만 제거한다. trust score/memory refactor는 Phase 4에 남긴다.

해결 결과:

- authored source를 `definitions.ron`/`EmployeeDefinitionDatabase` 하나로 교체했다.
- runtime roster는 authored ID/Profile과 run-local UUID를 함께 보존한다.
- candidate와 roster public shape, command/result, server, Unity가 같은 definition identity/Profile을 사용한다.
- personality authority는 definition의 risk/social 두 축이며 trust에는 trait projection이 남지 않는다.
- weapon base interval 뒤 speed percent 적용 순서를 Core가 소유한다.

## 정책 질문

결정이 필요한 질문:

- 12명의 필수 risk/social personality 조합을 어떤 authored 값으로 시작할 것인가?

현재 runtime 동작:

- roster 합류 시 run UUID modulo 6으로 여섯 단일 trait 중 하나를 선택한다.
- 같은 definition도 run UUID가 달라지면 trait가 달라질 수 있고 risk/social 두 축을 모두 가지지 않는다.

현재 live data:

- starter 6명, recruitment 6명 모두 personality 필드가 없다.

가능한 선택지:

1. 사용자가 12명의 exact risk/social 조합을 제공한다.
2. 사용자가 비최종 provisional authoring 권한을 위임하면 작업자가 각 background를 참고해 두 축을 명시 작성하고, gameplay modifier 없이 Profile 표시만 제공한다.
3. 전원 동일 기본값을 승인한다. 가장 빠르지만 캐릭터 가시성 목적이 약해져 권장하지 않는다.

권장안:

- 선택지 2. 이번 Goal은 기반 구축이고 사용자가 추후 수정한다고 승인했으므로 provisional authored 조합을 data에 명시하되 gameplay 수치 효과는 만들지 않는다. 단, 캐릭터 설정 authoring 권한은 명시적으로 확인해야 한다.

승인 요청용 provisional 배정안:

| employee_definition_id | 이름 | risk_disposition | social_disposition |
| --- | --- | --- | --- |
| `field_medic` | 이서연 | `Brave` | `Comradely` |
| `breach_guard` | 강민재 | `Brave` | `Obedient` |
| `records_clerk` | 한유진 | `Cautious` | `Defiant` |
| `scout_runner` | 오지훈 | `SurvivalInstinct` | `Comradely` |
| `containment_trainee` | 박하린 | `Cautious` | `Obedient` |
| `maintenance_tech` | 최도윤 | `Brave` | `Defiant` |
| `hq_field_medic` | 윤채린 | `Cautious` | `Comradely` |
| `hq_security_guard` | 서강우 | `Brave` | `Obedient` |
| `hq_recovery_runner` | 임하늘 | `SurvivalInstinct` | `Defiant` |
| `hq_containment_aide` | 장유나 | `Cautious` | `Obedient` |
| `hq_maintenance_aide` | 민도겸 | `SurvivalInstinct` | `Comradely` |
| `hq_records_observer` | 백수아 | `Cautious` | `Defiant` |

- 이 표는 2026-07-18 사용자가 승인한 이번 Goal의 provisional live authoring 값이다.
- 초기 배정은 세 social disposition을 각각 4명으로 분산하고, 기존 짧은 background의 행동 인상을 참고했다.
- personality는 이번 Phase에서 표시/향후 typed 소비 입력만 제공하고 stat, 장비 제한, Event modifier 또는 명령 거부 효과를 만들지 않는다.
- 사용자는 이후 live definition 값만 수정할 수 있고 schema/runtime code 변경은 필요하지 않도록 기반을 만든다.

승인 결과:

- 사용자 응답: `승인`
- 승인 범위: 위 12명 risk/social 조합과 gameplay modifier를 추가하지 않는 provisional Profile authoring
- Goal Status를 `active`로 복귀하고 production baseline 및 구현을 진행한다.

별도 결정이 필요하지 않은 migration 제안:

- current combat behavior를 보존하도록 전원 base stats 100/10/0/0, speed percent 100, aptitude `Balanced`로 시작한다.
- starter eligibility와 initial loadout은 현재 두 file membership/data를 그대로 옮긴다.
- role/background 문자열에서 gameplay stat이나 specialty를 추론하지 않는다.

## 외부 시스템과의 직접 의존성

- `/mnt/f/work/simulator/game_resources` production live employee data
- `/mnt/f/work/simulator/game_server` Core DTO transport
- `/mnt/f/unity projects/ark` reader/store/presentation/tests 및 canonical Unity 계약
- workspace 밖 외부 durable save/database 소비자 존재 여부는 확인할 수 없었다.

## 범위 밖 후속 후보

- Phase 2 RecruitmentQueue ownership
- Phase 3 Employee Condition lifecycle
- Phase 4 Employee Psychology state
- 최종 직원별 stats/personality/aptitude 콘텐츠 authoring

현재 Child에서 새 Goal이나 Master를 자동 생성하지 않는다.

## 적용하지 않은 대안과 이유

- starter/recruitment catalog를 별도 유지하지 않는다. 승인 정책은 하나의 authored definition source를 요구한다.
- compatibility/dual schema를 계획하지 않는다. 승인 정책은 complete cutover를 요구한다.
- UUID나 trust state에서 personality를 계속 추론하지 않는다. personality authority는 authored definition이다.
- RecruitmentQueue나 condition/Psychology migration을 함께 구현하지 않는다. 독립적인 후속 source-of-truth 경계다.

## Dirty Worktree 주의사항

- Core의 employee/data/stat/world/snapshot/test와 canonical 문서에 기존 수정이 있다.
- game_resources와 game_server에도 기존 수정 및 untracked 파일이 있다.
- Unity의 `RosterModel`, `StarterSelectionModel`, `SelectedEventModel`, presentation/tests/canonical 문서를 포함한 다수 파일에 기존 수정이 있다.
- production 수정 전 관련 저장소 상태와 대상 파일 diff를 다시 확인해 Goal 변경과 사용자 변경을 분리한다.
- 관련 없는 기존 변경을 포맷·정리·복원·stash하지 않는다.
- 구체적 겹침: `employee.rs`에는 strict serde와 growth test, `employee_data.rs`에는 typed validation error 전환, `battle/stat_pipeline.rs`와 `battle/types.rs`에는 Artifact 제거가 이미 수정돼 있다. 이 변경을 보존한 현재 파일 위에서만 작업해야 한다.
- `behavior.rs`, world state/snapshot과 Unity reader/tests/canonical은 다른 완료 Goal의 대규모 변경이 겹친다. 수정 전 파일별 diff와 현재 call site를 다시 확인한다.

보존 결과:

- 관련 파일의 현재 diff 위에 scoped patch를 적용했고 reset/restore/checkout/stash/clean을 사용하지 않았다.
- 관련 없는 modified/deleted/untracked 파일을 삭제하거나 정리하지 않았다.
- Core/simulator와 Unity의 scoped `git diff --check`가 통과했다.

## 접근하거나 검증하지 못한 소비자

- 현재 workspace 밖 employee DTO 소비자
- 배포된 외부 durable save/database
- 실제 배포 버전과 legacy payload 보유 상태

## 생략한 검증과 근거

- Goal 문서 초기화 전 production test/formatter/runtime 조사를 수행하지 않았다. 신규 Goal 초기화 규칙에 따라 문서 3개를 먼저 만들기 위해 생략했다.
- 외부 durable save/database와 workspace 밖 DTO 소비자는 접근할 수 없어 baseline에서 검증하지 못했다. 현재 접근 가능한 Core/server/Unity 계약에는 compatibility를 두지 않는다.
- Unity PlayMode는 DTO/read model/command/presentation 계약 변경의 필수 증거가 아니어서 실행하지 않았다. 실제 Core fixture를 소비하는 focused EditMode와 전체 EditMode 240개로 접근 가능한 Unity 경계를 검증했다.

## 남은 위험

- 외부 durable persistence가 확인되면 save migration 범위가 달라질 수 있다.
- workspace 밖 배포 DTO consumer와 legacy payload 보유 상태는 확인하지 못했다. 승인 정책에 따라 compatibility는 추가하지 않았다.
- authored content는 승인된 provisional personality와 행동 보존용 stat/aptitude 값이다. 최종 밸런스 authoring은 범위 밖 후속 후보다.
- Phase 2 전까지 recruitment는 unified catalog의 비starter definition을 기존 node-seeded shuffle/truncate로 보여 준다. 유한 queue/dedupe는 아직 구현되지 않았다.

## Master 재개 기록 복구

- 2026-07-18 Master 재개 중 경로 확인 명령의 실패를 성공으로 오판해 기존 complete Child 문서 3개를 덮어썼다.
- 이전 Codex session JSONL에서 이 Goal에 적용된 9개 patch를 원래 순서대로 `/tmp`에 재생하고 최종 225줄 PLAN, 458줄 EXPERIMENTS, 155줄 EXPERIMENT_NOTES를 복구했다.
- 복구된 세 문서는 처음부터 끝까지 다시 읽었고, 이전 `Status: complete`, 정책 승인, implementation history와 verification evidence가 보존됐다.
- production 파일은 이 사고와 복구 과정에서 변경하지 않았다.
- 재검증에서는 기존 증거에 더해 Unity PlayMode 57 pass/4 explicit skip을 확인했다.
