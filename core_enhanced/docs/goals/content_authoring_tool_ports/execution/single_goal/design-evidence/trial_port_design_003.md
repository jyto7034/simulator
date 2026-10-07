# R03 Trial port minimum design 003 — 읽기 설계 보조

이 문서는 구현 제안이다. GEX-20 완료 감사, 테스트 결과, 이전 보조 보고서의 승인 또는 현재 미구현 surface의 성공 판정이 아니다. Root는 별도 targeting/control 코드를 계속 수정 중이다. 저장소 수정·시험·build를 하지 않았고 지정한 새 `/tmp` 보고서 두 개만 작성했다. Goal/PLAN의 정해진 범위 안에서 타입·grammar·budget 기술 선택을 제안하며 새 gameplay/자동 콘텐츠 선택 정책은 추가하지 않는다.

## 1. 바로 구현할 수 있는 최소 단위

`authoring/mod.rs`에 기존 validate를 유지하고 `query.rs`(describe/explain과 bounded Serialize writer), `scenario.rs`(strict DTO·pointer 검증), `trial.rs`(opaque session·candidate·receipt), `observation.rs`(private capture DTO/retention)를 추가하는 정도면 충분하다. 공개 callback/scope/범용 상태기계는 필요하지 않다. CLI `author_content`는 동일 함수의 decode/call/serialize adapter다. 실제 구현하지 않은 route는 describe에 지원으로 내보내지 않는다.

권장 공개 surface(이름은 기술 제안):

- `describe(&AuthoringContent, &QueryLimits, &CancellationToken) -> Result<Description<'_>, AuthoringError>`: immutable borrowed Serialize projection. `explain(&AuthoringContent, &ExplainRequest, ...) -> Result<Explanation<'_>, ...>`도 동일하다.
- `TrialScenario::from_json_slice(&[u8], &ScenarioLimits) -> Result<TrialScenario, ScenarioError>`: private fields, 검증 생성자. API의 직접 입력도 같은 검증을 거쳐야 한다.
- `TrialSession::prepare(&AuthoringContent, &TrialScenario, &TrialLimits, TraceOptions, &CancellationToken) -> Result<TrialSession, TrialFailure>`: initial+명시 preparation prefix만 준비하고 실제 battle entry/timezero 확인.
- `session.step(&TrialStep, &CancellationToken) -> Result<&StepReceipt, TrialFailure>`; `session.run(&[TrialStep], ...)`은 이 경로를 유한 반복한다. `simulate(...)`는 prepare+run 공통 함수다. 실패시 이미 accepted된 prefix가 있는 run 결과를 구분하고 실패 뒤 자동 진행하지 않는다.
- `session.load(&AuthoringContent, &TrialScenario, &TrialLimits, TraceOptions, &CancellationToken) -> Result<&LoadReceipt, TrialFailure>`: 독립 B를 prepare한 뒤 complete aggregate swap. `receipt(&self)`/`trace(&self)`는 immutable retained projection이다. 외부 content acquisition도 합치는 convenience load는 아래 scope 규칙을 따른다.

`TrialInitial`은 PLAN이 명시한 `player_id: Uuid`, `run_seed: u64`, `profile: DefaultMemory`를 명시한다. 실제 새 app의 `snapshot(PlayerProfile)` 전체를 최초 receipt에 넣어 provenance를 뒷받침한다. 개인 파일을 읽지 않는다. 시나리오가 설정하지 않은 VN choice/map node/deployment를 runner가 고르지 않는다. session 내부는 CoreApplication, immutable validated token, initial snapshot, limits, accepted request/results/receipt, observation과 집계만 보유한다. Public Clone/inner borrow/raw GameCore getter는 없다.

현재 `CoreApplication`은 Clone이 없다(application.rs781–791). 필요한 추가 seam은 **crate-private** `fork_for_authoring(&self) -> Result<Self, ApplicationError>` 또는 같은 의미의 private error다. 기존 complete GameCore clone으로 새 opaque app을 만들고 clone 전 opt-in admission/charge, 직후 sticky poll을 수행한다. V6 checkpoint roundtrip이나 fixture constructor로 대신하면 active battle·profile·seed/UUID authority를 보존하지 못한다. 불변 token Arc 공유는 가능하지만 mutable receipt/quota/observer 공유는 금지다.

