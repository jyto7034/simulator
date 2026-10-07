# D07-B F03 C04/C05 successor 단계검토

2026-09-28 · **PARTIAL_SCOPE_PASS — C04/C05 부분 범위 통과**.

실제 입력이 바뀌었을 때 Snapshot은 release 입력을, Live는 소비 시점 입력을 사용하며, 기록의 preview/explain 재평가는 이후 상태 변화에 영향을 받지 않는다는 근거를 확인했다. 이번 범위에서 추가 교정을 요구하는 finding은 없다. F03과 D07-B 전체는 미완료다.

## 대상과 변경 범위

- [구현 인계서](../../corrections/wp02_d07_b_f03_replay_001/REVIEW_HANDOFF.ko.md), [동결 manifest](../../corrections/wp02_d07_b_f03_replay_001/target/d07_b_f03_replay_001/MANIFEST.json).
- Manifest SHA-256: `31421ec1cea97fd573aa21ba3faa305ae5c606c0adf15da7851ac553a54cb4ba`.
- before 341개 digest: `e431c18332d1336ef4f0c0f7ef062ed7569c49875acf3d8a18e21ca32a456730`.
- after 342개 digest: `f5cbc267ea1e86995e6c85433c8d20fdb81cd59bd64eeddac838cddff985183d`.
- 변경: `value_expr_runtime.rs`의 cfg(test) 내부 helper와 하위 module 연결. 추가: `value_expr_runtime/tests/amount_replay.rs`. 나머지 선행 340개 파일은 byte 동일하다.

원본 구현 대화와 분리된 사본에서 검토했다. 설계·지시서를 공유하므로 설계까지 독립된 감사라는 뜻은 아니다. 검토자는 제품 소스와 동결 자료를 수정하지 않았다.

## 코드와 시험 판단

새 fixture는 실제 catalog의 authored plan으로 action을 만들고 수치 관측과 ObservationKey를 연결한다. release helper는 finite source를 예약하고 실제 `build_value_expr_delivery_capture`가 만든 carrier를 template에 보관한다. 입력을 변경한 다음 기존 `runtime_transaction` 안의 `execute_ordered_skill_effects`로 Damage/Heal을 소비한다. 시험이 기대값을 record/frame에 직접 넣지 않는다.

소비 helper의 `runtime_fact_source` 교체·복원은 실제 `skill_runtime/projectile.rs::advance_skill_projectile`의 source scope 계약과 일치한다. Snapshot 기록은 release frame과 동일 Arc인지 확인하고, Live capture에는 snapshot frame이 없음을 확인한다. 이는 release/consume 사이의 내부 실행 경계 시험이며 실제 투사체 이동·충돌·queue 전체를 새로 통과했다는 의미는 아니다.

기존 resource helper 변경은 `builder.build()` 후 catalog를 교체하던 시험 준비를 `with_resources` 후 build로 고친 것이다. compiler가 자원 정의를 실제로 검증할 수 있게 하며, production validation이나 공개 차단을 우회하지 않는다.

| 입력 | 독립 기대값과 확인 |
| --- | --- |
| Source.Attack | release 10 → consume 전 20 → 이후 30. 실제 피해·기록·문서는 Snapshot 10 / Live 20 |
| Target.HP | release 80 → consume 전 60 → 이후 30. HP/4 피해는 Snapshot 20 / Live 15, frame 입력은 80 / 60 |
| Resource | 초기 7 → 실제 Grant 4 → 이후 Grant 5. 피해는 Snapshot 7 / Live 11. 단위 변환도 실제 compiler/evaluator 사용 |
| 완료 action result | 실제 피해 7 완료 → release → 피해 3 완료 → Heal. Snapshot 7 / Live 10. 이후 완료 피해 합계 13에서도 보존된 기록의 문서 동일 |
| Live 앞 effect | Physical 7의 실제 피해 4 및 Grant 4 후 Heal. MissingHP 24 / EarlierEffect 실제 피해 4 / resource 11, 최종 HP 80 / 60 / 67 및 순번 2 확인 |

