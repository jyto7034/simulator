# Employee Definition Profile Contract Experiments

신규 Goal 초기화 전에는 첨부 실행 지침을 읽고 Goal 문서 존재 여부·관련 저장소 루트·초기 dirty 상태만 확인했다. 이후 실제 조사, 정책 결정, baseline, 구현과 검증을 아래에 시간순으로 기록한다.

> 경로 정리: 당시 `docs/tmp/employee_domain_policy_discussion_2026-07-17.md`였던 정책 입력은
> 현재 `docs/decisions/employee_domain_policy_2026-07-17.md`에 보존된다.
> 아래 읽기 명령의 경로만 현재 위치로 정규화했다.

## 2026-07-18 — Child 직접 의존성과 현재 계약 조사

### 검증하려던 가설

- Master Phase 1의 승인 경계와 선행 조건을 확인할 수 있다.
- 현재 Core/live data/server/Unity 계약과 dirty overlap을 식별할 수 있다.
- production 구현 전에 추가 사용자-visible 정책 결정이 필요한지 판단할 수 있다.

### 수행한 조사 또는 검증

- Child 문서 3개를 처음부터 끝까지 읽었다.
- Master Phase 1, 난이도 감사와 직원 정책 논의의 Profile 결정을 읽었다.
- Core definition/loader/roster/starter/recruitment/stat/DTO 경로를 검색하고 관련 파일을 읽었다.
- live employee RON 2개와 embedded loader source를 확인했다.
- game_server Core snapshot transport와 fixture 소비를 확인했다.
- Unity starter/recruitment/roster reader, contract tests와 canonical 문서를 확인했다.
- 관련 파일의 기존 diff와 numstat을 확인했다.

### 변경한 내용

- production code/data/server/Unity/canonical 문서는 변경하지 않았다.
- 조사 근거에 맞춰 Child PLAN과 EXPERIMENT_NOTES를 갱신하고 Status를 `policy_decision_required`로 변경했다.

### 실행한 정확한 주요 명령

```bash
sed -n '1,10000p' docs/goals/employee_definition_profile_contract/PLAN.md
sed -n '1,10000p' docs/goals/employee_definition_profile_contract/EXPERIMENTS.md
sed -n '1,10000p' docs/goals/employee_definition_profile_contract/EXPERIMENT_NOTES.md
```

```bash
sed -n '150,200p' docs/goals/employee_domain_implementation_master/PLAN.md
sed -n '65,155p' docs/goals/employee_domain_implementation_difficulty_audit/PLAN.md
sed -n '1,540p' docs/decisions/employee_domain_policy_2026-07-17.md
sed -n '540,625p' docs/decisions/employee_domain_policy_2026-07-17.md
```

```bash
rg --files src tests docs | rg 'employee|starter|recruit|snapshot|stat_pipeline|battle/types|game_rulebook|data_loading|core_runtime'
rg -n "StarterEmployee|RecruitmentEmployee|EmployeeCandidate|EmployeeTrait|new_for_employee|apply_weapon_profile|employee_default|EmployeeSnapshotDto|StarterSelection|Recruitment" src tests
```

```bash
find /mnt/f/work/simulator/game_resources/data/employees -maxdepth 2 -type f -print -exec sed -n '1,260p' {} \;
rg -n "employee|starter|recruit" /mnt/f/work/simulator/game_resources/data/bundle.ron /mnt/f/work/simulator/game_resources/data -g '*.ron'
```

```bash
sed -n '1,320p' src/game/employee.rs
sed -n '1,220p' src/game/data/employee_data.rs
sed -n '1,220p' src/game/employee_trust.rs
sed -n '300,540p' src/game/battle/types.rs
sed -n '1,260p' src/game/battle/stat_pipeline.rs
sed -n '220,380p' src/game/data/live_ron.rs
sed -n '140,380p' src/game/data/mod.rs
sed -n '220,285p' src/game/data/validation.rs
```

