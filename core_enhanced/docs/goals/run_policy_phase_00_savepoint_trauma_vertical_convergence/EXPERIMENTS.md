# SavePoint And Gate Trauma Vertical Convergence Experiments

Status: complete

Implementation complete: true

이 문서는 P00의 실제 조사, baseline, 수정과 검증 결과를 기록한다. 아직 production 구현이나
completion review를 수행하지 않았다.

Current-status correction (2026-07-30): 위 문장은 child 초기화 당시의 historical statement다.
Current authoritative status는 `PLAN.md` 및 `COMPLETION_REVIEW.md`와 같이 `Complete / High`
(2026-07-27)이며, 과거 실험 history는 재작성하지 않았다.

## 2026-07-27 — Child Goal 초기화

### 입력

- Master P00 status: `active; not started`
- Hard prerequisite: 없음
- Master completion review: 아직 없음
- Child 경로는 생성 전에 존재하지 않았다.

### 생성한 문서

- `PLAN.md`
- `EXPERIMENTS.md`
- `EXPERIMENT_NOTES.md`

### Production 변경

- 없음. Child 문서 3개만 생성했다.

### 다음 작업

- 필수 policy/canonical/Unity 문서 선독
- scoped dirty diff와 readiness audit
- master baseline 이후 관련 상태가 달라졌는지 확인 후 필요한 baseline 재실행

## 2026-07-27 — 필수 정책·canonical 선독

### 끝까지 읽은 문서

- Master Goal `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
- Child Goal `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
- `docs/README.md`, `engineering_workflow.md`, `goal_completion_review_guide.md`
- SavePoint/Trauma, local compendium, combat power, facility map, elite monolith decision 5개
- `game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md`
- 외부 Unity `unity_core_contract.md`, `unity_client_implementation_goal.md`

### 결과

- P00은 직접 선행 Phase나 completion review가 없는 첫 Phase임을 다시 확인했다.
- SavePoint decision의 target policy는 tagged `TraumaRecovery`, SavePoint 15%/Gate 10%,
  Critical/Breakdown 포함 생존자, Dead 제외, Physical Damage 불변, SavePoint psychology 미생산으로
  구현에 필요한 의미를 닫고 있다.
- 현재 durable Core canonical과 Unity canonical은 `Support`, `Rest`, `SupportState`,
  `ChooseSupport`를 여전히 current contract로 기술한다. 이는 정책 질문이 아니라 P00에서 실제
  producer/consumer와 함께 제거해야 할 migration 대상이다.
- Unity implementation goal은 장기 canonical이 아닌 과거 실행 지침이며, P00에서는 승인 decision과
  최신 실제 Core/wire를 기준으로 관련 Support/Rest 부분만 수렴시킨다.
- Master가 가리킨 시설 정책의 실제 파일명은
  `facility_map_and_node_manifestation_policy_2026-07-25_26.md`다. 존재하지 않는 축약 파일명으로
  처음 읽기를 시도해 실패한 뒤 실제 경로로 전체를 읽었다.

### Production 변경

- 없음. 이 단계는 문서 선독과 Goal 기록만 수행했다.

### 다음 작업

- P00 범위의 Core/RON/server 및 외부 Unity dirty 상태와 diff를 파일별로 감사한다.
- 실제 producer/consumer와 live loader 상태를 확인한 뒤 baseline을 현재 worktree에서 재실행한다.

## 2026-07-27 — P00 dirty worktree gate

### 확인 결과

- 저장소 전체에는 P00과 무관한 대규모 기존 변경이 있으며 모두 보존한다.
- P00 관련 Core/RON/server diff는 하나의 중단된 `Support/Rest -> SavePoint` migration으로
  일관된다. `MapNodeCategory::SavePoint`, `MapNodePayload::SavePoint`, `SavePointSessionState`,
  `SavePointState`는 이미 추가됐고 Support choice runtime 대부분은 제거됐다.
- 같은 diff에 최종 정책과 다른 임시 상태도 남아 있다. RunPolicy와 runtime은 flat
  `save_point_trauma_heal`, percent Gate의 병렬 field를 사용하며 SavePoint completion이
  `RecoveredAtSavePoint` Psychology event를 생산한다.
- live `generation_policy.ron`의 weight는 SavePoint로 바뀌었지만 safe category 목록 두 곳은
  삭제된 `Support`를 보존한다.
- game_server message test는 제거된 `SupportNodeType`/`ChooseSupport`를 참조한다.
- 외부 Unity도 장기 goal들의 대규모 dirty 상태이며 P00 관련 Support/Rest reader/router/
  presenter/mock/test가 남아 있다. 해당 파일의 기존 목적은 SavePoint pass-through와 Safezone/Rest
  분리였고, P00 target은 그 후속 cleanup이므로 전체 파일을 되돌리지 않고 관련 seam만 수정한다.

### Gate 판정

