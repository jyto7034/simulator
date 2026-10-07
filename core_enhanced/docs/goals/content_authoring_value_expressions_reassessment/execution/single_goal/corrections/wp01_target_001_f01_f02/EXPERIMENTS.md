# R06 WP01 F01/F02 교정 실행 기록

## 진입

- `python3 .../reviews/wp01_content_input_compile_001_review_001/verify_target.py`: PASS. 요청된 predecessor manifest SHA-256와 source/archive/reference currentness 일치.
- 새 baseline `R06_WP01_CONTENT_INPUT_COMPILE_CORRECTION_BASELINE_001` 고정. 자세한 명령·exit·원시 출력은 `evidence/validation/*.json|*.log`에 누적한다.

## 실행 분류

- 직접 실행: 이 correction pass에서 새로 생성한 receipt와 raw log.
- 재사용: predecessor target과 review의 frozen evidence를 hash/currentness로 확인한 결과. 직접 실행 test 수에 합산하지 않는다.

## 수정 전 직접 실행

- `pre_fix_build`: exit 0.
- `pre_fix_review_probe_compile`: exit 0.
- `pre_fix_review_probe`: exit 101. 새 fixture 부모를 만들지 않은 setup 오류가 외부 pack case보다 먼저 발생했으므로 invalid attempt로 보존하고 성공/결함 재현 수에 세지 않는다. 같은 실행의 direct parser F01은 실패했다.
- `pre_fix_review_probe_reproduced`: exit 101. 새 `/tmp/r06-wp01-f01-f02-pre-sLwwuB/pack-fixture`에서 원 assertion을 유지해 1 pass / 2 expected finding failures / 1 ignored를 재현했다. F01은 unknown Mul이 조용히 성공했고, F02는 authored `-1` Sum Damage가 publication gate까지 도달했다.

## 구현 중 직접 실행

- `intermediate_value_expr_unit`: 36/36 PASS.
- `intermediate_static_fault_shared_evaluator`: 1/1 PASS. AggregateMember가 제공한 scalar 0을 분모로 evaluator 단계 fault를 확인했다.
- `intermediate_strict_parse_source_ids`: 7/7 PASS.
- `intermediate_external_contract_regression`: 1/1 PASS.
- 초기 post-fix review probe도 3 pass / 1 ignored였다. 이후 최종 source에서 다시 실행했으므로 최종 판정에는 final receipt를 사용한다.

## 최종 source 직접 실행

- `post_fix_focused_fmt`: exit 0.
- `post_fix_value_expr_unit`: 36/36 PASS.
- `post_fix_content_pipeline_regression`: 59/59 PASS.
- `post_fix_authoring_regression`: 15/15 PASS.
- `post_fix_skill_runtime_regression`: 15/15 PASS.
- `post_fix_checkpoint_save_regression`: 18/18 PASS; current V7 envelope/checksum 회귀 포함.
- `post_fix_all_targets_check`: exit 0.
- `post_fix_clippy_with_baseline_allows`: exit 0. predecessor와 동일한 command-line allow `clippy::never_loop`, `clippy::eq_op`만 사용했다. 기존 warning 집합은 raw log에 남겼다.
- `post_fix_final_probe_build`, `post_fix_final_review_probe_compile`: exit 0.
- `post_fix_final_review_probe`: 3 pass / 0 fail / 1 ignored. 외부 authored `-1` Sum은 `value_expr_overflow`, unknown Mul은 `ron_parse`, 정상 표현식은 `content_capability_unsupported`였고 실제 definition/action/effect 식별자가 유지됐다.
- `post_fix_parser_depth_probe`: ignored case를 별도 실행해 1/1 PASS; depth 64, nodes 127, 2 MiB worker stack의 typed parse가 정상 반환했다.

모든 fixture는 correction 전용 새 `/tmp` 디렉터리를 사용했다. 저장소 `game_resources`는 source baseline 대조에서 불변이다.

## 재사용한 검증

- predecessor target manifest SHA-256 `6f55cfa0fae0d88971eba6cb3c47e39ab5d12a2e99b2656a40eb2b51e7cf76de`의 구현자 verifier와 reviewer 독립 verifier 결과를 entry currentness로 재사용했다.
- predecessor의 source before/after, records/evidence archives와 441 reference hash를 검증했으나 그 target의 기존 test 수를 이번 직접 실행 수에 합산하지 않았다.
