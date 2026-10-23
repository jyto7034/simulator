# U11 — Lab 검증에서 실제 mod 적용까지: 코드 기반 검토

2026-10-10. 검토 owner는 main Astra/high다. 입력 정책은 `P2G_10_POLICY_001.json`의 `WR-CMVA-002`와 15개 decided revision/30개 수용 항목이며 의미를 바꾸지 않는다. 조사 범위는 `P2G_20_SOURCE_PLAN_002.json`의 `RSP-CMVA-002`다. plan001은 두 경로 표기의 교정 전 원본으로 보존한다. 아래는 actual과 **아직 구현하지 않은 제안**을 구별한 P2G-20 검토다. 제품·시험 수정과 실행은 하지 않았다. 정확 source bytes와 관련 span은 review artifact의 evidence가 결속한다.

## 확인한 현재 경로와 차이

### 콘텐츠·검증·세대

`content_pack/schema.rs`의 `PackManifest`는 pack ID/version/namespace/contract/capabilities/dependencies/files/operations를, `ContentSelection`은 선택 경로와 writer를 소유한다. 한 pack에 여러 파일이 있지만 여러 pack을 한 mod로 묶는 별도 identity는 없다. `snapshot::acquire`는 bounded 상대 경로·파일 identity·중복·전체 bytes를 검사하고 끝에서 파일을 다시 읽어 중간 교체를 거절한다. `SourceReceipt`에는 pack/path/bytes/SHA-256이 남는다. `resolver::resolve`는 의존 closure, 누락/순환/중복, 명시 Replace/Remove writer와 선택을 실제 검사하며 최종 정의의 semantic identity에 pack 구성과 writer 선택을 포함한다. `publish_snapshot_with_runtime_safety`는 최종 참조·컴파일·runtime publishability·generated preview를 통과한 immutable `ValidatedGameData`를 발행한다.

이것은 정적 입력 검증이다. `authoring::validate_with_runtime_safety`가 같은 loader를 작업 예산/취소 안에서 실행하고 `AuthoringContent`를 반환하지만, 실험실 합격·mod 승인·재시작 승인 캐시는 없다. `ContentAdmissionIdentity`는 콘텐츠와 runtime safety를 구별하므로 기반으로 재사용할 수 있다. 콘텐츠 fingerprint가 같아도 서로 다른 runtime profile을 쓸 수 있다는 현재 `bootstrap` 시험의 기대를 보존한다. source receipt를 semantic identity와 혼동하지 않으며 주석/경로만 바뀐 입력의 캐시 무효화는 더 엄격한 source binding으로 처리할 수 있다.

### 실제 trial·관찰·대규모 준비

`TrialSession::prepare`는 private application을 만들고 명시적 preparation을 실행한 뒤 실제 battle entry/time-zero lifecycle을 확인한다. `advance`는 candidate를 fork하고 명령·clock·snapshot·assert와 receipt 준비가 성공해야 교체한다. 실패하면 accepted prefix와 원인/위치를 보존한다. `run_to_end`는 유한 입력을 소진할 뿐 승리나 기능 발동을 보장하지 않는다. `ScenarioLimits`의 현행 256 steps는 typed trusted 인자에서도 ceiling이며 bulk setup 입력은 없다.

실제 World 경로는 `world/combat::plan_explicit_combat_node_with_state` → `CombatExecutor::resolve_live_combat_setup_plan_with_authored_policy` → `commit_planned_live_battle_start`다. encounter/preview/mission/reward를 검사하고 roster·inventory·fragments·relic numeric/structural owner, run resources, entry snapshot, attempt/power authority, battle UUID와 time-zero lifecycle을 함께 설치한다. `combat_setup/enemy_spawns`는 authored wave count를 여러 actor로 확장하며 RetainedEntities/work/poll과 unique spawn cells를 검사한다. 큰 적 집단에 actor마다 별도 TrialStep이 필요한 것은 아니다. 500/1000의 유효 board·동시 생존·실제 효과는 아직 측정하지 않았다.