## 2. 기존 공개 API에 대한 정확한 매핑

| Step / 초기화 | 호출 | 보존할 실제 결과 |
| --- | --- | --- |
| Command | `ApplicationCommand::from_document(ApplicationDocument::new(resolved))`, `CoreApplication::execute(player_id, command, context)` | 기존 ApplicationResult 전체: kind, meaning, disposition, payload |
| Snapshot | `CoreApplication::snapshot(ApplicationSnapshotKind)` | `{kind,read_model}` 전체; SelectedEvent null도 실제 결과 |
| Clock Elapsed | `advance_active_battle_by(delta_ms, after_seq)` | ApplicationResult |
| Clock HostTick | `advance_active_battle_for_host_tick(delta_ms, after_seq)` | `Some(result)`와 `None`을 구별. None은 blocked VN/pause 등의 실제 no-update이며 성공을 조작하지 않음 |
| Clock NextEventBucket | `advance_active_battle_to_next_event_bucket()` | ApplicationResult; 이 API에 after_seq 인수는 없음 |
| Assert | 이전 accepted public result/snapshot pointer에 대한 bounded JSON equality/exists/type 검증 | assertion 결과만; 새 core 호출이나 자동 선택 없음 |

실제 method spans: application.rs150–218, 220–238, 686–724, 827–952. `ApplicationCommandKind::ALL`과 `ApplicationSnapshotKind::ALL`을 그대로 사용한다. command decoder를 별도 gameplay validator로 복제하지 않는다. strict는 authoring envelope/binding grammar의 strictness이며, 기존 command 내부 필드 수용 규칙을 임의로 다시 정의하지 않는다.

`ApplicationExecutionContext`는 serde DTO가 아니다. `TrialExecutionContext`를 deny_unknown_fields로 추가하고 `source_command_id: Option<String>`, `battle_event_after_seq: u64`, `expected_battle_authority: Option<ApplicationBattleAuthority>`를 그대로 변환한다. player_id/run_seed는 initial authority이며 binding destination이 아니다. source_command_id도 기존 값을 지정하거나 prior public string을 복사할 수 있고 step 번호를 gameplay ID로 주입하지 않는다.

Readonly reconnect는 이미 `active_battle_reconnect(after_seq)`로 존재한다. 유한 grammar를 늘리지 않으려면 시나리오는 기존 `request_battle_state` Command를 쓸 수 있다. 설명/query 또는 session 외부 read-only 조회를 accepted-step cursor/집계 fuel에 몰래 넣지 않는다. 반대로 script의 Snapshot/Assert는 명시 accepted 입력이므로 receipt cursor가 늘 수 있지만 gameplay authority는 바뀌지 않는다.

기존 host continuation은 application.rs972–1002의 `export_active_run_checkpoint`/`continue_from_active_run_checkpoint`다. world/active_run.rs225–277은 V6 continue가 저장된 run_seed를 읽고 UUID manager를 다시 만들며 loads_used를 늘리지 않음을 보여준다. 이 기능을 private live fork로 간주하면 안 된다. 이 최소 초기화는 PLAN의 새 default-memory session을 사용하며 V6/profile 포맷에 새 trial/trace 필드를 넣지 않는다. 기존 ContinueRun/LoadRunCheckpoint 등 공개 Command 자체는 기존 core 의미로 실행할 수 있다.

## 3. finite pointer binding 계약

`TrialStep`은 `#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]`의 Command/Snapshot/Clock/Assert 네 variant다. Command는 `command: Value`, strict context template, `bindings: Vec<Binding>`을 갖는다. `Binding { from: {step: usize, pointer: String}, to: String, value_type: JsonValueKind }`는 값 복사 하나다.

