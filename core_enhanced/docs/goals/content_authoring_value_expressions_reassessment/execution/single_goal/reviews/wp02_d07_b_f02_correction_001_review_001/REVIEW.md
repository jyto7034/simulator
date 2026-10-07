# D07-B F02/C02 successor 단계검토

2026-09-28 · **PARTIAL_SCOPE_PASS — F02/C02 부분 범위 통과**.

legacy Damage/Heal 관측 배열의 할당 전 검사 누락은 해소되었다. 이번 범위에서 추가 교정을 요구하는 finding은 없다. F01의 선행 부분 PASS를 유지하며 F03은 미완료다. D07-B·WP02·R06 전체 완료 판정이 아니다.

## 대상과 검토 경계

- [구현 인계서](../../corrections/wp02_d07_b_f02_001/REVIEW_HANDOFF.ko.md)
- [동결 manifest](../../corrections/wp02_d07_b_f02_001/target/d07_b_f02_correction_001/MANIFEST.json)
- Manifest SHA-256: `2eb98e729fe0cc73f8cedebb455c73879a078c1d4989d5120f74b1e502dca0ba`.
- before: 341개, `74da8e17a0809b8a93f1d9c9dad97a994c7549952fc92a77985df6c33305cb3a`.
- after: 341개, `e431c18332d1336ef4f0c0f7ef062ed7569c49875acf3d8a18e21ca32a456730`.
- 직전 F01 대비 변경은 `sim.rs`, `value_expr_runtime.rs`, `runtime_control.rs` 3개다. 나머지 338개는 byte 동일하고 추가·삭제가 없다.

원본 구현 대화와 분리된 사본 대화에서 source delta·실제 호출 경로·동결 증거를 검토하고 집중 시험을 재실행했다. 설계와 교정 지시서를 공유한 단계검토이며 설계까지 독립된 감사라는 뜻은 아니다. 제품 소스와 기존 동결 자료는 수정하지 않았다.

## 코드 판단

`sim.rs:61`의 `allocate_evaluated_amounts`는 `count * size_of::<RuntimeAmount>()`를 checked 곱으로 계산한다. 기존 `value_expr_evaluated_amounts` site의 `admit_retained_growth`가 성공한 뒤에만 `Vec::with_capacity`를 호출한다. expression 경로(`sim.rs:2971`)와 관측 on·catalog 등록 legacy 경로(`sim.rs:3023`)가 이 함수를 공유한다. 관측 off와 catalog 없는 legacy에 새 배열이나 평가를 추가하지 않는다.

`runtime_control.rs`의 새 admission 관찰기는 모두 `cfg(test)`다. 기존 누적 admission 계산을 그대로 실행하고 성공한 site·직전/직후 누적값만 기록한다. production 한도·거부 정책을 바꾸거나 특정 시험 입력만 우회하지 않는다. thread-local 수집은 scope 종료 시 해제된다.

실제 legacy Damage/Heal의 다중 대상 처리는 기존 `runtime_transaction` 후보 상태에서 실행된다. 배열 검사나 이후 할당이 거부되면 후보를 commit하지 않는다. 신규 시험은 이미 성공한 record A가 있는 상태에서 gameplay snapshot·queue·runtime facts·동일 Arc·순번 보존과 A의 amount/trace 재평가를 확인한다. 배열 자체의 정확 한도와 전체 operation의 정확 한도를 별도로 시험하므로 배열 승인만으로 전체 처리가 성공했다고 오인하지 않는다.

## 검증 근거

| 구분 | 이번 검토의 확인 |
| --- | --- |
| 직접 재실행 | `cargo test -p game_core --lib d07_f02_c02 --offline --locked -- --nocapture`: 3 passed / 0 failed |
| 경계 시험 | 실제 helper의 정확 bytes·1 byte 부족·곱 overflow, legacy Damage/Heal 대상 2명, 기존 record 보존·재시도·순번, expression 공통 경로와 legacy 관측 on/off |
| 최종 실행 영수증 | 11개 명령의 source 전후 digest와 원본 log SHA 확인. 일반 명령 exit 0, R05 raw exit 101 보존 |
| 전체 시험 | 구현자가 최종 source에서 실행한 1,545 passed / 0 failed 원본 로그와 영수증 대조. 검토자가 전체 시험을 다시 실행한 것은 아님 |
| 집중·정적 검사 | 같은 source의 authoring 112개·value_expr 94개 및 check·Clippy·변경 파일 format 성공 근거 확인. Clippy 경고는 원문 보존 |
| PC | predecessor/current 각 576행을 대조. 측정·비용 필드를 제외한 gameplay·결과·오류 차이 0. 기존 36조건의 성공 31 / typed 거부 5 유지 |
| R05 | 원자료 모든 allocation sample의 operations·retained·peak와 repeatability/parity 확인. concurrent peak 9,049,037 bytes, cap 대비 18,849 bytes 초과만 기존 정책상 경고. 다른 hard guard 실패 없음 |
| 동결 | source 341개 currentness·membership, 기록 327개, 격리 before→after→before byte 복원 확인 |

재실행 로그는 [FOCUSED_RECHECK.log](FOCUSED_RECHECK.log), 영수증·원자료 대조는 [SCOPE_AND_RECEIPTS_CHECK.json](SCOPE_AND_RECEIPTS_CHECK.json)에 남겼다. 전체 회귀와 benchmark는 동결 source·원자료 결속을 확인하여 검토 근거로 사용했고 재실행하지 않았다. PC의 기존 관측 off 비용 회귀를 새 관측 on의 포괄적 성능 보장으로 확대하지 않는다. R05 latency gate는 원자료상 적용 대상이 아니며 peak 경고를 숨기거나 cap을 바꾸지 않았다.

## 잔여 작업과 재개

F02/C02만 닫는다. record A 보존 시험은 이번 메모리 거부 경계의 증거이며, public selector/query·terminal 수명·동적 Snapshot/Live·취소·큰 정수 출력까지 통과한 근거가 아니다.

다음은 사용자 재개 요청에 따라 원본 대화에서 [기존 교정 지시서](../../handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md)의 **F03을 다음 의미 있는 구간으로 나누어 구현·검증**하는 것이다. 실제 입력을 변경하는 Snapshot/Live 소비와 기록 조회 일치부터 근거를 보강하고, C03–C15의 미완료 항목을 계속 표시한다. 해당 구간이 닫히면 successor target을 동결하고 사본 단계검토를 요청한다. 이번 PASS가 F03 전체 구현이나 R06 전체 완료를 자동 승인하는 것은 아니다.

검토 시작/종료의 동결 확인은 [ENTRY_CURRENTNESS_AND_RESTORATION.json](ENTRY_CURRENTNESS_AND_RESTORATION.json)과 [EXIT_CURRENTNESS_AND_RESTORATION.json](EXIT_CURRENTNESS_AND_RESTORATION.json), 문서·근거 결속은 [REVIEW_VALIDATION.json](REVIEW_VALIDATION.json)에 기록한다.
