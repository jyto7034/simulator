# Maintenance Action Capability Projection Experiments

Status: complete

Implementation complete: true

Created: 2026-08-03

이 문서는 실제로 수행한 조사, 실험, 변경과 검증 결과만 시간순으로 기록한다. 실행하지 않은 명령이나
test를 완료 증거처럼 기록하지 않는다.

## 2026-08-03 — 신규 Goal 초기화

### 검증하려던 가설

- 사용자 요청과 기존 승인 계약에 Goal 이름, 범위, 사용자-visible 완료 조건과 대상 저장소가 충분히
  주어져 있다.
- `maintenance_action_capability_projection` 경로가 기존 Goal과 충돌하지 않는다.
- Production 조사 전에 Core/server와 Unity 저장소의 현재 dirty 상태를 확보할 수 있다.

### 수행한 확인

- 사용자와 앞선 대화에서 Goal 이름을 `maintenance_action_capability_projection`으로 합의했다.
- Goal 경로와 필수 문서 존재 여부를 확인했다.
- workspace와 Unity repository root 및 `git status --short`를 확인했다.
- 지침에 따라 repository-wide production 코드 검색, runtime 조사, test와 formatter는 수행하지 않았다.

### 실행한 정확한 명령

```bash
test ! -e core_enhanced/docs/goals/maintenance_action_capability_projection
git rev-parse --show-toplevel
git status --short
git -C '/mnt/f/unity projects/ark' rev-parse --show-toplevel
git -C '/mnt/f/unity projects/ark' status --short
mkdir -p core_enhanced/docs/goals/maintenance_action_capability_projection
```

### 실제 test 수와 결과

- Test runner를 실행하지 않았다.
- 실제 test 수: runner 실행 없음.
- Pass/fail/skip: 해당 없음.
- Goal 경로는 생성 전에 존재하지 않아 신규 Goal로 확인됐다.
- Workspace root는 `/mnt/f/work/simulator`, Unity root는 `/mnt/f/unity projects/ark`로 확인됐다.
- 두 저장소 모두 광범위한 tracked/untracked 기존 변경을 보고했다.

### 변경한 내용

- Goal 디렉터리를 생성했다.
- `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`를 초기화했다.
- Production runtime, live data, DTO, server, Unity, fixture, test와 canonical은 변경하지 않았다.

### 실패 원인

- 명령 실패 없음.
- Production baseline과 실제 defect 재현은 아직 수행하지 않았다.

### 되돌리거나 수정한 내용

- 없음.

### 결과에 따른 판단

- 신규 단일 Goal을 `active`로 초기화할 입력이 충분하다.
- Current runtime/call graph, 대상별 dirty overlap과 정확한 baseline은 discovery pending이다.
- 다음 production action 전에 필수 문서 세 개를 완독하고 current source 및 scoped diff를 조사해야 한다.

## 2026-08-03 — 초기 문서 완전성 검증

### 검증하려던 가설

- 필수 문서 세 개가 모두 존재하고 처음부터 끝까지 읽을 수 있다.
- `PLAN.md`가 지침의 필수 계획 항목을 포함하며 세 문서의 Status와 구현 상태가 서로 일치한다.
- 생성한 문서에 Git whitespace 오류가 없고 Goal 밖 파일은 이 검증에서 변경되지 않는다.

### 수행한 실험 또는 검증

- 세 파일의 존재 여부와 줄 수를 확인했다.
- Status, 구현 상태, Goal 이름과 모든 section heading을 검색했다.
- 세 문서를 처음부터 끝까지 읽고 승인 범위, discovery pending, migration, 검증 및 중단 조건의
  모순 여부를 검토했다.
- Scoped Git 상태를 확인했다.
- 허용 Status가 아니었던 `EXPERIMENT_NOTES.md`의 `active notes`를 `active`로 수정했다.

### 실행한 정확한 명령

