# Maintenance Action Capability Projection Experiment Notes

Status: complete

Implementation complete: true

Created: 2026-08-03

이 문서는 구조적 판단, source-of-truth 충돌, 정책 질문, 외부 의존성, dirty 상태와 남은 위험을 기록한다.
Gameplay/runtime/wire의 durable source of truth가 아니다.

## 1. 구조적 판단

이 Goal은 다섯 개 command를 각각 독립 기능으로 구현하는 작업이 아니다. 하나의 Maintenance domain이
생산하는 대상별 operation eligibility를 하나의 Core typed context capability 경계에 연결하고,
game_server와 Unity가 같은 projection을 소비하게 하는 단일 계약 수정이다.

저장소는 Core/server와 Unity 두 개지만 변경 이유와 사용자-visible 완료 조건은 하나다. DTO shape,
Maintenance gameplay, live recipe와 save를 바꾸지 않는 한 Goal 분리는 필요하지 않다.

## 2. 초기 Source-Of-Truth 충돌

기존 completion/remaining-risk 기록은 다음 충돌을 보고한다.

- Maintenance domain은 실행 가능한 대상별 operation preview를 생산한다.
- Unity는 operation `can_execute`와 root action capability를 함께 요구한다.
- Root common capability projector는 Maintenance 다섯 action을 생산하지 않는다.
- 따라서 실행 가능한 operation이 있어도 Unity command gate가 열리지 않는다.

이는 새 gameplay 정책이 아니라 기존 두 승인된 authority 사이의 연결 누락으로 해석한다. Current source,
serialization, server mapping과 Unity consumer의 재확인은 Goal 문서 초기화 뒤 discovery에서 수행한다.

## 3. 정책 질문

현재 없음.

승인된 기준은 다음과 같다.

- Parameterized action은 실행 가능한 payload가 하나 이상일 때 enabled다.
- 관련 없는 action은 생략하고 관련 있지만 실행 payload가 없으면 typed disabled reason을 사용한다.
- 대상별 상세 eligibility와 비용/결과는 Maintenance domain이 소유한다.
- Unity는 capability를 추론하지 않고 Core projection을 소비한다.
- Command는 최신 상태에서 payload를 재검증한다.

Current source 또는 canonical이 이 의미와 충돌하면 임의로 선택하지 않고
`policy_decision_required`로 전환한다.

## 4. 외부 시스템과 직접 의존성

- Unity 프로젝트: `/mnt/f/unity projects/ark`
- Unity Editor/MCP test runner: focused 및 broad consumer 검증에 필요
- Local `game_server`: explicit live Maintenance flow 검증에 필요
- Live Core data bundle: representative executable Maintenance payload와 recipe validation 확인에 필요

접근 가능 여부와 현재 readiness는 production baseline 단계에서 다시 확인한다. 파일을 읽었다는 사실을
Unity runner 또는 actual live-server 검증으로 표현하지 않는다.

## 5. 범위 밖 후속 후보

초기화 시점에 자동 생성하거나 구현할 후속 Goal은 없다.

조사 중 독립 문제를 발견하면 이 절에 근거와 사용자-visible 영향을 기록하되 새 Goal/Master를 만들거나
현재 patch에 포함하지 않는다. Hidden boss, Maintenance balance/content 확대와 일반 capability 전면
리팩토링은 명시적 비범위다.

## 6. 적용하지 않은 대안

### Unity가 `maintenance_options`만 보고 action을 추론

적용하지 않는다. Core-owned root capability 계약과 command preflight 권위를 중복한다.

### Maintenance 진입 중 다섯 action을 항상 enabled

적용하지 않는다. 실행 가능한 payload가 없는 action을 열어 parameterized capability 불변식을 위반한다.

### Capability projector에서 recipe와 재료 규칙을 별도 재구현

적용하지 않는다. Maintenance preview/validator와 경쟁하는 eligibility authority를 만든다.

### Live fixture 전용 production special case

적용하지 않는다. 실제 gameplay state와 분리된 test-only behavior는 완료 증거가 아니다.

### Maintenance 전체 gameplay/화면 리팩토링

적용하지 않는다. 현재 연결 누락과 독립적으로 구현·검증할 수 있는 범위 확장이다.

## 7. Dirty Worktree 주의사항

- 2026-08-03 신규 Goal 경로 확인 당시 `/mnt/f/work/simulator`와 `/mnt/f/unity projects/ark` 모두 매우
  광범위한 tracked/untracked 변경을 포함했다.
