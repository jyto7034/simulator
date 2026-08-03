# Maintenance Action Capability Projection Goal

Status: complete

Implementation complete: true

Goal name: `maintenance_action_capability_projection`

Created: 2026-08-03

Audit finding ID: 없음

Execution authority: 2026-08-03 사용자 요청 및 승인된 단일 Goal 범위

## 1. 승인된 Goal 설명

Maintenance node의 기존 대상별 operation eligibility와 Core-owned typed context capability를 연결한다.
`UpgradeSkillFragment`, `AwakenSkillFragment`, `DismantleSkillFragment`,
`DismantleEquipment`, `EnhanceEquipment`가 현재 Maintenance 상태와 실제 실행 가능한 payload 존재 여부를
반영해 root `action_capabilities`에 투영되게 하고, Unity가 그 권위를 사용해 실행 가능한 Maintenance
작업을 실제 command로 보낼 수 있게 한다.

이 Goal은 Maintenance gameplay, 비용, recipe 또는 결과를 새로 설계하지 않는다. 이미 승인된 operation
preview와 command validator를 capability 경계에 수렴시키는 하나의 완결된 계약 수정이다.

## 2. 해결할 사용자-visible 문제

Maintenance 화면이 대상별 `maintenance_options.operations.*.can_execute`를 정상적으로 받더라도 Core root
capability에 해당 action이 없어서 Unity command gate가 닫힌다. 그 결과 실행 가능한 강화·개화·분쇄
작업이 화면에 존재해도 플레이어가 실행할 수 없고, explicit live Maintenance flow가 executable
operation을 찾지 못한다.

## 3. 사용자-visible 완료 조건

- Maintenance node에서 다섯 action 각각은 대응하는 실행 가능 payload가 하나 이상 있을 때 enabled
  capability로 투영된다.
- Maintenance node와 무관한 상태에서는 다섯 action이 관련 없는 capability로 생략된다.
- Maintenance context이지만 해당 action에 실행 가능한 payload가 하나도 없으면 capability는 typed
  `NoEligiblePayload` reason으로 disabled된다.
- Unity는 최신 `maintenance_options`와 enabled capability를 함께 확인한 작업만 전송할 수 있다.
- snapshot 이후 대상·재료·상태가 바뀐 command는 기존 Core domain validator가 다시 검증하며 부분
  mutation 없이 실패한다.
- 기존 실패 `LiveAdmin_MaintenanceFixtureExposesOperationOptions`가 실제 local game_server와 Unity
  consumer를 사용해 통과한다.
- 관련 Core, server, Unity focused 검증과 영향 범위 broad 검증이 새 실패 없이 통과한다.

## 4. 대상 저장소와 경로

- Goal 및 Core/server workspace: `/mnt/f/work/simulator`
- Core: `/mnt/f/work/simulator/core_enhanced`
- game_server: `/mnt/f/work/simulator/game_server`
- Unity consumer와 canonical: `/mnt/f/unity projects/ark`
- Goal 문서: `core_enhanced/docs/goals/maintenance_action_capability_projection/`

2026-08-03 current source 조사에서 확인한 production 경계는 다음과 같다.

- Core projection/preflight: `core_enhanced/src/game/world/action_capability.rs`
- Maintenance option producer: `core_enhanced/src/game/world/facility.rs`
- selected-payload validator: `core_enhanced/src/game/world/helpers.rs`
- atomic mutation handler: `core_enhanced/src/game/world/maintenance.rs`
- root snapshot producer: `core_enhanced/src/game/world/snapshot.rs`
- server wrapper/pass-through: `game_server/src/game/player_game_actor/handlers.rs`
- Unity capability store/gate: `Assets/Scripts/ArkClient/Runtime/Core/GameSnapshotStore.cs`,
  `GameCommandBus.cs`, `Runtime/Views/MaintenanceCommandHandler.cs`
- Unity operation reader: `Assets/Scripts/ArkClient/Runtime/Core/Reading/MaintenanceOptionsModel.cs`

## 5. 현재 runtime 및 live data 근거

2026-08-03 current source 조사 결과:

