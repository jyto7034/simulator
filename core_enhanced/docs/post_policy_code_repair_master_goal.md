# Post Policy Code Repair Master Goal

이 master goal은 최근 두 audit goal의 결과를 순서대로 실제 코드 수정으로 연결한다.

순서는 반드시 아래와 같다.

1. `post_policy_test_verification`에서 발견한 깨진 테스트/계약 검증을 먼저 green으로 만든다.
2. 그 다음 `post_policy_technical_debt_review`에서 발견한 구조 부채를 별도 sub-goal 단위로 정리한다.

이 문서는 구현자가 다음 세션에서 길을 잃지 않도록 순서, source of truth, 중단 조건, 완료 조건을 고정한다.

## Source Documents

반드시 먼저 읽을 문서:

- `docs/post_policy_test_verification_goal.md`
- `docs/goals/post_policy_test_verification/REPORT.md`
- `docs/goals/post_policy_test_verification/EXPERIMENTS.md`
- `docs/goals/post_policy_test_verification/EXPERIMENT_NOTES.md`
- `docs/post_policy_technical_debt_review_goal.md`
- `docs/goals/post_policy_technical_debt_review/REPORT.md`
- `docs/goals/post_policy_technical_debt_review/EXPERIMENTS.md`
- `docs/goals/post_policy_technical_debt_review/EXPERIMENT_NOTES.md`
- `docs/core_policy_decisions_2026_06.md`
- `docs/goals/core_policy_implementation_master/EXPERIMENTS.md`

Unity-facing 계약 문서는 이 저장소 내부 copy가 아니라 아래 외부 문서를 canonical로 본다.

- `F:\unity projects\ark\docs\unity_core_contract.md`
- `F:\unity projects\ark\docs\unity_client_implementation_goal.md`
- WSL path:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

## Working Method

이 master goal을 시작하면 아래 작업 기억장치를 만든다.

```text
docs/goals/post_policy_code_repair_master/PLAN.md
docs/goals/post_policy_code_repair_master/EXPERIMENTS.md
docs/goals/post_policy_code_repair_master/EXPERIMENT_NOTES.md
docs/goals/post_policy_code_repair_master/REPORT.md
```

각 sub-goal을 실제로 수행할 때도 가능하면 독립 작업 기억장치를 만든다. 예:

```text
docs/goals/post_policy_test_suite_green/PLAN.md
docs/goals/game_server_unity_dto_test_refresh/PLAN.md
docs/goals/server_behavior_result_dto_typing/PLAN.md
```

작업 중 판단, 실패한 접근, 테스트 결과, 정책 질문은 반드시 goal 문서에 남긴다.

## Engineering Rules

- 코드 수정은 장기적인 방향으로 한다. 임시방편, 최소 수정, 특정 테스트만 통과시키는 패치를 피한다.
- 문서를 무조건 신뢰하지 않는다. 실제 runtime code, live RON/data, Unity-facing snapshot/command 계약, 최신 정책 문서 순서로 source of truth를 확인한다.
- 레거시는 과감하게 제거한다. compatibility layer, fallback path, dual schema는 기본적으로 만들지 않는다.
- 테스트가 예전 정책을 고정하고 있으면 기대값만 바꾸지 말고 삭제하거나 최신 정책 테스트로 교체한다.
- 새 정책에 대한 사용자-visible behavior, Unity-facing DTO, live RON loading, data validation, gameplay flow를 테스트로 고정한다.
- 0개 테스트를 실행한 cargo filter를 성공으로 보지 않는다. 모든 검증 명령은 실행 test count를 기록한다.
- `cargo test -p game_core ron_loading` 같은 integration test 이름 filter를 쓰지 않는다. integration target은 `cargo test -p game_core --test ron_loading`처럼 실행한다.

## Git And File Safety

- `git restore`, `git reset` 등 git file modifier 명령은 수행하지 않는다.
- git의 과거 내역 코드를 현재 파일에 `cp`하지 않는다.
- 파일을 과거로 돌려야 한다면 이유를 설명하고 사용자 허락을 먼저 받는다.
- worktree가 dirty여도 사용자 변경으로 간주하고 되돌리지 않는다.
- unrelated 변경은 건드리지 않는다.

