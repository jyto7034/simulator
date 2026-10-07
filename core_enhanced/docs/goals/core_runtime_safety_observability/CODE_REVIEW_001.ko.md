# U02 코드 기반 설계 인계

입력 WR-CRSO-002, 범위 PE-CRSO-01..12. 이 문서는 구현 결과가 아니라 P2G-20 설계 검토다.
현재 source는 REVIEW_SOURCE_001.tar.gz와 SOURCE_INDEX_001.json으로 고정한다.
정책 목표와 수락 기준의 소유자는 P2G_10_POLICY_001.json의 각 r2다. 아래 검증은 그 의미를 바꾸지 않는다.

## Current Actual

RQ01: native RuntimeFactLimits는 숫자 default를 validation ceiling에도 사용한다. authoring ExecutionLimits,
ExprParseLimits HARD_MAX, ExprLimits, TrialLimits, BattleEventLog는 별개 scope다. ExprLimits typed API는
nonzero 검증만 하므로 parser ceiling을 compiler에 복사하면 기존 합법 입력을 거절한다.
RQ02: CoreApplication은 유일한 공개 실행 owner다. GameCore와 도메인 command/result는 private다.
실제 server는 옛 private API를 참조하여 baseline check가 실패한다(evidence/server_before_check_001.json).
world/application.rs의 clone→operation→reconcile→검증→PreparedApplicationCommit이 원자 경계다.
events/combat.rs는 BattleCore 생성 직후 최초 실행을 시작한다. active-run은 V9 checkpoint를 검증하고
node_flow 복원 후 실행하며 저장된 battle을 그대로 deserialize하는 경로가 아니다.
server는 persistence 실패 때 non-battle run 상태를 되돌리고 별도 영구 profile과 run peak를 보존하려 한다.
옛 capture/restore는 현재 cfg(test)이며 그대로 공개하면 검증 없는 rollback 권한을 추가하게 된다.
RQ03: content_pack load→snapshot→publish_snapshot→into_validated→GameDataBase에서 expression catalog가
현재 default로 컴파일된다. plan은 ExprLimits를 소유하며 semantic fingerprint는 식 의미를 해시한다.
authoring query는 generation을 읽고 amount_query는 보관된 exact plan/frame을 재평가한다.
따라서 profile identity는 semantic fingerprint와 분리하고 동일 generation/plan을 끝까지 사용해야 한다.
RQ04: history는 8192 events/4MiB/180000ms 창이며 accounting, floor_seq와 cursor resync를 갖는다.
derive Deserialize와 from_entries도 존재하므로 append 경로만 바꾸는 것은 불충분하다.
RQ05: native/authoring TLS는 nested allowance와 first fault를 공유한다. retained extent는 현재 안전 검사에서
이미 계산된다. rollback되는 draft 안에 관측 counter를 두거나 exporter가 extent를 재계산하면 계약이 깨진다.
RQ06: 현재 metrics 의존성은 없다. 고정 검토 버전 exporter0.18.3의 histogram record는 AtomicBucket에
샘플을 쌓고 run_upkeep이 정리한다. idle_timeout의 series 제거는 render에 의존하므로 그것만으로
collector 부재 시 boundedness를 보장하지 못한다. 공식 crate 원본/해시는 evidence에 보관했다.
RQ07: U01은 별도 독립 감사 Complete/High로 종료됐고 55조건/R06/R07 및 current CLI 증거가 있다.
R05 concurrent peak +343587bytes 경고는 해소됐다고 주장하지 않는다. 기존 frozen 원본은 보존한다.

## Proposed Implementation

### TD01 — S1 설정 소유권과 parser

Core의 새 runtime_safety 모듈은 파일/환경을 읽지 않고 &[u8]에서 validated immutable profile을 만든다.
DTO는 deny_unknown_fields, 필수 named fields, schema_version, u64 정수와 checked usize 변환을 사용한다.
64KiB bytes 및 고정 nesting guard는 DTO deserialize보다 먼저 적용한다. unknown/duplicate/negative,
overflow/zero/ceiling 오류는 source path와 field를 포함한다. host loader는 최대64KiB+1만 읽는다.
명시 경로 실패는 publication 이전 실패이며 fallback하지 않는다. no-file은 include_bytes!와 OnceLock으로
같은 RON을 한 번 검증한다. 기존 Default 구현은 이 profile의 필드를 사용한다. validation ceiling은
별도 engine 상수로 유지하여 parse→Default→parse 재귀를 막는다. typed ExprLimits 범위는 유지한다.
raw bytes SHA와 canonical DTO effective SHA는 별도다. profile 값은 private이고 생성 뒤 수정할 수 없다.

### TD02 — generation, application와 authoring 연결

