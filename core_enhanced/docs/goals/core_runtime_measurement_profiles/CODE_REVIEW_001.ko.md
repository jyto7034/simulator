# U07-M 실제 코드 검토와 구현 설계

기준은 `WR-CMRP-002`의10정책, `RSP-CMRP-002`의 required6/N·A2, [81개 실제 소스](evidence/REVIEW_SOURCE_MANIFEST_002.json)다. source plan001은 보존하고 밖에서 호출되는 resource clone helper를002에 추가했다. 계획자와 지정 Sol/xHigh 구현자가 읽기 전용 조사했다. 제품 변경·빌드·시험은 수행하지 않았다. 아래는 필요한 owner와 소비 경계를 추적한 정적 검토이며81파일 전체 줄 검사를 주장하지 않는다. U07 product `886d427882319b3147260f394f1204acf903af654f772709bdaaa045f1c62146`의 독립 Complete/High/open0를 선행 근거로 인수한다.

## Current Actual

| Requirement | 실제 owner와 처리 흐름 | 부족분과 변경 경계 |
| --- | --- | --- |
| RQ01 | `runtime_safety.rs::from_ron`은64KiB/depth16/strict DTO와 engine ceiling을 검사하고 raw/effective SHA256을 만든다. `ValidatedGameData`와 `GameDataBase`가 immutable Arc 설정을 소유한다. bootstrap이 trusted 파일을 읽으며 콘텐츠/저장이 profile을 선택하지 않는다. query/trial은 identity를 출력한다. | 별도 개발 mode가 없고 native ceiling도8MiB다. `verify_effective_sha256`은 기존 production caller가 없어 출력 hash만으로 replay mismatch를 막았다고 할 수 없다. optional 명시 profile과 실제 replay 소비 gate를 추가한다. |
| RQ02 | `runtime_control::run/Scope/DeferredScope`가 TLS Budget을 소유하고 중첩은 재사용한다. `seed_retained_extent`는 검사 전에 초기값을 기록한다. growth는 양수만 누적하며 rollback/해제도 환급하지 않는다. `BattleCore`는 settings에서 `RuntimeFactState.limits`를 만든다. | 기존 공개 limits layout/엄격 검증을 보존하면서 trusted entry에서만 개발 ceiling을 허용한다. sim6개 entry, runtime transaction2개, World active/pending/deferred가 모두 필요하다. facts345/583/1734/2600, World resource122/147·node_flow1549의 직접 retained 검사도 같은 effective owner cap이어야 한다. expression scratch는 별개다. |
| RQ03 | `OperationSummary`는 Copy이고 queue1024/batch128이다. scope 종료에서 work/clone/admission을 전송한다. extent scan 성공 뒤 hook이 있다. host collector는 try-lock 등록, weak session, 유한 labels, selected ring300, drop counters를 가진다. | admitted 기존 필드는 순수 accepted가 아니며 요청 최대/first site/실제 heap/RSS가 없다. 살아 있는 registry의 절대 상한·operation raw ring·유한 export를 보완한다. 기존 public allocator의 TRACK 창은 창 이전 allocation의 해제를 정확히 셀 수 없어 연속 play에 그대로 복사하지 않는다. |
| RQ04 | 정상 `CoreApplication` command/admin/tick/prepare/commit이 observation과 연결된다. 실제 host `load_balance_actor/handlers`가 application 생성 직후 attach한다. U04 Continue/SavePoint는 현재 host의 데이터/설정으로 pre-entry를 재검증한다. | 측정 실행 manifest를 공개 application API와 연결하고 profile mismatch를 실행 전에 거절한다. 현재-host restore를 저장 profile과의 무조건 mismatch 거절로 바꾸지 않는다. live clone/profile isolation·U03 terminal fault와 원본 보호를 보존한다. |
| RQ05 | normal external pack loader, `consume_content_pack`, 공개 `live_run_with_checkpoint`, U07 normal object fixtures, runtime/host observation/fault/heap tests가 재사용 seam이다. PC576과 R05는 별도 기존 numeric oracle다. | 새 finite measurement binary와 실제 host 표본은 아직 없다. focused profile/native/observer 경계부터 검증하고 default PC를 먼저 확인한 뒤 필요한 최종 통합을 한다. development 비교는 원래 default 결과와 분리한다. |
| RQ06 | U07 target002와 audit002가181선행조건(161+20), 별도31R06의 근거다. 등록 FOLLOWUP은 U07-M 완료 후 U08, 최종값은 U30-C/P로 지정했다. | U07-M10조건·10차원과 선행 적용성/현재 증거를 결속한다. 과거 target/실패/oracle는 수정하지 않는다. 독립 감사 후 현황을 게시한다. |
| RQ07/08 | 외부 Released/Deployed Actual 요구는 없다. local release 및 실제 host는 위 required 소스에 포함했다. | N/A는 Unity/원격 배포 결과를 주장하지 않는다는 뜻이다. 현재 정상 Core/host 표본 생략 사유가 아니다. |

