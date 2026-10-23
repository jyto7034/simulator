# 실행 안전 설정·메트릭 계획

작성일: 2026-09-27 KST. 범위: 별도 후속 작업. R06 필수 수식 기능의 완료 범위에 추가하지 않는다.
근거: [검토](REVIEW.ko.md), [결정·인계](README.ko.md). 아래 S1–S4는 작업 묶음이며 새 formal Goal ID가 아니다.

후속 범위 확장에 따른 전체 배치는 [별도 로드맵 편입안](../../roadmaps/mod_validation_safety/ROADMAP.ko.md)을 따른다.
S1/S2는 MS01/MS02 설계 입력이며, S3는 MS03 시험 기반·MS05 승인 측정·MS06 통합 증거로 나뉜다.
S4는 필수 구현 순서에 포함되지 않는다. 기존 계획의 상세 수치는 구현 전 최신 source·host 경계와 다시 대조한다.

## 1. 목표와 보존 계약

긴 전투를 고려할 수 있는 측정 가능한 안전 정책을 만든다.
**20분 검증은 사용자 요청으로 전체 단계에서 제외한다.** 자동 검증, 실게임 등록, 구현/인수 완료 조건,
S3/S4 및 후속 작업에 포함하지 않는다. 기반이 마련된 뒤 사용자가 직접 수행하며 현재 선행 조건으로 두지 않는다.
계산 폭주·순간 메모리 증가·지속 누적을 구분하고, 정상 사용량의 평균만으로 순간 한도를 대체하지 않는다.
현재의 한도 검사·first fault latch·candidate rollback·공유 owner·로그 cursor를 유지한다.
관측이 꺼져 있거나 exporter가 실패해도 gameplay 결과·RNG·event 순서·안전 검사 결과는 같아야 한다.

최초 구현은 **기존 설정과 동작을 보존**한다. 전투당 3MB, 평균 기반 거부, 무제한 실행, 새 총 메모리 상한은 넣지 않는다.
모든 메모리를 합친 per-battle 실측 cap과 host 전체 admission은 현재 계측 범위를 확장한 뒤 결정한다.

## 2. S1 — 기존 한도의 trusted host RON 분리

### 형식과 소유권

후보 설치 경로는 `config/runtime_safety.ron`. 콘텐츠 pack 안에 넣지 않는다.
host가 선택한 경로 또는 배포 기본 파일에서 읽고, validated immutable settings를 core에 전달한다.
임의 콘텐츠 RON·scenario가 자기 한도를 바꾸지 못한다. core는 파일 시스템이나 환경변수를 직접 읽지 않는다.

다음은 **schema 개요**이며 아직 parser가 지원하는 실행 파일이 아니다. 생략한 필드는 표와 기존 타입의 전부를 포함한다.

```text
RuntimeSafetySettingsV1 {
  schema_version: 1,
  native_runtime: RuntimeFactLimitsDto,
  authoring_execution: ExecutionLimitsDto,
  expression: { parse: ExprParseLimitsDto, compile_and_evaluate: ExprLimitsDto },
  authoring_trial: { max_total_work, max_receipt_bytes },
  battle_history: { max_events, max_encoded_entry_array_bytes, max_span_ms }
}
```

이름 없는 배열 대신 named field를 사용한다. RON DTO의 byte/count 값은 유한 정수이며
`usize` 변환 시 target 폭·checked 범위를 검사한다. 시간 단위는 `_ms`, 공간 단위는 `_bytes`로 명시한다.
기존 `native_runtime.max_retained_bytes` 이름은 DTO 설명에서 **operation admission/extent bound**로 정의한다.
새 `max_battle_memory_bytes`라는 이름으로 같은 값을 포장하지 않는다.

### 최초 기본값·상한·scope

