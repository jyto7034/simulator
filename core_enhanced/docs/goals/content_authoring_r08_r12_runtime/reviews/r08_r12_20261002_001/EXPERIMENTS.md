# 감사 실행 원자료

2026-10-02~03. 명령별 receipt에는 cwd/env/rustc·전후 source membership/hash·exit·시간·raw log hash가 있다.

일반 명령의 test count0은 시험 통과로 세지 않는다. cancellation_oracle은 별도 공개 시나리오 oracle5개이며 결과는2 PASS/3 FAIL, exit1이다.

| 명령 기록 | exit | Rust tests pass/fail | 원명령 |
| --- | --- | --- | --- |
| [all_default_001](evidence/all_default_001.json) / [raw](evidence/all_default_001.log) | 0 | 1677/0 | `cargo test -p game_core --all-targets --offline --locked` |
| [all_feature_001](evidence/all_feature_001.json) / [raw](evidence/all_feature_001.log) | 0 | 1677/0 | `cargo test -p game_core --all-targets --features test-fixtures --offline --locked` |
| [cancellation_oracle_001](evidence/cancellation_oracle_001.json) / [raw](evidence/cancellation_oracle_001.log) | 1 | 0/0 | `python3 core_enhanced/docs/goals/content_authoring_r08_r12_runtime/reviews/r08_r12_20261002_001/check_cancellation_oracle.py` |
| [dependency_fetch_001](evidence/dependency_fetch_001.json) / [raw](evidence/dependency_fetch_001.log) | 0 | 0/0 | `cargo fetch --locked` |
| [release_001](evidence/release_001.json) / [raw](evidence/release_001.log) | 0 | 0/0 | `cargo build -p game_core --release --bins --offline --locked` |
| [timeout_cancel_001](evidence/timeout_cancel_001.json) / [raw](evidence/timeout_cancel_001.log) | 0 | 0/0 | `author_content simulate (exact full command in receipt)` |
| [timeout_detach_001](evidence/timeout_detach_001.json) / [raw](evidence/timeout_detach_001.log) | 0 | 0/0 | `author_content simulate (exact full command in receipt)` |
| [timeout_marker_cancel_simulate_002](evidence/timeout_marker_cancel_simulate_002.json) / [raw](evidence/timeout_marker_cancel_simulate_002.log) | 0 | 0/0 | `author_content simulate (exact full command in receipt)` |
| [timeout_marker_cancel_trace_002](evidence/timeout_marker_cancel_trace_002.json) / [raw](evidence/timeout_marker_cancel_trace_002.log) | 0 | 0/0 | `author_content simulate (exact full command in receipt)` |
| [timeout_marker_cancel_validate_002](evidence/timeout_marker_cancel_validate_002.json) / [raw](evidence/timeout_marker_cancel_validate_002.log) | 0 | 0/0 | `author_content validate (exact full command in receipt)` |
| [timeout_marker_detach_simulate_002](evidence/timeout_marker_detach_simulate_002.json) / [raw](evidence/timeout_marker_detach_simulate_002.log) | 0 | 0/0 | `author_content simulate (exact full command in receipt)` |
| [timeout_marker_detach_validate_002](evidence/timeout_marker_detach_validate_002.json) / [raw](evidence/timeout_marker_detach_validate_002.log) | 0 | 0/0 | `author_content validate (exact full command in receipt)` |
| [timeout_marker_early_cancel_simulate_002](evidence/timeout_marker_early_cancel_simulate_002.json) / [raw](evidence/timeout_marker_early_cancel_simulate_002.log) | 0 | 0/0 | `author_content simulate (exact full command in receipt)` |
| [timeout_marker_early_cancel_validate_001](evidence/timeout_marker_early_cancel_validate_001.json) / [raw](evidence/timeout_marker_early_cancel_validate_001.log) | 1 | 0/0 | `author_content validate (exact full command in receipt)` |
| [timeout_marker_early_cancel_validate_002](evidence/timeout_marker_early_cancel_validate_002.json) / [raw](evidence/timeout_marker_early_cancel_validate_002.log) | 0 | 0/0 | `author_content validate (exact full command in receipt)` |
| [timeout_marker_inflight_cancel_simulate_003](evidence/timeout_marker_inflight_cancel_simulate_003.json) / [raw](evidence/timeout_marker_inflight_cancel_simulate_003.log) | 0 | 0/0 | `author_content simulate (exact full command in receipt)` |
| [timeout_marker_inflight_cancel_validate_003](evidence/timeout_marker_inflight_cancel_validate_003.json) / [raw](evidence/timeout_marker_inflight_cancel_validate_003.log) | 0 | 0/0 | `author_content validate (exact full command in receipt)` |
| [timeout_marker_state_exit_simulate_003](evidence/timeout_marker_state_exit_simulate_003.json) / [raw](evidence/timeout_marker_state_exit_simulate_003.log) | 0 | 0/0 | `author_content simulate (exact full command in receipt)` |
| [timeout_marker_state_exit_validate_003](evidence/timeout_marker_state_exit_validate_003.json) / [raw](evidence/timeout_marker_state_exit_validate_003.log) | 0 | 0/0 | `author_content validate (exact full command in receipt)` |
| [x09_retry_002](evidence/x09_retry_002.json) / [raw](evidence/x09_retry_002.log) | 101 | 0/0 | `cargo test -p game_core --lib --offline --locked game::data::content_pack::snapshot::tests::unix::invalid_utf8_directories_and_socket_inputs_return_errors -- --exact` |
| [x09_retry_003](evidence/x09_retry_003.json) / [raw](evidence/x09_retry_003.log) | 0 | 1/0 | `cargo test -p game_core --lib --offline --locked game::data::content_pack::snapshot::tests::unix::invalid_utf8_directories_and_socket_inputs_return_errors -- --exact` |
| [x09_sandbox_001](evidence/x09_sandbox_001.json) / [raw](evidence/x09_sandbox_001.log) | 101 | 0/0 | `cargo test -p game_core --lib invalid_utf8_directories_and_socket_inputs_return_errors --offline --locked` |

기존 동결 원자료 검증은 AUDIT_TARGET.json(1435 records/9 archives)에 있고, 재사용49명령(검사4+CLI45)은 REUSED_EVIDENCE.json에 있다.
PC_RECOMPARE/R05_RECHECK는 원 workload를 새로 돌린 결과가 아니라 보존 원자료의 새 비교·guard 계산이다.

실패 이력: x09_sandbox_001/x09_retry_002는 의존성 캐시 부재로0 test/exit101; dependency_fetch_001 뒤 x09_retry_003과 전체 원명령을 재실행했다.
timeout_marker_early_cancel_validate_001은 최초 새 probe의 unreachable control node를 거부했다. 원 source 결함으로 계산하지 않고 원자료를 보존했다. 후속002/003은 공개 validate 통과 뒤 정상 simulate했다.
cancellation_oracle_001의 exit1은 실제 제품 계약 반례이며 F01에 연결한다. 테스트나 제품을 수정하지 않았다.
격리 복원과 before/after archive 검사는 verify_target.py에서 직접 수행했다. 사용자 원본에 역 patch를 적용하지 않았다.

R06/R07 영향33개 integration/copy 그룹과 R08~R12의55조건은 verify_evidence.py에서 각 selector가 새 두 전체 raw log에 실제로 나타나는지 확인했다.
