# Rust 테스트 병렬 실행 사전 분류

지속적인 분류 관리의 진입점은 [유지·등록·실행 묶음 절차](MAINTENANCE.ko.md)다.
사람이 읽는 [분류 기준표와 등록표](CLASSIFICATION.ko.md)는 `policy.json`과 검토된 `inventory.csv`에서 생성한다.
아래 숫자는 기존 정적 스냅샷의 수치이며, 작업 중 추가된 테스트의 현재 개수는 검사 출력으로 확인한다.

후속 [실제 Core 128개 실행 비교](runs/pilot_20261008T113448_229873Z/REPORT.ko.md)에서 개별·묶음 순차·4스레드 병렬을 각 3회 확인했다. 모두 통과했고 중앙값은 18.414초 / 16.900초 / 4.848초였다. 특정 기존 기본 구성 바이너리의 표본 결과이며 아래 정적 분류 전체를 병렬 승인한 것은 아니다.

후속 [Core 전체 테스트 확인](runs/core_all_20261008T114330_485301Z/REPORT.ko.md)에서는 갱신된 고정 소스의 선언 2,045개를 전부 실행해 **2,035개 통과·10개 실패·누락 0개**를 확인했다. 기본·전체 기능 설정에서 같은 실패가 순차 재실행으로 재현됐고, 통합 테스트 73개는 모두 통과했다. 아래 내용은 최초 정적 분류 당시의 범위와 기록이며 실제 실행 결과는 후속 보고서를 따른다.

2026년 10월 8일 기준으로 `game_core`와 `game_server`의 테스트를 정적으로 분류했다. 이 자료는 진행 중인 실행이 끝난 뒤 병렬 검증을 설계할 때 사용한다. **실제 병렬 실행과 실행 설정 적용은 하지 않았다.** `parallel_candidate`도 아직 병렬 검증을 통과했다는 뜻은 아니다.

## 분류 결과

저장한 스냅샷은 소스 파일 225개의 테스트 선언 2,164개를 포함한다. Core 2,039개, Server 125개다. `cfg`, feature, 플랫폼 조건 및 `include!` 확장 전의 소스 선언 수이므로 특정 명령에서 실행되는 테스트 수와 다르다. 기존의 약 1,800개 실행 목록과 직접 비교하면 안 된다.

| 분류 | 선언 수 | 실행 조건 |
| --- | ---: | --- |
| `parallel_candidate` | 1,432 | 공통 입력을 고정하고 직렬 기준 결과와 병렬 결과를 비교할 후보 |
| `resource_serial` | 691 | 같은 자원 키를 가진 테스트끼리 직렬화하며, 관련 없는 후보와는 병행 가능 |
| `isolated_process` | 4 | 새 테스트 프로세스에서 해당 테스트 하나만 선택하고 파일 자원 제약도 준수 |
| `quiet_host` | 37 | 실제 타이머를 쓰는 Actor 테스트의 첫 검증은 다른 빌드·테스트 부하 없이 수행 |

691개와 37개에는 보수적으로 묶은 모듈·파일 범위가 포함된다. 모두가 본질적으로 직렬 실행을 요구한다는 판정은 아니다. 첫 병렬 검증에서 이들을 제외해 나머지 후보부터 검증할 수 있게 한 분류다. 전체 테스트 호출 그래프나 외부 라이브러리 내부의 전역 상태를 형식적으로 증명한 결과도 아니다.

## 결과 파일

| 파일 | 용도 |
| --- | --- |
| [inventory.csv](inventory.csv) | 선언별 위치·함수명·대상 힌트·feature 힌트·실행 분류·자원 키·규칙·소스 해시 |
| [policy.json](policy.json) | 기본 규칙과 예외, 충돌 자원의 적용 범위, 환경 변수 조건 |
| [source_snapshot.json](source_snapshot.json) | 분류 당시 소스·Cargo 설정 해시와 선언 수 |
| [check_inventory.py](check_inventory.py) | 빌드나 테스트 없이 목록·규칙·소스 변경을 확인하는 검사 도구 |

CSV의 `test_name`은 소스 함수명이다. Rust 테스트 하네스의 완전한 `module::test` 선택자가 아니다. `target_hint`와 `feature_hints`도 실행 명령을 자동 생성하기 위한 확정 계약이 아니다. 특히 부모 모듈의 조건과 여러 대상에 포함되는 파일은 실행 목록으로 대조해야 한다.

## 확인한 공유 자원

### 전투 진단 파일

World의 전투 종료는 테스트·디버그 구성에서 `core_enhanced/target/combat_traces/run_<seed>/<battle_uuid>.json`을 쓴다. battle UUID도 시드와 카운터에서 결정되므로 서로 다른 테스트가 같은 경로를 사용할 수 있다. 고정 경로에 파일을 만들거나 삭제하는 실패 주입 테스트도 있다.