`MandatoryObservation`은 HP/Stats/Buff/resonance의 누계 및 최신 witness를 보유한다. `RuntimeFactView`는 action/source, damage/protection/heal/cost/resource/death와 actor/provider 요약, latest settled action을 제공한다. 그러나 여러 정의의 중간 결과가 같은 clock 안에서 최신 슬롯을 덮을 수 있다. 객체·구조 변형·causal 효과 전체에 대해 마지막 snapshot과 전역 횟수만으로 정의별 coverage를 주장할 수 없다. 이 차이는 새 bounded coverage producer/consumer의 구현 범위다.

### 실제 host와 저장

`game_server/bootstrap::load_game_data`는 safety를 먼저 검증하고 선택 JSON을 읽어 정적 loader를 바로 호출한다. `main` → `AppState.game_data` → `GetOrCreatePlayerActor` → `CoreApplication::new`가 실제 세션 생성 경로다. 일반 actor 명령은 `prepare_command` 또는 `execute`를 호출하며 host의 승인된 새 generation을 선택하는 owner는 없다. `author_content`는 validate/describe/explain/simulate이고, `consume_content_pack`은 정적 token으로 실제 briefing/starter/map에 도달하는 진단 probe다.

`CoreApplication::prepare_command_with_generation`/`execute_with_generation`과 World draft는 적격 StartNewGame의 기존 gate 뒤, briefing/starter 전에 새 token을 설치한다. active run A의 소유를 B로 강제 교체하지 않는 기반이다. `ensure_battle_not_faulted`는 generation 경로도 막는다. U03 첫 원인·mutating/export gate·다른 actor 격리를 유지해야 한다.

durable export는 `ContinueState`의 cached V11 continuation이며 독립 SavePoint codec V12와 다르다. host가 V11을 요구하는 것은 불일치가 아니다. Core restore는 schema/checksum과 현재 generation의 full content identity를 검사한다. 다만 `ActiveRunStore::snapshot`은 Core content 검사 **전에** backup을 current로 복구할 수 있고 `protect_versions`는 envelope/Core version만 보호한다. `publish_prepared_application`과 paired intent는 active/profile 원자적 수락을 소유한다. mod 미승인/누락에 대한 사전 읽기 검사와 원본 보호를 이 실제 writer-capable read·write 경로에 연결해야 한다. 단순 Continue 실패 검사만 추가하는 것은 부족하다.

### 측정

`config/runtime_safety.ron`의 current native admission/extent는 16MiB이며 process heap/RSS 한도가 아니다. 기존 8MiB 증거는 과거 profile이다. `runtime_measurement::allocator`의 연속 requested-byte ledger/window와 bounded RSS 읽기, `playback`의 finite 입력·public Core driver·identity·결과 digest는 재사용 가능하다. `playback`은 feature 전용이고 observation=None도 plain default-release가 아니다. 입력 load 이후 measurement window가 열리며 session drop 후에도 inputs/export owner가 남는다. `VmHWM`은 생애 peak다. `authoring_trial_api_probe/examples/cost.rs`의 feature 분리와 receipt equality는 예제이며 U11의 workload·반복수·합격선을 대신하지 않는다.

## 권장 구조 — 구현 전 제안

### T01: mod registry와 immutable candidate

Core의 공통 mod-validation 계층에 explicit mod ID/version, 여러 pack member, mod dependency와 선택을 나타내는 유한 입력을 둔다. pack의 의미·writer resolver는 그대로 사용한다. registry는 mod와 pack의 중복 소유·누락·순환·oversize를 admission에서 거절한다. 특정 mod 실패 시 그 전체를 보류하며 dependent는 `dependency_held`로 구별한다. 나머지 선택은 원래 authored definitions에서 다시 resolve한다. 앞선 조합의 Replace/Remove 결과나 winner를 임의로 남기지 않는다. winner 선택이 더 이상 유효하지 않으면 typed 선택 오류로 반환한다.

