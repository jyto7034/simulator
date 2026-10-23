# Core·서버 전체 테스트 실행

2026-10-15 사용자 전체테스트1회 실행 요청. 현재 Core/서버 대화범위의 Rust workspace4package가 대상이다. 제품/시험을 수정하지 않고 현재 실패를 수집한다. 실패가있어도 다른target은 --no-fail-fast로 계속한다. 이전교정 메시지9pass만 전체PASS로 확대하지 않는다.

실행범위: (1) cargo test --workspace --all-targets --offline --locked --no-fail-fast, (2) --workspace --doc --offline --locked --no-fail-fast, (3) game_core/game_server의 runtime-measurement 기능구성으로 lib/integration/bin 전체검사. 기본구성과 계측구성을 분리하며 invalid_* compile-fail probe와 재설계대기 event_log_checks, 전용test-fixtures opt-in을 all-features로 자동활성화하지 않는다. ignore는실행PASS로세지않고이유를기록한다. 별도콘텐츠조립기/Windows/Unity는현재Rust검사범위밖이다.

환경: rustc/cargo1.98.1, CARGO_HOME=/tmp/wp02-cargo-home, CARGO_TARGET_DIR=.cache/u164-review-dev-target, dev/test debug0, offline/locked. 기존빌드cache재사용. 기본Core실행약4분+통합/서버와빌드예상합계5–12분, 추가기능빌드2–8분추정. 첫60초로그진행확인,15분초과시동작/로그/자원진단. 출력없음만으로중단하지않는다. 로컬socket/IO를실제검사하므로sandbox권한실패는제품실패와구분한다.

완료후종료코드·실제test수·실패명·ignore·환경실패를설정별로집계한다. 이번요청은실행/보고이며새로발견한제품/fixture를실행중수정하지않는다.


## 기본 실행 종료와 추가 검사 범위 조정

기본 workspace 전체는 중단 없이 1916.06초에 exit101로 종료했다. Core 2228pass/0fail/6ignore, 서버lib72pass/26fail/22ignore, 독립probe 통합11pass다. 서버lib 자체는1522.22초. 26실패 중21개는 handlers.rs:1746의 공통 Defense 준비 helper가 seed0..1000을 모두 소진한 panic이다. 나머지5개는 프로필 persistence 미설정, 시작/교체 계약 불일치2종(3시험), 프로필 버전 기대4/실제6이다. 제품 결함26개라는 뜻이 아니며 해당 목표 연산은 미검증이다.

15분·20분·26분·29분 진단에서 서버는 약21core·RSS4.6GB로 실행 중이었다. 5초 perf 샘플은 /tmp/full-tests-20261015-perf.data에 남겼으며 무출력만으로 중단하지 않았다. 시간은 성능 합격 판정이 아니다.

추가 runtime-measurement 구성은 기본 전체를 다시 반복하지 않고 --all-targets 빌드와 측정/관측 모듈 및 관련Core feature 전용 테스트를 선택한다. 기본 실패의 공통 준비를 사용하는 서버 feature 전용 2시험(runtime_observation_real_battle_preparation_failure_commit_and_end_are_distinct, u04_prepared_terminal_tick_observes_discard_and_commit_without_premature_end)은 이번 추가 실행에서 미선택으로 명시한다. 전체feature 구성이 green이라고 주장하지 않는다. 기본 전체26실패가 남은 상태이며 새코드/시험수정은없다. exporter 전용 heap 검사는 빌드가 되면 요구된 단독조건으로 실행한다.


## 최종 결과

모든 소유 실행은 종료했다. 이번 요청은 검사와 결과 보고로 마쳤으며 제품·테스트·콘텐츠는 수정하지 않았다. 전체 통과 상태가 아니다.

| 구성 | 통과 | 실패 | 제외 | 종료 코드 / 실제 시간 |
| --- | ---: | ---: | ---: | --- |
| 기본 workspace 전체 | 2311 | 26 | 28 | 101 / 1916.06초 |
| 문서 테스트 | 0 | 0 | 0 | 0 / 68.25초, 등록된 문서 시험 없음 |
| runtime-measurement 선택 검사 | 43 | 3 | 1 | 101 / 654.95초 |
| exporter heap 단독 검사 | 1 | 0 | 0 | 0 / 시험 7.83초 |
| Core outer observation 추가 검사 | 1 | 0 | 0 | 0 |