| 그룹 | 파일에 옮길 값과 최초 기본값 | 적용/초기화 범위 |
| --- | --- | --- |
| native runtime | work10,000,000; clone slots1,048,576; active actions4096; retained results4096; deliveries8192; actors4096; providers16384; control steps4096; strike segments8192; predeath buffs4096; retained bytes8,388,608 | 기존 public operation 및 state owner. 중첩/rollback은 allowance 공유 |
| authoring execution | work10,000,000; pending8192; bucket4096; effect commands8192; geometry65536; retained entities4096; definitions16384; metadata/result 각8,388,608 bytes; clone slots65536 | 기존 authoring attempt |
| expression parse | input bytes65536; depth64; nodes1024; authored members4096 | 실제 deserialize 이전/도중. 기존 HARD_MAX 보존 |
| expression compile/eval | depth64; nodes1024; stack1024; dynamic members4096; groups4096; eval work65536; compile work65536; retained bytes1,048,576 | compiler/evaluator/plan의 기존 scope와 cost 의미 |
| authoring trial | total work1,073,741,824; receipt bytes67,108,864 | 도구 trial 전체. live battle 수명 한도로 사용하지 않음 |
| battle history | events8192; encoded bytes4,194,304; span180,000ms | 최근 diagnostic history 창. 전투의 총 실행 시간 제한이 아님 |

공간 획득·scenario·trace 등 다른 limit family까지 전부 이관하지 않는다. 기존 값과 검증을 유지하고
adapter가 겹치는 한도를 함께 적용한다는 사실을 문서화한다. `TraceOptions` 등은 기존 owner가 계속 가진다.
새 limits family가 필요하면 별도 scope로 귀속한다.

운영 기본값과 엔진이 지원하는 안전 ceiling을 분리한다. S1에서 native ceiling, authoring validation ceiling,
parser HARD_MAX를 높이지 않는다. 기본값은 RON을 단일 원본으로 삼고, `Default`와 no-file embedding도 동일한
검증된 embedded RON에서 도출해 두 숫자 원본의 drift를 막는다. 파싱은 첫 사용/host bootstrap 때 한 번 수행한다.
기존 typed API가 허용하던 ExprLimits의 범위까지 우연히 축소하지 않는다. compiler와 parser의 상한 관계를 따로 검증한다.

### 파일 오류·적용·identity

- 설정 경로를 지정하지 않으면 embedded 기본 RON을 사용한다. 명시한 파일이 없거나 잘못되면 초기화 실패이며 조용한 fallback은 없다.
- 파일 크기 상한 64KiB를 loader에 고정한다. 단순 고정 DTO·입력 depth 제한·unknown/duplicate field 거부로 설정 parser 자체도 제한한다.
  설정 파일이 자기 parser 제한을 완화하지 못한다. 이 크기는 게임 메모리 cap이 아니다.
- schema mismatch, 음수·overflow·기존에 불법인 0·ceiling 초과는 경로와 field를 포함한 설정 오류다.
- raw file SHA와 canonical effective settings SHA를 별도 기록한다. 값이 같고 주석/공백만 다른 파일은 effective SHA가 같다.
- 새로운 host/session/generation을 생성할 때 고정한다. active battle에 hot reload하지 않는다.
- expression 설정은 parse→compile→catalog→plan→runtime/preview/explain까지 같은 validated profile을 사용한다.
  cache/admission key에 profile identity를 결속한다. gameplay 식 의미가 같다는 이유로 다른 admission 설정의 plan을 무검증 재사용하지 않는다.
- `with_runtime_fact_limits`의 초기화 제한을 존중한다. scenario 생성/첫 spawn 전에 설정이 들어가도록 constructor 경로를 연결한다.
- restore는 host가 설정한 profile을 새 실행에 적용하고 active state가 맞는지 검증한다. save가 임의 한도를 승인하지 않는다.
  재현 실행은 profile hash를 비교하며 불일치를 명시한다. 기존 save schema를 무조건 변경하는 것을 S1의 목표로 삼지 않는다.
- history 설정은 기존 floor_seq/eviction/resync/결과 통계를 보존한다. 직접 생성/deserialize/import 경로도 bounded policy를 거치게 한다.

### S1 완료 기준

실제 server bootstrap과 authoring CLI의 명시적 설정 입력 경로, library embedding을 검증한다.
CLI에는 선택적 `--runtime-safety <PATH>`를 추가하는 방향으로 기존 positional 계약을 보존한다.
기본 RON·기존 default의 결과/오류 parity, 작은 유효 한도에서 같은 typed fault,
설정 오류 시 콘텐츠 publication/전투 시작 없음, nested budget 유지, 두 profile의 격리,
history cursor/eviction과 expression parse/compile/runtime identity를 시험한다.
읽기 성공만으로 완료하지 않는다. S1에서 metrics dependency·처리 분할·행동 취소를 함께 구현하지 않는다.