## Phase 1. Test And Contract Green

목표: `post_policy_test_verification`에서 발견한 깨진 검증을 먼저 고친다. 이 phase가 끝나기 전에는 큰 구조 리팩토링으로 넘어가지 않는다.

### 1.1 `post_policy_test_suite_green_goal`

Source:

- `docs/goals/post_policy_test_verification/REPORT.md`

문제:

- `cargo test -p game_core`가 실패한다.
- 실패 테스트: `game::events::combat::tests::authored_protect_unit_tactical_plan_overrides_default_defense_contract`
- 실패 원인 후보: test fixture가 `black_box_breach_main` route id를 하드코딩하지만 generated battlefield에 해당 route가 없을 수 있다.

수행:

- 실패 테스트와 관련 runtime path를 다시 읽는다.
- strict battlefield route validation을 약화하지 않는다.
- fixture가 최신 정책을 표현하도록 고친다.
- 더 나은 장기 방향이 있으면 test fixture 수정 대신 runtime/data boundary를 고쳐도 된다. 단, validator를 느슨하게 만들어 통과시키지 않는다.

완료 조건:

- 실패 원인과 수정 이유가 goal 문서에 기록된다.
- `cargo test -p game_core authored_protect_unit_tactical_plan_overrides_default_defense_contract -- --nocapture`가 실제 테스트 1개 이상을 실행하고 통과한다.
- `cargo test -p game_core`가 통과한다.

### 1.2 `game_server_unity_dto_test_refresh_goal`

Source:

- `docs/goals/post_policy_test_verification/REPORT.md`
- 외부 Unity canonical docs

문제:

- `cargo test -p game_server -- --list`가 test cfg에서 compile fail한다.
- stale fixture가 current DTO/schema field를 빠뜨렸다.
  - `AbnormalityMetadata.mobility_kind`
  - `AbnormalityMetadata.target_traits`
  - `LiveBattleDeploymentDto.unit_deploy_costs`

수행:

- 서버 test fixture를 최신 core 계약에 맞춘다.
- 단순히 빈 필드를 채우는 데서 끝내지 말고 Unity-facing payload에 중요한 필드가 직렬화되는지 assert한다.
- 최소 포함할 assert:
  - `deployment.unit_deploy_costs[*].base_deploy_cost`
  - `deployment.unit_deploy_costs[*].effective_deploy_cost`
  - battle/timeline payload의 `mobility_kind`
  - 가능하면 `feedback_tags`와 `damage_type`

완료 조건:

- `cargo test -p game_server -- --list`가 compile되고 실제 test 목록을 출력한다.
- 관련 focused server tests가 실행되고 통과한다.
- `cargo check -p game_server`가 통과한다.

### 1.3 `cargo_test_filter_hygiene_goal`

Source:

- `docs/goals/post_policy_test_verification/REPORT.md`

문제:

- 이전 goal들에서 아래 명령이 0개 테스트를 실행했다.
  - `cargo test -p game_core damage_feedback`
  - `cargo test -p game_core ron_loading`
  - `cargo test -p game_core skill_refactor_validation`
  - `cargo test -p game_core skill_test_suite`
- `tests/unit_test.rs`는 empty target이다.

수행:

- goal 문서, README, test command 문서에서 0-test filter를 찾아 올바른 명령으로 교체한다.
- `tests/unit_test.rs`는 제거하거나 실제 의미 있는 테스트로 채운다. 빈 target을 유지하지 않는다.
- 자동화가 필요하면 test count를 확인하는 작은 script/checklist를 추가한다. 단, 도입 비용이 크면 문서화만 먼저 해도 된다.

완료 조건:

- 새/갱신 문서에서 integration target은 `--test <target>` 형태로 안내된다.
- `cargo test -p game_core --test unit_test`가 더 이상 0-test 성공으로 남지 않는다. 제거하거나 의미 있는 테스트가 실행된다.
- REPORT에 고쳤거나 의도적으로 남긴 항목이 기록된다.

