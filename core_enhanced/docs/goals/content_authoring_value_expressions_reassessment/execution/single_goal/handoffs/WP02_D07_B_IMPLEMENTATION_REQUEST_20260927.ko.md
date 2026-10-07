# R06 WP02 D07-B — 실제 평가 문맥·수치·계산 과정 관측 구현 지시서

2026-09-27 · 계획/독립 검토는 원 대화, 제품 구현은 별도 대화.
**사용자 결정: 전투별 최근 평가 1건만 보관. 교체된 과거 건은 ‘보관되지 않음’으로 명시한다.**
이 문서는 다음 구현 요청이다. D07-A 동결 문서는 수정하지 않는다.

## 1. 구현 담당자에게 전달할 요청

> 이 문서의 D07-B를 구현해줘. 실제 effect 소비에서 사용한 정확한 입력 문맥과 공통 evaluator 결과를
> 전투별 최근 1건의 불변 기록으로 보관하고, trial의 수치 preview/explain이 그 기록을 읽게 해줘.
> 현재 전투 상태로 과거 입력을 재구성하거나 ActionHandle·장비 권한을 조회용으로 만들지 마.
> 아래 구현 순서·파일 경계·B01–B20 기준을 지켜줘. 기존 legacy gameplay와 공간 preview,
> D07-A 정적 explain, authored expression production publication 차단을 유지해줘.
> 구현 전에 현재 source와 실제 before를 보존하고, 최종 자체 검증 뒤 새 단계검토 target을 동결해줘.
> 인계서·manifest SHA를 보고한 뒤 중단해줘. WP03/04나 전체 완료 판정으로 이어가지 마.

## 2. 시작점과 범위

