# Experiment Notes

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`

## Source precedence

`game_rulebook.md`를 최신 gameplay 정책으로, current runtime을 stale할 수 있는 구현 evidence로 취급한다.
Core/data/skill contract는 relevant section마다 상태를 판정하고 Unity contract는 stale historical reference로만
사용한다. 모든 candidate 문서를 일괄 정독하지 않고 criterion별 필요한 section과 직접 필요한 문맥만 읽는다.
Goal 실행 중 새 conflict나 source role 변경을 발견하면 다음 investigation 전에 여기에 기록하고
`DOCUMENT_SOURCE_MAP.md`를 갱신한다.

각 source read는 다음을 남긴다.

- criterion/change scenario와 질문
- 읽은 heading 또는 line range와 필요한 cross-reference
- 선택 이유와 읽지 않은 section/문서의 이유
- 판정한 역할: `normative`, `current technical contract`, `future pressure`, `historical`, `irrelevant`
- Rulebook/current code와 충돌할 때의 처리 상태

## Past-audit access lock

A00~A11 독립 1차 판정이 잠기기 전 다음 경로는 접근 금지다.

- `docs/goals/current_runtime_structural_integrity_audit/**`
- 그 밖의 과거 `docs/goals/**`

현재 상태: `unlocked after E-022; selected cross-check completed as E-023`

## Bounded investigation queue

현재 queue: none; E-025 completion audit complete.

## Adjacent access notes

S01에서 compiled Boss Runtime과 authored encounter 연결을 확인하기 위해 DATA-001/002만 직접 읽었다. 최초 접근
시점의 adjacent RON/JSON aggregate identity와 개별 identity는 `DOCUMENT_SOURCE_MAP.md`에 기록했다.

## Additional probe pre-registration

E-020/E-021 probe는 실행 전에 다음으로 한정했다: current common command/tick/snapshot/checkpoint 비용, retained
history/entity scaling, public save/continue의 다음 행동 동등성. `/tmp/core_arch_perf`만 작성하고 production test는
수정하지 않았다. 결과와 한계는 `EXPERIMENTS.md`, 예산은 `PERFORMANCE_BUDGET.md`에 기록한다.

## Architecture alternatives under consideration

E-024에서 다섯 대안을 동일 evidence로 비교했다. Application/protocol shell과 battle-session/data-construction
boundary의 단계적 재작성이 권고되며 whole-core big bang은 증거 요건을 충족하지 못했다.

## Out-of-scope follow-up candidates

- Target transaction/application shell, indexed battle archive, validated/scoped data, typed session state and versioned
  protocol implementation are separate user-approved follow-up work; this Goal creates no remediation Goal automatically.
- GAP-001/002 release policy and certification work remains explicitly outside this architecture decision.

## Corrections and sequencing deviations

- SRC-03 selected prose says 23 gameplay sources while current macro/exact inventory owns 24; classified as documentation drift.
- E-020 release measurements are not numerically combined with the past audit's different debug/synthetic probe.
- No past-audit path was accessed before E-022; E-023 records the first bounded cross-check.