기본 구성은 Core 단위2156+통합72=2228통과/실패0/제외6, 서버lib72통과/26실패/22제외, 독립probe 통합11통과다. 실행된60target 중 시험0개인 target을 기능 통과 수로 세지 않았다. 앞서 교정한 deserializes_snake_case_node_flow_requests는 이번 전체 실행에서도 통과했다.

추가 측정 구성의 선택 고유48시험은 **45통과/3실패**다. main run의 heap ignore1은 이후 요구된 단독 실행으로 통과했으며, 처음 선택 필터에서 빠진 Core feature 시험 cmrp_raw_outer_nested_default_core_keeps_unknown_warning도 같은 빌드 실행 파일에서 추가 확인했다. 기본 구성과의 합산을 전체 기능 수로 주장하지 않는다. 전체 계측 구성 회귀와 위 서버 feature 전용2시험은 미실행이다.

### 기본 실패 원인과 한계

- **21개:** handlers.rs:1746에서 공통 Defense 준비 helper의 seed0..1000 탐색이 소진됐다. 목표 전투·복원·취소 연산을 검증하기 전에 실패했으므로 그 연산의 결함21개로 해석하지 않는다. 현재 경로에서 목표 전투에 도달하는 준비 전략의 교정이 필요하다.
- **1개:** corrupt_local_active_run_requires_explicit_reset — fixture에 profile persistence가 없어 paired reset이 player_profile_persistence_unavailable로 거절됐다(handlers.rs:2871).
- **2개:** application_preintent_write_failure_keeps_core_and_profile_unchanged / accepted_application_profile_write_failure_commits_core_and_requires_recovery — Tutorial 시작 요청이 현재 상태에서 invalid_action으로 먼저 거절되어 의도한 저장 장애 검증에 도달하지 못했다(handlers.rs:2529,2616).
- **1개:** local_active_run_is_projected_and_requires_explicit_new_run_replacement — 기대 active_run_replacement_confirmation_required와 실제 active_run_requires_continue_and_abandon이 불일치했다(handlers.rs:2795).
- **1개:** u12_profile_store_current_checksum_positive_then_duplicate_raw_vault_key_rejects — 정상 저장 직후 버전 기대4/실제6에서 실패했다(normal_host_tests.rs:142). 현행 PROFILE_VERSION은6이다. 뒤의 raw duplicate 변조 단계는 미도달이며, 그 변조 전용 동작을 새 제품 finding으로 올리지 않는다.

이는 관측된 실패 지점의 분류이며 제품의 정확성을 새로 보증하는 근본원인 감사가 아니다. 새 Goal이나 제품 완료 차단으로 자동 승격하지 않았고 교정은 수행하지 않았다.

### 추가 측정 실패 3개

- terminal_observation_keeps_failure_work_and_lease_without_a_fake_commit_or_repeat: fault_tests.rs:96, 전투 진입 battle_entry_snapshot_owner의 work requested1084 > fixture limit1000으로 목표 실패 관측 전에 거절됐다.
- cmrp_script_preserved_u07_actor_trap_wall_reach_normal_public_combat: playback.rs:1088, 과거 pack의 catalog_schema5/execution2가 현재6/3 계약과 맞지 않아 로드가 거절됐다.
- cmrp_playback_normal_external_combat_modes_keep_typed_results_and_gameplay: playback.rs:1018, 제거된 Standard 모드 역직렬화가 거절됐다(현재 Tutorial/Expedition). 앞선 교정 때 별도 마이그레이션으로 기록했던 측정기 경로의 실제 실패다.

[결과·재현 명령](RESULT.json), [기본 원시 로그](default.log), [기본 실패26개 원문](failures.json), [target별 집계](default_targets.json), [기본 ignore28개와 사유](ignored.json), [문서 로그](doc.log), [추가 측정 로그](measurement.log), [추가 실패3개 원문](measurement_failures.json), [단독 heap 로그](exporter_heap.log), [추가 Core 관측 로그](observation_outer.log).

U16-4 감사004의 과거 고정 판정과 증거는 변경하지 않았다. 이 전체검사의 실패는 별도 현재 회귀 결과로 남기며, 과거 제한된 감사 통과를 전체 workspace green으로 확대하지 않는다. 후속 제품 구현이나 실패 교정은 이번 요청에서 수행하지 않았다.