- source step은 **현재 generation의 이미 accepted public Command/Snapshot/Some Clock result**만 참조한다. 실패한 B/실패한 step/optional trace/assertion receipt/다음 step/이전 generation은 참조할 수 없다.
- 각 source pointer의 root는 ApplicationResult 또는 ApplicationSnapshot 자체다. 경로는 `/payload/visual_novel/session_id`, `/read_model/game_state_context/candidates/0/employee_definition_id`, `/payload/battle_update/battle_state_revision` 등이다. wrapper의 `/result` 같은 가상 path를 혼용하지 않는다.
- destination root는 `{command,context}`다. `/command/session_id`, `/context/expected_battle_authority/battle_uuid`처럼 기존 leaf/hole만 허용한다. root 전체 교체, `/command/type` 교체, array append `-`, 부모 자동 생성은 금지한다. array는 존재하는 canonical unsigned index만 쓴다.
- JSON Pointer는 빈 root(읽기 전용 source에서만 허용 가능), `/` 분할, `~0`→`~`, `~1`→`/`만 해석한다. 잘못된 escape는 오류다. duplicate/ancestor-overlap destination은 **decoded token sequence**로 비교한다(`/a`와 `/ab`는 겹치지 않음). template mutation 전에 모든 source와 destination을 검증한다.
- explicit `value_type`이 null hole의 기대 타입을 정한다. source JSON 종류가 다르면 오류; 모든 copy 후 기존 command/context decoder가 UUID/enum/u64 등 semantic type을 검사한다. 문자열 interpolation/coercion/numeric arithmetic/검색/filter/loop는 없다.
- 복사 전 source 값의 serialized bytes와 structural node/depth를 bounded traversal로 계산하고 copy budget을 admit한다. 이후 clone한다. resolved 전체도 재검증한다. template·binding list·resolved input·accepted result를 receipt에 저장해 replay와 오류 위치가 검토 가능해야 한다.
- JSON raw input의 duplicate object key도 감지한다(serde_json Value의 last-write-wins에 맡기지 않음). unknown authoring fields, 잘못된 step variant, oversized/deep input을 AST 전체 생성 전에 bounded deserialize/scan으로 거부한다. trusted Rust 생성 경로도 동일 검사를 통과한다.

예시(0번의 실제 start 결과를 이미 accepted했다고 가정):

```json
{"kind":"command","command":{"type":"advance_visual_novel","session_id":null,"current_scene_id":null},"context":{"source_command_id":null,"battle_event_after_seq":0,"expected_battle_authority":null},"bindings":[{"from":{"step":0,"pointer":"/payload/visual_novel/session_id"},"to":"/command/session_id","value_type":"string"},{"from":{"step":0,"pointer":"/payload/visual_novel/current_scene_id"},"to":"/command/current_scene_id","value_type":"string"}]}
```

이것은 문법 예시다. 실제 fixture의 scene 타입/step0 wrapper와 맞는지는 구현 뒤 external consumer에서 확인해야 한다. 기존 audit probe main.rs40–96은 해당 공개 continuation이 존재함을 보여주지만 `take(required_count)`/`find(Available Combat|Boss)` 자동선택 부분을 새 runner로 옮겨서는 안 된다. 작성자가 유한 배열 index와 선택을 명시한다.

## 4. trusted limits와 mandatory output

기존 `AuthoringLimits { input, execution }`와 `ExecutionLimits`는 Deserialize하지 않는다. 새 `QueryLimits`, `ScenarioLimits`, `TrialLimits`, `TraceLimits`도 trusted host settings이며 CLI flag는 별도 검증해 hard ceiling 이하만 허용한다. content/scenario JSON 안에 quota override를 두지 않는다. Typed requests의 재검증도 필요하다.

구현 시작에 사용할 수 있는 보수적 **기술 제안값**(정책 확정값/측정된 적정값 아님): scenario raw 8 MiB, depth64, 총 step256, step당 binding64, pointer UTF-8 1024 bytes·decoded segments32, 단일 command/context template256 KiB, step당 복사1 MiB·전체 복사8 MiB, 단일 mandatory result8 MiB·session mandatory64 MiB, query result8 MiB, optional trace8192 rows·4 MiB. 단일 요청 clock delta60,000 ms, 총 요청 delta3,600,000 ms; 실제 observed elapsed 합도 별도로 cap(HostTick speed/pause와 NextEventBucket을 raw delta로 오인하지 않음). 기존 ExecutionLimits와 in-call fuel가 내부 무한 same-time work를 막고, actual next-bucket elapsed는 호출 결과의 candidate에서 checked admission 후 commit한다. 모든 더하기/곱하기는 checked, inclusive limit, overflow는 typed failure다. 상한 내 구현 메모리는 input/result/retained state의 합으로 설명하며 OS 전체 메모리의 정확한 계측이라고 주장하지 않는다.

