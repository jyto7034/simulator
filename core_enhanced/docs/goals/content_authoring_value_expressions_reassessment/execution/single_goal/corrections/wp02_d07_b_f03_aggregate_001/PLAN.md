# F03 C06 집계 관측 — 현재 위치

2026-09-28 · C06 부분 자체 검증 종료·단계검토 요청 대기. 템플릿 2026-09-27-v1.

- [교정 지시서](../../handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md) C06/B05 및 이번 C16.
- [C03 runtime 부분 PASS](../../reviews/wp02_d07_b_f03_legacy_001_review_001/REVIEW.md), [before](baseline/entry_001/MANIFEST.json). predecessor verify 후 실제 before 보존.
- Side/Employee/장비2슬롯 네 aggregate node를 실제 release carrier→ordered executor→record→두 encoder로 확인한다. ByType/Any3, Snapshot/Live, 명시적 필터/빈 집계와 부적격 owner 거부를 포함한다.
- 기존 owner_fixture와 실제 command tap을 재사용하며 새 cfg(test) module을 추가한다. frame/record를 조립하거나 공개 dynamic 차단을 풀지 않는다.
- 신규 집중 시험, authoring/value_expr 및 기본 병렬 전체 회귀, check/Clippy/변경 format. 시험만 바뀌면 PC36/R05 선행 근거의 적용성을 확인해 재사용한다.
- 현재 위치는 PLAN, 시도/실패는 EXPERIMENTS, 판단은 EXPERIMENT_NOTES. 검증 결과와 target은 최종 판정표/manifest에 연결한다.
- C06의 검토 가능한 동작과 필수 회귀가 닫히면 동결·사본 단계검토에서 중단. C07–C15와 C03-OBS01(RON MIN)은 후속, F03/D07-B 전체 미완료.

## 동시 입력 기록

작업 중 C06과 별도인 tests/battle_showcase.rs가 추가됐다. [WORKSPACE_ADDITION](evidence/WORKSPACE_ADDITION.json)으로 실제 실행 입력을 구분하고 최종345개 source를 결속한다. 최초344개만 묶은 실행은 보존하되 최종 검증 근거로 사용하지 않는다. 파일 내용은 수정하지 않는다.

## 현재 target·다음 행동

- [최종 판정표](evidence/FINAL_VALIDATION.json), [집계 원자료 판정](evidence/AGGREGATE_MATRIX.json), [측정 재사용](evidence/EVIDENCE_REUSE.json).
- [검토 인계](REVIEW_HANDOFF.ko.md), [successor manifest](target/d07_b_f03_aggregate_001/MANIFEST.json), [격리 복원](target/d07_b_f03_aggregate_001/RESTORATION_VERIFICATION.json).
- 다음은 사본 대화의 C06 독립 단계검토. 구현자는 PASS를 선언하지 않는다.
- C07–C15 및 C03-OBS01(RON MIN)은 후속. F03/D07-B 전체 미완료이며 다음 구현은 재개 요청 뒤 진행한다.