검증 candidate는 한 번 취득한 immutable generation과 원본 mod/pack receipt, dependency/writer 구성, engine/compiler/capability 및 두 profile identity를 함께 보유한다. A의 validation 이후 파일 B를 재읽어 A approval로 적용하지 않는다. 프로세스 재시작에는 다시 취득한 token의 exact receipt/identity를 캐시와 대조한다. 동일 semantic content라도 exact byte/selection이 달라졌으면 해당 strict cache 재사용을 거절할 수 있다. 개별 mod는 trusted base와 필요한 dependency closure에서 검증하고, 최종 선택 조합은 별도 actual trial을 수행한다. 모든 부분집합을 무한 탐색하지 않는다.

### T02: 공개 Lab 준비와 실행 coverage

versioned authoring preparation 입력은 검증된 generation의 encounter/employee/loadout/relic 정의 ID, seed와 유한 workload를 지정한다. authoring facade가 정상 World run/node 준비 및 위 battle plan/commit을 소비한다. 필요한 시작 상태는 명시적으로 구성하고 정규 owner validation을 거친다. 외부에서 `BattleCore` 또는 arbitrary runtime struct를 넘겨 lifecycle을 우회하는 API로 만들지 않는다. 기존 명시적 TrialScenario 경로도 보존한다. 작은 공통 helper 추출이 필요하면 World와 Lab이 같은 준비 소비자를 호출하도록 한다. 세부 타입·함수명·normal navigation과 bulk 준비의 분배는 Sol이 현재 source에 맞춰 정한다.

검증 전에 candidate에서 실행 의미가 추가·변경된 정의/site와 현재 suite가 요구하는 capability 경로를 유한 목록으로 산출한다. workload mapping은 generation/site/trigger 및 scenario/seed를 식별한다. remove/비전투 변경도 대조군·snapshot/continuation 등 적합한 oracle를 배정한다. 기능 이름만 등록하거나 mod가 기대값/PASS를 제공했다고 coverage가 충족되지는 않는다. 준비 recipe는 untrusted 입력이 될 수 있지만 합격 predicate와 필수 목록은 issuer의 versioned suite가 소유한다.

실제 producer에 authoring 전용 bounded 집계를 붙여 `미도달`, `trigger 도달`, `조건/대상 부적격`, `합법적 무변화`, `실제 소비/효과`, `종료/회수`를 필요한 항목별로 구별한다. source의 ActionSite/EffectSite, object epoch, relic copy/effect/flow occurrence와 generation을 재사용한다. 같은 clock 안의 여러 site와 생성 후 같은 bucket 종료도 보존한다. 동작을 다시 실행하는 collector나 무한 event log를 만들지 않는다. 용량 부족·overflow·unsupported witness는 명시 incomplete이며 승인하지 않는다. 0 damage/full-health heal 자체를 불법으로 바꾸지 않고 요구된 양성 대조군과 각 합법 분기를 구별한다.

### T03: 유한 profile, 실제 calibration, 측정 격리

Lab profile은 schema/revision, workload/suite/coverage ID, 준비·실행·drop 구간, seed set, 반복·집계 규칙, 지원 환경, plain/instrumented mode, 시간·heap/native 등 각 지표와 concrete criterion 및 calibration evidence identity를 소유한다. trusted host만 profile을 선택/등록하며 콘텐츠는 수정할 수 없다. 미교정 profile의 결과는 diagnostic-only다. 엔진 fingerprint는 단순 고정 package version이나 클라이언트가 넘긴 source string으로 대체하지 않는다. 실제 빌드/컴파일러/feature identity 또는 host가 검증한 binary identity를 사용한다. 세부 build wiring은 구현 시 정하되 missing identity는 승인 근거가 아니다.