```bash
test -f core_enhanced/docs/goals/maintenance_action_capability_projection/PLAN.md && test -f core_enhanced/docs/goals/maintenance_action_capability_projection/EXPERIMENTS.md && test -f core_enhanced/docs/goals/maintenance_action_capability_projection/EXPERIMENT_NOTES.md && wc -l core_enhanced/docs/goals/maintenance_action_capability_projection/PLAN.md core_enhanced/docs/goals/maintenance_action_capability_projection/EXPERIMENTS.md core_enhanced/docs/goals/maintenance_action_capability_projection/EXPERIMENT_NOTES.md && rg -n "^(Status:|Implementation complete:|Goal name:|## )" core_enhanced/docs/goals/maintenance_action_capability_projection/*.md && git diff --check -- core_enhanced/docs/goals/maintenance_action_capability_projection && git status --short -- core_enhanced/docs/goals/maintenance_action_capability_projection
sed -n '1,320p' core_enhanced/docs/goals/maintenance_action_capability_projection/PLAN.md
sed -n '1,180p' core_enhanced/docs/goals/maintenance_action_capability_projection/EXPERIMENTS.md
sed -n '1,220p' core_enhanced/docs/goals/maintenance_action_capability_projection/EXPERIMENT_NOTES.md
test -s core_enhanced/docs/goals/maintenance_action_capability_projection/PLAN.md && test -s core_enhanced/docs/goals/maintenance_action_capability_projection/EXPERIMENTS.md && test -s core_enhanced/docs/goals/maintenance_action_capability_projection/EXPERIMENT_NOTES.md && awk '/[ \t]+$/ { print FILENAME ":" FNR ":" $0; goal_doc_whitespace_error=1 } END { exit goal_doc_whitespace_error }' core_enhanced/docs/goals/maintenance_action_capability_projection/PLAN.md core_enhanced/docs/goals/maintenance_action_capability_projection/EXPERIMENTS.md core_enhanced/docs/goals/maintenance_action_capability_projection/EXPERIMENT_NOTES.md && rg -n "^Status: (active|scope_decision_required|policy_decision_required|externally_blocked|complete)$|^Implementation complete: false$|^Goal name: `maintenance_action_capability_projection`$" core_enhanced/docs/goals/maintenance_action_capability_projection/*.md && sed -n '60,180p' core_enhanced/docs/goals/maintenance_action_capability_projection/EXPERIMENTS.md && git status --short -- core_enhanced/docs/goals/maintenance_action_capability_projection
test -s core_enhanced/docs/goals/maintenance_action_capability_projection/PLAN.md && test -s core_enhanced/docs/goals/maintenance_action_capability_projection/EXPERIMENTS.md && test -s core_enhanced/docs/goals/maintenance_action_capability_projection/EXPERIMENT_NOTES.md && awk '/[ \t]+$/ { print FILENAME ":" FNR ":" $0; goal_doc_whitespace_error=1 } END { exit goal_doc_whitespace_error }' core_enhanced/docs/goals/maintenance_action_capability_projection/PLAN.md core_enhanced/docs/goals/maintenance_action_capability_projection/EXPERIMENTS.md core_enhanced/docs/goals/maintenance_action_capability_projection/EXPERIMENT_NOTES.md && rg -n '^Status: (active|scope_decision_required|policy_decision_required|externally_blocked|complete)$|^Implementation complete: false$' core_enhanced/docs/goals/maintenance_action_capability_projection/*.md && rg -n '^Goal name: `maintenance_action_capability_projection`$' core_enhanced/docs/goals/maintenance_action_capability_projection/PLAN.md && git status --short -- core_enhanced/docs/goals/maintenance_action_capability_projection
```

### 실제 test 수와 결과

- Test runner를 실행하지 않았다.
- 실제 test 수: runner 실행 없음.
- Pass/fail/skip: 해당 없음.
- 문서 줄 수는 검증 시점에 `PLAN.md` 266, `EXPERIMENTS.md` 67,
  `EXPERIMENT_NOTES.md` 125로 보고됐다.
- 필수 문서 세 개가 모두 존재했고 전부 읽을 수 있었다.
- 계획의 필수 항목과 세 문서의 초기화 기록을 확인했다.
- `git diff --check`는 오류 없이 종료했지만 Goal 디렉터리가 untracked이므로 이를 문서 내용 검사의 단독
  근거로 사용하지 않았다. 별도 `awk` whitespace 검사는 출력 없이 성공했다.
- 안전하게 인용해 재실행한 Status, 구현 상태와 Goal 이름 검사는 모두 기대한 줄을 찾았다.
- Goal 디렉터리는 scoped status에서 untracked로 보고됐다.

### 변경한 내용

- `EXPERIMENT_NOTES.md`의 Status를 허용 값 `active`로 정규화했다.
- 이 문서 검증 기록을 `EXPERIMENTS.md`에 추가했다.
- Production 파일은 변경하지 않았다.

### 실패 원인

- 초기 Status 표기 한 건은 문서 검토에서 발견해 수정했다.
- 첫 최종 검사에서 double-quoted `rg` 정규식의 Goal 이름 백틱이 shell command substitution으로
  평가되어 `/bin/bash: line 1: maintenance_action_capability_projection: command not found`가 발생했다.
  뒤의 `rg`가 일치해 compound command 전체 exit code는 0이었으므로 이 실행을 Goal 이름 확인 증거로
  사용하지 않았다. 파일 변경은 없었고 single-quoted 정규식으로 분리해 재실행한 검사는 exit code 0으로
  통과했다.

### 되돌리거나 수정한 내용

- 허용되지 않는 Status 설명어 `notes`를 제거했다.

### 결과에 따른 판단

