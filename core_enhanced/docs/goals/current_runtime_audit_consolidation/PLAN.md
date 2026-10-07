# Plan — Current Runtime Audit Consolidation

- 상태: `completed`
- 작성일: 2026-08-22
- 실행 시작: `2026-08-22T16:27:54+09:00`
- 실행 종료: `2026-08-22T16:42:40+09:00`
- production change: 금지
- primary output: `CONSOLIDATED_AUDIT.md`

## 1. Goal

`current_runtime_structural_integrity_audit`와 `current_runtime_architecture_deep_review`를 하나의 일관된 감사 결론으로 통합한다.

이 Goal은 수정 계획을 구현하지 않는다. 이후 수정 Goal들이 동일한 문제를 중복 해결하거나, 중요한 finding을 놓치거나, 전면 재작성 판단을 다시 처음부터 반복하지 않도록 canonical decision base를 만드는 작업이다.

## 2. Questions to answer

1. 두 감사의 finding 중 같은 defect, 같은 root cause, 독립 문제는 각각 무엇인가?
2. severity와 confidence가 다를 때 어떤 근거 차이 때문인가?
3. 구조 결함과 architecture pressure의 인과 관계는 무엇인가?
4. 반드시 보존할 runtime 자산은 무엇인가?
5. refactor, boundary rewrite, subsystem rewrite가 필요한 범위는 어디인가?
6. whole-core big-bang rewrite를 정당화하는 증거가 있는가?
7. remediation을 어떤 독립 단위와 종속성으로 나누어야 하는가?
8. 아직 정책 결정이나 추가 검증이 필요한 것은 무엇인가?

## 3. Scope

### In scope

- source audit scope/baseline/criterion 비교
- active/rejected/gap/root cause 전수 crosswalk
- 중복과 인과관계 판정
- severity/confidence 정규화
- 여섯 architecture axis 통합 평가
- preservation/disposition/rewrite 판단 통합
- remediation dependency group과 권장 sequencing
- 하나의 canonical audit 작성

### Out of scope

- production remediation
- 새 architecture prototype
- code, test, data, server, Unity 변경
- 새로운 gameplay policy 결정
- game rulebook 전체 alignment 재감사
- stale contract 최신화
- 새로운 performance benchmark
- 후속 Goal 문서 생성 또는 실행
- 미리 작성된 transaction remediation 초안을 결과로 채택

## 4. Evidence model

### Tier 1 — Final audit outputs

최종 conclusion, finding, rewrite decision, preservation register를 직접 사용한다. source finding의 상태·severity·confidence는 각 audit의 final catalog를 우선한다.

### Tier 2 — Supporting audit evidence

중복/충돌/인과관계를 판정할 때만 evidence matrix와 관련 map으로 내려간다. source audit가 이미 확정한 사실을 이유 없이 재감사하지 않는다.

### Tier 3 — Targeted current-tree check

두 감사만으로 해결되지 않는 모순에 한해 현재 code/policy의 필요한 symbol 또는 구간만 확인한다. 이 확인은 finding을 새로 발굴하기 위한 전면 감사가 아니다.

## 5. Canonicalization rules

1. 모든 source finding은 canonical finding 하나 이상에 연결되거나 `rejected history`에 남아야 한다.
2. source IDs는 삭제하거나 재번호화하지 않는다.
3. 같은 root cause는 같은 defect를 뜻하지 않는다.
4. trigger, affected authority, failure mode, user/operation impact가 다르면 별도 canonical finding을 유지할 수 있다.
5. architecture finding이 structural finding의 상위 원인이라면 `cause/effect`로 연결하고 무리하게 병합하지 않는다.
6. severity는 발생 가능성만이 아니라 영향, reachability, 증거 확신도, migration risk를 분리해서 설명한다.
7. rejected finding은 historical record로 남기되 remediation backlog에 넣지 않는다.
8. gap은 finding이 아니며, finding을 증명하거나 remediation을 검증하기 위해 부족한 evidence로 분리한다.
9. recommendation은 `evidence-backed`, `conditional`, `policy-required` 중 하나로 표시한다.

## 6. Execution phases

### Phase 0 — Activate and register sources

작업:

- Goal 상태를 `active`로 변경
- source audit 두 개의 완료 상태와 baseline ID 기록
- source audit의 scope/exclusion/baseline 차이 기록
- 현재 source 문서가 감사 완료 이후 수정되었는지 최소 확인
- `SOURCE_LOG.md` 생성

문서 byte size와 hash를 관성적으로 전수 수집하지 않는다. 감사 완료 이후 source 변조 여부가 실제 쟁점일 때만 필요한 파일에 한정한다.

Exit:

- source authority와 제외 범위가 명확함
- remediation 초안이 source에서 배제됨

### Phase 1 — Extract final catalogs

작업:

- structural active 8, rejected 1의 source facts 확인
- architecture active 6의 source facts 확인
- 각 audit의 root causes, gaps, preservation/rewrite conclusions 추출
- source별 severity/confidence/criterion/scope 기록

Exit:

- source catalog count와 crosswalk count exact match
- silent omission 없음

### Phase 2 — Build finding crosswalk

작업:

