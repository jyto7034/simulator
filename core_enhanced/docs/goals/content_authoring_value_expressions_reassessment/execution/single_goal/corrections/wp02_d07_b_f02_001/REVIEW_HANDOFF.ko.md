# D07-B F02/C02 — legacy amount 배열 사전 admission 교정

2026-09-28 · **F02 부분 교정·자체 검증 종료 / successor 단계검토 인계**. 동결 identity와 검증은 아래 manifest 기준.

## 범위와 구현

[교정 지시서](../../handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md)의 F02/C02와 해당 C16을 수행한다.
[F01 사본 검토](../../reviews/wp02_d07_b_f01_correction_001_review_001/REVIEW.md)의 부분 PASS 후 사용자 재개 요청을 받았다.
F03의 동적 관측·조회·수명·취소 등 C03–C15 전체를 완료한 결과가 아니다.

`sim.rs::allocate_evaluated_amounts(count)`에서 RuntimeAmount 배열 bytes를 checked_mul로 계산하고,
기존 native admit_retained_growth가 성공한 뒤 Vec::with_capacity를 실행한다.
기존 expression 경로와 관측 on + catalog 등록된 legacy Damage/Heal 경로가 같은 함수를 사용한다.
기존 site `value_expr_evaluated_amounts`를 유지한다. 관측 off와 catalog 없는 legacy에는 새 배열·평가를 만들지 않는다.

`runtime_control.rs`의 추가 계측은 전부 cfg(test)다. 성공한 admission의 site·직전 누적값·직후 누적값을
thread-local로 수집하고 scope 종료 시 해제한다. production 정책/한도/rollback/평가 순서를 바꾸지 않는다.
`value_expr_runtime.rs`에는 기존 실제 fixture와 transaction을 사용한 C02 시험 3개를 추가했다.

## C02 시험과 원자료

모든 시험 이름 접두사는 `game::battle::core::value_expr_runtime::tests::d07_f02_c02_`다.

| 시험 suffix | 입력과 실제 확인 |
| --- | --- |
| amount_buffer_admits_exact_bytes_and_rejects_short_or_overflow | 실제 공통 함수에 3개 RuntimeAmount. 정확 bytes에서 capacity 확보, 1 byte 부족 및 checked 곱 overflow는 allocation 이전 RetainedByteLimit와 기존 site로 거부 |
| legacy_multitarget_admission_preserves_accepted_record_on_refusal | Damage7 및 Heal3, 대상2명, 이미 성공한 record A 존재. 성공 operation의 누적 요구량 측정 → 배열 site 기준 1 byte 부족한 새 operation → accepted gameplay/queue/runtime facts/동일 Arc A/ordinal/재평가 결과 보존. 배열 정확 한도까지는 통과하되 후속 할당은 별도 거부. 전체 성공 요구량의 정확 한도로 새 operation 재시도 성공 및 ordinal+2 |
| buffer_shared_with_expression_and_skipped_for_unobserved_or_uncatalogued_legacy | catalog legacy on/off의 동일 피해와 transaction snapshot/queue/runtime facts 대조, source 없는 programmatic legacy 정상 소비·새 배열 없음, expression actual executor의 동일 helper site 사용 |

실제 target2 배열 bytes는 16이다. 이는 전투 전체 retained 크기가 아니다.
Damage 배열 admission 누적24531 / 전체 operation46785 bytes, Heal 배열 누적19563 / 전체36811 bytes를 관찰했다.
기존 retained ledger와 operation 누적 admission을 혼동하지 않았고, 이전 작업 allowance를 재사용하지 않았다.

- 연결 전 회귀 `f02_before_001`: helper와 expression refactor/계측/시험만 추가한 상태에서 1 pass / 2 fail. legacy 배열의 admission 부재 검출.
- 연결 후 첫 시험 `f02_after_001`: 2 pass / 1 fail. 배열이 첫 admission일 수 있는데 이전 trace index를 가정한 **시험 코드** 오류.
- 관찰값에 직전 누적값을 추가한 `f02_after_002`: 3/3 통과.
- on/off 상태 대조를 보강한 최종 `f02_final_001`: 3/3 통과, 최종 source 기준.
- [수행 이력](EXPERIMENTS.md), [현재 판단](EXPERIMENT_NOTES.md), `evidence/BEFORE_REGRESSION_SOURCE.tar.gz`에 실패/교정 전 자료 보존.