```bash
rg -n "starter_employee_data|recruitment_employee_data|starter_candidates =|initialize_selected_starter_employees|validate_starter_employee_selection|from_starter_candidate|recruitment_candidates_for_node" src/game -g '*.rs'
rg -n "Employee::new\(|from_starter_candidate\(|employee\.name" src tests -g '*.rs'
rg -n "employee\.trust\.traits|EmployeeTrait|new_for_employee" src tests /mnt/f/work/simulator/game_server -g '*.rs'
```

```bash
rg -n "starter_candidates|recruitment_candidates|StarterEmployeeCandidate|employee_definition|combat_aptitude|risk_disposition|social_disposition|basic_attack_speed|traits" '/mnt/f/unity projects/ark/Assets/Scripts' '/mnt/f/unity projects/ark/Assets/Tests' '/mnt/f/unity projects/ark/docs/unity_core_contract.md'
sed -n '1,280p' '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/Reading/StarterSelectionModel.cs'
sed -n '1,360p' '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/Reading/RosterModel.cs'
```

```bash
git -C /mnt/f/work/simulator diff --numstat -- core_enhanced/src/game/employee.rs core_enhanced/src/game/data/employee_data.rs core_enhanced/src/game/employee_trust.rs core_enhanced/src/game/battle/types.rs core_enhanced/src/game/battle/stat_pipeline.rs core_enhanced/src/game/behavior.rs core_enhanced/src/game/world/snapshot.rs core_enhanced/src/game/world.rs core_enhanced/src/game/world/state.rs core_enhanced/src/game/world/map_content.rs core_enhanced/src/game/world/headquarters.rs core_enhanced/src/game/resources/selection.rs
git -C '/mnt/f/unity projects/ark' diff --numstat -- Assets/Scripts/ArkClient/Runtime/Core/Reading/RosterModel.cs Assets/Scripts/ArkClient/Runtime/Core/Reading/StarterSelectionModel.cs Assets/Scripts/ArkClient/Runtime/Core/Reading/SelectedEventModel.cs Assets/Tests/EditMode/ArkClientContractTests.cs Assets/Scripts/ArkClient/Runtime/Mock/ArkMockSnapshots.cs docs/unity_core_contract.md
```

### 실제 실행된 test 수

해당 없음. 정책 결정 전 read-only 조사이며 test runner를 실행하지 않았다.

### pass/fail/skip 수

- pass: runner가 수치를 보고하지 않음
- fail: runner가 수치를 보고하지 않음
- skip: runner가 수치를 보고하지 않음

### 실제 결과

- Master Phase 1은 선행 Phase 없이 시작 가능하고, one-catalog/Profile/stat/DTO/Unity cutover를 소유한다.
- 12개 live definition 모두 personality/aptitude/base stat/speed 필드가 없다.
- 기존 combat 수치는 공통값이어서 새 explicit fields로 행동 보존 migration이 가능하다.
- 기존 personality는 run UUID-derived single trait라 authored two-axis 값으로 보존할 수 없다.
- 12명의 two-axis assignment는 사용자-visible 캐릭터 설정을 새로 정하므로 승인 없이 구현할 수 없다.
- Core/server/Unity와 canonical 문서가 모두 구형 candidate/Profile 계약의 접근 가능한 소비자다.
- Goal 대상 파일 다수가 기존 사용자 변경과 겹치므로 current diff를 보존한 병합이 필요하다.

### 실패 원인

- 구현 입력이 부족하다. 각 직원의 exact authored personality 조합 또는 provisional authoring 위임이 없다.

### 되돌리거나 수정한 내용

없음. production 변경을 시작하지 않았다.

### 결과에 따른 판단

영향받는 code/data migration과 baseline 실행을 중단하고 Status를 `policy_decision_required`로 변경한다. 정책과 독립적인 현재 경로 조사는 완료했다.

## 2026-07-18 — Provisional personality 정책안 준비

### 검증하려던 가설

- 사용자가 12개 조합을 처음부터 작성하지 않아도 구체적인 비최종 제안을 검토해 정책 결정을 내릴 수 있다.
- 제안만 Goal 기록에 남기고 production data에는 적용하지 않을 수 있다.