- [D07-A 독립 PASS](../reviews/wp02_d07_static_explain_001_review_001/REVIEW.md)
- [시작 manifest](../segments/wp02_d07_static_explain_001/target/d07_static_explain_001/MANIFEST.json)
- [이번 실제 코드 대조](WP02_D07_B_CODE_EVIDENCE_20260927.json)
- [D07 원 설계](../../../CODE_REVIEW_003.ko.md#d07--previewexplaingeneration저장cache)
- [D07 A/B/C 단계 계획](WP02_D07_IMPLEMENTATION_REQUEST_20260927.ko.md)

| 대상 | 기준 |
| --- | --- |
| manifest SHA-256 | `ae6609c1e75b711a27cacf8467ac0c42eaa99635a0fc19c5ae486ecd949bb1ac` |
| source 338개 after digest | `5279b8333c6be8f90992ac94a47f09beb083bf82bfcdf02ebbe3e0d179c8f530` |
| 제품 334개 digest | `17e41fa7625d001242fcfcec65834ab533fea43ea74e3f89f7ce78f3bae5acfd` |

작성 시 currentness와 격리 정·역 복원을 확인했다. 구현 시작 때도
`segments/wp02_d07_static_explain_001/freeze.py verify`를 실행한다.
불일치를 reset/clean/stash로 제거하지 않는다. 새 segment는 `segments/wp02_d07_runtime_amount_001/`,
target은 `d07_runtime_amount_001`을 권장한다. 기존 디렉터리가 있으면 먼저 확인한다.

이번 범위는 **이미 실제로 평가된 occurrence의 base amount와 계산 과정 조회**다.
미래 시전 예측, 전투 임의 시점 정지, 전체 실행 이력, 임의 시점 재생, 편집기 UI는 만들지 않는다.
‘preview’의 근거는 `recorded_evaluation`이라고 표시한다. ‘지금 새로 시전하면 입힐 피해’라고 표시하지 않는다.
Snapshot/Live 둘 다 실제 소비가 선택한 문맥을 따른다. 입력이 없거나 기록되지 않은 경우 수치를 발명하지 않는다.
D07-C/WP03의 effective stat·CertifiedInputFingerprint·save와 WP04 public authored 실행 인수는 남긴다.

## 3. 코드에서 확인한 연결점

아래 경로는 `core_enhanced/` 기준이다. 줄 번호보다 함수명을 기준으로 수정한다.

| 위치 | 현재 사실 / 구현 방향 |
| --- | --- |
| `battle/core/value_expr_runtime.rs::evaluate_value_expr_capture_with_resolved` | generation/site/fingerprint/live action/delivery/target 검사 후 Snapshot의 release frame 또는 Live의 현재 frame을 선택한다. 이 검사와 frame 선택 순서를 유지한다. |
| 같은 파일 `RuntimeAmountEvaluation` | amount, EvaluatedAmount(value/trace), ExprUsage를 반환한다. 선택된 frame은 반환하지 않는다. 관측 활성 시 정확히 사용한 frame을 소유권 이동/Arc 공유로 함께 넘길 수 있게 한다. |
| `battle/core/sim.rs::execute_ordered_skill_effects` | expression의 대상별 평가에서 현재 `values.push(value.amount)`만 남긴다. 바로 이 평가 결과가 기록의 생산자다. effect 완료 뒤 current state로 다시 계산하지 않는다. |
| 같은 파일 `build_ordered_skill_effect_commands` | legacy Damage는 unsigned_abs, Heal은 i32 literal을 사용한다. expression은 evaluated_amounts를 소비한다. 관측이 켜진 legacy의 공통 평가값을 연결할 때 이 실제 소비 분기도 함께 수정해야 한다. |
| `battle/core/authoring_observation.rs::MandatoryObservation` | 기존 latest witness와 ObservationKey를 소유한다. 새 숫자 기록을 별도의 제한된 필드로 둔다. HP application witness와 동일한 사실로 합치지 않는다. |
| `world/authoring_observation.rs::TerminalCapsule` | 마지막 종료 전투의 immutable 관측을 보관한다. 새 기록도 같은 active→terminal 수명에 연결한다. |
| `authoring/trial.rs::inspect` | Inspection scope와 native provenance를 설정하며 query의 work를 accepted trial quota와 분리한다. 새 직접 조회도 이 경계를 사용한다. |
| `TrialSession::preview` / `preview.rs` | 현재 공간 preview는 application candidate를 사용한다. 기존 enum/공간 의미는 유지하고 숫자 조회를 별도 닫힌 메서드로 추가한다. |
| `authoring/scenario.rs::TrialStep` / `trial.rs::prepare_step` | 기존 step의 public_result는 serde_json::Value다. 큰 i128 trace를 이 경로에 새로 넣지 않는다. CLI는 기존 scenario를 실행한 뒤 숫자 API 문서를 별도로 출력한다. |
| `battle/core/target_usefulness.rs` | DamageExpr는 적 eligibility만 허용한다. legacy final-damage fold는 DamageExpr를 건너뛴다. 미정 수치를 0으로 취급하는 소비자를 추가하지 않는다. |
| `value_expr/mod.rs::EvalFrame` / `EvaluatedAmount` | frame의 값과 resolved view는 private Vec/Arc 구조이고 trace는 필수 계산 결과다. optional authoring NDJSON trace와 다른 데이터다. |

## 4. 관측 설정·수명·공개 계약

### 4.1 설정과 보관 정책

신뢰된 host 설정 `TrialLimits`에 숫자 관측 opt-in을 추가한다. 기본은 꺼짐으로 두어 기존 trial과
일반 gameplay가 새 frame/trace 보관 비용을 자동 부담하지 않게 한다. 설정은 콘텐츠 RON의 실행 권한이 아니다.
켜진 세션에서는 optional NDJSON trace가 꺼져 있어도 숫자 기록과 재평가가 동작해야 한다.
설정은 세션 생성 전에 결정하고, 중간 on/off 전환은 이번에 추가하지 않는다.

- active 전투당 최근 성공 평가 1건. 다중 대상은 실제 executor의 기존 대상 순서를 따른 마지막 평가를 보관한다.
- 새 기록은 기존 1건을 교체한다. per-skill map, per-target map, ring buffer, append-only Vec는 만들지 않는다.
- 종료 시 기존 last-terminal capsule로 넘긴다. 보관 owner는 기존처럼 active와 마지막 종료 전투까지다.
  ‘전투별 1건’은 모든 과거 전투를 무한 보관한다는 뜻이 아니다.
- action/delivery가 정리돼도 보관된 immutable 관측은 조회할 수 있다. 이것은 과거 사실의 조회이며 실행 권한 재사용이 아니다.
- checkpoint에 관측 기록이나 frame을 넣지 않는다. 새 저장 schema/버전 또는 검증 우회도 추가하지 않는다.
- 대상이 없는 D04-F01 경로는 평가하지 않는다. 앞선 기록이 남아 있더라도 새 무대상 occurrence의 결과로 반환하지 않는다.

### 4.2 기록의 내용

이름은 저장소 관례에 맞춰도 되지만 아래 관계는 고정한다.
`RecordedAmountEvaluation`은 private 생성자와 immutable `Arc` owner를 사용한다.

| 내용 | 제약 |
| --- | --- |
| 식별 | content generation, BattleKey(entry ordinal/battle UUID), 전투 내 checked 단조 증가 evaluation ordinal |
| 출처 | 실제 EffectSite, compiler/plan fingerprint, kind/provenance/moment, 실제 RuntimeFactSource의 actor·장비 사본·action·delivery |
| 대상/시간 | release target와 consumption target을 별도 보존, release/consume time, effect index. timestamp만을 유일 식별자로 쓰지 않음 |
| 계산 입력 | runtime이 실제 사용한 immutable plan과 frozen EvalFrame. Snapshot은 release frame, Live는 해당 effect의 mutation 직전 선택된 frame |
| 계산 결과 | 동일 실행의 typed value, narrowed RuntimeAmount, 필수 evaluator trace, runtime ExprUsage |
| 계량 | frame·resolved view·trace·source 등 record가 붙잡는 retained extent와 사전 admission에 사용할 checked 크기 정보 |

원본 snapshot Arc는 공유하고 Live frame은 가능하면 이동해 Arc로 감싼다. 새 현재 frame을 다시 만들거나
Snapshot 전체를 deep clone하지 않는다. record의 plan/frame/RuntimeFactSource/ActionHandle을 public API로 반환하지 않는다.
공개 DTO에는 읽기용 identity와 값만 투영한다. ordinal은 조회 선택자이지 실행 capability가 아니다.

### 4.3 조회 API

`authoring/amount_query.rs` 같은 모듈에 닫힌 요청/응답을 둔다.
`TrialSession::preview_amount`와 `TrialSession::explain_amount`는 같은 조회·재평가 helper를 사용한다.
preview도 비교 가능한 typed amount와 evaluator trace를 제공하고 explain은 정적 plan 설명을 추가한다.
D07-A의 기존 `TrialSession::explain(ExplainQuery::Effect)`는 계속 정적 설명이다.

요청은 다음 둘로 한정한다.

1. `Latest { battle: Active | LastTerminal }`: 해당 owner의 최근 record를 발견하는 조회.
2. `Exact { battle_key, evaluation_ordinal }`: 앞서 받은 읽기용 identity와 현재 보관 record가 정확히 같은지 조회.

`Exact`에 action/resource/frame 입력이나 임의 효과 실행 요청을 받지 않는다. 요청 전체는 작은 Copy 필드로 유지한다.
응답은 태그가 있는 상태로 구별한다.

- `evaluated`: `basis=recorded_evaluation`, identity/출처/두 시간·대상, moment, typed base amount, narrowed 값, trace.
- `unavailable`: `observation_disabled`, `no_battle`, `not_yet_evaluated`, `not_retained` 등 명시적 이유. amount/trace를 0/빈 성공값으로 채우지 않는다.
- 잘못된 요청/손상된 identity와 query work·metadata·result·취소 오류는 typed 오류다. 정상적인 기록 부재와 예산 실패를 합치지 않는다.

미래 수치 판단이 필요한 호출자는 `Indeterminate(requirements)`를 사용한다. 기록 부재를 ‘피해량 0’이나
‘시전 불가’로 번역하지 않는다. base amount는 방어/저항/난수/실제 HP 변화 이전의 값이다.
실제 피해와의 연결은 기존 runtime facts/HP witness를 사용하되 일대일 correspondence가 없는 값을 억지로 붙이지 않는다.

## 5. 구현 순서와 주의점

### 5.1 공통 numerical helper 분리

기존 runtime adapter의 **identity 검증 및 frame 선택 이후** `ExprMeter → evaluate → narrow` 부분을
crate 내부 helper로 추출한다. runtime, recorded preview, recorded explain이 이 helper를 사용한다.
숫자 계산식·rounding·집계·narrowing을 query에 복제하지 않는다.

runtime 입구는 기존 live action/delivery 검사를 모두 유지한다. recorded query는 이 입구를 재호출하지 않는다.
정리된 action 때문에 과거 관측을 거부하지 않도록, 신뢰된 record의 generation/battle/ordinal/plan 결속을 검사한 뒤
그 record가 소유한 exact frame으로 **pure numerical helper만** 호출한다.
임의 외부 EvalFrame을 받는 public 생성자나 토큰 위조는 금지한다.
query용 fresh meter와 host admission을 사용하고 runtime usage와 query usage는 별도로 표시한다.

runtime이 기록한 value/narrowed amount/trace와 조회 결과가 다르면 정상 응답 대신 invariant typed 오류다.
그 비교 자체의 trace/string 순회도 계량한다. 조회 시 optional trace 설정, 현재 HP/stat/resource,
현재 target set, 이미 종료된 action의 새 source를 참조하지 않는다.

### 5.2 실제 소비와 record commit

`RuntimeAmountEvaluation`의 관측용 payload는 설정이 켜진 경우에만 만든다.
대상별 평가에서 실제 소비할 amount를 기존 Vec에 넣고, 임시 최신 record 후보는 1건만 유지한다.
앞선 후보를 교체하며 모든 대상의 frame/trace를 별도 Vec에 누적하지 않는다.
필수 계산 trace는 기존 evaluator가 생성한 결과를 이동한다. 관측용 두 번째 runtime 평가를 하지 않는다.

effect command 처리·projection settle·action prefix settle이 성공한 뒤에만 observer의 최신 record를 교체한다.
다음 effect/같은 timestamp/receipt 단계가 실패하면 기존 application/timestamp candidate rollback에 따라
원 accepted record와 ordinal도 복구돼야 한다. Arc<Mutex<...>>, 전역 collector, thread-local 기록 저장소처럼
candidate 밖으로 결과가 새는 구조를 만들지 않는다.
숫자 0이라 실제 Damage command가 생기지 않더라도 실제 평가가 있었다는 사실은 기록 가능하다.
이 경우도 ‘피해 발생’이라고 표시하지 않는다. ordinal overflow는 typed 거부한다.

### 5.3 legacy 공개 경로를 실제로 검증할 수 있게 연결

현재 production authored expression은 차단되어 있다. 이를 풀어서 공개 시험을 통과시키지 않는다.
대신 숫자 관측이 켜진 authoring 전투에서 **실제 catalog를 가진 legacy Damage/Heal**도 공통 평가의 소비 경로에 연결한다.

- legacy plan은 기존 compiler가 만든 context-free constant다. provenance와 공통 context 분류를 먼저 확인한 뒤
  입력 reference가 없는 정확한 빈 frame을 사용할 수 있다. 이것은 동적 식에 default frame을 대입하는 fallback이 아니다.
- 실제 executor가 가진 generation/site/source/actor/장비/action/delivery/대상 정보를 붙인다.
  legacy 경로에 없던 action handle이나 release capture를 숫자 조회 때문에 만들지 않는다. 원래 없는 출처 슬롯은 null이다.
- `build_ordered_skill_effect_commands`는 관측 경로에서 평가된 amount를 실제 command에 사용해야 한다.
  계산만 관측하고 명령은 다른 literal 값을 소비하는 두 권위를 만들지 않는다. 필요한 variant·길이·target 순서를 검증한다.
- 기존 legacy 음수 Damage의 unsigned_abs, Heal i32, 0 처리와 hit 순번·source snapshot·명령 순서를 보존한다.
- 관측 꺼짐/비authoring 경로의 literal 실행은 유지한다. catalog 없는 programmatic effect는 기존 실행을 유지하며
  임의 plan을 compile하지 않는다. 이번 compiled amount 관측 대상이 아니므로 기록을 만들지 않는다.

이 연결은 임의 공격·buff tick 전체의 관측 체계가 아니다. 이 executor의 compiled skill amount에 한정한다.
authored 식의 동적 경로는 기존 test-only staged fixture로 검증하고 공개 지원과 구분해 인계한다.

### 5.4 owner·clone·terminal·초기화

숫자 기록 owner를 `MandatoryObservation`에 연결하되 자동 serde로 private frame/plan/trace 전체가
기존 매 step receipt에 복제되지 않게 한다. 기존 일반 observation에는 작은 availability/key summary만 추가하거나
별도 숫자 조회 포트로만 노출한다. 설정 꺼짐의 새 optional 필드는 생략해 기존 출력 회귀를 최소화한다.

`BattleCore`의 exhaustive CloneShape와 timestamp scratch clone/reuse, observer reset,
`world::TerminalCapsule`, `CoreApplication` forwarding, `TrialSession` 생성·교체를 함께 점검한다.
특히 `sim.rs`의 observer 재초기화가 새 숫자 설정을 꺼버리거나 ordinal/key를 잘못 재사용하지 않게 한다.
`events/combat.rs`의 새 전투 생성은 world의 host 설정을 전달해야 한다.
전투 종료 시 action/queue 정리 후에도 immutable record를 retained-accounted terminal owner가 소유해야 한다.

### 5.5 한도와 취소

기존 ExprLimits, authoring work/metadata/result/native retained/clone 한도를 재사용한다. 한도를 올려 통과시키지 않는다.
상수 크기 key, trace Vec capacity와 문자열/단위 payload, EvalFrame의 values 및 모든 resolved aggregate view,
source/identity, Arc 소유 객체와 header를 실제 구조에 맞게 checked 계산한다.
`frame_storage_bound`는 scalar reference bound이므로 그 값만으로 전체 resolved members를 계량했다고 하지 않는다.

현재 native retained 합산은 record를 모른다. `retained_runtime_fact_extent` 및 terminal/clone preflight에서
새 소유권을 반영한다. snapshot capture가 사라져 record만 Arc를 소유하게 되어도 계량이 빠지면 안 된다.
공유 Arc를 중복 소유로 보수적으로 계산할 수 있지만 ‘Arc 복사 비용만’으로 실제 보존 payload를 누락하지 않는다.
교체 중 old record + new candidate + evaluator scratch가 공존하는 peak도 allocation 전에 admission한다.
계량만 위해 대상/장비/전체 catalog를 새로 순회하지 말고 생산 시 계산한 extent를 재사용한다.

큰 trace/frame의 Serialize/비교는 bounded writer와 work/cancellation 경계에서 수행한다.
`serde_json::to_value/to_string`으로 큰 private payload를 먼저 만들고 나중에 검사하지 않는다.
trace 저장 한도 초과를 잘라 성공시키거나 관측 꺼짐으로 몰래 전환하지 않는다.
켜진 trial에서는 typed 실패와 기존 accepted session 보존으로 처리한다. 스킬 자동 취소 기능은 추가하지 않는다.

### 5.6 usefulness의 미정 표현

기존 hostile eligibility와 수치 판단을 분리한 작은 내부 enum/구조를 만든다.
DamageExpr의 실제 occurrence frame이 없으면 `Indeterminate(requirements)`이며 기존 eligible=true 경로를 유지한다.
mixed legacy+expression step을 legacy partial sum만으로 Known(final_damage)이라고 표시하지 않는다.
관측된 과거 amount를 다음 시전의 정량 usefulness에 재사용하지 않는다.
현재 legacy 방어·저항 기반 usefulness와 basic attack의 기존 minimum damage 의미는 유지한다.
한도/취소 오류를 수치 0 또는 Indeterminate로 숨기지 않고 기존 control failure 전파를 보존한다.

### 5.7 API/CLI의 닫힌 연결

`CoreApplication → GameCore → active/last-terminal observer`로 private readonly 조회를 전달한다.
직접 `TrialSession` query는 Inspection scope의 독립 한도에서 실행하며 accepted_work/receipt/index를 바꾸지 않는다.

CLI는 기존 finite scenario를 실행한 뒤 두 숫자 API로 active와 last-terminal을 각각 조회한다.
최대 4개(preview/explain × active/last-terminal) bounded JSON 문서를 기존 receipt 뒤에 별도 NDJSON 행으로 출력한다.
각 문서에 `kind=amount_preview|amount_explanation`과 요청 selector를 넣어 기존 receipt와 구별한다.
기록이 없으면 동일한 unavailable 문서를 출력한다. CLI 편의를 위해 그 상태를 생략하거나 성공 수치로 바꾸지 않는다.
각 조회는 CLI가 정한 기존 ExecutionLimits에서 수행하며, 실패는 기존 typed CLI 오류/비정상 종료로 표시한다.
이 조회는 scenario 실행이나 accepted quota를 더 진행하지 않는다.

기존 `TrialStep`/public_result/JSON pointer/Assert 문법은 확장하지 않는다.
특히 현재 `public_value`의 serde_json::Value를 새 숫자 문서가 경유하면 i128 trace의 정밀도를 잃을 수 있다.
새 응답은 typed struct에서 `output::encode_result`로 직렬화하고 CLI의 기존 `line`으로 document 원문을 쓴다.
중간 `to_value`/`from_str::<Value>`/f64 변환을 쓰지 않는다. 큰 정수 시험도 원문 또는 typed i128 deserializer로 검증한다.

CLI `simulate`에 명시적 `--amount-observation` host 옵션을 추가한다.
현재 인자 parser는 최대 6개와 고정 trace 위치를 가정하므로 bounded 옵션 파싱으로 고친다.
`--trace-output`과 독립적으로 함께/각각 작동하고 중복·모르는 옵션은 거부한다.
`validate/describe/explain`에서 이 실행 설정을 받지 않는다. 기존 정적 explain 명령은 유지한다.

## 6. 필수 검증 B01–B20

| ID | 확인할 실제 행동 |
| --- | --- |
| B01 | actual executor의 legacy Damage/Heal amount와 공통 evaluator 결과 일치. 음수 Damage/i32 범위 Heal/0 및 관측 on/off gameplay 동일. 실제 소비값을 검증하고 DTO끼리만 비교하지 않음 |
| B02 | staged Snapshot: release 뒤 Source.Attack/HP/resource/result/대상 상태가 변해도 runtime과 두 숫자 query는 release 입력을 사용 |
| B03 | staged Live: 앞 effect가 바꾼 결과·HP/resource를 해당 effect가 읽음. 이후 변경·query 반복으로 그 기록의 결과가 바뀌지 않음 |
| B04 | 동일 frozen frame의 runtime/preview/explain typed value, narrowed amount, node/member trace의 순서·내용 일치 |
| B05 | 실제 Side/Employee/EquippedSlot·명시적 필터·복수 aggregate node의 독립 view와 중첩 member trace 유지 |
| B06 | 동일 timestamp의 복수 effect·여러 target·piercing delivery에서 실제 순서에 따른 최근 1건/ordinal. 오래된 Exact는 not_retained이며 최신 기록으로 대체하지 않음 |
| B07 | 다른 generation/battle/ordinal과 손상된 site/fingerprint/source-copy 결속 거부. public 입력으로 frame/action/resource capability를 만들 수 없음 |
| B08 | action/delivery settlement·unit 제거 뒤 retained 기록은 exact replay 가능. active→terminal→다음 전투와 last-terminal 교체 경계 확인 |
| B09 | disabled/no battle/no evaluation/not retained를 수치 0과 구별. D04-F01 무대상 impact는 새 평가/ordinal/가짜 결과 없이 후속 effect·step·delivery 완료 유지 |
| B10 | 앞 target 성공 후 다음 target 실패, 앞 effect 성공 후 다음 effect 실패, timestamp/receipt 실패·취소에서 accepted observer/key/ordinal과 gameplay가 함께 복원 |
| B11 | record/교체 peak/native retained/clone/work/result 한도 정확 경계 및 1 미만 typed 거부. 긴 trace/member payload의 중간 취소. 누락 extent나 unlimited fallback 없음 |
| B12 | 관측 켜짐에서 optional NDJSON trace off/on/포화에 관계없이 필수 amount/trace가 같음. 일반 event log off/eviction에도 기록 정확 |
| B13 | controlled clone/timestamp scratch는 immutable Arc 공유. candidate 밖 mutable alias 없음. 기록 수가 action/전투 시간/대상 수와 함께 계속 증가하지 않음 |
| B14 | 직접 TrialSession query 전후 snapshot/RNG/queue/resources/content identity/accepted work/index/receipt 불변. 동일 요청 문서와 query usage 결정적 |
| B15 | 실제 외부 legacy publication의 TrialSession 두 API 및 CLI simulate 후 별도 숫자 문서 출력 성공. 최대 4건·kind/selector·기록 부재 상태 확인. 새 API만 별도 가짜 harness로 검증하지 않음 |
| B16 | public DamageExpr/HealExpr 거부 유지. 동적 preview 증거는 staged 내부 actual executor임을 구분. D07-A 정적 performed=false 유지 |
| B17 | usefulness의 DamageExpr/mixed step은 미래 수치를 Indeterminate로 표현하면서 기존 eligibility 유지. legacy/basic/공간 range·rotation 결과 회귀 없음 |
| B18 | i128 범위 trace의 API/CLI 직렬화 정밀도 유지. optional 필드/strict parse/CLI 옵션 조합·중복/과도한 입력 거부 |
| B19 | 기존 전체 제품 시험, PC36 비교, R05 비용 guard. native ownership 변경이므로 D07-A의 비용 재측정 생략을 그대로 복사하지 않음 |
| B20 | 338개 before와 추가 파일의 부재, after 확장 집합, currentness, 정·역 patch 격리 byte 복원 및 전 실행 로그/실패 기록 결속 |

WP03 미구현 stat을 임시 값으로 보충해 B02/B03을 통과시키지 않는다. 현재 지원되는 Attack/HP/resource/result로 검증한다.
별도 V17 증거에는 actual runtime consumer가 사용한 frame/amount와 query 출력을 연결한다.
`evaluate`를 같은 수동 frame에 두 번 호출한 시험만으로 B02–B04/V17 전체를 완료 처리하지 않는다.

## 7. 실행·인계·종료

최종 source에서 변경 영역 format과 아래 명령을 수행한다. 전체 시험에 `--test-threads=1`을 붙이지 않는다.
로그는 시작 즉시 파일에 기록하고 주기적 진행 표시, 중단 상태/자식 종료를 보존한다.
실패/재시도마다 새 이름을 사용하며 종료하지 않은 실행을 성공 영수증으로 만들지 않는다.

```bash
cargo check -p game_core --all-targets --offline --locked
cargo test -p game_core --lib authoring --offline --locked
cargo test -p game_core --lib value_expr --offline --locked
cargo test -p game_core --all-targets --offline --locked
cargo clippy -p game_core --all-targets --offline --locked -- -A clippy::never_loop -A clippy::eq_op
```

추가 B01–B20 selector는 실제 시험 이름·개수로 인계한다. 0건 selector는 통과 증거가 아니다.
CLI fixture는 현재 contract로 만들고 실제 명령·입력 파일을 동결한다. 20분 시험은 수행하지 않는다.
PC36/R05 harness는 직전 교정 구간의 원자료·실행 도구를 먼저 읽고 현재 source에 결속해 재사용한다.
제품 target과 계측 hook target을 구분하고, 관측 꺼짐의 기존 조건과 새 on 조건을 따로 기록한다.
기존 약 9MB concurrent peak 초과만의 경고 정책을 유지하며 다른 hard failure를 경고로 바꾸지 않는다.

최종 인계에는 다음을 포함한다.

- B01–B20 대응 및 public legacy / staged dynamic / 미지원 미래 예측의 구분.
- record의 수명·교체·terminal·rollback과 실제 allocation/retained/clone 계량 근거.
- Snapshot과 Live 각 1개 이상의 실제 runtime 소비 → 보관 입력 → 두 query amount/trace 대조 원자료.
- 기존 PC36 결과와 관측 on/off의 gameplay·source·hit/order·RNG 비교, 비용 변화 및 모든 실패/재시도.
- before/after archive, 신규 파일 부재↔존재를 포함한 정·역 patch, currentness와 격리 복원, manifest SHA.

동결 후 **D07-B 부분 구현·자체 검증 종료 / 단계검토 target 동결 / 독립 검토 요청 대기**에서 중단한다.
D07 전체/WP02/R06 완료, production authored publication 허용, 독립 검토 PASS를 구현자가 선언하지 않는다.
