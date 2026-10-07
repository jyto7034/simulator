# U02 코드 검토 진행 기록

Input: `WR-CRSO-002`, [source plan](P2G_20_SOURCE_PLAN_001.json). 아직 complete Review Batch가 아니다.
P2G-00/10은 완료, P2G-20은 조사 중이며 제품 수정은 없다. 이 문서는 조사 view다.

## 확인한 현재 동작과 필요한 변경

- `runtime_control.rs`의 `RuntimeFactLimits::default`가 숫자 원본이고 `validate`도 같은 default를 ceiling으로 쓴다.
  native `run/run_deferred`는 TLS allowance와 첫 fault를 중첩·candidate rollback에서 공유한다.
  RON default를 도입할 때 embedded parse와 `Default::default` 사이 재귀를 만들지 않도록 ceiling 검증을 독립시켜야 한다.
  실제 기존 숫자를 유지하고 admission 사용량을 live memory로 바꾸지 않는다.
- `authoring/control.rs`의 `ExecutionLimits`는 별도 숫자 default와 validation ceiling, attempt의 `ExecutionUsage`를 가진다.
  `TrialLimits`는 전체 work 2^30, receipt64MiB이고 total work validation ceiling은 2^38이다.
  동일 max_work 이름만 보고 native/attempt/trial scope를 합치지 않는다.
- `ExprParseLimits`는 HARD_MAX 및 TLS parser guard를 갖고, compile/evaluate의 `ExprLimits`는 nonzero 검증만 한다.
  compiler typed API의 기존 상한 없는 합법 범위를 parser HARD_MAX로 잘라서는 안 된다.
  `GameDataBase`의 실제 catalog 생성은 현재 `ExprLimits::default`를 고정 사용한다. 입력 parser만 바꾸면 불충분하다.
- 외부 content publication은 `content_pack::load → publish_snapshot → parsed.into_validated → GameDataBase`를 거친다.
  `ValidatedGameData`는 identity와 source receipt를 보유하지만 현재 safety profile identity가 없다.
  profile을 이 immutable generation과 compilation에 묶고 runtime/preview/explain 소비까지 따라가야 한다.
- `CoreApplication`은 공개 application owner, `world::GameCore`는 private다. 첫 전투 생성은
  `events/combat.rs`의 `BattleCore::new_from_database` 뒤 곧바로 `start_battle_execution_in_candidate`를 호출한다.
  profile은 이 start 이전에 들어가야 하며 public operation의 기존 deferred allowance를 보존해야 한다.
- `BattleEventLog`는 현재 events8192/encoded4MiB/span180000 상수를 사용한다. entries는 공개 wrapper이고 derive Deserialize다.
  profile별 창, import/deserialize 검증과 accounting 갱신 경계를 함께 다뤄야 한다. 진단 로그 창은 gameplay 수명이 아니다.
- author_content는 현재 옵션 수/위치가 고정되고 각 실행에서 AuthoringLimits/TrialLimits default를 만든다.
  선택 `--runtime-safety PATH`를 validate/describe/explain/simulate에 연결하면서 기존 positional·trace·amount 동작을 보존한다.

## 실제 host의 선행 불일치

[baseline 실행](evidence/server_before_check_001.json): `cargo check -p game_server --all-targets --offline --locked`
exit101, source 변경 없음. library16개, library test31개의 compiler 오류가 보고됐다.

서버는 private `GameCore`, `PlayerBehavior`, `BehaviorResult`, 삭제/비공개된 projection과 checkpoint 경로를 참조한다.
`GameDataBase::load_live_embedded`, `new_with_player_profile`도 현재 API와 맞지 않는다. 일부 fixture의 신규 필드가 빠졌다.
이는 U02 전 baseline이며 U01의 제품 교정 때문이라고 주장하지 않는다.

실제 host→Core 설정 전달을 시험하려면 이 adapter를 현재 공개 `CoreApplication`/`ValidatedGameData`로 연결해야 한다.
비공개 Core를 다시 노출하는 우회는 사용하지 않는다. 새 세션 fault 정책은 U03 소유로 유지하되
U02에서 기존 공개 typed 오류를 host가 받아 전달하는 최소 연결은 구현 범위에 포함한다.
기존 단일 플랫폼 선택 원칙에 따라 현재 저장소의 server host를 검증하고 Unity 구현은 포함하지 않는다.

## 관측 의존성 조사

현재 Core/server manifest에는 metrics/exporter가 없다. 이 환경에서 docker/podman/Prometheus/Grafana 실행 파일도
찾지 못했다. 로컬 재현을 위한 도구 확보 경로는 아직 확인 중이며 이 사실을 서비스 구현 불가능으로 단정하지 않는다.

2026-10-04 확인한 [metrics-exporter-prometheus 0.18.3 공식 API](https://docs.rs/metrics-exporter-prometheus/0.18.3/metrics_exporter_prometheus/struct.PrometheusBuilder.html)는
`upkeep_timeout`이 histogram maintenance를 수행하고 `idle_timeout`의 registry 제거는 render 요청에 의존한다고 명시한다.
따라서 idle timeout 하나로 collector 부재 시 bounded storage/종료 회수를 주장하지 않는다. 실제 고정 version의
source와 유한 bucket backend를 확인하고, Core label을 유한 집합으로 고정하며 종료 집계를 별도로 차감해야 한다.
wall time과 simulation time, extent/admission과 RSS는 구분한다.

## 다음 조사

실제 restore·preview/query·host command/result 변환, profile 전달에 필요한 data compiler 경계,
operation summary와 peak의 추가 순회 없는 수집 경계를 끝까지 확인한다. 정확한 reviewed source archive와
각 요소의 delta/검증 seam을 고정한 뒤에만 P2G-20 complete를 발행한다.