- Goal 경로는 생성 전에 존재하지 않았다.
- 기존 변경은 모두 사용자 소유로 취급한다.
- Production 수정 전 실제 대상 파일의 current content와 scoped diff를 다시 읽어 overlap을 기록한다.
- 안전하게 병합할 수 없는 파일은 수정하지 않으며 reset, restore, clean, stash 또는 unrelated formatting을
  사용하지 않는다.
- 초기 status 전체 출력은 실행 기록의 command 결과에 존재하며, 이 notes에서 사용자 변경 목록을
  축약하거나 현재 Goal 성과로 재분류하지 않는다.

## 8. 접근하거나 검증하지 못한 소비자

- Goal 초기화 단계에서는 production source, serialized payload와 Unity consumer를 새로 조사하지 않았다.
- Unity Editor runner와 local server도 Goal baseline으로 실행하지 않았다.
- 존재 여부가 알려지지 않은 배포 build, 실제 사용자 save, 외부 계정/cloud consumer는 확인하지 않았다.

필수 consumer에 접근할 수 없으면 대체 검증의 충분성을 판단하고, 사용자-visible 완료 조건을 입증할 수
없다면 `externally_blocked`로 전환한다.

## 9. 생략한 검증과 근거

- 신규 Goal 초기화는 문서 생성 단계이므로 production test와 formatter를 실행하지 않았다.
- Goal 활성화 전 별도 감사에서 실행된 Rust/Unity 결과는 current production baseline으로 재사용하지 않는다.
- 정확한 focused/broad runner와 test count는 current source 조사 및 변경 전 baseline에서 기록한다.

## 10. 남은 위험

- Capability preflight를 공통 gate에 추가하면서 선택한 payload의 더 구체적인 domain error를 부당하게
  가릴 수 있다. Any-executable context capability와 selected-payload validator의 책임을 구분해 검증해야 한다.
- Maintenance option producer와 command validator가 현재 완전히 같은 eligibility 의미를 쓰는지는 discovery
  pending이다. 충돌이 gameplay 의미를 요구하면 정책 결정 없이 수정할 수 없다.
- Live fixture가 실제 production recipe/재료 조합을 제공하는지 현재 Goal baseline에서 다시 확인해야 한다.
- 두 저장소의 광범위한 dirty 상태 때문에 target overlap과 새 회귀를 scoped diff와 baseline으로 구분하지
  못하면 Goal을 complete로 판정할 수 없다.
- Unity live test가 local server 환경에 의존하므로 runner 또는 server 접근이 없으면 completion blocker가
  될 수 있다.

## 11. 2026-08-03 Current Source 조사 판단

### 실제 계약 경로

1. `GameCore::get_action_capabilities()`가 root common capability를 계산한다.
2. `get_run_snapshot_dto()`가 계산 결과를 root `action_capabilities`에 넣는다.
3. game_server `build_state_snapshot()`은 Core DTO를 `PlayerStateSnapshotDto`로 직렬화하며 Maintenance
   capability를 재계산하지 않는다.
4. Unity `GameSnapshotStore`가 root capability를 strict typed map으로 읽는다.
5. `MaintenanceOptionsModel`은 대상별 operation preview를 읽고 `MaintenanceCommandHandler`는 최신
   `CanExecute`와 capability gateway를 모두 확인한다.
6. Core command gate 뒤 기존 selected-payload validator와 mutation handler가 최신 state를 다시 검증한다.

### Source-Of-Truth 충돌 판정

- 현재 runtime 결함은 Maintenance preview와 Unity consumer 사이의 Core projection 누락이다.
- Canonical은 관련 없는 action 생략, 관련 있지만 실행 payload가 없는 action의 typed disabled reason,
  Core-owned projection과 command-time 재검증을 이미 요구한다.
- 승인 목표와 canonical 사이에 정책 충돌은 없으며 DTO, live RON, save 또는 compatibility 결정이 필요하지
  않다.

### 기술적 구현 선택

- Action-to-operation mapping은 Core facility 계층의 작은 helper로 둔다.
- Helper는 `maintenance_options()`가 이미 계산한 `can_execute`를 target kind와 operation별로 집계한다.
- Capability 계층에서 recipe, 비용, 보유량 또는 장착 규칙을 다시 구현하지 않는다.
- Context capability가 enabled여도 선택한 payload가 invalid/stale이면 기존 domain validator의 구체
  실패가 유지되도록 common preflight는 any-payload gate까지만 소유한다.

### Dirty overlap

- `world/action_capability.rs`와 `world/facility.rs`는 현재 Git 기준 untracked인 기존 사용자 파일이다.
  전체 파일을 교체하지 않고 좁은 helper/match arm만 병합한다.