### 수행한 조사 또는 검증

- 재개 규칙에 따라 Child 문서 3개를 다시 처음부터 끝까지 읽었다.
- 앞선 조사에서 확인한 12개 live definition의 ID, 이름과 background를 기준으로 두 축 배정안을 작성했다.
- 제안이 gameplay modifier, aptitude, stat 또는 specialty 결정을 추가하지 않는지 검토했다.

### 변경한 내용

- EXPERIMENT_NOTES의 정책 질문에 12명 provisional personality 배정표와 적용 제한을 추가했다.
- production code, live data, server, Unity와 canonical 문서는 변경하지 않았다.

### 실행한 정확한 명령

```bash
sed -n '1,10000p' docs/goals/employee_definition_profile_contract/PLAN.md
sed -n '1,10000p' docs/goals/employee_definition_profile_contract/EXPERIMENTS.md
sed -n '1,10000p' docs/goals/employee_definition_profile_contract/EXPERIMENT_NOTES.md
```

### 실제 실행된 test 수

해당 없음. 정책 제안 문서화이며 test runner를 실행하지 않았다.

### pass/fail/skip 수

- pass: runner가 수치를 보고하지 않음
- fail: runner가 수치를 보고하지 않음
- skip: runner가 수치를 보고하지 않음

### 실제 결과

- 12개 definition 모두에 exact risk/social 조합을 제안했다.
- social disposition은 각 4명으로 분산했다.
- 제안은 표시/향후 typed 입력만 소유하며 현재 gameplay 수치를 바꾸지 않는다.

### 실패 원인

없음. 다만 사용자 승인이 없으므로 정책 결정 자체는 완료되지 않았다.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

Status는 `policy_decision_required`를 유지한다. 사용자가 표를 승인하거나 수정값을 제공하면 `active`로 복귀해 baseline과 구현을 시작할 수 있다.

## 2026-07-18 — Provisional personality 정책 승인

### 검증하려던 가설

- 사용자가 제안한 12명 personality 조합을 명시적으로 승인했는지 확인할 수 있다.

### 수행한 조사 또는 검증

- 사용자 응답 `승인`을 현재 정책 질문과 대조했다.

### 변경한 내용

- PLAN Status를 `active`로 복귀했다.
- PLAN의 미결정 정책을 없음으로 갱신했다.
- EXPERIMENT_NOTES의 12명 표를 승인된 provisional live authoring 값으로 기록했다.
- production 파일은 아직 변경하지 않았다.

### 실행한 정확한 명령

해당 없음. 사용자 대화의 명시적 승인 기록을 확인했다.

### 실제 실행된 test 수

해당 없음. 정책 승인 기록 단계다.

### pass/fail/skip 수

- pass: runner가 수치를 보고하지 않음
- fail: runner가 수치를 보고하지 않음
- skip: runner가 수치를 보고하지 않음

### 실제 결과

- 12명 provisional personality 배정안이 승인됐다.
- 구현을 막던 정책 질문이 해결됐다.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

Goal 범위 안에서 baseline, 구현과 검증을 안전하게 계속할 수 있다.

## 2026-07-18 — Production 변경 전 dirty baseline

### 검증하려던 가설

- 현재 dirty worktree에서 Core, live-loading 경로, game_server와 Unity 소비자의 변경 전 상태를 재현할 수 있다.
- 이후 변경으로 추가된 실패를 기존 실패와 구분할 수 있다.

### 수행한 실험 또는 검증

- Core formatting과 전체 Rust test baseline을 실행했다.
- 별도 target directory를 사용해 employee-data focused test와 game_server compile/test baseline을 실행했다.
- 현재 열려 있는 Unity 6000.3.4f1 프로젝트에서 전체 `ArkClient.EditModeTests` baseline을 실행했다.

### 변경한 내용

- production 파일은 변경하지 않았다.
- 이 baseline 기록과 관련 PLAN/EXPERIMENT_NOTES만 갱신했다.

### 실행한 정확한 명령 또는 test runner job