- Goal 문서는 안전하게 초기화됐고 Status는 `active`다.
- 구현과 production baseline은 시작하지 않았으며 `Implementation complete: false`다.
- 다음 실행은 세 문서를 먼저 읽은 뒤 current source 조사와 dirty-overlap 확인부터 재개해야 한다.

## 2026-08-03 — Current Source 및 계약 경계 조사

### 검증하려던 가설

- 결함은 하나의 Core capability projection 경계에 국한되고 DTO/live RON/save 정책 변경이 필요하지 않다.
- Maintenance option producer와 selected-payload validator가 이미 존재하며 Unity는 두 authority를 함께
  소비한다.
- 기존 dirty 변경과 안전하게 병합 가능한 작은 구현 경계를 찾을 수 있다.

### 수행한 실험 또는 검증

- 두 저장소의 전체 `git status --short`를 다시 확인했다.
- 다섯 action의 enum, behavior, dispatch, preview, validator, mutation과 snapshot 경로를 추적했다.
- game_server root snapshot wrapper와 Unity reader/store/command gate를 확인했다.
- gameplay/Core/Unity canonical과 embedded live loader 및 equipment recipe coverage를 비교했다.
- 후보 production/test 파일의 current content와 tracked diff를 확인했다.

### 실행한 주요 정확한 명령

```bash
git status --short
git -C '/mnt/f/unity projects/ark' status --short
rg -n --glob '*.rs' 'UpgradeSkillFragment|AwakenSkillFragment|DismantleSkillFragment|DismantleEquipment|EnhanceEquipment|NoEligiblePayload' core_enhanced/src core_enhanced/tests game_server/src
rg -n --glob '*.rs' 'action_capabilities|maintenance_options|ActionCapability' core_enhanced/src game_server/src
rg -n --glob '*.cs' --glob '*.md' 'UpgradeSkillFragment|AwakenSkillFragment|DismantleSkillFragment|DismantleEquipment|EnhanceEquipment|NoEligiblePayload|maintenance_options|action_capabilities' Assets docs
sed -n '1,605p' core_enhanced/src/game/world/action_capability.rs
sed -n '1,540p' core_enhanced/src/game/world/facility.rs
sed -n '210,440p' core_enhanced/src/game/world.rs
sed -n '420,620p' core_enhanced/src/game/world/helpers.rs
sed -n '260,560p' core_enhanced/src/game/world/maintenance.rs
sed -n '350,480p' core_enhanced/src/game/world/snapshot.rs
sed -n '630,810p' game_server/src/game/player_game_actor/handlers.rs
sed -n '65,180p' Assets/Scripts/ArkClient/Runtime/Core/GameSnapshotStore.cs
sed -n '1,180p' Assets/Scripts/ArkClient/Runtime/Views/MaintenanceCommandHandler.cs
sed -n '1,240p' Assets/Scripts/ArkClient/Runtime/Core/Reading/MaintenanceOptionsModel.cs
rg -n 'Maintenance|maintenance|action_capabilit|NoEligiblePayload|parameterized' core_enhanced/docs/game_rulebook.md core_enhanced/docs/core_runtime_contract.md docs/unity_core_contract.md
rg -n 'load_live_embedded|bundle.ron|equipments/base.ron|skill_fragments' core_enhanced/src/game/data core_enhanced/src game_resources/data/bundle.ron
sed -n '1725,1785p' game_resources/data/equipments/base.ron
git diff -- core_enhanced/src/game/world/tests/equipment.rs
git -C '/mnt/f/unity projects/ark' diff -- Assets/Scripts/ArkClient/Runtime/Core/GameSnapshotStore.cs Assets/Scripts/ArkClient/Runtime/Views/MaintenanceCommandHandler.cs Assets/Scripts/ArkClient/Runtime/Core/Reading/MaintenanceOptionsModel.cs Assets/Tests/EditMode/ArkClientContractTests.cs Assets/Tests/PlayMode/ArkClientLiveFlowTests.cs docs/unity_core_contract.md
```

### 실제 test 수와 결과

- Test runner를 실행하지 않았다.
- 실제 test 수: runner 실행 없음.
- Pass/fail/skip: 해당 없음.
- 다섯 action/behavior/validator/handler와 typed disabled reason은 이미 존재한다.
- Root projection과 common preflight의 후보 목록에는 다섯 action이 없다.
- `maintenance_options()`는 target kind별 operation `can_execute`를 이미 생산한다.
- Server wrapper는 Core root snapshot을 보존하고 Unity는 capability와 operation eligibility를 함께 확인한다.
- Live embedded loader와 `standard_armor`의 explicit dismantle/enhancement recipe를 확인했다.

### 변경한 내용

- 조사 결과를 `PLAN.md`와 `EXPERIMENT_NOTES.md`에 반영했다.
- Production 파일은 변경하지 않았다.

### 실패 원인

- 통합 검색 출력 일부는 길이 제한으로 절단됐다. 판정에 필요한 파일은 이후 좁은 `sed`/`rg` 명령으로
  다시 읽었다.
