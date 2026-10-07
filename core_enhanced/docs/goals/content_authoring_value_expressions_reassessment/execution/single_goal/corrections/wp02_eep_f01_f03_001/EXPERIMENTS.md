# 교정 자체 검증 기록

모든 경로는 이 교정 디렉터리 기준이다. `run.mjs` receipt는 명령·cwd·시작/종료 시각·exit code·333개 source의 시작/종료 hash를 보존한다. `summarize.mjs`가 최종 성공 receipt의 source currentness를 다시 검사했다.

## 직접 실행

- `evidence/all_tests_002.*`: 승인된 환경, `cargo test -p game_core --all-targets --offline --locked`, lib 1435 + integration 52 = **1487 PASS**.
- `evidence/check_final.*`: all-target check exit 0.
- `evidence/clippy_final.*`: all-target Clippy, 기존 `clippy::never_loop`, `clippy::eq_op` allow 두 개. exit 0이며 warnings-free라는 뜻은 아니다.
- `evidence/changed_format.*`: baseline/current/rustfmt 결과를 비교. 변경 영역과 format diff 교집합 없음. value_expr_runtime 파일 전체는 clean; sim의 기존 차이 14 opcode 구간은 범위 밖으로 유지했다. 이 구간 수를 이전 handoff의 hunk 수와 직접 비교하지 않는다.
- `evidence/review_probes_002.*`: 기존 감사 5 + 추가 경계 2 = **7 PASS**. 실제 동결 전 교정 source의 격리 복사에서 수행.
- `evidence/pc_001.*`, `pc_raw.json`, `PC_COMPARISON.json`: 동일 PC fixture의 timing 11 + allocation 3 samples 및 각 warm-up. 36조건에서 기존과 같은 31완료/5거부. 원래 probe allocator에 감사 probe의 thread-local 요청량 관측을 연결했으므로 timing의 미세한 직접 비교를 합격 근거로 사용하지 않는다.
- `evidence/r05_guard_001.*`, `r05_guard_raw.json`, `R05_SUMMARY.json`: allocation observer 활성 R05 release guard 재실행. event peak 255302, concurrent max peak 8945386, 원 cap 9030188. repeatable/기준 event parity/두 allocation guards PASS.

`evidence/isolate_final_identity.json`은 최종 격리 source의 exact 두 hook과 333개 source를 검증한다. `evidence/review_probes.rs`, `pc_probe.rs`는 실제 실행된 최종 probe bytes이다. 최초 `isolate_identity.json`은 fixture의 ResourceId unwrap 수정 전 hash이므로 최종 probe identity로 사용하지 않는다.

## 실패·설정 오류도 보존

- `evidence/check_initial.*`: 추가 제품 시험에서 SkillStepResult의 잘못된 모듈 경로로 compile 실패. 시험 타입 경로와 unused Arc import를 교정했다. 이 실행 중 source를 수정했으므로 성공 검증으로 재사용하지 않는다.
- `evidence/all_tests_001.*`: 1434 PASS / Unix socket 생성 시험 1개 EPERM. 제품 결함으로 우회하거나 시험을 skip하지 않고 동일 전체 명령을 승인 재실행한 `all_tests_002`로 검증했다.
- `evidence/review_probes_001.*`: 격리 추가 fixture가 ResourceId::new의 Result를 unwrap하지 않아 compile 실패. 제품 변경 없이 fixture를 교정한 `review_probes_002`가 최종 실행이다.
- baseline archive 생성의 최초 Node pipe 실행이 끝나지 않아 중단했고, 다음 샌드박스 시도는 spawnSync EPERM이었다. 제품 수정 전 승인된 재실행으로 baseline을 완성했다. 중단된 임시 archive를 유효한 baseline으로 사용하지 않았다.
- 요약 스크립트 최초 실행은 PC receipt 생성 완료 전에 요청되어 종료됐다. 완료 후 재실행으로 동일 bytes의 기존 산출물만 재사용하고 summary를 완성했다. 실패한 요약은 시험 PASS로 계산하지 않는다.

## 재사용 증거와 한계

수정 전 finding 재현은 `../../reviews/wp02_effect_execution_prefix_001_review_001/probes_expanded.log`의 검토 결과를 재사용했다. 이번 교정에서 before source에 그 실패 시험을 다시 돌리지는 않았다. predecessor source와 새 baseline bytes 일치를 먼저 확인했다. 이전 제품 전체 시험을 이번 최종 검증으로 재사용하지 않고 위 명령들을 직접 실행했다.

새 native/authoring cap은 만들지 않았고 기존 cap도 늘리지 않았다. 미구현 action-wide witness/ResolvedTargets를 이 검증의 지원 범위로 포함하지 않는다. PC fixture는 새 effect execution의 모든 조합이나 실제 Windows/macOS 하드웨어 시험을 대체하지 않는다.