- 기존 변경의 방향과 P00 목표가 같고 충돌 구간도 최종 정책으로 교체해야 하는 임시 migration
  seam으로 식별된다. 사용자 변경을 초기화하거나 반대 의미로 덮어쓸 필요가 없으므로 안전하게
  이어서 작업할 수 있다.
- P00과 무관한 battle, inventory, VN, skill, UI 변경에는 손대지 않는다.

### 다음 작업

- 현재 worktree baseline을 재실행해 stale RON/server 참조의 정확한 실패를 고정한다.
- tagged recovery 및 preview/transaction 구현 surface를 실제 DTO와 producer에서 설계한다.

## 2026-07-27 — 현재 baseline 재검증

Master 기록 이후 관련 파일이 계속 변경된 dirty worktree이므로 현재 상태에서 다시 실행했다.

### `cargo test -p game_core --test ron_loading -- --nocapture`

- 결과: 실패, exit 101
- 실제 수: 23 tests, 1 passed, 22 failed, 0 ignored
- 공통 원인: `data/map/generation_policy.ron`의
  `normal_safe_categories[0] = Support`가 삭제된 enum variant라 strict parse 실패
- Master baseline과 동일한 root cause와 수를 재현했다.

### `cargo check -p game_server`

- 결과: 성공, exit 0
- 의미: production server target은 현재 compile된다. Master의 과거 compile failure는 관련 code가
  그 뒤 변경되어 해소됐지만 test-only stale Support 참조는 별도로 확인해야 한다.

### 다음 작업

- `cargo test -p game_server --no-run`으로 test target baseline을 별도 고정한다.
- stale live RON을 final schema와 함께 수정한 뒤 동일 live-loading gate를 재실행한다.

### `cargo test -p game_server --no-run`

- 결과: 실패, exit 101
- compile errors: 2
- 원인: test module의 삭제된 `SupportNodeType` import와 `PlayerBehavior::ChooseSupport` assertion
- production `cargo check -p game_server`와 test target 상태를 구분해 기록한다.

## 2026-07-27 — Core/live RON/server final recovery 수렴

### 구현

- `RunPolicyData`의 legacy `support` group을 제거하고 다음 category-wide tagged recovery를 추가했다.
  - `save_point_trauma_recovery: TraumaRecovery`
  - `gate_trauma_recovery: TraumaRecovery`
- `TraumaRecovery`는 `Flat { amount } | CurrentPercent { percent }`이며 두 variant 값 모두
  `1..=100`을 검증한다. percent recovery는 현재 Trauma 기준 올림으로 계산한다.
- live `run/policy.ron`은 SavePoint `CurrentPercent(15)`, Gate `CurrentPercent(10)`을 authoring한다.
- live map generation의 category와 safe-category 목록을 모두 `SavePoint`로 전환했다.
- SavePoint/Gate NodePreview에 Core-calculated `trauma_recovery_preview`를 추가했다. 각 employee row는
  eligibility, Physical Damage, recovery 전후 Trauma와 Mental state를 포함한다.
- SavePoint selected-event와 SavePoint command result에도 같은 authoritative preview를 투영한다.
- SavePoint와 Gate completion을 non-battle state rollback guard로 감싸 recovery, pending delivery,
  node/Floor transition과 process-local checkpoint가 오류 시 함께 rollback되게 했다.
- SavePoint의 `RecoveredAtSavePoint` Psychology producer/projection/test를 제거했다.
- stale game_server `SupportNodeType`/`ChooseSupport` positive test를 제거하고 legacy
  `choose_support` payload rejection test로 교체했다.

### 집중 검증

- `cargo check -p game_core`: 성공
- `cargo test -p game_core --no-run`: 성공
- `cargo test -p game_core trauma_recovery -- --nocapture`: 관련 2 passed, 0 failed
- SavePoint recovery/preview focused test: 관련 1 passed, 0 failed
- SavePoint checkpoint focused test: 관련 1 passed, 0 failed
- Gate recovery/preview focused test: 관련 1 passed, 0 failed
- `cargo test -p game_core --test ron_loading -- --nocapture`: 23 passed, 0 failed
- `cargo test -p game_server --no-run`: 성공

### 레거시 검색

Core/live RON/game_server production 경로에서 다음 symbol/schema를 검색했다.

- `SupportNodeType`, `SupportNodeMode`, `SupportSessionState`, `SupportState`
- `ChooseSupport`, `MapNodeCategory::Support`, `MapNodePayload::Support`
- `RecoveredAtSavePoint`, `RestedAtSupportRest`
- legacy flat/percent field와 `StagedSupportEffect`

production 잔존은 없었다. 검색 결과 두 건은 strict rejection fixture와 game_server의
`choose_support` 음성 deserialize test이며 compatibility producer/consumer가 아니다.

### 아직 닫히지 않은 범위