Query는 immutable borrowed projection을 **counting Serialize writer**로 먼저 검사해 byte/work 상한을 통과하면 반환할 수 있다. full database clone 또는 먼저 거대한 JSON Value를 만들 이유가 없다. CLI가 같은 불변 projection을 나중에 serialize하는 작업은 transport이며 실패가 validate/query 성공을 소급 취소하지 않는다. iterator/HashMap order는 descriptor 정의 registry 또는 DefinitionKey의 canonical 순서를 사용한다. 순서에 따라 failure usage가 바뀌는 per-entry size admissions를 만들지 않는다.

Session은 binding을 위해 owned public Value가 필요하다. 기존 public API 자체가 ApplicationDocument를 만드는 비용은 domain/runtime caps 및 opt-in work로 제한하고, 새 layer의 보관/복사에는 별도 before-copy admission과 capped writer를 적용한다. `serde_json::to_vec`로 제한 없이 할당한 뒤 len만 검사하지 않는다. result temporary 전체 크기가 output budget 이하로만 할당된다고 과장하지 않는다.

## 5. acceptance와 usage 자기참조를 닫는 절차

기존 `control::run`(control.rs163–195)은 closure 종료 뒤 마지막 poll을 하고 그제야 usage를 반환한다. 이 반환값을 받은 다음 receipt의 fallible serialization을 하는 방식은 PLAN의 mandatory-ready-before-acceptance 경계를 명확히 닫지 못한다. 새 공개 상태기계 없이 private `usage_snapshot`과 고정 header reservation으로 해결할 수 있다.

1. 새로운 attempt control을 **fork/load/constructor 이전**에 설치한다. A의 gameplay state·accepted counters·receipt는 untouched다. trusted limits와 request admission을 수행한다.
2. private opaque candidate fork에서 기존 public API를 호출한다. domain error가 나더라도 sticky control error가 있으면 최종 failure precedence는 control이다. 정상 결과로 normalize된 내부 failure도 최종 poll이 잡아야 한다.
3. Receipt phase에서 resolved input/public output 같은 가변 mandatory body를 capped writer로 encode하면서 동일 deterministic work를 charge한다. 새 aggregate vectors/owned Values의 공간도 이때 prepare한다.
4. usage/indices/fixed coverage를 담는 header의 **고정 최대 serialized 크기**를 먼저 admit하고 charge한다. 각 u64 최대20자리, 고정 키/enum 최대 문자열/옵션의 최대 형태를 포함한 검증 가능한 상수다. 행 수에 비례하는 optional trace body는 이 reservation에 들어가지 않는다.
5. private usage snapshot을 얻는다. header를 그 reserved 영역에 **추가 execution charge 없이** encode한다. 고정 primitive schema만 들어가므로 계산한 ceiling 안에서 끝난다. self-usage의 가변 byte 길이를 다시 work로 charge하지 않는다. 아직 failure 가능한 preparation은 모두 final poll 전에 끝난다.
6. 마지막 control acceptance poll 뒤에는 prepared aggregate와 A를 infallible swap한다. 반환되는 run usage와 header usage가 일치해야 한다. 이후 cancellation/delivery 오류로 accepted 결과를 Err나 rollback으로 바꾸지 않는다.

고정 header reservation은 trace off/on에 똑같이 부과한다. 선택 trace의 실제 serialize/copy/row-retention은 execution charge·mandatory writer·binding Value table을 호출하지 않는다. trace가 overflow하면 고정 coverage status/dropped counts/gap으로 표현하며 필수 receipt slot을 더 소비하지 않는다. 외부 transport 실패는 accepted semantic result의 delivery failure다.

`validate()` 자체가 이미 run을 설치한다(authoring/mod.rs103–108). 외부 acquisition+prepare를 한 attempt로 만드는 convenience load는 `validate()`를 중첩 호출할 수 없다. input-limit 검증/production load/token packaging을 private in-scope helper로 추출하고 공개 validate와 load가 각각 한 번만 run을 소유한다. 또는 이미 validated content를 받는 prepare/load는 새 별도 attempt라는 사실과 validation usage를 receipt에 구분한다. 유효한 token도 startup control을 건너뛰지 않는다. B preparation prefix는 하나의 outer scope에서 내부 candidate staging을 순차 호출하여 내부 rollback이 fuel를 초기화하지 못하게 한다. interactive step attempts의 누적 성공 work는 별도의 checked session aggregate에 반영하고 실패는 A 집계를 보존한다. 총 session cap과 per-attempt cap을 혼동하지 않는다.

