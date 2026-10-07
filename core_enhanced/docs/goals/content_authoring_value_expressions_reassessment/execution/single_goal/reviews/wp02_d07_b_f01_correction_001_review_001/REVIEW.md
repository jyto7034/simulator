# R06 WP02 D07-B F01 — 교정 단계검토 001

2026-09-28 · **F01 부분 범위 PASS (PARTIAL_SCOPE_PASS)** · 이번 범위의 추가 수정 요구 없음.

필수 REQUEST가 없는 simulate의 panic을 typed 입력 오류/exit 2로 교정한 것을 확인했다.
이 판정은 F01/C01, B18의 CLI 입력 부분과 C16의 해당 근거에 한정된다.
**F02·F03은 계속 미완료**이며 D07-B/WP02/R06 전체 완료나 다음 구간의 자동 실행을 뜻하지 않는다.

## 1. 대상과 검토 역할

- [교정 지시서](../../handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md)
- [선행 수정 필요 판정](../wp02_d07_runtime_amount_001_review_001/REVIEW.md)
- [구현 인계](../../corrections/wp02_d07_b_review_001/REVIEW_HANDOFF.ko.md)
- [동결 manifest](../../corrections/wp02_d07_b_review_001/target/d07_b_f01_correction_001/MANIFEST.json)
- Manifest SHA-256: `bf51ac15713acc3afa6a850452072e917be94df34039a226cadc03624b97da8d`
- before 340개 digest: `ad8021359785e365c3f9560268bcef2a95205495f9e33a22314c3f97743480c4`
- after 341개 digest: `74da8e17a0809b8a93f1d9c9dad97a994c7549952fc92a77985df6c33305cb3a`

원본 대화가 구현하고 분기한 이 대화가 동결 뒤 읽기 전용으로 검토했다. 검토 중 제품 소스와 동결 target을 수정하지 않았다.
공유 설계 이력이 있으므로 설계 참여까지 배제한 엄격한 독립 완료 감사로 부르지 않는다.

## 2. 코드 판정

제품 변경은 `core_enhanced/src/bin/author_content.rs::Arguments::parse`의 한 조건이다.
`args.len() < expected`를 모든 operation에 적용하고, 비simulate의 정확한 길이 검사는 유지했다.
이로써 필수 인자 수 확인이 옵션 순회 및 `args[3]` 접근보다 먼저 실행된다.
새 parser abstraction, catch-panic, 기본 REQUEST 대입이나 runtime 동작 변경은 없다.

신규 `core_enhanced/tests/author_content_cli_arguments.rs`는 실제 `CARGO_BIN_EXE_author_content`를 실행한다.
4개 시험에서 총 30회 subprocess를 호출하며 exit 2, 빈 stdout, 단일 JSON request diagnostic,
`authoring_cli_input`/`field_path=cli`를 함께 확인한다. 필수 positional 누락, 잘못된 옵션·중복·trace path 누락,
비simulate 옵션 사용, 명령/길이/개수 제한을 확인한다. 모든 30회가 서로 다른 입력이라는 뜻은 아니다.

F01 교정으로 유효 입력의 parser/CLI 출력 경로가 바뀌지 않았음을 정상 실행 행렬로 대조했다.
F02의 legacy Vec admission이나 F03의 동적 관측을 이 수정으로 해소했다고 주장하지 않았다.

## 3. 직접 실행과 근거 재사용

직접 재실행:

- `cargo test -p game_core --test author_content_cli_arguments --offline --locked`:
  **4 passed / 0 failed**, exit 0. [로그](CLI_ARGUMENT_TESTS.log).
- 실제 정상 CLI 8건: simulate 기본, trace만, amount만, amount→trace, trace→amount,
  validate, describe, explain. **8건 모두 exit 0**, stdout/stderr가 해당 동결 원자료와 byte 동일.
  trace 출력 경로만 검토 전용 새 경로로 바꿨다. [명령·결과·바이너리 해시](CLI_MATRIX_RECHECK.json).
- currentness와 격리 정·역 byte 복원: 340→341 및 341→340 통과.
  src/tests membership, 신규 test 부재↔존재, 동결 기록 243개까지 확인했다.
  [진입](ENTRY_CURRENTNESS_AND_RESTORATION.json), [종료](EXIT_CURRENTNESS_AND_RESTORATION.json).

실행 환경: repository root, `/home/blast/.cargo/bin/cargo`,
`CARGO_HOME=/tmp/wp02-cargo-home`,
`CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/d07-runtime-amount-product-target`.
검토 진입/종료에 대상이 동일함을 확인했다.

선행 340개 중 **339개 byte 동일**, 변경은 CLI 파일 하나, 추가는 integration test 하나임을 predecessor와
직접 대조했다. core library/data/dependency/tool에 변경이 없으므로 해당 runtime의 전체 1,538개 시험,
authoring 112/value_expr 91, PC36/R05 근거 재사용은 이번 CLI 교정 범위에 타당하다.
선행 로그 SHA와 구현자의 새 check/clippy/format/CLI/test 영수증 및 교정 전 실패 로그를 확인했다.
[범위·영수증 대조](SCOPE_AND_RECEIPTS_CHECK.json).

전체 시험·check·clippy·format·PC36/R05를 이번 검토에서 새로 실행했다고 표시하지 않는다.
전체 successor의 시험 수를 선행 1,538에 단순 합산해 새 전체 실행 결과로 보고하지 않는다.
R05 concurrent peak 경고 정책도 그대로이며, F03의 미검증 조건을 이 재사용 근거로 닫지 않는다.

## 4. 기록과 종료

구현자는 PLAN/EXPERIMENTS/EXPERIMENT_NOTES의 세 역할을 사용했고, 실제 교정 전 3 pass/1 fail과
교정 후 4 pass를 구분했다. 첫 동결 후보의 결과 파일 자기 포함 문제도 실패 후보·원본 도구와 함께 보존했다.
최종 target은 해당 실패 후보와 분리되어 있으며 이번 검증에서 실제 일치·복원을 확인했다.
동결된 인계/PLAN의 ‘검토 대기’는 당시 상태로 보존하고, 새 판정은 이 보고서와 비대상 현황 문서에 연결한다.

**D07-B-F01은 이 successor에서 해소되었다.** 다음 작업은 원본 구현 대화에서 기존 교정 지시서 §7에 따라
F02/C02의 legacy amount 배열 사전 admission을 교정하고 검증하는 것이다. F03의 관측 검증 보강은 이후에도 남는다.
이 검토에서는 다음 구현을 시작하지 않았으며 재개 요청 후 진행한다.