- 각 source finding의 counterpart 후보 검토
- 관계 유형 판정
- supporting evidence가 필요한 관계만 Tier 2로 확인
- 충돌, 부분 중복, no-counterpart를 명시
- canonical finding 후보 ID 부여

Exit:

- 모든 active/rejected source row에 관계와 canonical disposition 존재
- 단순 제목 유사성으로 병합한 row 없음

### Phase 3 — Consolidate findings and root causes

작업:

- canonical finding catalog 작성
- severity/confidence 정규화 근거 작성
- source RC-01~03과 RC-A~C의 관계 모델 작성
- symptom, defect, architectural pressure, root cause 구분
- gap을 별도 verification debt로 정리

Exit:

- canonical finding마다 source ID, evidence, trigger, impact, owner, disposition 연결
- root cause마다 연결 finding과 반대 증거 존재

### Phase 4 — Consolidate architecture decision

작업:

- 도메인 구조
- 의존 방향
- 확장성
- 변경 용이성
- 테스트 가능성
- 운영 경계

각 축을 `PASS`, `PARTIAL`, `FAIL`, `UNKNOWN` 중 하나로 판정하고 source evidence 및 제한을 기록한다.

그 뒤 subsystem별로 다음 disposition을 부여한다.

- `RETAIN`
- `REFACTOR`
- `REWRITE_BOUNDARY`
- `REWRITE_SUBSYSTEM`
- `DEFER_POLICY`
- `REJECTED_REMEDIATION`

마지막으로 whole-core rewrite test를 다시 적용한다. 기존 deep review 결론을 그대로 복사하지 말고 structural finding이 그 결론을 뒤집는지 명시적으로 확인한다.

Exit:

- 여섯 축 누락 없음
- 모든 주요 subsystem에 disposition 존재
- whole/partial/no-rewrite 결론과 반대 증거 존재

### Phase 5 — Build remediation dependency map

작업:

- canonical findings를 함께 해결해야 하는 독립 remediation unit으로 묶음
- unit 간 선행 조건, preservation oracle, migration seam, 위험 기록
- 사용자 가치/심각도/종속성/가역성 기준으로 권장 순서 작성
- 각 unit이 별도 Goal이 되어야 하는지 제안

제한:

- 구체 trait/API 이름을 확정하지 않음
- production file allowlist나 commit 계획을 미리 만들지 않음
- transaction 작업을 자동으로 첫 Goal로 결정하지 않음

Exit:

- 각 active canonical finding이 remediation unit 또는 명시적 defer에 연결됨
- circular dependency나 hidden prerequisite가 설명됨

### Phase 6 — Write canonical audit

`CONSOLIDATED_AUDIT.md`를 다음 순서로 작성한다.

1. Executive conclusion
2. Source scope and trust boundary
3. Consolidated finding catalog
4. Finding relationship/crosswalk summary
5. Root-cause model
6. Six-axis architecture assessment
7. Preservation assets and subsystem disposition
8. Rewrite decision
9. Remediation dependency map and recommended sequence
10. Gaps and policy decisions
11. Evidence traceability
12. Handoff

Exit:

- 문서 하나만 읽어도 현재 결함, 설계 판단, 보존 범위, 다음 의사결정이 이해됨
- 세부 증거는 source audit로 추적 가능함

### Phase 7 — Integrity audit and close

작업:

- `ACCEPTANCE_GATES.md` 전수 판정
- crosswalk exact-set 재확인
- source audit와 production tree 미변경 확인
- 과도한 certainty, 누락, 중복 count 검토
- `COMPLETION_AUDIT.md` 작성
- PLAN 상태를 `completed`로 전환

Exit:

- mandatory gate 전부 PASS
- 후속 Goal 자동 생성 없음

## 7. Required artifacts

준비 시점:

- `GOAL_PROMPT.md`
- `PLAN.md`
- `FINDING_CROSSWALK.md`
- `ACCEPTANCE_GATES.md`

실행 중/완료 시:

- `SOURCE_LOG.md`
- `CONSOLIDATED_AUDIT.md`
- `COMPLETION_AUDIT.md`

추가 문서는 위 세 산출물로 표현할 수 없을 때만 만든다.

## 8. Verification policy

- 이 Goal은 문서 통합이므로 Cargo test/fmt/clippy/performance 실행을 완료 gate로 요구하지 않는다.
- source count와 crosswalk count는 가능한 한 기계적으로 대조한다.
- source path/ID/상태/severity/confidence 인용은 원문과 대조한다.
- production current-tree check는 conflict resolution에 필요한 경우만 수행한다.
- source audit 또는 production 변경이 감지되면 contamination으로 기록하고 원인을 해소하기 전 완료하지 않는다.

## 9. Completion prohibitions

다음 상태에서는 완료할 수 없다.

- source finding이 crosswalk에서 누락됨
- 같은 root cause라는 이유만으로 distinct failure mode가 사라짐
- severity/confidence 조정 근거가 없음
- 여섯 architecture axis 중 하나가 빠짐
- 보존 자산 없이 rewrite 범위만 나열함
- whole-core rewrite 판단에 반대 증거가 없음
- remediation 순서가 종속성보다 취향에 의존함
- transaction remediation 초안을 감사 결과로 역수입함
- production 또는 source audit를 수정함
- primary output이 여러 상충 문서로 분산됨
