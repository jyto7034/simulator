# R06 GEX-10 실행 기록

## 2026-09-13 — 인수와 구현 전 고정

- `git status --short --branch`, `git rev-parse HEAD`: branch `cleanup/simulator-dirty-worktree-20260803`, HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`; 대규모 기존 dirty/untracked tree 확인.
- `r06_execution_baseline.py`: exit 0. `baseline/initial_001/MANIFEST.json`, actual-byte archive, scoped/global status, binary tracked diff와 detached manifest hash 생성.
- archive 검증: 307 members, `MANIFEST.sha256` OK. planning-close 비교는 462 unchanged / 7 changed / 0 missing.
- 첫 fixture selector에 `--exact`를 잘못 사용한 실행: exit 0이나 0 tests / 1361 filtered. 검증 PASS로 사용하지 않는다.
- 고유 이름 selector 재실행: `cargo test -p game_core r06_preimplementation_capture_v7_fixture --lib`; exit 0, 1 passed / 0 failed. raw V7 fixture를 생성했다.
- 임시 capture test 제거 후 `checkpoint_validation.rs` SHA-256은 initial manifest의 `fcadd623062eb4e3dccfc0162ebe6e72bc02d0aed890dda322e35cc9c2699eb5`와 일치한다.
- fixture identity: V7 raw bytes 17,299, SHA-256 `47af09db635ab1691dc61d7094d6b2414237562c27cd6e1d1b6ce9a4e49024c9`, envelope version 7, embedded checkpoint checksum `7211a94a83142ececa43e1c938febcc6093193b4698a249ebe9659d6064f31ea`; literal RON SHA-256 `1bd51a81d6b528ea4a80895bf95348c8417fd7113c77a73c6a2543d7821cf8c0` and source-copy equality confirmed.

## Validation ledger

각 실행의 command/시간/exit/full output hash는 같은 이름의 `evidence/validation/*.json`, 원시 출력은 `*.log`가 소유한다. 실행되지 않은 명령과 0-test 결과는 성공으로 세지 않는다.

### Initial nonzero baseline

| Label | 결과 | 판정 |
|---|---:|---|
| `initial_authoring` | exit 0, 4 passed | authoring selector baseline PASS |
| `initial_skill_runtime` | exit 0, 15 passed | literal/runtime integration baseline PASS |
| `initial_v7_checksum` | exit 0, 1 passed | current V7 envelope/checksum baseline PASS |
| `initial_all_targets_check` | exit 0 | all-target build baseline PASS; 기존 dead-code warning 5건 |

### Kernel implementation feedback

| Label | 결과 | 조치 |
|---|---:|---|
| `checkpoint_kernel_attempt_001` | exit 101, compile error 7건 | fold typing, ordered key traits, partial move를 수정 |
| `checkpoint_kernel_attempt_002` | exit 101, borrow error 1건 | iterative task push를 borrow 종료 뒤 수행하도록 수정 |
| `checkpoint_kernel_attempt_003` | exit 101, 1 passed / 7 failed | RON 0.5.1 serializer의 i128 미지원에 의존한 fingerprint를 명시적 canonical binary encoder로 교체 |
| `checkpoint_kernel_attempt_004` | exit 0, 8 passed | 최초 green kernel; 재실행 `004b`도 8 passed |
| `checkpoint_kernel_attempt_005` | exit 0, 10 passed | nested aggregate worst-case member product와 compile-time grouping key admission 보완 후 PASS |
| `checkpoint_kernel_attempt_006` | exit 0, 11 passed | typed dynamic frame과 owner-domain 보완 후 PASS |

자체 검토에서 단순 owner 문자열이 D04의 side/employee/equipment-copy domain 구분을 담지 못하는 것을 발견했다. `GroupOwnerDomain/GroupOwner`를 추가하고 authored 및 resolved-target member의 missing/empty/mixed domain을 evaluation 전에 거부하도록 고쳤다. authored/dynamic member identity empty 거부도 추가했다.

### Final checkpoint validation

| Label | command 요약 | 결과 |
|---|---|---:|
| `final_value_expr_unit` | `cargo test -p game_core game::value_expr::tests --lib` | exit 0, 11 passed / 0 failed / 1360 filtered |
| `final_authoring_regression` | `cargo test -p game_core game::ability::authoring::tests --lib` | exit 0, 4 passed |
| `final_skill_runtime_regression` | `cargo test -p game_core --test skill_runtime_contract` | exit 0, 15 passed |
| `final_resource_regression` | `cargo test -p game_core game::resource:: --lib` | exit 0, 27 passed |
| `final_v7_checksum_regression` | current V7 checksum selector | exit 0, 1 passed |
| `final_all_targets_check` | `cargo check -p game_core --all-targets` | exit 0, 기존 dead-code warning 5건 |
| `final_focused_fmt` | 새 value_expr 4파일 `rustfmt --check --config skip_children=true` | exit 0 |
| `final_clippy_with_baseline_allows` | lib/tests Clippy, 아래 기존 deny 2종 allow | exit 0; warnings retained |
| `post_restore_value_expr_unit` | 최종 scope 복원 뒤 value_expr unit 재실행 | exit 0, 11 passed |
| `post_restore_all_targets_check` | 최종 scope 복원 뒤 all-target build 재실행 | exit 0, 기존 dead-code warning 5건 |

### Known baseline-wide check failures

- `checkpoint_kernel_clippy`: exit 101. deny 3건은 `boss_runtime_data.rs:1764`의 `clippy::never_loop` 1건과 `world/tests/combat.rs:194-195`의 `clippy::eq_op` 2건이다. 두 파일의 현재 SHA-256/bytes가 initial manifest와 정확히 일치해 R06 변경이 아니다. 같은 두 lint를 `-A`로 낮춘 `final_clippy_with_baseline_allows`는 exit 0이다.
- `checkpoint_fmt_check`: exit 1. repo-wide `cargo fmt --all -- --check`가 다수 initial-scope 파일의 기존 포맷 차이를 보고했다. 이 실행은 PASS로 세지 않는다. R06이 새로 추가한 네 파일은 `final_focused_fmt`에서 exit 0이다.

scope hygiene 재검사에서 이전 rustfmt 호출이 `resource/mod.rs`의 child인 `resource/runtime.rs`까지 따라가 두 곳을 포맷한 사실을 target 고정 전에 발견했다. 의미 변경은 없었지만 R06 의도 범위가 아니므로 initial archive의 layout으로 복원했고 SHA-256 `401cb130765652c95d6fdc7f499a967089e588affa500bc8384857fdc72951b0`가 initial manifest와 다시 일치함을 확인했다. 이후 위 두 `post_restore_*` 검증을 재실행했다.

### Frozen inputs

`evidence/preimplementation/PREIMPLEMENTATION_FIXTURES.json`이 pre-schema V7/literal fixture, 실제 SHA-256, 생성 selector와 임시 test 복원을 기록한다. V7 SHA-256은 `47af09db635ab1691dc61d7094d6b2414237562c27cd6e1d1b6ce9a4e49024c9`, literal RON은 `1bd51a81d6b528ea4a80895bf95348c8417fd7113c77a73c6a2543d7821cf8c0`이다.