- Test 실패/성공은 아직 측정하지 않았다.

### 되돌리거나 수정한 내용

- 없음.

### 결과에 따른 판단

- Status는 `active`를 유지한다.
- DTO/live RON/save/compatibility migration 없이 두 Core 파일의 좁은 수정으로 구현 가능하다.
- Production 변경 전에 current dirty tree에서 Core, server, Unity와 live explicit baseline을 확보한다.

## 2026-08-03 — Production 변경 전 Baseline

### 검증하려던 가설

- 현재 dirty worktree의 Core, server, live RON과 Unity focused 계약은 기존 회귀 없이 통과한다.
- 실제 local game_server와 Unity consumer를 사용하면 Maintenance option은 노출되지만 Core root capability
  누락 때문에 실행 가능한 operation을 하나도 찾지 못하는 결함이 재현된다.

### 수행한 실험 또는 검증

- Core 전체 lib, game_server 전체 lib와 live RON integration suite를 실행했다.
- Unity Maintenance command-handler focused EditMode 3개를 실행했다.
- admin command를 활성화한 current game_server를 8081에서 실행했다.
- Unity explicit live Maintenance PlayMode test를 단독 실행했다.

### 실행한 정확한 명령 또는 runner job

```bash
cargo test -p game_core --lib
cargo test -p game_server --lib
cargo test -p game_core --test ron_loading
/bin/bash -lc 'ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run'
```

Unity EditMode runner:

```text
mode=EditMode
test_names=[
  ArkClient.Tests.EditMode.ArkClientContractTests.MaintenanceCommandHandler_RequestsConfirmationBeforeDismantle,
  ArkClient.Tests.EditMode.ArkClientContractTests.MaintenanceCommandHandler_SendsExecutableEnhanceThroughGateway,
  ArkClient.Tests.EditMode.ArkClientContractTests.MaintenanceCommandHandler_DoesNotSendWhenSelectedTargetIsMissingFromLatestSnapshot
]
job_id=03aa7e37e80e434484c308f3951eda48
```

Unity PlayMode runner:

```text
mode=PlayMode
test_names=ArkClient.Tests.PlayMode.ArkClientLiveFlowTests.LiveAdmin_MaintenanceFixtureExposesOperationOptions
job_id=47abc2335c6f49b6a13fa7ed17984247
```

### 실제 test 수와 pass/fail/skip

- `cargo test -p game_core --lib`: 798 total, 798 pass, 0 fail, 0 ignored.
- `cargo test -p game_server --lib`: 54 total, 54 pass, 0 fail, 0 ignored.
- `cargo test -p game_core --test ron_loading`: 23 total, 23 pass, 0 fail, 0 ignored.
- Unity EditMode job `03aa7e37e80e434484c308f3951eda48`: 3 total, 3 pass, 0 fail,
  0 skip.
- Unity PlayMode job `47abc2335c6f49b6a13fa7ed17984247`: 1 total, 0 pass, 1 fail,
  0 skip.

### 실제 결과와 실패 원인

- Core, server, live RON과 Unity focused baseline은 모두 통과했다.
- Local server는 embedded game data를 성공적으로 load하고 `0.0.0.0:8081`에서 시작했다.
- Explicit live test는 `maintenance_options.items`와 operation DTO를 정상적으로 읽은 뒤 실패했다.
- 정확한 assertion은 `No executable live Maintenance operation was found. Expected: greater than 0 But was: 0`다.
- Test helper는 `operation.CanExecute`와 `store.IsActionEnabled(actionGate)`가 모두 true인 target만 선택한다.
  Current Core가 다섯 capability를 투영하지 않으므로 이 결과는 조사한 projection 누락과 일치한다.

### 변경한 내용

- Production 파일 변경 없음.
- Baseline 실행으로 game_server runtime log가 갱신될 수 있으나 이를 Goal production 성과로 분류하지
  않는다.

### 되돌리거나 수정한 내용

- 없음.

### 결과에 따른 판단

- Existing baseline failure는 explicit live Maintenance test 1개이며 현재 Goal이 직접 해결해야 한다.
- 다른 baseline suite에는 실패가 없다.
- 변경 후 동일 명령과 runner를 재실행해 새 실패와 기존 결함 해소를 비교할 수 있다.

## 2026-08-03 — Core Projection 구현 및 첫 회귀 실행

### 검증하려던 가설

- 기존 `maintenance_options`를 재사용하면 다섯 action을 target kind/operation별로 정확히 투영할 수 있다.
- 관련 없는 context에서는 absent, 관련 있지만 executable payload가 없으면 `NoEligiblePayload`, 하나라도
  있으면 enabled가 된다.
- Any-payload capability가 enabled인 경우 invalid selected payload는 기존 domain validator에서 실패하고
  snapshot을 변경하지 않는다.