공통 helper는 독립 기대 amount·frame 입력·trace 내 입력과 실제 HP/완료 fact를 검사한다. 실제 생성된 record를 두 인코더로 재평가하여 value·trace·시각·순번을 비교한다. 반복 출력 bytes와 usage, gameplay/queue/facts/record Arc/순번 불변도 확인한다. 완료 result의 나중 변화는 관측만 잠시 끄고 실제 damage/settlement를 추가하는 private 시험 장치다. 관측 전환 API나 기록 교체 정책의 검증으로 확대하지 않는다.

`encode_record_for_test`는 production `encode_record`의 검증·공통 evaluator·문서 생성을 사용하지만 public selector를 건너뛴다. [교정 지시서](../../handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md)가 허용한 staged dynamic 수치 대조 범위에 해당한다. Exact를 문서에 넣었다는 사실만으로 Latest/Exact 선택, 현재 content generation 대조, 권한, terminal 수명을 검증했다고 표시하지 않는다.

## 검증과 증거 재사용

- 직접 집중 시험: 기본 병렬 5 passed / 0 failed. 진단 출력 일부가 섞여 같은 5개만 단일 스레드로 재실행했고 역시 통과했다. 11개 시나리오의 변경 전후 22쌍 preview/explain 문서를 파싱하여 amount·trace·release/consume 시각과 기대값을 대조했다.
- 구현자 최종 실행 7건: source 전후 digest와 log SHA, 실제 test summary를 대조했다. 전체 1,550개, authoring 112개, value_expr 99개 통과 및 check·Clippy·변경 파일 format exit 0. 검토자가 전체 회귀를 재실행한 것은 아니다. Clippy 경고 원문은 보존한다.
- currentness: 342개 source membership, 동결 기록 375개, 격리 341→342 및 342→341 byte 복원 확인. 선행 F02 source의 모든 before row가 일치한다.
- PC36/R05: 제품 실행 코드·빌드 입력·도구 보존을 확인했다. 변경 parent의 cfg(test) 이전 byte도 동일하고 새 module은 test에서만 연결된다. 재사용 근거 14개 파일의 hash를 확인했으며, 선행 단계에서 검토한 측정을 재사용할 수 있다. 새 측정이나 관측 on 비용 보장으로 주장하지 않는다.
- PC36 성공 31 / typed 거부 5 유지. R05 raw exit 101과 concurrent peak 9,049,037 bytes, cap 초과 18,849 bytes 경고를 그대로 유지한다. 이번 시험 보강으로 이 경고가 해소된 것은 아니다.

근거: [기본 병렬 재실행](FOCUSED_RECHECK.log), [문서 대조용 단일 스레드 재실행](FOCUSED_SERIAL_RECHECK.log), [source·영수증·출력 대조](SCOPE_AND_RECEIPTS_CHECK.json), [시작 확인](ENTRY_CURRENTNESS_AND_RESTORATION.json), [종료 확인](EXIT_CURRENTNESS_AND_RESTORATION.json), [검토 결속](REVIEW_VALIDATION.json).

## 잔여와 다음 구간

F01·F02 선행 부분 PASS를 유지하고 이번 C04/C05를 닫는다. **C03·C06–C15, F03·D07-B·WP02·R06 전체는 미완료**다. C16 확인은 이번 checkpoint에만 적용한다.

다음은 사용자 재개 요청 후 원본 구현 대화에서 기존 지시서의 잔여를 의미 있는 구간으로 진행한다. 우선 후보는 C03의 legacy 경계값·관측 on/off 대조이며, 이후 C06 집계와 C07/C08의 실제 선택·identity 검증으로 연결한다. 공개 selector/terminal/한도/취소/큰 정수 출력 의무를 이번 내부 인코더 시험으로 대체하지 않는다. 다음 구간도 자체 검증·target 동결 뒤 사본 단계검토에서 멈춘다.