C02 record 보존은 동일 immutable Arc와 실제 재평가 amount/trace로 확인했다. public selector/query 경계나 terminal 수명까지
새로 검증한 것으로 확대하지 않는다. 그 영역은 F03에 남는다.

## 검증과 비용

실행 영수증·명령·환경·source 전후 SHA·로그는 evidence/에 있다.
[최종 판정표](evidence/FINAL_VALIDATION.json)는 새 실행 결과를 연결한다.
이번에는 native 경로가 바뀌므로 선행 전체 회귀/PC36/R05만 재사용하지 않고 최종 source에서 재실행했다.

| 최종 검증 | 결과 |
| --- | --- |
| F02/C02 | 3 passed / 0 failed |
| authoring / value_expr | 112 / 94 passed, 모두 실패0 |
| 전체 target 기본 병렬 | 1545 passed / 0 failed, 약137.5초 |
| check / Clippy / 변경 파일 format | exit0. Clippy 경고 원문 보존(새 helper의 기존 GameError 크기 경고 포함) |
| PC36 | 성공31 / typed 거부5. gameplay·상태·오류 차이0 |
| R05 | raw exit101, concurrent peak9,049,037 bytes, 원 cap 대비18,849 bytes 초과 단일 경고. 다른 hard failure 없음 |

[PC36 비교](evidence/PC36_COMPARISON.json)는 보존 probe를 최종 source 격리 복사본에만 붙여 측정한 결과다.
probe와 두 hook의 수정 파일 원본·복사 전후 source identity는 evidence/pc_isolate_final_001.json 및 hook archive에 보존한다.
제품 작업 트리에는 PC hook을 넣지 않았다. PC는 기존 관측 off 비용/동작 회귀이며 새 on 검증은 위 C02 시험과 구분한다.

[R05 원자료 판정](evidence/R05_SUMMARY.json)은 원 tool/input/baseline 및 모든 sample의 operations·retained·peak·parity를 확인한다.
concurrent peak 예외만 [기존 정책](../../handoffs/R06_COST_POLICY_UPDATE_20260927.ko.md)을 적용하며 raw exit/실패를 보존한다.
20분 시험, production expression 공개, 스킬 자동 취소, 새 상한은 추가하지 않았다.

## 동결과 후속

- [직전 F01 target](../wp02_d07_b_review_001/target/d07_b_f01_correction_001/MANIFEST.json): SHA bf51ac15713acc3afa6a850452072e917be94df34039a226cadc03624b97da8d.
- before341 digest: 74da8e17a0809b8a93f1d9c9dad97a994c7549952fc92a77985df6c33305cb3a.
- after341 digest: e431c18332d1336ef4f0c0f7ef062ed7569c49875acf3d8a18e21ca32a456730.
- 직전 대비 변경3개: sim.rs, value_expr_runtime.rs, runtime_control.rs. 나머지338개 byte 동일, 파일 추가/삭제 없음.
- 최초 D07-B 대비 누적 변경4개(위3개+F01 CLI), 신규 CLI integration test1개. F01 교정은 보존한다.
- [successor manifest](target/d07_b_f02_correction_001/MANIFEST.json), [격리 복원](target/d07_b_f02_correction_001/RESTORATION_VERIFICATION.json).
- before/after 전체 archive, 정·역 patch, 실행·실패·검토·인계 원자료 archive를 결속한다. `python3 freeze.py verify`는 읽기 전용 재확인이다.
- 현재 위치와 다음 행동은 [PLAN](PLAN.md)에 연결한다. 다음은 사본 대화에서 이 successor의 F02/C02 단계검토다.

구현자는 F02 검토 PASS나 D07-B/WP02/R06 전체 완료를 선언하지 않는다. F03은 후속 재개 요청으로 진행한다.