### 변경한 내용

- `world/facility.rs`에 다섯 action과 target kind/operation의 단일 mapping helper를 추가했다.
- `world/action_capability.rs`의 root projection과 common preflight에 다섯 action을 추가했다.
- Snapshot projection에서는 `maintenance_options()`를 한 번만 계산해 다섯 action이 공유한다.
- `world/tests/facility.rs`에 absent/disabled/serialized shape/node-completion 및 live-data enabled mapping,
  selected invalid payload atomicity 회귀를 추가했다.

### 실행한 정확한 명령

```bash
cargo test -p game_core maintenance_capabilit -- --nocapture
cargo test -p game_core maintenance_capabilit -- --nocapture
cargo test -p game_core independent_maintenance_node_exposes_maintenance_actions -- --nocapture
cargo test -p game_core --lib
```

첫 명령은 import 수정 전, 두 번째는 수정 후 재실행이다.

### 실제 test 수와 pass/fail/skip

- 첫 `maintenance_capabilit` 실행: compile 실패로 test runner 실행 전 종료, 실제 test 0개.
- 재실행한 `maintenance_capabilit`: 1 total, 1 pass, 0 fail, 0 ignored. 다른 binary는 0개 실행이라
  완료 증거 수에 포함하지 않는다.
- `independent_maintenance_node_exposes_maintenance_actions`: 1 total, 1 pass, 0 fail, 0 ignored.
- 첫 변경 후 `cargo test -p game_core --lib`: 799 total, 794 pass, 5 fail, 0 ignored.

### 실제 결과와 실패 원인

- 첫 focused compile은 새 test 모듈에 `ActionCapabilityDto`, `ActionScope`,
  `MaintenanceTargetKind` import가 없어 Rust E0433 6건으로 실패했다.
- Import를 추가한 뒤 두 focused 회귀는 통과했다.
- Core 전체의 5개 실패는 모두 구형 error assertion이었다.
  - `bound_equipment_cannot_be_dismantled_or_enhanced_in_maintenance`
  - `dismantle_equipment_is_maintenance_only_and_grants_material_stacks`
  - `enhance_equipment_is_maintenance_only_and_updates_owned_instance`
  - `skill_fragment_dismantle_is_maintenance_only_and_grants_dust`
  - `skill_fragment_upgrade_consumes_fragment_dust_and_updates_progress`
- Maintenance 밖 action은 이제 `ActionUnavailable(ContextIrrelevant)`, Maintenance 안에서 해당 family의
  executable payload가 하나도 없으면 `ActionUnavailable(NoEligiblePayload)`로 common preflight에서
  거절된다. 이는 Goal의 승인된 capability/preflight 계약과 일치한다.
- Enabled family에서 존재하지 않는 equipment UUID를 보낸 새 회귀는 기존
  `InventoryItemNotFound`를 유지했고 명령 전후 root snapshot이 동일했다.

### 되돌리거나 수정한 내용

- 누락된 test import를 추가했다.
- Production projection 구현은 되돌리지 않았다.

### 결과에 따른 판단

- 5개 실패는 baseline failure가 아니라 승인된 새 typed preflight 의미로 인해 드러난 stale test
  expectation이다.
- 테스트가 보호하던 Maintenance-only/invalid payload 거절을 삭제하지 않고 새 typed error 계약으로
  교체한 뒤 Core 전체를 재실행한다.

## 2026-08-03 — Typed Preflight 회귀 보정 및 Core/Unity Focused 검증

### 검증하려던 가설

- 기존 Maintenance-only 회귀의 실패 기대값을 새 common preflight 계약으로 바꾸면 보호하던 거절 의미를
  유지하면서 Core 전체가 다시 통과한다.
- Unity command handler는 최신 snapshot의 해당 capability가 absent 또는 disabled인 두 경우 모두 실제
  transport 전송을 차단한다.

### 변경한 내용

- 다섯 기존 Core 회귀의 구형 `InvalidAction` 기대값을 실제 context에 맞는
  `ActionUnavailable(ContextIrrelevant)` 또는 `ActionUnavailable(NoEligiblePayload)`로 교체했다.
- Unity EditMode 회귀에 `EnhanceEquipment` capability absent/disabled 두 경우를 추가했다.
- 기계적 expectation 교체 과정에서 범위와 무관한 `EquipSkillFragment` 회귀 2개가 잠시 변경된 것을 scoped
  diff 검토로 발견해 원래 `InvalidAction` 기대값으로 복구했다.

### 실행한 정확한 명령 또는 runner job

```bash
cargo test -p game_core --lib game::world::tests::equipment:: -- --nocapture
cargo test -p game_core --lib game::world::tests::equipment:: -- --nocapture
cargo test -p game_core --lib --quiet
```

첫 equipment 명령은 import 수정 전, 두 번째는 수정 후 재실행이다.