ValidatedGameData/ContentGeneration에 Arc profile과 admission identity를 결속한다. 기본 loader는 embedded
profile을 사용하고 explicit-profile variant는 parse guard→catalog compile에 같은 값을 전달한다.
content semantic identity와 expression semantic fingerprint는 기존 의미를 유지한다. admission identity는
semantic identity와 effective profile hash를 함께 사용하며 profile이 다른 generation/plan을 재사용하지 않는다.
GameDataBase/CompiledAmountPlan의 실제 평가 한도를 연결하고 explain/preview도 해당 plan을 읽는다.
공개 CoreApplication은 validated generation의 profile을 사용한다. BattleCore constructor가 profile의 native와
history를 적용한 뒤 scenario/첫 spawn을 실행한다. 기존 초기화 후 with_runtime_fact_limits로 덮어쓰지 않는다.
public operation의 deferred/nested allowance는 같은 profile을 적용하며 예외/rollback에서도 리셋하지 않는다.
checkpoint는 현재 host profile 아래 resource/state 검증을 거친 뒤 commit한다. save bytes에서 settings를
승인하지 않고 원본 bytes를 수정하지 않는다. no-file 기존 API는 source-compatible wrapper로 남긴다.
authoring explicit profile API/CLI는 execution, parser, compile, native, trial, history를 같은 profile로 연결한다.
기존 InputLimits/TraceOptions 등 별도 family는 보존하고 교차 제한을 문서화한다. --runtime-safety 옵션을
공통 파싱하여 모든 기존 CLI command와 trace/amount 옵션 조합에서 positional 의미를 유지한다.
validation/trial receipt에는 effective profile identity를 포함해 replay mismatch를 명시적으로 비교할 수 있게 한다.

### TD03 — 실제 server adapter 전환

bootstrap에서 settings 검증 후 generation을 만들고 LoadBalanceActor→PlayerGameActor에 전달한다.
CoreApplication command/context/result/snapshot/checkpoint/error를 host DTO에 연결한다. private GameCore나
BehaviorResult를 다시 export하지 않는다. transport JSON 계약·request ID·battle authority·event cursor를 유지한다.
기존 checkpoint persistence 원자성을 위해 공개 opaque non-battle transaction capability를 추가하거나 동등한
prepared-application 경계를 사용한다. capability는 해당 application/정확한 revision에 결속하고 외부에서
구성·다른 session에 적용·두 번 소비할 수 없어야 한다. stale restore는 typed rejection이며 active battle의
first fault/allowance 우회 수단으로 사용하지 않는다. 복원은 기존 monotonic profile/run peak를 보존한다.
구체 타입/함수명은 GEX 상세 설계지만 위 소유권·원자성 검증은 S1 필수다. 옛 fixture도 공개 경계로 이관한다.
이 변경은 실제 host 설정 경로의 선행 수리다. U03의 새로운 세션 fault 정책을 이 단계에서 정의하지 않는다.

### TD04 — bounded history

history owner에 validated policy를 저장한다. 기본 생성/explicit 생성/from_entries/serde import가 accounting과
동일 eviction 규칙을 거친다. untrusted deserialize가 serialized limits로 ceiling을 완화하지 못하게 한다.
application import에서는 host policy로 재검증·재제한한다. floor_seq는 삭제 범위를 반영하고 result 통계는
별도 owner에 유지한다. span이 지났다는 이유로 전투를 종료하지 않는다. public mutable wrapper의 accounting
무효화 경계도 검사하여 serialize/query 직전 숫자가 오래된 상태를 성공으로 내보내지 않는다.

### TD05 — S2 operation 요약과 commit 관측

S1 감사 후 선택 feature에서만 fixed-size operation summary를 만든다. 기존 charge/admit와 extent 계산의
상수 비용 수치 seam을 활용하며 관측을 위해 owner 전체를 추가 순회하지 않는다. 최상위 operation guard가
nested 호출을 통합하고 성공/실패별 work/admitted/wall-time/returned-fault를 한 번 확정한다.
시도 통계와 interval peak는 draft rollback 밖에 둔다. commit된 gauge는 PreparedApplicationCommit이 실제
publish된 때만 갱신한다. 읽기 전용 query/scrape는 safety charge/오류/상태를 유발하지 않는다.
종료/drop에는 전투 집계를 차감하고 선택을 해제한다. aggregate sum/max 유지용 관측 owner는 활성 session
수에 대응하며 session 종료 때 제거된다. per-operation history를 무제한 보관하지 않는다.

### TD06 — host exporter와 bounded 저장

