# U03 검증 기록

P2G-40/50 current validators passed; semantic mapping and Handoff complete.
Initial baseline reused from exact U02 audited source; all post-change U03 evidence pending.
GEX-00 source/dirty archive complete; no production mutation.

- core_check_001, host_check_001: initial implementation compiles with observation; existing warnings retained.
- core_fault_tests_001: compile failure in new test API usage (Result vs Option; non-PartialEq counter struct), fixed tests.
- core_fault_tests_002: 3 passed / 2 fixture assumptions failed (tiny profile rejected by generated-content admission; no checkpoint before a SavePoint).
- core_fault_tests_003: 5/5 focused Core tests passed, actual native failure/gates/diagnostic bounds/observer lifecycle.
- host_fault_tests_001: no runtime assertion reached; incorrect assumption battle entry creates a SavePoint checkpoint.
- host_fault_tests_002: 1/1 actual actor integration passed after creating a genuine checkpoint through public admin SavePoint/CompleteNode. Tick, late calls, management, replacement, TTL, stopped writer, recreated owner Continue verified.
- core_fault_tests_004: 5 passed / 2 failures; missing-skill input exposed classification defect, postcommit probe hit precommit clone allowance.
- core_fault_tests_005: 6 passed / 1 test allowance failure (chosen values exceeded existing hard ceilings). Actual dynamic expression/manual fault and unchanged complete root passed; client-missing-skill now preserves original public envelope without fault. The ceilings remain unchanged; probe corrected to existing maxima.
- core_fault_tests_006 and host_fault_tests_003 are current pending focused runs. Receipts live under ../../evidence; every run includes exact source before/after, toolchain and raw log. No final baseline/audit verdict yet.


2026-10-04 추가 실행: `core_fault_tests_007`은 provenance Result의 unwrap_err가 Debug를
요구하는 시험 코드 compile 실패였다. `.err().expect`로 변경한 `core_fault_tests_008`은7/7 PASS:
실제 dynamic DivideByZero/manual, checked revision overflow, postcommit diagnostic cancellation,
일반 missing-skill 입력 wire 보존, fixed capsule, all public gates, observation first failure/lease.
`host_fault_tests_003`은3/3 PASS(two player, valid/missing/invalid Continue 포함).
`release_fault_build_001`은 production release host+public fault_runtime example 빌드 PASS.
`public_fault_small_ticks_001`은 external official content/seed23/100ms host tick62번째
실제 fact_summary_map_copy work1105>1000을 만들고 terminal 상태를 확인했다.
`public_fault_observed_001`은 실제 collector 선택→runtime fault→ended ring/selected cleanup,
owner lease 유지와 commits1 불변을 확인했다. `public_fault_none_001`, `detached_001`,
`queue_full_001`은 동일 public 결과; 포화 drop7(실제1024queue+정상authority 거절로 채움).
Websocket `public_fault_001`은 admin_result envelope 가정,002는 admin SavePoint가 새 map을
만든 후 오래된 node ID 재사용,003은 confirm entry가 normal state_snapshot 대신
battle_setup_snapshot을 보내는 기존 정상 순서를 driver가 놓친 실패였다. 원본은 모두 보존.
003 raw messages는 실제 scheduled tick work fault와 최소 fault state_snapshot 수신까지
확인하며 완전한 복구 PASS는 아니다. 제품 수정 없이 driver를 보정한004 실행 중.

`public_fault_004` PASS: release stock game_server/실제 /ready+RFC6455, native profile1000,
실제100ms 예약 tick에서 최초 fault,7종 player command+admin gate,detach/reconnect/replacement,
원 active/profile 전체 file hashes 보존,같은 LoadBalance의 명시적 Quit→new actor→기존 Continue,
loads_used불변.118 wire messages와 server log/config/summary 보존. No fault injection.
`fmt_apply_001`은 이번 변경11개 Rust파일만 포맷했으며 before/after와 변경목록 보존.
이 형식 변경 이후 final matrix와 release artifact 결속은 별도 수행한다.

`host_manual_fault_001`/002/003 were fixture-discovery failures, not terminal gate failures:
Retreat at max_work1000 succeeded; max_work100 failed earlier at entry; public observer measured
entry541work/0clone versus Retreat173work/85clone. A legal immutable native max_clone_slots84
therefore admits entry and fails the first actual manual Retreat. `host_manual_fault_004` PASS:
first typed clone limit, Core command cause, immediate host timer cancellation and late callback gate.
No error injection or change to production allowance defaults. Final source formatted after this probe;
`tools/matrix.py` starts source-bound full matrix,all inherited warnings retained.

final_fault_001 final builds/eight command receipts PASS; all five mode result payloads exact.
Coverage assembly first attempt stopped on an inherited observation row without suite_receipts (KeyError).
No coverage/result/target had been published. The mapper now preserves an explicit empty predecessor field
and binds current suites; no source change or execution retry is required.

최종 결과: GEX_10_RESULT.json. 모든 final_* matrix 및 compat_* receipt의 raw log/source identity를 검증했다.
final_fault_001_*은 최종 release source에서 native failure와 다섯 관측 모드 exact parity, 실제 stock websocket
예정 tick 오류→reconnect/replacement/save 보호→Quit/new owner/valid Continue를 검증했다.
R05 raw exit 101 / warning_concurrent_peak_only는 원본과 별도 분류를 유지한다.
새 독립 Complete/High 판정은 이 구현 결과에 포함하지 않는다.