## 6. B load와 실제 time-zero 준비

새 `CoreApplication::new(B.token().clone(), explicit_seed)`와 default profile snapshot부터 독립 B를 만든다. request가 적은 finite prefix의 start/VN/starter/map/select/confirm Commands만 실행한다. start_new_game에서 `execute_with_generation` 한 번이 성공했다고 B battle-preparation 성공은 아니다.

현재 실제 startup은 events/combat.rs219–220 `BattleCore::new_from_database` 및 `start_battle_execution`이다. live battle UUID는 world/combat.rs683에서 나중에 할당되고 686에서 active에 설치된 뒤 688에서 elapsed0을 수행한다. capture는 startup 전에 설치되어야 하며 준비 중 provisional observation은 실제 live entry 확정 때 battle UUID에 결속한다. Loader의 generated-preview 검증에서 생긴 battle을 이 trial의 실제 entry로 세지 않는다.

prefix 성공 뒤 실제 live battle-start/timezero marker가 있는지 검사한다. 단순 `Run.game_state == in_battle`는 충분하지 않다. timezero에서 바로 terminal이면 world/combat.rs1566→1393–1406가 active를 제거한다. 실제 setup/timezero를 마친 terminal은 capture로 증명할 수 있으며 readiness를 in_battle 문자열로 강제해 거짓 실패를 만들지 않는다. 준비가 battle에 도달하지 않았으면 `trial_battle_not_prepared`와 실제 last public result/snapshot을 반환하고 A를 교체하지 않는다. 필수 output/cancel/work 실패도 B 전체를 drop한다. 실패 receipt는 반환값으로만 소유하고 A의 receipt/step numbering/quota에 붙이지 않는다.

## 7. observation과 terminal receipt의 독립 경로

현재 log는 event_log.rs42–44의 count/byte/span retention을 가지며 effect source는 752–756의 serde(skip)다. release CombatBattleState의 event_log 자체도 world/combat.rs1380–1381에서 빠진다. 그러므로 after-step log 재검색만으로 actual occurrence/full delayed/terminal trace를 구현할 수 없다.

최소 안전한 소유권은 실제 domain draft에 따라 clone/drop되는 private observer component다. BattleCore에는 fixed semantic counters와 optional bounded owned rows, GameCoreState에는 완료 battle capsule을 보유하고 terminal 전에 옮긴다. 이 private metadata는 wire/save/RNG/UUID/projection revision과 무관하며 serde/V6 밖에 둔다. observer 상태의 clone은 깊은 소유 또는 immutable sharing이며 A/B shared mutable Arc sink는 안 된다. 기존 core draft rollback과 함께 provisional observation도 rollback되므로 임의 TLS 전역 output sink보다 commit 의미가 분명하다. TLS를 쓰더라도 설정/현재 attempt 연결만으로 제한하고 내부 draft failure의 row를 accepted로 남기지 않는다. Trace storage clone/encode를 gameplay execution work에 합산해서는 안 된다.

실제 source 결속은 commands.rs481–518 `process_skill_commands`가 유지하는 authored+cast source에 기대고, actual delta는 DamageRuntime `execute_committed_damage`의 actual HP change 및 modifier/HP producer에서 capture한다(commands.rs598–645, damage_runtime.rs179–280). 단순 command 생성/targets 존재/append log를 실제 효과 성공으로 표시하지 않는다. 0 delta, missing target, skipped op는 실제 처리 status로 구분할 수 있으나 관찰하지 않은 값을 추론하지 않는다. delayed area/projectile가 보유하는 source가 실제 apply 시점에 전달되는지는 wiring 구현 때 해당 producer를 다시 확인해야 한다(이번 읽기는 전체 leaf 검증이 아니다).

occurrence key 제안은 `(trial generation, battle identity, semantic application ordinal)`이고 ordinal은 실제 적용 지점에서 trace off에도 동일하게 증가한다. definition/action/effect/caster/target/cast_seq/time/delivery instance를 payload/correlation으로 보존한다. cast+effect만으로 반복 tick/다중 target을 식별하지 않으며 event-log seq를 새 semantic allocator로 쓰지 않는다. 현재 cast/ability log sequence는 원래 값의 correlation으로만 보존한다. 같은 immutable source를 여러 실제 occurrence가 공유해도 된다.

