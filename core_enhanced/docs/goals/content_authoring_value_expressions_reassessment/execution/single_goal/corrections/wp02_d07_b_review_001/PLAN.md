# D07-B F01 CLI 교정 — 현재 계획

2026-09-28 · 상태: F01 자체 검증 완료 / 단계검토 인계.

요청: [교정 지시서](../../handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md) §7의 첫 checkpoint.
이 원본 대화에서 구현·자체 검증·동결하고 사본 대화에서 읽기 전용 검토한다.
이번 범위는 F01/C01, B18 중 CLI 인자 검증, C16의 해당 부분뿐이다.
F02 및 F03의 동적 입력·수명·계량·큰 정수 검증은 미완료로 남긴다.

## 기준선과 설계

- before: [baseline](baseline/entry_001/MANIFEST.json), 340개 및 새 integration test 부재.
- [predecessor 확인](evidence/ENTRY_PREDECESSOR_VERIFY.json): 현재 소스·정역 복원 일치.
- 제품 변경: author_content.rs에서 operation별 필수 positional 개수를 옵션 순회 전에 확인.
- 회귀 시험: 실제 Cargo 바이너리를 실행하는 author_content_cli_arguments integration test.
- 새 parser abstraction이나 production 의존성은 추가하지 않는다.
- 정상 CLI는 기존 동결 외부 pack/scenario로 amount/trace 단독·조합·순서와 출력 동등성을 확인한다.

## 기록과 검증

이 PLAN이 현재 위치를 소유한다. [EXPERIMENTS](EXPERIMENTS.md)는 시도/명령/원자료,
[EXPERIMENT_NOTES](EXPERIMENT_NOTES.md)는 현재 판단·미검증·후속을 소유한다.
기존 실패 이력·검토 finding을 읽었으며 source/target/실패 원본은 덮어쓰지 않는다.

순서: 실제 CLI regression을 추가해 before 실패 확인 → 최소 수정 → 새 시험/정상 CLI/format/check/clippy
→ 라이브러리 증거 적용성 대조 → 인계·successor 동결·중단.
빌드와 C01이 통과하고 미완료 범위를 분리할 수 있으면 첫 의미 있는 지점으로 선택한다.
전체 라이브러리·PC36/R05는 library/data/dependency가 동일하면 근거를 재사용한다.
구현자가 검토 PASS나 D07-B 전체 완료를 선언하지 않는다.

현재 위치: F01 minimum argc 교정 후 4개 integration 시험(30조건), 정상 CLI 8명령, check/format/clippy 통과.
현재 근거: [최종 검증](evidence/FINAL_VALIDATION.json), [재사용 판정](evidence/EVIDENCE_REUSE.json),
[인계서](REVIEW_HANDOFF.ko.md), [successor target](target/d07_b_f01_correction_001/MANIFEST.json).
동결·복원 결과는 target/RESTORATION_VERIFICATION.json과 freeze.py verify의 실제 결과를 기준으로 한다.
다음 작업: 사본 대화에서 F01 successor 읽기 전용 단계검토. 검토 뒤 F02/F03 재개 요청을 별도로 받는다.
선택 checkpoint: F01/C01의 CLI 입력 거부와 정상 실행 경로. F02/F03은 열린 상태로 인계한다.