같은 의미의 public driver를 plain default-release와 instrumented 빌드에서 실행한다. command/error/gameplay/checkpoint 결과의 동일성을 확인하고 시간 집계는 섞지 않는다. validate/compile/prepare도 별도 구간으로 측정한다. profile별 process-local allocator window는 격리된 프로세스에서 수행하며 서로 다른 동시에 도는 검증에 한 process heap을 임의 분배하지 않는다. 내장 native work/admission, requested heap, RSS를 따로 보고한다. 필수 계측 미지원·불확실 window·drop 누락·overflow는 incomplete이고 gameplay/content failure와 구별한다.

GEX에서 대표 정상, 예상 확장, 병적 입력을 실제 실행하고 분포·peak·회수·환경·여유 근거를 검토한 뒤 유한 concrete criterion을 채택한다. exact-boundary/one-over 시험과 해당 criterion의 정상 Lab→Apply를 완료해야 U11을 닫는다. 500/1000은 유효 준비 가능성과 실제 발동을 확인할 workload 후보이지 모든 스킬의 target 수나 gameplay 인원 제한이 아니다. 낮은 PC 자동 완화·기존 R05 9MB 복사·임의 백분율 증가를 하지 않는다. 기존 실패 prefix는 보존하고 필요하면 별도의 유한 trusted measurement profile로 진단한다. 그 결과를 default profile PASS로 재표기하지 않는다.

### T04: issuer job·approval·cache

local/server마다 별도 issuer owner를 둔다. job은 immutable candidate, request/selection revision, cancellation, required suite/profile와 bounded 진행/결과를 보유한다. `pending/running/passed/content_failed/incomplete/cancelled/dependency_held/stale`를 구별하고, 모든 필수 실제 실행·coverage·criteria가 충족된 경우에만 opaque approved-generation capability를 발행한다. public 결과 DTO의 PASS 문자열을 역직렬화해 capability로 바꾸지 않는다. duplicate/late/cancelled 결과는 현재 selection revision과 job nonce 대조에서 commit할 수 없다.

cache는 issuer-local 신뢰 저장 영역에 exact identity와 결과만 보존한다. content root/client의 cache를 직접 받아들이지 않는다. issuer별 비밀 키 또는 동등한 owner 인증과 무결성 보호로 타 주체 복사·변조·부분 기록을 거절하고, 원자적 durable publication과 single-writer/revision 검사를 사용한다. 키를 cache나 mod와 함께 전달하는 구조는 배제한다. 검증된 암호 primitive/기존 crate의 구체 선택, private root 관리와 IO API는 Sol 상세 설계에서 review한다. threat boundary는 untrusted mod/client와 다른 issuer이며 동일 프로세스에서 임의 Rust 코드를 실행하는 악의적 host 자체를 격리한다는 주장은 하지 않는다.

cache IO 실패는 기존 유효 record/selection/live state를 보존한다. content 기준 초과와 계측/환경/IO incomplete를 별도 원인으로 기록하고 어느 것도 승인하지 않는다. 독립 mod PASS는 조합 FAIL로 소급 폐기하지 않되 최종 조합 approval은 발행하지 않는다. 최초/변경 선택은 자동 재검증하고 exact restart 재사용에는 issuer·engine·content·source·dependency/writer·safety/Lab profile·suite/scenario/seed·coverage/계측 환경 모두를 대조한다.

### T05: 실제 host Apply와 공개 진입점

공통 admission façade가 opaque approved generation과 현재 selection revision을 확인한 뒤 future-session 선택을 commit한다. local API/CLI와 실제 server bootstrap/management owner가 이 façade를 소비한다. server AppState/load-balance/actor의 현재 raw static load-only 경로를 함께 전환한다. host 선택은 일반 player command의 path/profile/PASS 필드로 전달하지 않는다. 서버 관리 API의 권한은 기존 host owner에 한정하고 인증 없는 원격 관리 endpoint를 새로 열지 않는다.

실행 중 A는 기존 immutable generation을 계속 보유한다. B는 승인 후 신규 actor 또는 적격 StartNewGame의 기존 staged generation 경계에서 설치한다. fault actor는 U03 guard를 우회하지 않으며 다른 정상 actor의 사용은 막지 않는다. prepare 실패·stale selection·취소·영속 수락 실패 전에 A의 state/seed/profile identity를 바꾸지 않는다. 기존 U04 accepted-intent 이후 recovery_pending 의미는 유지하며 새 전체 rollback 보장을 만들지 않는다.