### 1.4 Focused Coverage Gap Tests

Phase 1의 필수 green 작업이 끝난 뒤, 아래 coverage gap을 작고 명확한 테스트로 보강한다.

우선순위:

1. attempt/retreat edge cases
   - third-attempt exhaustion
   - command rejection after `BattleEnd`
   - all units dead but `BattleEnd` not yet emitted
   - re-entry command availability
2. server Unity DTO coverage
   - `feedback_tags`
   - `damage_type`
   - `mobility_kind`
   - `unit_deploy_costs`
3. threat warning false-rumor lifecycle
   - 20% false rumor
   - retreat/re-entry `Disproved`
   - unused `Observed` contract removal

완료 조건:

- 각 추가 테스트는 최신 정책의 사용자-visible behavior 또는 Unity-facing contract를 고정한다.
- broad filter가 아니라 focused command와 full relevant command를 둘 다 실행한다.

## Phase 1 Integration Gate

Phase 1 완료 전 반드시 실행한다.

```text
cargo check -p game_core
cargo check -p game_server
cargo test -p game_core --test ron_loading
cargo test -p game_core --test skill_refactor_validation
cargo test -p game_core --test skill_test_suite
cargo test -p game_core --test live_item_skill_activation
cargo test -p game_core --test live_skill_catalog_audit
cargo test -p game_core
cargo test -p game_server -- --list
```

`game_server`의 전체 test command가 너무 크거나 외부 의존성이 있으면 이유를 기록하고 가능한 focused server tests를 실행한다.

## Phase 2. Technical Debt Repair

목표: `post_policy_technical_debt_review`의 Medium/Low 항목을 장기적인 구조로 정리한다.

Phase 2는 Phase 1이 green이 된 뒤 시작한다.

### 2.1 `server_behavior_result_dto_typing_goal`

Source:

- `docs/goals/post_policy_technical_debt_review/REPORT.md` M3

문제:

- server boundary가 `BehaviorResult`를 manual `json!` payload로 조립한다.
- field rename/missing field가 compile 단계에서 충분히 보호되지 않는다.

수행:

- typed serializable payload structs를 도입한다.
- 서버 mapping은 string key 중심 `json!` 조립에서 typed DTO serialization으로 이동한다.
- Unity-facing shape가 바뀌는 경우 외부 Unity docs를 갱신한다.
- shape 변경이 의도되지 않았다면 기존 serialized shape를 테스트로 고정한 뒤 내부 구현만 typed로 바꾼다.

완료 조건:

- 주요 `BehaviorResult` variants가 typed payload를 사용한다.
- 최소 battle/live/deployment/skill/timeline 관련 payload가 compile-time field protection을 받는다.
- server focused tests와 `cargo test -p game_server -- --list`가 통과한다.

### 2.2 `combat_preview_validation_boundary_goal`

Source:

- `docs/goals/post_policy_technical_debt_review/REPORT.md` M1

문제:

- `GameDataBase::new`가 combat preview generation을 실행해 warning contract를 검증한다.
- data construction과 runtime preview generation boundary가 섞여 있다.

수행:

- `GameDataBase::new`는 structural schema/index/cross-reference validation에 집중하게 한다.
- generated preview validation은 명시 API나 live-data audit/test path로 이동한다.
- live RON test가 이 명시 validation을 호출하게 한다.

완료 조건:

- `GameDataBase::new`가 runtime preview generation에 의존하지 않는다.
- live RON/generated preview contract는 별도 명시 test에서 유지된다.
- `cargo test -p game_core --test ron_loading`가 통과한다.

### 2.3 `effective_profile_projection_helper_goal`

Source:

- `docs/goals/post_policy_technical_debt_review/REPORT.md` L1

문제:

- actual runtime/snapshot effective profile helper와 projected item-slot validation이 profile construction logic을 중복한다.

수행:

- projected equipment slot을 받는 pure helper를 추출한다.
- runtime/snapshot/projected validation이 같은 source-of-truth helper family를 공유하게 한다.