- `ActionKind`, `PlayerBehavior`와 target-only command payload에는 다섯 Maintenance action이 모두 존재한다.
- `facility.rs::maintenance_options()`가 대상별 `dismantle`/`enhance`/`awaken` preview와
  `can_execute`, 비용, 결과, confirmation 및 disabled reason을 생산한다.
- `helpers.rs`의 다섯 payload validator는 최신 Maintenance context, 보유 대상, recipe, 재료와 상태를
  재검증하고 `maintenance.rs` handler는 검증 뒤 mutation한다.
- Root snapshot은 `snapshot.rs`에서 `get_action_capabilities()` 결과를 직접 직렬화한다.
- Server `PlayerStateSnapshotDto` wrapper는 Core root field/value를 보존하며 Maintenance capability를 별도로
  재해석하는 mapping은 없다.
- Unity `GameSnapshotStore`는 root `action_capabilities`를 typed store에 넣고
  `MaintenanceCommandHandler`는 최신 option의 `CanExecute`와 gateway capability를 모두 통과한 경우에만
  command를 전송한다.
- 실제 결함은 `action_capability.rs::get_action_capabilities()`, `evaluate_common_action()`와
  `preflight_common_capability_behavior()`가 다섯 action을 포함하지 않는 것이다.
- Production loader는 embedded `game_resources/data/equipments/base.ron`과 skill-fragment catalog를 읽는다.
  Live equipment recipe는 canonical대로 `standard_armor`에 존재하고 fragment operation은 기존 run policy를
  사용하므로 schema/content migration은 필요하지 않다.
- 2026-08-03 Goal 활성화 전 Rust/Unity 결과는 production baseline으로 재사용하지 않는다. 변경 전
  baseline은 이 Goal의 `EXPERIMENTS.md`에 별도로 기록한다.

## 6. Source Of Truth와 소유 계층

| 책임 | Authority | 소비 계층 |
| --- | --- | --- |
| context relevance, action scope, enabled/disabled 상태 | Core typed action capability projector | game_server snapshot, Unity store/command gate |
| 구체 대상별 실행 가능성, 비용, 결과, confirmation | Core Maintenance domain preview/validator | root selected-event DTO, Unity read model/presentation |
| command 최종 admission과 mutation | Core Maintenance command validator/handler | game_server command result, Unity refresh |
| root snapshot 전달 | game_server | Unity strict reader/store |
| 화면 선택과 표시 | Unity | 플레이어 |
| gameplay 의미 | `docs/game_rulebook.md` | 모든 계층 |
| runtime capability 불변식 | `docs/core_runtime_contract.md` | Core/server/test |
| exact Unity wire/consumer 계약 | 외부 `docs/unity_core_contract.md` | Core/server/Unity |

Capability는 authorization token이 아니다. Core command 실행은 최신 runtime state에서 같은 domain
preflight와 payload validator를 다시 적용한다.

## 7. 유지할 불변식

- Parameterized action capability는 현재 실행 가능한 payload가 하나 이상일 때만 enabled다. Enabled는
  모든 payload가 실행 가능하다는 뜻이 아니다.
- 관련 없는 context의 action은 배열에 없고, 관련 있지만 막힌 action은 disabled reason을 가진다.
- `maintenance_options`가 대상별 eligibility, 비용, 결과, warning, 자동 해제와 confirmation의 단일
  authority다. Unity와 capability projector가 recipe나 비용을 재계산하지 않는다.
- 최종 command는 대상 identity, 보유 상태, 재료, recipe, 장착 상태와 현재 Maintenance context를 다시
  검증한다.
- 작업은 원자적이며 실패한 command가 재료, 장비, 파편, 장착 상태를 부분 변경하지 않는다.
- 장비와 fragment 경제, live recipe coverage, 비용·보상·강화 수치와 Maintenance node 소비 의미를
  바꾸지 않는다.
- Save, checkpoint, rollback, active-run, battle와 non-Maintenance capability 의미를 바꾸지 않는다.
- 기존 사용자 변경과 현재 Goal 밖 dirty 파일을 보존한다.

## 8. 범위

- Maintenance 다섯 action의 root typed capability projection
- 각 action과 대응 target kind/operation의 명시적 mapping
- 현재 Maintenance context relevance와 any-executable-payload 판정
- 공통 capability preflight와 실제 Maintenance validator의 책임 정렬
- Core root snapshot, game_server pass-through와 Unity command gate의 producer-consumer 확인
- 기존 explicit live Maintenance failure를 직접 고정하는 회귀
- 가장 가까운 canonical의 capability/Maintenance seam 갱신

