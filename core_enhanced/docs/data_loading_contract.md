# Data Loading Contract

이 문서는 core live RON authoring, external content-pack loading, loader ownership, fixture, validation의 canonical 계약이다.
Runtime에서 검증된 data를 소비한 뒤의 결정론과 event/snapshot 경계는 [core_runtime_contract.md](core_runtime_contract.md)를 따른다.

## 문서 책임

| 구분 | 내용 |
| --- | --- |
| 문서 등급 | Data canonical |
| 소유 범위 | live RON authoring strictness, external pack entrypoint, loader ownership, schema publication, fixture 경계, validation gate와 live file 색인 |
| 소유하지 않는 범위 | 검증된 data를 소비한 뒤의 gameplay 의미, runtime state transition, 타겟팅 실행 알고리즘, Unity wire payload |
| 함께 확인할 문서 | [game_rulebook.md](game_rulebook.md), [core_runtime_contract.md](core_runtime_contract.md), [skill_target_contract.md](skill_target_contract.md), domain data module과 validation test |

## Source Of Truth

이 문서가 live RON authoring·loading·validation 경계의 source of truth다. Runtime code와 live RON/data는
이 계약을 구현하며, 다른 경우 data-loading implementation drift로 기록하고 문서에 맞게 수정한다.
Live RON은 이 계약이 허용하는 schema 안에서 exact authored ID·value·catalog membership을 제공한다.

Core 콘텐츠 publication entrypoint는 [content_pack::load](../src/game/data/content_pack/mod.rs)다. Host는 trusted root,
명시적인 `ContentSelection`과 trusted `InputLimits`를 전달한다. 현재 production release는 외부 파일만
읽으며 CWD 탐색, embedded 보충, 이전 성공 세대 반환으로 실패를 대체하지 않는다.

- `ValidatedGameData`는 database, `ContentIdentity`, source byte receipt를 함께 소유하는 opaque strong
  reference다. Public API는 읽기 전용 catalog/identity/receipt만 노출한다. Generation은 최종 검증 이후에만
  반환되고 전역 registry 없이 마지막 strong reference가 해제될 때 회수된다.
  `source_metadata()`는 동일 external resolution의 final definition/source와 actual Add/Replace/Remove
  writer 관계를 borrowed readonly 형태로 제공한다. Selected Remove는 writer 목록에 남고 final definition은 없다.
  원 파일을 변경해도 이미 게시한 metadata는 변하지 않는다. 기존 SourceTable/writer receipt를 generation으로
  옮겨 보존하며 raw RON text나 별도 mutable database는 유지하지 않는다. Bare fixture database 변환에는 이 출처가
  없어 None이다. Writer receipt의 기존 semantic hash 입력·순서는 유지한다.
- 각 선택 디렉터리의 `pack.ron`은 pack ID/version, Global 또는 Named namespace, contract,
  required capabilities, dependencies, typed catalog file 목록과 명시적 operations를 모두 요구한다.
  Unknown field와 duplicate pack ID는 거부한다. 현재 contract는 pack format 1, catalog schema 6,
  authoring 1, execution 3, fingerprint 1이며 exact match가 필요하다. 지원 capability는
  `gameplay_catalogs_v2`, `literal_effect_authoring_v1`, `boss_runtime_v1`, `summoned_units_v1`,
  `typed_resources_v1`, `structured_skill_flow_v1`, `source_owned_sustained_effects_v1`, `expedition_lifecycle_v1`이다.
- 전체 파일을 bounded owned snapshot으로 확보한 뒤 typed raw definition을 읽는다. Structural schema와
  unsupported opcode는 나중에 폐기될 writer에서도 검증한다. Catalog별 조기 참조 해석 대신 모든
  Add/Replace/Remove를 해결한 최종 definition에서 참조·preset·compiled domain·generated combat
  preview/briefing을 검증한다. 실패 후보는 token을 반환하지 않는다.
- `CatalogKind::ALL`/`official_path()`는 공식 26개 catalog의 exact adapter inventory다. 기존 fixture source
  inventory와 실제 공식 source 파일 집합을 검사하는 테스트도 유지한다. 신규 catalog는 raw typed adapter와
  최종 assembly/validation을 함께 확장해야 한다. Generic JSON field patch로 gameplay를 구성하지 않는다.
- Source 진단은 owned file path, domain, object, action/effect, field locator와 cause를 보존한다. Replacement가
  원래 실행 슬롯을 유지해도 진단은 winning 원문 파일/배열 위치를 가리킨다. 여러 파일이 원인인 aggregate
  오류는 contributing source를 명시하며 임의의 한 파일로 오인시키지 않는다.
  `LiveRonError.related_sources`는 각 contributor의 owned `source_file`, `object_id`,
  `field_path`를 제공하며 Display에도 표시한다. Generated preview의 wave 오류는 실제 실패한
  wave/field를 encounter 원문 또는 활성 RunPolicy의 `tuning.extra_waves` 원문에 결속한다.
  함께 소비한 encounter·tuning·선택된 전장 template와 적용 가능한 revival/boss binding,
  해당 compiled boss encounter가 사용하는 behavior·pattern 정의들은
  관련 출처로 유지한다. 단일 실패 필드를 제공하지 않는 setup 오류에서는 encounter를 검증
  문맥으로 유지하고 관련 입력을 함께 제시한다. 이 진단 정보는 성공한 generation의 semantic
  fingerprint나 저장 authority에 포함되지 않는다.
- `LiveRonError`는 동일 오류를 `Serialize`할 때 producer의 stable `code`와 `phase`를 제공한다.
  Phase/code는 acquisition/`content_acquisition`, parse/`ron_parse`, structure/`content_structure`,
  resolution/`content_resolution`, final_validation/`content_validation`, compile/`content_compile`,
  generated_setup/`generated_setup`이다. Manifest contract/capability 불일치는 structure에서
  `content_contract_unsupported`/`content_capability_unsupported`와 실제 `expected`/`actual`을 제공한다.
  `pack_id`, `catalog_kind`, `definition_kind`는 decoded manifest·owned receipt·실제 resolved key가
  제공하는 경우에만 채운다. 관련 출처에도 같은 metadata를 보존한다. Manifest를 파싱하기 전에는
  디렉터리 이름을 pack ID로 사용하지 않는다. RON typed Parser 오류의 `position`은 실제 one-based
  line/column이며 `parser_reason`은 typed reason의 설명이다. Serde Message와 위치가 없는 오류는
  `position: null`이다. Expected/actual·definition/span을 message에서 추측하지 않는다.
  이 정보는 기존 Display/cause와 validation 의미를 보완하며 CLI·시험 세션의 완료를 뜻하지 않는다.

### Mod grouping and approved host admission (U11)

일반 local launcher와 `game_server`는 [AdmissionSession](../../game_server/src/mod_admission/session.rs)이
검증한 `ApprovedHost`만 사용한다. 위 production loader와 public authoring/trial은 실제 동일 콘텐츠를
진단·시뮬레이션할 수 있지만 private runtime approval을 발급하지 않는다.
실행 설정과 명령은 [CMVA 사용 가이드](tools/cmva_workflow.ko.md)를 따른다. 2026-10-12 [실제 literal entry 검증](goals/core_mod_validation_apply/execution/single_goal/test_parts/runtime001/evidence/reviewed_adoption_functional002/24__public_entry__literal_smoke.json)은
launcher 정상 run/Continue와 server 재시작 Continue·exact checkpoint 보존·invalid content의 persistence 이전 거절을 확인했다.

- `ModRegistry`는 stable mod ID별 pack 묶음, version, dependencies와 선택을 소유한다. Trusted base와
  dependency closure로 각 mod를 실제 검증한 뒤 최종 조합을 별도 실제 실행한다. 한 mod의 acquisition/
  content/coverage 실패는 독립 mod의 결과를 지우지 않으며 그 종속 항목은 `DependencyHeld`다.
  Individual winner는 실제 captured closure에 포함된 writer만 투영한다. Reduced final selection은 원
  authored winner를 그대로 유지하므로 held winner를 다른 writer로 자동 대체하지 않는다.
- Candidate는 선택한 전체 bytes와 source membership을 한 번 frozen capture하고 이후 individual/final
  projection·Lab·교체는 그 capture를 사용한다. Trusted base 재사용은 full source receipt와 selection/settings
  equality를 bounded 비교한 뒤 immutable validated publication을 공유하며 최종 publication 검증은 생략하지 않는다.
  Caller 파일 변경·삭제가 retained candidate를 바꾸지 않으며 다른 base bytes는 후보 획득을 거절한다.
- Content semantic identity, exact source receipts, candidate/required mod version·digest, trusted effective runtime
  profile, suite revision/hash, actual Engine build와 worker pair/environment를 구분한다. Semantic identity가 같다는
  사실만으로 source/profile/build의 승인·cache authority가 일치하지 않는다. 결과 DTO나 공개 generation token으로
  `ApprovedGeneration`을 만들 수 없으며 private issuer가 실제 completed evidence를 확인해 발급한다.
- Versioned `coverage_support()`/`mod_launcher --cmva-coverage-support`는45개 kind의 operation 지원을 조회한다.
  현재 Add/Replace consumer는 Employee, Equipment, SkillFragment, Relic, PveEncounter, RunPolicy, Skill,
  Buff, IndependentObject의9종이다. Remove는 앞5종이며 frozen trusted-base의 실제 positive reference preparation,
  candidate의 정확한 removed-reference refusal와 surviving same-kind normal preparation을 모두 요구한다.
  다른 mod에서만 도입된 definition 등 counterfactual이 없는 removal은 `Incomplete`다.
- 자동 coverage는 변경한 모든 definition/site와 trusted suite의 추가 요구를 합친다. Catalog parse, trigger 또는
  serialization만으로 actual consumption/effect와 terminal/reclamation을 대체하지 않는다. LegalNoop·ineligible는
  별도 witness이며 positive effect를 대신하지 않는다. 합법 empty/zero-site shape와 아직 지원하지 않는 consumer는
  `Incomplete`로 보존하며 valid authored shape를 금지해 승인 조건을 우회하지 않는다.
- 실제 gameplay/runtime 오류는 `ContentFailed`다. 필요한 eligible target/trigger/coverage, setup·IO·worker transport,
  measurement/parity completeness가 없으면 `Incomplete`다. Complete measurement의 criterion 초과는 `ContentFailed`며,
  동시에 부족한 coverage가 있으면 양쪽 reasons를 보존한다. Individual PASS를 합쳐 final PASS를 추정하지 않는다.
- 승인 Lab은 실제 production preparation·command·clock·effect 경로를 plain/instrumented 별도 child에서 실행하고
  nonce/candidate/build/mode-bound 결과를 확인한다. Crash·timeout·잘못된 envelope를 complete sample로 바꾸지 않는다.
  Requested heap·RSS·native work는 서로 다른 measurement며 plain native0은 uninstrumented sentinel다.
- Criterion은 해당 executable/context의 실제 반복 측정·parity·coverage를 검토한 trusted host가 유한 versioned
  limits로 채택하고 private issuer가 seal한다. Cache는 exact candidate/source/build/profile/suite/environment/method/
  criterion 문맥을 분리하고 bounded private HMAC-authenticated record만 재사용한다. Cancelled/incomplete/stale 결과와
  public report는 PASS cache나 승인으로 승격하지 않는다. Accepted rename 이후 IO 실패는 `RecoveryPending`이며
  그 기록의 복구도 현재 exact authority 확인을 통과해야 한다.

### Common authoring validation

