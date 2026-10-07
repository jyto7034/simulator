# 기존 코드 검토

2026-09-27 KST. 읽기 전용 조사이며 아래 미지원 항목은 구현 완료로 간주하지 않는다.

## 1. 현재 안전 한도는 서로 다른 범위를 가진다

| 소유자 / 코드 | 현재 의미 | 계획에 미치는 영향 |
| --- | --- | --- |
| [RuntimeFactLimits](../../../src/game/battle/runtime_control.rs) | native public operation의 work/clone, action/result/delivery/actor/provider 등 개수, retained admission. 기본 work 10,000,000·retained 8MiB. validate는 기본값을 ceiling으로 사용 | 단순한 “전투 전체 메모리”로 이름을 바꾸지 않는다. 초기 설정은 기존 기본값/ceiling 보존 |
| 같은 파일의 run/run_deferred | 중첩 호출과 rollback은 바깥 operation의 fuel과 첫 fault를 공유. scope 종료 시 해제 | 매 이벤트나 실패마다 카운터를 리셋하면 기존 보호가 약해짐 |
| seed_retained_extent/admit_retained_growth | live owner extent로 시작해 operation 안에서 양의 성장량을 누적 승인. 해제해도 같은 operation의 청구량을 환급하지 않음 | “현재 보유량”과 “operation admission 사용량”을 별도 메트릭으로 표시 |
| [retained_runtime_fact_extent](../../../src/game/battle/core/runtime_facts.rs) | facts/resources/carrier/queue 등의 계산된 extent. unit 고정 기여 등 보수적 계산 포함 | RSS/정확한 heap도 아니고 모든 전투 소유 공간을 포괄하는 장부도 아님 |
| [ExecutionLimits](../../../src/authoring/control.rs) | authoring attempt의 work·pending/bucket/effect·metadata/result·clone 제한. 기본값과 별도 validation ceiling | runtime과 같은 max_work 이름이어도 서로 다른 scope를 합치지 않음 |
| [TrialLimits](../../../src/authoring/trial.rs) | trial 전체의 max_total_work 기본 2^30, max_receipt_bytes 64MiB 및 trace 제한 | 도구 trial 전체 예산을 정상 플레이의 전투 수명 제한으로 오인하지 않음. 첫 설정 분리에서 기존 의미 유지 |
| [ExprLimits](../../../src/game/value_expr/mod.rs), [parser](../../../src/game/value_expr/parse.rs) | 평가와 컴파일 work 각 65,536, depth64/node1024 등. parser 별도 HARD_MAX 존재 | ExprLimits의 nonzero 검증만으로 parser 안전을 대체할 수 없음 |
| [compiled plan](../../../src/game/value_expr/compile.rs), [catalog](../../../src/game/value_expr/catalog.rs) | 계획이 evaluation limits를 보유하며 catalog 생성 때 limits 전달 | runtime만 바꾸면 기존 compiled limits와 어긋남. generation/admission 설정 결속 필요 |

앞선 대화에서 설명한 “메모리는 현재 보유량 기준”은 장기 설계 방향이다.
현재 구현의 cumulative admission을 즉시 live-only ledger로 바꾼다는 뜻이 아니다.
여러 장부를 합산해 프로세스 메모리로 표시하면 공유 Arc·복제·보수적 기여를 중복 계산할 수 있다.

## 2. 실패는 현재 스킬 취소를 보장하지 않는다

[runtime_transaction/runtime_timestamp_reusing](../../../src/game/battle/core/runtime_facts.rs)는 candidate를 처리하고
성공 경계에서 교체한다. [sim](../../../src/game/battle/core/sim.rs)의 same-time bucket은 동일 timestamp 이벤트와
재생성 이벤트를 처리하며, 오류는 `?`로 상위에 전파된다. public until/by 요청은 여러 timestamp를 처리할 수 있고
바깥 native operation budget을 공유한다. 더 바깥 authoring/application transaction 유무에 따라 최종 복원 범위도 달라진다.