## TD01 — trusted profile와 유한 cap

확정 정책은 기본 배포값/거절/identity 보존, trusted 명시 선택, warning과 hard admission 분리다. 권장 DTO는 기존 schema1에 optional `development_measurement: Option<DevelopmentMeasurementDto>`를 추가하고 None은 canonical serialization에서 생략한다. `revision: 1`과 `emergency_retained_bytes`만 허용하고 unknown/duplicate/zero/overflow/미지원 revision을 strict 거절한다. 원래 native DTO는 기존 validate로 검사한다. warning은 그 profile의 기존 `native_runtime.max_retained_bytes`에서 얻어 별도 임의 상승 필드를 두지 않는다. 개발 emergency는 warning보다 커야 하고 임시 engine ceiling32MiB 이하로 제한한다. embedded 배포 RON의 바이트는 그대로 보존한다. 개발 예제는 별도 trusted 파일이며 content pack에 포함하지 않는다.

32MiB는 현재8MiB admission의 두 배/네 배인16/32MiB까지 비교하는 유한 실험 창이다. 단위는 operation 시작 extent+양의 allocation growth이며 RSS/heap의 측정값에서 유도한 값이 아니다. 지원 플랫폼/정상 최대 gameplay를 결정하지 않고 배포 권장값으로 주장하지 않는다. 기존32bit usize 범위와 checked arithmetic 안에서 유한 테스트·출력으로 관리한다. 이 범위를 넘는 후속 실험은 새 profile revision과 이유/전후 측정이 필요하다. user 위임 근거는 FOLLOWUP4.1이 임시 ceiling/API를 코드 조사 후 결정하도록 명시하고 연속 요청이 내부 설계를 위임한 것이다.

권장 representation은 `RuntimeFactLimits`11필드와 `ENGINE_MAX`8MiB/공개 `validate()`를 그대로 두고 `RuntimeSafetySettings`가 nominal limit·개발 설정을 소유하는 것이다. `native_runtime()`는 선택된 effective retained cap을 담은 limits를 반환한다. default는 동일하고 개발 생성 owner는 selected cap을 저장한다. private `run_with_profile(limits, settings, closure)`만 retained validation을 trusted selected ceiling으로 수행한다. 나머지10개 필드는 원래 validate와 같은 검사를 한다. 공개 `with_runtime_fact_limits`는8MiB strict이고 명시 낮은 owner cap을 그대로 저장하므로 개발 프로필에서도 그 override를 높이지 않는다. 설정은 이미 공유되는 GameData Arc에서 접근하며 per-BattleCore policy 복사/새 설정 Arc를 만들지 않는다.

## TD02 — 모든 operation/owner와 fault 정합

`runtime_transaction`, `runtime_timestamp_transaction`, sim의6개 run과 World `prepare_application_internal`을 selected profile entry로 전환한다. 최초 battle 이전 World deferred가 예산 lifetime을 소유하며 battle/pending owner의 limits가 활성화할 때도 같은 immutable settings를 참조한다. nested 호출은 새 profile/cap으로 기존 Budget을 교체하거나 validate해 확대하지 않는다. 이미 발생한 sticky fault를 먼저 poll한다. public low-limit overrides·pending transfer·retained result/owner direct check·buffer capacity 선택까지 현재 effective owner cap을 인수한다. 모든 `RetainedByteLimit`/`max_retained_bytes` callsite를 스캔해 authoring/expression와 native 범위를 구분한다.