Trusted host profile의 로드·실행 계약은 [runtime safety profile](core_runtime_contract.md#trusted-runtime-safety-profile)을 따른다.
`content_pack::load_with_runtime_safety(root, selection, input, Arc<RuntimeSafetySettings>)`는 parse guard부터
compiled catalog publication까지 같은 profile을 전달한다. 기존 `load`는 embedded profile wrapper다.
별도 `InputLimits.value_expr`는 profile의 parse 한도와 교집합으로 적용한다. `ValidatedGameData::admission_identity`
는 기존 content identity와 effective profile hash를 함께 제공하며, source receipt와 semantic identity는 유지한다.

`authoring::validate_with_runtime_safety`는 `InputLimits`와 profile을 받고 authoring execution 한도를 profile에서
만든다. 같은 content의 trial에는 `TrialLimits::with_runtime_safety(content.token().runtime_safety())`를 전달한다.
이 helper가 execution·total-work·receipt 한도를 연결하며 trace/amount 옵션은 별도다. 기존 `validate`/`TrialLimits`
typed API는 신뢰한 호출자의 명시 한도를 그대로 지원한다. Describe/explain/preview는 retained generation/plan을
읽는다. Validation/description/effect-explanation/trial receipt의 `runtime_safety_sha256`로 실제 profile을 식별한다.

[`game_core::authoring::validate`](../src/authoring/mod.rs)는 같은 production loader에 명시 root/selection,
trusted `AuthoringLimits`와 `CancellationToken`을 전달한다. 성공한 `AuthoringContent`는 동일 generation token과
이번 검증의 `ExecutionUsage`를 소유한다. Content 오류는 기존 `LiveRonError`를 보존한다. 요청 내 실행 제한·취소가
관찰되면 `AuthoringError::Execution`의 stable code/phase/site/limit/requested를 반환하며 token을 게시하지 않는다.
최종 snapshot 재검증 오류도 이미 decoded된 pack/catalog를 보존한다. Writer 충돌의 aggregate primary는 pack을
임의 선택하지 않고 실제 writer source를 related sources에 담는다.

제작 검증의 동기식 제어 scope는 gameplay clone 외부에서 시도한 작업량·첫 실패를 유지하고 종료 시 정리한다.
Nested top-level 진입은 새 budget으로 바꾸지 않고 거절한다. Raw definition 수, source/writer metadata의 JSON encoded
bytes, resolver 작업, generated setup의 enemy count 확장과 startup/queue 경계에 제한을 연결한다.
Metadata byte 수는 heap byte나 source receipt 전체 크기가 아니다. Loader의 모든 내부 CPU/할당을 정확히 계측한
수치로 해석하지 않는다. 기본·hard ceiling과 현재 연결 범위는 [control](../src/authoring/control.rs)을 따른다.
이 API는 개인 저장 파일을 열거나 기존 application generation을 교체하지 않는다.

### Prepared authoring trials

[`TrialSession::prepare` / `simulate`](../src/authoring/trial.rs)는 검증된 `AuthoringContent`와 bounded `TrialScenario`를
소비하여 실제 application 명령·snapshot·clock을 실행한다. 세션의 describe/explain은 소비된 동일 세대를 읽는다.
준비 전체가 성공해야 세션이 게시되며 replace 실패는 기존 세대를 유지한다. 실행 실패의 SimulationFailure는 마지막
accepted session을 보존한다. 유한 입력 완료는 전투 승리를 뜻하지 않는다. Borrowed receipt/관찰/source/trace 및 별도
Inspection scope의 snapshot/reconnect와 closed typed Preview가 공개되어 있다. 별도 `render_error`/`render_trial_failure`는
원 진단을 빌려 독립 한도 안에서 출력한다. 한도·수명은 [runtime 계약](core_runtime_contract.md)을 따른다. 공통 CLI는 아래 실행 경로에 연결되어 있으며 전체 외부 인수·비용·fresh 감사는 아직 미완료다.

### Common default-release authoring CLI

[`author_content`](../src/bin/author_content.rs)는 명시한 외부 pack selection의 전체 packs/winners schema와 공개 제작 API를 사용한다.
Validate는 동일 검증 세대의 identity/source receipt/validation usage를 bounded 문서로 출력한다. Describe/explain은 동일 immutable
metadata를 읽고 simulate는 명시 finite 시나리오만 실행한다. 자동 briefing/starter 선택은 production CLI에 없다.

아래 [외부 비교 도구](../tools/authoring_cli_external_probe.py)는 별도 default-release Rust 소비자가 실제 public 결과로 만든 시험
시나리오를 저장하고 CLI와 비교한다. 출력 디렉터리는 새 경로여야 한다. 단순 실행 예시를 위한 fixture 선택은 이 시험 소비자에만 있다.

```sh
cargo build -p game_core --release --bin author_content --bin package_official_content
python3 core_enhanced/tools/authoring_cli_external_probe.py --repository . --output-dir /tmp/r03-cli-example
./target/release/author_content validate /tmp/r03-cli-example/packs /tmp/r03-cli-example/api/selection.json
./target/release/author_content describe /tmp/r03-cli-example/packs /tmp/r03-cli-example/api/selection.json
./target/release/author_content explain /tmp/r03-cli-example/packs /tmp/r03-cli-example/api/selection.json /tmp/r03-cli-example/api/explain.json
./target/release/author_content simulate /tmp/r03-cli-example/packs /tmp/r03-cli-example/api/selection.json /tmp/r03-cli-example/api/scenario.json
```

Selection/explain 파일64KiB, scenario8MiB와 내부 parser 한도를 적용한다. 성공 문서·accepted receipt는 stdout, original typed
실패는 stderr에 출력한다. 종료0/1/2/3은 각각 성공/library 실패/CLI 입력 오류/transport 또는 진단 delivery 실패다. 선택
`--trace-output NEW_PATH`는 simulate 뒤 최종 active/last-terminal window를 별도 새 파일로 내보낸다. 전체 과거 trace가 아니다.
세부 한도와 실패 수명은 [runtime 계약](core_runtime_contract.md)을 따른다. 전체 R03 인수는 CLI 존재만으로 완료되지 않는다.

모든 CLI command는 `--runtime-safety PATH`를 받으며 simulate의 `--amount-observation`, `--trace-output`과
함께 사용할 수 있다. 지정 파일을 콘텐츠/selection보다 먼저 bounded-read·검증한다. 잘못 지정한 파일은 exit2로
거절하며 embedded 기본값으로 fallback하지 않는다. 생략하면 embedded profile을 사용한다.

실제 `game_server`는 `content.trusted_root`, `content.selection_path`를 필수 host 설정으로 받는다.
선택 `runtime_safety_path`는 TOML root 또는 `APP__RUNTIME_SAFETY_PATH`에 지정한다. 환경 prefix 뒤와 nested
설정 경로 사이는 현재 host config adapter의 `__` 구분자를 사용한다. 상대 경로의 기준은
프로세스 working directory다. 서버의 live embedded fixture fallback은 없다. 저장 경로와 profile은 host 소유다.
저장소 root에서 다음과 같이 공식 팩을 준비하고 `game_server` 디렉터리에서 서버를 시작할 수 있다.

```sh
cargo run -p game_core --bin package_official_content -- game_resources content_packs/official
printf '%s\n' '{"packs":["official"],"winners":[]}' > content_selection.json
cd game_server
cargo run -p game_server --bin game_server
```

`content_packs` 부모 디렉터리는 먼저 만들고, 기존 pack/selection을 덮어쓰지 않도록 배포 위치를 선택한다.
기본 host 설정은 `../content_packs`, `../content_selection.json`을 사용한다. 서버는 settings→content→persistence→
actor 순서로 bootstrap하며 effective profile hash를 startup log에 남긴다.

### Relics catalog and immutable variants

[`CatalogKind::Relics` / `DefinitionKind::Relic`](../src/game/data/content_pack/raw.rs) reads the
strict `(relics: [...])` catalog at `data/relics/base.ron`. Relic participates in the same typed
Add/Replace/Remove resolution, final-source diagnostics, semantic identity and source receipt as the
other catalogs. Schema5 includes this catalog; there is no separate Relic capability string. The
official catalog is currently empty and does not provide invented balance definitions.

[`RelicDefinition`](../src/game/data/relic_data.rs) requires `id`, non-nil `uuid`, `name`,
`description`, `rarity`, `allow_duplicates`, `effects` and positive `dismantle_equipment_dust`.
`incompatible_with` defaults to an empty list and `corruption` to absent. Definition IDs and UUIDs
must be unique; incompatible IDs must exist, differ from self and not repeat. Unknown fields are
rejected in definitions, effects, selectors and corruption payloads.

Each effect has a unique `id`, selector, and typed integer `flat`/`percent` coefficients. The selector
defaults to `AnyDamage`; `SkillDamage` and `BasicAttackDamage` are the other targets. `all_tags`,
`damage_type`, `equipment_id`, `fragment_id` and `form_id` narrow the same selector conjunctively.
Tags and IDs are validated, form requires fragment, and provider references are resolved against the
final published generation. Coefficients must be nonnegative and cannot both be zero. Floats,
including nonfinite RON values, do not satisfy the integer schema.

Normal uses the definition's `effects`. Optional corruption must contain nonempty replacement
`effects` and nonempty `penalties`; Corrupt uses that replacement rather than also stacking Normal
effects. Penalties are typed `DefenseFlat(amount: i32)` or `MagicResistFlat(amount: i32)` and require
strictly negative amounts. Owned copies refer to the immutable definition ID and retain their own
UUID/variant in the run graph. They do not modify the published catalog. See the
[runtime contract](core_runtime_contract.md#relic-copies-numeric-authority-and-save-graph) for admission,
original-source bindings and persistence.

### Readonly authoring description and explanation

[`authoring::describe` / `explain`](../src/authoring/query.rs)는 같은 `AuthoringContent`를 읽고 크기를 검사한
owned JSON document와 별도 `ExecutionUsage`를 반환한다. Describe는 production registry의26 catalog/45 definition kinds,
실제 final inventory·identity/source receipt·supported capabilities 및 기존 `EFFECT_SUPPORT`를 제공한다.
이 editable support는 현재 Instant Damage/Heal의 literal property 계약이며 전체 engine operation 목록이 아니다.

`ExplainQuery::Definition`은 exact key의 final source·writer 관계와 published resolved record를 반환한다.
선택 Remove는 `present: false`, `representation: removed`, `source/value: null`이며 unknown key는 명시 오류다.
Boss Pattern/Actor/Encounter는 `compiled_graph`와 `index_space: compiled`로 구별한다. Actor→Pattern과
Encounter→Actor는 actual definition ID 참조이며, 컴파일된 배열 index에서 원문 child 위치를 만들지 않는다.
SummonedUnit은 compiled combat profile, BossSignal은 실제 선언 ID를 설명한다. 별도 CompiledGraph query는
현재 존재하는 세 boss graph kind만 받으며 다른 kind는 unsupported, 제거된 graph는 not-found다.

Effect query는 exact skill/action/effect ID, operation·origin·shared target/delivery와 compiled `source()`를 함께 반환한다.
Origin definition의 current source는 실제 Original/Added ID로만 조회한다. Tags나 같은 effect ID로 원본 action/effect 계보를
추론하지 않는다. Nonwinning/Remove payload·정밀 원문 위치와 original RON text는 보존되지 않았으므로 제공하지 않는다.
파일을 게시 뒤 변경해도 이 설명은 이미 게시된 세대의 내용이다.

`ExecutionLimits.max_result_bytes`는 기본8MiB/hard64MiB의 inclusive encoded-document 상한이다.
Mandatory writer는 각 chunk를 buffer에 추가하기 전에 byte admission과 work를 검사한다. 초과·취소는
성공 문서를 반환하지 않으며 query 결과는 generation 전체를 추가 pin하지 않는다. `result_bytes_peak`는 encoded 길이로,
allocator capacity/전체 heap 측정이 아니다. Query는 gameplay state/clock을 호출하지 않는다.

### Definition resolution and identity

- 일반 string catalog key는 Global의 exact key 또는 Named의 명시적인 `namespace::local`이다. 자동 prefix,
  참조 재작성, alias는 없다. UUID는 기존 owner의 typed identity/중복 검사를 유지한다. Definition-local
  task/state/action/effect/handle ID는 각 도메인의 local grammar를 유지한다. Recipe의 equipment reference,
  enum/floor key와 singleton은 독립 string catalog와 구분한다.
- Payload만 있으면 Add다. 한 target에는 base Add가 정확히 하나여야 한다. Replace는 같은 target의 complete
  typed payload 하나와 명시적 `after` writer를 요구한다. Remove는 payload가 없어야 한다. Dependency는
  availability만 제공하며 implicit override/order/winner가 아니다.
- 각 `after`는 선택된 dependency ancestor이면서 같은 target의 실제 writer여야 한다. Missing/duplicate/self
  dependency, cycle, 없는 target/writer, 중복 payload/operation과 Remove 이후의 후속 변경은 오류다.
  독립 sibling Remove/Replace는 서로를 지우지 않는다. Maximal writer가 여러 개면 host가 명시한 maximal
  winner가 필요하다. 단일 maximal head는 결정적이다. 명시적 diamond merge는 완전한 replacement다.
- 실행 순서는 catalog, base pack ID, 해당 pack 내 source file, 원문의 numeric definition ordinal로 결정한다.
  Replace는 base Add 슬롯을 유지하고 Remove는 그 슬롯을 없앤다. Host pack/file 열거 순서는 우선순위가 아니다.
- Semantic SHA-256는 current contracts, 정렬된 pack identity/capability/dependency 및 explicit writer 관계와
  selected winner, 최종 typed raw payload의 실행 순서를 포함한다. Array 순서는 보존하고 unordered map만
  canonical key로 정렬한다. Enum/Option/type과 finite float bits는 구분하며 nonfinite는 거부한다.
  주석/공백, source root/path, byte receipts, compiled cache와 폐기된 loser payload body는 포함하지 않는다.
- 별도 source receipt는 실제 manifest/catalog의 path, byte count, SHA-256를 기록한다. 같은 semantic identity라도
  주석 등으로 byte receipt가 달라질 수 있다. Saved identity는 receipt를 대신하여 source 파일을 재구성하지 않는다.

### Filesystem and official packaging

현재 검증된 backend는 Unix/WSL이다. Trusted root handle을 고정하고 untrusted relative path의 각 component를
`openat`/`NOFOLLOW`로 연다. 절대/탈출/빈/dot/backslash/drive/control 경로, directory prefix의 case collision,
중복 inventory와 hardlink identity, nonregular file, non-UTF-8를 거부한다. 파일 전후 metadata를 비교하고
전체 acquisition 뒤 모든 path를 다시 열어 identity와 bytes를 재확인한다. 관측된 변경은 오류이며 retry나
약한 fallback은 없다. 이 절차가 filesystem 전체의 atomic snapshot을 보장한다는 뜻은 아니다. Non-Unix는
동등하게 검증된 backend가 없다는 명시적 오류를 반환한다.

Trusted 기본 한도는 128 packs, manifest 포함 4,096 files, 파일당 8 MiB, 합계 64 MiB, dependency와 writer
edge 합계 4,096, parser nesting 128이다. Host nesting은 1..=128이어야 한다. RON 0.5에는 depth 설정이 없어
문자열/char/raw string/comment를 구분하는 bounded lexical check를 먼저 수행한다. Manifest가 한도를 늘릴 수 없다.

[package_official](../src/game/data/content_pack/packaging.rs)는 명시적 source root의 `data/bundle.ron`과
26개 catalog를 읽는다. Bundle은 exact schema 6와 strict fields를 검증한다. 같은 raw/resolve/publication
검증 후 catalog bytes를 그대로 별도 출력에 쓰고 current `pack.ron`을 마지막에 생성한다. 출력은 source
밖의 존재하지 않는 디렉터리여야 한다. 빈 디렉터리·기존 pack·symlink도 merge/overwrite하지 않는다.
Directory-relative exclusive creation/write 후 public loader로 다시 읽어 semantic identity와 모든 byte receipt를
확인한다. 쓰기 실패는 새 불완전 출력 경로를 보고하며 기존 source/custom/personal 파일을 삭제하거나 변환하지 않는다.

최소 도구는 default release에 별도 등록되어 있다:

```sh
cargo build -p game_core --release --no-default-features --bin package_official_content --bin consume_content_pack
./target/release/package_official_content game_resources /tmp/new-official-pack
./target/release/consume_content_pack /tmp new-official-pack
```

출력 parent는 미리 존재해야 한다. Consumer는 고정 seed로 실제 briefing/starter flow를 최대 128 commands까지
진행해 identity/receipt와 Run snapshot을 출력한다. 이 기존 consumer의 범위는 Map 도달 검사다. 공통 validate/describe/explain/
simulate는 위 author_content를 사용한다. Interactive winner 선택과 이 probe의 reload는 제공하지 않는다. 임의 기존 live_data를 자동 동기화하지 않는다.

검증된 replacement manifest 예시는 다음과 같다. `b/policy.ron`은 current RunPolicy의 **전체** payload다.
`official`과 `b`를 함께 선택하며, `after`는 pack ID를 가리킨다. 아래 원문은 실제 release A/B 검사에서
사용하는 [manifest 생성부](../tools/content_pack_release_probe.py)의 current contract와 같다.

```ron
(
    pack_id: "b", pack_version: "1", namespace: Global,
    contract: (pack_format: 1, catalog_schema: 5, authoring: 1, execution: 2, fingerprint: 1),
    required_capabilities: [], dependencies: ["official"],
    files: [(kind: RunPolicy, path: "policy.ron")],
    operations: [(target: (kind: RunPolicy, id: "run_policy"), operation: Replace, after: ["official"])],
)
```

기존 임의 폴더를 만들거나 덮어쓰지 않고 별도 fixture를 재현하려면
`python3 core_enhanced/tools/content_pack_release_probe.py --repository . --output-dir /tmp/new-pack-probe`
를 사용한다. 출력 경로는 존재하지 않아야 하며, script는 실제 source 복사본과 current release binaries를
사용한다. 733은 이 격리된 검사의 starter currency일 뿐 공식 gameplay 값을 바꾸지 않는다.

### Expedition authoring input

Employee 정의의 required role은 `Dealer`/`Tank`/`Healer`이며 starter-eligible은 role별2/총6이다.
세 직원 Tutorial preset와 starter/hire initial loadout은 같은 최종 catalog에서 참조/무기 적합성을 검증한다.
RunPolicy는 tutorial preset, initial deterrence/first-entry cost와 HQ recruit price/depth/session limit을 소유한다.
현재 Expedition 개발 schedule은 깊이1–4의 실제 HQ/조우/SavePoint/보상/Gate를 제공하며 fixed3층 Tutorial과
구분한다. retained `standard_*`/`endless_*` authored schedule key와 difficulty band `Standard`는 mode alias가 아니다.
새 `expedition_lifecycle_v1`, catalog6/execution3은 candidate/cache/required-save의 exact content identity에 포함한다.
Profile vault는 metadata 사본이 아니라 canonical definition ID와 실제 사본 상태를 저장하며 dormant ID를 보존한다.

### Fixture and durable runtime boundaries

`ValidatedGameData::load_live_embedded`, fixture builder와 bare `Arc<GameDataBase>` 변환은 명시적
`test`, `debug_assertions` 또는 `test-fixtures` 경계에만 있다. Default release의 production fallback이 아니다.
Fixture identity도 실제 catalog 내용으로 계산하며 상수나 Arc 주소를 사용하지 않는다.

Core active-run continuation은 checksum-protected strict V12만 읽고 쓴다. Current와 독립 SavePoint의
required `content_identity`는 contract version과 semantic SHA-256를 포함하고 checksum의 일부다.
Retained generation과의 일치 및 instance/UUID/profile binding과 graph/seed/power/resource 검증을 유지한다. V11과 이전 버전,
missing/unknown field, bad checksum과 unknown version은 변환이나 guessed identity 없이 거부한다.
동일한 current/SavePoint의 codec 공유는 reader에서 독립 checkpoint로 복원하며 의미를 병합하지 않는다.

Continue는 최신 확정 상태의 run seed/UUID를 복원하며 constructor bootstrap seed와 달라도 된다.
미완료 node는 진입 전 상태, 완료 결과/필수 선택은 저장된 receipt를 사용한다. Explicit load는 별도
SavePoint로 되감고 현재 run seed와도 일치해야 한다. Continue는 load count를 소비하지 않고 explicit
load는 최대 세 번이다. Profile/compendium과 run peak, 보존 대상 VN session counter의 단조성을 유지한다.
Live battle은 저장하지 않으며 현재 stable VN session/queue는 continuation이 소유한다. 현재 승인된 host writer는
required configuration을 포함하는 V7 envelope와 accepted application journal V3를 사용한다. OS save path,
lock, latest backup와 V4 profile는 host가 소유한다. 독립 SavePoint payload는 V13이다. 이전 envelope/journal은
readonly preflight에서 원본을 보호하고 명시 거절하며 다른 mod generation의 승인을 추정하지 않는다.
Readonly required-mod preflight와 accepted replay 경계는 [Core Runtime Contract](core_runtime_contract.md#durable-active-run-host-boundary)를 따른다.
과거 R02 증거로 배포 호환을 주장하지 않는다.

### Typed resource catalog and execution

`Resources` catalog의 공식 경로는 `data/resources/base.ron`이며 현재 공식 내용은 `(resources: [])`다.
외부 정의는 `id`, `owner`, `lifetime`, `initial`, `cap`을 모두 명시한다. ID는 정의 ID이며 실제 소유 UUID가 아니다.
owner는 `Employee` 또는 `EquipmentCopy`, lifetime은 `Run`, `Attempt`, `Deployment`, `Activation`,
`Delivery`, `Chain` 중 하나다. initial/cap은 u32이며 initial≤cap이다. 빈 ID·중복·필수 field 누락·unknown field를
거부하며 catalog 최대4,096정의와 ID 최대128bytes를 적용한다. 같은 ID의 선언 변경도 semantic identity를 바꾼다.

`SkillEffectDef::Resources { operations }`는 비어 있지 않은 최대4,096개 ordered operation 묶음이다.
각 원소는 resource ID, `SourceEmployee`/`SourceEquipment`/`TargetEmployee` owner selector와
`Grant(n)`/`Decay(n)`/`Consume(n)`을 가진다. 최종 catalog 검증은 unknown ID·선언과 selector의 owner kind
불일치를 실제 skill/action/effect/field source 위치와 함께 거부한다. Source selector는 effect당 한 번,
TargetEmployee는 실제 선택된 대상별로 확장되며 최대 전개 수4,096을 사전 확인한다.

Runtime은 현재 World의 직원 UUID/장비 사본 UUID와 실제 발생한 lifetime capability를 사용한다.
정의 UUID·runtime unit/item ID를 persistent owner로 추정하지 않는다. Grant는 cap으로 제한하고 Decay는
잔량만 감소시키며 Consume 부족은 같은 operation 묶음과 enclosing transaction을 전부 거부한다.
자세한 수명·Run 반영·저장은 [core runtime contract](core_runtime_contract.md#typed-resources-r05)에 있다.
이 opcode는 literal Damage/Heal 편집 지원 표와 별개이며 새 literal editor 지원을 주장하지 않는다.

## Live RON Authoring

Live RON은 gameplay source data다. 새/core live schema는 가능한 한 authoring typo를 deserialize 단계에서 잡아야 하며, gameplay 의미가 있는 필드는 조용히 무시하거나 legacy alias로 받아들이지 않는다.

- Core gameplay schema는 `#[serde(deny_unknown_fields)]`를 기본으로 사용한다.
- Unknown field, stale field, removed alias는 compatibility layer 없이 reject한다.
- Gameplay 의미가 바뀌는 필드는 명시 authoring을 우선한다.
- 단순 표시, optional presentation hint, harmless authoring convenience는 명시적 `serde(default)`를 둘 수 있다.
- `serde(default)`를 둔 gameplay field는 코드 이름, validation, 테스트가 기본값의 의도를 설명해야 한다.
- 중복 top-level id/UUID, pool id, cross-item UUID는 first/last winner를 만들지 않고 bundle을
  거부한다.
- Live-authored `basic_attack` data does not rely on `BasicAttackDef` runtime defaults. Abnormality and corroded employee live RON must pass through authored input validation where combat-meaningful fields are explicit: `range_policy`, `defense_tile_range`, `damage_type`, `targeting_profile`, `air_capable`, `range_role`, `interval_ms`, `windup_ms`, `ranged_reposition_ms`, and `delivery`.
- `DeliveryDef::Projectile` publication requires positive `speed_units_per_ms`; every present
  `max_range_tiles`, `max_lifetime_ms`, `max_kills`, `max_pierces`, and `collision.max_hits` value is also
  positive. Raw RON parsing and programmatic construction may form a candidate, but the candidate is not published
  until this one semantic contract succeeds with object ID, exact field path, and cause. Optional zero values are
  not normalized and do not use the runtime zero-flight guard as a compatibility fallback.
- Skill delivery may use its validated TargetLocked or DirectionalCollision policy. Abnormality/corroded employee
  `basic_attack` and equipment `weapon_profile` consumers accept only `Instant` or a positive-speed default
  TargetLocked projectile. `allow_targetless_cast`, range/lifetime/kill/pierce stops, nondefault collision fields,
  DirectionalCollision, and TileArea are skill-only and reject the whole basic/weapon candidate rather than being
  silently ignored by runtime.
- Every `equipment_type: Weapon` requires one `weapon_profile` that explicitly authors
  `deployment_affinity`, base `block_capacity`, `ground_defense_focus` and `platform_rest`. The two profile keys
  are required even when their value is `None`; explicit numeric zero is valid. `GroundOnly` requires focus and
  forbids rest, `PlatformOnly` requires rest, forbids focus and requires base block 0, and `Any` requires both.
  Damage reduction above 100, a missing key, unknown tactical field, or mismatched affinity/profile rejects the
  bundle. No archetype inference or live-RON default fills these fields.
- Live skill `cast_targeting` is required, and `EnemySingle` requires an explicit `rule` even when
  the intended value is `Nearest`. Missing rule, weapon-profile inheritance variant, and unknown /
  dual targeting authority are strict load failures.
- Skill cast acquisition explicitly authors `reach` and `air_capable` plus optional `target_range`;
  missing reach/air authority fails loading, and `global` rejects a competing target pattern. Steps
  reject removed range/air fields, while every `TileArea` requires its own
  concrete `footprint`. Target/impact-centered footprints require `tile_origin: Anchor` and are
  rotation-invariant until an explicit orientation contract exists.
- `defense_tile_range: None` is still an explicit authored value. `range_policy: Pattern` may use a preset/resolution layer to fill the runtime tile pattern, but the authoring field itself must be present.
- Enum casing은 도메인 내부에서 일관되어야 한다. 별도 `rename_all`이 없는 live RON enum은 Rust variant 이름을 그대로 사용하고, `snake_case`로 선언된 enum은 RON도 snake_case를 사용한다.
- String id는 기존 live data convention을 따른다: item/profile/skill/event/preset id는 lower snake_case, Lobotomy-style abnormality id는 catalog id 형식을 유지한다.
- Live RON loading과 focused schema tests는 gameplay 시작 전 authoring mistake를 잡는 gate다.

현재 strictness가 고정된 핵심 도메인:

- `boss_runtime/base.ron`은 PvE/abnormality body에 graph를 embed하지 않고 `signals`, `patterns`,
  `actor_behaviors`, `encounters` logical collection을 별도로 소유한다. Production content opt-in은 교체 가능한
  Little Red reference이고, `cwbs_reference_*` rows는 summon/signal vertical path만 검증하는 명시적
  nonproduction reference다. 어느 쪽도 final character content가 아니다.
- Boss catalog의 top-level stable ID는 lower snake case local ID 또는 명시적인 `namespace::local`이고,
  definition-local state/task/parameter ID는 기존 local grammar를 유지한다. Unknown field, duplicate/dangling ID, 잘못된
  entry/target, priority 충돌, incomplete Pattern outcome/finite Task result mapping, intrinsic task
  capability 충돌, unbounded/zero-time graph, invalid target query, cooldown duration 충돌과 typed parameter
  binding 오류가 있으면 bundle candidate 전체를 거부한다. Pattern call은 현재 `FixedSequence`만
  compile하며 known future `PriorityEligible`/`WeightedEligible`와
  `RepathToProtectedObjective` task는 다른 의미로 fallback하지 않고 publication을 거부한다.
- `PveEncounter.boss_runtime`은 optional `{ encounter_id, actors[] }` reference다. Actor binding은 Boss
  encounter의 모든 slot을 정확히 한 번 bind하고 `{ actor_slot_id, wave_id, spawn_index }`로 preview가
  만든 opponent spawn identity를 지정한다. Missing definition/slot/wave, duplicate spawn과 manual-wave
  authored count 밖 index는 candidate validation에서 실패한다.
- 현재 production content opt-in은 `suppress_little_red`이며 `primary_abnormality` slot을
  `primary_abnormality` wave의 spawn index 0에 bind한다. Route-dependent behavior 때문에 해당 exact wave만
  `defense_main` route를 authoring한다. 별도 `reference_cwbs_final_boss`는 `ReferenceFinalBoss` class로
  `cwbs_reference_final_boss_runtime`을 bind하지만 normal map/BossOmen/reward/progression registry에서 제외된다.
- Boss raw catalog는 parse/validation 중 immutable indexed definitions와 call-site별 typed argument frame으로
  정확히 한 번 compile되고 private validated state가 그 결과를 publication까지 보유해 같은
  `GameDataBase` candidate에만 넣는다. Boss parse/compiler/PvE reference 중 하나라도
  실패하면 기존 domain만 따로 publish하지 않으며 startup은 fail closed한다. Hot reload와
  last-known-good generation publisher는 이 loader에 존재하지 않는다.
- PvE encounter/wave authoring은 unknown top-level/nested fields를 reject한다.
- Wave는 required `activation { trigger_id, priority, primary, optional deadline_ms }`와
  `required_for_victory`를 authoring한다. Primary는 closed time/group-clear/actor HP/actor death/actor point set이고
  dependency cycle, invalid actor binding, equal time/deadline와 unreachable required contract는 publication에서
  실패한다. Runtime은 missing field를 time default로 보정하지 않는다.
- `summoned_units/base.ron`은 Boss-authored hostile definition의 독립 stat/limit/summoner-removal/behavior를
  소유한다. Boss compiler는 missing definition/behavior/route, recursive Summon, invalid occupancy/wait matrix,
  unbound batch/removal signal을 bundle publication 전에 거부한다.
- PvE `reward_sources`는 생략할 수 없는 strict collection이다. 각 row는 stable `reward_source_id`,
  exact `encounter_id`/`fragment_id`, `first_equipment_ids`, `repeat_rewards`, `revival`을 소유한다.
  source/encounter/fragment ID는 각각 중복될 수 없으며 source encounter는 primary abnormality가 있는
  Elite/NormalBoss/FinalBoss여야 한다. 파편 origin은 해당 abnormality와 같아야 한다. 엘리트의 최초
  장비 목록은 비어 있으면 안 되며 정상 참조·중복을 검사한다. 반복 보상은 양수 수량/AliveRoster XP나
  명시 유물만 허용하고 최초 파편·구 FragmentDust·고정 장비 재지급은 거절한다.
  공식29개 source는 Elite13/NormalBoss13/FinalBoss3이다. 마탄 source는 식별용으로 남되 일반 보상과
  revival은 비어 있어야 한다. `ReferenceFinalBoss`는 source가 될 수 없고 모든 outcome의
  reward/bonus/progression을 비활성화해야 한다.
- Ordinary Reward catalog가 위 29 source fragment를 직접 지급하면 bundle publication을 거부한다.
  제거된 suppression research, research progress와 pending-delivery field/effect는 alias나 default로
  받아들이지 않는다.
- Event scene/panel/employee-check/success-action authoring은 unknown fields를 reject한다.
  Panel/image/text 비어 있음, scene-level legacy `required_roll`, DC 8/12/16/20 이외 값,
  invalid/duplicate choice key 또는 equipment Event metadata, choice/scene identity 중복, 둘 이상의
  reachable checked scene, checked-scene return cycle를 포함한 graph defect, unsupported action,
  broken encounter/reward/key reference도 bundle publication 전에 거부한다.
- Employee definition은 Event aptitude를 필수로 authoring하고 생성 시 stored
  `EmployeeProfile.event_aptitude`로 복사한다. Equipment의 `event_check_modifiers`는 aptitude별
  +1/+2만, `event_choice_keys`는 valid lower-snake-case unique key만 허용한다. Event가 요구하는
  key는 published equipment catalog에 provider가 없으면 bundle 전체를 거부한다.
- Equipment `triggered_effects`와 `ability_activations`는 equipment-local contract 뒤 같은 candidate의
  Buff/Skill catalog와 교차 검증한 후에만 publish한다. `ApplyBuff.buff_id`와 activation `ability_id`는
  반드시 실제 catalog row를 가리켜야 하며 `Permanent` activation은 허용하지 않는다.
  Effect matrix는 runtime occurrence와 동일하다: Permanent는 SelfUnit/AllAllies Modifier만, battle start는
  counterpart가 아닌 ordinary effect만, attack은 Counterpart damage effect와 모든 ordinary effect,
  hit은 Self current-hit damage·Counterpart BonusDamage retaliation·모든 ordinary effect를 허용한다.
  OnDeath ordinary effect는 dead SelfUnit을, OnKill/OnAllyDeath ordinary effect는 dead CounterpartUnit을
  대상으로 authoring할 수 없고 damage effect는 이 death-family occurrence에서 허용하지 않는다.
  위반은 exact equipment/path/cause로 candidate 전체를 거부하며 runtime의 missing-reference 방어는
  publication fallback이나 별도 authoring authority가 아니다.
- Live `sealed_archive_recovery_01`은 `event_sealed_archive` map definition으로 명시적으로
  참조되며 `archive_scanner`/`maintenance_tech` provider path를 사용한다. WhiteNight는 별도
  advance-only Event로 유지된다.
- Visual Novel definition과 trigger authoring은 unknown/removed fields를 reject한다. Definition은
  scene/presentation만 authoring하고 `blocking`은 trigger에만 필수로 존재한다;
  removed definition `default_blocking`은 compatibility field로 수용하지 않는다.
- Run policy의 recovery는 top-level `save_point_trauma_recovery`와
  `gate_trauma_recovery`를 exact tagged `TraumaRecovery`로 authoring한다. 허용 variant는
  `Flat(amount: 1..100)`과 `CurrentPercent(percent: 1..100)`뿐이며 variant에 다른 variant의 field를
  섞거나 제거된 `support`/Rest recovery group을 작성하면 reject한다. Production live 값은 각각
  `CurrentPercent(percent: 15)`와 `CurrentPercent(percent: 10)`이다.
- Run policy의 required `tactical_resource` group은 `initial`, `maximum`, `recovery_per_second`,
  `base_deploy_cost`, `redeploy_cost_multiplier_pct`를 소유한다. Production 값은 `20`, `99`, `1`, `10`,
  `150`이며 initial/base/redeploy cost는 maximum 이하여야 하고 redeploy multiplier는 양수다. Required
  `live_deployment` group은 withdraw/defeat redeploy cooldown과 first instance salt만 소유한다. 제거된
  `initial_cost`, `max_cost`, `cost_per_second` 또는 resource/price field를 deployment group에 작성하면
  strict parse가 실패하고 alias/dual ownership은 없다.
- Run policy의 required `tactical_stance` group은 positive `stance_change_cooldown_ms`와
  `rest_tick_interval_ms`만 소유한다. Production 값은 각각 `1500`, `1000`이며 missing/zero/unknown field와
  제거된 generic toggle/cadence 이름을 default나 alias로 받지 않는다. Cooldown은 양방향 stance command,
  interval은 Platform Rest event-time scheduler의 단일 tuning authority다.
- Run policy의 required `tactical_relocation` group은 positive `cost`와 `cooldown_ms`만 소유한다. Production
  값은 `5`, `10000`이며 cost는 `tactical_resource.maximum` 이하여야 한다. Missing/zero/unknown field는
  strict reject한다. `TacticalRelocateUnit`과 `TacticalSwapUnits`가 이 값을 공유하며 swap 전용 cost/cooldown,
  Phase 8 수동 skill focus 또는 deployment cost-reduction field를 이 group에 authoring하지 않는다.
- 모든 `SkillDef.cast_targeting: Explicit`은 required `selection`을 갖는다. Live wire/RON 값은 `self`,
  `automatic_unit`, `player_unit`, `player_tile`이고 omission, unknown field, legacy alias와 target kind 기반
  default를 허용하지 않는다. Selection/target pair는 `self/SelfUnit`,
  `automatic_unit|player_unit/EnemySingle`, `player_unit/AllySingle`, `player_tile/CastTarget`이 유효하다. Player-selected skill에
  `RetargetOnStep` step이 하나라도 있으면 bundle publication을 거부한다.
- 현행 live skill 36개는 selection을 모두 직접 authoring한다. `player_unit`은
  `fragment_freischutz_black_round`, `freischutz_magic_bullet`, `training_single_heal` 3개,
  `self`는 8개, `automatic_unit`은 20개다. `player_tile`은 `training_area_heal`과 U07의
  `laetitia_place_friend`, `laetitia_place_great_friend`, `common_place_ice_wall`,
  `melting_love_place_meat_wall` 5개다. U05의26개에 U06 훈련6개와 U07 생성4개가 추가됐다.
  U07 생성은 Instant로 독립 owner를 만들며 이후 공격/TTL은 그 owner가 담당한다.
  두 벽 skill은 현재 standalone consumer이고 새 reward/fragment 참조를 뜻하지 않는다.
  Rust fixture용 target-to-selection convenience는 live deserialization 경로나 production fallback이 아니다.
- PvE `tactical_plan.field_work_objects`는 tactical point/static object/tile kind와 별도인 strict object list다.
  각 object는 non-empty globally unique `id`, `presentation_kind`, `position`, 하나 이상의 work cell, positive
  `required_work_ms`와 typed `completion_effect`를 필수로 작성한다. 각 cell은 object 안에서 unique한 non-empty
  `id`, 전체 encounter field-work cell 중 unique한 `position`, explicit `tile_kind`를 가진다. Empty cell, zero work,
  duplicate ID/position, unknown/missing field는 publication 전에 거부한다.
- 현재 presentation은 `EmergencyRepairTerminal`, completion은
  `RepairProtectedUnit(unit_ref, flat_hp)`만 있다. Repair target은 같은 encounter의 authored ProtectUnit objective와
  win condition을 정확히 가리키고 `flat_hp > 0`이어야 한다. `Reset`, generic effect/status/script, compatibility
  mode/default/alias는 deserialize되지 않는다.
- 첫 live definition은 `suppress_freischutz.black_box_emergency_repair` 하나뿐이다. Object와 `repair_cell` 위치는
  `(2, 6)`, cell은 `ground`, required work는 `10000`, completion은
  `RepairProtectedUnit(unit_ref: "black_box_device", flat_hp: 100)`이다. 이 encounter가 선택할 수 있는 세
  Ambush/Medium battlefield template 모두 `(2, 6)`을 Ground deployment cell로 author하므로 runtime은 invalid
  terrain을 보정하거나 다른 cell로 fallback하지 않는다.
- Skill RON의 전술 포인트 current 증감 effect는 `ModifyTacticalResource(amount)`만 허용한다. 제거된
  `ModifyStabilization` 이름은 deserialize되지 않으며 effect는 maximum이나 natural-recovery tuning을
  변경하지 않는다.
- Corroded employee profile/range preset authoring은 unknown fields를 reject한다.
- Abnormality and corroded employee `basic_attack` authoring rejects missing combat fields instead of falling back to runtime `BasicAttackDef::default()`.
- Shop authoring은 reusable merchandise preset과 Shop별 preset reference/local entry를 사용한다.
  모든 Shop은 positive `offer_count`/buy percentage와 충분한 unique candidate를 가져야 한다.
  Entry quantity/weight/optional buy percentage는 양수이고 equipment quantity는 1이다. Missing typed
  merchandise, duplicate composition, inconsistent tags, invalid refresh/sell policy, zero-price item,
  quote overflow와 same/cross-Shop arbitrage를 bundle publication 전에 reject한다. Prototype
  `shop_type`, `visible_items`, `stock_items`, `can_reroll`은 unknown field이며 compatibility schema가
  없다.
- Run policy, skill raw schema, reward equipment pools, boss omen chains, corroded wave presets, facility
  templates/node-assignment profiles/node definitions are also strict by domain-owned validation/tests.
- Reward/Event `GrantConsumable`은 `consumable_id`와 양수 `quantity`를 명시해야 한다. Missing
  quantity, zero quantity, unknown consumable은 publication 전에 거부하며 runtime은 per-copy
  UUID나 implicit quantity 1을 보정하지 않는다.
- Run growth policy는 비어 있거나 0인 XP requirement를 거부한다. Battle tier threshold는 비어
  있을 수 없고 `min_level > 0`인 strictly increasing 순서여야 한다. Runtime은 authored XP
  curve 범위를 넘으면 마지막 requirement를 반복하고, level에 맞는 가장 높은 tier threshold를
  선택한다.
- Consumable `rarity/live_pool`, generated Skill identity/timing/step fields, Abnormality and
  CorrodedEmployee core combat fields, CorrodedWave role balance fields, PVE manual enemy
  `tier/count`, Wave `activation` and `required_for_victory` are presence-required. Current live generated data already authors
  them explicitly.
- Option omission, empty collection defaults, targeting/repeat/presentation defaults are retained
  only where omission itself is the canonical neutral/optional meaning; they are not legacy schema
  acceptance.

## Main Bundle

아래 표는 공식 source bundle의 live RON group, domain loader/type, authored file 위치를 연결하는 canonical index다. Production은 이 원본을 별도 pack으로 패키징한 뒤 `content_pack::load`로 선택된 외부 pack을 검증한다.

Live data root는 `/mnt/f/work/simulator/game_resources/data`다.

| Live RON group | Runtime loader/types | Live files |
| --- | --- | --- |
| Abnormalities | [data/abnormality_data.rs](../src/game/data/abnormality_data.rs) | `/mnt/f/work/simulator/game_resources/data/abnormalities/base.ron` |
| Boss omen chains | [data/boss_omen_data.rs](../src/game/data/boss_omen_data.rs) | `/mnt/f/work/simulator/game_resources/data/boss_omen/chains.ron` |
| Buffs | [data/mod.rs](../src/game/data/mod.rs), [battle/buffs.rs](../src/game/battle/buffs.rs) | `/mnt/f/work/simulator/game_resources/data/buffs/base.ron` |
| Consumables | [data/consumable_data.rs](../src/game/data/consumable_data.rs) | `/mnt/f/work/simulator/game_resources/data/consumables/base.ron` |
| Corroded employees | [data/corroded_employee_data.rs](../src/game/data/corroded_employee_data.rs) | `/mnt/f/work/simulator/game_resources/data/enemies/corroded_employees.ron` |
| Corroded waves | [data/corroded_wave_data.rs](../src/game/data/corroded_wave_data.rs) | `/mnt/f/work/simulator/game_resources/data/enemies/corroded_wave_presets.ron` |
| Employees | [data/employee_data.rs](../src/game/data/employee_data.rs) | `/mnt/f/work/simulator/game_resources/data/employees/definitions.ron` |
| Equipment | [data/equipment_data.rs](../src/game/data/equipment_data.rs) | `/mnt/f/work/simulator/game_resources/data/equipments/base.ron` |
| Events | [data/event_data.rs](../src/game/data/event_data.rs) | `/mnt/f/work/simulator/game_resources/data/events/story/base.ron` |
| PvE encounters | [data/pve_data.rs](../src/game/data/pve_data.rs) | `/mnt/f/work/simulator/game_resources/data/pve/encounters.ron` |
| Rewards | [data/reward_data.rs](../src/game/data/reward_data.rs) | `/mnt/f/work/simulator/game_resources/data/events/rewards/base.ron` |
| Run policy | [data/run_policy_data.rs](../src/game/data/run_policy_data.rs) | `/mnt/f/work/simulator/game_resources/data/run/policy.ron` |
| Shops | [data/shop_data.rs](../src/game/data/shop_data.rs) | `/mnt/f/work/simulator/game_resources/data/events/shops/base.ron` |
| Skills | [data/skill_data.rs](../src/game/data/skill_data.rs) | `/mnt/f/work/simulator/game_resources/data/skills/base.ron` |
| Skill fragments | [data/skill_fragment_data.rs](../src/game/data/skill_fragment_data.rs) | `/mnt/f/work/simulator/game_resources/data/skill_fragments/base.ron` |
| Visual novels | [data/visual_novel_data.rs](../src/game/data/visual_novel_data.rs) | `/mnt/f/work/simulator/game_resources/data/visual_novels/base.ron` |
| Visual novel triggers | [data/visual_novel_trigger_data.rs](../src/game/data/visual_novel_trigger_data.rs) | `/mnt/f/work/simulator/game_resources/data/visual_novel_triggers/base.ron` |

Employee loading은 하나의 strict `EmployeeDefinitionDatabase`만 publication한다. 각 definition은 전역적으로 유일하고 비어 있지 않은 `employee_definition_id`, 이름, `starter_eligible`, explicit `initial_loadout`, role/background, 두 personality 축, `CombatAptitude`, base combat stats와 양수 `basic_attack_speed_percent`를 가져야 한다. starter-eligible definition 수와 initial loadout의 장비/스킬 파편 참조를 확인하며, 각 starter loadout은 실제 `Weapon`을 하나 이상 가져야 한다. First-live 6개 starter는 `standard_training_sword`를 명시하고 이 sword는 general equipment reward pool의 `exclude_equipment_ids`에 있어야 한다. Exclusion ID의 missing reference나 duplicate도 startup validation에서 거부한다. 제거된 starter/recruitment 두 파일이나 필드 alias를 dual-read하지 않는다.

## Map And Battlefield Catalogs

Facility templates, node-assignment profiles, map node definitions, battlefield archetypes and battlefield
templates are domain-owned types but not separate production readers. Their five live files are parsed once
inside the same versioned source set, validated before publication, stored in `GameDataBase`, and used by
facility generation/navigation or combat preview through that exact instance.

| Catalog | Runtime owner | Live file |
| --- | --- | --- |
| Facility templates/pools | [map/facility.rs](../src/game/map/facility.rs) | `/mnt/f/work/simulator/game_resources/data/map/facility_templates.ron` |
| Node assignment profiles/schedules | [map/facility.rs](../src/game/map/facility.rs) | `/mnt/f/work/simulator/game_resources/data/map/node_assignment_profiles.ron` |
| Map node definitions | [map/types.rs](../src/game/map/types.rs) | `/mnt/f/work/simulator/game_resources/data/map/node_definitions.ron` |
| Battlefield archetypes | [combat_preview/mod.rs](../src/game/combat_preview/mod.rs) | `/mnt/f/work/simulator/game_resources/data/map/battlefield_archetypes.ron` |
| Battlefield templates | [combat_preview/mod.rs](../src/game/combat_preview/mod.rs) | `/mnt/f/work/simulator/game_resources/data/map/battlefield_templates.ron` |

### Facility Template And Assignment Authoring

- `facility_templates.ron` owns versioned semantic slots, allowed categories, authored tags, typed terminal
  roles, directed connections, optional connection reveal gates, manifestation rules and template pools.
  Duplicate/missing slot references, unknown tags, invalid terminal category/role, unreachable terminal,
  directional deadlock, invalid manifestation dependency and Tutorial guardian-dominator violations reject
  the bundle.
- `node_assignment_profiles.ron` owns Tutorial exact Floor schedules and Expedition contiguous/open-ended
  stages/cycles through the retained authored keys `standard_schedule`/`endless_stages`. A Tutorial fixed profile uses `fixed_slot_assignments` only: every non-Start semantic slot has
  exact `slot_id`, `kind_id`, typed payload and optional authored combat policy, and runtime consumes it without
  category solving, shuffle or seed-dependent content selection. Adaptive profiles instead use category
  min/max/exact constraints, remaining-category weights and tagged assignments; the two modes cannot be mixed.
  Each profile has a required `difficulty_profile_id` that must resolve in the same live RunPolicy publication.
  Missing/overlapping schedule coverage, incomplete/duplicate fixed slots, kind/category/payload mismatch, unknown
  template/profile/pool/reward/encounter/actor reference and an adaptive assignment with no complete feasible slot
  matching reject publication; runtime does not repair categories or fixed content.
- `node_definitions.ron` chooses kind/content only within an already assigned category. It has no depth range
  or procedural topology authority. Structural Start nodes are synthesized from Start slots and intentionally
  have no node-definition pool entry.
- Removed `generation_policy.ron`, depth/lane slot schema, min/max depth gates and procedural generator are not
  accepted as compatibility input or fallback.
- `run/policy.ron` owns required `combat_difficulty.power_model_version = 2`, whose certified roster membership is
  `alive && physical_damage <= 4 && trauma < 100`, plus non-zero certified-power weights,
  qualitative threat thresholds, exact Tutorial/open-ended Expedition anchor schedules (`standard_floor_anchors`/`endless_floor_anchor_stages`), and complete ordinary/role
  profiles. Easy/Standard/Hard must occur once with quotas totaling 100; every special role must occur once;
  Hard and role reward multipliers must exceed Standard. Removed `floor_combat_scaling`, last-stage clamp and
  missing-profile defaults are rejected rather than migrated.
- The fixed Tutorial schedule resolves `tutorial_fixed_v1`, whose `authored_fixed: true` profile retains the
  structurally complete band/role table but applies exact 100% tuning and no certified-power/role adaptation.
  Its live bootstrap owns exactly three single-entry reward pools:
  `tutorial_f1_training_budget`, `tutorial_f3_salvage`, `tutorial_f3_budget`. The F3 final fixed assignment owns an
  actor-only `o-06-20_nothing_there` stat override `720/53/36/33`; validation requires that actor to belong to the
  referenced encounter and applies it only to that matching abnormality spawn, never other actors or occurrences.
- 구 `run/policy.ron.monolith` group과 global stage multiplier는 제거했다. 일반보스 source의
  `revival`은 `catalyst_material_id`, 양수 `catalyst_amount`, 고유 `part_material_id`, `stage_2`,
  `stage_3`를 작성한다. 마탄을 제외한 일반보스에는 필수이며 엘리트/최종보스에는 금지한다.
- 각 단계는 같은 primary kind의 NormalBoss `encounter_id`, `stat_multiplier_percent`, `rewards`,
  `equipment_choices`를 가진다. 2단계>100%,3단계>2단계이며 모든 실제 profile/role 합성은 overflow
  없이 실행 가능해야 한다. 촉매는 `RevivalCatalyst`, 파츠는 `AbnormalityPart` 타입이어야 하고 각
  단계는 자기 작성 파츠를 양수 지급해야 한다. 장비 후보는 유효·비중복이어야 하며0개도 유효하다.
  빈 후보를 다른 장비나 보상으로 보충하지 않는다. 강화는 전투 stat에, stage rewards는 확정 수량에
  쓰며 별도 옛 보상 배율은 없다. 무억제 수량2배는 공통 outcome 계약을 따른다.
- 촉매는 기존 일반 상인의 `revival_catalyst` merchandise로 공급한다. 옛 이름의
  `wandering_monolith_shop`도 현재는 일반 상점이며 모노리스 offer/자동 상점 교체/잔재 교환은 없다.
  `ShopMerchandise::Monolith`의 static authoring은 계속 거절하며 runtime에서도 제안하지 않는다.

### Authored Facility Layouts (U13)

`facility_templates.ron.layouts`는 typed `FacilityLayout` record로 pack 분해/합성 및 definition 조회에 참여한다.
한 publication에서 기본 Expedition layout은 최대 하나다. 방은 stable ID, depth/wing, 내부 template 참조,
예약 활동, 정예 후보, 전조 호환 slot, 표시 좌표/asset ID와 명시적 지원 상태를 가진다. 실행 불가 콘텐츠도
유효한 참조와 미지원 사유를 유지하며 누락 참조를 무시하는 호환 경로를 만들지 않는다.

검증은 ID·참조·연결 방향·도달성·심층 동간 횡단·7층의 지정 최종방 연결·숨은 최종방의 전용 접근방,
예약 충돌·고정 조우 우회·전조/정예 후보와 지원 개수 실현 가능성을 검사한다. 균등 추첨을 받는 자유 slot은
가능한 배정 category 전체를 허용해야 한다. 제약을 만족하지 않는 제작 입력은 publication 전에 거절한다.

지원 총수를 예약된 HQ/정비/상점 포함으로 먼저 뽑는다. 고정 지원0개는0/1/2개40/45/15,1개는1/2개85/15,
2개는2개다. 이후 이벤트·추가보상·정예의 유효 개수 조합을 곱가중치로 뽑고 일반전 최소수를 유지한다.
지원 종류30/45/25와 별도 `facility_rooms_v1` 일반 난도20/50/30은 authored 초기값이다. 위치는 적격 후보의
균등 shuffle이며 좌표나 선택 유도 가중치를 사용하지 않는다. 보상 유형은 초기 고정 SimpleChoice를 제외하고
SimpleChoice/ResourceInvestment/Gambling/RelicGacha/StressCombat 각20%다. 추첨 결과는 run이 보존한다.

공식 전체89방/846활동의 구조 검증, 실제 지원 활동의 개발 profile 플레이, 미지원 고유/최종 조우는 서로 다른
검증 범위다. 다섯 유물 활동은 배정·유형 공개만 지원하고 실제 실행은 U16에서 제공한다. SimpleChoice를 기존 default_treasures의 장비 보상으로 대체하지 않으며 초기 공식 방 전체 진행 성공을 주장하지 않는다. layout이 없는 기존 지원 pack은 기존 node-assignment profile 경로를 유지한다.

### Battlefield Template Authoring

- Live 전장 데이터의 authoring source는 `battlefield_templates.ron`의 ASCII `rows`와 선택적
  `routes.overlay`다.
- ASCII row의 공백은 전장 밖이다. 전장은 사각형일 필요가 없다.
- `width`는 가장 긴 row 길이, `height`는 row 개수로 계산한다.
- 공백이 아닌 타일은 `valid_tiles`가 되고, `tiles`에는 좌표와
  `Ground`/`Platform`/`Obstacle` kind가 함께 노출된다.
- `P`는 지상 배치, `T`는 플랫폼 배치, `#`은 장애물, `N/L/Q/A/B/W/R`은 적 출현 구역,
  `X/Y/Z`는 route endpoint/전술 marker로 해석된다.
- route가 필요한 템플릿은 terrain과 같은 크기의 `routes.overlay`를 작성한다. Overlay는 route의
  시작/종료 marker와 방향 화살표를 표현하며 유효 타일 위에만 놓을 수 있다.

Test-only cached catalogs and `RunPolicyData::builtin()` may support minimal fixtures, but are not
production loaders and must parse/validate the same current schema rather than provide compatibility
or fallback data.

## Fixture Builders

`GameDataBuilder::empty()`와 fixture helper는 테스트용 최소 DB를 만드는 도구다.
`GameDataBuilder::live_defaults()`는 embedded live buff data가 필요한 fixture convenience이며, production live bundle loader가 아니다.

`GameDataBuilder`, bare parts/new, fixture-owned `Arc` conversion과 contract-fixture binaries는 `cfg(test)`,
debug fixture build 또는 명시적인 non-default `test-fixtures` capability에서만 제공된다. 기본 release API와
기본 release binary set에는 존재하지 않는다. 운영 mutation은 data setter가 아니라 typed Core command를
사용한다. Production Core는 `content_pack::load`가 반환한 opaque generation을 사용한다.

## Validation Gates

Live data loading은 gameplay 시작 전에 authoring mistake를 잡는 gate다.

- Parse 단계는 exact manifest version, unknown/missing fields, raw preset resolution을 검사한다.
- Final parsed-bundle validator는 domain shape, duplicate keys, cross-catalog references, map/event/
  reward/RunStart/extra-wave contracts를 검사한다.
- BossOmen chain은 identity, ordered step/encounter reference, 선행 조건과 의존 관계를 bundle publication
  전에 전부 검증한다. Optional required fragment/awakening은 exact source reachability를 가져야 하며
  dependency가 필요한 FinalBoss source를 건너뛸 수 없다. Unknown/self reference, dependency cycle과
  authored progression으로 달성 불가능한 조건은 초기화를 거부하며 fallback chain, 조건 무시 또는
  자동 해금으로 복구하지 않는다. 현재 live membership은 one-step `white_night` 하나뿐이다.
- Generated combat preview validation은 preserved baseline generation과 별도로 모든 live difficulty profile의
  세 ordinary band, Elite/NormalBoss/FinalBoss/BossOmen role과 각 source가 실제 참조하는
  Revival Stage2/Stage3를 고정된 다섯 seed에서 생성한다. Authored Boss node는 `Boss`, 나머지는 `Combat` category를 사용한다.
- 각 manifested preview는 briefing warning을 검사한 바로 그 instance로 empty-player full setup feasibility를
  수행한다. 이 gate는 runtime의 `ResolvedCombatSetupPlan` authority를 재사용해 reward, enemy groups, field/
  tactical plan, Boss actor binding, scenario validation, `BattleCore` construction과 initial execution까지
  성공해야 candidate를 publish한다. Runtime battle start의 defensive validation도 그대로 유지한다.
- Publication은 위 단계가 모두 성공한 뒤 한 번만 일어난다. 실패 시 partial bundle이나 fallback bundle이 없다.
- `tests/ron_loading.rs`와 live catalog tests는 실제 embedded bundle이 runtime 계약을 만족하는지 확인한다.

## Validation Surfaces

- Cross-reference와 generated preview validation: [data/validation.rs](../src/game/data/validation.rs), [combat_preview/validation.rs](../src/game/combat_preview/validation.rs)
- 전체 embedded bundle loading: [tests/ron_loading.rs](../tests/ron_loading.rs)
- Live skill catalog와 activation: [tests/live_skill_catalog_audit.rs](../tests/live_skill_catalog_audit.rs), [tests/live_item_skill_activation.rs](../tests/live_item_skill_activation.rs)
- Skill runtime/data integration: [tests/skill_runtime_contract.rs](../tests/skill_runtime_contract.rs)
- Battle contract validation: [battle/validation](../src/game/battle/validation)

Live RON schema를 바꿀 때는 해당 [data module](../src/game/data), [data/mod.rs](../src/game/data/mod.rs), [data/validation.rs](../src/game/data/validation.rs)를 함께 확인한다. Range/skill authoring 의미가 바뀌면 [skill_target_contract.md](skill_target_contract.md)와 관련 skill validation도 함께 갱신한다.

Live RON schema를 바꿀 때는 보통 다음을 확인한다.

```text
cargo test -p game_core --test ron_loading -- --nocapture
cargo check -p game_core
```

## Skill effect authoring addresses

Schema 2 requires a nonempty effect `id` within each action/step. The effect is one
`AuthoredSkillEffect` containing `id`, optional metadata `tags`, optional `origin`,
and the existing typed `op`. Bare schema-1 effect variants are rejected; there is
no old/new parser alias or second executor. Example inside a step's `effects`:

```ron
(id: "impact", tags: ["red"], op: Damage(amount: 17, damage_type: True))
```

`EffectSite` is `(definition_id, action_id, effect_id)`. Reordering vectors changes
the raw field locator, never the authored address. `SkillDatabase::from_ron` and
its typed constructor bind `EffectSource` records before publication. A source
contains the logical file, authored address, raw field path and original/additional
attribution. `origin: Added(source_definition_id: "other_skill")` requires an
existing definition and records authored provenance; it does not grant a runtime
relic or create a persistent copy. Original attribution uses the containing
SkillId. Missing/duplicate IDs, dangling origin/buff references and unsupported
operations/properties fail without a candidate token. `LiveRonError` reports
source/domain/object/action/effect/field where those identities are readable.
Unparseable text can only report the available parser location; fabricated IDs or
line numbers are never returned.

`ability::authoring::EFFECT_SUPPORT` exposes literal Damage/Heal on Instant, with
typed units and property editability. Damage amount is nonnegative i32, including
zero; Heal is the existing signed i32 HP delta. Damage type is inspectable but not
an amount edit. Existing other ops and spatial delivery continue through their
existing executor; their future editing combinations are not advertised by this
minimal descriptor. Tags cannot enable an opcode. `SameEffect` evaluates all
predicates on one effect; `EachExists` evaluates separate existence predicates.
An empty effect predicate matches existing effects; an empty existence list is
vacuously true for an existing definition. Exact SkillId lookup is independent of
tags and distinguishes authored forms with different IDs.

`set_effect_amount` edits a candidate's supported scalar only. A target/delivery
request returns an action diagnostic and every affected effect address without
mutation. Callers still use normal validated publication for a complete gameplay
bundle. There is no arbitrary property path evaluation or external code entry.
Definition metadata is compile-local and absent from run checkpoint/save formats.
The headless parser is a skill-domain API; `ValidatedGameData` remains the whole
bundle's publication authority.

### Value-expression input and publication boundary

`SkillEffectDef` also has strict authored `DamageExpr` and `HealExpr` forms. Their
amount is exactly `(moment: Snapshot|Live, root: ValueExpr)`; authored input cannot
set provenance. `ValueExpr` is the closed typed set `Const`, `Read`, `Add`, `Mul`,
`Ratio` with required rounding, `Clamp`, and finite `Aggregate`. Existing `Damage`
and `Heal` spellings and numeric fields remain unchanged and are lowered to the
same generation-local immutable plan catalog with legacy provenance.

The external loader bounds the whole file first, scans each expression wrapper
for a 64 KiB input ceiling before retaining the snapshot clone, and then uses
Serde visitors that admit every recursive node/depth and authored member before
owned AST insertion. Default/hard expression parsing ceilings are depth 64,
nodes 1024, and authored members 4096; the existing whole-RON nesting ceiling is
also enforced. RON 0.5.1 supplies authored integers only as signed/unsigned
64-bit values, which are expanded losslessly to the signed i128 calculation
kernel. Floats, string numbers, implicit units, hidden rounding, and unknown
fields/variants are rejected.

After pack resolution, only final winning skill definitions are compiled. The
compiler validates `EffectSite`, units, selected Snapshot/Live delivery
capabilities, resource/result references, constant faults, final Damage u32 or
Heal i32 narrowing, work, and retained limits before placing an `Arc` plan in
`GameDataBase::value_expr_catalog`. Errors preserve the actual source file plus
definition/action/effect/field identity and never substitute zero, a literal, or
a previous value.

Supported authored expressions publish after compiler and runtime capability
checks. Snapshot captures release inputs; Live reads actual effect-start inputs.
Immediate, projectile and area deliveries use the same compiled catalog/kernel.
Resolved members support Attack and HP with explicit ByResolvedOwner relations;
ByOwner on resolved units and unsupported member fields fail publication with
`content_capability_unsupported`. Losing writers do not enter the final catalog.

Signed literal Damage/Heal/ModifyResonance/ModifyTacticalResource amounts decode
through checked i64-to-i32 conversion, including i32::MIN. Negative authored
Damage remains a semantic error; wire spelling and integer range are unchanged.

Amount observation retains one immutable evaluation in the active battle and
one in the last terminal battle. Queries replay that exact plan and frame;
stale exact selectors report not_retained. Optional trace and ordinary event
logs do not own required numeric traces. Release CLI `--amount-observation`
emits at most four documents (preview/explain for active/last-terminal).
Static explanation performs no simulation.

Persistent SpellPower and weapon Attack/SpellPower default to zero and omit
zero fields, preserving prior content/V7 identities. Weapon numbers enter the
shared persistent composition once before growth and binding-ordered Permanent
and exact-copy enhancement. AttackSpeedPoints retain the original percentage;
cadence uses checked ceil(base*100/points). CoreApplication exposes a separate
canonical certified_input_fingerprint; power model weights remain unchanged.
SpellPower's certified weight is pending. Current HP, consumables and readiness
magnitudes do not contribute to this persistent fingerprint.


대표 외부 Damage A/B 검증은 [fixture 생성기](../tools/authoring_combat_fixture.py)와
[실제 combat 비교 도구](../tools/authoring_combat_external_probe.py)를 사용한다. 공통 시험 activation을 가진 두 외부 사본에서
동일 effect amount18→36만 다르며 official source는 바꾸지 않는다. 동일 public finite input의 실제 HP79→61/43 및
16950/12450ms Victory, 결과 소비 후 source/terminal 유지와 off/on/gap·CLI/API receipt equality를 확인했다.
원본 source·출력·실패 discovery·현재 한계는 [실행 근거](goals/content_authoring_tool_ports/execution/single_goal/EXPERIMENTS.md)의045에 보존한다.

같은 실제 A/B 사본의 [공개 교체 검증](../tools/authoring_trial_api_probe/examples/replacement.rs)은 sourced A가 진행 중일 때
B acquisition6/valid-token 준비48실패 뒤 공개 상태·receipt 보존과 A continuation3/B 정상 교체3의 독립 replay 일치를
확인한다. 준비 final receipt/work one-under와 actual authority/command 오류를 포함한다. 전체 loader producer 오류·
세대 참조 drop/allocator 비용은 이 대표 검증으로 닫지 않는다. 원 결과와 도구의 첫 실패·교정은 같은 실행 근거046에 있다.

공개 authoring ApplicationChange에는 실제 ModifyResonance 명령의 before/after/max 채널을 추가했다. 실제 값이
변한 경우만 현재 sourced command의 authored/cast와 연결한다. Resource-only EnemySingle의 기존 usefulness 필터를
우회하지 않으며 시험 self-target은 기존 RON 의미를 따른다. HP 외 다른 미관측 효과 전체를 지원 완료로 표시하지 않는다.

[외부 resource A/B 도구](../tools/authoring_resource_external_probe.py)는 두 prototype의 self-target amount17/31만 바꾸고
동일 실제 실행7950ms에서 resonance11→28/42를 source `skills[21].steps[1].effects[0]`와 결속했다. 공개API6/CLI simulate2/
Explain2가 실제로 통과했고 off/on/gap의 전체 mandatory receipt/usage가 동일했다. 두 generation의 semantic identity는
다르며 공식 RON 변경은 없다. 원 source·출력·binary와 제한은 실행 근거048에 보존한다.

실제 ApplyBuff effect의 immutable authored/cast 출처는 private queue carrier가 이벤트와 함께 소유하고 actual Buff
apply/reapply 때 관측한다. 공개 BattleEvent 필드는 바꾸지 않는다. BuffApplicationState는 actual stacks/expiry/tick의
before/after와 BuffId/kind를 담고 unknown/zero-duration/inactive는 적용으로 세지 않는다. 단순 enqueue를 효과 적용으로
보내거나 기존 Buff tick HP의 direct source를 추론하지 않는다. Queue/source 핸들 회수와 전체 generation 비용 검증은 구분한다.

동일 source의 외부 Buff 검증은 실제 ApplyBuff5회/세대의 source와 Silence700ms/stacks1/expiry를 확인한다. 첫7950ms
적용은 traceoff의 step10 mandatory receipt에도 있고, 결과 소비 후 terminal trace에도 보존된다. 추가 CLI Buff Explain2는
실제 authored source/op과 같은 generation identity를 반환했다. 같은 source/input의 이전 공개 step 결과13개는 변하지
않았고 새 관측·검사 비용은 별도 변화다. 전체12명령과 원 출력은 실행 근거050에서 추적한다.


## Runtime facts in current authoring output

`describe` and effect `explain` expose `runtime_observation`: actual-mutation authority, the supported
Damage/Heal/Cost/ResourceChange/Death kinds, independently optional source slots, action milestones,
and bounded current/aggregate/latest-witness retention. This describes the engine observation
contract; inspection itself does not execute an effect. It does not advertise later Flow/root/fragment
features or add a new pack capability. The existing trial/CLI receipt carries the native active or
terminal runtime-fact projection alongside the R03 mandatory observation and captured SourceReceipt.
Trace disabled, enabled or truncated never determines the native facts.

Player equipment drafts carry `BattleEquipmentBinding { base_uuid, persistent_copy_uuid }`. Actual
EquippedRef.instance_uuid is copied through player setup/runtime construction; definition lookup,
stat aggregation and deterministic runtime item UUID retain their prior roles. Scenario equipment
without inventory identity explicitly carries None. A transient runtime UUID or index never becomes
a persistent copy ID. Existing inventory serialization retains its actual IDs; runtime facts and
opaque action validity tokens are not a new durable save representation.

The [R04 external consumer probe](../tools/authoring_runtime_facts_external_probe.py) uses a separate
current RON fixture and ordinary release authoring/API/CLI ports. Its actual runs and remaining
acceptance work are recorded in the [R04 experiments](goals/content_authoring_runtime_facts/execution/single_goal/EXPERIMENTS.md);
source presence alone is not a completed acceptance result.


## R07 사본별 파편과 명시 저장 전환 (2026-09-29)

스킬 파편 정의와 소유 사본을 구분한다. 단일 UUID-keyed inventory의 `OwnedFragment`가 definition, unlocked forms, selected form, equipped owner를 소유한다. 직원 active slot은 copy UUID 하나만 참조하며 양쪽 binding을 함께 검증한다. 지급은 전용 checked UUID stream과 기존 grant 후보를 사용한다. count는 사본 목록에서 파생한다. 일반 파편 강화 command와 shared rank gameplay 경로는 제거했다.

공개 ActiveSkill RON은 `forms` map의 필수 `base` 및 선택적 형태들을 제공한다. 각 형태는 skill ID, 추가 weapon compatibility, 선택적 bloom recipe를 가진다. 사본 최대1024, 형태 최대16, recipe 항목 최대64이며 중복 map key와 알 수 없는 field/reference를 거부한다. recipe의 enkephalin과 material ID/amount는 checked 합산한다. 기존 공식 콘텐츠에 recipe가 없으면 unavailable이며 임의 비용을 발명하지 않는다. 비활성 BasicAttackModifier placeholder는 intrinsic 사용만 유지한다.

`equip_skill_fragment`/`unequip_skill_fragment`는 employee UUID와 copy UUID, `transfer_skill_fragment`는 copy UUID와 target employee UUID를 받는다. 장착 슬롯이 차 있으면 이전을 거부한다. `awaken_skill_fragment`는 Maintenance에서 exact copy/form을 해금만 하고 `select_skill_fragment_form`이 별도 선택한다. 반복 해금·동일 선택은 무비용 무변경이다. `dismantle_skill_fragment`는 장착 사본일 때 confirmed=true를 요구하며 해제·사본 제거·명시 recipe 산출·typed resource owner 정리를 원자적으로 처리한다. 사망은 exact binding만 풀고 해금·선택을 보존한다.

실제 profile에서 생성한 FragmentSourceSnapshot(copy/definition/form/skill)은 runtime actor, cast/delivery, amount preview/explain 및 trace로 전달된다. FragmentCopy/SourceFragment는 기존 R05 Run/Attempt/Deployment/Activation/Delivery/Chain 수명과 같은 lease 검증을 사용한다. 장비 identity로 대체하지 않는다. 기존 delivery의 immutable source는 이후 inventory 선택에 의해 바뀌지 않는다. 새 사본·형태·owner 상태는 다음 Certified input fingerprint에 반영된다.

R07 도입 당시 live data schema5/active-run V8와 명시 사본 변환은 역사적 계약이다. 현재는 catalog schema6,
Core Continue12/독립 SavePoint13을 사용한다. `CoreApplication::convert_active_run_v7`/`convert_active_run_v8`은
현재 미지원으로 명시 거절하며 원본을 쓰거나 새 bytes를 반환하지 않는다. 이전 V7/V8/V9 원자료와 동결 target은
당시 변환/보존 증거로 남기며 현재 수용이나 자동 fallback의 근거가 아니다.


## U05 resonance authoring

`SkillFragmentForm.resonance_max` is required and positive. The selected copy/form supplies the employee
capacity; the persistent employee `battle_profile` rejects the removed `resonance` object. Opponent autonomous
profiles retain their separate start/max/recovery contract. `PlatformRestProfile` accepts only HP recovery;
`resonance_recovery_per_tick` is a strict unknown-field error.

Every `SkillDef` requires `use_lifetime: Action` or `ActionAndEffects(effects: [(step: "...", effect: "...")])`.
Each reference is unique and must identify a finite beneficial Sustained effect or ShieldExpr. Every such effect
requires a reference. Hostile Control/ApplyBuff (CC/DoT), permanent stats and ordinary immediate operations
cannot be selected. An independent persistent TileArea producing a finite beneficial use effect is currently
rejected with an explicit publication error; separate independent areas and finite effects in one skill remain
supported. RON and typed builders use the same validator. Employee manual fragment forms reject RetargetOnStep,
including AutomaticUnit selection; opponent autonomous targeting retains its existing authoring contract.

Employee의 personality/condition/성장은 유지하고 trust/memory aggregate와 전용 입력은 제거한다.
현재 Continue12/SavePoint13 strict decoder는 제거 필드/구 mode/이전 version을 거절한다.
V7/V8 conversion은 현재 미지원이며 역사적 default 변환으로 현재 reader를 우회하지 않는다.

## U06 기본 Heal 및 초기 active loadout

Weapon `action: Heal(flat: ..., spell_power_percent: ..., footprint: None|Some(...))`은 Instant delivery와
엄격한 양/geometry 검증을 사용한다. `AllySingle`의 공개 selector 이름은 `ally_single`이다.
`CombatModifiers` sustained는 duration과 damage_reduction_percent/basic_lifesteal_percent/
basic_lifesteal_max_heal을 명시한다. 비율은0..100, 효과 전체0은 거절하고 흡혈 enabled일 때만
양수 signed-range cap을 허용한다. 유한 beneficial 효과이므로 `ActionAndEffects`의 exact 참조가 필요하다.

EmployeeInitialLoadout의 `active_skill_fragment_id`는 선택적 실제 active 사본 지급이다. baseline IDs에는
BasicAttackModifier만 허용한다. 초기 장비를 적용한 실제 profile과 active base form의 호환성을 로더에서
검사한다. 공식6starter+6HQ는 각각 active1개와 역할 무기를 명시하며 새 훈련 무기는 일반 보상 pool에서 제외한다.

`Damage`는 기존 암묵 기본값이며 BasicAttackDef/작성 basic profile/WeaponCombatProfile의 직렬화에서 생략한다. `Heal`은 명시 보존한다. 같은 V12 저장의 기본 피해 profile에 새 필드를 덧붙여 기존 canonical checksum을 무효화하지 않는다. 원래 암묵 Damage 문서의 checksum 검증·Continue·재출력 동일성과 신규 Heal loadout의 정상 저장을 각각 검증한다.

## U07 independent object authoring

SummonedUnit catalog의 optional `objects`는 기존 BossRuntime `definitions`와 별개다. `IndependentObject`
정의 kind는 Actor/Trap/Wall이며 ID/UUID, 양의 TTL·생존/누적 상한, family 일관성과 비순환 동종 evolution
참조를 strict 검증한다. Actor의 실제 combat profile/skill reference, Trap의 범위·양의 optional max_targets,
Wall의 HP/방어 계약을 함께 검사한다. 기존 objects 부재의 canonical 직렬화는 유지한다.

`CreateObject`와 `SegmentedShield`는 schema뿐 아니라 실제 실행/공개 explain/trial consumer를 가진다.
Instant/Once·명시 target·flow/damage-attachment 제한, 비용/진화 조합과 정확한 definition 참조를 발행 전에
검사한다. 세부 작성 예시는 [작성 모델](skills/skill_authoring_model.ko.md#u07-독립-객체분절-pool-작성-포트)을 따른다.
새 래티씨아 forms와 벽 consumer는 기존 catalog identity 안의 명시 정의이며 누락 참조를 fallback으로 숨기지 않는다.


## U14 작성 필드

- `MapNodeDefinition.first_entry_cost`와 `FacilityActivityDefinition.first_entry_cost`는 선택적 u32다.
  activity→node→run 기본값 순서이며0은 무료다. Start/Gate는 무료이고 `breakdown_collapse_cost_limit`의
  공식값은4다. 이전 `breakdown_collapse_node_limit`은 지원하지 않는다.
- PVE `deterrence_tuning`은 무억제 HP/ATK 각150–200%, 봉인 HP/ATK 각100% 초과를 검증한다.
  공식 초깃값은 모두150이다. 조우 안에서 다시 배율을 추첨하지 않는다.
- PVE `bonus_objectives`는 고유ID, `ClearWithin { time_ms }` 또는
  `DecisiveDamage { max_health_percent }`, 양수 수량 효과를 작성한다. 시간은 양수이고 해당 조우의 주환상체가
  시작시 존재해야 한다. 피해비율은1–100이다. 재화/장비가루/장비재료/AliveRoster XP만 허용하고 참조를 검증한다.
- Facility activity `golden_bough_candidate`, layout `golden_bough_plans`를 함께 작성한다. 계획은 고유ID,
  양수weight/refill, 고유room/slot 목록을 소유한다. 후보가 있으면 여러 계획을 요구하고 각 계획은 모든 후보
  wing을 포함하며 ENTRY에서 무조건 도달 가능한 비최종보스 후보만 참조한다. 정상 지원 후보는 NormalBoss
  조우여야 한다. 공식 예약 미지원 본체는 명시 Unsupported 상태로 남고 U21에서 본체를 연결한다.
  적합 계획 누락을 보스별 랜덤 선택으로 대체하지 않는다. 공식 두 계획은 A5/B4/C4 또는 A5/B7/C4,
  각 weight50/보충8이다. 이는 초기 작성 튜닝이며 개별 네 보스의 플레이 완료 증거가 아니다.


## U15 재료 목적과 성장 recipe

`EquipmentMaterialType`의 `EquipmentDust`, `AbnormalityPart`, `RevivalCatalyst`는 서로 다른 목적이다.
무기 enhancement recipe는 Weapon 대상과 EquipmentDust 비용, Attack/SpellPower modifier만 허용한다.
Armor 대상이나 방어력/HP 강화, 다른 목적의 재료를 무기 강화 비용으로 작성하면 publication에서 거절한다.
파편 개화 recipe의 재료는 AbnormalityPart여야 하며 요구량과 동반 enkephalin은 각 형태의 RON에서 온다.
부활 촉매와 장비 가루를 개화 비용으로 대체할 수 없다. 중복 cost row는 checked 합산하며 0·overflow·
미지 material reference는 거절한다. 공식 기본 무기는 현재 개발값 dust2/단계최대5와 Attack/SpellPower
보정을 제공한다. 이 값은 최종 균형 확정을 뜻하지 않는다.

성장 후 분쇄 환급의 기준은 저장된 실제 결제 원장이며 새 generation의 가격표가 아니다.
새 사본의 빈 투자 원장과 구 저장의 필수 투자 필드 누락은 구별한다. U15 구 개발 저장은 자동 변환하지 않는다.


### U16-1 상점 상품 작성

`ShopMerchandise`는 기존 장비·소비품 외에 `relic`(relic_id),
`revival_catalyst`(material_id), `relic_corruption_catalyst`를 지원한다.
유물과 촉매 상품은 양수 `base_price`를 직접 작성한다. 장비·소비품은 기존 카탈로그 가격을
사용하므로 이 필드를 작성하지 않는다. 유물·장비 수량은1이며 촉매 수량은 양수이다.
부활촉매 참조는 `RevivalCatalyst` 목적의 재료만 허용한다. 장비 가루·환상체 파츠는 상품이 아니다.
모노리스는 원정 진행에서 생성하므로 일반 상품 목록으로 직접 작성할 수 없다.

`guaranteed: true` 항목은 구매이력에 따른 자격을 확인한 뒤 가중 추첨보다 먼저 페이지에 넣는다.
기본값은false이며 기본상품 개수는 페이지 용량 이하여야 한다. 떠돌이 상점은 초기 모노리스
두 자리도 확보해야 한다. `repeatable: false` 상품은 구매 후 같은 세션의 갱신에서 제외된다.
모든 상품은 기존 상점 비율·올림으로 총가격을 확정한다. 조회의 유물 정의·촉매 목적과 구매 결과의
`relic_diffs`/`crafting_resources`/재료스택을 통해 소비자가 실제 지급 owner를 구분한다.


### U16-2 유물 활동 공급 작성

FacilityLayout의 `relic_rewards`는 `candidates: [(relic_id, weight)]`와
`dust_thresholds: (3,9,18)` 형태의 누적 비용 세 개를 가진다. 생략 시 빈 후보와 이 초기 비용을
사용한다. 후보 ID는 유물 정의를 참조하고 중복이 없어야 하며 weight는 양수, 전체 합계는 u32
범위여야 한다. 공급 후보의 rarity는 common/rare/special이어야 한다. 이 조건은 다른 용도의
기존 유물 전체에 적용하지 않는다. 비용은 양수이며 엄격히 증가한다. 빈 공급은 유효하다.

`reward_activity`는 Reward category와 Reward payload를 갖는 활동에만 작성한다. 초기
공급 가중치1과 비용3/9/18은 작성 튜닝이며 최종 균형 확정이 아니다. 선택형은 전체 후보에서,
투자/합성은 해당 등급에서 가중 추첨하며 선택 후보는 종류 중복 없이 최대3개다. 합성은 투입
두 종류를 제외한 뒤 양수 확률 모든 등급의 후보 존재를 확인한다. 후보 부족을 다른 등급으로
재분배하지 않는다. 외부 pack은 실제 효과 정의와 함께 공급을 작성해야 한다. 공식 유물 catalog의
확장과 나머지 유물 활동은 후속 단계의 범위다.

### U16-3 딜러 작성

FacilityLayout의 optional `dealer: Some((...))`가 Gambling의 실행/조건/VN/상품 정책을 소유한다. 생략한 기존layout은 딜러미지원이며 silent default policy로바꾸지않는다. None은직렬화에서생략하여기존pack의정규화의미를보존한다. 공식layout은초기기본30·갬빗비중1:1·최소판돈1·Flip손실10/1위보너스20·원금10/30/60·기타rarity default_pledge_value10·빈상품과제안/VN참조를작성한다. 지원외부pack은실제유물정의/효과를상품에작성한다.

기본포인트/최소판돈/Flip손실/등급별원금/default_pledge_value는양수이고갬빗2/3비중의u32합계는양수/overflow없음이어야한다. 상품은고유비어있지않은id,하나이상유효유물id,양수가격/재고를갖는다. 묶음은여러정의(동일종류여러사본포함)를허용한다. first_vn/repeat_vn은terminal accept/decline 두선택, gambit_vn은terminal yes/no 두선택을갖는유효VN entry를참조한다. 숫자·참조·선택계약오류는publication에서거절한다. 카드/주사위/정산규칙은고정Core규칙이고pack이새게임코드·RNG배열·임의포인트상태를주입하지않는다.

### U16-4 스트레스 전투 작성

FacilityLayout의 optional `stress_combat: Some((encounter_id:..., rounds:[(wave_ids:[...],rewards:[...]), ...]))`가 활동 전용 연속 방어전을 정한다.5개의 비어 있지 않은 라운드가 조우의 모든 required 웨이브를 중복 없이 분할해야 한다. 조우 class는 StressCombat, mission_variant는Defense이며 ProtectUnit 목표와 같은 보호 대상의 UnitDefeated 패배 조건을 작성한다. 승리는 AllRequiredEnemyGroupsDefeated이고 별도 시간 승리/bonus_objectives는 사용하지 않는다. GroupCleared와 actor trigger 바인딩 모두 뒤 라운드의 wave를 참조하여 앞 라운드를 잠그는 입력은 거절한다.

라운드 rewards는 고정 엔케팔린/장비 가루/재료/XP/유물의 공통 RewardEffect이며 참조와 XP 대상 계약을 로더가 검증한다. 선택 직원을 추가로 요구하는 SelectedEmployee XP와 고정 무억제2배 수량을 표현할 수 없는 보상은 거절한다. 엔케팔린·장비 가루·동일 material_id의 재료는 같은 라운드의 반복 효과와 모든 라운드를 합한 잔고별 누적량에도 이 검사를 적용한다. GrantEquipmentMaterial의 예약 ID equipment_dust는 GrantEquipmentDust와 같은 가루 잔고로 합산하며, 서로 다른 재료·잔고는 합치지 않는다. XP는 개별 지급마다 레벨업 비용을 소비하므로 기존 효과별 배율 검사를 유지한다. 이 작성 검사는 고정 보상 자체의 표현 가능성을 확인하며, 기존 보유량과 더할 때의 공통 런타임 잔고 한도·원자성 처리는 유지한다. explicit supported StressCombat 활동에 policy가 없으면 로더가 거절하며, 기존 optional policy 없는 layout은 이 활동을 미지원으로 안내한다. 공식 대표 조우는1/1/2/2/3명의I/II/II/III/III 적, 라운드당 엔케팔린10/20/30/40/50이다. 이 수치와 지원 외부 pack의 유물·재료·XP 표는 U21 최종 콘텐츠/밸런스 확정을 뜻하지 않는다.
