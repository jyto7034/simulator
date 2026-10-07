# S2 구현·자체 검증 결과

선택적 runtime observation과 표준 Prometheus/Grafana 경로를 구현했다. Core의 상수 크기 attempt 수집은 실제 native charge/admission/extent·삽입 지점에서 사용량과 순간 peak를 관측한다. 최종 반환 오류는 한 번만 기록한다. rollback·준비 객체 폐기에도 시도량을 남기며 committed gauge는 실제 root 공개에서만 전진한다. 공개 preparation이 반환되기 전에 TLS scope를 닫아 다른 application의 호출과 섞이지 않는다. 게임플레이 제한·오류·RNG·저장 정책은 변경하지 않았다.

Host는 유한 try-send queue·사전 등록 series·고정 histogram bucket·유한 worker batch와 즉시 upkeep을 소유한다. 선택 battle 한 개의 slot0과 300행 ring을 제공하며 session/battle identity를 series label로 만들지 않는다. 종료/drop cleanup은 summary queue가 가득 차도 전달된다. 별도 observability.ron과 feature를 사용하고 설정 오류·수집기 단절·queue overflow·exporter bind 실패·epoch 지연은 실제 게임플레이 결과를 바꾸지 않았다. 표준 dependency의 원문·버전·기능 그래프와 저장량 유도는 DEPENDENCY_REVIEW.ko.md 및 METRICS_AND_STORAGE.md에 고정했다. allocator 실측은 fixture bucket을 쓰는 실제 host worker/recorder 경로에 한정하며 process RSS로 일반화하지 않는다.

실제 release public driver 35회(7모드×5회)에서 모든 결과·오류·event·최종 Run/reconnect·retreat projection hash가 같았다. 각 실행은 일반 1100×10ms, 집중 2000ms, pause/resume, authority/native refusal과 종료를 포함한다. queue-full은 각 84 summaries를 잃었지만 게임플레이는 같았다. cost의 raw timing과 중앙값은 COST_AND_PARITY.md에 남겼다. feature-enabled refusal의 측정 지연도 보존하며 무비용·새 latency gate를 주장하지 않는다. release 증거 뒤의 유일한 제품 변경은 release에 포함되지 않는 cfg(test) 순간 projectile peak 검증 추가이며 재사용 근거는 COVERAGE.json에 명시했다.

실제 Prometheus 3.15.0과 Grafana OSS 13.2.3을 실행했다. live006은 4197 samples, 2개 실제 session, 정확히 한 번씩의 두 거절 이유, 선택 1→2→1, retreat/drop 뒤 active/selected 0을 확인했다. render007은 같은 실제 wall interval로 duration p50/p95 panel11 및 simulation time XY panel19를 브라우저에 그렸다. SERVICE_VERIFICATION.json은 251개 실제 datasource frame을 검사한다. 초기 XY No data와 provision/migration 문제, 실패·중단 receipt 및 재시작은 EXPERIMENT_NOTES.md에 보존했다. 현재 서비스 helper는 종료됐다. 원격 배포나 Unity 검증을 주장하지 않는다.

현재 전체 suite는 Core default1701·fixtures1701·observation1707, host default77·observation84 통과다. host observation의 ignored allocator test1은 별도 실행하여 통과했다. 1024512 raw samples 후 idle739308bytes, pinned754060→754060으로 추가 축적 없이 gate/upkeep을 검증했다. fmt/check/event-log/Clippy/release도 통과했다. 기존 Clippy 경고와 명시적으로 허용한 기존 lint는 원본대로 남겼다.

S1 F02의 실제 host 저장 실패를 두 번 연속 주입하여 disk/root 불변과 각 discarded attempt의 사용량·peak, commit/current 불변, 가짜 오류 부재를 확인했다. 재시도는 peak204→213을 영속화했다. 공개 prepared boundary8, 일반 boundary13, API21commands, profile24commands/6실제 websocket host, public oracle5·trace5·CLI를 현재 source에서 실행했다. 잘못된 observability 설정에서도 실제 command/reconnect와 safety identity가 유지됐다. PC36의 576 rows는 즉시 선행 S1 F02와 timing을 제외한 모든 값·allocation·native usage가 같았다.

R05 raw exit는101이고 분류는 기존 승인된 warning_concurrent_peak_only다. concurrent peak9373578bytes는 기존9030188 cap을343390 초과하며 다른 hard guard failures는[]다. event-heavy의 원래 allocation guard는 통과한다. cap이나 latency policy를 바꾸지 않았고 이 경고를 raw 성공으로 표현하지 않는다. COVERAGE.json은 5개 S2 acceptance·V01–12·10개 필수 차원과 88 inherited conditions를 34개 source-bound receipts 및 실제 selectors에 연결한다.

이 기록은 GEX-10 자체 검증이며 독립 완료 판정이 아니다. target/s2_001의 정확한 forward/reverse 복원과 기록 membership를 동결한 뒤, 구현 비참여 GPT-6 Astra/High reviewer가 정책 적합성·증거 현재성·선행 applicability·실제 경로를 독립 평가한다. Complete/High 이후 별도 Master GEX-30을 진행한다.
