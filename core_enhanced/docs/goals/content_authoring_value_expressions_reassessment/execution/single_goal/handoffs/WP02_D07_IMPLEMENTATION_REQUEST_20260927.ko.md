# R06 WP02 D07 — 코드 검토·단계 계획과 첫 구현 지시서

2026-09-27 · 현재 Goal/WP와 사용자 정책 유지. 계획·독립 검토는 원 대화, 구현은 별도 대화.
**다음 단계검토 범위는 D07-A: 정적 compiled amount 설명과 평가 문맥의 명시**다.
D07-B 실제 frame 수치 관측과 D07-C/WP03 저장·최종 입력 연결은 뒤에 남긴다.
A/B/C는 이 문서 안의 단계 구분이며 새 로드맵 번호나 별도 Goal이 아니다.

## 1. 구현 담당자에게 전달할 요청

> 이 문서의 D07-A를 구현해줘. 기존 `ExplainQuery::Effect`가 해당 효과의 실제 compiled amount plan을
> 읽어 수식 구조·단위·moment·필요 입력·출처·fingerprint·비용 설명을 반환하도록 연결해줘.
> 문맥 없이 계산 가능한 식과 runtime 문맥이 필요한 식을 구별하되, 이번 정적 설명에서는
> 값을 계산하지 않았다는 사실을 명시해줘. 미평가 값을0이나 실제 피해량으로 표시하지 마.
>
> 기존 authoring API/CLI 흐름과 출력 한도를 재사용하고, 전체 authored expression의 production publication
> 차단은 유지해줘. 아래 파일·순서·A01–A10 검증 기준을 따라 구현해줘.
> 구현 전 currentness와 실제 before baseline을 보존하고, 자체 검증 뒤 새 단계검토 target을 동결해줘.
> 검토 인계서와 manifest SHA를 보고한 뒤 중단해줘. D07-B·WP03/04나 전체 완료 판정으로 이어가지 마.

## 2. 근거와 정확한 시작점