Unity EditMode runner:

```text
mode=EditMode
test_names=ArkClient.Tests.EditMode.ArkClientContractTests.MaintenanceCommandHandler_DoesNotSendWhenLatestCapabilityIsAbsentOrDisabled
job_id=dd1de324264f42ef809f27e7990b25ea
```

### 실제 test 수와 pass/fail/skip

- 첫 equipment focused 실행: compile 실패로 test runner 실행 전 종료, 실제 test 0개.
- 재실행한 equipment focused suite: 28 total, 28 pass, 0 fail, 0 ignored.
- `cargo test -p game_core --lib --quiet`: 799 total, 799 pass, 0 fail, 0 ignored.
- Unity EditMode job `dd1de324264f42ef809f27e7990b25ea`: 2 total, 2 pass, 0 fail,
  0 skip.

### 실제 결과와 실패 원인

- 첫 equipment focused compile은 test 모듈의 `ActionDisabledReasonCode` import 누락으로 실패했다.
- Import 추가 후 관련 28개와 Core 전체 799개가 모두 통과했다.
- Unity 실제 `GameSnapshotStore` → `GameCommandBus` → `GameCommandBusMaintenanceGateway` →
  `MaintenanceCommandHandler` 경로는 absent와 disabled capability에서 모두 전송 0건을 유지했다.

### 되돌리거나 수정한 내용

- Core test import 누락을 수정했다.
- 범위 밖 `EquipSkillFragment` expectation 2개를 원래 상태로 되돌렸다.

### 결과에 따른 판단

- Core projection, typed preflight, selected-payload validator 및 Unity consumer gate의 focused 계약은
  통과한다.
- Baseline에서 실패한 explicit live Unity test는 변경 후 server binary로 재검증해야 한다.

## 2026-08-03 — 변경 후 Explicit Live Producer-Consumer 검증

### 검증하려던 가설

- 변경된 Core를 포함해 다시 빌드한 실제 game_server는 root capability와 `maintenance_options`를 일치시켜
  전송한다.
- Unity live consumer는 적어도 하나의 executable Maintenance operation을 발견하고 실제 명령을 전송할
  수 있다.

### 수행한 실험 또는 검증

- Baseline server를 SIGINT로 정상 종료했다.
- 동일한 admin 설정으로 server를 다시 빌드·실행해 live data load와 8081 listen을 확인했다.
- Baseline에서 실패했던 explicit Unity PlayMode test를 단독 재실행했다.
- Test 직후 server log에서 실제 Maintenance 보상 mutation을 확인했다.

### 실행한 정확한 명령 또는 runner job

```bash
/bin/bash -lc 'ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run'
```

Unity PlayMode runner:

```text
mode=PlayMode
test_names=ArkClient.Tests.PlayMode.ArkClientLiveFlowTests.LiveAdmin_MaintenanceFixtureExposesOperationOptions
job_id=b05c039aaa194707bead2d47c6466ed8
```

### 실제 test 수와 pass/fail/skip

- Server startup: test runner가 아니므로 test 수를 보고하지 않음.
- Unity PlayMode job `b05c039aaa194707bead2d47c6466ed8`: 1 total, 1 pass, 0 fail,
  0 skip.

### 실제 결과와 실패 원인

- Server는 변경된 `game_core`와 `game_server`를 다시 compile했고 embedded live data를 load한 뒤
  `0.0.0.0:8081`에서 시작했다.
- Unity test는 baseline의 `No executable live Maintenance operation was found` 실패 없이 통과했다.
- Server log에는 test가 Maintenance node에 진입한 뒤 skill-fragment dust와 equipment material reward를
  실제로 지급한 기록이 남아 있어 mock-only 검증이 아님을 확인했다.
- 실패 없음.

### 변경한 내용

- Production 파일 변경 없음.
- Test 실행으로 server runtime log가 갱신될 수 있다.

### 되돌리거나 수정한 내용

- Baseline server process는 정상 종료했다.
- 변경 후 server는 broad Unity 검증을 위해 계속 실행 중이다.

### 결과에 따른 판단

- Baseline 1 fail은 변경 후 1 pass로 해소됐다.
- 실제 Core producer → server wrapper/transport → Unity typed reader/store/command 경계에서 사용자-visible
  Maintenance 명령이 다시 실행 가능하다.

## 2026-08-03 — Broad Rust/Unity 및 인접 Live 검증

### 검증하려던 가설

- 변경이 Core/server/live RON workspace와 Unity 전체 suite에 새 회귀를 만들지 않는다.
- Maintenance 외의 explicit live flow도 동일 server에서 유지된다.

### 수행한 실험 또는 검증

- game_server lib와 live RON suite를 변경 후 재실행했다.
- Rust workspace all targets를 실행했다.
- Unity LiveServer category 4개를 실행하고, 첫 실행의 전투 screenshot 실패를 단독 및 전체 category
  재실행으로 재현성 확인했다.
