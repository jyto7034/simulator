# Employee Definition Profile Contract

Status: complete

Created: 2026-07-18

Last updated: 2026-07-18

Goal type: `employee_domain_implementation_master` Phase 1 Child Goal

## 목표

starter와 recruitment에 흩어진 직원 정적 정의를 하나의 strict `EmployeeDefinition` catalog로 통합하고, authored identity와 run-local identity, Profile, 전투 가시성 정보와 직원 고유 공격 속도를 Core 생산자부터 접근 가능한 Unity 소비자까지 하나의 계약으로 전환한다.

## 해결할 사용자-visible 문제

현재 직원의 authored identity와 Profile이 시작 후보·채용 후보·roster 사이에서 일관된 하나의 정의로 보존되지 않고, 성격은 authored data가 아니라 runtime UUID에서 추론되며, 직원 고유 공격 속도와 전투 성향을 일관되게 읽을 계약이 없다.

사용자는 이 Goal 완료 후 starter, recruitment와 roster 어디서든 같은 직원 definition과 Profile을 보고, 직원의 두 personality 축·전투 성향·기본 능력치·공격 속도 특성이 명시적인 authored 정보로 일관되게 적용되는 결과를 보게 된다.

## 감사 Finding ID

없음. 사용자 승인 정책과 `employee_domain_implementation_difficulty_audit`에서 도출된 Master Phase 1이다.

## Master와 직접 의존성

- Master PLAN: `docs/goals/employee_domain_implementation_master/PLAN.md`
- 현재 Phase: `### Phase 1 — EmployeeDefinition/Profile 계약`
- 승인 순서: Phase 1이며 선행 Phase 없음
- 선행 completion review: 없음
- 직접 의존 correction/Goal 결정: 없음
- 정책·난이도 근거: `docs/goals/employee_domain_implementation_difficulty_audit/PLAN.md`와 `docs/decisions/employee_domain_policy_2026-07-17.md`

Master는 read-only 의존 기록으로 사용한다. 현재 Child가 complete된 뒤 Master 상태 갱신과 다음 Phase 전환은 Master Goal을 별도로 재개해 수행한다.

## 대상 저장소와 경로

- Core runtime, schema, DTO, tests와 Goal/canonical 문서: `/mnt/f/work/simulator/core_enhanced`
- production live employee data: `/mnt/f/work/simulator/game_resources`
- game_server producer transport와 tests: `/mnt/f/work/simulator/game_server`
- 접근 가능한 Unity reader/store/presentation/tests 및 canonical 문서: `/mnt/f/unity projects/ark`

## 구현 전 Runtime 및 Live Data 근거

- production loader `GameDataBase::load_live_embedded()`는 `GameplayRonSourceSet`에서 `employees/starter_candidates.ron`과 `employees/recruitment_candidates.ron`을 별도 `StarterEmployeeCandidateDatabase`/`RecruitmentEmployeeCandidateDatabase`로 parse한다.
- 두 live 파일에는 총 12명의 `id`, name, role, background가 있고 starter 6명만 `starter_loadout`을 가진다. personality, aptitude, authored base stats와 speed percent는 없다.
- `GameDataBase`와 builder/fixture가 두 DB를 별도 필드로 보유하며 cross-catalog ID uniqueness를 검증하지 않는다.
- 새 게임 시작은 starter DB 전체를 `GameCoreState.starter_candidates`로 clone하고 선택된 candidate를 `Employee::from_starter_candidate`로 변환한다.
- Headquarters는 recruitment DB를 node seed로 shuffle/truncate하고 같은 candidate 타입을 roster로 변환한다. Queue ownership은 Phase 2 범위다.
- runtime `Employee`는 run UUID와 name을 저장하지만 authored definition ID, role/background, personality와 aptitude를 보존하지 않는다.
- 모든 직원은 `UnitCombatProfile::employee_default()`의 공통 max HP 100, attack 10, defense 0, magic resist 0과 기본 interval 1500ms를 사용한다.
- `UnitCombatProfile::apply_weapon_profile`은 weapon interval을 `basic_attack.interval_ms`와 `stats.attack_interval_ms`에 직접 덮어써 직원 고유 speed 단계가 없다.
- `EmployeeTrustState::new_for_employee`는 run UUID modulo 6으로 단일 `EmployeeTrait`을 선택하고 roster snapshot의 `trust.traits`로 노출한다.
- Core candidate DTO는 production data struct를 직접 직렬화하고 roster DTO는 UUID/name과 legacy trust traits를 별도로 투영한다.
- game_server는 Core `RunSnapshotDto`를 `serde_json::to_value`로 전달하며 별도 직원 의미를 계산하지 않는다. 다만 server fixture가 구형 두 DB/type을 직접 생성한다.
- Unity `StarterSelectionModel`과 `RecruitmentCandidateView`는 candidate `id`; `RosterModel`은 UUID/name과 현재 combat fields를 읽는다. Unity 관련 reader/test/canonical 파일은 이미 큰 사용자 diff를 가진다.
- 현재 canonical data/Unity 문서는 두 employee RON과 candidate `id`/legacy roster shape를 계약으로 기록해 승인된 목표와 충돌한다.