완료 조건:

- 장비 조합/장착으로 active fragment compatibility가 깨지는 경로가 같은 helper를 사용한다.
- 기존 compatibility tests와 equipment tests가 통과한다.

### 2.4 `movement_steering_dead_policy_cleanup_goal`

Source:

- `docs/goals/post_policy_technical_debt_review/REPORT.md` L3

문제:

- unit overlap 허용 정책 이후 anti-overlap steering branch가 기본값 0/neutral 상태로 남아 있다.

수행:

- live movement path에서 죽은 anti-overlap steering logic을 제거하거나 experimental/test-only helper로 분리한다.
- core movement/Rapier는 blocking, targetability, route progress, deploy occupancy, airborne terrain immunity의 source of truth가 되지 않도록 유지한다.

완료 조건:

- 이동 유닛끼리의 overlap 허용 정책이 더 명확해진다.
- `cargo test -p game_core movement -- --nocapture`가 실제 테스트를 실행하고 통과한다.

## Phase 2 Policy-Gated Items

아래 항목은 구현 전 사용자와 먼저 의논한다. 임의로 DTO shape나 gameplay lifecycle을 정하지 않는다.

### P1. `snapshot_effective_profile_error_surface_goal`

Source:

- `docs/goals/post_policy_technical_debt_review/REPORT.md` M4

정해야 할 것:

- roster snapshot에서 effective profile 계산 실패 시 Unity에 어떻게 전달할지.
  - snapshot 전체 실패
  - `effective_profile_error` field 추가
  - 별도 invalid-profile report DTO

정책 확정 전 완료 조건:

- 사용자에게 DTO shape 선택지를 보고하고 goal을 중단한다.

### P2. `legacy_resource_file_cleanup_goal`

Source:

- `docs/goals/post_policy_technical_debt_review/REPORT.md` L2

정해야 할 것:

- `../game_resources/data/equipments.ron` 같은 stale top-level resource file을 제거해도 되는지.
- 외부 authoring tool이 아직 참조하는지.

정책 확정 전 완료 조건:

- 사용자 허락 없이 외부 resource file을 삭제하지 않는다.

## Phase 2 Integration Gate

각 technical debt sub-goal마다 focused tests를 먼저 실행하고, 마지막에 아래를 실행한다.

```text
cargo fmt
cargo check -p game_core
cargo check -p game_server
cargo test -p game_core --test ron_loading
cargo test -p game_core --test skill_refactor_validation
cargo test -p game_core --test skill_test_suite
cargo test -p game_core
cargo test -p game_server -- --list
```

server full tests가 가능하면 실행한다. 불가능하면 이유와 대체 focused server commands를 기록한다.

## Master Completion Conditions

- Phase 1이 완료되어 `game_core` full test 실패와 `game_server` test cfg compile 실패가 해결된다.
- 0-test filter false positive가 문서/프로세스에서 정리된다.
- Phase 2의 non-policy-gated technical debt items가 별도 sub-goal로 완료된다.
- policy-gated items는 사용자와 의논 후 구현하거나, reserved/후속 보류로 명시된다.
- 외부 Unity canonical docs와 runtime/server DTO shape가 다시 일치한다.
- 최종 `REPORT.md`에 다음을 기록한다.
  - 변경 요약
  - 제거한 레거시
  - 새로 고정한 계약
  - 갱신한 테스트
  - 남은 위험
  - 실행한 검증 명령과 test count

## Master Stop Conditions

- 사용자 정책 결정이 필요한 DTO shape, lifecycle, resource deletion 문제가 발견된다.
- `cargo check` 실패가 너무 광범위해 원인 분석 없이 다음 단계로 넘어갈 수 없다.
- 외부 Unity canonical docs와 core runtime 중 어느 쪽이 맞는지 판단할 수 없다.
- worktree의 사용자 변경과 충돌해 안전하게 진행할 수 없다.

Stop condition이 발생하면 임의로 결정하지 말고 질문 목록과 현재까지의 증거를 보고한다.