경고는 실제 hard fault 이전 독립 기록이다. 기존 admission을 실패시킨 뒤 fault를 지우거나 예외로 삼지 않는다. 기본 success/typed code/site/cap과 원래 누적 계량을 보존한다. start extent+growth 누적, 후보 실패/해제/rollback 비환급, 다음 public operation 새 seed는 바꾸지 않는다. emergency failure는 기존 candidate rollback 및 U03 terminal session/save 보호로 전달한다. 설정 invalid는 session 시작 전 거절하고 fallback하지 않는다.

추가 outside-scope 경계: 공개 `BattleCore::runtime_fact_snapshot`은 native scope 없이도 `admit_retained_growth`를 호출하므로 현재 inactive fallback8MiB가 개발 owner32MiB의 projection을 재거절할 수 있다. private helper에 이미 검증된 owner cap을 전달하고 active Budget이 있으면 항상 그 cap/누적을 우선하도록 연결한다. default 순서/비용을 보존하면서 inactive local allocation의 유한 cap만 owner와 일치시킨다. caller가 임의 raw cap을 전달하는 공개 API는 만들지 않는다. 반면 `capture_persistence_rollback`의 non-battle resource clone은 `preflight_resource_clone`이 inactive이면 native 계량을 하지 않는 기존 경계다. 이는 fallback8MiB refusal과 다르며 이번에 native operation 범위를 새로 넓혀 기본 거절을 추가하지 않는다. outside-native capture는 allocator 측정에 포함하고 native metrics의 scope/missing을 명시한다. 신규 profile은 live generation 교체와 rollback에서 실제 GameData owner를 따르며 deferred scope를 중간에 교체하지 않는다.

## TD03 — 계량 의미와 고정 크기 observation

Budget의 optional observation-feature 필드는 allocation-free 고정 크기로 둔다. `seed_attempted`, `seed_accepted`, `accepted_growth`, `accepted_total`, `max_cumulative_requested` 및 별도 `max_owner_requested`, `first_soft_exceed`, `first_hard_exceed`를 구분한다. first site는 static site+kind+requested+limit이고 overflow는 unavailable/overflow 표시를 가지며 그 값을 정상 유한 요청으로 가장하지 않는다. warning 초과는 `>`이고 exact는 초과가 아니다. failed seed는 accepted_total0이지만 기존 `admitted_bytes`의 역사적 attempted 의미는 몰래 바꾸지 않고 문서화한다. authoring/work fault로 실제 allocation에 도달하지 못한 요청은 accepted로 세지 않는다.

owner current extent는 전체 scan/게시 owner에서만 얻는다. candidate proposed/attempted peak는 committed current가 아니다. 샘플은 operation 시작 owner extent, 관찰된 max owner extent, 성공 commit current를 따로 보관하며 실패/미계량은 None/지원상태로 표시한다. 메모리 합산 순위를 만들어 admission+extent+heap+RSS를 더하지 않는다. clone/retained 구조 계량은 observation metadata 때문에 gameplay guard가 줄어들지 않도록 observer 소유 상태로 분리한다. feature off나 observer 부재에서도 선택 profile의 admission 의미는 동일하다.

## TD04 — host allocator/RSS와 bounded 수집

Core에 OS/file I/O 또는 global allocator를 넣지 않는다. `game_server`의 optional `runtime-measurement` feature는 기존 runtime-observation을 포함하고 새 host-side `measurement` module의 allocator/RSS/snapshot helper를 사용한다. host main과 별도 측정 binary가 각 binary당 한 global System-wrapper를 선택한다. library는 global allocator를 지정하지 않는다. 원래 public guards와 R05 코드/oracle는 그대로 둔다.

allocator는 프로세스 시작부터 성공 alloc/zeroed/realloc과 dealloc을 원자 카운터로 계속 추적한다. hook 안에서 allocation/로그/잠금/파일을 실행하지 않는다. live bytes는 allocator requested size이고 allocator 내부 fragment/OS RSS가 아니다. window 시작 baseline·관측 peak·종료 live/signed delta와 alloc/realloc/dealloc counts를 기록한다. 시작 전 객체의 해제, 실패 realloc, 동시 thread, window 종료 뒤 수집기 회수를 검증한다. process-wide 동시성을 표시하며 session별 allocation으로 귀속하지 않는다. concurrent snapshot은 측정 한계를 표시하고 엄밀한 heap snapshot으로 주장하지 않는다. old TRACK reset 방식으로 cross-window free underflow를 숨기지 않는다.