- [D07 원 설계](../../../CODE_REVIEW_003.ko.md), 특히 D07/D09, V17.
- [successor 계획 §5](../../../successors/r06_scope_20260927_001/REVIEW_AND_PLAN.ko.md#5-구현-순서와-완료-판단).
- [최신 독립 재검토 PASS](../reviews/wp02_d04_owner_f01_correction_001_review_001/REVIEW.md).
- [시작 manifest](../corrections/wp02_d04_owner_f01_001/target/d04_owner_f01_correction_001/MANIFEST.json).
- [이번 코드 대조 증거](WP02_D07_CODE_EVIDENCE_20260927.json).

| 대상 | 값 |
| --- | --- |
| manifest SHA-256 | `58505bbcd34089b9c1ec3d70f4d9df4ca32a2397e1115c68386364e44514e549` |
| source337개 after digest | `8aa03d6bd04f8d524fa61b465a9bd7a4171062a1190f615b794e20f915190f04` |
| 제품333개 after digest | `8f6a58059254d4db4dc726bc8aeb2a6c6ec348e9aef685052837df903fd94bcd` |

문서 작성 시337개 after 파일의 크기·SHA가 현재 source와 일치했다. 제품 구현·시험은 수행하지 않았다.
구현 시작 시 `corrections/wp02_d04_owner_f01_001/freeze.py verify`를 다시 실행한다.
불일치는 먼저 조사하며 reset/clean/stash로 기존 변경을 제거하지 않는다.
새 구간은 `segments/wp02_d07_static_explain_001/`, target은 `d07_static_explain_001`을 권장한다.
이미 있으면 내용을 먼저 확인하고 덮어쓰지 않는다. before는 위 successor의 **after**다.

## 3. 실제 코드에서 확인한 현재 상태

경로는 `core_enhanced/` 기준이다.

| 위치 | 현재 동작과 의미 |
| --- | --- |
| `src/authoring/query.rs::ExplainQuery::Effect` / `EffectExplanation` | skill/action/effect를 실제 publication에서 찾고 정의·effect 출처, target/delivery/support를 설명한다. compiled amount plan은 출력하지 않는다 |
| `src/authoring/compiled.rs` | 현재 boss graph의 borrowed projection이다. 이름만 보고 ValueExpr 공통 실행기로 재작성하지 않는다 |
| `src/game/value_expr/catalog.rs::ValueExprCatalog::get` | 최종 winning EffectSite별 immutable plan을 찾을 수 있다. legacy Damage/Heal도 이미 plan을 갖는다 |
| 같은 파일 `ensure_runtime_publishable` | authored DamageExpr/HealExpr production publication은 여전히 거부한다 |
| `CompiledAmountPlan` | `segment`, `segment_count`, `result_unit`, `moment`, `provenance`, `requirements`, `cost`, `source`, fingerprint accessor가 있다 |
| `catalog.rs::can_evaluate_without_external_context` | resolved targets가 없고 references가 bare AggregateMember뿐인 경우를 context-free로 분류한다. authored aggregate와 runtime member stat을 구별한다 |
| `src/authoring/output.rs` / `query.rs::run_query` | bounded JSON writer와 Inspection 실행 한도/취소를 이미 적용한다 |
| `TrialSession::explain` | content의 정적 explain으로 위임한다. 세션이 있다고 현재 전투 frame을 읽는 것은 아니다 |
| `TrialPreview` / `TrialSession::preview` | 현재 deployment range와 skill target 공간 질의다. application fork를 사용하며 숫자 amount/trace API가 아니다 |
| `battle/core/value_expr_runtime.rs` | capture identity를 확인하고 Snapshot의 release frame 또는 Live의 현재 frame으로 공통 evaluator를 호출한다. 결과는 `RuntimeAmountEvaluation { amount, evaluated, usage }` |
| `battle/core/sim.rs` | effect target별 평가 결과에서 `amount`만 보관한 뒤 mutation한다. `evaluated.trace`를 공개 조회 가능한 역사로 보관하지 않는다 |
| `target_usefulness.rs` | DamageExpr에는 적 대상 eligibility를 허용하고 소비 전 amount 평가를 하지 않는다. 기존 literal final-damage 경로와 구별된다 |

따라서 공간 preview를 숫자 preview로 이름만 바꾸거나, static explain에 기본 EvalFrame을 넣어
동적 식의 수치를 추정하는 방법은 사용할 수 없다. 공개 expression 차단을 풀어서 설명 시험을 통과시키지도 않는다.

## 4. D07 전체 진행 순서와 의존성

| 단계 | 산출 범위 | 완료를 주장할 수 없는 것 |
| --- | --- | --- |
| **D07-A: 이번 구현** | 정적 plan 설명·context-free/needs-runtime-context·명시적 미평가·실제 출처와 fingerprint·bounded 출력 | actual frame 수치 preview, runtime/preview/explain amount·trace 동등성 |
| D07-B: A 검토 후 별도 지시 | 실제 읽기 전용 frame과 evaluator 연결, 숫자 amount와 trace, quantitative usefulness의 Indeterminate 표현 | 미래 result·미확정 target의 가짜 확정값, full stat/save 호환 완료 |
| D07-C: WP03 연계 | 최종 effective stat/copy 입력, CertifiedInputFingerprint, zero-field omission과 실제 old-build V7 호환 | 모델 weight 미정인 SpellPower를 평가 total 지원이라고 표시하는 것 |
| WP04 | public RON/API/CLI→실제 실행→관측 전체 증거와 기존 회귀 통합 | 단계별 test-only 성공만으로 전체 public capability 지원 선언 |

### D07-B에 전달할 설계 방향

공통 계산은 기존 `evaluate`와 `narrow_damage/narrow_heal`을 사용한다. UI/query용 별도 계산식을 만들지 않는다.
실제 runtime 소비가 고른 **동일한 frozen frame**에서 amount/trace를 비교해야 한다.
Snapshot은 capture의 release frame, Live는 해당 occurrence/순서의 읽기 frame이다.
pre-release 예측을 제공한다면 prospective로 표시하며 실제 committed Snapshot과 동일하다고 하지 않는다.

현재 trial은 effect 중간 frame에 직접 멈추는 공개 포트가 없고, 처리 후 trace도 보관하지 않는다.
B 구현 지시 전에는 `evaluate_value_expr_capture_with_resolved`와 ordered executor 사이의 읽기 전용 관측 경계를
구체화해야 한다. 새 영구 history나 무제한 frame cache를 당장 추가하는 방식은 택하지 않는다.
runtime의 `RuntimeAmountEvaluation`을 재사용할 수 있는 범위를 먼저 확인한다.
optional trace가 꺼졌다는 이유로 필수 numeric query를0으로 반환하거나, 이미 정리된 occurrence를 현재 state로 재현하지 않는다.

generation/site/plan fingerprint/action/delivery/source-copy/target identity를 결속하고
외부 입력이 임의 EvalFrame이나 action/resource capability를 만들어 넣게 하지 않는다.
같은 frame의 amount는 RNG/mitigation 이후 최종 피해와 별개로 설명한다.
숫자를 판단할 수 없는 경우는 Indeterminate/needs_runtime_context이며, 기존 적 대상 eligibility와 분리한다.
**이번 A에서는 target selection/gameplay bool을 변경하지 않는다.**

D07-C는 [원 설계 D05/D07](../../../CODE_REVIEW_003.ko.md)의 WP03 의무를 유지한다.
새 schema를 쓰기 전에 이전 build의 실제 저장 bytes를 보존하고 checksum/identity bypass 없이 검증한다.
이번 A가 그 저장 준비나 최종 fingerprint 구현을 끝냈다고 표시하지 않는다.

## 5. D07-A 출력 계약

기존 `ExplainQuery::Effect { site }`를 유지하고 `EffectExplanation`에 **선택적 `value_expression` 필드**를 추가한다.
금액 plan이 없는 정상 effect는 `null`, Damage/Heal 및 내부 staged DamageExpr/HealExpr는 아래 descriptor를 갖는다.

| 필드 | 값/출처 |
| --- | --- |
| `representation` | `compiled_amount_plan` |
| `kind` | catalog entry의 Damage/Heal |
| `compiler_version` | 현재 `VALUE_EXPR_COMPILER_VERSION` |
| `provenance` | plan의 legacy damage/heal 또는 authored-expression 출처 |
| `moment`, `result_unit` | plan accessor 그대로 |
| `source` | plan의 실제 `EffectSource` |
| `semantic_fingerprint` | plan의 기존 hex fingerprint, 새 query 전용 digest를 만들지 않음 |
| `requirements` | 기존 ContextRequirements 전체: references, resolved set, owner/tile 관계, 명시적 필터 포함 |
| `cost` | 기존 PlanCost 전체. static_work/dynamic 여부/worst_case_work를 구별 |
| `segments` | compiler 순서의 `{ index, ops }` 목록. aggregate의 member_segment 연결 유지 |
| `evaluation.context` | `context_free` 또는 `needs_runtime_context` |
| `evaluation.performed` | 이번 descriptor에서는 항상 `false` |
| `publication` | legacy plan은 기존 지원 상태, authored plan은 `staged_only` |

`context_free`는 **외부 문맥 없이 평가할 수 있다**는 뜻이며 이미 평가했다는 뜻이 아니다.
A에서는 신규 `amount`, `trace`, `final_damage`, 임의 예상값을 채우지 않는다.
기존 effect 원문 안의 literal amount는 원래대로 보존한다. 그 원문과 계산 결과를 혼동하지 않는다.
context가 필요할 때 기존 requirements가 그 이유를 설명하며 필수 입력 목록을 별도 손코딩하지 않는다.
액션이 없는 정적 설명에서 ActionHandle/capture/source-copy를 발급하거나 runtime callback을 호출하지 않는다.

`describe`에는 작은 additive 지원 설명을 넣는다. 정적 plan 설명은 가능하지만 이번 단계의
새 numeric preview는 미지원이고 authored expression publication은 여전히 차단되어 있음을 명시한다.
기존 공간 preview/기존 runtime facts 기능을 없다고 표시하지 않는다.
`PackManifest::SUPPORTED_CAPABILITIES`에 새 실행 capability를 추가하지 않는다.

## 6. 구현 순서와 파일별 지시

### 6.1 context 분류를 공유

`catalog.rs::can_evaluate_without_external_context`의 기존 의미를 재사용한다.
가장 작은 변경은 이 함수를 `pub(crate)`로 하고 `value_expr/mod.rs`에서 crate 내부 재노출하는 것이다.
catalog validation과 query 모두 같은 함수에 의존하게 한다. 새로운 public 게임 API로 넓힐 필요는 없다.
`references.is_empty()`나 `owner_grouping == false`만으로 대체하면 authored AggregateMember를 잘못 분류한다.
resolved unit의 Attack/HP, Source/Target stat, resource/time/result 참조는 기존 분류대로 문맥을 요구한다.

### 6.2 borrowed descriptor를 추가

`src/authoring/amount.rs` 같은 작은 내부 모듈에 `CompiledAmountExplanation`과 segments의 Serialize view를 둔다.
`authoring/mod.rs`에서 내부 모듈로 연결한다. `compiled.rs`의 boss graph 코드는 그대로 둔다.
`CompiledOp`는 이미 Serialize를 지원하므로 독립 AST를 다시 만들 필요가 없다.
전체 segments/requirements/source/compiled plan을 clone하거나 `serde_json::Value`로 복제하지 않는다.

segment view는 `0..plan.segment_count()`를 순서대로 SerializeSeq에 출력하고,
각 index에서 `plan.segment(index)`의 borrowed ops를 사용한다. member segment를 재귀 전개하지 않는다.
없는 segment를 빈 ops로 숨기지 않는다. 정상 immutable compiler plan의 범위를 사용한다.
분류와 descriptor builder는 immutable `CompiledAmountEntry`를 입력으로 받고 게임 state를 읽지 않는다.

정수는 기존 typed serializer를 통해 JSON writer로 직접 출력한다.
i128 payload를 float나 `serde_json::Value` 왕복으로 변환하지 않는다. 큰 값의 출력 검증은 원문 또는
해당 typed 정수 deserializer로 하며, JSON 소비자의 부동소수 정밀도를 값 의미로 끌어들이지 않는다.

### 6.3 실제 EffectSite로 연결

`query.rs::ExplainQuery::Effect`에서 기존 skill/step/effect 존재 확인을 먼저 유지한다.
그 뒤 `content.token.value_expr_catalog().get(site)`에서 winning plan을 읽는다.
plan의 site/source와 조회한 effect가 일치하는지 확인한다. effect가 원래 다른 정의에서 추가됐다면
기존 `origin_definition_source`와 plan source의 origin을 함께 보존한다.
entry kind도 해당 Damage/Heal effect 종류와 일치해야 한다. 불일치를 정상 descriptor로 직렬화하지 않는다.

Damage/Heal/DamageExpr/HealExpr인데 plan이 없으면 `null`로 숨기지 말고
`authoring_amount_plan_unavailable` 같은 명시적인 RequestFailure로 반환한다.
plan이 없는 정상 비금액 effect만 `value_expression: null`이다.
존재하지 않는 effect/action/definition의 기존 오류 코드는 유지한다.
요청 때 RON을 다시 읽거나 compile하지 않는다. catalog의 immutable publication만 사용한다.

static descriptor helper는 최종 `output::encode_result(&EffectExplanation { ... })`에 연결한다.
`TrialSession::explain`과 CLI `explain`은 기존 위임 경로로 이 결과를 받는다.
새 ExplainQuery variant·arbitrary command·TrialPreview variant를 추가하지 않는다.

### 6.4 한도와 취소를 보존

query는 계속 `run_query`의 Inspection scope 안에서 실행한다.
descriptor 준비/plan 검사/segment 순회/fingerprint 문자열에 필요한 작업을 기존 control에 계량한다.
새 catalog lookup도 catalog 크기와 EffectSite의 문자열 비교 길이를 고려해 조회 전에 계량한다.
설명 하나를 만들기 위해 catalog 전체를 순회하거나 모든 plan을 수집할 필요는 없다.
길이 합산은 checked 방식으로 하며 overflow를 작은 값이나 성공으로 바꾸지 않는다.
compiler의 기존 PlanCost와 길이 정보로 보수적 scan 상한을 잡을 수 있지만 비용 숫자를 query가 재정의하지 않는다.
정렬·중복 제거는 이미 compiler가 수행했으므로 descriptor에서 반복할 필요가 없다.

borrowed plan의 기존 소유 메모리를 새로 clone한 것처럼 할당하지 않는다.
새로 소유하는64자리 fingerprint 문자열 등 작은 descriptor 데이터는 allocation 전에 metadata admission한다.
출력은 기존 `output::encode_result`의 ResultBytes/work 제한을 통과해야 하며
먼저 무제한 `to_string`/`to_value`를 한 뒤 길이를 확인하는 방법은 금지한다.
긴 AuthoredMembers·reference/source 문자열에도 취소/낮은 한도에서 완성된 성공 응답이 반환되지 않아야 한다.
이 단계에는 evaluator 호출이 없으므로 runtime 평가 예산이나 메모리 한도를 새로 조정하지 않는다.

## 7. 필수 검증 A01–A10

public authoring 시험은 기존 `src/authoring/tests.rs`의 `ExternalFixture`와 `attempt` 흐름을 사용한다.
authored expression은 production 차단 중이므로 내부 descriptor 시험과 공개 거부 시험을 명확히 분리한다.
staged plan을 관찰하려고 검증 토큰을 위조하거나 loader의 publishability 검사를 우회하는 public API를 만들지 않는다.

| ID | 시험과 기대 결과 |
| --- | --- |
| A01 | 실제 외부 legacy Damage/Heal의 public `explain`: 기존 필드 보존, catalog와 kind/unit/moment/source/fingerprint/ops가 일치 |
| A02 | 정상 비금액 effect는 null, 없는 definition/action/effect는 기존 오류. 금액 effect의 plan 부재는 명시적 오류로 분리 |
| A03 | 내부 compiled plan에서 Const, AuthoredMembers+bare AggregateMember는 context_free. Source/Target stat, resolved member stat/HP, resource/time/result는 needs_runtime_context. 둘 다 performed=false |
| A04 | 실제Side/Employee/EquippedSlot·명시적 필터·여러 aggregate/중첩 segment의 구조와 requirements를 손실 없이 출력. member_segment index를 평탄화하거나 잘못 연결하지 않음 |
| A05 | same-ID override를 실제 loader로 두 generation에 적용하면 설명이 각 winning source와 plan fingerprint를 따른다. 옛 publication의 immutable 설명은 바뀌지 않음 |
| A06 | 낮은 max_work/max_result_bytes/metadata, 취소 token, 긴 plan payload에서 typed 거부.0건 selector나 결과를 잘라 성공 반환하는 것은 불가 |
| A07 | 조회 전후 publication identity/plan fingerprint 불변, 같은 입력의 설명 결정적. runtime action/RNG/queue/resource/target 상태를 생성하거나 소비하지 않음 |
| A08 | 공개 DamageExpr/HealExpr는 기존 content_capability_unsupported 유지. describe가 정적 설명 지원과 authored publication 미지원을 혼동하지 않음 |
| A09 | CLI `explain`과 `TrialSession::explain` 위임 결과에 새 descriptor가 연결됨. 기존 compiled boss graph와 공간 preview의 결과/지원 상태 회귀 없음 |
| A10 | 정적 descriptor 출력에 실제 numeric evaluation/trace/final damage가 없고 performed=false임을 확인. 부족한 문맥을0/false amount로 표시하지 않음 |

기존 `explain_preserves_actual_definition_effect_origin_and_rejects_missing_sites`,
`explain_prices_long_action_and_effect_comparisons_before_searching`, compiled graph 및 trial preview 시험을 유지한다.
내부 descriptor에서 evaluator와 비교해도 그것만으로 실제 frame preview 동등성(V17)을 완료 처리하지 않는다.
기존 D04/F01 회귀를 전체 제품 시험에서 함께 유지한다.

## 8. 실행·동결·종료

편집 전에 새 segment PLAN과 actual before archive/manifest를 만든다.
기존337개 source를 시작 집합으로 하고 새 `authoring/amount.rs` 등 파일은 after 집합/추가 목록/양방향 복원에 포함한다.
추가 파일의 before는 부재다. 기존 freeze 도구의 고정337개/변경5개 경로 assertion을 무조건 재사용하지 않는다.
구간에 맞는 복사본을 작성하고 이전 frozen target·evidence는 보존한다.

최종 source에서 변경 영역 format, 다음 check/시험/clippy를 실행한다.

```bash
cargo check -p game_core --all-targets --offline --locked
cargo test -p game_core --lib authoring --offline --locked
cargo test -p game_core --lib value_expr --offline --locked
cargo test -p game_core --all-targets --offline --locked
cargo clippy -p game_core --all-targets --offline --locked -- -A clippy::never_loop -A clippy::eq_op
```

CLI 연결은 기존 root/selection/request fixture로 실제 명령까지 확인한다.
명령 형식은 `author_content explain TRUSTED_ROOT SELECTION.json REQUEST.json`이다.
예를 들어 같은 product target에서 `cargo run -p game_core --bin author_content --offline --locked -- explain ...`로
최종 source의 실행 파일을 빌드·실행한다. `...`에는 실제 fixture의 세 경로를 넣어 영수증에 기록한다.
fixture 파일 경로는 구현 중 실제 존재하는 값으로 기록한다.0개 시험 실행이나 compile-only 결과를 API 실행 증거로 쓰지 않는다.
이번은 authoring 설명 경로이므로 **변경된 query의 비용/취소 시험이 필수**다.
runtime 제품 경로나 소유/저장 구조를 바꾸지 않았다면 PC36/R05의 재측정을 관례적으로 강제하지 않는다.
관련 경로 변경이 필요해 범위가 넓어졌다면 이유와 영향에 맞춰 해당 guard를 추가하고 기록한다.
[기존 비용 정책](R06_COST_POLICY_UPDATE_20260927.ko.md)은 계속 적용하며20분 시험·자동 스킬 취소는 제외한다.

제품/격리 probe는 서로 다른 CARGO_TARGET_DIR를 사용한다.
이 환경의 cargo는 `/home/blast/.cargo/bin`, offline cache는 `/tmp/wp02-cargo-home`이었다.
빌드 target은 `/tmp` 공간 부족을 피하고 다른 실행과 분리한다. 예:
`/home/blast/.cache/d07-static-explain-product-target`.
실제 command/cwd/env/toolchain·stdout/stderr·exit·시험 수와 전후 source SHA를 모두 보존한다.

자체 검증 후 before/after archive·추가/수정/삭제 목록·정/역 patch·필수 지시/검토와 실행 증거를
새 target에 동결한다. currentness와 격리된 양방향 byte 복원을 검증하고 `REVIEW_HANDOFF.ko.md`에
A01–A10 대응 및 **D07-B/C·WP03/04 잔여**를 적는다.

**D07-A 부분 구현·자체 검증 종료 / 단계검토 target 동결 / 독립 검토 요청 대기**로 응답하고 중단한다.
이번 결과를 D07 전체, V17 actual-frame 동등성, WP02/R06 완료 또는 production publication 지원으로 표시하지 않는다.