근거: [전투 종료의 export 호출](../../../src/game/world/combat.rs), [경로와 File::create](../../../src/game/world/combat_trace.rs), [결정적 UUID 생성](../../../src/game/managers/uuid_manager.rs), [실패 경로 조작 테스트](../../../src/game/world/tests/combat.rs)의 `combat_trace_export_failure_does_not_block_result_transition`.

또한 `battle_showcase`와 `skill_test_suite`의 Nothing There 테스트는 `run_abnormality_scenario`를 통해 같은 이름의 이벤트 로그를 쓴다. 이 호출은 [공통 실행 도우미](../../../tests/skill_test/common/run.rs)와 [파일 저장 도우미](../../../tests/common/mod.rs)에 있다. BattleCore의 `advance_basic_attack_projectile_is_idempotent_for_same_projectile_id`도 고정 이름의 로그를 쓴다.

이 경로에는 `checkout:core-debug-artifacts`를 적용했다. World·Application·Trial 및 서버 fixture 경계는 호출 도우미가 쓰는 경로까지 보수적으로 포함한다. **프로세스나 `CARGO_TARGET_DIR`만 분리해도 이 파일 경로는 분리되지 않는다.** 추후 이 범위를 줄이려면 실제 파일 접근 경로나 테스트별 출력 분리가 근거가 되어야 한다.

### 프로세스 공통 관측 상태

[관측 테스트](../../../../game_server/src/observability/tests.rs)는 각자 recorder를 생성하지만, [RetirementGate](../../../../game_server/src/observability/mod.rs)는 `crossbeam_epoch::pin()`의 기본 collector를 사용한다. 한 테스트의 pinned guard가 다른 테스트의 회수 시점에 영향을 줄 수 있으므로 `process:crossbeam-default-collector` 그룹으로 분류했다. 별도 프로세스에서는 collector가 분리된다.

다음 4개는 `isolated_process`다.

| 파일 | 테스트 |
| --- | --- |
| `game_server/src/observability/tests.rs` | `exporter_heap_is_bounded_without_scrapes_and_retires_on_idle` |
| `game_server/src/runtime_measurement/playback.rs` | `cmrp_playback_normal_external_combat_modes_keep_typed_results_and_gameplay` |
| 같은 파일 | `cmrp_playback_actual_queue_full_and_absent_modes_preserve_final_state` |
| 같은 파일 | `cmrp_playback_maximum_sessions_repeats_keep_footer_and_exact_truncation` |

첫 번째는 프로세스 전체 live bytes의 일치를 검사하며 원래부터 `ignore`와 단독 실행 지시가 있다. 나머지는 [playback execute](../../../../game_server/src/runtime_measurement/playback.rs)가 하나의 프로세스 측정 창을 열고 heap·RSS를 기록한다. 게임 결과 검사가 통과하더라도 동시 실행으로 오염된 계측값까지 유효해지는 것은 아니다. 성능 수치 비교를 할 때는 별도 프로세스 외에 호스트 부하도 통제해야 한다.

하네스 밖의 `measure_runtime` 실행 파일과 `observe_runtime`, `fault_runtime` 예제는 `policy.json`의 `non_harness_jobs`에 별도 구간으로 기록했다. 위 선언 수에는 포함하지 않는다.

### 타이머와 임시 경로

서버 Actor의 `handlers.rs`, `handlers/tests/fault.rs`, `handlers/tests/durable.rs`에는 실제 2~120ms sleep 후 상태를 확인하는 흐름이 있다. 첫 비교에서는 이 세 파일의 37개 선언을 `quiet_host`로 보수적으로 묶었다. 이후 실제 시간 민감도 근거가 생기면 해당 테스트만 남기도록 줄일 수 있다. 단순히 시뮬레이션 시간이나 `delta_ms`를 다루는 Core 테스트는 이 분류에 넣지 않았다.

`ExternalFixture`, snapshot/packaging fixture와 서버 저장소 fixture는 UUID 임시 디렉터리 또는 `tempfile::TempDir`를 사용한다. 파일 I/O 자체를 직렬화 사유로 삼지 않았다. `env.rs`의 고정 임시 경로 예제는 경로를 계산할 뿐 실제 파일을 생성하지 않는다. 다만 fixture에서 World를 호출하는 경우에는 별도의 진단 파일 제약을 유지한다.

fresh-process/crash 테스트는 자식 명령에만 환경 변수를 전달하고, UUID/TempDir 또는 테스트별 접두사와 PID를 쓴다. 자식 프로세스가 있다는 이유만으로 전체 테스트 묶음을 직렬화할 필요는 없다. 기존 자식의 정확한 테스트 선택과 crash 단계는 유지해야 한다.

### 병렬 후보로 유지한 메모리 검사

[runtime_control](../../../src/game/battle/runtime_control.rs)의 예산·admission 기록, [authoring control](../../../src/authoring/control.rs), [표현식 파서](../../../src/game/value_expr/parse.rs), [유물 조건 파서](../../../src/game/relic/structural/parse.rs)는 스레드별 상태와 scope 정리 또는 이전 값 복원을 사용한다. 이 상태만으로 전체 native budget 테스트를 직렬화하지 않는다. World 경로를 쓰는 테스트에는 별도의 파일 제약이 적용된다.