## 3. S2 — 표준 메트릭과 외부 그래프

core의 내부 검사 장부 → 작은 operation 관측 요약 → `metrics` → host recorder/exporter → Prometheus → Grafana.
허용·거부는 내부 정수 장부가 수행한다. 지연되거나 float로 표현된 exporter 값을 다시 안전 판정에 사용하지 않는다.

`metrics`는 core의 선택적 계측 feature로, exporter/HTTP/실행 task는 host 쪽에서 소유한다.
Prometheus/Grafana 및 datasource/dashboard 설정은 개발용 관측 구성으로 제공하고 게임 실행의 필수 서비스로 만들지 않는다.
기존 tracing은 오류 상세/선택한 전투의 action identity에 활용한다. Tracy는 S2 필수 의존성이 아니다.
재사용 근거·공식 문서는 [검토 §4](REVIEW.ko.md)에 연결했다. 버전/MSRV/platform/라이선스와 정확한 histogram backend 동작을
확인해 lockfile을 고정한다. 범용 exporter·시계열 저장소·차트 엔진을 자체 구현하지 않는다.

### 첫 metric 계약

| 제안 이름 | 종류 / 의미 |
| --- | --- |
| `battle_runtime_accounted_extent_bytes` | gauge: 기존 owner extent 계산값. 실제 heap/RSS라는 이름 사용 금지 |
| `battle_operation_admitted_bytes` | histogram: operation 종료 시 admission 사용량. 해제 후 감소하는 live gauge와 구분 |
| `battle_operation_work` / `battle_operation_duration_seconds` | histogram: 실제 시도한 작업량 / wall time. outcome 구분 |
| `battle_pending_events`, `battle_active_actions`, `battle_active_projectiles`, `battle_units` | gauge: commit된 상태의 현재 개수 |
| `battle_safety_rejections_total` | counter: 외부에 반환된 fault당 1회. 내부 poll마다 중복 증가 금지 |
| `battle_history_events`, `battle_history_encoded_bytes`, `battle_history_span_seconds` | gauge: 기존 bounded history 지표 |
| `battle_observation_dropped_total` | counter: 상세 관측 buffer 포화/손실. 게임 이벤트 손실과 별개 |
| `battle_selected_simulation_time_seconds` | gauge: 선택 전투의 simulation time. wall-clock 관측 시각과 함께 표시 |

평균/최대/분포와 시간 추세를 보여준다. simulation time과 wall time을 구분해 표시한다.
기존 extent 함수를 scrape마다 호출하면 비용 청구/오류를 유발할 수 있다. 정상 검사 중 이미 계산한 값이나
부작용 없는 bounded observation seam을 사용하며, 새 전체 owner 순회를 exporter가 유발하지 않는다.
프로세스 RSS를 추가한다면 host 지표로 따로 표시하고 전투별 계측치와 합치거나 나누어 정확한 소유량이라고 주장하지 않는다.

### 수집 주기·bounded storage

관측 설정은 별도 `observability.ron`으로 두며 gameplay settings hash를 바꾸지 않는다.
최초 개발 profile: local snapshot200ms, Prometheus scrape1s, dashboard refresh1s, 상세 선택 전투1개,
선택 전투 최근 ring300개. 이 숫자는 조정 가능한 관측 설정이며 게임 한도가 아니다.
매 snapshot의 값만으로 사라지는 순간 급증은 interval high-water/counter/histogram으로 보완한다.
진단 ring은 60초 창이고 그보다 긴 관측 이력은 Prometheus가 보관한다. 기본 local 보존은24시간으로 두고 개발용 저장량을 관리한다.
이 보존 설정은 같은 시간 동안의 부하 시험 의무를 뜻하지 않는다.

label은 `outcome`, 제한된 `reason`/`operation` 등 유한 집합으로 고정한다. UUID/action/event/player ID를
모든 time series의 label로 쓰지 않는다. 전체 집계(sum/max/분포)를 기본 제공하고 상세 선택은 고정 slot0 하나로 둔다.
slot의 실제 battle identity와 선택 변경 시각은 상세 기록에 두고 그래프에는 선택 경계를 명시한다.
종료된 전투의 gauge는 registry에서 제거/집계 차감해 값이 영원히 남지 않게 한다.

