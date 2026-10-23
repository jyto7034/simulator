# U17 검증 재현과 자료 읽기

먼저 `CURRENT_RESULTS.json`, `COVERAGE.json`, `BASELINE_REUSE.json`과 target record를 읽는다.
원시 테스트/명령 로그와 과거 실패는 `TEST_RESULTS_INDEX.json`이 연결한다. 보존된 공개 시나리오의
`host/PROCESS.json`에는 실행 argv·바이너리/config SHA와 exit가 있고 `host/trace.jsonl`에는 요청과 전체
공개 응답이 있다. `milestones`는 정상 publish가 끝난 실제 active/profile 쌍의 사본이다.

현재 release host는 `artifacts/normal002`, 계측 승인 worker는 `artifacts/instrumented002`다.
`normal_build002`/`instrumented_build002`의 prepared-build와 dependency tree, 각 바이너리 receipt,
`runs/*package002`, `*calibration002`, `*limit-review002`, `*adoption002`를 함께 확인한다. admission suite는
10초 전투 준비용이며333명령 전체 기능 시나리오를 대신하지 않는다. 최종 정상 입력은
`test_parts/normal_inputs002`, 확장 입력은 `expanded_inputs003`이다.

환경은 PATH의 `/home/blast/.cargo/bin`, CARGO_HOME=/tmp/wp02-cargo-home, 기존 공유 dev target,
CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0, `--offline --locked`다. 공유 target 빌드는
동시에 실행하지 않는다. Core 일반 영향 검사는 `cargo test -p game_core --lib`의 `u17_`,
`game::world::tests::combat`, `checkpoint_validation`, U08 Continue/event owner 및 G11 선택 목록을 사용했다.
native35는 같은 영향 목록에 `--features runtime-observation`를 사용했다. 전체 workspace는 한 번 실행한
뒤 실패·변경 영향만 다시 실행했으며 원시 로그의 실제 selected/pass/fail/ignore를 확인한다.

`test_parts/full_flow.py [ADMISSION_CONFIG] [CHOICE_COUNT]`가 현재 정상 경로를 실행하며 기본값은
normal_inputs002와3이다. 확장은 expanded_inputs003의 admission.json과64를 전달한다.
새 출력 디렉터리를 사용한다. `fresh_readers.py NORMAL_ROOT`와
`io_replay.py NORMAL_ROOT/host/trace.jsonl`는 동일 issuer를 공유하므로 순차 실행한다. 동일 issuer 병렬
실행은 admission cache.lock 충돌을 만들며 게임 IO 원자성 검사로 인정하지 않는다. 기존 issuer가 없으면
지원 준비 도구로 새 issuer/calibration/adoption을 발행하고 config의 새 경로를 결속해야 한다. 기존
동결 config/receipt를 편집해 재현 성공으로 표기하지 않는다.

진단 비용 바이너리는 `artifacts/diagnostic004`다. `u17_completion_cost CONFIG TRACE NEW_RESULT`는
공개 Core 경로를 재생하고 `restore_cost CONFIG INPUT_DIRECTORY EXPECTED_RESPONSE NEW_RESULT`는
실제 저장에서 base64 decode만 한 Core payload로 Continue를 측정한다. `collect_cost.py NORMAL_ROOT
EXPANDED_ROOT`는 각각3회 실행한다. 이번 cost002는 옵션으로 기존 normal forward3회를 byte-identical
바이너리/입력 근거로 재사용했고 새 Continue6회와 expanded forward3회를 실행했다. 원문 EXECUTIONS,
BINDINGS, PROVENANCE, 결과 JSON, `/usr/bin/time -v`가 모두 보존되어 있다. dev 비용을 배포 성능으로
읽거나 parsed trace 메모리를 게임 단독 heap으로 읽지 않는다.

`SOURCE.tar.gz`는 현재 snapshot, `BEFORE.tar.gz`는 실제 baseline+명시 unchanged external references다.
`DELTA.json`과 `SCOPED.diff`가 범위를 식별한다. `RESTORE.json`은 소유한 임시 디렉터리에서 archive
member·bytes 전체를 정방향/역방향으로 대조한 결과다. 실제 dirty 작업 트리에 patch/reset/clean을 적용하지
않는다. finalization script를 과거 출력 위에 재실행하지 않는다. 감사 시 entry/exit currentness는 새 감사
경계에서 확인한다. 최종 감사와 U18 이후 구현은 이 재현 안내의 자동 후속 작업이 아니다.