Terminal capsule은 `active_battle=None` 전에 마지막 actual outcome/time/semantic counter/source observation/terminal public projection을 candidate 소유로 capture한다. `CombatResult` 이후 receipt 조회가 제거된 ActiveBattleSession을 다시 읽지 않게 한다. optional trace off면 mandatory receipt의 actual outcome/result와 fixed coverage는 그대로 있고 trace rows는 disabled 상태다. row cap/byte cap/observer encoding 실패는 optional gap status다. mandatory public result와 initial/resolved-input/usage body의 admission은 같은 값이어야 한다. 기존 `record_event_log`는 sequence exhaustion 시 바로 반환(recording.rs13–16)하므로 semantic observation을 그 호출 뒤에만 연결하면 안 된다.

## 8. describe/explain의 정직한 최소 내용

Description은 `ContentContract::current`, `PackManifest::SUPPORTED_CAPABILITIES`, `CatalogKind::ALL`, command/snapshot registry, 실제 trusted ceilings와 구현한 clock/binding grammar를 빌려 직렬화한다. DefinitionKind에는 현재 ALL이 없다(raw.rs143–155). 필요하다면 기존 definitions! macro에서 같은 registry를 생성하며 수동 이중 목록을 두지 않는다.

Literal edit 지원은 ability/authoring.rs157–180 `EFFECT_SUPPORT`/`effect_support`의 현재 Instant Damage/Heal와 PropertyDescriptor를 그대로 사용한다. 이 matrix는 literal editing support이지 core의 전체 effect execution 가능 목록이 아니다. trace가 가능한 모든 delivery를 literal-edit editable로 확장해 표기하지 않는다.

Explain은 ContentSourceMetadata의 `definitions/source_for/writers`(content_pack/mod.rs25–50)를 사용한다. DefinitionKey로 final winner와 Add/Replace/Remove 관계를 반환한다. 선택 Remove는 final definition이 없다는 사실을 명시한 유효한 explanation이며 metadata가 없는 unknown key와 구별한다. skill detail은 `database.skill_data.get_by_id`와 `effect(&EffectSite)` 및 `AuthoredSkillEffect::source`에서 actual authored site/origin/literal/support를 제공한다. typed compiled graph detail은 boss_runtime_data.rs809–830 `get_pattern/get_actor_behavior/get_encounter`, compiled struct의 실제 id/index/tasks/transitions 필드를 borrowed explicit Serialize projection으로 내보낼 수 있다. 내부 index는 compiled_index로 이름 붙인다.

**공통 위험:** WriterReceipt는 현재 source를 갖지 않으며 resolver.rs59–65의 semantic fingerprint 입력이다. source 주소를 여기에 덧붙이면 경로/원문 위치가 semantic hash에 들어가는 회귀다. 비선택 writer/Remove의 exact source가 필요하면 hash와 분리된 bounded immutable sidecar에 원래 Writer.source를 유지해야 한다. 그 전에는 source precision을 unavailable로 명시한다. 최종 source를 비선택 writer source로 재사용하지 않는다.

BossRuntimeDatabase의 `authored`와 authored_catalogs_for_fixture_identity는 default release에서 없다(boss_runtime_data.rs782–807). compiled graph를 authored text/line/column이라고 설명하거나 fixture accessor를 production으로 호출할 수 없다. source ordinal은 retained SourceTable이 실제 알려주는 definition 위치까지만이다. Query error는 known source/typed selector/phase와 현재 available precision을 보존하고 파일 재오픈·Display 파싱·compiled index→원문 위치 추정을 하지 않는다.

## 9. 다음 구현 전에 닫을 구체적 gap