Linux RSS는 bounded `/proc/self/status` 읽기에서 VmRSS와 VmHWM을 얻으며 HWM은 process lifetime peak다. 다른 OS/실패/필드 없음은 explicit unsupported/missing이고 heap값으로 대체하지 않는다. OS/wall timing은 admission/RNG의 입력이 아니다.

기존 queue1024/batch128/ring300을 재사용한다. 새 registry/owners/details 상한은256 live observed sessions, operation raw ring300, session identity의 hash는 고정32bytes/seed u64/build metadata 각 bounded 길이다. 등록 포화는 attachment만 건너뛰고 별도 drop counter를 올린다. registry·worker·detail 모든 파생 container가 같은 cap을 지켜야 한다. weak lease를 통해 dead application/game data를 붙잡지 않는다. profile/content/seed/site를 무한 metric label로 만들지 않는다. raw ring은 `/state` detail 및 local runner로 소비하고 샘플 누락/overwrite/drop/지원상태를 포함한다. error 문자열과 metadata 크기도 유한하게 한다.

## TD05 — 정상 측정 실행과 identity gate

새 `game_server/src/bin/measure_runtime.rs`와 host-side helper를 권장한다. 이 binary는 `content_pack::load_with_runtime_safety`와 공개 `CoreApplication` command/admin/tick/checkpoint API로 실행하고 내부 BattleCore test state를 주입하지 않는다. normal/none/detached/full observation 모드를 받는다. trusted bounded 실행 manifest는 settings/content 선택, expected effective profile SHA, seed/player, case identity, 공개 명령·tick·checkpoint/restore 순서를 포함한다. expected hash는 입력필수이며 settings load 후 첫 게임 mutation 전에 기존 verify helper로 확인한다. content/build/input identity도 raw receipt와 대조한다. malformed/mismatch는 실행0·원본무변경이다. host의 일반 Continue는 현재 host policy로 재검증하는 원래 규칙을 유지한다.

유한 engineering 기본: manifest1MiB, 최대10000 public steps, 최대64회 반복/16개 동시 session, 최대4096 export samples와16MiB output이다. 초과는 실행 manifest를 사전 거절하거나 export를 중단하고 truncation/drop을 명시한다. collector/export 실패가 이미 정상 실행중인 gameplay를 fault로 바꾸지 않는다. output은 새 경로 create-new이며 기존 원시 파일을 덮지 않는다. 구현자는 포맷 이름·helper 분리·이보다 엄격한 상한을 자율 결정할 수 있으나 기록한다. 전체 표본 보관이 안 되는 경우 실제 retained 표본의 평균/p95/max와 관측 전체 count/drop을 구분한다. 작은 sample p95는 확정 최악값이 아니다.

실제 host 검증은 위 binary만으로 대체하지 않는다. 측정 feature로 실제 game_server를 실행하고 기존 WebSocket 정상 session 입력→load_balance attach→actor command/tick/fault/reconnect→loopback detail 샘플을 연결한다. trusted 설정/빌드/OS/기기·프로세스·관측 feature·동시성·입력 hash·seed·전투 규모/조합·window·정상/typed거절/중단을 raw와 summary에 결속한다. 시간/OS 숫자는 gameplay oracle에서 제외하지만 누락 여부는 검증한다.

## TD06 — 검증 순서·복원·독립 인수