## 9. 명시적 비범위

- Maintenance 비용, 획득량, 성공률, recipe, 강화 한도와 balance 변경
- 새로운 equipment/fragment recipe 또는 live content authoring
- Maintenance DTO shape, operation 이름, command payload 또는 화면 layout 재설계
- Save/checkpoint/active-run schema와 persistence 변경
- 다른 Safe Node, battle capability 또는 일반 inventory capability 리팩토링
- 실패와 독립적인 Unity 화면/UX 기능
- hidden boss, monolith, account profile 또는 다른 gameplay 시스템
- 범위 밖 테스트 실패 수정, 새 Goal 또는 Master 자동 생성
- dirty worktree 정리, commit, stash, reset 또는 관련 없는 formatting

## 10. 승인된 정책과 근거

- 사용자 승인: 기존 Maintenance 다섯 action을 capability projection에 연결하는 단일 Goal을 만든다.
- 기존 contextual capability 계약: parameterized action은 실행 가능한 payload가 하나 이상 있을 때만
  enabled이며 관련 없는 action은 생략한다.
- 기존 Unity 계약: 버튼과 command admission은 root `action_capabilities`와 대상별
  `maintenance_options`를 함께 사용한다.
- 기존 Maintenance 계약: `can_execute`, typed operation disabled reason, costs/gains, before/after,
  `requires_confirm`, `will_unequip`과 warning은 Core가 생산한다.
- Compatibility, fallback, alias 또는 Unity-local capability 추론은 승인되지 않았다.

## 11. 미결정 정책

현재 없음.

Current source/canonical 조사에서 action presence, disabled reason, DTO shape, UX, recipe 또는 command failure
의미가 위 승인 정책과 충돌하면 임의로 선택하지 않는다. 영향을 받는 수정을 중단하고 Status를
`policy_decision_required`로 변경한다.

## 12. 구현 범위

- 다섯 Maintenance action을 common/root capability projection 후보에 포함한다.
- Maintenance context이 아니면 `Irrelevant`로 판정한다.
- action별 대응 operation에서 하나 이상의 `can_execute` payload가 있으면 `Enabled`, 없으면
  `Disabled(NoEligiblePayload)`로 판정한다.
- `facility.rs`에 action-to-target-kind/operation mapping을 한 곳에 두고 기존
  `maintenance_options()` preview의 `can_execute`만 집계한다. 별도 recipe/재료 판정 authority를 만들지
  않는다.
- 다섯 behavior가 common capability preflight 대상이라면 projection과 같은 context gate를 거친 뒤 선택한
  payload의 기존 validator로 이어지게 한다.
- server와 Unity에 새 schema를 추가하지 않고 기존 typed capability entry를 전달·소비한다.
- 실제 조사에서 이 범위를 넘는 독립 source-of-truth 변경이 필요하면 구현하지 않고 중단 조건을 적용한다.

## 13. Migration

### DTO migration

없음이 승인 목표다. 기존 `ActionCapabilityDto`, `ActionKind`, Maintenance option DTO와 command payload를
유지하고 현재 context에서 누락된 capability entry만 생산한다. Current source 조사에서 breaking DTO가
필요하면 `policy_decision_required`다.

### Data 또는 live RON migration

없음. Existing live recipe/content가 실행 가능한 대표 payload를 제공하는지 검증하되 schema, recipe와
balance는 변경하지 않는다. Live data 변경 없이는 완료 조건을 충족할 수 없다면 현재 Goal을 확장하지
않고 중단한다.

### Save migration

없음. Capability와 Maintenance options는 derived projection이며 save authority가 아니다.

## 14. 필수 Focused 검증

Current discovery에서 다음 검증 경계를 확정했다. 실제 test 이름과 runner 결과는
`EXPERIMENTS.md`에 기록한다.

