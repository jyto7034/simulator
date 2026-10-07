# R06 WP02 D04 직접 검증 기록

최종 제품 333개 source digest: `4c63ca018cd39d50e4a896711debaaf7970dc2636841a77eb74457f83313582f`. 아래 최종 직접 실행 receipt는 실행 전후 같은 digest를 기록한다. Rust 1.98.1, Cargo 1.98.1, offline locked dependency, `CARGO_HOME=/tmp/wp02-cargo-home`이다. 각 `evidence/*.json`에 정확한 명령·cwd·환경·toolchain·전후 333개 파일 hash·종료 코드·시험 수·로그 hash가, 동명 `.log`에 전체 출력이 있다.

| 실행 receipt | 결과 |
| --- | --- |
| `check_all_targets_001` | `cargo check -p game_core --all-targets --offline --locked` 성공 |
| `focused_value_expr_001` | `cargo test -p game_core --lib value_expr --offline --locked`: 74 통과 |
| `d04_workload_001` | 실제 battle adapter의 A/B/C 및 중복·순열 5개 조합 직접 측정, 1 통과 |
| `all_targets_sandbox_001` | 1,447 통과·1 실패. Unix 소켓 생성의 sandbox `Operation not permitted`; 제품 결함으로 분류하지 않음 |
| `all_targets_native_001` | 동일 `--all-targets` 명령을 승인된 환경에서 실행, 전체 1,500 통과 |
| `clippy_all_targets_001` | 지정 allow 2개를 유지한 `cargo clippy -p game_core --all-targets --offline --locked -- -A clippy::never_loop -A clippy::eq_op` 성공. 기존 경고는 로그에 보존 |
| `changed_region_format_001` | baseline 대비 변경된 9개 파일의 모든 변경 hunk에서 rustfmt 차이 0. `sim.rs`의 기존 14개 format hunk는 변경 영역 밖 |
| `pc_36_final_001` | 격리 source에 기존 PC test-only hook을 붙인 release 시험 1개 통과; `PC_SUMMARY.json`의 36조건 성공 31·typed refusal 5, gameplay oracle·상태·오류 코드 차이 0 |
| `r05_guard_001` | release allocation observer 실행 성공. `R05_SUMMARY.json`의 event peak 255,302, concurrent peak 8,997,016 bytes, 원 cap 9,030,188 bytes 대비 여유 33,172 bytes. allocation guard와 deterministic baseline parity 통과. latency gate는 적용 대상 아님 |

초기 `focused_runtime_001`의 36 통과는 비용 수치 출력 추가 전 revision에 결속된 탐색 실행이다. 최종 revision의 `focused_value_expr_001`에 runtime 시험이 포함돼 있다. 중단된 초기 PC 명령은 결과로 사용하지 않았고, 완성된 `pc_36_final_001`만 비교했다.

PC 격리 복사 전 제품 digest는 위 최종 digest와 같다. `pc_isolate_001.json`과 `pc_isolate_hooks_001.tar.gz`는 시험 전용 `EvalMember.value: Some(...)` 적응을 포함한 실제 빌드 source digest `594e585247f3fbcf5f5a70b6638f870e54b7658363b34752a528f90153f8f2a8` 및 hook bytes를 고정한다. `pc_36_final_001.json`은 그 digest의 전후 일치를 기록한다. PC는 이전 36조건의 회귀 비교이고 실제 D04 workload의 근거는 `d04_workload_001` 및 제품 시험이다.

D04 측정에서 raw 입력 4개 `[C,A,B,A]`는 canonical 3개다. WholeSet Sum/Max/Count는 10/5/3, evaluator work 34, group 1, retained 4,520 bytes다. ByProjectedTile inner Max→outer Sum 및 inner Count→outer Max는 7/2, work 36, group 2, retained 5,720 bytes다. raw 4개의 adapter 보존 상한은 2,336 bytes, raw 3개는 1,788 bytes다. 같은 canonical set의 trace와 결과는 순열에 무관하다. 이 숫자는 해당 fixture·limit의 관찰값이며 전체 게임의 메모리 상한을 대체하지 않는다.