```bash
cargo fmt --all -- --check
CARGO_TARGET_DIR=/tmp/employee_profile_target cargo test employee_data --lib
CARGO_TARGET_DIR=/tmp/employee_profile_target cargo test --all-targets
```

Core 명령은 `/mnt/f/work/simulator/core_enhanced`에서 실행했다.

```bash
CARGO_TARGET_DIR=/tmp/employee_profile_target cargo test --all-targets
CARGO_TARGET_DIR=/tmp/employee_profile_target cargo check
```

game_server 명령은 `/mnt/f/work/simulator/game_server`에서 실행했다.

Unity test runner:

- project: `ark@b49d32f6f2f70bbe`
- Unity: `6000.3.4f1`
- job: `751451a399e1473a935b25f16cc6ffec`
- mode: EditMode
- test filter: `ArkClient.EditModeTests`

### 실제 실행된 test 수와 pass/fail/skip 수

- Core `cargo fmt --all -- --check`: test runner 아님, exit 0
- Core focused `cargo test employee_data --lib`: 8 pass, 0 fail, 0 ignored, 703 filtered out
- Core `cargo test --all-targets`: 총 755 pass, 0 fail, 0 ignored
  - lib 711
  - main 및 generator binary 6개 각각 0
  - `live_item` 3
  - `live_skill_catalog` 4
  - `ron_loading` 20
  - `skill_runtime` 13
  - `skill_test_suite` 4
- game_server `cargo test --all-targets`: 총 30 pass, 0 fail, 0 ignored; main 및 example binary 3개는 각각 0
- game_server `cargo check`: test runner 아님, exit 0
- Unity `ArkClient.EditModeTests`: 238 pass, 0 fail, 0 skip, 총 238

### 실제 결과

- 관련 baseline은 모두 통과했다.
- 변경 전 existing failure는 확인되지 않았다.
- Unity runner가 접근 가능하며 실제 reader/contract 회귀 검증에 사용할 수 있다.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

현재 dirty baseline을 확보했으므로 기존 사용자 변경을 보존하며 Phase 1 production 구현을 시작할 수 있다.

## 2026-07-18 — Unified EmployeeDefinition/Profile complete cutover

### 검증하려던 가설

- 하나의 authored definition source가 starter, recruitment와 roster에 같은 identity/Profile을 제공할 수 있다.
- authored base stats와 speed percent를 기존 enemy semantics와 Phase 2 recruitment 동작을 바꾸지 않고 직원 전투 pipeline에 적용할 수 있다.
- Core producer, game_server와 Unity가 compatibility alias 없이 같은 새 wire 계약으로 전환될 수 있다.

### 수행한 구현과 조사

- `EmployeeDefinition`, Profile/personality/aptitude/base-stat/loadout typed schema와 단일 database를 구현했다.
- live employee 두 RON을 12개 definition의 `definitions.ron` 하나로 migration했다.
- roster가 authored ID/Profile을 보존하도록 변경하고 UUID-derived trust trait authority/projection을 제거했다.
- starter/recruitment source, 선택·채용 command/result, roster DTO와 game_server fixture를 complete cutover했다.
- weapon base interval 뒤 speed percent ceiling 공식을 적용하고 non-employee profile은 100% 중립으로 유지했다.
- Unity starter/recruitment/roster reader, command bus, presentation, mock/test와 probe를 새 계약으로 전환했다.
- 실제 Core runtime snapshot에서 employee candidate/roster JSON을 뽑는 Rust generator와 Unity fixture 소비 test를 추가했다.
- gameplay/runtime/data/server/Unity canonical 문서를 실제 계약으로 갱신했다.

### 변경한 내용

- production Core, live data, game_server, Unity, fixture/test/probe와 canonical 문서를 승인 범위 안에서 변경했다.
- RecruitmentQueue, condition lifecycle, trust score/memory semantics와 specialties는 변경하지 않았다.

### 구현 중 실행한 정확한 주요 명령과 runner job