- Unity full EditMode와 full PlayMode를 실행했다.

### 실행한 정확한 명령 또는 runner job

```bash
cargo test -p game_server --lib --quiet
cargo test -p game_core --test ron_loading --quiet
cargo test --all-targets --quiet
```

Unity PlayMode runner:

```text
category_names=LiveServer
job_id=646ae57c884043ebbaba96520f8fa01f

test_names=ArkClient.Tests.PlayMode.ArkClientLiveFlowTests.LiveGame_CombatPathAcceptsBattleControls
job_id=f6dbee947b5d4f019201f3d76aa64932

category_names=LiveServer
job_id=6cf43e77b94e4bcc95e155f4a2175578

mode=PlayMode
job_id=5861628afaee490ba79c752ad1e1264d
```

Unity EditMode runner:

```text
mode=EditMode
job_id=2bfcf40dd1714c46885b6779ac3feab6
```

### 실제 test 수와 pass/fail/skip

- `cargo test -p game_server --lib --quiet`: 54 total, 54 pass, 0 fail, 0 ignored.
- `cargo test -p game_core --test ron_loading --quiet`: 23 total, 23 pass, 0 fail,
  0 ignored.
- 첫 `cargo test --all-targets --quiet`: non-zero test binary 합계 900 total, 900 pass,
  0 fail, 0 ignored. 0-test harness는 완료 증거 수에서 제외했다.
- 첫 LiveServer category job `646ae57c884043ebbaba96520f8fa01f`: progress는 4/4 completed와
  failure 1개를 보고했지만 failed job의 result summary가 null이어서 pass/skip 수는 runner가 보고하지
  않음.
- 전투 live 단독 job `f6dbee947b5d4f019201f3d76aa64932`: 1 total, 1 pass, 0 fail,
  0 skip.
- 재실행 LiveServer category job `6cf43e77b94e4bcc95e155f4a2175578`: 4 total, 4 pass,
  0 fail, 0 skip.
- Full EditMode job `2bfcf40dd1714c46885b6779ac3feab6`: 262 total, 262 pass,
  0 fail, 0 skip.
- Full PlayMode job `5861628afaee490ba79c752ad1e1264d`: 66 total, 62 pass, 0 fail,
  4 skip. 네 skip은 별도로 실행한 `[Explicit]` LiveServer test다.

### 실제 결과와 실패 원인

- Core/server/live data와 두 Unity full suite는 통과했다.
- 첫 LiveServer category 실행에서
  `LiveGame_CombatPathAcceptsBattleControls`의 screenshot battlefield pixel 비율이 expected `> 0.03`,
  actual `0.0`으로 한 번 실패했다. Maintenance projection/command failure가 아니었다.
- 동일 전투 test 단독 재실행은 통과했고, Maintenance/Event/RunStart/Combat 네 live test 전체 재실행도
  4/4 통과해 일시적 rendering capture 실패로 판정했다.

### 변경한 내용

- Production 변경 없음.

### 되돌리거나 수정한 내용

- 없음.

### 결과에 따른 판단

- 영향 범위 broad 검증과 인접 live flow는 최종 재실행 기준 새 실패가 없다.
- Full PlayMode의 네 explicit skip은 explicit LiveServer category 4/4 결과로 별도 보완됐다.

## 2026-08-03 — Canonical, Formatting, Scoped Diff 및 최종 재검증

### 검증하려던 가설

- 구현 파일은 repository Rust format과 whitespace 규칙을 만족한다.
- 승인되지 않은 `allowed_actions`, fallback, local capability inference 또는 dual projection이 추가되지
  않았다.
- Blocking VN에서도 Maintenance capability가 사라지고 최종 all-targets가 다시 통과한다.
- Gameplay/runtime/Unity canonical이 실제 mapping과 consumer gate를 설명한다.

### 변경한 내용

- `docs/game_rulebook.md`에 다섯 action-operation mapping과 absent/disabled/any-payload 의미를 기록했다.
- `docs/core_runtime_contract.md`에 root parameterized admission, common preflight와 selected-payload
  validator의 책임을 기록했다.
- Unity `docs/unity_core_contract.md`에 exact action names, `no_eligible_payload`, option/capability 공동 gate와
  local inference 금지를 기록했다.
- Core 회귀에 blocking Visual Novel 동안 다섯 Maintenance capability가 모두 absent임을 추가했다.
- 첫 formatting check가 제안한 네 Core 파일의 formatting만 좁게 적용했다.

### 실행한 정확한 명령 또는 runner job

