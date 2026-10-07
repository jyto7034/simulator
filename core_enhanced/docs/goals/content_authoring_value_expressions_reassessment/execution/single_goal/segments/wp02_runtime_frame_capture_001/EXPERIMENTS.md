# R06 WP02 runtime frame/capture 실험 색인

상세 명령, 종료 코드, stdout/stderr, source identity는 `evidence/validation/`의 JSON receipt와 log에 실제 실행 결과로 보존한다. target은 이 bytes를 별도 evidence archive에 고정한다.

계획된 직접 검증 범주는 다음과 같다.

1. runtime frame/capture focused tests
2. resource resolver와 R05 resource 회귀
3. R04 action/delivery identity 회귀
4. WP01 value-expression/content-pipeline 회귀
5. 기존 literal battle 회귀와 publication gate
6. all-target check, focused rustfmt, 기존 조건 Clippy
7. baseline/target/end currentness verifier

## 직접 실행 결과

| 구분 | receipt | 결과 |
|---|---|---|
| 새 frame/capture adapter | `focused_runtime_frame_capture` | 7 passed / 0 failed |
| R04 action/delivery facts | `r04_runtime_fact_regression` | 11 passed / 0 failed |
| R05 resource store | `r05_resource_store_regression` | 37 passed / 0 failed |
| R05 lifetime consumer | `r05_resource_lifetime_regression` | 17 passed / 0 failed |
| WP01 ValueExpr | `wp01_value_expr_regression` | 36 passed / 0 failed |
| WP01 content pipeline | `wp01_content_pipeline_regression` | 59 passed / 0 failed |
| authoring control | `authoring_budget_regression` | 15 passed / 0 failed |
| checkpoint/save | `checkpoint_save_regression` | 18 passed / 0 failed |
| 기존 literal battle | `literal_skill_runtime_regression` | 15 passed / 0 failed |
| WP01 원 review probe | `wp01_review_probe_after_fixture_fix` | 3 passed / 0 failed / 1 ignored |
| parser depth probe | `wp01_parser_depth_probe` | 1 passed / 0 failed |
| build/probe compile | `probe_library_build`, `wp01_review_probe_compile` | exit 0 |
| all-target check | `all_targets_check` | exit 0 |
| focused rustfmt check | `focused_rustfmt_check` | exit 0 |
| 기존 조건 Clippy | `clippy_with_predecessor_allows` | exit 0; allow는 `never_loop`, `eq_op`만 사용 |

새 external pack fixture는 `/tmp/r06-wp02-runtime-frame-probe-*` 아래에 만들었고 원본 `game_resources`는 수정하지 않았다. 첫 probe 실행은 fixture 부모를 만들지 않은 driver setup 오류로 exit 101이었다. `wp01_review_probe.json/.log`와 `development_failures.*`에 무효 시도로 보존하고 검증 성공 수에서 제외했다. 부모만 만든 뒤 동일 binary와 assertion을 별도 라벨로 재실행했다.

## 재사용 증거

- 승인 correction target과 detached SHA, 감사 보고서는 mutation 전 currentness 확인 및 계보 입력으로 재사용했다.
- R01/R04/R05 과거 closure/target 보고서는 의미 계약 조사에만 재사용했다. 이번 변경의 성공 수에는 합산하지 않았고, 영향 seam은 위 R04/R05 회귀로 직접 재실행했다.
- 과거 WP01 receipt를 성공 근거로 재사용하지 않고 원 review probe, ValueExpr, content pipeline을 모두 직접 재실행했다.