```bash
CARGO_TARGET_DIR=/tmp/employee_profile_target cargo check --lib
CARGO_TARGET_DIR=/tmp/employee_profile_target cargo check --tests
CARGO_TARGET_DIR=/tmp/employee_profile_target cargo check --all-targets
```

```bash
CARGO_TARGET_DIR=/tmp/employee_profile_target cargo check --all-targets
```

두 번째 명령은 `/mnt/f/work/simulator/game_server`에서 실행했다.

```bash
CARGO_TARGET_DIR=/tmp/employee_profile_target cargo test employee_data --lib
CARGO_TARGET_DIR=/tmp/employee_profile_target cargo test employee_attack_speed_scales_default_and_weapon_intervals_with_ceiling --lib
CARGO_TARGET_DIR=/tmp/employee_profile_target cargo test employee_definition_profile_json_preserves_authored_and_run_identities --lib
CARGO_TARGET_DIR=/tmp/employee_profile_target cargo test --test ron_loading load_game_data_from_ron_reads_unified_employee_definitions
```

Unity focused EditMode job:

- job: `9194364390624e199226a86e9b36069d`
- tests: Rust-generated fixture, employee command identity, starter Profile, roster Profile, recruitment Profile 5개

### 실제 실행된 test 수와 pass/fail/skip 수

- employee-data focused 최종: 10 pass, 0 fail, 0 ignored, 705 filtered out
- attack-speed focused: 1 pass, 0 fail, 0 ignored, 712 filtered out였으며 이후 test 추가 뒤 같은 test를 전체 suite에서 재검증했다.
- exact candidate/roster JSON focused: 1 pass, 0 fail, 0 ignored, 712 filtered out였으며 이후 전체 suite에서 재검증했다.
- live unified catalog focused: 1 pass, 0 fail, 0 ignored, 19 filtered out
- Unity focused: 5 pass, 0 fail, 0 skip
- compile/check 명령: test runner 아님, 모두 exit 0

### 실제 결과

- 12명의 approved provisional personality와 behavior-preserving stats/aptitude/speed가 strict live data로 로드된다.
- starter/recruitment/roster가 `employee_definition_id`와 같은 nested Profile을 사용하고 roster만 별도 run UUID를 가진다.
- missing Profile, duplicate ID와 zero speed percent가 publication 전에 거부된다.
- 125% 직원은 기본 1500ms를 1200ms로, weapon 1001ms를 ceiling 801ms로 적용한다.
- Unity는 Profile summary를 starter, HQ recruitment와 roster loadout detail에 표시한다.

### 실패 원인과 수정

- 초기 compile에서 구형 fixture/type/field와 u32/u64 interval 차이가 드러났다. 모든 접근 가능한 consumer를 새 type/field로 migration하고 interval 공식을 runtime의 u64와 맞췄다.
- 첫 최종 Unity 전체 job `d9ba78ed52e44fb6b53d0127b5d1ee14`는 240개 중 1개가 실패했다. 구현 오류가 아니라 HQ presentation test가 제거된 `Candidate ID` 문구를 기대했기 때문이다. 기대값을 `Definition ID`와 Profile summary로 교체했다.

### 되돌리거나 수정한 내용

- compatibility alias나 fallback을 추가하지 않았다.
- Unity 실패 기대값만 승인된 새 의미로 수정하고 전체 suite를 재실행했다.

### 결과에 따른 판단

구현 경계는 승인된 Phase 1 안에 있고 focused 계약이 성립한다. 전체 broad 검증과 legacy/dirty audit으로 완료 여부를 판단한다.

## 2026-07-18 — Final broad validation and completion evidence

### 검증하려던 가설

- 최종 변경이 baseline에 없던 회귀를 만들지 않는다.
- actual Core fixture, server pass-through, Unity 소비자와 canonical 문서가 같은 계약을 사용한다.
- 제거 대상 legacy와 승인되지 않은 specialties/compatibility가 production 경계에 남지 않는다.

### 수행한 실험 또는 검증

