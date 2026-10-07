# R03 공통 제작 포트 — 실행 계획

Current correction001: the closed [INITIAL audit](COMPLETION_AUDIT.md) found **Not Complete / R03-AUDIT-F01** on target001. [Correction001](CORRECTION_001.md) now supplies exact immutable SourceReceipt in all initial/ready/step payloads and has completed focused59/full1327, API/CLI/source-byte/replacement and separate cost validation on [INPUT119](evidence/INPUTS_119.json)318 inputs. [GEX10 result002](GEX_10_RESULT_002.json), [evidence map002](ACCEPTANCE_EVIDENCE_002.md) and [predecessor impact002](PREDECESSOR_IMPACT_002.md) own the current implementation handoff. The next exact target is AUDIT_TARGET_002.json; a new separate fresh reviewer must determine predecessor currentness and R03 completion. Source mutation is stopped for that audit. R03 remains unproven until the actual new verdict. All earlier pass statements below are historical and retain their original evidence.

## Current 인수와 실행 권한

2026-09-11 KST. GEX-00 완료, GEX-10의 구현·인수 검증을 정리했다. Current full core1327pass, all-target/default release, 외부 API5정상·6privacy와 제작 CLI21명령, actual A/B54실패·3교체/회복, Weak 해제·plain/allocator 비용 및 기존 가드를 확인했다. 13 acceptance/9 IVA/10 dimension의 source·실행 근거표를 작성했다. GEX-10 구현·검증을 종료하고 exact target을 고정한다. 다음은 별도 R01/R02 선행 currentness 및 R03 fresh 감사다. 감사 verdict와 R03/전체 R00–R24 완료는 아직 아니다.
상위 사용자가 채택한 [AUTORUN](../../../../roadmaps/content_authoring/AUTORUN_REQUEST.ko.md) §3·5·6의 별도 자식 선택으로
`(GS-CATP-P2G50-001, GOAL-CATP-01)`을 `goal-run` root/selected Single로 지정한다. 대응 R은 R03,
Phase는 `PHASE-CATP-01`, assigned nodes는 `AG-CATP-01`과 `WP-CATP-03`이다.
이 자식은 R04 이후를 자동 승인하는 GEX root가 아니다. 후속 선택은 상위 요청이 별도로 소유한다.

Exact upstream: [HOF-CATP-P2G60-001](../../P2G_60_HANDOFF_001.json),
[FPF-CATP-P2G30-001](../../P2G_30_FREEZE_001.json), [PP-CATP-P2G40-001](../../P2G_40_PHASE_PLAN_001.json),
[GS-CATP-P2G50-001](../../P2G_50_GOAL_SET_001.ko.md), `WR-CATP-P2G20-001`.
P2G의 Authored / Not Activated / Implementation Authorized: No는 당시 작성 상태로 보존한다.
적용 실행 지침은 현재 [GEX](../../../../goal_execution_pipeline%281%29.md) Reader Contract·§5·6.1–6.7·GEX-00/10/20이다.
인수 시 exact 문서 해시는 [기준선](baseline/BASELINE_001.json)의 upstream과 실제 원본을 따른다.

Canonical execution directory는 `core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal`이다.
PLAN 역할은 이 파일, 실행 이력은 [EXPERIMENTS](EXPERIMENTS.md), 현재 판단은 [EXPERIMENT_NOTES](EXPERIMENT_NOTES.md),
향후 감사 역할은 이 directory의 `COMPLETION_AUDIT.md`다. 감사 문서는 아직 만들지 않았다.
새 Topic을 이번 요청 안에서 작성했고 원본 Goal/Handoff는 미실행을 기록한다. 기존 directory 부재,
1,007개 실행 역할 문서의 exact member 부재, current source 일치로 신규 초기화임을 확인했다.
[GEX-00 실제 결과](GEX_00_RESULT_001.json)가 인수 사실과 선행 비교를 소유한다.

## Actual baseline와 선행

HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`, branch `cleanup/simulator-dirty-worktree-20260803`,
rustc/cargo 1.91.1. 기존 dirty는 사용자/선행 작업이다. 284개 source/test/tool/config/data/계약/lock 파일
7,212,675 bytes를 `/tmp/catp-r03-baseline-001`에 byte-exact 복사하고 각 해시·경로를 기준선에 보존했다.
Scoped preexisting diff와 전체 porcelain은 baseline directory에 보존하며 clean HEAD diff를 R03 성과로 세지 않는다.
후속 검사에서 원 /tmp 사본 부재를 확인하여 [RECOVERY001](baseline/RECOVERY_001.json)으로284개 최초 hash를 전부 복구했다. 이후 delta는 [durable archive](baseline/RECOVERED_001.tar.gz)를 사용하며 최초 기록을 소급 수정하지 않는다.
새 파일은 생성 직전 부재를, 기존 파일은 사본 대비 delta를 기록한다. 이번 인수의 production mutation·게임 시험은 0이다.

R01 `ATR-CAMEC-R01-003 / COMPLETION_AUDIT_003.md`와 R02 `ATR-CAPG-R02-003 / COMPLETION_AUDIT_002.md`의
기존 Complete/High를 인수했다. 각각 756/757개 direct target, 189개 external files와 정확한 inventory,
2개 release binary, 6개 감사 출력까지 실제 재해시하여 current/unchanged임을 확인했다.
이것은 새 runtime 실행이나 새 감사 verdict가 아니다. 소스 변경 후 영향받은 선행 target/audit는 과거 사실로 보존하고
R03 delta에 맞는 검증·별도 fresh 감사로 currentness를 복구해야 한다.

별도 Shared Validation Baseline Reuse Directive는 N/A다. 동일 입력의 R02 final full core 1217은 기존 경로의
인수 근거로만 재사용한다. 새 control/source/observer/fork/CLI 비용과 동작을 증명하지 않는다.
로컬 source·기존 tests/fixture·등록 release target·cargo 환경에 접근 가능하다. 시험은 임시 외부 pack과 메모리 profile을
사용하며 기존 socket test는 필요 시 이미 승인된 cargo sandbox escalation을 사용한다. 개인 저장·서비스 배포는 범위 밖이다.

## 구현 범위와 상세 설계 준비

[accepted 코드 검토](../../CODE_REVIEW_001.ko.md) P01–07, Goal I01–06와 C01–08을 따른다.
AG의 WP01과 WP02는 공유 source/control/transaction 계약을 함께 전환한다. WP03은 검증만 소유한다.
Root가 유일한 production writer이며 AUTORUN §8의 읽기 보조는 조사 결과만 반환한다.
기존 two design-helper reviews는 완료 감사가 아니다.

공통 module은 `game_core::authoring`, CLI는 명시 Cargo bin `author_content`로 설계한다.
Library의 `AuthoringContent`는 동일 production loader가 게시한 opaque token을 소유한다.
`TrialSession`은 private CoreApplication·token·명시 initial player/run_seed/default-memory-profile provenance·유한 입력·
확정 결과와 receipt를 소유한다. Public mutable inner borrow나 public Clone은 추가하지 않는다.
CoreApplication 내부의 crate-private `fork_for_authoring`만 complete GameCore clone을 반환하는 opaque app을 만든다.

요청은 strict tagged `Command`, `Snapshot`, `Clock`, `Assert`의 유한 step 목록이다. Command template와
execution context의 JSON pointer target에 prior accepted public result/snapshot의 pointer 값만 복사한다.
Forward/missing pointer/중복·겹치는 destination/잘못된 타입은 명시 실패다. 표현식·loop·자동 VN 선택·콘텐츠 이름 분기는 없다.
Clock은 `HostTick { delta_ms, after_seq }`, `Elapsed { delta_ms, after_seq }`, `NextEventBucket`으로 구분해
각 public method를 그대로 호출한다. 사용자 입력이 이미 지정한 유한 순서만 실행한다.

각 step은 opaque fork에서 public method를 실행하고 mandatory result·resolved input·receipt를 bounded serialization으로
준비한 후 complete aggregate를 교체한다. Optional trace는 별도 count/byte allowance와 fixed counters를 갖고
mandatory admission/fuel/bindings에 들어가지 않는다. B reload는 별도 aggregate에서 실제 요청된 battle entry/time-zero까지
검증한 후 A를 교체한다. 실패한 B receipt를 A에 붙이지 않는다. 각 실패는 candidate만 drop하며 prior accepted steps는 유지한다.
Transport 출력 실패는 accepted semantic result의 delivery 실패로 구분한다.

진단은 `LiveRonError`의 기존 Display/cause/object/action/effect/related를 보존하면서 producer metadata를 추가한다.
Phase는 acquisition/parse/structure/resolution/final_validation/compile/generated_setup/trial_preparation/trial_execution으로
구별한다. RON typed Parser만 line/column·typed reason을 제공하며 Message에서 위치를 추론하지 않는다.
Manifest validation 후 알고 있는 pack ID와 raw/resolver/SourceTable의 actual kind·winner를 전달한다.
Contract/capability처럼 비교 operand가 있는 producer만 expected/actual을 채운다. Unknown은 None이다.
원문 Display를 재파싱하거나 파일을 다시 열어 주소를 추정하지 않는다.

Immutable explanation metadata는 SourceTable의 final DefinitionKey/source와 resolver의 actual writer receipt를 유지한다.
Semantic hash에 들어가는 기존 writer field/order를 변경하지 않는다. Explain은 actual effect site/origin과 compiled graph
accessor만 읽고, source ordinal이 없는 compiled index를 authored 위치로 표시하지 않는다.

실행 control은 authoring operation의 시작부터 loader generated validation, battle construction/start, same-time drain과
실제 command/target/deferred/queue/entity 증가 경계에 연결해야 한다. 구체적인 전달 수단·charge와 모든 early return은
source 조사로 확정해 아래 current 상세 설계에 기록한 후 해당 production seam을 수정한다.
Cancel signal만 외부 read-only 공유가 가능하며 A/B quota와 observer는 공유하지 않는다. 일반 caller는 control 미설치 상태의
기존 semantics를 유지한다. Fuel/cancel은 tool failure이며 Draw나 gameplay timeout이 아니다.
Actual application occurrence와 terminal-before-removal capture도 같은 방식으로 producer/consumer를 함께 상세화한다.

## 검증·완료·중단

Exact 13 acceptance·9 IVA·10 required dimensions는 Goal의 matrix를 그대로 소비한다. 추가 registry를 authority로 만들지 않는다.
WP03의 최종 근거는 실제 default-release API/CLI external A/B nonzero effect delta, code/phase/source parity,
replay/readonly/trace-on-off/repeated/delayed/terminal, valid-load failed-prep/retry/reload/cancel/mandatory-output rollback,
정확 cap/cap+1/in-call fuel/queue/entity, generation/drop/새 경로 시간·할당·retained 비용이다.
Source/test/harness/config/lock/toolchain/fixture/command/exit/raw receipts를 결속한다. Canonical 두 계약과 실행 예시도 실제로 검증한다.
최종 integrated core lib/tests, all-target/default-release/public privacy와 현재 V6/profile/pin/R01/R02 영향 검증이 필수다.

정상 실패는 AG에서 교정하고 영향받은 focused 검증을 복원한다. 새 material gameplay/공식 balance/저장 손실/호환·권한 의미,
유일하지 않은 ownership/기록 손실·필수 consumer/reviewer 접근 불가가 있으면 해당 owner와 실제 blocker를 기록한다.
타입·함수·한도 선택은 승인 의미 안에서 실제 workload/경계/비용 근거로 결정하며 별도 사용자 재승인 gate가 아니다.
구현 종료 뒤 stable target을 고정하고 별도 fresh reviewer가 필요한 선행 감사와 R03 GEX-20을 각각 수행한다.
현재 완료 조건은 모두 구현·검증 대기이며 긍정 감사나 R03 완료를 선반영하지 않는다.

## GEX-10 상세 설계 001 — 실제 content 진단 producer

첫 source slice는 LiveRonError와 existing snapshot/schema/raw/resolver/final/compiler/generated producer다.
`LiveRonPhase`의 enum serialization은 snake_case, stable code는 `content_acquisition`, `ron_parse`,
`content_structure`, `content_resolution`, `content_validation`, `content_compile`, `generated_setup`로 시작한다.
Contract/capability mismatch는 `content_contract_unsupported` / `content_capability_unsupported`로 구별한다.
공통 source DTO에는 optional pack_id/catalog_kind/definition_kind, parser error에는 `RonPosition { line, column }`과
typed reason을 둔다. 알려진 expected/actual은 serde_json::Value이며 없으면 None이다. 기존 Display는 그대로다.

`parse_live_ron`에서 typed `ron::de::Error`를 문자열로 낮추기 전에 Parser variant의 position/reason을 보존한다.
Raw parse 호출자는 decoded manifest의 pack_id와 실제 CatalogKind를 덧붙인다. SourceTable의 실제 DefinitionKey
일치 지점에서 kind/pack을 연결하며 related source에도 같은 relocation을 적용한다. Aggregate source는 실제
SourceTable contributor를 DTO로 보존한다. Resolver 오류는 실제 target에서 kind를 취하고, primary file의 pack은
owned snapshot receipt의 exact path equality로만 결속한다. Selection 자체에 임의 단일 pack을 붙이지 않는다.

Default validation constructor는 final_validation이다. Snapshot acquisition·delimiter precheck, manifest structure,
resolver와 실제 summoned/boss compile 호출, GeneratedValidationContext 생성 지점에서 phase/code를 확정한다.
Publication wrapper는 source를 결속할 뿐 producer가 정한 더 세밀한 phase/code를 덮지 않는다.
Source inventory는 generation 소유로 이어질 예정이며 이 slice만으로 facade 지원/완료를 게시하지 않는다.

Focused oracle: malformed typed RON의 actual position vs serde Message의 position None; manifest 이전 pack None;
decoded valid manifest contract/capability expected/actual; raw catalog actual pack/kind; resolver/final/compile/generated phase
구별과 R02 policy-wave/custom-boss primary/related source 보존. Existing fingerprint·Display/runtime semantics 불변을 확인한다.
각 파일은 baseline 사본과 일치하는지 확인한 뒤 수정한다. 새 지침/검증기·공식 RON·public inner authority는 변경하지 않는다.

## GEX-10 상세 설계 002 — immutable source/writer 보존

기존 loader가 이미 생성하는 `SourceTable.0`와 `ResolvedContent.writers`를 성공한 generation에 move한다.
`ContentSourceMetadata`의 private vectors는 actual final `(DefinitionKey, DefinitionSource)` 순서와 기존 WriterReceipt다.
Public accessor는 borrowed immutable slices와 exact key lookup만 제공한다. WriterReceipt field/order/serde 형태와
semantic_bytes의 기존 입력을 유지한다. Raw RON text·두 번째 DB·새 global cache는 보존하지 않는다.
Normal external publication은 Some(metadata), bare fixture database 변환은 None으로 availability를 정직하게 표시한다.
기존 typed source/receipt 생성 알고리즘과 validation owner는 바뀌지 않으며 이전에 임시 생성했던 metadata 수명을 generation에 맞춘다.

이전까지 해당 vectors의 생성 비용은 loader에 있었고 지금 새로 도입하는 비용은 generation별 retention이다.
Authoring entry는 기존 input ceiling 외에 metadata count/bytes를 publication 전에 실제 admission해야 한다.
이번 slice만으로 runtime/metadata 한도 검증 완료를 주장하지 않는다. 동일 source metadata의 count/bytes·drop 비용과
새 source-generation strong reference 보존은 후속 actual authoring/control 통합과 WP03 비용 측정에 포함한다.
Readonly metadata accessor는 source 이동/변경 뒤에도 같은 A 원문 주소를 반환하고, replacement/remove/winner 관계는
원 resolver의 actual receipt와 일치해야 한다. V6/semantic identity 변경은 없음을 현 pack tests와 별도 before/after oracle로 확인한다.

## GEX-10 상세 설계 003 — 요청 단위 실행 제어와 validate entry

현재 동기식 실행의 crate-private `authoring::control` TLS scope를 선택한다. Public API는 closure/scope를 노출하지 않고
`validate(root, selection, limits, cancellation)`만 actual common loader를 호출한다. `AuthoringContent`는 opaque token과
load usage를 소유한다. 이후 session entry도 같은 private scope를 사용한다. 아직 없는 simulate/trace 지원을 advertise하지 않는다.

`ExecutionLimits`는 trusted host config이며 Deserialize하지 않는다. max_work/queue/bucket/retained_entity/definition/metadata_bytes는
양수·hard ceiling 이하로 검증한다. 초기 engineering defaults는 work 10,000,000, queue 8,192, bucket 4,096,
retained entities 4,096, definitions 16,384, metadata 8 MiB이며 각각 hard ceiling 2^30/2^20/2^20/65,536/65,536/64 MiB다.
단위는 논리 work·항목 수·encoded metadata bytes이며 inclusive cap다. 입력·출력과 새 비용의 실제 측정 뒤 최종 설정을 검증한다.
현재 공식 balance/시간 제한이 아니며 final safe bound를 미리 주장하지 않는다.

Scope는 remaining work·각 peak·first failure·현재 phase·read-only atomic cancellation을 소유한다. State는 Core/Battle clone에
들어가지 않아 내부 rollback이 연료나 failure를 되살리지 않는다. Charge/admit는 checked arithmetic 후 작업 전에 수행하고
실패를 latch한다. 기존 GameError 경계에는 explicit InvalidStaticData로 전달하되 authoring outer는 latched control failure를
우선 반환한다. Error precedence는 acceptance 이전에 관찰한 scope failure 우선이다. Domain error와 source를 성공으로 바꾸지 않는다.
Final poll 이후 infallible aggregate swap을 acceptance point로 정하며 그 뒤 도착한 cancel은 과거 성공을 취소하지 않는다.

RAII guard는 정상/Err/unwind에서 TLS를 비우고 phase guard는 이전 phase로 복구한다. Panic을 cancellation 수단으로 사용하지 않는다.
Nested top-level authoring entry는 시작 전에 거부하여 새 budget 설치로 우회하지 못하게 한다. 내부 calls는 같은 scope를 공유하며
RefCell borrow는 helper 안에서만 유지한다. 실행 중 logging callback에서 일반 core를 재진입시키는 것은 같은 thread scope의
accounting을 공유한다; public authoring callback API는 없다. Thread migration/async 실행은 현재 지원하지 않는다.

Private bounded queue wrapper는 기존 heap ordering/peek/pop/iter/clear/clone을 유지하고 push 전에 queue+work를 검사한다.
실패한 push는 sticky failure를 남기며 actual fallible caller가 즉시 poll한다. Bucket transfer도 별도로 제한한다.
Loader phases/generated cases와 startup/event/clock/transaction success paths부터 연결하고, leaf expansion/constructor/entity/expect
경계는 이어서 모두 닫는다. 연결 중 상태는 Goal 미완료이며 host-call 수만으로 in-call safety를 주장하지 않는다.
Optional observation buffer와 work/admission은 분리하고 helper의 trace-overflow rollback 제안은 적용하지 않는다.

## GEX-10 상세 설계 004 — 명령 생성·반복 실행과 source 교정

Effect builder는 실패 가능한 내부 Result로 바꾸고 실제 Instant/area/projectile consumer에서 즉시 반환한다.
효과 목록의 modifier scan 전에 work를 선차감하고, 각 effect와 target/extra-attack 반복 직전에 charge한다.
Effect command vector는 push 전에 `max_effect_commands` inclusive 상한을 검사한다(기본8192, hard2^20).
실패 시 부분 vector는 폐기되며 실제 command executor에 전달되지 않는다. Peak는 vector 항목 수이고 heap byte 측정이 아니다.
Command/batch loop는 sticky failure 이후 다음 명령을 실행하지 않는다. Skill command가 오류를 삼키는 내부 함수를 호출해도
temporary recording source를 원래 값으로 복원한 뒤 poll하고, 바깥 실제 event/application acceptance에서도 실패를 확인한다.
Instant/tile-area repeat는 반복마다 charge한다. 이 연결은 아직 남은 targeting/geometry/deferred/clone/control 범위를 대체하지 않는다.

Snapshot final recheck는 이미 검증된 manifest 소유 pack과 당시 catalog kind를 FileRecord에서 오류로 옮긴다.
재검증 때 바뀐 bytes에서 새 ID를 추론하지 않는다. Resolver aggregate 오류는 aggregate primary의 pack을 비워 두고
실제 writers를 deterministic pack order의 related sources로 보존한다. Writer 수는 이미 selected pack 상한으로 제한된다.
[원 보조 검토](design-evidence/validate_control_review_001.md)의 VCR01/02에 대한 root 교정이며 별도 완료 감사가 아니다.

## GEX-10 상세 설계 005 — geometry와 retained cast records 연결 중

`TileReachQuery`의 검증된-pattern expect에 control error를 주입하지 않는다. BattleCore의 실제 area projection 전
row byte 길이로 가능한 cell 수를 상한 검사하고, validation/anchor scan/projection/sort의 source-sized logical work를
선차감한다. `max_geometry_cells`는 기본65536/hard2^20의 inclusive upper cell count이며 clipped 결과 크기가 아니다.
Area target의 units×tiles membership과 sort도 실제 후보 크기로 선차감한다. Accepted gameplay 알고리즘·순서·ID는 유지한다.

Skill cast/projectile/persistent-area의 ID 할당 전에 현재 BattleCore materialized object/record 수와 실제 추가량을 검사한다.
포함한 root maps 및 cast의 progress/deferred/impact/last/area IDs, projectile/area hit vectors를 합산하며, HashMap 열거 순서가
resource failure requested 값에 영향을 주지 않도록 합계를 낸 뒤 admission한다. 이 count는 spawn/boss/backend/모든 transient
buffer 또는 allocator bytes 전체를 포괄하지 않으며, 나머지 범위는 여전히 구현·측정 의무다.
Impact context 교체는 제거될 old vectors와 새 map/last vector 두 개를 구별한다. 같은 key 교체를 영구 누적 growth로 세지 않는다.
Clone CPU와 transient byte peak는 별도 항목이며 이 retained-record count만으로 닫히지 않는다.

Deferred scan/release/cleanup, cast child entry, prior context copy, target candidate scan과 projectile repeat를 연결한다.
실패는 sticky scope에 남기고 direct dispatcher는 성공 개수로 해석하기 전에 poll한다. Optional trace는 이 admission에 들어가지 않는다.
[좁은 leaf 보조](design-evidence/skill_leaf_review_001.md) 원문 전체를 읽고 ID allocator 보존·실제 producer admission과
replacement/detached-state/expect/후속 poll 제안을 소비한다. 아직 미연결된 fixed motion/area tick/history/clone/usefulness 경로는
보고서의 남은 항목으로 유지하며 complete라고 표기하지 않는다.


## GEX-10 상세 설계 006 — bounded readonly query 구현 단위

Common describe는 current CatalogKind/DefinitionKind macro와 EFFECT_SUPPORT의 실제 값, 같은 token의 identity/source receipt/final inventory를 반환한다.
현재 editable descriptor의 Instant Damage/Heal 범위를 확대하지 않는다. Query는 immutable token을 읽으며 source 파일을 다시 열지 않는다.
Definition explain은 actual final source와 기존 writer receipt를 설명한다. Remove/nonwinning의 원 payload/정밀 위치는 현재 metadata에 없으므로 추론하지 않는다.
Effect explain은 compiled SkillDef의 exact action/effect ID, source()·origin·delivery/target context와 실제 editable support를 읽는다.
Compiled graph와 readonly public Preview는 이어서 구현하며 이 첫 query 단위의 지원 목록에 미리 넣지 않는다.

Query 결과는 private capped JSON writer가 각 chunk를 추가하기 전에 mandatory result byte admission과 work를 검사하여 확정한다.
Trusted ExecutionLimits.max_result_bytes는 기본8MiB/hard64MiB, inclusive이며 result_bytes_peak는 실제 encoded bytes다.
결과는 owned bounded JSON과 usage를 반환하여 generation 전체를 추가 보존하지 않는다. 이 usage는 query document 외부의 측정값이다.
Session receipt는 별도의 누적 mandatory 상한과 fixed header reservation을 설계한 뒤 연결하며 optional trace에는 result/work 예산을 적용하지 않는다.
공개 query는 private TLS scope 안에서 호출하고 first execution failure가 중간 encoding/domain 오류보다 우선한다.
부재 정의/효과와 알려지지 않은 query는 명시 request 오류이며 fake forecast/빈 성공을 반환하지 않는다.


Readonly definition 후속 설계는 [explain 보조002](design-evidence/explain_port_design_002.md)의 actual42-kind lookup을 사용한다.
Final membership을 먼저 확인한 exhaustive match로 같은 DB의 resolved typed record를 bounded writer에 빌려 준다.
선택 Remove는 present=false/source/value=null과 실제 writers를 반환하며 존재하지 않는 key와 구별한다.
Compiled Pattern/Actor/Encounter는 compiled index 공간으로 표시하고 Actor→Pattern/Encounter→Actor는 정의 ID 참조로 내보낸다.
Array index에서 authored child path를 조립하지 않는다. Pattern의 non-Arc leaf 타입만 Serialize를 추가하며 재귀 Arc graph를 serialize하지 않는다.
SummonedUnit은 실제 compiled combat profile, Signal은 실제 선언 ID만 설명한다. Effect origin은 실제 origin definition의 current source까지만 제공한다.


## GEX-10 상세 설계 007 — 유한 scenario grammar 구현 단위

TrialScenario는 explicit player_id/run_seed/DefaultMemory profile와 preparation_steps/steps를 private ownership으로 보존한다.
공개 JSON 생성자는 raw byte 상한을 먼저 검사한 뒤 duplicate object key·depth·node count를 제한하는 DeserializeSeed로 읽는다.
Strict authoring envelope의 Command/Snapshot/Clock/Assert만 받으며 내부 command semantic validation은 기존 ApplicationCommand decoder가 소유한다.
Command/context template에 prior public result의 bounded JSON pointer 값을 복사하는 grammar만 제공한다. Forward/assert-result/미존재 경로,
잘못된 escape, noncanonical array index, duplicate/ancestor destination와 command type 교체는 명시 거절한다. 표현식/loop/자동 선택은 없다.
Raw byte·shape·step/binding·pointer/template·clock 합계의 trusted ceiling을 기록한다. Binding 후 command는 기존 public decoder를,
context는 실제 ApplicationExecutionContext를 만드는 strict private DTO를 사용한다(기존 context Deserialize는 없다).
이 grammar 단위는 실행 성공이나 battle preparation을 반환하지 않는다. 실제 opaque trial transaction/receipt와 연결한 뒤 simulate를 공개한다.

## GEX-10 상세 설계 008 — bounded result binding과 post-binding decode

TrialScenario는 parser에 사용된 trusted ScenarioLimits를 함께 소유한다. 이후 resolver는 이 checked owner와 현재까지 승인된
public result slot만 읽고 원 template/result/누적 quota를 변경하지 않는다. 실패·Assert·HostTick None은 binding source를 만들지 않는다.
Source pointer는 실제 container에서 canonical array index와 존재 여부를 검사하고 선언 JSON type을 확인한다.
Decoded destination trie는 source Value를 빌린다. 치환될 전체 request를 virtual serialization하여 bytes/nodes/depth를 먼저 검사한다.
그 전에는 source subtree나 command/context를 복사하지 않는다. 실제 materialization 및 기존 command decoder의 내부 clone은
검사된 shape 크기로 work를 선차감한다. 이 수치는 논리 work/encoded bytes이며 allocator byte 상한 증명이 아니다.

Engineering 기본/hard ceiling은 resolved request1MiB, copied source1MiB/step 및8MiB/session이다. 모두 positive/inclusive이며
max_depth/max_nodes는 resolved command/context root에도 적용한다. 이전 누적 값에 새 복사량을 더한 후보 값만 반환하고
session은 mandatory receipt 준비와 final poll 뒤 aggregate commit에서만 이를 채택한다. 더 좁은 caller 설정을 defaults로 되돌리지 않는다.
Context DTO는 unknown fields를 outer/authority 양쪽에서 거절하고 UUID/revision을 typed decode한다. 빠진 seq는 기존 default0,
명시 null seq는 오류이며 nullable source_command_id/authority는 None이다. Command decoder의 actual ApplicationError를 보존한다.
이 단위는 private resolver의 구현·focused 검증이며 complete application fork/step acceptance/receipt/trace를 대신하지 않는다.

Assert는 같은 prior-public source traversal을 소비한다. Source step의 결과 자체가 없으면 Exists(false)라도 오류이며,
실제 result 안의 pointer 부재는 Exists(false)로 검사할 수 있다. Equals(null)는 present null만 통과한다.
Equals는 양쪽 shape/bytes와 비교 work를 검사한 후 borrowed JSON equality를 사용한다. Exists/Type은 전체 subtree를 복사하지 않는다.
작은 exists/value_type observation만 mandatory receipt용으로 반환하며 Assert 결과를 후속 binding source로 게시하지 않는다.

## GEX-10 상세 설계 009 — mandatory receipt의 고정 길이 usage header

Receipt는 bounded writer에서 usage header와 payload를 함께 만든다. Usage의 nonnegative integer는 JSON 숫자를 유지하고
20자리 공간 안에서 뒤쪽 공백으로 padding한다. 따라서 counter 값 변화가 header 길이를 바꾸지 않는다.
처음 zero placeholder를 실제 result bytes로 admission하고 payload/닫는 괄호까지 encoding한 뒤 header 재작성 비용을 선차감한다.
그 시점의 usage snapshot으로 같은 길이의 prefix만 덮어쓴다. Result byte peak는 attempt 전체의 실제 encoded byte 최대값이다.
앞선 더 큰 출력이 없으면 완성 receipt 길이와 같으며, 재귀적인 usage 재계산은 없다.
Header/schema 오류·per-result byte/work/cancel·누적 receipt byte 실패는 모두 candidate 반환 전에 거절한다.
누적 ceiling은 trusted positive/hard64MiB이며 old bytes를 바꾸지 않고 새로운 total만 반환한다. Optional trace는 이 writer의 입력/예산에 넣지 않는다.
Session caller의 최종 receipt는 마지막 mandatory 작업이며 outer scope의 final poll 뒤 infallible aggregate commit만 수행한다. 준비 중 intermediate receipt는 상세 설계025의 scope cumulative cut을 따른다.
현재 private writer focused 구현이며 receipt payload provenance/schema·actual session acceptance·trace buffer는 이어서 연결한다.

## GEX-10 상세 설계 010 — Rapier clone의 structural storage admission

[backend 조사005](design-evidence/backend_clone_design_005.md)의 pinned Rapier0.32/Parry0.26/Hashbrown0.16 source 근거를 사용한다.
Root의 offline cargo feature tree에서도 hashbrown/default 경로를 확인했다. Wrapper의 기존 생성·삭제·refresh·clear에 private copied history를
연결하여 body/collider occupancy peak, insertion/removal totals와 broad-phase event count를 보존한다. Ordinary 사용 때도 기록해
뒤늦게 authoring scope를 설치한 clone에서 과거 capacity를 잃지 않는다. Clear가 backend owners를 교체할 때만 history를 reset한다.

Arena slots는 max(4,next_power_of_two(peak)), body modified queue는 insert total, collider pending은 insert+remove total의 상한을 쓴다.
BVH의 current nodes/parents·leaf-map·refit/rebuild workspace는 source의 정상 return invariant에 결속한5×collider peak+arena slots다.
Pair table은 누적 refresh event P의 source-backed hashbrown bucket 상한을 사용한다. Public wrapper map의 current capacity는
bucket 상한이 아니므로 [std source 후속001](design-evidence/std_hashmap_bound_001.md)의 lifetime insertion count를 별도로 유지한다.
Backend clear에서도 그 두 map의 allocation은 남으므로 map insertion count는 reset하지 않고 같은 checked bucket 식을 적용한다.
카운터/산술 overflow는 sentinel로 보존하여 controlled preflight가 실패하며 ordinary gameplay를 실패시키거나0으로 wrap하지 않는다.
이 수치는 structural records/native storage의 논리 단위이며 private Arena entry bytes·allocator/RSS를 정확히 측정했다고 주장하지 않는다.

새 trusted max_clone_slots는 기본65536/hard2^20, clone_slots_peak는 검사한 합산 storage units다. 복제 전 slot admission과 work를
검사하고 기존 derived Clone 의미를 유지한다. 첫 연결은 실제 boss-boundary BattleCore clone의 backend 부분이다. Whole GameCore/entry/checkpoint,
다른 BattleCore/ActiveBattleSession 복제·history COW·모든 owned payload를 포괄하는 공통 visitor는 이어서 연결한다.
Optional trace storage는 mandatory clone allowance에 더하지 않는다. Actual source/candidate coexistence와 비용 측정은 후속 전체 clone owner 의무다.

## GEX-10 상세 설계 011 — 후보 로그의 쓰기 준비와 COW 비용

BattleEventEntries의 기존 Arc/derived Clone과 infallible record_event_log의 실제 seq 반환은 유지한다.
Controlled candidate를 복제한 직후, 해당 log의 첫 쓰기 전에 fallible prepare_authoring_write를 호출하여 공유 Vec를 분리한다.
No-scope 호출은 즉시 반환한다. Shared 상태에서는 retained entry 수를 먼저 검사하고 기존 exhaustive event payload shape에서
owned string bytes·nested vector elements와 serde-skipped source Arc handles를 세어 native 복사 전에 slot/work를 검사한다.
Source Arc pointee는 다시 복제하지 않는다. Vec/String clone의 실제 live 길이를 사용하며 HashMap capacity 식은 적용하지 않는다.
그 뒤 Arc::make_mut와 poll을 수행한다. 실패 전에는 원본 log와 공유 identity가 보존되고, 복사 뒤 취소는 후보 owner가 폐기한다.
Log accounting fields·retention 정책·이벤트 순번·ordinary caller의 COW 시점은 바꾸지 않는다.

Eager preparation은 해당 candidate가 로그를 쓰지 않고 끝나도 실제 복사 비용을 부담한다. 이는 첫 mutation까지 failure 전달을 미루지 않는
명시적 후보 준비 비용이며 readonly/trace flag와의 실제 parity·비용은 Session 검증에서 확인한다. 기존 application/tactical/boss/summon/spawn
복제 및 생성 이후 경로에 연결할 지점은 source 보조 조사와 함께 닫는다. 이 slice만으로 모든 로그 mutation/whole native clone을 닫았다고 하지 않는다.

## GEX-10 상세 설계 012 — private owned map의 복제 이력

[owned-map 조사007](design-evidence/owned_map_clone_design_007.md)에 따라 private OwnedHashMap은 기존 std map 하나와
copied allocation credit을 소유한다. Empty 생성은0, insert/vacant Entry는 가능한 insertion을1씩 기록하며 clear/remove는 이력을 유지한다.
Immutable Deref만 허용하고 mutable map borrow를 밖으로 내보내지 않는다. Native Clone은 bucket/hasher/iteration을 그대로 보존한다.
Equality·Debug·Serialize는 inner map에만 delegate하여 gameplay 분기와 wire shape에 이력을 노출하지 않는다.
Pinned serde_core1.0.228의 fresh HashMap Deserialize는 reserve+insert만 하므로 그 직후 capacity로 allocation credit을 초기화한다.
임의 기존 map import나 tombstone이 있는 map의 현재 capacity를 증거로 삼지 않는다. clone_from은 기존 destination allocation을 재사용하며
source credit으로 덮어쓰지 않고 trait의 whole replacement 기본 경로를 유지한다.

첫 전환은 공개 Inventory/EmployeeRoster의 private maps다. Public owner methods와 concrete DTO/save shape는 유지한다.
후속 private world/battle maps와 HashSets를 같은 원칙으로 연결한다. 공개 fixture BattleCore.units의 raw HashMap 계약은 유지하고
application 내부 생성·삽입 경로의 sidecar로 다루며 외부 임의 core/map import를 trial provenance로 받지 않는다.
Pure history와 checked bucket 상한은 TLS를 소유하지 않는다. 실제 clone visitor는 각 독립 owner의 retained bucket scan과 payload copy를
복제 전에 admission한다. 아직 연결되지 않은 visitor를 구현된 것처럼 표시하지 않는다.

## GEX-10 상세 설계 013 — 집합의 native storage와 유입 경계

HashSet도 pinned std→hashbrown의 native table Clone을 유지한다. 생성 뒤 insert/clear만 하고 외부 임의 값이 유입되지 않는
owner는 tombstone-free capacity를 bucket envelope의 credit으로 쓸 수 있다. 이는 임의 HashMap/Set의 현재 capacity를
상한으로 쓰는 것과 다르며, 실제 mutation/capture/deserialize 경로의 증명이 있어야 한다. 새 remove/retain/외부 import가
추가되면 이 증명을 다시 확인하거나 owner history로 전환해야 한다.
BattlefieldLayout.static_obstacles는 private 저장소이지만 public remove API가 있어 copied sidecar 이력을 사용한다.
Valid tile set은 생성 뒤 불변이며 공개 메서드는 borrowed raw mutation을 제공하지 않는다. 기존 ordinary/native Clone과
공개 함수/정렬 결과를 유지한다. Clear 뒤 static history를 reset하지 않는다: native clear는 이미 빈 table에서 조기 반환할 수 있다.
Public VN consumed sets 등은 concrete 타입을 바꾸지 않고 future fresh-only Session의 실제 유입 경계와 함께 방문한다.

## GEX-10 상세 설계 014 — 한 복제에 대한 공통 합산기

Private CloneMeter는 native storage slots와 owned string bytes의 checked/saturating 합만 소유하며 gameplay state·TLS budget·
실제 원본 reference를 보존하지 않는다. Unknown native bound는 sentinel로 남겨 final storage admission에서 거절한다.
Container를 순회하기 전 해당 storage 상한과 scan work를 검사하고, 모든 child의 합을 actual Clone 전에 다시 admission/charge한다.
Source를 복제해 크기를 알아내거나 Arc definition pointee를 재귀 serialization하는 방식은 쓰지 않는다.
첫 소비자는 이미 검증된 log COW 및 Rapier preflight다. 기존 비용·실패 site·ordinary 동작은 유지하면서 같은 meter에
후속 whole-owner 방문 결과를 합산할 수 있게 한다. Whole GameCore/BattleCore 방문과 모든 actual clone seam은 아직 후속 구현이다.

## GEX-10 상세 설계 015 — borrowed native owner와 닫힌 leaf 방문

Private CloneShape는 scalar/String/Vec/Option/Box/Arc와 실제 native collection을 구분한다. Arc는 pointee trait 없이 handle만 방문한다.
OwnedHashMap은 retained history로 table 전체 scan을 먼저 admission하고 key/value children을 빌려 방문한다. BTree의 정상 완료된
관찰 상태는 Rust1.91.1의 non-root 최소5 keys/root 최소1 invariant로 node 수를 상한화한다. Node당 최대37개 typed field storage를
40으로 보수적으로 세고 outer record1을 더한다. Empty source가 마지막 leaf를 보유해도 native Clone은 empty root=None이므로0 nodes다.
이는 native logical slots이며 allocator bytes/RSS·panic 뒤 임의 중간 상태 인증이 아니다. 실제 key/value child는 별도 합산한다.

Struct owner macro는 모든 direct field를 wildcard 없이 destructure하여 새 필드 추가 시 방문 누락이 compile 오류가 되게 한다.
Inventory3/roster의 native maps와 shared metadata, Employee의 독립 owned fields, Uuid counters·profile BTree·fragment inventory를 연결한다.
GrowthStack의 public raw map은 opaque production constructor/fresh serde/add/complete restore만 통과한 유입에서 tombstone-free proof를 쓴다.
새 delete/raw import가 생기면 history 전환이 필요하며 arbitrary external core/Employee/GrowthStack import는 이 증명에 포함하지 않는다.

Explicit whitelist leaf만 borrowed Serialize 방문을 허용한다. UnitCombatProfile·직원 profile/definition/condition/modifier·심리·item slot·
fragment loadout의 String/Vec/fixed records는 source 검토된 닫힌 graph다. Blanket Serialize trait impl은 없으며 native map과
allocating collect_str 및 길이를 모르는 sequence는 거절한다. Container cardinality/storage와 scan work를 순회 전에 검사하며
String은 O(1) len으로 copy bytes만 합산한다. Field/type 이름은 owned 문자열로 만들지 않고 출력 JSON/DTO를 임시 생성하지 않는다.
Leaf serializer는 wire encoder가 아니므로 schema field/scalar의 보수적인 logical units를 포함하며 정확 allocator byte 수로 설명하지 않는다.
전체 합산 후 finish가 actual native Clone 전 copy work를 검사한다. 현재 owner visitor slice이며 whole fork/entry/checkpoint/trace 연결은 남아 있다.

## GEX-10 상세 설계 016 — runtime 소유 필드와 enum 분기 전수 방문

Struct visitor에 실제 cfg field를 지원하고 enum/newtype macro도 모든 분기와 필드를 wildcard 없이 방문한다.
Fixed record의 단위 계수는 concrete Copy bound로 닫아 heap-owning 필드를 추가하면서 조용히 scalar로 처리하지 못하게 한다.
RuntimeUnit의 pending skill/이동 경로/프로필, projectile·area source/hit history, active cast의 각 BTree와 독립 impact context,
private spawn request/trigger/settlement/compendium owners를 직접 방문한다. DamageSourceSnapshot의 serde되지 않는 source Arc와
bonus vector를 분리하며 ActiveBuff는 실제 scalar runtime fields만 방문한다. DB/compiled definition은 소유하지 않는 곳에서 조회하지 않는다.
이 단계는 통합 visitor의 타입 경계를 만드는 작업이다. 신규 Session, 모든 whole owner와 실제 clone call의 admission은 별도로 연결한다.

## GEX-10 상세 설계 017 — BattleCore 전체 소유 합산

BattleCore는 모든 direct field를 destructure하고 scenario·queue·unit/runtime maps·boss/summon/wave·fieldwork·result stats·
backend·battlefield·recording stacks를 같은 meter로 방문한다. GameData/compiled boss definition/argument frame/log entries는
actual Arc Clone의 handle만 합산한다. Log COW 쓰기 준비는 후보를 실제 clone한 뒤 별도의 복사로 admission한다.
Unit raw map은 기존 private sidecar의 complete application provenance로만 방문하고, static obstacle은 삭제 이력을 사용한다.
Boss latch/cleared batch와 spawned group sets는 조사009의 fresh/insert-only 유입에 한해 native capacity envelope를 쓴다.
HashSet에 blanket trait를 추가하지 않는다. Set serde는 sequence여서 native bucket 증명을 대신하지 못한다.

전투 종료 신호·실행 상태와 소환 request/view/batch, BTree tuple key/values도 개별 복사 소유로 합산한다.
O(1) backend history는 복제 storage를 추가하며 실제 backend traversal을 측정 목적으로 실행하지 않는다.
Whole preflight는 합산 후 actual clone 전에 storage/copy work를 검사하고 무제어 ordinary 호출에서는 즉시 반환한다.
첫 연결 대상은 실제 boss/summon/spawn의 BattleCore 복제이며 GameCore/ActiveBattleSession/entry/checkpoint는 별도 후속 owner다.
다른 outer root에 있는 Monolith의 삭제 가능한 public map 등 새로 확인된 storage 경계는 이 BattleCore slice에 포함하지 않는다.

## GEX-10 상세 설계 018 — complete world 방문과 후보별 native provenance

GameCore→GameCoreState의 모든 소유 root와 ActiveBattleSession/BattleEntrySnapshot/RunCheckpointPayload의 독립 복사를
exhaustive field 방문으로 연결한다. Checkpoint macro는 실제 선언 필드를 같은 방문기에 공급하며 skipped encoded payload는
Arc handle로, CombatPreview의 skipped enemy_stat_scale은 native field로 방문한다. Pending sweep의 grant state·progression·
completion map도 현재 live run과 별도로 합산한다. 공개 타입/serde 형태와 ordinary native Clone은 유지한다.
Reviewed closed String/Vec/fixed world leaves, native BTree 및 source-proven shop/VN/attempt sets를 구별한다.
Recruitment VecDeque는 [013](design-evidence/r03_vecdeque_clone_design_013.md)의 pinned native Clone len allocation과 borrowed
front-to-back iterator를 사용한다. Source capacity/wrap/clone_from 또는 allocator/RSS 총량과 혼동하지 않는다.

Monolith raw HashMap은 실제 삭제/drain을 거치므로 [012](design-evidence/r03_monolith_provenance_design_012.md)의
private fixed Copy NativeStorageProvenance를 candidate scope에 값으로 설치한다. Generic run은 unknown(None)을 유지한다.
새 opaque construction family의 fresh()만 known-zero를 만들며 임의 public raw Core/RunState import를 인증하지 않는다.
Actual begin_remanifestation insert 직전 credit1, fresh typed V6 import의 검증 성공 직후/첫 clone 전에 capacity max 병합을
기록한다. Native clone은 credit을 늘리지 않고 내부 rollback/restore/drain은 credit을 줄이지 않는다. 각 독립 Monolith table에
동일한 보수 bucket bound를 각각 합산한다. Unknown/overflow는 복사 전 거절하며 current len/capacity로 대체하지 않는다.

run_with_provenance는 candidate ledger를 반환할 뿐 커밋하지 않는다. T=domain Result의 Err도 outer Ok일 수 있으므로
후속 TrialSession은 실제 domain 성공·mandatory receipt 준비·최종 poll 뒤 app+ledger+결과를 한 aggregate로 교체해야 한다.
Failed B/step은 candidate ledger를 버리고 A를 보존한다. V6 wire/hash·게임 RNG·optional trace·fuel에 이 ledger를 저장하지 않는다.
이 단계에서는 storage history와 copy admission을 연결하며 아직 없는 public Session commit을 완료로 간주하지 않는다.

controlled_clone은 opt-in에서 entry poll→borrowed whole visit→finish→native clone→post poll 순서다. 실패 시 local copy를
폐기하고 enclosing transaction은 caller가 소유한다. 실제 GameCore application staging·ActiveBattleSession tactical staging과
crate-private opaque CoreApplication fork에 연결한다. Boss plan_next의 owned candidate와 BuffDef3개도 같은 helper를 사용하며
boss loop의 read-only 중간 runtime clone은 borrowed 접근으로 제거한다. AutoCastStart는 기존 priced skill-definition helper를 쓴다.
Startup/capture별 직접 복사와 spawn normalization/settlement/소환후 복사, DTO projection/queue 비교 work는 별도 후속 의무다.

## GEX-10 상세 설계 019 — projected capture 복사 전 admission

전체 clone 방문만으로 capture constructor의 선행 필드 복사를 허용하지 않는다. Private capture_shape macro는 각 source를
borrow한 같은 필드 목록으로 방문과 native field Clone을 생성하고 complete Self construction으로 누락 필드를 compile 오류로 닫는다.
Set 필드는 별도 provenance 방문, fresh None/default-empty fields는 실제 독립 복사가 없는 명시 생성으로 구분한다.
Fresh 값도 완성 candidate의 고정 저장 단위를 가진다. Entry poll 뒤 heap allocation0인 현재 fresh 값을 한 번 생성하고
그 자체를 방문한 후 move한다. Fresh empty set은 별도 fresh_sets 목록으로 native record1을 포함한다. 원본 profile은 방문하지 않는다.
동적 factory/기존 소유 값을 fresh로 넘기는 것은 허용하지 않는다. Capture와 완성 owner의 독립 CloneShape 비용을 비교한다.
Entry poll→projected meter sum/finish→기존 native fields 복사→post poll을 통해 한도/취소는 Result로 전달하고 local candidate를 버린다.
Lifecycle phase는 snapshot()이 아니라 phase()를 빌려 첫 문자열 복사 전 검사한다. Capture source의 다른 retained root는 복제하지 않는다.

BattleEntrySnapshot/RunCheckpointPayload/EconomyGrantState/BattleEmployeeAttemptState와 non-battle transaction capture는
외부 public export가 없는 world private owner다. Fallible 반환을 실제 production Result caller까지 전파하며 cfg(test) fixture 호출도
성공 unwrap을 명시한다. Ineligible non-battle capture는 Ok(None), control 실패는 Err로 구별한다. 실패를 None/InvalidAction으로
바꾸거나 capture 실패 때 부분 상태를 반환하지 않는다. 기존 rollback은 live profile을 move-preserve하여 sticky failure 뒤 새 복사
admission이 복원을 가로막지 않게 한다. Public Session의 complete outer rollback/atomic acceptance는 여전히 별도 owner다.

## 상세 설계020 · 스폰 정규화·예약·반환 복사의 제어 실패

Private normalization은 outer SpawnKernelError와 inner domain settlement를 분리한다. Identity fallback은 inner rejection에만
원래 owned request를 move하며 control failure를 원본 clone/거절 결과로 바꾸지 않는다. ID trim 전 byte work를 admission하고
정규화 candidate 및 request key·trigger ID·캐시/최종 settlement의 actual owned copy에 controlled_clone을 연결한다.
Settlement getter는 private borrow로 바꾸고 반환 소유자에서 검사한다. 예약 전에 가능한 trigger/request ID 복사를 마치며
원 요청을 move하여 reservation용 중복 전체 복사를 줄인다. Pending reservation은 기존 snapshot 이전 위치를 유지한다.

배치 mutation 후 최종 settlement copy는 성공/게임 거절 모두 rollback snapshot이 살아 있는 동안 수행한다. Control failure면
원 snapshot으로 복원하여 Pending만 유지하고 ExecutionInterrupted를 반환한다. Domain rejection이면 복제 성공 뒤 원 snapshot을
복원한 후 기존과 같이 terminal rejection을 저장한다. 실제 callback 중단·최종 copy의 one-under 실패·재시도·cached 반환을 검사한다.
이 전환은 transient sort/BTree 비교/성장·호출자 인수 생성·compendium/constructor 전체 work bounds를 닫았다는 주장이 아니다.

## 상세 설계021 · 보스의 임시 compiled/active owner 복사 제거

읽기 [boss015](design-evidence/r03_boss_inner_copy_design_015.md)/[5-source·12-read](design-evidence/r03_boss_inner_copy_design_015.json)를
전체 읽었다. Drift0의 당시 source 조사이며 root 후속 변경의 완료 감사가 아니다. Phase와 control node는 local immutable Arc에서
borrow하고 selected phase transition은 timing/target scalar만 추출한다. Active pattern의 tasks/maps/set 전체 clone 두 곳은
local definition/frame Arc와 scalar activation으로 대체하여 state 취소 후에도 필요한 immutable owner가 살아 있게 한다.
Winner tie 순서·ContinueState condition 소비·outcome route 비교·node 검증 순서를 유지한다. Target query와 route cells는
실제 immutable owner의 borrow를 반환한다. Route lifetime은 core에만 묶는다.

Active summon 확인은 view Vec 생성/복사/정렬 대신 기존 eligibility와 여섯 identity를 borrowed 순회로 비교한다. Retained view
생성은 여전히 별도 projection 의무다. Detached pending은 controlled_clone하고 그 local view는 borrow한다. Outer boss boundary의
staged core가 complete rollback owner이며 inner summon 함수의 일부 before만으로 모든 Err 복원을 주장하지 않는다.

## 상세 설계022 · retained summon view 목록의 전체 선행 projection

일곱 typed source reference를 가진 private projection의 단일 field-list가 visit_owned_shape와 into_owned를 생성한다. Projection
자체의 native clone shape로 오인하지 않으며 완성 BossSummonTaskView literal의 exhaustive field/type check를 유지한다. 기존
owned view/Vec CloneShape는 독립 비용 oracle로 남긴다. 같은 allocation-free traversal callback을 두 번 사용한다. 첫 pass는
actor/task마다 두 순회의2 work를 선계수하고 Vec root1 + selected element1 + view owned shape를 하나의 meter에 누적한다.
Full finish 성공 뒤 with_capacity(selected_count)와 첫 native field copy를 실행한다. 두 번째 pass의 actor/task poll과 마지막 poll
실패는 local vector를 폐기한다. 선택 항목0이어도 원본 actor/task 순회 비용을 빠뜨리지 않는다.

활성 scope의 정렬은 stable insertion sort로 각 String 비교 전에 min(byte lengths)+1 work를 charge하고 실패 즉시 Result로
종료한다. Equal key는 이동하지 않아 기존 stable lexical 순서를 유지한다. Swap은 owned value move이며 새 clone/할당이 아니다.
No-scope는 기존 한 순회 Vec::new/push와 native stable sort를 유지한다. 임의 native sort 상수/복잡도 multiplier를 쓰지 않는다.
Retained field 복사와 순회/정렬 work는 함께 관리하되 allocator bytes나 모든 task/constructor bounds 완료로 확대하지 않는다.

## 상세 설계023 · 실제 private trial 단계의 aggregate transaction

기존003 제안과 현재 complete fork/resolver/receipt를 연결하는 private trial.rs를 먼저 구현한다. Initialize는 fresh native provenance
scope 안에서 CoreApplication::new와 실제 DefaultMemory PlayerProfile snapshot을 만들고 initial mandatory receipt를 준비한다.
아직 없는 actual live-entry/timezero/terminal marker를 추론하지 않으며 public prepare/simulate export는 그 생산자 전환 후에 연결한다.

각 Command/Snapshot/Clock/Assert는 같은 opaque fork를 사용한다. Resolved command document는 execute가 소비하기 전에 bounded
writer와 미리 계수한 JSON parse로 보존하고 context Value는 move한다. Public result slot은 전체 public wrapper이며 HostTick None과
Assert는 index를 차지하는 None 슬롯을 남긴다. Slot reference Vec와 immutable accepted-record Arc Vec는 복사 전 admission한다.
Result/receipt 원문은 immutable Arc 뒤에 두어 과거 JSON 전체를 다시 복제하지 않는다. 새 App·record Vec·copy bytes·receipt bytes를
모두 준비한 뒤 scope의 final poll이 성공하고 inner domain Result도 Ok일 때만 candidate provenance와 함께 하나의 aggregate를 바꾼다.

Trusted TrialLimits의 누적 work/receipt 상한을 검증하며, 원 per-attempt ExecutionLimits를 먼저 검증한 후 남은 session work로
이번 max_work를 제한한다. Receipt payload에는 prior_accepted_work를, 단일 step의 최종 usage header에는 final attempt usage를 기록한다. 준비 중 intermediate header의 누적 기준은 상세 설계025를 따른다.
최종 scope 반환 뒤 합은 remaining bound 안의 infallible 정수 연산이고 새 serialization/admission을 하지 않는다. 실패한 단계는
원 application·prefix·budget·receipt와 provenance를 보존한다. Preparation prefix의 단일 outer scope·actual marker·A/B swap과
optional observer/CLI는 이 내부 실행 경계를 실제로 소비하는 후속 단위다. 기존 결과 생성자의 나머지 복사/순회 bounds도 남아 있다.

## 상세 설계024 · 실제 live battle lifecycle witness와 elapsed 소비

Private GameCore::new_for_authoring만 Copy scalar sidecar를 활성화한다. Ordinary new/loader generated 실행은 None이다.
실제 commit_planned_live_battle_start가 checked entry ordinal과 UUID를 기록하고, 실제 zero advance 성공 뒤에만 time-zero 완료를
기록한다. Retreat는 UUID manager를 되돌리므로 entry ordinal은 실제 producer 진입을 구별하며 gameplay UUID/RNG를 바꾸지 않는다.
실제 advance의 execution pre/post 시간 차이를 checked 누적하고 terminal transition은 active 제거 전에 실제 verdict/time을 보존한다.
Pending/Running/Finished/Exited와 timezero bool은 fixed owned 값이다. String outcome ID나 log Arc는 이 witness에 복사하지 않는다.

PreparedApplicationCommit은 새 sidecar도 함께 이동하고 with_state_transaction 오류는 Copy snapshot으로 복원한다. Application
operation의 성공 뒤 실제 active/lifecycle root와 sidecar를 대조하여 retreat·reset·checkpoint/root 교체의 exit를 표시한다. 이 대조는
이미 producer가 만든 entry의 무효화만 소유하며 새 entry/readiness를 추론하지 않는다. 내부 overflow/시간 역행/identity 불일치는
명시 오류로 버리고 saturating/UUID만 재인식하지 않는다. Trial step은 mandatory receipt 전에 actual elapsed 상한을 검사하며 witness를
receipt에 직렬화한다. 초기 준비를 완료했다고 공개하지 않으며 full terminal source capsule/semantic occurrence/optional trace는 별도다.

INPUTS065 실제 진단에서 immediate terminal의 inner handler는 성공하지만 execute_behavior_draft의 후속 VN/capability 재투영이
active 제거 후 실패하는 기존 문제를 확인했다. INPUTS066은 finished BattleAdvanced만 그 중복 투영을 생략하고 actual advance가
terminal 제거 전에 만든 결과를 반환한다. TimeReached threshold를 terminal 실제 시각으로 가정하지 않으며 witness는 actual execution
pre/post와 public update 시각에 대조한다. 게임 종료 조건/시간 stepping 규칙은 변경하지 않는다.

## 상세 설계025 · 단일 scope의 전체 preparation과 private replacement

Initialize의 scope 내부 본문만 추출하여 initial→명시 preparation_steps 전체→actual is_ready→별도 preparation_ready receipt를
한 run_with_provenance 안에서 실행한다. Intermediate headers는 각 encoding 시점까지 scope_cumulative usage이며 합하지 않는다.
준비 동안 prior_accepted_work=0/ledger placeholder는 외부 accepted owner가 아니고 실제 ledger는 같은 TLS에 단조 누적된다.
마지막 ready header만 최종 scope usage와 같으며 outer/inner 성공 뒤 전체 usage/ledger를 한 번 채택한다. 이후 한 단계 advance는
기존 old accepted work + 새 scope usage를 유지한다. Ready receipt는 source index를 차지하지 않는 별도 Option<Arc>로 보존한다.

새 private B prepare는 fresh family이고 마지막 명시 prefix가 Exited/never entered이면 실패한다. B 준비 전체와 final mandatory output/
poll 성공 뒤에만 A aggregate를 교체한다. Prefix를 자동 확장하거나 중간 ready에서 조기 종료하지 않는다. Public Session/실패 관찰
surface/semantic occurrence·terminal source/optional trace·CLI와 첫 public projection bounds는 별도 미완료다.

## 상세 설계026 · 최초 public JSON projection 전 counting admission

새 crate-private projection helper는 active scope일 때만 buffer 없는 Serialize writer로 실제 serialized bytes를 검사·계수한다.
전체 native Value 변환 비용을 그 bytes로 선계수한 뒤 기존 serde_json::to_value를 실행하고 postpoll한다. Ordinary는 기존 to_value를
바로 호출한다. Encode/parse 왕복은 Number/깊이 의미가 native와 다를 수 있어 사용하지 않는다. 실제 source DTO의 순수 Serialize를
두 번 방문하는 비용이며 DTO 생성 앞의 native clone/collection scan까지 닫았다는 주장은 아니다.

application BattleUpdate와 snapshot7 routes, world snapshot4 JSON adapters, BehaviorResult application projection의 직접 to_value 및
json! object producer를 연결한다. 고정 keyliteral/borrowed value의 heterogeneous tuple fields가 SerializeMap으로 같은 object를
만들어 첫 Value 할당 전에 계수한다. Nested completion은 child와wrapper각각 선행검사하며 json!로 Value를 만든 뒤 검사하지 않는다.
Sticky control 오류가 기존 infallible RecoveryPending으로 바뀌어도 outer authoring acceptance는 실패한다. 그 작은 오류 payload와
나머지 diagnostic producer 비용은 별도이며 public ordinary 결과/disposition/필드 의미를 바꾸지 않는다.

## GEX-10 상세 설계 027 — diagnostic JSON과 명시 control interruption bridge

Success result mapper/from_read_only/recovery_pending을 private Result로 바꾸고 기존 Result public ports가 실패를 그대로 전달한다.
HostTick은 Option.transpose로 None 의미를 유지한다. 실제 control inspect를 문자열 없이 확인하는 typed checkpoint를 두며 이미
sticky/cancel failure이면 allocation-free ProjectionFailure Err envelope만 만든다. Scope 바깥으로는 outer run의 실제 ExecutionFailure가
나가고 envelope는 버려진다. Accepted/RecoveryPending의 빈 성공으로 위장하지 않는다. Ordinary actual projection error는 기존의
AcceptedRecoveryPending/code/kind/details를 보존한다.

Recovery와 invalid_input은 Display를 빌린 채 bounded fmt String writer에 넘긴다. 매 write_str의 push 전에 ResultBytes와 work를 검사하고
실패 뒤 JSON 변환/메시지 formatting을 재시도하지 않는다. 다섯 serde error.to_string 선행 호출을 borrowed Display로 전환한다.
Domain error의 literal object는 같은 first-Value helper를 거쳐 오류 builder의 Self 형식을 유지한다. 실패는 typed checkpoint가 확인된
private error envelope로만 바꾸고 일반 serde 오류를 성공 Null로 바꾸지 않는다. 고정 literal/string 진단의 serializer invariant 위반은
기존 json!와 같이 별도 결함이다. 기존 caller가 만든 domain reason/command parse/DTO 소유는 이 JSON 경계 앞의 별도 의무다.

검사는 실제 snapshot/command 거절 후 내부 Err·outer original phase/site, 모든 기존 error mapping·일반 recovery JSON parity,
이미 sticky/cancel 상태에서 Display 호출0, 긴 diagnostic message의 첫 write 전 one-under/정확 한도와 같은 값 보존을 사용한다.
이 source slice는 optional trace/semantic occurrence/public Session 완료가 아니다.

## GEX-10 상세 설계 028 — actual HP/Stats occurrence의 첫 mandatory owner

026의 source 지도를 따라 BattleCore에 default-disabled private mandatory observer와 별도 direct source context를 둔다. 실제 HP damage,
nonzero-request signed HP, modifier 적용의 유효 target 확인 뒤 mutation 전에 fixed observation work와 checked ordinal/counters를 예약하고
mutation 직후·log/death/callback 전에 witness를 확정한다. HP delta는 i64(after)-i64(before)이고 modifier에 동반한 HP clamp는 같은 Stats
관측이다. 이 counter는 전체 authored effect 성공 수가 아니라 위 적용 channel의 횟수다. 두 fixed witness(latest/latest nonzero sourced HP)는
전체 history가 아니며 source Arc만 보유한다. Semantic key의 entry 연결/terminal capsule/optional trace와 buff/resource 등은 후속 연결이다.

Reserve의 실패는 mutation 전에 return하고 control sticky failure가 후보를 폐기한다. Checked counter invariant 오류는 별도 static typed
control failure와 observer fault bit에 남긴다. Opt-in owner 조회도 fault를 오류로 반환하며 ordinary disabled 호출은 기존 실행 의미를 유지한다.
Native Copy change enum은 Copy-bound clone visitor, source를 가진 owner는 exhaustive visitor에 포함한다. Runtime reset은 enabled 설정만
남기고 per-battle 관측을 초기화한다. Skill loop는 실제 authored+cast source를 관측 context에 설치/복원하고 unsourced command/batch 입구는
그 context를 take/restore하여 중첩 death effect가 바깥 source를 잘못 상속하지 않도록 한다. 기존 recording source/log 의미는 변경하지 않는다.

검사는 기존 실제 skill fixture의 반복 HP 적용/중간 unrelated log/unsourced command, native clone owner·source 유지, reserve work 실패 전 HP/
ordinal 불변 및 retry, explicit counter overflow rejection을 사용한다. 아직 public battle enable/terminal view/trace나 R03 완료가 아니다.

## GEX-10 상세 설계 029 — 실제 trial startup·entry와 terminal capsule 소유 연결

Lower setup의 private bool을 ordinary/generated-feasibility에서는 false, 실제 authoring GameCore live plan에서만 true로 전달한다.
BattleCore constructor 직후/start 전 observer를 켜며 reset은 enabled 상태를 유지한다. Actual authoring_battle_entered의 확인된 ordinal/UUID를
provisional observer key에 결속한다. GameCore에는 Copy lifecycle와 별개로 last_terminal Arc를 가진 private owner를 추가하고 모든 whole clone,
PreparedApplicationCommit, nonbattle transaction과 cfgtest rollback에서 이동/복원한다. V6/profile/GameCoreState에는 추가하지 않는다.

Actual terminal transition은 lifecycle finished 검증 뒤, active 제거 전에 controlled outcome_id clone·고정 capsule owner를 준비하고 유효 observer를
move한다. 이 immutable capsule은 actual elapsed/verdict/outcome/key와 mandatory witness를 소유하며 result 소비 뒤에도 유지한다.
Time-zero completion 여부는 이후 실제 zero 성공을 기록하는 lifecycle가 계속 소유한다. Retreat에 가짜 terminal을 만들지 않으며 active가 없어지면
current observation은 None, 이전 terminal은 그 entry key와 함께 별도 남는다. 새로운 B 전체 교체 시 옛 owner도 교체된다.

Private facade의 borrowed view는 실제 active observer/key/fault를 확인하고 active summary와 last terminal을 함께 반환한다. Ready/step mandatory
receipt는 이 view를 bounded writer로 직렬화하되 prior-public-result binding source에는 넣지 않는다. Optional trace는 아직 붙이지 않으며 이
payload에 trace flag/row/gap을 넣지 않는다. 실제 world immediate/positive terminal·결과 소비와 existing private prepare/rollback fixture로 owner
전파를 먼저 확인한다. 실제 external nonzero source/terminal·nested callback·optional trace/API/CLI 인수는 별도 필수 작업이다.

## GEX-10 상세 설계 030 — 선택 trace의 별도 제한·소유와 시도 비용

TraceOptions는 trusted Copy 설정이고 disabled도 같은 고정 clone slot을 차지한다. 유효한 enabled 설정만 내부 live setup에 전달한다.
행 기본1024/hard8192, 전체 encoded bytes 기본1MiB/hard8MiB, 행 기본16KiB/hard256KiB, 시도 work 기본16MiB/hard2^30을
양수·inclusive로 검증한다. Work는 고정 row 작업·문자열 사전 scan·encoded append·명시 새 buffer allocation bytes/기존 prefix 복사를
합산한 논리 단위다. 전체 prepare prefix 또는 advance 한 번의 별도 TLS scalar allowance이며 내부 clone/rollback이 되살리지 않는다.
Missing optional scope·overflow·allocation/encoding 실패는 optional gap만 남긴다. ExecutionControl/usage/ledger/mandatory receipt는 건드리지 않는다.

OptionalTrace는 Arc<Vec<u8>> NDJSON과 Copy counters/gap을 소유한다. Native Clone은 handle만 복사하며 exhaustive CloneShape는 options/rows/
bytes/gap 값과 무관한 고정 slots를 예약한다. 행 serialization 전에 현재 fixed witness의 source 문자열 len을 allocation 없이 선차감한다.
Writer는 행/전체 cap과 별도 work를 각 write 전에 검사한다. Shared owner 또는 capacity 부족이면 새 Vec를 명시 reserve하고 prefix를 복사한다.
그 전에 새 capacity·prefix len·append bytes를 선차감하여 native realloc의 숨은 복사에 의존하지 않는다. 실패한 partial row는 원 len으로 줄이고
count/last ordinal은 성공한 완전한 행에서만 갱신한다. 첫 gap 이후 serialization을 중단하고 고정 counters만 갱신한다. 전체 history 주장은 없다.

Terminal은 trace owner를 함께 move하되 serde(skip)로 mandatory payload에서 제외하고 별도 borrowed bytes/counters view로만 제공한다.
Config도 시나리오 SHA/binding에 포함하지 않는다. Actual producer의 mandatory witness가 확정된 뒤 optional writer를 호출한다. Trace on/off·행/byte/
work overflow가 complete mandatory usage·receipt·gameplay·순번·RNG·source를 바꾸지 않는지, clone 후 gap/append와 terminal retention을 검증한다.
이 한도는 각 retained buffer와 각 시도의 논리 비용이며 allocator/RSS/전체 session coexistence의 실제 측정 및 모든 effect coverage를 대신하지 않는다.

## GEX-10 상세 설계 031 — 공개 opaque Session·실패 위치와 borrowed 조회

TrialSession은 private TrialState 하나를 감싸며 Clone/Serialize/Deserialize/Deref/inner borrow를 제공하지 않는다. prepare는 기존 fresh 단일
scope prefix/ready 성공만 게시한다. Copy TrialPosition(Initial/Step global index/PreparationReady/End)을 scope 밖에 두고 실제 stage 시작 직전에
갱신하여 outer final poll과 inner 오류 모두 위치를 보존한다. TrialFailure는 operation phase와 original AuthoringError를 소유하며 code/source/
diagnostic phase를 덮지 않는다. replace는 같은 B 준비 성공 후 전체 상태를 move한다. run_to_end/simulate는 이미 주어진 유한 남은 step만
advance하고 첫 실패에 멈춘다. 실행 실패는 마지막 accepted session을 move한 SimulationFailure로 반환하며 준비 실패의 provisional prefix는
accepted로 반환하지 않는다. Complete는 입력 step 소진이며 전투 종료를 자동으로 뜻하지 않는다. 추가 완료 receipt는 만들지 않는다.

Receipt/Step view는 기존 &str/Option<&Value>를 빌리며 외부 missing step과 accepted None/JSON null을 구분한다. Iterator 순서는 initial→준비
records→ready→실행 records다. Identity/initial/input SHA/limits는 immutable borrow, accepted counters/위치는 Copy다. Mandatory 관측과 source/
terminal은 private owner를 감싼 borrowed 값 view로 조회하고 optional trace는 기존 complete bytes/Copy gap을 별도로 빌린다. 조회에 첫 JSON
생성·source copy·RNG/clock·quota 변경은 없다. Public native owner/Arc나 세대 mutable accessor는 추가하지 않는다.

소비된 content의 describe/explain은 별도 Inspection scope에서 기존 함수를 사용한다. Live snapshot/reconnect 역시 최초 facade 호출 전 별도
scoped limits/cancel을 설치하고 output admission 뒤 완료 문서를 반환한다. Preview는 허용된 실제 query kind만 owned 명령/context로 받고,
기존 public authority 검사와 private controlled fork를 거쳐 실행한 뒤 결과를 encode하고 후보를 폐기한다. Query의 usage/ledger/receipt는
accepted session에 채택하지 않는다. 명령 문서/결과 첫 allocation·source 문자열 검사 한계는 현재 공통 control 경계로 검증한다.

Transport는 이미 준비된 문서/NDJSON borrowed slices를 출력하며 I/O failure는 session rollback이나 gameplay error가 아니다. Typed 실패의
직렬화는 실패한 scope 종료 뒤 별도 bounded renderer가 수행한다. Renderer 실패가 original typed failure/accepted session을 대체하지 않는다.
실제 standalone public API/CLI·privacy negative·partial simulate·준비 stage 오류·None slot/receipt 순서·query 후 동일 다음 실행·external A/B
및 source terminal·최종 비용/현재성 검증이 이어져야 하며 공개 선언만으로 R03 인수를 완료하지 않는다.


## GEX-10 상세 설계 032 — closed typed Preview와 입력 사전 검사

TrialPreview는 실제 RequestDeploymentRangePreview/PreviewSkillTarget 두 variant만 받는다. UUID/Position/Target 및 context의
권한/cursor는 Copy guard와 exhaustive destructure로 고정하고 skill ID/source command ID는 첫 serde 문자열 scan 전에 길이로
MetadataBytes admission과 work를 예약한다. 고정256 logical bytes와 두 가변 문자열 길이를 합산하며 overflow는 거부한다.
기존 ExecutionLimits의 유한 상한을 적용한다. 이는 encoded 길이/heap 측정과 다르며 실제 JSON은 별도 projection/output cap도 받는다.
Private ApplicationCommand factory는 admitted public JSON을 만든 뒤 같은 typed fields를 실제 PlayerBehavior로 move한다.
Arbitrary Value의 clone-and-decode를 새 경로에 넣지 않는다. Factory의 command document/domain 의미는 실제 일반 decoder와 대조한다.

공개 Session.preview는 별도 Inspection scope/committed provenance 사본 안에서 input admission→factory→controlled application fork
→실제 public execute→bounded output을 수행한다. 후보/ledger를 폐기하고 accepted session quota·receipt·clock·RNG를 채택하지 않는다.
실제 query 종류는 기존 requires_battle_authority=false 판단도 그대로 따른다. 임의 권한 강화나 BeginSkillTargeting 자동 실행은 없다.
실제 prepared fixture의 배치 범위 결과를 일반 public execute와 대조하고 미배치 skill query 오류·work/result/clone/input cap 및
취소 뒤 전체 finite replay receipt를 검증한다. Native query DTO/source copy의 기존 미완료 범위와 외부 인수를 이 검사로 닫지 않는다.


## GEX-10 상세 설계 033 — 실패 이후 독립 bounded diagnostic renderer

공개 render_error/render_trial_failure는 original AuthoringError/TrialFailure를 빌리고 별도 DiagnosticLimits/cancel을 사용한다.
Gameplay TLS/accepted ledger/실패한 scope를 재사용하지 않는다. 실패 시 Copy DiagnosticRenderError를 반환하고 원 오류·accepted
session을 소유하거나 대체하지 않는다. 출력 자체 I/O는 여전히 caller 소유다. 진단 단계와 TrialPosition은 원 Serialize 의미를 보존한다.

Limits는 trusted 값이며 source 문자열 합계 기본1MiB/hard8MiB, output 기본1MiB/hard8MiB, nodes 기본16384/hard262144,
depth 기본32/hard64, work 기본16Mi/hard2^30의 양수 inclusive 한도를 사용한다. 단위는 논리 byte/node/작업이며 RSS가 아니다.
첫 실제 JSON encode 전에 allocation 없는 private serde preflight가 문자열 길이·container 길이와 depth/node를 검사하고 work를
선차감한다. 알려진 진단 graph만 공개 entrypoint로 받으며 custom Display serialization은 지원 오류로 거부한다. 임의 source
문자열/관련 진단 Vec/expected·actual·Application details의 Value도 같은 preflight를 거친다. Node/depth limit 이전에 거대한
Vec를 순회하거나 serde_json 문자열 escape scan을 시작하지 않는다. Error formatting에 진단의 Display/to_string을 호출하지 않는다.

Writer는 encode chunk를 받기 전에 총 byte/work/cancel을 검사한다. Buffer growth는 별도 새 Vec의 capacity와 이전 prefix 복사를
선차감한 뒤 fallible reserve하고 move한다. Complete JSON 이후 마지막 cancel 확인만 통과하면 immutable 문서/실제 usage를 반환한다.
모든 실패는 partial output을 폐기하며 original borrowed failure는 유지한다. Cancel된 gameplay token과 renderer token을 분리해
실패 출력 가능성을 검증하고, typed JSON parity·문자열/depth/node/encoded byte/work 경계 및 session 보존 oracle를 추가한다.


## GEX-10 상세 설계 034 — default-release CLI와 별도 transport

명시 Cargo bin `author_content`는 validate/describe/explain/simulate의 공개 library를 호출한다. 명령 형식은
`author_content OP TRUSTED_ROOT SELECTION.json [REQUEST.json] [--trace-output NEW_PATH]`다. Request는 explain/simulate에만
필수이며 trace output은 simulate에만 허용한다. Selection은 실제 ContentSelection 전체 packs/winners schema를 사용한다.
공통 validation_receipt query는 동일 published identity/source receipt/validation usage를 독립 bounded Inspection 문서로 만든다.
CLI는 content 이름·자동 VN/starter/target 선택·embedded fallback을 추가하지 않는다.

argv는6개·각4096 encoded bytes 이내다. selection/explain 입력은각64KiB, scenario는기존8MiB ceiling으로 읽기 전 file 크기와
regular-file 여부를 확인하며 실제 읽기도 cap+1로 제한한다. 고정 한도만 reserve하고 parse 전에 초과를 거부한다. 작은 selection/query는
serde의 기본 recursion limit을 유지하며 입력 bytes가 전체 token/string/node 수를 제한한다. Scenario는 기존 depth64/node262144
checked parser를 사용한다. OS 파일 I/O 지연이나 강제 process signal의 cooperative cancellation을 보장한다고 주장하지 않는다.

성공 query는 준비된 document 한 줄, simulate는 이미 accepted된 chronological receipt 각 줄을 stdout에 쓴다. 실패한 simulate도
last accepted session의 prefix를 출력하고 original typed positioned failure를 독립 renderer로 stderr에 쓴다. 출력 중단은 기존
실패/accepted owner를 대체하지 않으며 stdout 실패 뒤에도 original diagnostic 출력을 시도한다. 종료0=성공,1=library semantic
실패,2=CLI 입력 오류,3=transport/diagnostic delivery 실패다. Stdout/stderr의 실제 flush 실패도3이다.

선택 trace는 별도 새 파일(create_new)에 최종 active/last-terminal window만 내보낸다. 이는 전체 과거 battle history가 아니다.
고정 format header와 각 window의 Copy key/counter/gap header는1024-byte stack writer로 encode하고 기존 NDJSON slices를 그대로
쓴다. trace 설정은 library 별도 quota를 활성화하며 파일 생성/출력은 simulate 이후라 gameplay 결과·연료를 바꾸지 않는다.
기존 파일은 덮어쓰지 않는다. 실제 default-release 외부 pack에서 CLI JSON/실패 위치·accepted prefix/API replay parity와
trace-on/off·새 파일 거절·파이프 출력 실패를 검증한다. CLI 등록/컴파일만으로 외부 인수를 완료하지 않는다.


## GEX-10 상세 설계 035 — 실제 외부 effect A/B·terminal source 인수

기존 external official pack/public 준비 시나리오와 source hash를 인수하고, 시험 소비자만 실제 public deployment cell/직원
결과로 명시 DeployUnit과 유한 Clock 입력을 만든다. 첫 discovery는 최대로 정한 입력 안에서 실제 terminal producer를 관찰하여
후속 고정 replay 시나리오를 선택하는 fixture 생성이다. Discovery 중간 종료를 simulate 전체 완료로 표시하지 않는다.
각 accepted receipt/처음 actual sourced HP/실제 terminal 또는 typed failure를 보존하고 source/field/effect/순번·HP를 읽는다.

A/B는 외부 source 사본에서 확인된 실제 effect numeric operand만 변경한다. Source 선택을 태그/로그 문자열로 추정하지 않고
실행 witness와 exact authored definition/action/effect 및 loader explanation에 결속한다. 공식 RON/production runner/밸런스는
변경하지 않는다. 같은 seed/player/profile/template/resolved input/clock을 고정하고 두 generation의 semantic identity 변화와
실제 nonzero application/HP delta를 대조한다. B로 terminal 시점이 달라져 입력이 부적합하면 fixture owner에서 두 입력 모두에
유효한 공통 finite clock을 먼저 확정하며 실패한 시도는 보존한다.

실제 terminal 이후에는 active 제거와 last-terminal key/source 보존을 public 관측으로 확인한다. 실제 CompleteCombatResult가
허용되는 명시 command도 고정 replay에 포함하여 capsule 수명을 확인한다. 같은 고정 입력의 trace off/on/gap과 API/CLI receipt
parity를 비교하며 discovery의 가변 입력 SHA를 고정 replay의 SHA와 혼동하지 않는다. 전체 효과/지연/eviction/비용·fresh 감사는
이 대표 외부 인수와 구분한다.


## GEX-10 상세 설계 036 — 실제 A→B 교체 실패·회복의 외부 소비자 검증

검증된 외부 A/B fixture와 동일 fixed raw scenario를 재사용한다. A를 실제 배치·7950ms sourced HP 발생 후 accepted
index11까지 실행하고, content/initial/input/limits·전체 receipt/counters·lifecycle/관찰/trace·public snapshot/reconnect를
빌려 fixture snapshot으로 보존한다. 별도 B validate/prepare의 한도·취소·실제 authority/후속 command/ready 오류를 반복하며
실패한 B가 A의 어떤 관찰 가능한 상태도 바꾸지 않는지 비교한다. 측정한 B ready의 실제 work/receipt exact 및 one-under를
사용하여 late mandatory 실패 위치와 original phase를 검증한다. Trace off/on/one-byte-gap 세 mode에서 각각 수행한다.

실패 뒤 A의 남은 clock/result-consumption을 완료하여 기존 고정 A replay의 전체 bytes와 대조한다. 이후 실제 B 준비 성공으로
교체하고 index/identity/ready가 fresh B와 같음을 확인한 뒤 B도 끝까지 실행해 기존 B replay와 비교한다. 이 범위는 actual
valid-token failed-prep와 회복의 외부 oracle이며 native ledger/RNG 전체 직접 inspect나 allocator/generation drop 증거를
위조하지 않는다. Snapshot/reconnect·후속 완전 replay는 public 관찰 가능한 보존을 증명한다. 동시 final-poll cancel·모든
load producer 오류와 generation 참조 drop/비용은 별도 남은 검증이다.


## GEX-10 상세 설계 037 — 실제 명령의 공명 resource 관측

현재 ModifyResonance 명령은 add/modify_resonance의 기존 lock/active/cap/autocast 규칙으로 값을 바꾸지만 mandatory
관측에서 누락된다. 기존 실제 command owner에 observation-enabled일 때만 target의 raw before와 고정 reservation을
먼저 준비하고 기존 mutation을 그대로 호출한다. 실제 after가 달라진 경우에만 Copy Resonance(before/after/max) witness를
확정한다. direct source는 현재 process_skill_commands가 보존한 실제 authored/cast와 modifier source unit이다.
Ordinary gameplay는 추가 관측 조회/예약을 건너뛴다. HP delta는0이며 nonzero HP counter/대표 HP witness를 바꾸지 않는다.

이는 resonance 명령의 실제 적용 채널이며 passive gain·cast reset·Buff 큐 진입·전술 resource signal을 실제 resource
적용이라고 합산하지 않는다. 모든 효과 수명 관측의 남은 owner는 별도 전환한다. 예약 실패는 mutation 전에 sticky control
오류가 되어 후보가 게시되지 않는다. 기존 bounded-cardinality latest/source/trace/terminal·Copy clone-shape를 재사용한다.
실제 invoke_ability의 source와 양수/음수/cap/locked/dead/no-op, trace off/on/gap의 동일 mandatory usage/상태·다음 command,
clone 보존과 작은 work/cancel의 mutation 전 실패를 focused 검증하고 바뀐 생산자 영향의 authoring/battle 회귀를 복구한다.


## GEX-10 상세 설계 038 — 실제 외부 resource A/B 인수

검증된 combat A 외부 prototype을 두 번 복사하고 동일 judgement에 self-target ModifyResonance step을 추가한다.
두 사본의 유일한 차이는 그 실제 amount17→31이다. 기존 fixed finite raw scenario를 그대로 사용하며 public consumer의
전체 실제 battle/result consumption을 실행한다. 별도 trace에서 source-addressed 첫 resource application의 같은 before/
source/unit/ordinal/time 및 실제 delta17/31을 대조한다. 이는 compiled 값만 읽은 성공이 아니다. API off/on/gap의 mandatory
receipt·work·결과는 동일해야 하며 CLI simulate/Explain은 같은 published identity/receipt·실제 effect source를 반환해야 한다.
새 field를 로그 이름에서 추론하거나 공식 RON을 바꾸지 않는다. 실제 trace/file/command/binary/source/pack hash를 남긴다.


## GEX-10 상세 설계 039 — 큐 소유 Buff source와 실제 적용 관측

ApplyBuff command는 실제 modifier/direct source가 있는 동안 큐만 채우고, 이후 BattleEvent::ApplyBuff가 BuffDef/active/
duration을 검증한 다음 적용한다. 이 간격에서 현재 출처가 사라진다. 공개 BattleEvent constructor/wire는 바꾸지 않는다.
Private EventQueue의 항목을 QueuedEvent(event, optional immutable effect/cast source)로 바꾸고 clone-shape에 두 owner를
함께 포함한다. Compare/priority/동일시간 sort는 기존 event만 사용한다. Pop과 same-time bucket은 source를 함께 move하며
read-only peek/iter는 기존 event를 빌린다. Public/test raw event entry는 sourceNone으로 private dispatch에 들어간다.

실제 sourced ApplyBuff enqueue 직전 fixed handle-copy work를 예약하고 direct source만 복사한다. Observer off의 ordinary
queue는 sourceNone이며 ambient log source나 cause 번호에서 출처를 재구성하지 않는다. Dispatcher의 source는 Buff 분기에
명시로 전달하고 field-work settlement/CC의 nested effect context로 설치하지 않는다. 기존 HP/Stats/resonance 예약과 공유하는
helper가 explicit borrowed source를 받아 mutation 전 reservation을 만든다. 잘못된 Buff/0duration/inactive는 관측하지 않는다.

Valid apply는 기존 정책대로 actual reapply를 실행한다. Copy BuffApplicationState(stacks/expires_at_ms/next_tick_ms)의
beforeOption/after와 실제 BuffId/kind를 기록한다. Existing valid reapply는 상태 값이 같아도 log/CC/expiry 작업을 수행하므로
하나의 실제 apply assignment로 센다. Tick enqueue 자체를 HP 적용으로 세거나 기존 Buff tick 피해를 ApplyBuff의 direct HP
출처로 바꾸지 않는다. HP delta0/기존 nonzero HP slot을 유지하고 actual apply 뒤 trace/terminal이 같은 고정 witness를 소유한다.
새 Buff key의 retained-entity admission과 관측 예약은 최초 log/map mutation 전에 적용하고 실패는 outer candidate를 거부한다.

검증은 queue priority/metadata 독립·clone/pop/drop, actual delayed/동일time Buff apply/reapply/invalid와 sourced/unsourced
분리, traceoff/on/gap complete mandatory equality, 작은 fuel/queue/entity/예약 cap과 A clone/retry 보존을 포함한다. 실제 외부
prototype도 새 build에서 sourced Buff application/terminal을 관찰한다. Queued wrapper의 native memory/clone 비용은 최종
비용 owner에서 측정하며 기존 raw event debug fixtures만 private carrier 소비로 전환한다. 공개 event 필드 추가/저장 변경은 없다.


## GEX-10 상세 설계 040 — 외부 Buff 적용과 이전 gameplay 결과 대조

Resource001의 동일 A/B pack source와 raw finite scenario를 새 production build에서 다시 실행한다. 기존 external resource
harness가 API6/CLI simulate2/Explain2·terminal 및 traceoff/on/gap의 mandatory parity를 검증하고, 추가 consumer는 normal
trace의 실제 Buff 적용을 exact judgement_strike/effect_02 source와 대조한다. 각5회 비치명적 Damage cast의 Buff apply가
실제 source/cast/target을 공유하고 Silence/stacks1/700ms expiry/cadence 없음으로 기록되는지 확인한다. 단순 enqueue 기록은
성공 oracle가 아니다. Final terminal owner의 trace가 actual apply를 보존해야 한다.

Source095의 이전 Resource001과 current 결과를 같은pack hash/identity/input에서 대조한다. 각13step의 request/resolved input/
public result/lifecycle/initial provenance가 같아야 한다. 새 observation과 검사 비용은 바뀌므로 전체 old/new receipt equality를
요구하거나 주장하지 않는다. 이는 새 observer가 gameplay/권한/다음 공개 결과를 바꾸지 않았는지 실제 전후 검증하는 것이다.
출력·code·binary·source hash와 각 비교 필드를 보존하고 전체 effect/다른 delayed delivery/비용/선행·fresh 감사는 별도 남긴다.

## GEX-10 상세 설계 041 — 실제 invalid producer의 외부 API/CLI parity

V02/IVA02의 syntax, unknown field, malformed manifest, contract/capability, ambiguous writer,
final reference, summoned/boss compile, generated policy/custom boss setup을 실제 외부 RON으로 만든다.
Production validator를 우회하거나 fabricated diagnostic를 만들지 않고 public validate/render_error와
실제 default-release author_content의 전체 JSON을 대조한다. 알려진 phase/code/source/related/operands와
알 수 없는 precision의 부재를 별도 assertion으로 검증한다. 각 fixture를 두 번 검증하며 살아 있는
A TrialSession의 receipt/ready/index/lifecycle 보존 및 별도 valid A replay recovery를 확인한다.
Fixture/binary/source/명령/실패도 보존하고 오류 producer coverage를 전체 R03 완료로 간주하지 않는다.

## GEX-10 상세 설계 042 — 외부 delayed/repeated/projectile/area 실행 인수

V04의 실제 전달 경로를 먼저 관찰한다. 기존 외부 prototype의 공통 activation을 재사용하되 하나의 skill에
100ms delayed Instant×2, 200ms delayed Projectile, 300ms delayed persistent TileArea를 저작한다.
모두 기존 vocabulary이며 prototype 수치는 공식 RON에 반영하지 않는다. 같은 finite public preparation/deploy/clock/
result-consumption 입력을 off/on/gap 및 repeat로 실행한다. Production consumer는 기존 combat example/author_content다.

각 action의 실제 nonzero HP witness와 exact authored source/cast/target/time, 같은 시간 repeated 적용의 구별되는
mandatory application ordinal 및 persistent area의 연속 tick을 확인한다. On trace의 실제 witness와 off mandatory
latest/terminal, full receipt/next public command 결과가 같아야 한다. Semantic occurrence는 battle key와 actual mutation
ordinal 및 source를 대조하며 log seq를 RNG/identity로 승격하지 않는다. native delivery UUID를 임의 추정하지 않는다.
로그 eviction/sequence shift와 positive SkillPreview는 별도 직접 검증이 필요하며 이 fixture 성공으로 V04/V05 전체를
닫지 않는다. 실제 source 누락·잘못된 attribution·동일시 문제가 발견되면 producer owner에서 교정하고 재검증한다.

## GEX-10 상세 설계 043 — 실제 log eviction과 독립 occurrence/다음 결정

기존 focused owner fixture에서 actual invoke_ability로 source/cast를 할당하고 delayed Instant×2/Projectile/
persistent Area를 실행한다. 비교 분기는 log-only row를 cast 전에 추가하여 correlation seq를 바꾸고, cast 뒤 actual
BattleEventLog의8192개 보존 한계보다 많은 row를 record_event_log로 추가하여 cast row를 실제로 축출한다.
entries.clear로 eviction을 대신하지 않는다. Fixture noise는 별도 입력이며 이 검증의 budget oracle가 아니다.

Off/on/gap의 mandatory 관찰과 actual HP/source/target/time/ordinal, cast/projectile/area/damage allocation counter 및
이어지는 두 번째 actual cast와 다음 damage snapshot을 대조한다. Log correlation 차이만 정규화하고 semantic source/
occurrence는 남긴다. 실제 eventlog floor가 cast seq를 넘었고 cast row가 없는데도 delayed witness와 optional trace가
출처를 보존해야 한다. Explicit source를 테스트에서 조립하지 않는다. 이 시험은 log/noise가 gameplay/RNG allocation을
바꾸지 않는 focused 증거이며 독립 public scenario/terminal proof는 상세042의 외부 실행이 소유한다.

## GEX-10 상세 설계 044 — 실제 파편 획득·장착 후 positive SkillPreview

V05의 positive Preview를 외부 RON과 공개 명령으로 준비한다. Prototype의 첫 전투 보상에 기존 active fragment를
지급하고 실제 EquipSkillFragment를 호출한다. 해당 prototype의 compatibility와 imitation skill을 player_unit/global로
설정하고 다음 combat으로 가는 template 연결 및 기존 OnAttack의 self resonance 충전을 저작한다. Baseline fragment에
active skill을 주는 금지된 우회나 private roster/focus mutation은 사용하지 않는다. 공식 RON과 production은 변경하지 않는다.

첫 전투 완료/파편 지급·장착/두 번째 전투 deployment·clock 뒤 실제 readiness를 확인하고 BeginSkillTargeting을 실행한다.
그 공개 결과의 실제 revision/target으로 TrialPreview::PreviewSkillTarget를 반복한다. 각 조회 전후 snapshot/reconnect/
receipt/work/lifecycle을 대조하고 마지막 동일 ActivateSkill/clock의 actual sourced HP 및 완성된 finite receipt가
조회 없는 실행과 같아야 한다. Traceoff/on/gap과 CLI도 같은 시나리오를 소비한다. 두 번째 전투를 실제 terminal까지
실행하지 않은 경우 finite scenario completion과 battle completion을 구분한다. Invalid/cancel/low-cap 조회는 기존
focused 근거와 새 실제 기준에서 필요한 경우만 보강한다. 모든 fixture/readiness 실패와 수정도 기록한다.

상세044의 fixture 검증에서 ordinary Reward가 encounter-source fragment를 지급할 수 없다는 기존 계약을 확인했다.
기존 encounter fragment를 재분류하거나 validation을 완화하지 않고, 별도 Concept/RareReward prototype fragment가
같은 imitation skill을 제공하도록 저작한다. 실제 reward/equip 공개 흐름과 Preview/계속 실행 oracle는 유지한다.

## GEX-10 상세 설계 045 — 실제 세대 해제와 분리된 시간·할당 비용

IVA06/09의 실제 수명을 content generation owner에서 Arc::Weak로 관찰한다. Public validation/preparation으로 만든
A trial은 실패·취소·ready receipt 총량 거부의 B 교체 뒤에도 보존되어야 한다. B failure 객체가 살아 있는 동안에도
B generation은 해제되어야 하며, 성공 교체 뒤 A는 명시적 pin이 있는 동안만 살아 있고 마지막 pin/drop 뒤 해제되어야 한다.
Test 전용 공개 constructor나 generation registry를 추가하지 않는다. 실제 public preparation 입력을 사용한다.

Standalone default release consumer에서 같은 외부 pack/scenario의 validation/source query, preparation, step candidate
clone과 fuel을 포함한 실제 replay, trace off/on/gap, snapshot/reconnect, 성공·실패 교체 및 drop 비용을 기록한다.
Plain binary의 반복 wall time과 allocator observer binary의 allocation/peak/retained bytes는 별도 결과다. Allocator는
process 시작부터 모든 live bytes를 추적하여 scope 밖에서 만들어진 객체의 drop이 계측을 underflow시키지 않도록 한다.
Phase별 baseline live/peak/end와 alloc/realloc/free를 기록하며 출력 생성은 측정 scope 밖에서 수행한다. Native allocator
요청 byte 수는 RSS가 아니며 logical generation count를 실제 Weak 해제 증거와 혼동하지 않는다. 실제 toolchain/source/
fixture/binary/명령 및 실패를 보존한다. 일반 CoreApplication의 R03 전후 영향과 최종 전체 검증은 별도 근거가 필요하다.

## GEX-10 상세 설계 046 — 기존 runtime 성능 가드의 실제 영향 재검증

상세045의 새 authoring 비용과 별도로, 이전 R02가 실제 실행한 public_guards의 source/lock/fixture/threshold를
동일하게 사용해 current source에서 plain release와 allocation-observer를 다시 실행한다. Run snapshot/V6 export,
bounded history query, rejected staging의 history-size 독립성 및 rollback을 검증한다. 기존 allocator는 이 workload의
scope 안에서 생성·해제되는 rejected candidate만 관찰하므로 live A/drop 전체 수명 지표로 확장 해석하지 않는다.
기존 latency/size/scaling assertion을 변경하지 않으며 원 R02 결과와 실제 수치 차이를 기록한다. 그 밖의 P08 workload는
source/계약 적용 범위를 확인하여 필요한 baseline 비교를 별도로 수행한다. 전체 lib/tests와 all-target/default release,
외부 API privacy·선행 fresh 감사는 이 가드 통과로 대신하지 않는다.

상세046의 P08 direct BattleCore workload는 actual queue/effect/log/runtime가 R03에서 바뀌었으므로 비영향 재사용이
아니다. 기존 P01 schema2 baseline과 immutable preservation source hash를 입력에 추가하고 final_performance의
21 latency samples/5 allocation samples, affinity, deterministic parity, 기존 비교 assertion을 그대로 실행한다.
Public guard와 direct kernel의 적용 영역을 구별하며 계측 빌드 latency를 plain 결과에 합치지 않는다.

## GEX-10 상세 설계 047 — 오래된 direct-kernel fixture의 generation 입력 교정

상세046의 원 public_guards는 통과했으나 원 final_performance의64-session latency는10ms cap에 실패했다.
실제 결과는 baseline과 같았다. 조사상 이 도구는 각 BattleCore 생성마다 raw Arc<GameDataBase>를 넘겨 R02 이후의
Into<ValidatedGameData> fixture conversion에서 전체 database identity를64번 다시 계산한다. Production API는 이미
검증된 token을 받아 공유한다. P01의 당시 raw Arc 입력에는 이 generation publication 작업이 없었다.

실패 원문·binary·receipt와 unchanged threshold를 보존한다. Runtime 코드를 먼저 바꾸지 않고 도구의 동일 fixture를
한 번 실제 ValidatedGameData로 만든 뒤 clone해 입력하는 현재 API adaptation을 비교 실행한다. Content 숫자·scenario·
seed·session/step/lane counts·clock·log hash·affinity·측정 알고리즘·기존 cap/통계/할당 assertion은 유지한다. Raw→token
conversion 자체는 별도 plain 실측으로 남기고 새 loader의 비용을 숨기지 않는다. 이 adaptation 후에도 실패하면 실제
runtime/메모리 owner를 조사하며 threshold 완화로 성공 처리하지 않는다. Immutable P01 preservation source는 수정하지 않는다.

## GEX-10 상세 설계 048 — 선행 external actual 현재성 복원

감사 target 준비에서 R01/R02 target003의 일곱 `/tmp` fixture directory가 현재 없음을 직접 확인했다.
과거 verdict/receipt는 역사적 사실이며 삭제된 외부 actual을 current/unchanged로 표시하지 않는다.
Current release packager/consumer와 원 R02 A/B/A probe를 새 durable isolated directory에서 실행한다.
원 generated diagnostic probe도 경로와 input receipt ref만 새 directory/current source로 옮겨 같은 assertion을
실행한다. 과거 fixture를 재생성한 것으로 주장하거나 과거 receipt를 덮어쓰지 않는다. 현재 full core와 R03
producer 검증에 이 실제 20개 명령 결과를 더해 선행 fresh 감사의 새 관찰 입력으로 제공한다.

## 안정된 감사 대상과 출력 역할

GEX-10 구현·검증 종료는 [GEX_10_RESULT_001.json](GEX_10_RESULT_001.json), exact current source/evidence는 [AUDIT_TARGET_001.json](AUDIT_TARGET_001.json)이 소유한다. Current source [INPUT118](evidence/INPUTS_118.json)316개, 선행20명령 복원과 [영향표](PREDECESSOR_IMPACT_001.md)를 포함한다. Root source/data/test/contract/implementation evidence mutation을 멈춘다. 별도 감사 출력 역할은 R01 COMPLETION_AUDIT_004.md + audit-evidence/r03_successor_004/, R02 COMPLETION_AUDIT_003.md + audit-evidence/r03_successor_003/, R03 COMPLETION_AUDIT.md + audit-evidence/fresh_001/이다. 이 새 감사들과 파생 predecessor target004, 제외된 parent 운영 view만 후속 기록할 수 있다. 실제 current Complete/High를 R01→R02→R03 순서로 확인하기 전 R03/후속 R04를 완료/진입하지 않는다.

## GEX-10 상세 설계 049 — R03-AUDIT-F01 원본 바이트 receipt 교정

INITIAL fresh GEX20을 Not Complete로 닫고 [CORRECTION001](CORRECTION_001.md)의 정확한 owner/baseline/설계/검증/재감사 순서로 별도 구현 pass를 시작한다. 기존 token의 SourceReceipt를 initial/ready/step payload가 borrow하여 필수 bounded writer에서 함께 확정한다. 원본 바이트와 semantic fingerprint를 합치지 않는다. 새 필드의 work/bytes/cost와 실제 A/B/cancel/trace boundary를 현재 실행으로 확인한다. 과거 target001·audit·실패·측정은 덮어쓰지 않는다.

## Correction001 validation closure and fresh audit output roles

The six-line production repair and standalone regression consumer are identified by [source reconstruction](evidence/correction001_source_reuse_001.json), with original284 attribution [031](evidence/ATTRIBUTION_031.json). [EXP058–059](EXPERIMENTS.md) distinguish actual current runs, semantic-only old evidence reuse, historical failures and updated output/cost values. Current predecessor reports remain unchanged and their five changed target members plus rebuilt binaries are scoped by the new currentness record; the separate R03 reviewer independently decides prerequisite reuse. No whole old receipt/cost result is claimed current after metadata changed.

Root writes the final exact currentness record and AUDIT_TARGET_002.json, then makes no source/data/test/canonical/implementation-evidence changes during review. New reviewer output roles are COMPLETION_AUDIT_002.md and audit-evidence/fresh_002/. Excluded parent operational views can record factual transitions. If review finds a gap, close its pass before owner correction; do not force Complete. R04–R24 remain outside this R03 implementation and uncompleted under the parent AUTORUN request.