정적 validate와 저수준 trusted `CoreApplication`은 authoring·기존 embedding의 진단/실행 기반으로 필요하다. 그것을 mod 승인 API라고 재명명하지 않는다. **U11의 정상 mod launcher/server 경로는 approved façade를 의무 소비**하고 외부 normal-host probe로 이를 입증한다. raw static runtime probe는 진단용임을 문서화하거나 정상 launcher와 명확히 역할을 전환한다. arbitrary in-process host가 저수준 API를 호출하는 것까지 차단한다는 과장은 하지 않는다. 공개 API/CLI의 정상 사용 설명은 승인 경로를 안내해야 한다.

### T06: required-mod 저장 보호

durable run에 필요한 mod/pack 구성과 full content identity를 조회할 수 있는 bounded metadata/검증 포트를 둔다. 현재 Core checksum/schema/full identity 검사를 유지한다. 상세 metadata를 envelope 또는 codec에 더한다면 version·현재 V11 continuation/V12 SavePoint와 동반 전환 및 기존 strict unsupported-original 보호를 명시한다. 임의 누락 정의 삭제·save 재작성·숨은 migration은 없다.

host는 승인된 현재 구성과 required identity를 **writer-capable snapshot/backup repair, pending intent recovery, profile merge 및 새 run 교체 전에** 읽기 전용으로 검증한다. 거절은 current/backup/profile/active bytes를 보존하고 missing/held/stale 구성 이유를 반환한다. 호환 구성을 복구하면 같은 원본을 다시 읽어 Continue할 수 있어야 한다. 새 게임/다른 호환 저장은 독립된 host가 명시 선택한 persistence root/session에서 계속 사용할 수 있다. 기존 한 installation의 단일 active-run 교체 권한은 유지하되 보호된 불호환 원본을 자동 삭제하는 우회로 삼지 않는다. 자동 다중 슬롯·저장 전환 정책은 추가하지 않는다. 현재 별도 persistence root API를 실제 정상 경로로 소비하고 그 사용을 문서화한다.

### T07: 수명과 회수

job/context/coverage/native budget은 issuer·trial별이다. immutable generation 공유와 mutable trial state의 독립 clone을 구별한다. cache는 live battle/개인 save/whole generation을 붙잡지 않는다. active A와 approved B의 필요한 동시 보유는 허용하되 마지막 token/trial/application/manager owner 종료 후 회수를 검증한다. process heap의 drop 관찰에는 입력·export infrastructure의 의도적 baseline을 명시하며 zero heap을 oracle로 만들지 않는다. 모든 job/registry/cache/coverage/diagnostic의 크기와 work는 유한하며 saturation 결과는 보존/거절 규칙에 연결한다.

## 정책별 delta와 검증 연결

각 행의 target 및 두 수용 항목 원문은 P2G-10 exact revision이 소유한다. 다음 표는 구현 차이·제안·검증 seam이며 정책을 대체하지 않는다.

