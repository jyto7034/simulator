# D07-B F01 CLI 인자 누락 교정 — 단계검토 인계

2026-09-28 · **F01 부분 교정·자체 검증 종료 / successor target 동결 후 단계검토 요청 대기**.

## 이번 결과

simulate의 필수 REQUEST positional 인자가 없는 경우 panic/exit 101 대신
typed authoring_cli_input/exit 2로 거부한다. 최소 positional 개수를 옵션 순회 전에 검사했고,
비simulate의 exact 개수 검사와 기존 옵션 파싱은 유지했다.
제품 수정은 src/bin/author_content.rs의 한 조건이며 신규 실제 CLI integration 시험 파일을 추가했다.
F02 배열 admission과 F03 관측 검증 보강은 **미착수/미완료**다.

## 시작·결과 target

- [교정 지시서](../../handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md), F01/C01 및 해당 C16만 수행.
- [predecessor](../../segments/wp02_d07_runtime_amount_001/target/d07_runtime_amount_001/MANIFEST.json):
  SHA-256 7db52e95938de08131f352f7137be8196c5099b150d1b52874ee1d2ae0315f9f.
- before 340개: ad8021359785e365c3f9560268bcef2a95205495f9e33a22314c3f97743480c4.
- after 341개: 74da8e17a0809b8a93f1d9c9dad97a994c7549952fc92a77985df6c33305cb3a.
- [successor manifest](target/d07_b_f01_correction_001/MANIFEST.json).
  manifest 자체 SHA는 freeze.py 출력/최종 인계 메시지에서 제공한다. 자기 해시를 본문에 넣는 순환 참조를 만들지 않는다.
- before archive, after archive, source delta/restore patch, evidence/인계 archive를 target에 결속한다.
  실제 고정·복원 성공 여부는 [복원 검증](target/d07_b_f01_correction_001/RESTORATION_VERIFICATION.json)과
  이 디렉터리의 freeze.py verify로 재확인한다.

직전 검토 대상과 최초 D07-B target은 이번에 동일하다. 그 대비 변경은 CLI 조건과 시험 추가뿐이다.
predecessor 339개 경로가 byte 동일하며 src/tests의 파일 membership도 1개 추가 외 동일함을 확인했다.
동결 원본·실패·검토 판정을 수정하지 않았다.

## 실제 검증과 실패 보존

| 항목 | 결과·근거 |
| --- | --- |
| 교정 전 실제 회귀 | 3 pass / 1 fail, exit 101. simulate REQUEST 누락의 panic 검출. evidence/f01_before_regression_001.json 및 .log |
| 교정 후 integration | 4 pass / 0 fail, 실제 subprocess 30조건. evidence/f01_after_regression_001.json |
| 정상 CLI | simulate 기본/trace/amount/양 순서 조합과 validate/describe/explain 총8명령 exit 0. evidence/cli_matrix_001/executions.json |
| 출력 보존 | plain=trace-only, amount의 trace off/on·양 순서 동일, predecessor amount 출력과 byte 동일. evidence/cli_matrix_001/SUMMARY.json |
| 변경 target check/clippy | exit 0. 기존 warning 보존. evidence/f01_check_001.json, f01_clippy_001.json |
| 변경 두 파일 format | exit 0. evidence/f01_format_001.json |

integration은 동일 Cargo invocation이 빌드한 실제 CARGO_BIN_EXE_author_content를 실행한다.
정상 CLI script도 같은 target/debug 바이너리를 실행하며 바이너리 SHA와 명령·stdout/stderr를 기록했다.
로그는 즉시 저장했고 장시간 실행의 진행 표시를 유지했다. 실패 원자료를 성공 결과로 덮어쓰지 않았다.
첫 동결 후보는 stdout 결과 파일 자기 포함 때문에 후속 verify에서 거부됐다.
후보 전체와 원본 freeze.py를 evidence/failed_freeze_001에 보존하고 수집 규칙을 교정했다.
이 후보는 유효 target이 아니며, 위 successor manifest와 별도 verify 결과를 검토 기준으로 삼는다.
추가 무관한 formatting/refactor, 새 parser 의존성, panic catch는 없다.

## 증거 재사용과 남은 범위

[재사용 판정](evidence/EVIDENCE_REUSE.json)에 core library/data/dependency/tool source 무변경과 선행 로그 SHA를 연결했다.
선행 전체1,538/authoring112/value_expr91/PC36/R05는 해당 무변경 runtime 범위의 근거로 재사용했다.
**이번 successor에서 전체 시험1,538개나 PC36/R05를 다시 실행한 것은 아니다.**
새 CLI 행위는 위 신규 시험/실제 실행으로 별도 확인했다. 기존 R05 peak 경고 정책과 다른 hard guard는 그대로다.

B18의 CLI 인자 부분만 보강했다. 큰 i128 trace, 동적 Snapshot/Live, retained/clone/취소·교체 경계 등
F03의 남은 C03–C15 요구는 이 결과로 완료 처리하지 않는다. F02/C02는 다음 교정 대상이다.
20분 시험·production expression 공개·D07-C/WP03·WP04는 수행하지 않았다.

## 기록과 검토 요청

- [PLAN](PLAN.md): 현재 위치·근거·미완료 finding·다음 행동.
- [EXPERIMENTS](EXPERIMENTS.md): before 실패, 수정, 검증, 도구 오류·재시도 기록.
- [EXPERIMENT_NOTES](EXPERIMENT_NOTES.md): 선택 이유와 재사용 판단·후속 제약.

F01은 독립적으로 관찰 가능한 입력 오류 경로가 교정되고 관련 검증이 통과했으므로 첫 의미 있는 checkpoint로 선택했다.
사본 대화에서 고정 target·diff·원자료를 읽기 전용으로 검토해 달라.
구현자는 F01 검토 PASS, F02/F03 완료, D07-B/R06 전체 완료를 선언하지 않는다.