서버 [allocator 단위 테스트](../../../../game_server/src/runtime_measurement/allocator.rs)는 테스트별 ledger를 만들고, [RSS 테스트](../../../../game_server/src/runtime_measurement/rss.rs)는 파싱 및 상태 종류를 검사한다. 이 9개는 프로세스 전체 메모리 한도를 재는 테스트와 구분하여 병렬 후보로 유지했다. 불변 데이터의 `OnceLock` 캐시 역시 존재 자체를 직렬화 사유로 삼지 않았다.

## 실행 전 검사

저장소 루트에서 다음 명령은 소스를 읽기만 하며 Cargo·Rust 테스트 프로세스를 시작하지 않는다.

```sh
python3 core_enhanced/docs/testing/parallel_preflight/check_inventory.py
```

검사는 선언의 중복·누락, 비어 있는 규칙, 알 수 없는 자원 키, CSV 변조, 새 파일·삭제·소스 및 Cargo 설정 변경을 감지한다. `ok: true`는 저장한 정적 분류와 현재 입력이 일치한다는 의미이며 병렬 실행 통과를 뜻하지 않는다. 정책의 중첩은 허용하며 더 강한 실행 분류와 모든 자원 키를 함께 적용한다.

변경이 감지되면 출력되는 파일부터 분류 영향만 검토한다. 검토 후 `--write`와 변경 경로별 `--reviewed-source`로 스냅샷을 갱신한다. 정책 변경에는 `--reviewed-policy`도 필요하다. [정확한 명령과 유지 절차](MAINTENANCE.ko.md)를 따른다. **검사 실패를 없애기 위해 변경 검토 없이 재생성하지 않는다.** 현재 다른 담당자의 구현·테스트 추가가 계속되면 이후 검사에서 변경이 보고되는 것이 정상이다.

소스 스캐너는 주석·문자열을 제외한 `#[test]`, `#[tokio::test]`, `#[actix_rt::test]`, `#[actix_web::test]` 선언을 센다. Rust 매크로 확장, doctest, 도구 폴더의 별도 probe 크레이트, 과거 실행 증거 속 복사본은 이 목록의 범위 밖이다. 런타임 하네스에만 나타나는 항목은 나중에 미분류로 처리해야 하며 기본 병렬 허용으로 흘려보내면 안 된다.

## 현재 실행 이후의 적용 순서

아래는 후속 실행 담당자를 위한 절차이며 이번 작업에서 실행하지 않았다.

1. 현재 실행이 끝난 경계에서 소스 검사 결과를 확인하고, 필요하면 변경된 파일의 분류만 갱신한다. 소스·게임 데이터·feature·환경을 비교 동안 고정한다.
2. 기존 검증 범위에 맞춰 Core 기본 lib, `test-fixtures` 통합 테스트, `runtime-observation`, `event_log_checks`, Server 기본·관측·측정 구성을 각각 구분한다. 모든 조합을 무조건 추가하지 말고 기존 요구 구성을 보존한다. 각 구성의 하네스 목록과 CSV를 대조하여 완전한 이름·ignore·플랫폼 조건을 확정한다. 매핑 누락과 중복은 실행 전에 실패시킨다.
3. 같은 소스·구성의 유효한 직렬 기준 결과가 있으면 재사용한다. 부족한 범위만 직렬로 확보한다. 테스트 본문 출력은 순서가 섞일 수 있으므로 하네스의 완전한 이름별 상태를 비교한다.
4. `parallel_candidate`의 검증된 선택 목록을 한 테스트 바이너리 안에서 묶어 실행한다. 테스트마다 Cargo를 한 번씩 시작하면 컴파일 재사용만으로는 호출 비용이 줄지 않는다. 하네스 thread 수는 우선 4로 시작하고 결과·소요 시간·메모리 여유가 확인될 때 8 이상을 검토한다. Cargo의 빌드 job 수와 테스트 thread 수는 별도 설정이다.
5. `resource_serial`은 모든 자원 키의 충돌 범위를 지키고, `isolated_process`는 정확히 하나의 테스트만 선택한다. `quiet_host` 및 성능 측정은 별도 구간에서 진행한다. 하네스 자체가 이 JSON의 자원 키를 자동 집행하지는 않으므로, 실행 담당자가 선택 목록과 실행 순서로 보장해야 한다.
6. 실패·누락·새 ignore 없이 기준과 일치하는 범위만 병렬 실행 검증 완료로 승격한다. 시간 단축은 실제 측정 후 기록하며, 분류표만으로 속도 향상을 단정하지 않는다.

이 작업은 기존 러너·CI·제품 코드·테스트 코드·Cargo 설정·현재 실행의 기록을 수정하지 않았다. 새 분류 자료와 읽기 전용 검사 도구만 추가했다.