| 요소 | Current Actual과 필요한 delta | 제안/정상·거부 검증 seam |
| --- | --- | --- |
| PE01 | pack/file/dependency는 존재, multi-pack mod 원자 단위 없음 | T01. 두 pack 한 mod의 한 실패에서 전체 hold; 독립 mod 유지; mod/pack ID·edge·cycle·oversize 진단 |
| PE02 | 정적 token과 실제 trial은 존재, 둘을 승인으로 연결하지 않음 | T01/T02/T04. 외부 정상 RON validate→World entry→actual trial; malformed/final-reference/unsupported opcode 거절·live 무변화 |
| PE03 | 유한 명령 및 latest 관측, bulk/정의별 필수 coverage 없음 | T02. 실제 normal/expanded actor·trigger/effect/cleanup; 같은 bucket 여러 정의·target 없음·setup 실패·빠진 scenario에서 승인 보류 |
| PE04 | runtime limits와 측정 예제, 교정된 Lab criterion 없음 | T03/T04. 대표 실제 raw calibration→구체 profile→정상 Apply; exact/one-over·위조·미교정·다른 profile·저사양 완화 거절 |
| PE05 | TrialFailure 위치/phase 및 typed 도메인 오류, Lab 종합 분류 없음 | T02/T03/T04. content criterion FAIL과 환경/collector incomplete·cancel을 구별; 결과 digest parity와 bounded diagnostics |
| PE06 | source/semantic/safety identity 있음, issuer approval/cache 없음 | T01/T04. exact restart hit와 dependency/writer/engine/profile/suite 변동 miss; foreign/tampered/truncated cache·TOCTOU 거절 |
| PE07 | 전체 pack resolve만 수행, 개별/의존/조합 상태 없음 | T01/T04. A/B 각각 실제 PASS인데 조합 실행 FAIL; 개별 유지·조합 hold; 실패 mod dependent hold·최종 재resolve |
| PE08 | staged generation StartNewGame·active A pinning 존재 | T04/T05. 승인 B actual host 적용→new B·old A; 파일교체·실패/미완료/stale B→A state/seed/token 유지 |
| PE09 | Trial replace/step candidate 원자성 있음, job/cache commit 없음 | T04/T05. cancel·late·duplicate·selection revision 경쟁·cache 쓰기 실패에 original record/live 불변 |
| PE10 | Core full identity load 거절, host 사전 mod 보호 없음 | T06. default-release save/fresh Continue; missing/held/stale/불일치에서 current/backup/profile 원본 동일, 복구 후 같은 save 성공·독립 root 새 게임 |
| PE11 | U03 fault first cause/generation/export gates 존재 | T05/T06. Lab PASS로 fault actor 교체/쓰기를 우회하지 않음; 정상 입력 거절은 fault가 아니고 정상 두 번째 actor 계속 진행 |
| PE12 | API/CLI/actual server는 정적 token 사용 | T05. 외부 normal local launcher·actual server bootstrap/management에서 승인→gameplay; player/foreign 권한 거절·typed 상태 query |
| PE13 | current capabilities·profile identity 존재, suite 결속 없음 | T02/T03/T04. U07–10 경로별 coverage, capability/engine/suite 변경 invalidate; dev/default evidence 불일치와 unsupported family hold |
| PE14 | trial candidate/native provenance·immutable Arc lifetime 있음 | T02/T04/T07. 독립 두 issuer/trial/게임 profile, cancel/failure/clone·마지막 owner drop, finite job/cache/collector 포화 |
| PE15 | 선행 독립 완료 증거가 존재, U11 실행 없음 | 모든 T. 변경 영향 관련 기능/외부/설정/성능·필수 calibration·host integration·자체검증·whole Goal 독립 감사 |

## 검증 범위와 재사용

독립 테스트 Astra의 두 차례 read-only 분석을 인수했고 Root가 관련 source/consumer를 직접 읽었다. 기존 `same_binary_external_generations_retain_exact_content_and_sources_after_files_change`, `only_final_winning_expression_is_compiled_and_published`, `unsupported_unused_external_opcode_cannot_publish_even_in_a_losing_writer`, `public_trial_failures_keep_preparation_position_and_partial_simulation_owner`, generation late-preparation 보존 및 실제 U07/U08/U09/U10 외부 trial은 재사용할 input/helper/expectation이다. 아직 U11에서 실행한 결과가 아니다.