## Source of Truth와 소유 계층

승인된 목표 source of truth:

- authored identity와 Profile: Core가 strict loading하는 하나의 `EmployeeDefinition` catalog
- roster identity: `employee_definition_id` 참조와 합류 후 생성되는 run-local `employee_uuid`
- public projection: typed Core DTO
- transport: game_server는 직원 의미를 재계산하지 않고 Core 계약을 전달
- presentation: Unity는 Core의 typed identity/Profile을 소비하며 personality나 stat 의미를 재추론하지 않음

구현 전 actual authored source는 두 candidate RON/DB였고, run employee source는 `EmployeeRoster` 안의 `Employee`, public producer는 Core `RunSnapshotDto`/candidate data struct였다. 완료 후 실제 source와 producer-consumer 일치는 아래 완료 증거에 기록한다.

## 유지할 불변식

- 모든 authored 직원은 전역적으로 유일한 `employee_definition_id`를 가진다.
- `employee_definition_id`는 authored identity이고 `employee_uuid`는 roster 합류 뒤의 run-local identity다.
- starter, recruitment와 roster는 같은 EmployeeDefinition source를 참조한다.
- 각 definition은 승인된 두 personality 축에서 정확히 하나씩 authored 값을 가진다.
- UUID 기반 personality/trait 생성이나 trust 계층의 personality 소유를 남기지 않는다.
- `CombatAptitude`는 가시성 정보이며 장비 제한이나 불일치 페널티를 만들지 않는다.
- 직원 공격 속도는 무기 기본 공격 주기에 `ceil(base_interval * 100 / basic_attack_speed_percent)`로 적용하며 percent는 양의 정수이고 100은 중립이다.
- specialties를 production schema/runtime/DTO/save에 추가하지 않는다.
- Core producer, game_server와 접근 가능한 Unity 소비자가 같은 typed 계약을 사용한다.
- 승인되지 않은 compatibility, dual-read/write, fallback, optional alias 또는 unknown-value recovery를 만들지 않는다.
- 기존 사용자 변경과 Goal 밖 dirty 파일을 보존한다.

## 승인된 정책과 근거

- 무한 랜덤 생성이 아닌 authored 고정 직원 pool을 사용한다.
- starter와 recruitment를 하나의 `EmployeeDefinition` catalog로 통합하고 시작 자격을 명시한다.
- definition은 stable authored ID, role/background, starter eligibility, initial loadout, 두 personality 축, `CombatAptitude`, 직접 authored base stats와 `basic_attack_speed_percent`를 가진다.
- personality risk 축은 `Brave`, `Cautious`, `SurvivalInstinct`, social 축은 `Obedient`, `Defiant`, `Comradely`다.
- `CombatAptitude`의 초기 enum은 `Balanced`, `Defensive`, `Offensive`다.
- 공격 속도 100은 중립이고 무기가 base interval을 소유한다.
- specialties는 문서상 미래 후보로만 유지하고 실제 production 계약에는 추가하지 않는다.
- compatibility 없이 구형 필수 계약을 complete cutover한다.

승인 근거는 Master Phase 1, 난이도 감사의 `Approved Policies`, 직원 도메인 정책 논의의 2026-07-18 사용자 결정을 따른다. 실제 구현 전 해당 기록을 직접 읽고 현재 canonical/runtime과 구분한다.

## 미결정 정책

없음. 2026-07-18 사용자가 EXPERIMENT_NOTES의 12명 provisional personality 배정안을 승인했다.

기존 UUID-derived 단일 trait는 authored personality로 보존하지 않고 제거한다. risk/social에는 neutral 또는 unassigned 값을 추가하지 않는다.

다음 값은 기존 gameplay를 보존하는 비최종 migration으로 제안할 수 있다.

- base stats: 현재 공통값 max HP 100, attack 10, defense 0, magic resist 0
- `basic_attack_speed_percent`: 100
- `CombatAptitude`: 전 직원 `Balanced`
- starter eligibility/loadout: 현재 starter/recruitment 파일 membership과 현재 loadout을 그대로 authored 값으로 이동

직원별 최종 balance, aptitude와 personality는 후속 authoring에서 live definition 값으로 변경할 수 있다. 이번 Goal은 승인된 provisional 값을 명시 작성하고 gameplay modifier는 추가하지 않는다.