metric handle은 hot path 진입 전에 등록한다. allocator hook 내부에서 metric 등록/문자열 생성/출력을 하지 않는다.
fixed operation summary의 시도/실패 통계는 rollback 후에도 남기되, commit 수/상태 gauge는 commit에만 반영한다.
동적 registry·histogram sample buffer·선택 상세 ring은 장기 실행에서도 상한/회수 조건을 갖춰야 한다.
exporter의 maintenance가 scrape가 없을 때도 버퍼를 정리하는지 실제 버전으로 검증한다.
HTTP 노출은 host가 opt-in하고 개발 기본은 loopback이다. exporter 장애는 관측 오류로 보고하고 gameplay thread를 막지 않는다.

### S2 완료 기준

사용량·거부 원인이 외부 Grafana에 표시되고 dashboard/config가 저장소에서 재현 가능해야 한다.
관측 off/on·collector 미접속·scrape 중단·상세 buffer 포화에서 게임 결과/오류 동일성과 관측 storage 회수를 검증한다.
전체 집계·선택 전투·종료 제거·이중 counting 방지·짧은 peak 포착을 확인한다.
게임 내부 그래프는 S2 이후 이 관측 계약을 소비하는 별도 UI 작업이다.

## 4. S3 — 실제 콘텐츠 측정과 한도 조정