선정할 회귀는 resolver/source acquisition, runtime profile identity/strict restore, Trial prepare/replace/cancel/receipt, actual fact/source·object/relic/causal lifetime, U03 first fault gates, U04 paired persistence/backup/CAS/recovery와 actor/player authority다. 새 job/cache public API, restart/tamper/stale/TOCTOU, normal launcher/server, missing-mod original restore, plain/instrumented equivalence 및 calibration은 새 증거가 필요하다. hot producer coverage나 shared prepare가 변경되면 해당 U07–10 공개 검사와 비용 경로를 포함한다. 전체 suite는 자동 요구하지 않는다. 필요한 span을 실제 diff로 재평가하여 Astra가 tests/oracles를 작성하고 Sol이 하나의 runner 소유로 실행한다.

U02 Master, U03 successor003, U04 successor002와 U07-M의 완료는 당시 contract/prerequisite이며 현재 U10 whole audit의 predecessor 영향 판정과 현 source를 함께 사용한다. U10 `u10_current_002_pass001`의 Complete/High/open0를 인수한다. 과거 whole FAIL·PC/R05 경고·당시 8MiB profile은 보존하고 U11 현재 실행처럼 표시하지 않는다. 새로운 Lab 기준은 과거 회귀 guard를 소급 완화하지 않는다.

## Finding 분류·남은 선택·인계

F01(T01), F02(T02), F03(T03), F04(T04/T05), F05(T06)의 primary classification은 **implementation_finding**이며 disposition은 해당 후속 구현·검증 Work Package에 인계다. 이를 해결하는 T01–T07의 내부 realization 선택은 별도의 **technical_decision**이다. 접근 불가 source나 새 material policy 결정을 숨기는 상태가 아니다. 현재 numeric calibration·500/1000 성공·새 collector/API/cache·mod Apply는 미구현/미실행이며 GEX 완료 전 필수다. source plan의 external Released/Deployed/Unity/개별 최종보스는 scope상 N/A이고 workspace source로 호환을 추정하지 않는다.

대안과 선택 이유: T01은 pack resolver를 복제하는 대신 상위 mod grouping을 추가하여 기존 writer/ref 검증을 유지한다. T02는 actor마다 수백 명령 또는 raw BattleCore 조립 대신 정상 World 준비 adapter와 유한 실제 producer 집계를 택하여 lifetime/entry 의미와 coverage 누락을 함께 다룬다. T03은 feature off처럼 보이는 계측 binary나 단일 평균 대신 명시 plain driver·분리 구간/분포를 택한다. T04는 사용자 receipt 또는 전역 mutable PASS 대신 issuer owner와 인증 캐시를 택한다. T05는 모든 저수준 Core 생성 API를 무조건 제거하는 대신 정상 launcher/server의 approved façade 소비를 의무화하여 진단·기존 trusted embedding 기반을 보존한다. T06은 자동 save 재작성/다중 슬롯 도입 대신 read-only 보호와 기존 명시적 독립 persistence root를 사용한다. T07은 전체 trial/history를 캐시에 보존하는 대신 bounded identity/result만 남겨 live owner의 회수를 방해하지 않는다. 모두 P2G-10의 관찰 의미를 유지하는 선택이다.

Sol의 상세 구현 선택은 schema/API 이름, bounded 자료구조, helper 배치, cache의 검증된 인증 primitive/IO adapter, 현재 정상 prepare recipe, build identity wiring, process measurement driver 공통화다. Root는 공개 승인/저장/수명/오류 계약과 기준 채택을 review한다. threshold 채택은 최신 사용자 실측 위임과 P2G-10 PE04/05/13의 범위에서 수행하며 임의 수치로 대체하지 않는다. 구현 결과가 required normal workload나 명시 정책을 실현할 수 없다는 전제를 드러내면 owner에 반환한다.

권장 작업 경계는 하나의 atomic 구현·검증 묶음이다. registry/candidate→Lab coverage/criteria→issuer/cache→actual host/save가 모두 연결되기 전 정상 mod admission을 완료로 발표할 수 없다. 내부 구현 순서는 의존성에 따라 나눌 수 있지만 미교정 profile의 진단 결과를 중간 Apply 허가로 승격하지 않는다. 최종 Goal은 위 전체를 포함하며 비참여 Astra의 whole Goal 감사까지 필요하다.