## 범위

- 통합 `EmployeeDefinition` schema/catalog와 strict validation/loading
- starter eligibility를 포함한 시작·채용 후보 source 통합
- stable definition identity와 roster run UUID 분리
- role/background/initial loadout/Profile projection
- authored personality 두 축과 `CombatAptitude`
- 직접 authored base combat stats
- `basic_attack_speed_percent`와 weapon base interval 이후의 적용 pipeline
- candidate/roster typed DTO, exact serialization, game_server transport
- 접근 가능한 Unity starter/recruitment/roster reader와 presentation/test migration
- 관련 fixture/mock/test 및 책임에 가까운 canonical 문서 갱신
- 구형 candidate DB/schema, UUID-derived personality와 legacy consumer 제거

## 명시적 비범위

- RecruitmentQueue, active recruitment session, reroll/dedupe/checkpoint ownership 구현 — Phase 2
- Run HP 제거, physical damage/trauma, eligibility, death/treatment lifecycle — Phase 3
- trust score/band, semantic memory와 Psychology persistence — Phase 4
- specialties production 추가
- 장비 제한 또는 CombatAptitude 불일치 페널티
- 직원별 최종 밸런스와 최종 personality/aptitude 콘텐츠 authoring
- Dice System, employee-check Event와 trust high-risk/dialogue 콘텐츠
- account memorial/legacy, 별도 durable save 정책
- 관련 없는 refactor 또는 dirty 파일 정리

## 구현 범위

정책 결정 후 다음 범위를 complete cutover한다.

- `employee.rs`/`data/employee_data.rs`: definition/profile/personality/aptitude/base stat 타입과 roster definition identity
- `data/live_ron.rs`, `data/mod.rs`, `data/validation.rs`: 하나의 embedded employee catalog, strict validation과 loadout reference 검증
- `game_resources/data/employees`: 두 candidate RON을 하나의 definition RON으로 교체
- `world.rs`, helpers/map_content/headquarters/resources state: starter eligibility와 recruitment filter가 같은 catalog를 소비하고 Employee가 definition ID를 보존
- `battle/types.rs`와 `battle/stat_pipeline.rs`: authored base stats 및 weapon base interval 뒤 speed percent 공식
- `behavior.rs`/`world/snapshot.rs`: candidate/roster typed identity/Profile DTO와 legacy trust trait projection 제거
- Core/world/live loading fixture와 tests, game_server fixture/tests
- Unity starter/recruitment/roster reader, mock/contract tests와 `docs/unity_core_contract.md`
- `docs/game_rulebook.md`, `docs/data_loading_contract.md` 및 필요 시 `docs/core_runtime_contract.md`

Phase 2 queue ownership과 Phase 3 condition fields는 변경하지 않는다.

## 사용자-visible 완료 조건

- starter, recruitment와 roster가 같은 definition identity와 Profile을 표시한다.
- duplicate definition ID와 필수 Profile 누락을 production startup에서 명시적으로 거부한다.
- roster 합류 전 authored identity와 합류 후 run UUID가 혼동되지 않고 DTO/Unity에서 각각 올바르게 사용된다.
- 각 직원의 두 personality 축과 `CombatAptitude`가 authored definition에서 읽히며 UUID/trust에서 추론되지 않는다.
- 직원 기본 stats와 공격 속도가 authored data에서 로드되고 승인된 공식으로 무기 기본 공격 주기에 적용된다.
- Unity는 legacy candidate/Profile shape나 UUID-derived personality를 사용하지 않는다.
- specialties, 장비 제한/페널티 또는 승인되지 않은 compatibility가 production 계약에 추가되지 않는다.
- 구형 candidate DB/schema/fixture/mock/test/document가 현재 Goal 경계에서 제거되고 canonical 문서가 실제 계약과 일치한다.

## DTO Migration

필요. candidate와 roster의 identity/Profile shape를 actual typed Core producer, game_server serialization과 접근 가능한 Unity 소비자까지 한 번에 complete cutover한다. `employee_definition_id`를 authored identity로, `uuid`를 roster run identity로 명시하며 Profile에는 role/background, personality, combat aptitude, authored base stats와 speed percent를 제공한다. exact nesting/field shape는 승인 의미를 바꾸지 않는 typed mapping 세부사항으로 구현 전에 EXPERIMENT_NOTES에 기록한다.

## Data 또는 Live RON Migration

필요. `employees/starter_candidates.ron`과 `employees/recruitment_candidates.ron`을 하나의 strict EmployeeDefinition catalog로 전환하고 production embedded source도 단일 파일만 읽도록 바꾼다. 구형 두 schema/file fallback은 두지 않는다.

