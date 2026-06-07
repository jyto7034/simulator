# Post Policy Test Verification Goal

이 goal은 2026-06 core policy implementation 이후 테스트 체계가 최신 정책을 실제로 검증하는지 확인하는 audit 작업이다.

이 goal의 기본 산출물은 테스트 검증 보고서다. 실패한 테스트를 바로 고치거나 기대값을 바꾸지 않는다. 실패 원인과 정책 충돌 여부를 기록하고, 수정은 사용자 승인 후 별도 goal로 진행한다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/post_policy_test_verification/PLAN.md
docs/goals/post_policy_test_verification/EXPERIMENTS.md
docs/goals/post_policy_test_verification/EXPERIMENT_NOTES.md
docs/goals/post_policy_test_verification/REPORT.md
```

## Engineering Principles

- 이 goal은 verification/audit goal이다. 테스트 실패를 바로 수정하지 않는다.
- 테스트가 최신 사용자-visible behavior, Unity-facing DTO, data validation, live RON loading, 실제 gameplay flow를 고정하는지 확인한다.
- 내부 구현 모양만 고정하는 brittle test는 부채로 기록한다.
- 과거 정책을 고정하는 테스트는 삭제/교체 후보로 기록하되 이 goal 안에서 수정하지 않는다.
- 0개 테스트가 실행되는 filter 명령을 성공 검증으로 착각하지 않는다.
- 테스트 실패는 실패 원인, 관련 정책, 수정 후보, 필요한 사용자 결정 여부를 기록한다.
- 사용자와 의논하여 정해야 할 정책이 발견되면 goal을 종료한다.

## Goal Execution Contract

- source of truth 우선순위는 실제 runtime code, live RON/data, Unity-facing snapshot/command 계약, 최신 정책 문서 순서로 본다.
- 테스트와 코드/문서가 충돌하면 어떤 쪽이 stale한지 판단 근거를 기록한다.
- 이 goal은 기본적으로 코드와 테스트 파일을 수정하지 않는다.
- 테스트 실행은 작은 범위에서 시작해 점진적으로 넓힌다.
- 실패가 나오면 같은 명령을 무작정 반복하지 않고, 실패 로그와 의심 원인을 기록한다.
- long-running/full test는 비용과 목적을 `PLAN.md`에 적고 실행한다.
- 완료 시 “필수 검증 통과 목록”, “실패/불안정 테스트”, “0개 실행 명령”, “중복/취약 fixture”, “후속 테스트 정리 goal”을 보고한다.

## Git And File Safety

- `git restore`, `git reset` 등 git file modifier 명령은 수행하지 않는다.
- git의 과거 내역 코드를 복사하여 현재 파일에 `cp`하지 않는다. 만약 해야 한다면 사용자의 허락을 무조건 받는다.
- 파일을 과거로 돌리는 경우 이유를 설명하고 사용자의 허락을 무조건 구한다.
- worktree가 dirty여도 사용자 변경으로 간주하고 되돌리지 않는다.

## Canonical Unity Docs

Unity-facing 계약/구현 문서의 canonical 위치는 `F:\unity projects\ark\docs`다. Linux/WSL 경로로는 `/mnt/f/unity projects/ark/docs`다.

Unity-facing DTO 테스트를 해석할 때 아래 외부 canonical 문서를 확인한다.

- `F:\unity projects\ark\docs\unity_core_contract.md`
- `F:\unity projects\ark\docs\unity_client_implementation_goal.md`

이 저장소 안의 같은 이름 문서는 stale copy일 수 있다.

## Source Of Truth

우선 읽을 문서:

- `docs/core_policy_decisions_2026_06.md`
- `docs/goals/core_policy_implementation_master/EXPERIMENTS.md`
- `docs/goals/core_policy_implementation_master/EXPERIMENT_NOTES.md`
- 완료된 하위 goal들의 `EXPERIMENTS.md`
- `docs/post_policy_technical_debt_review_goal.md`

우선 확인할 테스트 영역:

- `tests/ron_loading.rs`
- `tests/skill_refactor_validation.rs`
- `tests/skill_test_suite.rs`
- `tests/live_item_skill_activation.rs`
- `tests/live_skill_catalog_audit.rs`
- `tests/unit_test.rs`
- `src/game/**` 내부 `#[cfg(test)]` 모듈
- `../game_server` 테스트 및 compile path

우선 확인할 기능 축:

- Damage feedback DTO
- Combat preview threat warnings
- Abnormality attempt/retreat/re-entry
- Consumable modifier re-entry duration
- Unit overlap and blocking
- Movement backend policy and Rapier quality
- Airborne enemy mobility
- Weapon archetype and targeting
- Skill fragment compatibility
- AD/AP balance validation

## Verification Axes

### Command Coverage

- 각 완료 goal에서 기록한 검증 명령이 실제로 테스트를 실행했는지 확인한다.
- `cargo test -p game_core <filter>` 명령이 0개 테스트를 실행한 경우를 찾는다.
- full test와 integration test가 누락된 영역을 기록한다.

### Policy Coverage

- 최신 정책별로 최소 하나 이상의 focused test가 있는지 확인한다.
- DTO serialization shape와 Unity-facing field가 테스트로 고정되어 있는지 확인한다.
- live RON/data validation이 최신 schema를 실제로 읽는지 확인한다.

### Regression Risk

- fixture가 너무 많은 정책을 우회하거나 기본값으로 숨기는지 확인한다.
- brittle float equality, seed nondeterminism, ordering/tie-break 취약성을 찾는다.
- generated preview/wave처럼 seed 기반 테스트가 대표성 있는지 확인한다.

### Test Maintainability

- 중복 fixture, stale helper, 레거시 naming, ignored test, 과거 정책 expectation을 찾는다.
- 테스트가 실패했을 때 무엇이 깨졌는지 알기 어려운 broad assertion을 기록한다.
- 새 source of truth를 우회하는 manual construction test를 기록한다.

## In Scope

- 테스트 명령 실행과 결과 기록.
- 테스트 목록/필터/0개 실행 여부 확인.
- 실패 로그 분석.
- coverage gap audit.
- live RON validation audit.
- game_server compile/test 필요 여부 확인.
- 후속 테스트 정리 goal 후보 작성.

## Out Of Scope

- 테스트 코드 수정.
- production 코드 수정.
- live RON 수정.
- 새 테스트 작성.
- 기대값 변경.
- 실패 테스트를 ignored 처리.

## Suggested Verification Order

작업자는 환경과 시간 비용을 고려해 조정할 수 있지만, 실행 순서와 이유를 `PLAN.md`에 기록한다.

```text
cargo check -p game_core
cargo check -p game_server
cargo test -p game_core feedback_tags -- --nocapture
cargo test -p game_core combat_preview -- --nocapture
cargo test -p game_core retreat -- --nocapture
cargo test -p game_core consumable -- --nocapture
cargo test -p game_core movement -- --nocapture
cargo test -p game_core fixed_defense -- --nocapture
cargo test -p game_core airborne -- --nocapture
cargo test -p game_core targeting -- --nocapture
cargo test -p game_core skill_fragment -- --nocapture
cargo test -p game_core --test ron_loading -- --nocapture
cargo test -p game_core --test skill_refactor_validation -- --nocapture
cargo test -p game_core --test skill_test_suite -- --nocapture
cargo test -p game_core -- --nocapture
```

`cargo test -p game_core -- --nocapture`가 너무 무겁거나 실패가 많은 경우, 먼저 실패 범위를 좁히고 `REPORT.md`에 이유를 기록한다.

## Report Format

`REPORT.md`는 아래 구조를 따른다.

```text
# Post Policy Test Verification Report

## Executive Summary

## Commands Run

## Passing Verification

## Failed Verification

## Zero-Test Or Suspicious Commands

## Coverage Gaps By Policy

## Brittle Or Stale Tests

## Suggested Test Cleanup Goals

## Policy Questions For User
```

각 실패/의심 항목은 아래 필드를 포함한다.

```text
- Command:
- Result:
- Tests Executed:
- Area:
- Evidence:
- Likely Cause:
- Policy Conflict:
- Recommended Follow-up:
- Needs User Policy Decision:
```

## Completion Conditions

- 작업 기억장치가 생성되고 최신 상태로 갱신된다.
- 주요 검증 명령을 실행했거나, 실행하지 못한 명확한 이유를 기록했다.
- 0개 테스트 실행 명령과 의심스러운 성공을 분리했다.
- 최신 정책별 coverage gap을 정리했다.
- 실패/불안정/취약 테스트를 수정 없이 보고했다.
- 후속 테스트 정리 goal 후보가 작성된다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- `cargo check`가 실패해 이후 테스트 해석이 무의미하다.
- full test 실행이 환경/시간/외부 의존성 때문에 불가능하다.
- 실패 테스트가 최신 정책과 과거 정책 중 어느 쪽이 맞는지 사용자 결정이 필요하다.
- 테스트 수정 없이는 더 이상 원인 분석을 진행할 수 없다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