- Maintenance 밖에서 다섯 capability가 모두 생략되는 Core 회귀
- Maintenance context에서 action별 실행 가능한 payload가 있을 때 enabled되는 회귀
- 관련 target은 있으나 실행 가능한 payload가 없을 때 `NoEligiblePayload`로 disabled되는 회귀
- 다섯 action과 target kind/operation mapping 회귀
- enabled capability와 선택한 valid payload의 command 성공
- stale/invalid payload의 최신-state 재검증과 비부분 mutation
- blocking Visual Novel 및 node 종료 뒤 Maintenance capability가 노출되지 않는 회귀
- root serialized snapshot의 exact action/scope/enabled/disabled-reason shape
- game_server가 Core capability를 재해석하지 않고 보존하는 focused 검증
- Unity가 최신 capability와 operation eligibility를 함께 사용하고 absent/disabled action을 전송하지 않는
  EditMode/PlayMode 검증
- actual local game_server를 사용하는
  `LiveAdmin_MaintenanceFixtureExposesOperationOptions` 실행

0개 test를 실행한 filter는 증거로 사용하지 않는다.

## 15. 필수 Broad 검증

- Core lib 전체
- 관련 live RON loading/validation suite
- game_server 전체 또는 조사 결과 정당화된 가장 작은 broad server suite
- Rust workspace all targets
- Unity full EditMode와 PlayMode
- explicit live-server Maintenance flow 및 인접 explicit live flow의 회귀 여부
- Rust formatting check, 대상 scoped `git diff --check`, Unity 대상 scoped diff check
- capability/Maintenance legacy 및 local-inference symbol search

실제 명령, runner job ID, test 수와 pass/fail/skip은 실행 뒤에만 `EXPERIMENTS.md`에 기록한다.

## 16. 생략할 검증과 근거

Goal 초기화 시점에 승인된 생략은 없다.

조사 결과 특정 계층이 현재 계약과 무관하면 파일/호출 근거와 영향을 기록하고 생략할 수 있다. 필수
Unity consumer, local server 또는 live data 검증에 접근할 수 없고 대체 증거로 사용자-visible 완료
조건을 입증할 수 없으면 Status를 `externally_blocked`로 변경한다.

## 17. Dirty Worktree Gate

- 2026-08-03 Goal 생성 직전 `/mnt/f/work/simulator`와 `/mnt/f/unity projects/ark` 모두 광범위한 tracked 및
  untracked 기존 변경이 있었다.
- Goal 경로는 생성 전 존재하지 않았다.
- Production 수정 전에 관련 저장소 status와 대상 파일별 current diff를 다시 읽고 기존 변경과 새 Goal
  변경을 구분한다.
- 기존 변경은 사용자 소유로 보존한다. 안전하게 병합할 수 없는 overlap이면 해당 파일을 수정하지 않고
  Status와 blocker를 기록한다.
- Reset, restore, clean, stash, repository-wide format과 관련 없는 파일 삭제를 수행하지 않는다.

## 18. Compatibility

승인된 compatibility는 없다.

Legacy `allowed_actions`, Unity-local inferred capability, dual projection, fallback, alias, mock-only production
branch를 추가하지 않는다. 배포 compatibility가 새로 필요하면 지원 대상·기간·종료 조건과 전용 테스트에
대한 사용자 승인 전에는 구현하지 않는다.

## 19. 중단 조건

- 여러 독립적인 source-of-truth 경계를 함께 바꿔야만 완료 가능한 경우
- 새 Unity-facing DTO, disabled reason/UX 의미, recipe/balance 또는 save 정책이 필요한 경우
- 기존 `maintenance_options.can_execute`와 실제 command validator가 서로 다른 승인되지 않은 gameplay
  의미를 가진 경우
- live data 변경 없이는 executable payload를 만들 수 없는 경우
- 필수 Unity project, local server, live data, dependency 또는 runner에 접근할 수 없는 경우
- 대상 dirty 사용자 변경과 안전하게 병합할 수 없는 경우
- production 변경 전 필수 baseline을 확보하지 못해 변경 전후 회귀를 구분할 수 없는 경우

범위 분리가 필요하면 `scope_decision_required`, 정책이면 `policy_decision_required`, 필수 외부 요소면
`externally_blocked`로 변경한다. 현재 Goal 밖 문제는 `EXPERIMENT_NOTES.md`의 후속 후보로만 남긴다.

## 20. 완료 조건