S3의 실측·조정 원칙은 이후 기능 개발에도 적용한다. S1의 기존 값 보존은 최초 설정 분리 작업의 범위이며 영구적인 한도 변경 금지가 아니다.
각 기능 단계는 [Core-first 로드맵 §6.4](../../roadmaps/core_first/ROADMAP.ko.md#64-예산안전성과-증거-재사용)에 따라 필요한 한도 재검토를 진행할 수 있다.
오프라인 검증·실시간 전투·개발 측정 profile을 구분하며, 유한한 측정용 상한을 쓰더라도 기존 profile의 실패와 동일 입력의 비교 결과를 보존한다.

일반 전투, 동시 효과가 집중되는 전투, 스킬 기능별 스트레스 조건,
의도적인 깊은 수식/연쇄/이벤트·메모리 압박 입력을 준비한다.
단순히 시간을 경과시킨 것으로 실제 시전·효과 경로의 coverage를 대신하지 않는다.
최대 지원 규모는 실제 콘텐츠/지원 환경을 기준으로 fixture에 명시하고, 대표 콘텐츠가 아직 없으면 synthetic 측정임을 표시한다.
기존64전투 fixture는 비교 관측으로 유지한다. 이를 모든 게임 콘텐츠의 최대 부하라고 주장하지 않는다.

source/config/input/seed/toolchain·simulation/wall time·관측 설정을 함께 기록한다.
현재/최대 extent, admission/누적 allocation, log encoded bytes, 프로세스 메모리를 구분하고
일반 평균·분포·최댓값·후반 증가 추세·종료 후 회수를 본다. 관측 off/on 비용도 짝지어 비교한다.
exporter의 저장 상한·유휴 cleanup은 경계/수명 시험으로 확인한다. 특정 장시간 실행을 필수 검증으로 추가하지 않는다.

기본값을 올리거나 낮출 때는 대상 profile·근거·실패 영향·여유를 명시한다. 새 메모리 한도를 정하려면
어떤 owner가 포함/제외되는지부터 확정한다. 평균만으로 안전 거부를 결정하지 않는다.
새 값이 기존 engineering ceiling을 넘으면 설정 파일 편집만으로 우회하지 않고 해당 ceiling의 근거와 구현 영향을 함께 재검토한다.
신뢰할 표본이 없는데 임의 p99 또는 고정 퍼센트 증가를 새로운 차단선으로 만들지 않는다.

## 5. S4 — 필요성이 확인된 처리 분할·총 메모리 정책의 후속 후보

S1/S2에서 이 기능을 구현했다고 주장하지 않는다. 관측 결과로 필요성이 확인될 때 상세 설계를 검토한다.
사용자의 “일단 스킬 취소는 제거하자”에 따라 **안전 한도 초과 시 스킬 자동 취소·환불·후속 delivery 일괄 종료는 범위에서 제거**한다.
기존 기절/퇴장에 의한 게임 규칙상 취소는 변경하지 않는다.

| 상황 | 채택한 방향 / 추가 계약 |
| --- | --- |
| 정상 작업이 이번 host 처리량에 몰림 | soft scheduling quota를 초과하면 continuation. hard safety fault를 삼켜 재시도하는 구현 금지 |
| 한 스킬의 유한성/한도 위반 | 우선 사전 validation/실험실 시험에서 발견. 실행 중 발생하면 기존 typed error와 복원 경계를 유지하며 자동 스킬 취소로 변환하지 않음 |
| host 전체 압박 | 새 전투/작업 admission과 backpressure를 host가 관리. 임의 스킬 취소로 가장하지 않음 |
| 내부 불변식 오류/복원 실패 | 명시적 전투 오류. 정상 진행처럼 표시하거나 부분 결과를 성공으로 반환하지 않음 |

분할은 deterministic event 순서·timestamp·RNG·동일시각 refill·boss/wave boundary·완료 판정을 보존해야 한다.
먼저 완전한 timestamp 사이의 양보를 검토한다. 하나의 timestamp가 과대한 경우에는 checkpoint 가능한
동일시각 continuation이 별도로 필요하며 부분 상태를 gameplay commit으로 공개하지 않는다.
기존 operation fuel을 API 호출 분할로 무한히 초기화하지 않도록 논리 작업/원인 chain의 hard allowance를 이어간다.
메트릭의 wall time이 gameplay 성공 여부를 바꾸지 않게 한다.

실험실에서 500/1,000 유닛 등 스트레스 검증을 통과한 콘텐츠만 실게임에 허용하는 방향은
[모드 적용 정책과 설계 제안](LAB_STRESS_PROPOSAL.ko.md)으로 연결한다. 실패 모드는 본편에 적용하지 않으며,
검증은 core 실행 주체가 수행하고 저사양 예외는 두지 않는다. 시험 profile·성능 수치·상세 적용 절차는 미확정, 제품 구현은 미실행이다.
사전 시험 통과를 근거로 기존 runtime 검사·오류 복원을 제거하지 않는다. 동일 실패의 무한 자동 재시도도 허용하지 않는다.

## 6. 검증·실행 경계·순서

| 작업 | 필수 검증 | 종료/인계 |
| --- | --- | --- |
| S1 | 위 config 경계 + 기존 제품 회귀, 변경 host/CLI 시험, format/check/clippy. 기본값 parity와 한도 경계 | 설정 분리 결과·before/after·실행 증거 동결 후 독립 검토 |
| S2 | observer 정확성·boundedness·실제 Grafana·off/on parity·부하 비용 | 관측 기능 결과와 dashboard 인계 |
| S3 | 대표/집중/기능별 스트레스/폭주 표본과 회수·추세 | 설정 조정 제안 및 근거. 측정 전 수치 확정 금지 |
| S4 후보 | 필요성 검토 후 continuation/동일결과·host 자원 관리 | 별도 설계. 스킬 자동 취소 제외, S1로 묶지 않음 |

새 source 기준으로 기존 test selectors를 확인하고 필요한 전체 `game_core --all-targets` 및 변경된 host 검증을 수행한다.
실패를 숨기거나 과거 성공을 새 source 직접 실행처럼 표시하지 않는다. R06 PC36 결과·D04-F01 기능은 보호한다.
성능은 [개정 정책](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/R06_COST_POLICY_UPDATE_20260927.ko.md)을 적용한다.
raw guard의 exit와 정책 판정을 따로 기록하며 검증 스크립트의 blanket `|| true`로 대체하지 않는다.

이 계획은 작성·자체 문서 검증을 마친 상태다. 제품 테스트, 실제 S1–S4 구현, 설치·배포, 별도 독립 설계 검토는 미실행이다.
동시에 같은 runtime/data 파일을 수정하는 R06 구현과 S1 writer를 실행하지 않는다.
R06 남은 기능을 진행하는 데 이 전체 작업의 완료를 요구하지 않으며, 기존 frozen R06 산출물은 원문으로 보존한다.
