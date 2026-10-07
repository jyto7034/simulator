# R03 공통 제작 포트 — 코드 기반 검토

`WR-CATP-P2G10-001`과 `RSP-CATP-P2G20-001`을 고정한 P2G-20 검토다. 정책·13개 acceptance의 원본은 [P2G-10](P2G_10_POLICY_001.json)이다. 본문의 P01–P07은 아직 구현하지 않은 기술 제안이다. 코드·데이터·시험·build를 변경하거나 실행하지 않았으며, 이 문서는 GEX 완료 판정이 아니다.

## 1. Source와 검토 범위

Workspace Actual은 `/mnt/f/work/simulator`, HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`의 기존 dirty 구현이다. R01/R02의 실제 current audit003/audit002와 target003을 인수했다. 정확한 per-file hash·읽은 범위·관찰 방법·선행 currentness는 `P2G_20_SOURCE_EVIDENCE_001.json`, 별도 읽기 보조 결과는 `review-evidence/runtime_trace_review_001.{md,json}`가 소유한다. 파일을 hash했다는 사실과 내용을 의미 검토했다는 사실을 구분한다.

Root가 공통 포트·진단 producer·immutable source/compiled definition·전환/검증을 검토했다. `/root/r03_runtime_trace_review`는 별도 no-conversation-fork 읽기 보조로 공개 시험 경로·trace·원자성·실행 예산을 조사했다. 이 보조는 구현자도 완료 감사자도 아니며 root가 결과를 검토하여 본문에 반영한다.

RQ01–06은 모두 Workspace Actual을 요구한다. RQ07의 server/Unity·deployed/personal-store compatibility는 exact input scope O01/O02/O04/O05/O06상 N/A다. 실제 local default-release 소비 증거는 그 source/build의 관찰이며 배포된 제품의 호환 판정이 아니다. 새 R03 runtime 실행은 0이다.

## 2. Current Actual

### B01 — 공개 authority와 release 경계

`application.rs`의 `CoreApplication`은 private `GameCore`를 소유한다. ValidatedGameData와 명시 seed로 생성하고, 검증된 `ApplicationCommand` 및 `ApplicationExecutionContext`로 실제 command를 실행한다. Context의 battle UUID/revision은 실제 조회 결과에서 받아야 한다. Snapshot/reconnect는 읽기 전용이고 elapsed/host-tick/next-event-bucket은 실제 공통 전투 경로를 사용한다. Public mutable inner borrow나 raw kernel 초기화 포트는 없다.

`world/application.rs:174–213`은 lifecycle 검사→complete authority clone→operation→fallible projection/validation→state/seed/game_data/output commit을 소유한다. 새로운 authoring control/observation 필드를 추가하면 현재의 prepared commit에 자동으로 포함되는 것이 아니다. 별도 owner/lifetime을 명시해야 한다.

Cargo는 `autobins=false`다. 현재 `package_official_content`/`consume_content_pack`만 일반 production pack 도구로 등록돼 있다. Consume은 실제 briefing/starter→Map까지이며 일반 simulate/reload 구현은 없다. 기존 `battle_kernel_api_audit/valid`는 외부 data와 실제 공개 명령으로 battle을 준비하고 readonly reconnect를 검증하지만 nonzero advance를 하지 않는다. 이를 R03 전투 실행 성공으로 확대하지 않는다.

### B02 — 로딩과 source 보존

공통 production `content_pack::load`는 bounded owned snapshot→24개 typed catalog parse→42 kind의 명시 Add/Replace/Remove와 winner→최종 typed 의미 hash→assembly/final validation→summoned/boss compile→generated setup 검증→opaque Arc generation을 반환한다. Compiler 호출은 `ParsedGameplayRon::validate_fallible_domain_contracts` 안에 있으므로 전체 `into_validated` 오류를 하나의 compile 단계로 표시할 수 없다.

Generation은 database/identity/source byte receipt를 보존한다. `SourceTable`은 winning DefinitionKey와 pack/file/original collection ordinal을 알지만 publication 뒤 폐기된다. Resolver도 실제 writer/after/selected 여부를 계산하나 현재 semantic hash 입력에만 사용하고 일반 설명 API로 보존하지 않는다. Source/path와 부가 설명 metadata는 semantic identity나 V6 authority의 원본이 아니다.

### B03 — 진단 생성과 손실

`LiveRonError`는 file/domain/object/action/effect/field/cause/related source를 보존하지만 stable code/phase/pack/kind/parser position/typed expected-actual이 없다. Raw parse는 pack ID를 알고, resolver는 writer source와 DefinitionKey를 알며, SourceTable은 winner를 알지만 일부 메타데이터를 오류로 전달하지 않는다. 여러 파일의 fallback contributor는 현재 cause 문자열에 들어간다. R02 correction003이 보존한 custom policy wave·template·boss behavior/pattern 관련 출처는 필수 회귀 기준이다.

실제 RON 0.5.1 dependency의 `de::Error::Parser(ParseError, Position)`는 typed parse reason과 line/col을 제공한다. 현재 `parse_live_ron`은 그 값을 문자열로 낮춘다. `Message`/IO error는 같은 정밀도를 제공하지 않는다. Manifest를 읽거나 파싱하기 전에는 pack ID가 없으며 폴더 이름을 ID로 추정할 수 없다. 일부 최종 validator는 object/field까지만 알며 정확한 span/expected 값을 제공하지 않는다. 이 부재는 사실로 유지해야 한다.

ApplicationError에는 기존 stable code/kind/details가 있다. Trial 단계는 이를 그대로 구조화해 담을 수 있다. `InvalidStaticData`의 free reason에서 가짜 source 위치를 추출해서는 안 된다.

### B04 — 현재 설명·trace의 실제 정보

R01 `EFFECT_SUPPORT`는 editable literal Damage/Heal과 Instant, 정확한 Amount/DamageType 속성·type/unit/editability를 설명한다. 이는 더 넓은 기존 skill executor 전체가 그 조합만 실행한다는 뜻이 아니다. Skill database는 exact EffectSite 조회와 same-effect/each-exists predicate를 제공한다. EffectSource는 원문 action/effect 주소와 Original/Added origin을 갖는다.

Boss database는 실제 compiled pattern/state/task/transition·actor behavior·encounter graph를 읽는 accessor를 제공한다. Compiled indices와 actual stable IDs를 설명할 수 있으나 release에서 authored catalog 전체는 보존되지 않는다. Compiled array 순서를 원문 위치로 추정하지 않는다.

실제 SkillCastExecutionSource의 caster/cast_seq와 EffectSource는 source attribution의 근거다. Ability/proc event seq는 log correlation이다. Skill effect→command→record_event_log에서 committed typed entry에 source가 붙고 projectile/area는 source Arc를 보존한다. `skill_effect_source`는 serde(skip)라서 현재 public JSON에서 빠진다. 로그는 8,192개/4 MiB/180,000 ms bounded window이며 skipped Arc heap은 serialized cap 밖이다. 완료 battle의 log는 test/debug/test-fixtures에서만 유지되고 default release에서는 active_battle이 제거된다. 호출이 끝난 뒤 active log만 읽으면 마지막 단계의 provenance를 잃는다.

### B05 — 준비·clock·실패와 상태 수명

실제 시작은 public StartNewGame→observed VN/starter 선택→observed available node 선택→ConfirmEnterNode이다. `commit_planned_live_battle_start`는 UUID/entry snapshot/active battle을 준비하고 실제 time-zero battle 처리를 수행한다. 단순 token 로딩이나 StartNewGame generation 교체만으로 새 trial battle 준비가 완료되지 않는다.

Elapsed와 next-event는 deterministic driver 포트다. Host-tick은 pause/speed fractional remainder 및 blocking VN 의미를 소유한다. 세 clock을 같은 뜻으로 정규화하면 안 된다. Core 공개 command의 성공은 각각 commit되며 나중 command 실패로 이전 성공을 소급 rollback하지 않는다.

Game save는 R02의 strict content-bound V6이며 active battle/diagnostic history를 durable checkpoint에 넣지 않는다. PlayerProfile/personal store는 host authority다. Tool의 process-local candidate/session/receipt를 저장 graph로 추가할 필요가 없다. Query가 RNG/revision을 바꾸지 않는지 공개 관찰과 이어지는 같은 실행으로 검증할 수 있다.

### B06 — 기존 한계와 남은 실제 실행 공백

R02는 file/pack/edge/nesting/owned input을 제한한다. Boss runtime은 자체 microstep guard와 staged error를 갖는다. 하지만 `sim.rs:1820–1948`의 same-time bucket drain/replenish와 target-time loop에는 authoring work budget이 없다. Command expansion, effect/target iteration, deferred deliveries와 queue growth도 host 호출 수만으로 제한되지 않는다. `process_commands` 계열에는 현재 infallible summary 경계도 존재한다.

따라서 유한한 scenario steps/clock만으로 runaway 한 호출을 중단한다는 주장은 성립하지 않는다. Trace/output cap도 work budget과 다르다. 기존 guard 수치는 식별된 source/workload의 관찰이고 새 metadata·clone·observer·fuel 경로의 비용을 증명하지 않는다.

## 3. Approved Target·acceptance와 delta

정책 문장과 Observable Policy Acceptance는 exact `PER-CATP-E01..E06-DECIDED-001`/현재 Decision을 변경 없이 소비한다. 아래는 승인 기준의 구현 대응이며 acceptance 재작성이나 추가 정책이 아니다. `ACn`은 각 owned acceptance array의 1-based ordinal이다.

| 요소 | Approved Target/Acceptance 원본 | Current Actual → 필요한 delta | 제안/검증 |
| --- | --- | --- | --- |
| E01 | PER-CATP-E01-DECIDED-001, AC1–2 | B01/B02/B04 → 같은 loader/core의 공통 facade, 현재 descriptor와 얇은 release CLI | P01/P03/P04/P07; V01/V02/V03 |
| E02 | PER-CATP-E02-DECIDED-001, AC1–2 | B02/B03 → 실제 producer code/phase/known source와 동일 API/CLI 진단 | P02/P07; V01/V02 |
| E03 | PER-CATP-E03-DECIDED-001, AC1–2 | B01/B04–06 → 일반 public scenario, nonzero battle, 실제 occurrence/receipt/clock 재현 | P04/P05/P06; V03/V04/V07 |
| E04 | PER-CATP-E04-DECIDED-001, AC1–2 | B04/B05 → immutable definition/source/compiled graph와 실제 관찰 조회 | P03/P05; V04/V05 |
| E05 | PER-CATP-E05-DECIDED-001, AC1–2 | B05/B06 → B actual battle prep 뒤 complete candidate swap, 실패 A 보존 | P04/P06; V06/V07 |
| E06 | PER-CATP-E06-DECIDED-001, AC1–3 | B02/B05/B06 → introduced input/work/output/control/retention 한계·회수·현재 회귀 증거 | P02/P04–07; V02/V06–09 |

## 4. Proposed Implementation

### P01 — 하나의 공통 library와 얇은 consumer

권장 구조는 public authoring facade가 validate/describe/explain/simulate 및 process-local trial session을 제공하고 CLI가 같은 request/result 타입과 serializer를 호출하는 것이다. `lib.rs` export와 명시 Cargo bin을 함께 전환한다. 내부 파일/타입 이름은 GEX 상세 설계 재량이다. 공통 loader/token/CoreApplication과 기존 domain transaction을 재사용하고 두 번째 validator·battle evaluator·content-name dispatch를 만들지 않는다.

CLI는 explicit trusted content root/selection과 bounded stdin/file request만 받는다. CWD 탐색, embedded 보충, implicit winner나 file watcher는 필요하지 않다. Common library가 모든 semantic validation·failure envelope를 소유한다. CLI의 usage/read/write transport 오류는 별도 code를 가지며 successful tool execution과 구분한다. 기본 release에서 외부 public consumer가 같은 API를 실제 호출해야 한다.

### P02 — 진단은 생성 지점에서 보존

공통 serializable diagnostic은 stable code/phase, source/related_sources, optional expected/actual/capability, 원래 message와 precision/availability를 표현한다. Input acquisition, parse/structural, resolve/final validation, compile, generated setup, trial preparation/execution/observation의 실제 producer에서 식별한다. 기존 LiveRonError의 source/cause/related 필드와 ApplicationError meaning을 보존하며 확장한다. 정확한 enum/code spelling은 GEX에서 고정하고 descriptor·시험·계약에 함께 등록한다.

Parse는 RON의 typed parser error/position와 serde path를 변환 전에 포착한다. 구조 오류에서 typed expected 값이 없으면 message만 유지하고 부재를 명시한다. Pack ID는 실제 manifest decode/validate 뒤에만 결속한다. Resolver와 SourceTable은 알고 있는 DefinitionKind/pack/winning field를 전달하고 aggregate contributors를 구조화한다. Compiled summoned/boss 경계는 final validation과 분리해 phase를 붙이며 더 정밀한 기존 오류를 wrapper가 덮지 않는다. Generated setup은 R02 typed primary/related source를 유지한다. Trial의 generic runtime error에는 step/context와 실제 retained content identity를 붙이고, 알 수 없는 authoring span은 만들지 않는다.

원문을 다시 열거나 Display/cause를 parsing해 source를 재구성하지 않는다. `unknown`/`unavailable`은 성공이나 정확한 source 위치가 아니다. 실제 오류가 가진 expected/actual은 구조화하고 없는 값은 조용히 fabricated default로 채우지 않는다. API/CLI는 같은 envelope serialization을 사용한다.

### P03 — immutable 설명 metadata와 현재 지원 registry

Generation publication 전에 winning definition source inventory와 실제 resolution receipt를 immutable metadata로 보존하는 것을 권장한다. 기존 SourceTable/WriterReceipt의 실제 producer를 재사용하며 hash용 typed payload/order/metadata 입력은 바꾸지 않는다. Metadata의 source/heap 크기를 제한·계측하고 마지막 generation strong reference와 함께 해제한다. 전체 raw RON text나 두 번째 mutable database를 영구 보관하지 않는다.

Describe는 current ContentContract/capability, 실제 CatalogKind/DefinitionKind, public command/snapshot/clock와 authoring operations를 해당 owner registry에서 가져온다. Editable effect support는 R01 EFFECT_SUPPORT를 그대로 소비한다. 기존 더 넓은 runtime vocabulary와 최소 editable surface를 구분하고 미지원 expression/chain/composition 기능은 명시한다. 지원 row와 parser/validator/executor의 왕복 검증으로 거짓 advertise를 막는다.

Explain은 exact DefinitionKey의 실제 source·selected writer/after/Remove disposition과 현재 identity를 설명하고, exact skill EffectSite의 op/tags/origin/support, 현재 compiled boss pattern/behavior/encounter의 실제 ID/edge/argument type을 읽는다. Enum/indices를 source span으로 위장하지 않는다. Definition-local source가 없는 곳은 parent definition 주소와 precision을 표시한다. Preview는 현재 공개 readonly projection 또는 이미 확정된 trial observation을 읽는다. 새 gameplay 결과를 예측한다고 속이거나 query 중 core clock을 진행하지 않는다.

### P04 — 일반 scenario와 실제 session 교체

권장 입력은 strict finite Command/Snapshot/Clock/Assert step 목록이다. 초기 authority는 explicit content selection, player UUID, run seed, 새 in-memory default profile의 출처와 공개 초기 명령이다. Arbitrary private world/battle state import는 지원하지 않는다. Persistent checkpoint/profile import가 향후 필요하면 기존 공개 검증·authority를 소비하는 별도 명시 입력을 설계해야 하며 이번 최소 surface는 새 session부터 시작한다.

Dynamic VN/starter/node/unit/battle UUID/revision은 앞선 public result/snapshot의 bounded JSON-pointer binding으로 가져온다. Binding은 prior step과 document member의 복사만 허용하고 forward/cyclic/missing/type mismatch를 거부한다. 산술·조건식·무한 loop·외부 코드 실행이나 임의 content-ID 분기는 없다. 작은 반복 선택이 필요하면 scenario 작성자가 유한 steps와 명시 배열 위치를 제공한다. 공식 예제의 IDs/수치는 fixture input에 있으며 engine 분기에 들어가지 않는다. Template와 실제 resolved command/context를 모두 receipt에 기록한다.

Clock step은 HostTick/Elapsed/NextEventBucket을 정확히 구분하여 동일 public method로 보낸다. HostTick의 pause/no-update·fractional remainder와 blocking VN을 보존하고 raw driver의 의미를 descriptor에 명시한다. 하나의 clock을 다른 clock으로 silent fallback하지 않는다.

Session은 generation token, CoreApplication, immutable initial/request basis, committed step cursor/results/receipt/trace를 하나의 aggregate로 소유한다. Reload는 A와 별개로 B를 load하고 요청된 초기 public flow를 실제 battle entry/time-zero 처리까지 준비한다. `in_battle` 또는 실제 prepared-and-immediately-terminal 결과의 검증된 battle identity로 preparation 성공을 판정한다. Mere load/Map/StartNewGame 성공은 충분하지 않다. 준비·입력/출력/예산/취소 검사를 모두 통과한 후 complete B aggregate로 한 번 교체한다. 실패한 B의 부분 progress는 실패 보고에만 속하고 A token/state/seed/input/receipt를 바꾸지 않는다.

일반 시험 진행은 step별 candidate에서 같은 public CoreApplication 호출을 수행하고, 해당 step의 observation·receipt serialization/caps까지 준비한 뒤 aggregate를 확정한다. 실패한 step만 미확정이며 이전 성공 steps는 유지한다. Core가 candidate 안에서 commit한 뒤 output cap 실패가 났다는 이유로 이미 live A가 바뀌도록 두지 않는다. Caller의 외부 transport write는 semantic commit과 별도이며 accepted 결과 전송 실패를 domain rejection으로 바꾸지 않는다.

### P05 — 실제 provenance observation과 bounded receipt

Authoring-only observer는 실제 runtime에서 생성한 typed source/command/effect/resource before-after/action milestone을 읽는다. 일반 public wire DTO에 serde(skip)를 해제해 metadata를 흘리는 방식은 피한다. 보존된 event log의 완전성을 가정하지 않고 actual producer에서 bounded staged observation을 포착하며, terminal transition이 active battle을 제거하기 전에 마지막 관찰도 준비한다. Observer buffer는 tool-step candidate에 속하고 rollback 시 폐기된다. 일반 gameplay result/statistics는 기존 owner가 계속 계산한다.

Semantic identity는 실제 battle/caster/cast/action/effect와 실제 delivery/hit/tick occurrence를 사용한다. 같은 action/effect가 여러 번 실행될 때 구별할 metadata가 부족하면 실제 execution boundary에서 trace와 무관하게 deterministic occurrence address를 포착한다. Log seq/source_command_id는 correlation 필드로 별도 유지하며 semantic ID나 RNG seed로 승격하지 않는다. 기존 cast/event/RNG allocation을 바꾸지 않는 것이 전환 조건이다. Delayed projectile/area는 actual captured source를 계속 소비한다. 현재 지원하지 않는 full derived chain/composition reasoning은 unsupported/absent로 표시하고 R06/R13 등의 executor를 임시 구현하지 않는다.

Receipt에는 schema/engine contract, content semantic fingerprint와 exact source bytes receipt, explicit initial authority/seed, input templates+resolved values, ordered clock/steps, command dispositions/results/errors, 실제 battle/time/event/effect observation counts, partial/completed status와 trace coverage를 기록한다. Terminal까지 실행하지 않은 유한 scenario는 partial로 기록한다. Scenario 끝을 Victory/완료 전투로 조작하지 않는다. 실행 오류는 실제 실패 step와 마지막 확정 basis를 식별한다.

Trace 용량 초과/eviction/unavailable source는 coverage gap으로 명시하고 complete trace라고 보고하지 않는다. Diagnostic retention drop은 gameplay 판정을 실패시키지 않는 기존 계약을 유지한다. Trace on/off가 동일 admitted 실행의 result/RNG를 바꾸지 않아야 하므로 trace의 수집 용량만 넘었다고 gameplay를 abort하지 않는다. 요청/receipt 자체의 hard limit 위반은 P04 candidate publication 전에 거부한다. Receipt 출력에서도 count/bytes/depth를 먼저 제한하고 bounded writer로 예상보다 큰 serialization allocation을 막는다. Source 문자열과 shared skipped metadata의 실제 heap 비용도 separately 계측한다.

### P06 — 실제 work control·취소·회수

Authoring opt-in execution control은 deterministic work fuel과 cancellation을 실제 domain draft까지 전달한다. Same-time pop/replenish, bucket iterations, command/effect/target expansion, deferred work와 queue/runtime entity 생성 등 입력이 증폭되는 경계에서 선행 charge/check를 수행한다. Time-zero battle 준비와 command-time 실행에도 적용한다. 기존 boss microstep guard를 제거하거나 대신했다고 주장하지 않는다. 큰 수를 처리한 뒤 output 길이만 검사하는 구현은 불충분하다.

Fuel 소진/취소는 authoring execution failure이고 gameplay outcome/Draw가 아니다. Infallible helper가 있는 경우 Result 전파로 바꾸거나 모든 반복이 promptly 중단되는 sticky fault를 draft commit 전에 검사하는 구현이 가능하지만 silent partial success는 금지한다. Mutable control counters를 A와 B/draft가 공유하여 실패 시 A의 quota를 소비하지 않도록 한다. 외부 cancel signal은 읽기 전용 원인이며 retained gameplay state가 아니다. 일반 core caller의 기존 semantics와 기본 경로는 보존하고 opt-in 비용·변경 영향은 새 측정으로 확인한다.

Limits는 trusted host config가 owner다. New scenario byte/depth/step/binding/string count, per-call·total clock/work, queue/entity/retained generation, output/trace/receipt 한계에 finite hard ceiling과 checked arithmetic이 있어야 한다. Exact 수치는 GEX가 actual baseline workload·boundary·over-limit 실행과 비용으로 결정하는 engineering 설정이며 공식 balance 값이 아니다. 선택 이유·단위·inclusive/exclusive 경계를 descriptor와 receipt에 남기고 unbounded sentinel은 허용하지 않는다. R02 InputLimits를 낮출 수는 있어도 untrusted request가 ceiling을 높일 수 없다.

취소·failed reload는 candidate만 버리고 마지막 committed A를 유지한다. Successful replacement는 이전 aggregate를 drop하고 명시 외부 strong reference가 없으면 이전 generation/observer/cast metadata를 회수한다. Receipt는 재현에 필요한 식별값을 소유하되 이전 전체 session/generation을 강하게 붙잡는 전역 history/cache를 만들지 않는다. Process 종료 시 session과 control은 사라지며 personal save/profile 파일을 읽거나 지우거나 변환하지 않는다. 동일 seed/content/request로 replay하고 lossless in-battle checkpoint 복원이라고 주장하지 않는다.

### P07 — 원자적 전환과 기존 기능 검증

Source diagnostic producer→public facade→CLI/descriptor/consumer, actual runtime control/observation→terminal extraction→tool-step commit/receipt→reload/lifetime은 완결된 하나의 최소 transition 단위다. 내부 구현 작업을 나눌 수 있지만 일부 support row/CLI만 먼저 성공으로 게시하지 않는다. 새 surface 없이 기존 R01/R02 public behavior를 유지하는 작업 중 상태 또는 모든 위 계약이 성립하는 완료 상태만 유효하다.

P2G-40은 하나의 primary Phase/Single Goal 안에서 최소 atomic group의 implementation packages와 이후 validation package를 권장한다. 세부 WP 분할은 공유 계약·actual dependency를 보존해야 한다. 성공 뒤 R04 이후를 수행하지 않아도 독립 외부 콘텐츠의 검증/설명/실제 시험/reload가 완결돼야 한다.

기존 source/data dirty는 GEX 전에 실제 snapshot/hash로 보존한다. R03 delta를 clean HEAD와 혼동하지 않는다. R01/R02를 침범하는 source/contract/input 영향은 새 target/current audit 또는 입증된 비영향 재사용으로 복구하며 역사적 target/audit/실패 log를 덮지 않는다. Canonical data_loading/core_runtime 계약은 새 실제 surface와 수명/예산/진단에 맞춰 같은 구현 pass에서 갱신한다. Server/Unity, personal store와 R04 이후 gameplay opcode는 이 전환에 포함하지 않는다.

## 5. 대안·제약·상세 설계 인계

| 대안 | 판정과 이유 |
| --- | --- |
| 기존 consume probe를 일반 simulator로 표시 | Nonzero combat/일반 입력/reload가 없으므로 E01/E03/E05를 충족하지 않음 |
| 콘텐츠 이름별 자동 진행 runner/private fixture world | 실제 public authority와 independent authoring을 우회하므로 채택하지 않음 |
| bounded JSONL interactive session | 같은 공통 finite step owner를 소비하는 추가 adapter로 가능하나 최소 batch CLI보다 lifecycle/transport가 늘어남. 이번 권장은 batch scenario + library session |
| source map을 request마다 파일에서 재구성 | winning owned snapshot/동시 변경/lineage 보존을 잃음. publication 때 실제 producer metadata를 보존 |
| 별도 AuthoringContent에 metadata 보관 | 같은 publication result에 결속하면 가능. 권장은 compact immutable generation metadata이며 상세 배치는 owner/lifetime/hash 불변을 유지하는 GEX 재량 |
| log seq만 semantic ID로 사용하거나 무제한 full log 유지 | RNG/retention/terminal source 경계와 불일치. actual occurrence와 bounded coverage를 분리 |
| host timeout/step count만 실행 제한 | 단일 call의 work/queue expansion을 차단하지 못함. 내부 scoped work control 필요 |
| 모든 production gameplay에 새 timeout/Draw 도입 | 승인된 의미가 아니며 채택하지 않음. authoring tool failure만 추가 |
| output 전송 실패 뒤 live Core rollback | 기존 accepted/publication 의미를 훼손. 후보 안에서 fallible receipt를 준비하고 transport 결과는 별도로 처리 |

GEX는 exact request/response types, stable code registry, internal control plumbing, bounded serializer/metadata storage, actual occurrence key component를 current source에 맞게 확정한다. 이는 무제한 재량이 아니다. Root authority·세대/후보 수명·step commit·실패/취소 결과·descriptor honesty·semantic determinism·필수 검증을 바꾸거나 미루면 해당 owner로 반환해야 한다. 새 gameplay/persistence/compatibility 의미가 필요하면 P2G-10 material finding이다.

## 6. 검증 연결 — 모두 향후 실제 실행

| ID | 실제 owner/입력·최소 기대 결과 | 정책 |
| --- | --- | --- |
| V01 | 같은 독립 external pack을 default-release public library와 CLI로 validate/describe/explain. 실제 token/fingerprint/source/지원 의미 일치; current R01 editable 조합 positive/unsupported negative; 다른 kind/compiled graph 설명 | E01/E02 |
| V02 | syntax/unknown field/malformed manifest·contract/capability/ambiguous winner/final dangling reference/summoned-boss compile/generated policy-wave·custom boss source/trial invalid authority의 같은 입력. API/CLI code/phase/source/related/available expected-actual parity, unknown precision 부재, zero candidate publication | E01/E02/E06 |
| V03 | source 사본의 actual effect 수치만 바꾼 A/B를 일반 public scenario로 준비하고 nonzero advance/실제 effect execution까지 호출. 관찰 HP/resource/result delta와 actual source/fingerprint 결속, actual commands/nonzero counts/exit와 raw receipt 저장 | E01/E03 |
| V04 | 같은 initial/player/profile provenance/content/seed/template+resolved input/clock 재실행; trace on/off와 repeated/delayed/area, log eviction/terminal source, semantic identity가 log correlation에 의존하지 않음. 동일 gameplay/RNG/next decision 및 정확한 gap 표시 | E03/E04 |
| V05 | exact definition/effect/origin/compiled graph 설명과 readonly Preview 반복 전후 공개 snapshot/reconnect/revision, 이어지는 같은 command/clock 결과 일치. 누락 정의·unsupported query는 명시 거절, 가짜 forecast 없음 | E04 |
| V06 | 실제 A→B 성공; bad syntax/compile/generated B 및 valid token+실제 trial prep 실패 B를 각각 반복; time-zero/출력 한계/취소 실패 A token/state/seed/input/receipt 불변; 이후 valid B 회복. 실패 단계 앞의 committed 성공 보존; 소유 참조 drop 확인 | E05/E06 |
| V07 | 빈/최소/정확 cap/cap+1/잘못된 nested request/binding/clock overflow, same-time 반복·command/deferred fan-out·queue growth의 작은 fuel 한계와 취소. 한 호출 내부에서 실제 중단·명시 오류·rollback, trace overflow는 gameplay 불변/gap, receipt 오류는 후보 미게시 | E03/E05/E06 |
| V08 | 기존 meaningful core suite, 관련 R01 effect/provenance/delayed·R02 loader/pin/V6/profile/failed-candidate invariants, all-target/default release와 실제 독립 positive API/negative privacy consumers. Source 변화로 침범한 선행 currentness·fresh audit 복구 | E01–E06 |
| V09 | 새 load/source inventory/trace/candidate clone/fuel의 실제 시간·할당·peak/retained/generation count와 비용을 source/toolchain/workload별 기록. Plain latency와 allocator timing을 분리. 기존 baseline/guards는 actual impact를 대조하여 재실행하거나 exact non-impact reuse 근거를 제시 | E06 |

완료 증거에는 실제 command/env/input/source hash, 선택 test/nonzero execution count, expected/actual/exit, 실패 로그와 교정 관계가 필요하다. Static source review나 validator PASS는 게임 PASS가 아니다. 재현용 prototype 수치는 별도 fixture에만 있고 공식 원본을 변경하지 않는다. R22 독립 제작·R23 전체 safety/performance·R24 최종 감사를 본 검증으로 완료 처리하지 않는다.

## 7. Finding·unknown disposition

| ID | 분류·관련 요소 | 근거와 처리 |
| --- | --- | --- |
| F01 | technical_decision; E01/E03/E05 | B01/B05: 기존 public caller를 그대로 확장한 일반 finite step facade, actual B battle prep/complete aggregate swap. 새 gameplay 선택 없음 |
| F02 | technical_decision; E02/E04/E06 | B02/B03: source/typed parse metadata의 실제 producer 보존, known/unknown 분리. P02/P03과 V02에서 closure |
| F03 | technical_decision; E03/E04/E06 | B04: release terminal 출처 소실과 diagnostic retention은 staged authoring observation/실제 occurrence로 해결. Trace cap 자체가 gameplay failure를 만들지 않음 |
| F04 | technical_decision; E03/E05/E06 | B06: host count만으로 불충분. P06 opt-in domain work control을 동반 구현·측정하고 no partial success. 현재 안전 완성 주장 없음 |
| F05 | technical_decision; E05/E06 | B01/B05: core call 뒤 fallible receipt failure 위험은 outer tool candidate step로 처리. Pre-existing step successes와 A reload preservation을 혼동하지 않음 |
| F06 | existing_decision; E01/E04/E06 | current registry/minimal editable 범위와 후속 unsupported를 표시. 새 expression/chain/composition/공식 값·save/UX 정책을 결정하지 않음 |
| F07 | scope_exclusion; E01/E06 | RQ07 exact approved scope: server/Unity/deployed/personal-store compatibility N/A. Local release를 배포 증거로 확대하지 않음 |
| F08 | technical_decision; E01–E06 | 기존 dirty/current audits는 실제 target로 재확인; GEX에서 영향받은 predecessor evidence/audit를 복구하고 원본 보존 |

Required source에서 inaccessible/unknown blocker는 확인되지 않았다. 원래 producer에 없는 세밀한 source span/derived-chain semantics는 E02/E04가 요구하는 정직한 capability/precision 표현으로 처리한다. Exact engineering limits와 내부 타입·계측 결과는 GEX 상세 설계·실행 소유의 미완료 일이며 현재 runtime PASS가 아니다. 새로운 material acceptance 변경 필요는 확인되지 않았다. Complete Review Batch publication은 root semantic disposition과 exact source/refs/coverage 검사를 마친 뒤 별도 stage 결과가 소유한다.