metrics feature, recorder/exporter/task/HTTP는 선택 사항이며 bootstrap에서 유한 label/handle을 미리 등록한다.
Core→host fixed-size summary queue는 고정 capacity와 try-send/drop 정책을 사용한다. host worker는 한 번에
고정 batch만 metrics histogram에 기록하고 즉시 PrometheusHandle::run_upkeep을 호출한다. worker 정지 시
queue 포화 뒤 observation만 drop된다. 직접 무제한 histogram record hot path는 사용하지 않는다.
classic finite buckets, 유한 operation/reason/outcome 집합으로 series 수를 제한한다. scrape가 없어도 worker가
유지보수를 호출한다. 상세 선택은 slot0 하나, ring300개이며 실제 identity/선택 시각은 ring metadata에 둔다.
관측 설정은 별도 RON이며 gameplay profile hash에 포함하지 않는다. snapshot200ms/scrape1s/dashboard1s,
개발 Prometheus24h 기본을 제공한다. 종료/선택 전환은 aggregate 차감과 ring 경계 표시를 수행한다.
HTTP는 opt-in loopback이다. exporter 실패는 host 관측 오류로 남기고 게임 thread의 blocking/결과 변경이 없다.
Prometheus/Grafana 표준 provisioning과 dashboard에 sum/max/distribution/rejection/선택 경계 및 두 시간축을
표시한다. 환경에 컨테이너가 없으면 로컬 공식 binary로 실제 datasource/dashboard 동작을 검증한다.

### TD07 — 전환, 대안과 GEX 소유 상세

환경변수/전역 mutable limits는 profile 격리를 깨므로 채택하지 않는다. settings를 content semantic SHA에 합치는
대안은 의미 identity를 변경하므로 분리한다. poll마다 metrics emit, scrape마다 extent 순회, idle timeout만으로
정리하는 대안은 이중 counting·비용·unbounded storage 때문에 채택하지 않는다. 자체 exporter/차트는 만들지 않는다.
S1과 S2는 독립 감사로 분리한다. S1에는 metrics dependency를 넣지 않는다. S2에서 S1 source currentness와
실제 exporter dependency graph를 재확인하고 정확한 patch version을 lockfile에 고정한다.
queue capacity/batch 크기·세부 함수명·dashboard panel layout은 의미를 바꾸지 않는 GEX 선택이다. 값과 상한
증명·시험은 구현 기록에 남긴다. 실제 frontend 접근 방식은 환경에 맞게 선택하되 Grafana 표시 검증은 생략하지 않는다.

## Validation Seams

V01(S1): baseline/default RON/no-file API·CLI·host effective identity 및 결과/오류 parity, 모든 원래 숫자 비교.
V02(S1): 64KiB/depth/unknown/duplicate/version/negative/overflow/zero/ceiling 경계, invalid 명시 경로와 publication 부재.
V03(S1): 서로 다른 두 profile 동시 load/compile/실행·query 격리, raw/effective SHA, mismatch와 immutable generation.
V04(S1): public host 첫 spawn 작은 한도 typed fault; nested/rollback allowance 공유; restore와 save 원본 보존.
V05(S1): CLI 전 command·옵션/API parity, existing session 실패 reload 유지, trial work/receipt scope.
V06(S1): expression parse/compile/runtime/explain/preview 경계, semantic fingerprint 유지, 합법 큰 typed ExprLimits.
V07(S1): history 생성/import/deserialize/eviction/floor/resync·통계, 창보다 긴 전투, 제한 초과 입력 bounded admission.
V08(S1): server build·실제 bootstrap/command/authority/reconnect/저장 실패 복원·profile monotonic 회귀.
V09(S2): 짧은 peak/실패1회/commit-only gauge/종료 차감/선택 변경/중복 소비·drop 정확성.
V10(S2): feature off/on·collector 부재·scrape 중단·worker stall/queue full parity와 bounded memory/histogram 회수.
V11(S2): 실제 Prometheus scrape와 Grafana datasource/dashboard 응답에 사용량·거부 원인·시간 구분 표시.
V12(각 Phase): 전체 core default/test-fixtures와 변경 server check/test/format/clippy, U01 공개5사례·45CLI와
R06/R07 영향 selectors; PC36/R05 current measurement 또는 exact-source 재사용 근거. raw exit와 정책 판정 분리.
관측 비용은 짝지어 측정하지만 새 임의 성능 차단값·20분 시험은 없다.

## Findings, Unknowns and Disposition

F01 existing-implementation-gap: server baseline compiler failure. TD03/V08이 S1 owner; 성공한 host로 가장하지 않는다.
F02 implementation-delta: default/constructor/history/compile 연결 부족. TD01–04/V01–08에서 구현·검증한다.
F03 technical-risk: exporter histogram 임시 샘플 보관. TD06/V10에서 bounded proof와 실제 시험으로 닫는다.
F04 preexisting-warning: R05 peak 경고. V12에서 raw와 정책 판정 분리 유지; 새 cap/임의 waiver 없음.
새 material_policy finding 없음. deployed/remote release 호환성은 승인 local 범위 밖이며 그런 PASS를 주장하지 않는다.
필수 source evidence gap 없음. 구현 성공·실제 Grafana 실행·성능 수치는 아직 unknown이며 GEX 결과 owner다.