1. parser/default hash/strict negative/32MiB 권한과 native seed/growth/owner/result/override exact/one-over를 먼저 실행한다. scaled 낮은 cap과 실제8/16/32MiB 수치 검증을 구분한다. nonretained work/clone/expression/overflow는 계속 거절해야 한다.
2. deferred/중첩/실패candidate/rollback/latefault/새operation·firstsite metrics와 normal/default PC 비교를 수행한다. 기본 비용은 실제 raw로 확인하며 layout 의도만으로 불변을 주장하지 않는다. 새 기본 refusal/경계 약화는 교정 대상이다.
3. allocator window·RSS·registry/ring/file 포화와 정상 Core/host 실행을 붙인다. on/off/absent/full의 같은profile gameplay/저장 동등, identity mismatch0execution, clone/currenthost checkpoint, 반복 생성/TTL해제/전투종료/회수·Pause/reconnect·동시다른profile을 시험한다.
4. normal 전투, U07 생성물/벽/함정·field 소비, 반복 lifecycle, default/dev warning/hard refusal의 현재 지원 최소 표본을 만든다. default PC36조건/576행과 원래5거절은 유지한다. 개발 비교는 별도 label에서 동일 입력을 재현하되 모든 default거절이 성공해야 한다고 가정하지 않는다. work 등 다른 보호가 먼저 걸린 결과도 그대로 기록한다.
5. 기능 workers 종료 뒤 PC/R05 성능측정을 실행한다. 최종 Core default/fixtures/observation와 host default/observation/measurement, fmt/check/event_log/clippy/release/public/numeric/fault/allocator/persistence의 영향 있는 실제 current receipts를 선정한다. inherited R05 concurrent peak 경고와20분stress 제외만 그대로 두며 새 예외를 추가하지 않는다.
6. selection→181+31 predecessor impact→harness adaptation→10acceptance/10dimension coverage→GEX10→문서최신화→exact target 정역복원 순서로 닫는다. 감사 중 inputs 쓰기를 멈추고 비참여 Astra/High가 whole-goal GEX20을 수행한다. finding은 Sol이 교정하고 같은 독립 감사자가 재감사한다. Complete/High/open0 뒤 U08로 진행한다. 단일 phase이므로 MasterGEX30은 없다.

## Element Coverage

| Element / 검증 seam | 기대 결과 |
| --- | --- |
| 01 / V01 trusted parser/loader/host | 생략/명시기본 hash와8MiB unchanged; 잘못된 dev/content/save 완화거절; 명시 dev만 effective identity 변경 |
| 02 / V02 native owner/operation/result | warning exact/one-over와 finite cap exact/one-over 구분; 낮은 override·다른보호·overflow 유지 |
| 03 / V03 nested/candidate/latefault | 누적 비환급/다음operation fresh seed, typed sticky fault와 atomic state/queue/RNG/save 보호 |
| 04 / V04 separated raw measurements | seed/accepted/request/extent/allocator/RSS 범위·단위·실패/missing/overhead 분리 |
| 05 / V05 bounded collector/export | absent/full/failed/off gameplay 동일; registry/ring/file/sample cap와 정확 count/drop/표본요약 |
| 06 / V06 identity/clone/restore | 측정 replay mismatch mutation0; liveclone profile 유지; 현재host Continue/SavePoint 원본 보호 |
| 07 / V07 normal Core/actualhost | 공개 경로 nonzero sample과 결과·환경·seed·input/profile/build·재현명령 결속; default/dev 분리 |
| 08 / V08 lifecycle/concurrent | 반복 생성·만료·종료 후 회수와 observer retirement; Pause/reconnect/rollback/다른profile isolation |
| 09 / V09 current integration/audit |10차원/181+31/current receipts/oracle/실패 보존·target복원·독립 CompleteHigh/open0 |
| 10 / V10 tuning handoff | profile revision/전후 raw/이유/영향·U08/U11/U30-C/P owner·미확정 플랫폼/최종값 명시 |

## Findings와 설계 재검토 조건

IF01은 trusted profile entry와 직접 owner cap 정합의 미구현, IF02는 attempted/accepted/committed 계량 구분 부족, IF03은 연속 allocator/RSS·유한 registry/export·정상 샘플 부족, IF04는 실제 replay identity 소비 부족이다. 모두 등록된 후속 범위의 구현 finding이며 새로운 게임정책 결정이 아니다. ED01은 현재 정적 검토만으로 비용/실측값을 확정할 수 없다는 증거 한계다. required source gap와 material policy finding은0이다.

권장 내부 타입명/파일분리/테스트 fixture/예제 command·collector 구현은 Sol이 자율 결정한다. native default 한도/오류/공개serialization·기존 저장 권한·누적계량·게임 결과·완료 acceptance를 바꾸거나 trusted profile 없이 ownercap확대가 필요하면 코드 근거와 대안을 계획자에게 반환한다. 자동 사용자 승인 대기가 아니다. 후속 U30-C/P가 목표플랫폼/정상최대·동시부하와 여유를 근거로 최종값을 조정하며 이번 작은 샘플을 그 증명으로 사용하지 않는다.