따라서 budget fault 하나를 특정 스킬 탓으로 지정하거나, 해당 이벤트만 지우고 성공으로 반환하면 안 된다.
현재 검토한 경로에는 budget 부족을 `Pending`으로 돌려주는 보편적인 continuation API도 없다.
후속 조사에서 기절/퇴장 취소는 존재하지만 budget fault 복구용 범용 취소가 아님을 확인했다.
`runtime_actions_cancelled_control_waits_for_actual_finite_delivery_once`는 취소 뒤에도 delivery 종료를 기다린다.
`skill_runtime/cast.rs::cleanup_finished_skill_cast`의 정리도 work를 청구한다. 소진된 예산 안에서 취소 함수만 호출해
전투 계속을 보장할 수 없다. **사용자 요청으로 안전 한도 초과에 따른 스킬 자동 취소를 계획에서 제외**했다.
일반 부하 분할은 필요성이 확인됐을 때 별도 owner/commit/반복 방지 계약을 검토한다.

`with_runtime_fact_limits`는 actor/action이 없는 초기 core에서만 허용된다.
이미 만들어진 전투에 RON을 사후 적용하는 adapter만으로는 충분하지 않다. 실제 생성 전에 설정을 전달해야 한다.
파일 읽기를 BattleCore의 tick 함수에 넣지 않고 host construction 경계에서 해결한다.

## 3. 전투 로그는 이미 유한하게 보존된다

[BattleEventLog](../../../src/game/battle/event_log.rs)는 **8,192 events / compact JSON entry-array 4MiB / 전투 시간 180,000ms**
제한으로 오래된 entry를 축출한다. sequence/floor를 유지하며 너무 오래된 cursor에는 SnapshotResyncRequired를 반환한다.
전체 전투 결과·계속 실행 권한은 로그 밖에 있다. encoded bytes는 실제 heap bytes와 다른 지표다.

따라서 “긴 전투에서는 로그가 무조건 무한히 쌓인다”는 가정은 이 코드에 맞지 않는다.
전체 전투 로그 보존을 새 기본값으로 요구하지 않는다. 최근 기록 창과 결과·재접속의 기존 의미를 보존한다.
관측 이력은 외부 메트릭 저장을 활용한다. 전체 상세 로그 archive가 필요해질 때만 별도 bounded streaming을 설계한다.
이는 장시간 부하 시험 요구가 아니며, 사용자 요청으로 20분 검증은 전체 단계에서 제외됐다.

## 4. 관측 라이브러리와 연결 지점

[core manifest](../../../Cargo.toml)와 [server manifest](../../../../game_server/Cargo.toml)에 `tracing`이 있다.
조사한 manifest에는 `metrics`/Prometheus exporter가 아직 없다. [server main](../../../../game_server/src/main.rs)은
설정·로깅을 초기화한 뒤 game data를 로드하므로 runtime settings 검증/recorder 설치의 host 진입점 후보다.
독립 CLI·테스트 embedding은 서버 의존성 없이 validated settings를 전달할 수 있어야 한다.

- [`metrics` 공식 문서](https://docs.rs/metrics/latest/metrics/): counter/gauge/histogram API, recorder는 executable이 설치.
- [Prometheus exporter](https://docs.rs/metrics-exporter-prometheus/latest/metrics_exporter_prometheus/): 표준 노출 계층 재사용.
- [Prometheus 계측 지침](https://prometheus.io/docs/practices/instrumentation/): label 조합의 수와 계측 비용에 주의.
- [Grafana Prometheus datasource](https://grafana.com/docs/grafana/latest/datasources/prometheus/): 외부 그래프 재사용.
- [tracing-tracy](https://docs.rs/tracing-tracy/latest/tracing_tracy/): 선택적 상세 profiling. 기본 운영 메트릭과 구분.

2026-09-27 공식 문서를 확인했다. 실제 dependency version·MSRV·target 지원·lockfile 및 유지 비용은 S2 진입 시 검증한다.
관측 라이브러리가 전투별 메모리 소유자나 취소 규칙을 자동으로 알아내지는 않는다.

## 결론과 조사 한계

설정 분리는 재사용 가능한 기존 limit 구조가 있어 실행 가능하다. config loader만 추가하면 끝나는 변경은 아니다.
host→생성→compile→실행의 전달, embedded defaults, ceiling과 effective settings identity까지 설계해야 한다.
관측은 기존 usage/error/queue/log 경계에서 시작할 수 있다. 전체 heap 귀속과 새로운 분할 기능은 후속 후보다.
새로운 스킬 자동 취소는 범위에서 제거했다. 실험실 사전 스트레스 검증은 별도 제안으로 작성했다.
이번 검토는 static source 조사와 기존 자료 재사용이다. 새 성능 수치나 장기 시험 성공을 주장하지 않는다.