| ID | 현재 근거 / gap | 최소 fix |
| --- | --- | --- |
| TPD03-01 | run usage는 final poll 뒤 반환; receipt work 자기참조 | fixed primitive header reservation + private usage snapshot; 모든 fallible 준비 후 마지막 poll |
| TPD03-02 | validate가 TLS run을 소유; load가 감싸면 nested attempt | private in-scope load helper 또는 token prepare의 별도 attempt를 명시; prep prefix는 한 scope |
| TPD03-03 | CoreApplication Clone 없음; V6는 live clone 아님 | crate-private complete opaque fork; clone-control와 mutable observer 소유권 확인 |
| TPD03-04 | actual startup은 active/timezero보다 이르고 즉시 terminal 가능 | startup capture + live-entry marker + 제거 전 terminal capsule; in_battle-only readiness 제거 |
| TPD03-05 | writer source 없음; writer serialization은 semantic hash 입력 | unknown 정직 표기 또는 hash와 별도 source sidecar |
| TPD03-06 | compiled boss type는 Serialize/retained authored source 아님 | actual compiled borrowed view와 precision labels; default release fixture fallback 금지 |
| TPD03-07 | arbitrary observer TLS sink는 inner draft rollback과 별개 | 실제 draft 소유 observer 또는 commit-aware capture; optional charge가 mandatory에 섞이지 않도록 명시 |

이 표는 현 design을 닫기 위한 구현 의무다. 아직 없는 session/trace/CLI 자체를 결함 또는 완료 실패로 판정하지 않는다. 구현 뒤 필요한 핵심 검증은 exact boundary/one-over/malformed pointer; same public call parity/HostTick None; known-valid B startup failure와 즉시 terminal; rejected candidate의 A byte/identity/receipt/quota 보존; trace off/on-only overflow에서 동일 mandatory usage/result/continuation; terminal/drop 후 retained source/lifetime; default release 실제 consumer다. 이번 보조는 실행하지 않았다.

## 10. source capture와 한계

첨부 JSON은 실제 읽은 source spans와 SHA-256을 담는다. 첫 hash와 종료 hash가 모두 있는 파일에만 unchanged/changed를 판정한다. 나중에 읽은 일부 파일은 종료 hash 한 개뿐이며 원자적 snapshot 또는 과거 baseline 동일성을 주장하지 않는다. Root의 다른 leaf 수정은 이번 한정 검토로 인증하지 않는다. Goal/PLAN과 이전 보조 reports의 완료 전제를 두지 않았고 새 정책 모호성보다는 위 기술 seam의 명시적 처리가 필요한 상태로 판단한다.

종료 source manifest:

| Source | SHA-256 | Spans / capture |
| --- | --- | --- |
| `core_enhanced/docs/goals/content_authoring_tool_ports/P2G_50_GOAL_SET_001.ko.md` | `bb0b69cfebceb17fd13374826e1dbb6ba6e38e711cb574acc29ca61ca2745f98` | 1-104, 121-149; unchanged_between_hashes |
| `core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/PLAN.md` | `d427b3c5afa03bb1f317cb01385317ecc1a5959455a12c8fafafcdd6f8e27026` | 1-87, 102-205; changed_since_first_hash |
| `core_enhanced/src/application.rs` | `636b3c9cc15ae2adfa39a15392e0dee22ee7c6683885087f9cc77cefda66081a` | 1-218, 220-359, 411-456, 562-590, 686-726, 781-1040; unchanged_between_hashes |
| `core_enhanced/src/authoring/mod.rs` | `fe4a6d69b11d0fcedc4e49625c760850dd12517cfd8a3eb6ca1da1c03c941ca6` | 1-116; unchanged_between_hashes |
| `core_enhanced/src/authoring/control.rs` | `ad20a4e3a75a9bb16b402f50631559ae5028b760411a11b612d8945944461f8d` | 26-139, 142-296, 299-338, 422-499; unchanged_between_hashes |
| `core_enhanced/src/game/data/content_pack/mod.rs` | `cdca9db0f33f6590406162db40c8a238fe195f55c5e4754383fb86065fa03fab` | 1-143; unchanged_between_hashes |
| `core_enhanced/src/game/data/content_pack/raw.rs` | `e6db361ad983b1fcb4f0a33b827b99ec2ecf4f62dfbbd51e578dfa874b71b7d7` | 49-105, 138-225; single_close_hash_only; source was read earlier, no atomic stable snapshot claimed |
| `core_enhanced/src/game/data/content_pack/schema.rs` | `3c44c1cb3fc4df3ef5ebea29cfbdb939a3614735ec77e2391f2ee4b4a1fd0616` | 1-35, 67-100, 154-198; single_close_hash_only; source was read earlier, no atomic stable snapshot claimed |
| `core_enhanced/src/game/data/content_pack/resolver.rs` | `984b7c782eafe3aadbc244b56570505386ffe0acb9b461f56102687682569399` | 1-83; single_close_hash_only; source was read earlier, no atomic stable snapshot claimed |
| `core_enhanced/src/game/ability/authoring.rs` | `c4ca67ceac4502c2c31c8c91a584e09dff3c87fd8976269d436dbddbc2e08e1a` | 1-181; unchanged_between_hashes |
| `core_enhanced/src/game/data/skill_data.rs` | `8a1cccf346cc3b45755a92a45e41be92deb5a0e641a0f519977dc6a2c7c62fb5` | 1-248; unchanged_between_hashes |
| `core_enhanced/src/game/data/boss_runtime_data.rs` | `e75f62fbe07326411ac2625f1285b2dd7551592f29850976616c48fed6ab1cdf` | 519-554, 642-648, 714-741, 782-863; unchanged_between_hashes |
| `core_enhanced/src/game/world.rs` | `fc72d1029c33c560f918dc2487071c5720ad231d6983b9aa66dc15e4d5ee75ed` | 31-176, 210-290; single_close_hash_only; source was read earlier, no atomic stable snapshot claimed |
| `core_enhanced/src/game/world/state.rs` | `129a4063f7e448b5795fd370a22a997f66eb09cd3029fb87d41fee892cf34bb0` | 1804-1836; single_close_hash_only; source was read earlier, no atomic stable snapshot claimed |
| `core_enhanced/src/game/world/application.rs` | `eda3e77e4509368aec58228b8070b5d7edfdab6fc6f98ae036cb45e97ea7eafd` | 128-220; unchanged_between_hashes |
| `core_enhanced/src/game/world/active_run.rs` | `c344f0fff0aa28e3f8afb9b5a48c2c706855dbca13c5c53342aa14d2b7d52b20` | 174-211, 219-278; unchanged_between_hashes |
| `core_enhanced/src/game/world/combat.rs` | `23d440b58516258e9e174d586c6d06a675f9ce8c9b21534cde102574b2feb35f` | 636-696, 1317-1410, 1420-1439, 1540-1645; unchanged_between_hashes |
| `core_enhanced/src/game/events/combat.rs` | `d88719417886eea018ae08a66c750884f163051f160d0b9b21ece3c1eddffc66` | 181-230; unchanged_between_hashes |
| `core_enhanced/src/game/battle/recording.rs` | `e3c1afedaf9b5ec61c76d415e2123dbea8a9c6d9213da5393dddfaf0e6beaf2d` | 1-38; single_close_hash_only; source was read earlier, no atomic stable snapshot claimed |
| `core_enhanced/src/game/battle/event_log.rs` | `e20c7ff9ab45a4797c615b744dbfc15848267af40b040f04a192ba6362ad2c5a` | 20-68, 748-775; single_close_hash_only; source was read earlier, no atomic stable snapshot claimed |
| `core_enhanced/src/game/battle/skill_source.rs` | `4fff389728a0dcb34fa50b56108e8790a750205fa0d391c21e65fd4553ca50d1` | 1-29; single_close_hash_only; source was read earlier, no atomic stable snapshot claimed |
| `core_enhanced/src/game/battle/core/commands.rs` | `b32f1b9a9f47c7124d3944cf6bd409a6e514fb489ee36b9cbba8ea251177a542` | 1-135, 251-345, 453-520, 598-680; unchanged_between_hashes |
| `core_enhanced/src/game/battle/core/damage_runtime.rs` | `c2faa80ae08c04c294defdc2474765128b4d336e0be51d3d57145ae591e43cbf` | 179-280; unchanged_between_hashes |
| `core_enhanced/src/game/behavior.rs` | `c38657dd64aeae18fbb6a4998cfeb65cc45b015a897f379360236b2b4c3e2552` | 1978-2025; single_close_hash_only; source was read earlier, no atomic stable snapshot claimed |
| `core_enhanced/tools/battle_kernel_api_audit/valid/src/main.rs` | `029179ae0b7d715ae01001e377800ec128276616be1406204750be94915cd745` | 40-120; unchanged_between_hashes |
