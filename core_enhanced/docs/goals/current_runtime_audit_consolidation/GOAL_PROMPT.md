# Goal Prompt — Current Runtime Audit Consolidation

이 문서는 두 완료된 감사를 하나의 최종 감사 문서로 취합하기 위한 운영 명령이다. production 수정 Goal이 아니며, remediation 설계를 구현하거나 새 transaction abstraction을 만드는 문서가 아니다.

이 문서를 작성한 시점에는 Goal이 시작되지 않는다. 사용자가 이 Goal의 실행을 명시적으로 요청했을 때만 활성화한다.

## 1. Goal identity

- Goal name: `current_runtime_audit_consolidation`
- Goal directory: `docs/goals/current_runtime_audit_consolidation`
- 상태: `complete`
- 대상: `core_enhanced`의 다음 두 완료 감사
  - `docs/goals/current_runtime_structural_integrity_audit/`
  - `docs/goals/current_runtime_architecture_deep_review/`
- 핵심 산출물: `CONSOLIDATED_AUDIT.md`

## 2. Objective

두 감사의 finding, root cause, 보존 자산, rewrite 판단, gap과 후속 권고를 상호 대조하여 하나의 canonical audit로 만든다.

통합 결과는 이후 remediation Goal을 설계하는 단일 입력이 되어야 한다. 어떤 remediation을 먼저 구현할지는 통합 과정에서 근거와 종속성을 비교한 뒤 제안하며, transaction 작업을 미리 정답으로 고정하지 않는다.

## 3. Operating authority

1. `PLAN.md`가 수행 순서와 범위의 source of truth다.
2. `FINDING_CROSSWALK.md`가 source finding의 누락·중복·관계 추적 장부다.
3. `ACCEPTANCE_GATES.md`가 완료 판정의 source of truth다.
4. 두 감사의 완료 문서와 finding catalog가 사실 입력이다.
5. `CONSOLIDATED_AUDIT.md`는 gate를 모두 통과한 뒤에만 이후 작업의 canonical summary가 된다.

두 감사 원문은 immutable evidence로 취급한다. 오기나 충돌을 발견해도 원문을 조용히 수정하지 않고 crosswalk와 통합 문서에 correction note를 남긴다.

## 4. Source access protocol

### 4.1 첫 번째 source tier

다음 결과 문서의 결론, finding catalog, root-cause, preservation/rewrite, gap, handoff 구간을 우선 읽는다.

Structural audit:

- `COMPLETION_AUDIT.md`
- `FINDINGS.md`
- `DESIGN_REVIEW_DRAFT.md`

Architecture review:

- `COMPLETION_AUDIT.md`
- `FINDINGS.md`
- `REWRITE_DECISION.md`
- `PRESERVATION_ASSET_REGISTER.md`

문서 전체를 기계적으로 처음부터 끝까지 읽으라는 요구는 없다. 먼저 heading과 result-bearing section을 식별하고, crosswalk의 특정 판단에 필요한 구간만 읽는다.

### 4.2 두 번째 source tier

다음 자료는 finding 관계, severity, 근거 범위 또는 migration seam을 판정할 때만 읽는다.

- 각 감사의 `EVIDENCE_MATRIX.md`
- structural audit의 authority/transaction/test/data/persistence map
- architecture review의 domain/aggregate/dependency/scenario/operation/evolution/performance map
- architecture review의 `ARCHITECTURE_OPTIONS.md`와 `PAST_AUDIT_CROSSCHECK.md`

### 4.3 세 번째 source tier

production code, tests, rulebook, runtime contract, Unity contract는 기본 입력이 아니다. 다음 경우에만 최소 범위를 확인한다.

- 두 감사의 사실 주장이 서로 충돌함
- source path 또는 symbol이 현재 checkout에서 식별되지 않음
- 최신 rulebook 정책과 감사의 rewrite/preservation 판단이 실제로 충돌하는지 확인해야 함
- 통합 결론을 내리기 위해 audit evidence만으로는 `UNKNOWN`을 피할 수 없음

stale contract는 최신 정책 증거로 승격하지 않는다. 관련 없는 recent feature note도 포괄적으로 읽지 않는다.

### 4.4 제외 source