- Core와 game_server 전체 Rust suite/compile을 최종 상태에서 실행했다.
- Rust generator로 checked-in Unity fixture가 actual typed serialization과 같은지 비교했다.
- Unity 전체 EditMode suite를 최종 상태에서 상세 결과로 실행했다.
- 수정한 Python probe 3개의 AST syntax를 검사했다.
- production legacy symbol/schema, old live files, specialties와 scoped whitespace error를 검색했다.

### 변경한 내용

- 검증 결과에 따른 production 추가 변경은 없었다.
- Goal 실행 문서만 최종 증거에 맞게 갱신했다.

### 실행한 정확한 명령 또는 test runner job

```bash
cargo fmt --all -- --check
CARGO_TARGET_DIR=/tmp/employee_profile_target cargo test --all-targets -- --quiet
```

```bash
CARGO_TARGET_DIR=/tmp/employee_profile_target cargo test --all-targets -- --quiet
CARGO_TARGET_DIR=/tmp/employee_profile_target cargo check
CARGO_TARGET_DIR=/tmp/employee_profile_target cargo run --example generate_employee_profile_contract_fixture -- '/mnt/f/unity projects/ark/Assets/Tests/Fixtures/employee_profile_contract.json'
```

두 번째 묶음은 `/mnt/f/work/simulator/game_server`에서 실행했다.

```bash
python3 -c 'import ast, pathlib; paths=[pathlib.Path("/mnt/f/unity projects/ark/docs/probe/unity_admin_debug_command_probe.py"), pathlib.Path("/mnt/f/unity projects/ark/docs/probe/unity_savepoint_checkpoint_probe.py"), pathlib.Path("/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py")]; [ast.parse(path.read_text(encoding="utf-8"), filename=str(path)) for path in paths]; print("3 probe scripts parsed")'
```

Unity final EditMode job:

- job: `a652e88490584eafba40f73ceb8831c0`
- assembly: `ArkClient.EditModeTests`
- include_details: true

Legacy/diff audit의 정확한 검색 대상:

```bash
rg -n "StarterEmployeeCandidate|StarterEmployeeLoadout|StarterEmployeeCandidateDatabase|RecruitmentEmployeeCandidateDatabase|starter_employee_data|recruitment_employee_data|from_starter_candidate|EmployeeTrait|new_for_employee|trust\.traits|serde\(.*alias" src -g '*.rs'
rg -n "StarterEmployeeCandidate|StarterEmployeeLoadout|StarterEmployeeCandidateDatabase|RecruitmentEmployeeCandidateDatabase|starter_employee_data|recruitment_employee_data|candidate_ids|candidate_id|selected_candidate_ids" /mnt/f/work/simulator/game_server/src /mnt/f/work/simulator/game_server/examples -g '*.rs'
git -C /mnt/f/work/simulator diff --check -- <Goal scoped paths>
git -C '/mnt/f/unity projects/ark' diff --check -- <Goal scoped paths>
```

### 실제 실행된 test 수와 pass/fail/skip 수

- Core lib: 715 pass, 0 fail, 0 ignored
- Core binary targets: 6 targets, 각 0 tests
- Core integration: 3 + 4 + 20 + 13 + 4 = 44 pass, 0 fail, 0 ignored
- Core 총계: 759 pass, 0 fail, 0 ignored
- game_server lib: 30 pass, 0 fail, 0 ignored
- game_server main/examples: 4 targets, 각 0 tests
- Unity EditMode: 240 pass, 0 fail, 0 skip
- formatter/check/generator comparison/Python syntax/diff check: test runner 수치 없음, 모두 exit/status success

### 실제 결과

- baseline의 Core 755, server 30, Unity 238에서 새 focused tests가 추가된 최종 Core 759, server 30, Unity 240이 모두 통과했다.
- actual Core fixture regeneration comparison이 일치했다.
- production Core/server에는 legacy employee candidate/UUID-trait symbol이 없고 old live RON 두 파일도 존재하지 않는다.
- Unity employee readers/commands는 `employee_definition_id(s)`만 사용한다. old field 이름은 compatibility를 거부하는 negative test/문서 설명에만 남는다.
- specialties production field가 없고 scoped `git diff --check`가 통과했다.
- 기존 광범위한 dirty worktree는 유지됐다.

