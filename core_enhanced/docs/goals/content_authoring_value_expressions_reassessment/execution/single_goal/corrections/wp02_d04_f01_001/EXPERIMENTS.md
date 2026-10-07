# R06 WP02 D04-F01 검증 기록

최종 제품 333개 source digest는 `34ceedbebc84b9396d42f92fa31033fe0b831a90e5325c8c4463e0c88f067145`다. 최종 직접 실행 receipt의 `source_before`·`source_after`는 모두 이 digest다. `run.py`가 명령·cwd·환경·Rust/Cargo 버전·각 333개 파일 hash·exit·실제 시험 수·로그 hash를 동명 JSON에, 전체 출력을 동명 `.log`에 기록한다. Rust/Cargo 1.98.1, offline locked dependency, 기본 `CARGO_HOME=/tmp/wp02-cargo-home`, `CARGO_TARGET_DIR=/tmp/wp02-cargo-target`다.

| 최종 receipt | 직접 결과 |
| --- | --- |
| `check_all_targets_final_002` | `cargo check -p game_core --all-targets --offline --locked` 성공 |
| `focused_d04_f01_final_002` | 신규 제품 9건 통과. target 소실 4조합, inactive, 이동 후 미명중 timeout, SourceEmployee 후속 자원, 빈 step 결과·capture identity, TargetEmployee 오류 rollback |
| `focused_value_expr_final_002` | 기존 value-expression 74건 통과. D04 empty frame·member·tile·action prefix 회귀 포함 |
| `target_locked_success_final_002` | 실제 target hit에서 release Snapshot Attack5 / impact Live Attack9, 1건 통과 |
| `area_live_success_final_002` | area tick별 Live member 재준비, 1건 통과 |
| `all_targets_native_final_002` | 전체 discovery 1,509건 통과. 이전 D04 실행에서 Unix 소켓 시험이 sandbox EPERM으로 실패한 기록에 따라 승인된 환경에서 직접 실행 |
| `clippy_all_targets_final_002` | 지시된 두 allow만 둔 all-targets Clippy 성공; 기존 경고는 로그에 보존 |
| `changed_region_format_final_002` | 4개 변경 파일의 수정 hunk rustfmt 차이 0. `sim.rs`의 기존 14개 format hunk는 수정 영역 밖 |
| `pc_36_final_002` | 최종 제품 복사본에 test-only probe를 붙인 release 시험 1건 통과; `PC_SUMMARY.json`의 기존 36조건 성공31/typed refusal5, gameplay oracle·완료 상태·오류 code 차이 0 |

독립 검토의 교정 전 실패 4건과 선행 상수식 통과 2건은 `PRIOR_EVIDENCE.json`이 D04 manifest 및 원 검토 파일과 복사본의 bytes를 대조한다. 이 교정의 신규 4건 성공은 `focused_d04_f01_final_002`와 최종 전체 시험에서 실제 제품 경로로 실행됐다. `four_reproductions_001`의 `query_action` assertion, `focused_extended_001`의 import/queue fixture, `focused_extended_002`·`focused_extended_003`의 resource catalog 구성은 탐색 단계의 시험 oracle·fixture 오류다. 해당 실패 로그는 보존하되 제품 회귀로 집계하지 않는다. `all_targets_native_final_001`은 effect-row 구조 확인 추가 전 source의 1,509건 통과로 보존하고 최종 판정에는 `_002`를 사용한다.

PC의 `pc_isolate_002.json`은 최종 제품 복사 digest 및 `pc_probe.rs`의 test-only `EvalMember.value: Some(...)` 적응 후 실제 빌드 digest `85cfa136f14716bdea5e9482ed1fdf2b6565f7c87ac08b050c560e205363503f`를 기록한다. `pc_isolate_hooks_002.tar.gz`가 hook bytes를 보존하고 `pc_36_final_002.json`은 그 built digest 전후 일치를 기록한다. `pc_isolate_001`은 구조 확인 전 source여서 최종 PC의 근거로 사용하지 않는다.

## R05 allocation guard 변동

원 concurrent peak cap은 **9,030,188 bytes**이며 변경하지 않았다. 동일 tool/workload hash·allocation observer·fixture에서 최종 교정 소스의 첫 두 직접 실행은 최대 9,040,041 및 9,040,546 bytes로 guard 실패했다. 동결 D04 소스를 별도 격리해 같은 도구로 직접 실행한 대조도 최대 9,050,125 bytes로 실패했다. 마지막 최종 교정 소스 직접 실행 `r05_guard_after_control_001`은 최대 **9,020,761 bytes**, 상한 대비 **9,427 bytes 여유**로 통과했다. 모든 완성 실행의 event-heavy guard, 결과 hash/parity, allocation 횟수, retained delta는 동일하다. `R05_SUMMARY.json`에 전체 sample·실패·대조·통과를 보존했다. 동시 스케줄의 peak 변동을 감추지 않으며, latency gate는 적용 대상이 아니다.

`r05_predecessor_control_001`은 격리 도구의 `include_bytes!` 보조 입력 하나가 누락돼 compile 실패했다. 원본과 정확히 같은 bytes를 추가한 `r05_control_supplement_001.json`과 함께 `_002`를 다시 실행했다. 격리 빌드의 333개 source identity는 `r05_predecessor_control_002.json`, tool 파일은 `r05_control_isolate_001.json`에 결속된다. R05 측정치는 observer의 heap peak delta이며 RSS로 해석하지 않는다.