`docs/goals/current_runtime_transaction_kernel_foundation/`은 감사 완료 전에 만들어진 remediation 초안이므로 이 Goal의 판단 근거로 사용하지 않는다. 통합 결과가 나온 뒤 후속 Goal 후보와 비교할 수는 있지만, source audit이나 canonical decision이 아니다.

## 5. Required work

Goal 실행 시 다음을 수행한다.

1. 두 source audit의 완료 상태, baseline, scope, exclusion을 등록한다.
2. active/rejected/superseded finding을 source별로 exact-set 수집한다.
3. 모든 source finding을 `FINDING_CROSSWALK.md`에 연결한다.
4. 관계를 `same defect`, `corroborates`, `cause/effect`, `same root cause`, `independent`, `conflicts`, `no counterpart` 중 하나로 판정한다.
5. 중복은 canonical finding으로 합치되 source ID와 독립 trigger/impact를 잃지 않는다.
6. severity/confidence가 다르면 더 큰 숫자나 더 강한 표현을 기계적으로 선택하지 말고 차이를 설명한다.
7. 통합 root-cause model을 작성하고 source root cause와 양방향 추적을 유지한다.
8. 도메인 구조, 의존 방향, 확장성, 변경 용이성, 테스트 가능성, 운영 경계의 여섯 축을 각각 최종 판정한다.
9. 보존, refactor, boundary rewrite, subsystem rewrite, defer, rejected 항목을 분리한다.
10. whole-core rewrite 여부를 두 감사 전체 근거로 재확인한다.
11. remediation 후보를 종속성과 위험 기준으로 묶되 구현 상세나 production 변경을 만들지 않는다.
12. `CONSOLIDATED_AUDIT.md`를 작성하고 acceptance gate를 판정한다.

## 6. Prohibited work

- production Rust, tests, live data, server, Unity 수정
- 새 기능 구현
- transaction kernel 또는 trait 구현
- prototype를 production tree에 추가
- source audit 문서 수정
- broad Cargo test 재실행을 관성적으로 수행
- 두 감사의 모든 supporting document를 무차별 정독
- 근거가 다른 finding을 root cause가 같다는 이유만으로 삭제
- rejected finding을 새 증거 없이 active로 복원
- 후속 remediation Goal 자동 생성 또는 실행
- 사용자 승인 없는 commit/push

코드 확인이 필요한 경우 read-only inspection과 좁은 비변경 probe만 허용하며, 왜 audit evidence만으로 부족했는지 기록한다.

## 7. Canonical output contract

`CONSOLIDATED_AUDIT.md`는 최소 다음을 포함한다.

- executive conclusion
- source audit scope와 baseline 차이
- canonical finding catalog
- source-to-canonical crosswalk
- consolidated root-cause model
- 여섯 architecture axis 판정
- preservation asset register 요약
- subsystem disposition
- whole/partial/no-rewrite decision
- remediation dependency groups와 권장 순서
- unresolved gaps, policy decisions, confidence limits
- source evidence traceability
- 이후 Goal 설계를 위한 handoff

이 문서는 finding을 많이 나열하는 목록이 아니라, 왜 그것들이 같은 문제이거나 다른 문제인지 설명하는 의사결정 문서여야 한다.

## 8. Stop conditions

다음 경우에는 임의로 해소하지 말고 `BLOCKED` 또는 `UNKNOWN` 후보로 기록한 뒤 사용자에게 보고한다.

- 두 감사의 final verdict가 실제로 양립하지 않음
- 동일 symbol/behavior에 대해 상반된 confirmed evidence가 존재함
- 최신 정책 확인 없이는 보존/폐기 판단이 뒤집힐 수 있음
- 통합 severity가 remediation 우선순위를 실질적으로 바꾸지만 근거가 부족함
- source audit의 baseline contamination 또는 완료 상태가 신뢰할 수 없음

## 9. Completion

완료 시 다음 조건을 모두 만족한다.

- `FINDING_CROSSWALK.md` exact-set과 관계 판정 완료
- `ACCEPTANCE_GATES.md`의 모든 mandatory gate PASS
- `CONSOLIDATED_AUDIT.md` 작성 완료
- source audits와 production tree 미변경 확인
- 통합 문서가 특정 구현안을 승인된 사실처럼 서술하지 않음
- 후속 Goal은 후보로만 제시되고 자동 시작되지 않음

Goal 관리 기능을 사용했다면 위 조건을 실제로 만족한 뒤에만 complete로 닫는다.
