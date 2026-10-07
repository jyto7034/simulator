# R06 WP02 CAP-01·CAP-02 포맷 교정 단계검토 인계

상태: **포맷 교정·자체 검증 종료 / 별도 단계검토 타깃 동결 / 재검토 요청 대기**. 이 문서는 재검토 PASS 또는 WP02·Phase·Goal 완료를 선언하지 않는다.

## 기준과 범위

- 직접 선행 타깃은 [CAP 교정 001](target/cap_01_02_correction_001/MANIFEST.json)이며 manifest SHA-256은 `179bff1593608283c82ed51a2923f183d47312b8df51ad3dcb7556fdca8a4a5f`이다. 동결 archive와 manifest는 변경하지 않았다.
- 포맷 입력은 선행 타깃의 source 333개와 현재 파일을 대조해 일치시킨 상태에서 시작했다.
- 변경은 선행 타깃에서 CAP-01·CAP-02와 action–cast 제품 회귀 시험이 추가한 다섯 파일의 해당 함수·시험 블록에만 적용했다. 실행 분기, 상수, 식, assertion, 시험 입력은 바꾸지 않았다.
- 전체 파일 rustfmt를 적용하지 않았다. 기존 사용자 포맷과 선행 코드의 범위 밖 차이를 보존했다.

## 포맷 판정

[범위 검증](verification/format_002_scope.json)은 선행 타깃 333개를 전수 대조하고, 달라진 다섯 파일의 delta가 지정한 CAP·회귀시험 블록 안에만 존재하는지 확인한다. 임시 사본의 rustfmt 결과를 그 블록에만 반영했다.

| 시점 | rustfmt 잔여 hunk |
| --- | --- |
| CAP 진입 baseline | 37개: `control.rs` 4, `runtime_control.rs` 6, `queue.rs` 27 |
| CAP 교정 001 포맷 전 | 57개: 기존 37개와 이번 변경 영역 20개 |
| 이번 포맷 후 | 35개: `control.rs` 4, `runtime_control.rs` 6, `queue.rs` 25 |

이번 변경에서 새로 생긴 `runtime_facts.rs` 4개, `value_expr_runtime.rs` 7개, `control.rs` 1개, `runtime_control.rs` 5개, `queue.rs` 3개 hunk는 모두 사라졌다. Queue의 기존 hunk 두 개는 CAP가 교체한 bucket/heap 성장 본문을 이번에 포맷하면서 함께 사라졌다. 남은 35개는 이번 변경 영역 밖의 선행 포맷 차이이므로 유지했다. 따라서 전체 파일에 대한 `rustfmt --check` 종료 코드는 1이지만, 이번 변경 영역의 focused rustfmt 조건은 충족한다. 원본 로그는 [진입 baseline](verification/focused_rustfmt_baseline_exact.log), [교정 001 포맷 전](verification/focused_rustfmt.log), [포맷 후](verification/focused_rustfmt_format_only.log)에 보존했다.

## 자체 검증과 재사용

| 검증 | 결과 | 증거 |
| --- | --- | --- |
| 포맷 delta 범위·선행 333 source 대조 | PASS | [범위 receipt](verification/format_002_scope.json) |
| all-target check, offline/locked, 별도 target | PASS | [로그](verification/format_002_all_target_check.log) |
| all-target test, Unix socket 포함 | 1,475 PASS | [로그](verification/format_002_all_target_test.log) |
| 기존 allow 조건 all-target Clippy | PASS | [로그](verification/format_002_clippy_with_existing_allows.log) |
| 이번 변경 영역 rustfmt | 새 hunk 20개 제거; 범위 밖 35개 유지 | [최종 로그](verification/focused_rustfmt_format_only.log) |

PC 36조건, R05 allocation guard, CAP 경계와 action–cast 의미 시험의 직전 결과는 [CAP 교정 인계](REVIEW_HANDOFF.ko.md) 및 선행 타깃 evidence를 재사용한다. 이번 delta가 포맷 대상 블록 밖으로 나가지 않았고, 전체 제품 suite를 다시 실행했기 때문이다. PC와 R05 부하 측정을 새 성능 표본으로 다시 만들지 않았으며, 이 재사용을 새 성능 개선 주장으로 해석하지 않는다.

새 동결 증거는 [after target](target/cap_01_02_correction_002/MANIFEST.json), [after archive](target/cap_01_02_correction_002/SOURCE_AFTER_FORMAT_CORRECTION.tar.gz), [선행 타깃 대비 delta](target/cap_01_02_correction_002/SOURCE_DELTA_FROM_CORRECTION_001.tar.gz), [복원 검증](verification/format_002_target_integrity.json)에 연결한다. CAP-01·CAP-02의 의미와 기존 제외 범위는 변경하지 않았다.