- `world.rs`, `helpers.rs`, `maintenance.rs`, `snapshot.rs`, Core/Unity canonical과 Unity consumer/test에는
  광범위한 기존 변경이 있다. 현재 조사상 runtime 수정은 앞의 두 Core 파일로 제한할 수 있다.
- Focused 회귀를 추가할 tracked test 파일도 기존 변경을 포함하므로 기존 hunk를 보존하고 독립 test만
  추가한다.
- Unity production consumer는 이미 승인 계약을 구현하므로 현재로서는 production 변경 대상이 아니다.
  Consumer 검증이 부족하면 기존 test 파일에 focused test만 추가한다.

### 현재 접근 가능한 소비자

- Workspace에서 확인: Core DTO/serializer, game_server wrapper, Unity read model/store/command handler.
- Canonical에 명시된 외부 소비자: Unity ArkClient. 현재 `/mnt/f/unity projects/ark`에서 접근 가능하다.
- 접근 불가 또는 배포 상태 미확인: 배포 build와 실제 사용자 save. DTO/save migration이 없으므로 완료에
  필수 소비자로 분류하지 않되 남은 위험으로 유지한다.

## 12. 2026-08-03 최종 구조 및 위험 판정

### 구현된 단일 계약 경계

- `maintenance_options()`가 생산한 target kind별 operation `can_execute`를 집계하는 Core helper가 다섯
  action family capability의 유일한 입력이다.
- Root snapshot projection과 common preflight가 같은 helper/mapping을 사용한다.
- Family capability는 any-executable-payload까지만 소유하고 선택한 target의 identity/보유/장착/recipe/
  재료 검증과 mutation은 기존 Maintenance domain validator/handler가 계속 소유한다.
- game_server는 Core root capability를 재해석하지 않고 전달하며 Unity는 최신 capability와 선택 option의
  `CanExecute`를 모두 요구한다.

### 해결된 초기 위험

- 구체 domain error 가림: enabled family에서 존재하지 않는 equipment UUID가 기존
  `InventoryItemNotFound`를 유지하고 snapshot을 변경하지 않는 회귀로 해소했다.
- Preview/validator 의미 충돌: live recipe-backed equipment와 awakenable fragment의 모든 다섯 operation이
  preview true, capability enabled, 기존 성공 command test 통과로 일치함을 확인했다.
- Live fixture coverage: 실제 embedded data와 local server/Unity explicit test에서 executable operation을
  찾아 실행했다.
- Dirty overlap: untracked Core production 파일과 광범위하게 수정된 tracked test/canonical에 좁은 hunk만
  병합했다. Reset/restore/clean/stash와 repository-wide formatting을 사용하지 않았다.
- Runner 접근: Core/server/live RON/Unity EditMode/PlayMode/local live server에 모두 접근해 필수 검증을
  완료했다.

### 범위 밖 후속 후보

- 현재 Goal 수행 중 새로 확인된 필수 후속 후보는 없다.
- Live equipment recipe가 `standard_armor` 하나뿐인 점은 기존 콘텐츠 상태이며 recipe 확대와 balance는
  계속 명시적 비범위다.

### 생략한 검증과 근거

- 실제 배포 build와 실제 사용자 save는 접근하지 않았다. DTO, live RON, save format, persistence와
  compatibility를 변경하지 않았고 현재 workspace의 실제 Core producer/server/Unity consumer를 모두
  검증했으므로 Goal 완료의 필수 검증으로 분류하지 않는다.
- Full PlayMode가 `[Explicit]` LiveServer 4개를 skip한 것은 해당 네 test를 `LiveServer` category로 별도
  실행해 4/4 통과한 결과로 보완했다.

### 남은 위험

- 두 저장소에는 Goal 전부터 광범위한 사용자 변경이 계속 존재한다. 이 Goal은 관련 hunk와 테스트만
  검증했으며 전체 dirty 변경의 의미를 보증하지 않는다.
- 첫 인접 live 묶음에서 전투 screenshot pixel assertion이 한 번 실패했지만 단독 1/1과 전체 live
  category 4/4 재실행에서 통과했다. 현재 재현되는 실패는 아니지만 rendering capture timing의 일반적
  변동성은 남는다.
- 실제 배포 artifact 상태는 확인하지 않았으므로 배포 과정의 별도 회귀는 이 Goal 증거에 포함되지 않는다.

### 최종 정책/Compatibility 판정

- 미결정 정책 없음.
- DTO/live RON/save migration 없음.
- compatibility, fallback, alias, dual read/write와 Unity-local capability inference 없음.
- 접근 불가하지만 완료에 필수인 consumer 없음.