- 외부 Unity runtime/mock/test가 `support` selected-event, `SupportChoice`, `Rest`,
  `ChooseSupport`를 아직 current flow로 소비한다.
- Unity NodeConfirm reader/presentation은 Core `trauma_recovery_preview`를 아직 읽거나 표시하지 않는다.
- durable Core/Unity canonical의 Support/Rest 서술과 recovery 계약을 아직 동기화하지 않았다.

## 2026-07-27 — Unity final consumer 수렴

### 구현

- strict `TraumaRecoveryPreviewModel`을 추가해 exact external tagged JSON의 `Flat.amount` 또는
  `CurrentPercent.percent` 하나만 읽고 1..100 및 직원별 Core 전후 값을 검증한다.
- NodeConfirm의 `game_state_context.trauma_recovery_preview`, SavePoint
  `selected_event.recovery_preview`와 `SavePointState`를 동일한 read model로 연결했다.
- router에서 `save_point`를 독립 route로 읽고 SupportChoice/Rest route, command, UI selection,
  presenter와 `SupportScreenComposer`를 제거했다.
- Node Exploration은 SavePoint/Gate의 Core-calculated mode/value, Physical Damage 불변,
  checkpoint 교체 여부와 직원별 Trauma 전후를 표시한다. Unity 계산은 추가하지 않았다.
- mock과 live-flow test를 SavePoint 15%, Gate 10% preview로 전환하고 구
  `support_save_point`/`support_rest` fixture를 제거했다.
- Roster reader에서 `RestedAtSupportRest`를 제거했다. 현재 Core에 production Psychology event
  variant가 없으므로 임의 event는 fail-closed다.

### Unity 검증

- compile/refresh: console error 0
- focused EditMode: 7 passed, 0 failed
- focused SavePoint PlayMode render: 1 passed, 0 failed
- 첫 full EditMode: 245 total, 244 passed, 1 failed
  - 원인: Support/Rest와 무관한 mock equipment 표시 이름을 이미 `Maintenance Blade`로 바꿨으나
    assertion만 `equipment_rest_blade`를 기대한 stale test
  - 수정 후 focused: 1 passed, 0 failed
- 최종 full EditMode: 245 passed, 0 failed, 0 skipped
- 최종 full PlayMode: 65 total, 61 passed, 0 failed, 4 skipped
  - 4건은 명시적인 live-server opt-in test다.

## 2026-07-27 — strictness·atomicity 강화와 canonical 수렴

### 추가 검증/수정

- `TraumaRecovery` variant에 `deny_unknown_fields`를 추가했다. 최초 mixed-field 음성 test가
  `CurrentPercent(percent, amount)`를 조용히 수용하는 문제를 발견했고 production schema를
  strict하게 고친 뒤 exact tagged parsing 3건이 통과했다.
- SavePoint/Gate 정상 경로에 exact-once assertion을 추가하고 Gate invalid pending 연구가
  employee/Floor/map/session/checkpoint 전체를 바꾸지 않는 rollback test를 추가했다.
- map payload unknown-field test가 삭제된 Support fixture를 더 이상 찾지 못해 broad test에서
  1건 실패했다. 현재 HeadquartersContact nested payload에 `legacy_mode`를 주입하는 동등한 strict
  rejection fixture로 고쳤고 focused test가 통과했다.
- live map row repair field `ensure_support_before_boss`를 실제 의미인
  `ensure_save_point_before_boss`로 교체했다. Core facility module, test module/helper/id의
  stale Support/Rest 명칭도 동작 변경 없이 정리했다.
- Core gameplay/runtime/data canonical과 외부 Unity contract/implementation goal/screen spec을
  SavePoint/Gate tagged recovery, preview, 원자성 및 Psychology 미생산 계약으로 동기화했다.

### 최종 Core/server 검증

- `cargo check -p game_core`: 성공
- `cargo check -p game_server`: 성공
- `cargo test -p game_core --test ron_loading -- --nocapture`: 23 passed, 0 failed
- `cargo test -p game_core`:
  - library 760 passed, 0 failed
  - integration suites 3/4/23/13/4 passed, 0 failed
  - 나머지 binary/doc target은 0-test success
- `cargo test -p game_server`: 31 passed, 0 failed

### 최종 legacy audit

- Core/live RON/server의 official flow에서 SupportState, ChooseSupport, SupportChoice, Rest,
  Support category/payload, removed Psychology event와 stale internal/RON 명칭은 0건이다.
- 남은 `choose_support`/`Rest` 한 건은 game_server가 삭제된 command를 거부하는 음성 test다.
- RunPolicy의 lower-case `support`/Rest 문자열은 삭제된 group을 strict reject하는 음성 fixture다.
- Unity official SavePoint flow의 제거 대상은 0건이다. 남은 `HeadquartersSupportChoice`와
  `headquarters_support`는 별도 HeadquartersContact action/shop domain이며 SavePoint 선택
  mode가 아니다.