### 실패 원인

최종 검증 실패 없음. 기존 baseline failure도 없음.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

모든 사용자-visible 완료 조건과 필수 focused/broad 검증에 실제 증거가 연결됐다. Status를 `complete`로 변경한다.

## 2026-07-18 — Master 재개 시 현재 상태 재검증과 문서 기록 복구

### 검증하려던 가설

- Phase 1의 `complete` 증거가 현재 dirty worktree에서도 그대로 성립한다.
- 이전 실행 기록을 손상 없이 보존하고 Master Phase 전환 근거로 사용할 수 있다.

### 수행한 조사 및 검증

- current runtime/live data/DTO/server/Unity/canonical 계약과 legacy symbol을 다시 조사했다.
- Core focused tests, 전체 Core/server suite, Rust-generated Unity fixture comparison을 재실행했다.
- 열린 Unity editor의 MCP runner로 focused EditMode, 전체 EditMode와 전체 PlayMode를 실행했다.
- 기존 Codex session log의 9개 `apply_patch` 기록을 순서대로 `/tmp`에 재적용해 이 Goal 문서 3개의 정확한 최종본을 복구하고 전부 다시 읽었다.

### 실행한 정확한 주요 명령과 runner job

```bash
cargo test employee_definition -- --nocapture
cargo test game::data::employee_data::tests -- --nocapture
cargo test employee_attack_speed_scales_default_and_weapon_intervals_with_ceiling -- --nocapture
cargo test headquarters_contact_enters_safe_choice_state_with_recruitment_candidates -- --nocapture
cargo test bundle_validation_returns_typed_starter_loadout_reference_errors -- --nocapture
cargo test --all-targets --quiet
```

```bash
cargo test --all-targets --quiet
cargo run --example generate_employee_profile_contract_fixture -- '/mnt/f/unity projects/ark/Assets/Tests/Fixtures/employee_profile_contract.json'
```

두 번째 묶음은 `/mnt/f/work/simulator/game_server`에서 실행했다.

Unity jobs:

- focused EditMode: `3c602826a40043ed985e82853dc19975`
- full EditMode: `3acc645b59ae451f9919f6ee58144189`
- full PlayMode: `a68e2b1d55aa460489ab62d019cd2421`

### 실제 실행된 test 수와 pass/fail/skip 수

- Core 전체: 759 pass, 0 fail, 0 ignored
- game_server 전체: 30 pass, 0 fail, 0 ignored
- Unity focused EditMode: 4 pass, 0 fail, 0 skip
- Unity 전체 EditMode: 240 pass, 0 fail, 0 skip
- Unity 전체 PlayMode: 57 pass, 0 fail, 4 explicit skip, 총 61
- Rust fixture comparison: 일치, test runner 수치 없음

### 실제 결과

- Phase 1 source-of-truth, strict loading, identity/Profile wire, attack interval과 Unity reader 계약은 현재 상태에서도 완료 증거와 일치했다.
- PlayMode의 4 skip은 local game_server/admin server를 요구하는 explicit tests이며 Phase 1 read-model/fixture 계약 실패가 아니다.
- production 추가 변경은 필요하지 않았다.

### 기록 복구 사고와 수정

- Master 재개 초기에 `test ! -e ... && git status ...`의 빈 출력을 성공으로 오판해 기존 Child 문서가 없고 worktree가 clean하다고 잘못 판단했다.
- 그 결과 기존 complete 문서 3개를 새 문서로 덮어썼다. Production code/data/server/Unity 파일은 이 과정에서 수정하지 않았다.
- 이전 세션 JSONL의 9개 실제 patch를 순서대로 재생해 최종 문서를 정확히 복구했고, 이후 이 절만 추가했다.
- 오류를 숨기지 않고 Master와 EXPERIMENT_NOTES에도 원인·복구 결과를 기록한다.

### 결과에 따른 판단

Phase 1 `complete`는 현재 증거로 재확인됐다. Master completion review 뒤 Phase 2로 전환할 수 있다.
