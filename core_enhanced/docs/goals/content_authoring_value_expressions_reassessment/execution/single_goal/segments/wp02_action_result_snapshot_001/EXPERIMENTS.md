# R06 WP02 action 결과 checkpoint 검증 기록

## 기준과 직접 실행

- 선행 교정 manifest SHA-256 `abe5a96830d8090f77e63fde7c6d7ff79675f701a382437d2492e0c66e9aa60a`, source 333개 currentness와 양방향 복원 검증 PASS. baseline `entry_001`에는 변경 전 실제 파일과 branch/HEAD 및 dirty/untracked 상태가 보존되어 있다. baseline `cargo check`도 직접 PASS였다.
- Linux `cargo 1.98.1`, `rustc 1.98.1`; `CARGO_HOME=/tmp/wp02-cargo-home`, 보통 `CARGO_TARGET_DIR=/tmp/wp02-cargo-target`, PC와 R05는 각 명령의 별도 target. `run.py`의 각 JSON에 command, cwd, 환경, toolchain, 실제 빌드 source 333개 시작/종료 hash, exit, 로그 hash와 test count가 있다. 실제 출력은 같은 이름의 `.log`다. locked dependency fetch만 수행했고 lockfile은 변경하지 않았다.
- 최종 제품 source identity: `60fe6643fa136a16773e93042d9db7d3e5b9398088c495771796abd410708717` (333개). `check_after_heap_001` 빌드 PASS, `focused_after_heap_001` value expression runtime 30/30 PASS, `format_after_heap_001` 변경 영역 PASS. `sim.rs`의 변경 영역 밖 기존 format debt 14 hunk는 그대로다.
- `all_targets_box_001`: 최종 source의 `cargo test -p game_core --all-targets --offline --locked` **1,491/1,491 PASS**. Unix socket test가 샌드박스에서 실패한 이력이 있어 승인된 실행 환경에서 직접 수행했다.
- `clippy_box_001`: 최종 source의 `cargo clippy -p game_core --all-targets --offline --locked -- -A clippy::never_loop -A clippy::eq_op` exit 0. 기존 allow 외에는 추가하지 않았다.

## PC fixture와 R05 guard

- `pc_isolate_002.json`은 최종 제품 source 333개의 격리 복사 identity와 test-only hook 2개 및 supplemental probe 2개를 묶는다. `pc_isolate_hooks_002.tar.gz`에 실제 hook bytes가 있다. `pc_box_001`은 built source digest `95540377796e526c562234273c27c82d932234bae42d960972f1bf04ff1a93d5`에서 기존 36조건 PC fixture를 직접 실행하며 `pc_raw_box.json`을 출력했다. `PC_SUMMARY_BOX.json`은 선행 결과와 성공 31/typed refusal 5, gameplay oracle 및 fault code가 같음을 확인했다. 거부의 requested 비용 상세 80행은 달라진다. 기존 fixture는 새 action prefix 사용을 직접 증명하지 않으며 새 focused 시험이 이를 담당한다.
- `R05_SUMMARY.json`은 동일 tool/workload hash, 원 concurrent cap **9,030,188 bytes**, event-heavy와 concurrent raw peak, parity, repeatability, guard를 보존한다. 최초 inline witness의 R05 두 번 실패를 숨기지 않았다. 변경 전 source 격리 대조 `r05_control_002`는 PASS였고, 일반 action의 메모리 비용을 줄인 최종 선택적 box `r05_guard_box_001`은 PASS였다. 대조의 첫 시도 `r05_control_001`은 격리 복사에서 tool의 `include_bytes!` 입력 한 파일이 빠져 빌드 실패했으며, `r05_control_supplement_001.json`의 동일 bytes를 보충해 재실행했다. 첫 샌드박스 dependency fetch는 DNS 제한으로 중단하고 승인된 fetch로 완료했다.
- R05 수치는 observer의 heap peak delta이며 RSS나 공식 PC 상한과 혼동하지 않는다. `R05_SUMMARY.json`의 현재 headroom은 관찰된 직접 실행의 값이고 이후 모든 스케줄의 보증은 아니다. 원 cap 자체를 변경하지 않았다.

## 재사용과 미실행

선행 correction의 1,490건 및 PC/R05 기록은 비교 기준일 뿐 이번 직접 PASS로 세지 않았다. 이번 초반 `all_targets_final_001` 1,491건 PASS와 `clippy_final_001` PASS는 inline witness revision의 직접 기록으로 보존하지만 최종 box revision 검증은 위 별도 receipt로 판단한다. `all_targets_002` 샌드박스 실패는 Unix socket 권한 문제이며 assertion/selector를 바꾸지 않고 승인된 실행에서 통과했다. D04 aggregate 및 D07 preview/explain의 새 기능 시험, GEX-20/30 독립 검토는 실행하지 않았다.