- 모든 사용자-visible 완료 조건과 유지 불변식에 current source/test 증거가 연결된다.
- Core Maintenance domain, root capability, game_server와 접근 가능한 Unity consumer가 같은 계약을 사용한다.
- 기존 operation validator를 재사용하고 경쟁하는 eligibility authority가 없다.
- DTO/live RON/save migration이 없거나, 예상과 달랐다면 승인된 정책과 근거가 기록돼 있다.
- 승인되지 않은 compatibility/fallback/local inference가 없다.
- 필수 focused/broad 검증이 통과하고 실제 test 수와 결과가 기록된다.
- 기존 baseline failure와 새 failure가 구분된다.
- 관련 canonical이 current implementation과 일치한다.
- Goal 범위 밖 변경이 섞이지 않고 기존 사용자 변경이 보존된다.
- 필수 미검증, policy question과 blocker가 없다.

위 조건을 모두 충족하기 전에는 Status를 `complete`로 변경하지 않는다.

## 21. 완료 증거

2026-08-03 최종 상태에서 다음 증거를 연결했다. 정확한 명령, runner job ID, 중간 실패와 수치는
`EXPERIMENTS.md`에 기록한다.

| 완료 조건/불변식 | 실제 증거 |
| --- | --- |
| 다섯 action의 exact target kind/operation mapping | `facility.rs::maintenance_action_has_executable_payload`; live equipment/fragment mapping 회귀 |
| Maintenance 밖 absent, 안에서 payload 없음 typed disabled | `independent_maintenance_node_exposes_maintenance_actions`; exact root JSON의 `scope: gameplay`, `enabled: false`, `no_eligible_payload` assertion |
| 실행 가능한 payload가 있을 때 다섯 action enabled | `live_maintenance_capabilities_follow_executable_operation_projection`; embedded live recipe/catalog 사용 |
| Blocking VN 및 node 완료 뒤 capability 미노출 | `independent_maintenance_node_exposes_maintenance_actions`의 blocking session/complete-node assertions |
| Projection과 command preflight 일치 | `action_capability.rs`가 snapshot/common preflight에서 같은 Maintenance option mapping을 사용; Core lib/all-targets 통과 |
| Selected payload 최신 재검증과 원자성 | enabled family에서 invalid UUID가 `InventoryItemNotFound`를 유지하고 command 전후 root snapshot exact 동일 |
| Valid payload 성공 | 기존 equipment/skill-fragment 성공 회귀 28개와 Core lib 799개 통과 |
| game_server pass-through | game_server lib 54/54와 실제 local server를 통한 Unity live test 통과 |
| Unity absent/disabled 전송 차단 | `MaintenanceCommandHandler_DoesNotSendWhenLatestCapabilityIsAbsentOrDisabled` 2/2; 실제 store/bus/gateway/handler와 transport 0건 |
| 실제 사용자-visible live 실행 | `LiveAdmin_MaintenanceFixtureExposesOperationOptions` baseline 0/1에서 변경 후 1/1; server log의 실제 fragment/equipment reward mutation |
| Live data/schema 불변 | `ron_loading` 23/23; live RON 수정 없음 |
| Broad 회귀 없음 | Rust all-targets 900/900, Unity EditMode 262/262, PlayMode 62 pass/0 fail/4 explicit skip, explicit LiveServer 4/4 |
| Formatting/diff/legacy audit | `cargo fmt --all -- --check`, 두 repository scoped `git diff --check`, trailing-whitespace 및 legacy/local-inference search 통과 |
| Canonical 일치 | `docs/game_rulebook.md`, `docs/core_runtime_contract.md`, Unity `docs/unity_core_contract.md` 갱신 |
| Migration/compatibility 없음 | DTO/live RON/save 변경 없음; fallback/alias/dual read-write/local inference search 결과 없음 |
| Dirty 사용자 변경 보존 | 관련 status/scoped diff 재확인; reset/restore/clean/stash/repository-wide format 미사용 |

### 최종 판정

- 감사 finding ID: 없음.
- 미결정 정책, scope blocker, 외부 blocker와 필수 미검증 항목이 없다.
- Baseline explicit live failure는 변경 후 해소됐고 최종 필수 focused/broad 검증에 새 실패가 없다.
- 따라서 Status는 `complete`다.
