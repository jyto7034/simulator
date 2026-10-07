# D07-B F02/C02 실행 계획

2026-09-28 · F02 부분 교정·자체 검증 종료 / successor 단계검토 인계. 사용자가 F01 사본 검토 PASS 후 F02/C02 재개를 요청했다.

- 지침: [교정 지시서](../../handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md) §4 F02, C02 및 해당 C16.
- 시작점: [F01 target](../wp02_d07_b_review_001/target/d07_b_f01_correction_001/MANIFEST.json), SHA bf51ac15713acc3afa6a850452072e917be94df34039a226cadc03624b97da8d.
- before 341개 digest 74da8e17a0809b8a93f1d9c9dad97a994c7549952fc92a77985df6c33305cb3a.
- 현재 기록: 이 디렉터리 PLAN / EXPERIMENTS / EXPERIMENT_NOTES. F01 동결 기록은 수정하지 않는다.
- 설계: actual expression/legacy Vec 생성용 작은 공통 함수에서 checked_mul과 native admission을 먼저 수행한다.
- 시험: helper 정확/1 byte 부족/overflow, 실제 legacy 다중 대상 transaction과 기존 record/ordinal 보존, 관측 off 및 catalog 없는 경로, expression 공통 연결.
- 실제 operation 누적 admission은 test-only native 관찰로 측정한다. 낮은 cap에서 실패 site가 배열 지점인지 반드시 확인한다.
- 검증: 집중 시험, 전체 기본 병렬 회귀, check/clippy/변경 파일 format, PC36/R05 재측정. 실패와 재시도 원자료 보존.
- F02 동작과 검증이 닫히면 successor 동결과 사본 검토 인계. F03/C03–C15는 별도 후속으로 남긴다.

## 현재 위치와 인계

최종 source341개 digest e431c18332d1336ef4f0c0f7ef062ed7569c49875acf3d8a18e21ca32a456730.
F02 3/3, authoring112/112, value_expr94/94, 전체1545/1545, check/clippy/format 통과.
PC36 기존 결과 동일. R05 raw exit101은 concurrent peak18,849 bytes 초과 단일 경고다.
[최종 판정표](evidence/FINAL_VALIDATION.json), [인계서](REVIEW_HANDOFF.ko.md),
[successor target](target/d07_b_f02_correction_001/MANIFEST.json)에 실제 명령·원자료·범위를 연결한다.
직전 F01 대비 변경3개, 파일 추가/삭제 없음. 최초 D07-B 대비 누적 변경도 판정표에 기록했다.
다음 행동: 사본 대화의 읽기 전용 F02/C02 검토. F03/C03–C15와 D07-B/WP02/R06 전체 완료는 미완료다.
이 구현 대화에서는 다음 큰 교정을 자동으로 시작하지 않는다.