## Save Migration

현재 Core checkpoint/retreat는 in-memory `EmployeeRoster` clone이므로 `Employee`에 definition ID를 추가하면 함께 보존된다. 별도 serde save/database consumer는 발견하지 못했다. 접근할 수 없는 외부 durable save는 남은 위험이며, 배포 save compatibility 의미가 발견되면 임의로 결정하지 않는다.

## 필수 Focused 검증

최소한 다음 계약을 직접 증명해야 한다.

- live unified employee catalog 정상 loading
- duplicate ID, 필수 Profile 누락과 invalid percent startup rejection
- starter/recruitment/roster가 같은 definition을 참조하는 identity test
- base stat과 exact attack interval 공식 test
- actual Core producer의 exact candidate/roster JSON shape
- actual Core JSON을 소비하는 Unity reader/contract test
- legacy candidate catalog와 UUID trait symbol/schema 제거 검색

## 필수 Broad 검증

변경 blast radius에 맞는 Core `cargo test --all-targets`, live RON loading, game_server `cargo test --all-targets`/`cargo check`와 Unity EditMode 또는 접근 가능한 대체 runner를 포함한다. production 변경 전 dirty baseline을 확보하고 실제 test 수와 기존 실패를 기록한다.

## 생략할 검증과 근거

- Battle condition/result/retreat 상세 흐름은 Phase 3 책임이므로 Profile/stat pipeline과 definition identity 보존에 직접 영향을 주지 않는 테스트는 생략할 수 있다.
- game_server 별도 mapping 검증은 현재 transport가 Core typed snapshot을 그대로 serialize한다는 근거가 있으나, 실제 server fixture compile/test는 DTO 소비자이므로 생략하지 않는다.
- 외부 durable save/database와 workspace 밖 DTO 소비자는 접근할 수 없어 검증하지 못한다. 접근 가능한 Core/game_server/Unity 계약의 complete cutover와 no-compatibility 검색으로 현재 범위를 검증한다.

## 중단 조건

- 하나의 EmployeeDefinition/Profile 경계를 넘어 독립적인 source-of-truth 변경이 필수로 발견됨
- 직원별 authored 값, DTO, live schema, save, UX 또는 compatibility에 승인되지 않은 사용자-visible 결정이 필요함
- 필수 외부 consumer/runner/repository에 접근할 수 없어 완료 조건을 검증할 수 없음
- 기존 dirty 사용자 변경과 안전하게 병합할 수 없음
- 선행 Master 범위나 승인 정책을 확인하는 필수 기록에 접근할 수 없음

근거에 따라 `scope_decision_required`, `policy_decision_required` 또는 `externally_blocked`로 전환하고 영향받는 수정/migration을 중단한다.

## Compatibility

승인된 compatibility 없음. dual-read/write, fallback, schema alias 또는 unknown-value fallback을 구현하지 않는다. 필요성이 새로 발견되면 사용자 승인 전 구현을 중단한다.

## 완료 증거

- production authored source는 `game_resources/data/employees/definitions.ron` 하나이며 12개 definition, starter 6명/recruitment 6명을 포함한다. 구형 두 RON은 제거됐다.
- `EmployeeDefinitionDatabase`가 unique/nonempty ID, required Profile, starter 수, max HP, 양수 speed percent와 loadout 참조를 strict validation한다.
- roster `Employee`는 run-local `uuid`, authored `employee_definition_id`와 immutable authored Profile을 함께 보존한다. checkpoint/retreat의 roster clone도 이 값을 함께 보존한다.
- `EmployeeTrustState`의 UUID-derived `EmployeeTrait`, `new_for_employee`와 `trust.traits` projection을 제거했다.
- authored base stats가 직원 health/combat profile을 만들고 `UnitCombatProfile`이 weapon base interval에 speed percent ceiling 공식을 적용한다. non-employee profile은 100% 중립이다.
- starter/recruitment/roster와 선택·채용 command/result가 `employee_definition_id(s)` 계약을 사용한다. game_server는 pass-through하고 Unity reader/presentation은 Profile을 표시한다.
- Rust generator가 실제 Core producer JSON fixture를 만들고, Unity가 같은 fixture의 starter/recruitment/roster shape를 소비한다.
- canonical gameplay/runtime/data/server/Unity 문서와 접근 가능한 probe를 새 계약으로 갱신했다.
- 최종 Core 전체 759 tests, game_server 30 tests, Unity EditMode 240 tests가 통과했다. formatter/check, fixture regeneration comparison, Python probe syntax와 scoped diff/legacy 검색도 통과했다.
- 기존 baseline failure는 없었고 최종 새 failure도 없다. 광범위한 기존 dirty worktree는 reset/restore/stash 없이 보존했다.