```bash
cargo fmt --all -- --check
cargo fmt --all -- --check
cargo test -p game_core independent_maintenance_node_exposes_maintenance_actions -- --nocapture
cargo test -p game_core --lib independent_maintenance_node_exposes_maintenance_actions -- --nocapture
cargo test --all-targets --quiet
git diff --check -- core_enhanced/src/game/world/action_capability.rs core_enhanced/src/game/world/facility.rs core_enhanced/src/game/world/tests/facility.rs core_enhanced/src/game/world/tests/equipment.rs core_enhanced/docs/game_rulebook.md core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/goals/maintenance_action_capability_projection
git diff --check -- Assets/Tests/EditMode/ArkClientContractTests.cs docs/unity_core_contract.md
rg -n '[[:blank:]]+$' src/game/world/action_capability.rs src/game/world/facility.rs src/game/world/tests/facility.rs src/game/world/tests/equipment.rs docs/game_rulebook.md docs/core_runtime_contract.md docs/goals/maintenance_action_capability_projection
rg -n '[[:blank:]]+$' Assets/Tests/EditMode/ArkClientContractTests.cs docs/unity_core_contract.md
```

Legacy/local-inference search:

```bash
rg -n 'allowed_actions|allowedActions|infer(red|ence)?[_ -]?capabil|local[_ -]?capabil|capabilit(y|ies).*(fallback|alias)|dual[_ -]?(read|write)' src docs/game_rulebook.md docs/core_runtime_contract.md
rg -n 'allowed_actions|allowedActions|infer(red|ence)?[_ -]?capabil|local[_ -]?capabil|capabilit(y|ies).*(fallback|alias)|dual[_ -]?(read|write)' Assets/Scripts/ArkClient docs/unity_core_contract.md
rg -n 'UpgradeSkillFragment|AwakenSkillFragment|DismantleSkillFragment|DismantleEquipment|EnhanceEquipment' Assets/Scripts/ArkClient/Runtime/Core Assets/Scripts/ArkClient/Runtime/Views/MaintenanceCommandHandler.cs
```

Build artifact 진단/복구:

```bash
file target/debug/deps/generate_event_contract_fixture-9531c6a08fbc9052
od -An -tx1 -N16 target/debug/deps/generate_event_contract_fixture-9531c6a08fbc9052
rm -f target/debug/deps/generate_event_contract_fixture-9531c6a08fbc9052 target/debug/deps/generate_event_contract_fixture-9531c6a08fbc9052.d
```

### 실제 test 수와 pass/fail/skip

- 첫 `cargo fmt --all -- --check`: formatter check 실패; test runner가 아니므로 test 수 없음.
- 재실행 `cargo fmt --all -- --check`: 성공; test runner가 아니므로 test 수 없음.
- package-wide focused 명령: lib test 1 total, 1 pass, 0 fail, 0 ignored 후 0-test fixture binary
  실행 단계의 OS 오류로 command exit 101.
- `--lib` focused 재실행: 1 total, 1 pass, 0 fail, 0 ignored.
- 최종 `cargo test --all-targets --quiet`: non-zero test binary 합계 900 total, 900 pass,
  0 fail, 0 ignored. 0-test harness는 완료 증거 수에서 제외했다.
- Diff/whitespace/symbol search는 test runner가 아니므로 test 수 없음.

### 실제 결과와 실패 원인

- 첫 formatter check는 새 Core hunk 네 곳의 rustfmt 차이를 정확히 보고했다. 해당 hunk만 수정한 뒤
  check가 통과했다.
- package-wide focused 명령은 회귀 자체는 1/1 통과했지만
  `generate_event_contract_fixture-9531c6a08fbc9052` 실행에서 `Exec format error (os error 8)`로
  종료됐다. `file`은 artifact를 `data`로 식별했고 첫 16 byte가 모두 zero였다.
- 손상된 두 generated target artifact만 제거해 Cargo가 다시 생성하도록 한 뒤 `--lib` focused와 최종
  all-targets가 통과했다. Production/source 파일은 이 복구로 변경하지 않았다.
- Core/Unity scoped `git diff --check`는 성공했다. 두 trailing-whitespace `rg`는 match 없음(exit 1)이다.
- Core production/canonical legacy search는 match 없음(exit 1)이다. Unity production/canonical search의
  유일한 match는 canonical의 local inference 금지 문장이다. `allowed_actions` production 사용,
  compatibility fallback, alias와 dual read/write는 발견되지 않았다.
- Unity action symbol search는 exact five names가 `GameActionNames`, command bus와 Maintenance handler에만
  연결된 것을 확인했다.

### 되돌리거나 수정한 내용

- rustfmt가 제안한 Goal 관련 hunk만 수동 반영했다. Repository-wide formatter write는 실행하지 않았다.
- zero-filled generated test artifact와 dependency file만 제거해 Cargo 재빌드로 복구했다.

### 결과에 따른 판단

- 모든 필수 focused/broad 검증, formatting, scoped diff, canonical 및 legacy/local-inference audit가
  최종 상태에서 통과했다.
- 필수 미검증 항목과 새 실패가 남지 않았다.
