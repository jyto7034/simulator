# R06 재검토 — 코드 근거와 구현 설계 인계

Owner: `CRA-CAVER-P2G20-002`. Input `WR-CAVER-P2G10-001`. 아래 구조·타입·문법은 **구현 제안**이며 현재 지원 API가 아니다. 승인 정책은 P2G_10_POLICY_001.json의 9 Decision/27 acceptance가 소유한다. source identity·보존 원본·선행 비교는 P2G_20_SOURCE_EVIDENCE_001.json이 소유한다. 정적 조사이며 게임 실행 PASS가 아니다.

## C01 — 기존 결론 대조와 채택

기존 자료의 exact revision은 `../content_authoring_value_expressions/P2G_10_POLICY_001.json`의 `DR-CAVE-E01..09-001`, `PER-CAVE-E01..09-DECIDED-001` 및 `../content_authoring_value_expressions/CODE_REVIEW_001.ko.md`의 `CRA-CAVE-P2G20-001` C01–C09다. 이 파일들의 실제 bytes/hash는 baseline/intake/MANIFEST.json에서 원위치와 사본을 함께 찾는다. 기존 P2G_20_REVIEW_001.json에는 후보와 reviewed payload가 있으나 complete batch·result binding은 없으며 이를 새 current로 사용하지 않는다.

| 기존 위치/결론 | 새 확인 사실 | 판단·이유 | 영향 요구/소비자/시험 | 최종 채택 owner |
| --- | --- | --- | --- | --- |
| old C01/TD01 literal과 식을 같은 executor로 | ability.rs의 i32 Damage는 unsigned_abs, Heal은 signed flat; authoring/literal_amount·edit validation도 소비 | 유지·보완: 기존 음수 literal까지 같은 결과를 내는 lowering 명시 | E01/E08/E09; compiler/sim/usefulness; V01–V04/V18 | D01/D02 |
| old C02/TD02 Snapshot target 미확정이면 거부 | source snapshot은 공격자 중심, area/projectile의 실제 targets는 impact/tick에 생성 | 유지·보완: 루트 moment·capability matrix·capture carrier·시간 기준 분리 | E02/E05; release/impact/cancel; V05/V06/V12 | D03 |
| old C03/TD03 tile producer는 후속 owner라 일괄 제외 | UnitBody::projected_tile, TileRangePattern와 실제 target set이 이미 존재; CA05는 타일 단위 집계를 명시 | 교체: 기존 finite resolved targets를 tile별 명시적 fold; 전역 tile/network 탐색은 제외 | E03; group/source-copy; V07/V08 | D04 |
| old C04/TD04 UUID 누락·단일 stat authority | BattleEquipmentEnhancement는 base_uuid/level뿐; 각 binding마다 동일 definition 모든 level 합산. combat/Certified 두 producer 모두 copy UUID 보유 | 유지·보완: actual-copy join 검증, numeric weapon 한 곳, modifier 순서 보존 | E04/E06/E07; inventory/draft/stats; V09–V11 | D05 |
| old C05/TD05 query_action로 settled 결과 읽기 | query는 active ActionSnapshot도 반환, take만 NotSettled 검사·retained 결과 제거. command builder는 모든 effect를 먼저 구성 | 교체: exact handle + 완료된 effect prefix의 immutable witness; read-only query·순차 effect 실행·future 검사 필요 | E05/E02; fact/commands/transaction; V12–V14 | D06 |
| old C06/TD06 point→interval 재사용 | scale_attack_interval_ms는 u128 ceil, 0 assert, 큰 결과 expect; 범위 검사 없이 새 입력 연결 불가 | 유지·보완: 호출 전 양수/결과 범위 검사, fragment interval 조작 유지 | E06; weapon/basic cadence; V11 | D05 |
| old C07/TD07 serde default·save 유지 | V7은 deserialize 후 재직렬화한 checkpoint에 SHA 검증. 새 zero field의 기본 직렬화는 기존 checksum/semantic fingerprint를 바꿀 수 있음 | 보완: default + zero serialization omission, 실제 이전 bytes fixture; 버전/identity bypass 금지 | E07/E09; V7/content generation; V15/V16 | D07 |
| old C07 preview/explain 같은 계산 | ExplainQuery::Effect에는 runtime context가 없음; target usefulness는 별도 literal fold; range preview는 공간 DTO | 보완: static explain은 의존 입력/미평가 이유, actual trial에서 동일 frame 평가. 실제 난수 포함 final 피해와 static amount를 구분 | E07; CLI/query/preview; V17 | D07 |
| old C08/TD08 typed fault를 transaction까지 | sim/area/projectile의 let Ok else return, with_instant_resource_delivery의 void callback/close 오류 무시 존재 | 유지·보완: 모든 호출 경로 Result 전파; native/controlled candidate, queue/RNG/UUID/log/resource 함께 rollback | E08; runtime/source/cancel; V14/V19/V20 | D08 |
| old C09/implementation shape: 네 Phase | compiler/runtime/stat/공개 capability 사이 중간 공개 상태 근거 부족; AT06는 숫자/projection/save 동시 전환 | 교체 제안: 한 Phase, 한 atomic cutover 안의 WP 구현 순서. Phase 경계는 P2G-40 최종 owner | 모든 E; public support·최종 수용·비용 | D09 및 P2G-40 |

기존 정책 9개를 유지하는 이유는 서로 다른 변경 사유와 acceptance owner가 있기 때문이다. 구현 node 수를 9로 맞추지 않는다. 새 source finding은 정책 의미를 바꾸지 않는 `technical_decision`; R07+ 신규 기능·공식 weight는 `out_of_scope`. required source evidence gap은 없으며 실행 전 검증/측정 미수행은 D10의 명시적 준비 조건이다.

## D01 — 작성 정의·컴파일 정의·오류

변경 책임: `game/ability.rs::SkillEffectDef`에 명시적 DamageExpr/HealExpr 작성형 추가; `game/value_expr/{mod,compile,eval}.rs` 신규 유한 식 공통 모듈. `game/ability/authoring.rs::{AuthoredSkillEffect,LiteralEffectOp,EffectSupport}`, `game/data/skill_data.rs`의 실제 edit/reference validation과 source binding을 함께 전환한다. 콘텐츠 이름 전용 분기, 외부 eval, 일반 control-flow VM은 만들지 않는다.

권장 계약:

```rust
// Proposal only; names can be refined without changing these meanings.
struct AmountExpr { moment: EvalMoment, root: ValueExpr }
enum EvalMoment { Snapshot, Live }
enum ValueExpr {
  Const(TypedInteger), Read(ValueRef), Add(Vec<ValueExpr>),
  Mul { value: Box<ValueExpr>, scalar: Box<ValueExpr> },
  Ratio { numerator: Box<ValueExpr>, denominator: Box<ValueExpr>, rounding: Rounding },
  Clamp { value: Box<ValueExpr>, min: Box<ValueExpr>, max: Box<ValueExpr> },
  Aggregate { source: FiniteSource, grouping: Grouping, reduction: Reduction, member: Box<ValueExpr> }
}
struct CompiledAmountPlan {
  ops: Box<[Op]>, result_unit: ValueUnit, moment: EvalMoment,
  requirements: ContextRequirements, budget: PlanCost, source: EffectSource,
  semantic_fingerprint: [u8; 32],
}
fn compile(expr: &AmountExpr, site: &EffectSource, context: DeliveryCapabilities,
           limits: &ExprLimits) -> Result<CompiledAmountPlan, ExprDiagnostic>;
fn evaluate(plan: &CompiledAmountPlan, frame: &EvalFrame, meter: &mut ExprMeter)
  -> Result<EvaluatedAmount, ValueExprFault>;
```

의미 단위는 `Magnitude`(효과량/HP/Attack/SpellPower의 정수 크기), `Scalar`, `Count`, `Milliseconds`, `AttackSpeedPoints`, `Resource(ResourceId)`를 구별한다. Attack/SpellPower는 같은 크기 단위를 사용해도 **서로 다른 Stat selector**이며 피해 DamageType과 독립이다. resource A/B, ms/point, Count/Scalar는 implicit cast하지 않는다. HP ratio는 HP/HP, resource ratio는 동일 resource/resource로 Scalar를 만든 뒤 명시적으로 Magnitude와 곱한다. 이 테이블을 compiler·describe·unit test가 같이 소비한다.

정수 저장은 signed i128 checked 값(입력은 기존 u32/i32/u64에서 무손실 확장). 각 Add/Mul와 narrowing에 checked 연산; float·saturating은 expression 계산에 사용하지 않는다. Add/Clamp는 동일 unit, Mul은 한쪽 Scalar, Ratio는 같은 unit→Scalar 또는 unit/Scalar→동일 unit만 허용. Ratio는 항상 `TowardZero|Floor|Ceil|NearestTiesEven`을 명시하고 해당 노드에서 정수 반올림한다. 예: 5/2는 2/2/3/2, -5/2는 -2/-3/-2/-2; 7/2 nearest-even=4. 분모0, i128 MIN/-1, 중간 overflow, clamp min>max는 typed 오류. 상수 전체 식은 compile 때, runtime 참조 의존 오류는 해당 candidate consumption 때 거부한다. 분수 보존이 필요한 식은 곱셈을 먼저 작성한다(100×3÷2=150; 100×round(3÷2)=200일 수 있음을 explain에 표시). 숨은 rounding이나 ratio literal의 float parsing은 없다.

DamageExpr 최종 Magnitude는 0..u32::MAX로 checked 변환, HealExpr는 i32 범위로 checked 변환하여 기존 heal command 의미를 보존한다. 기존 Damage literal -5는 기존 unsigned_abs=5로 Constant lowering, i32::MIN은 2147483648; 기존 Heal -5는 signed -5. 이 호환 입력 정규화는 expression overflow fallback과 다르며 provenance에 `legacy_literal`로 표시한다. 기존 0 amount skip/effect count·damage modifiers·crit/strike/HP-loss gate는 유지한다.

컴파일 순서: 외부 acquisition/parse의 기존 byte limit → AST bounded visitor(재귀 진입 전 depth/node 검사) → EffectSite·unit·reference/delivery capability·result order 검사 → iterative postorder lowering → 상수 fold·range/checked 검증 → cost/retained admission → immutable catalog → generated preview validation → publication. `game/data/mod.rs::GameDataBase::new/from_validated_live_catalogs`와 `content_pack/mod.rs::publish_snapshot`의 fallible `?`/source relocation 경로를 재사용한다. AST JSON/RON parse 자체의 depth 공격은 parser 입력 제한+bounded Deserialize seed/visitor로 막는다; compile 이후 depth 검사만으로 stack 안전을 주장하지 않는다.

Compile key는 `(generation, EffectSite)`; 공유 정의의 plan은 immutable이고 개인 사본 평가 결과를 담지 않는다. authored edit/override 후 최종 winning definition에서 컴파일하며 duplicate/missing EffectSite는 publication 실패다. fixture builder도 같은 검증·compiler를 거쳐야 한다. bytecode는 postorder stack 평가, 각 opcode에 source node index를 남긴다. arbitrary jump/loop/recursive eval 및 graph clone expansion은 없다.

Fault는 `{code, site, node_index, expected_unit, actual_unit, phase}`와 available action/copy context를 포함한다. closed code: UnitMismatch, MissingRounding, DivideByZero, Overflow, InvalidClamp, MissingContext, UnsupportedMoment, InvalidOwner, StaleLease, DifferentAttempt, NotSettled, FutureResult, Unavailable, BudgetExceeded, Cancelled. static은 source-bound LiveRonError/authoring diagnostic, runtime은 GameError의 typed 표현으로 전달한다. `expect`, 0/last-value 대체 또는 false useful로 error를 소거하지 않는다.

## D02 — 공유 소비자와 처리 순서

실제 entry는 external `content_pack::load`→validated generation→`GameCore`/`authoring::simulate`. 현재 `build_skill_step_commands`는 effect 전체 literal을 command list로 만든다. 새 result 의존 식에는 evaluate→execute→settled witness 순서가 필요하다. `sim.rs`에 공통 `execute_skill_step_effects` 책임을 두고 immediate/area/projectile가 같은 경로를 호출하도록 제안한다:

1. 실제 ActionSite/source-copy/action/delivery context를 유지한다. 기존 전체 step ModifyDamage fold와 target/delivery 선택 순서는 유지한다.
2. effect authored order로 `EffectSite` plan을 찾고 해당 effect 전체 targets의 읽기 frame을 고정한다. self/target/aggregate member는 별도 selector다.
3. 해당 effect의 모든 target 값을 먼저 계산·범위 검사한다. target 순서는 기존 delivery 순서를 보존하고 aggregate reduction만 stable identity 순서로 정렬한다.
4. 그 effect commands를 기존 `process_skill_commands`·damage/heal authority로 실행한다. 첫 target mutation이 같은 effect의 다음 target Live 입력을 바꾸지 않게 한다.
5. 실제 완료 facts에서 effect prefix witness를 닫고 다음 effect로 이동한다. step 결과/count/HP-loss/strike gate는 actual results로 집계한다.
6. 실패면 바깥 runtime command 또는 timestamp candidate 전체를 버린다. 이전에 commit된 timestamp prefix는 되돌리지 않는다.

expression amount에 defense/crit을 먼저 적용하지 않는다. amount만 공통 evaluator가 소유하고 실제 `damage_runtime`/`calculate_damage`가 mitigation·crit·strike를 소유한다. 기본 공격 자체는 stat authority를 계속 사용한다. 효과 추가로 basic attack에 literal calculator를 복제하지 않는다. effect edit(Amount/DamageType), descriptors, auto-cast useful-payload 분류는 새 variants를 누락 없이 다룬다.

## D03 — Snapshot/Live·carrier·수명

moment는 AmountExpr root에 정확히 하나로 고정해 ambiguous mixed refs를 사전 거부한다. Constant-only는 context를 요구하지 않는다. Snapshot은 release/delivery materialization에서 **그 식의 필요한 값과 member identity**를 캡처하며 impact 때 Live로 대체하지 않는다. Live는 actual consumption의 effect-start read frame이다. 시간 인자는 기존 committed_at_ms를 무조건 재사용하지 않고 `release_time_ms`와 `consume_time_ms`를 구별한다.

| delivery/ref | Snapshot | Live |
| --- | --- | --- |
| source stat/HP, battle elapsed, source resource | release 값 캡처 | 유효 actual source/lease에서 consume 값 |
| 고정한 단일 target의 HP/stat/resource | release에 실제 target identity를 확인해 캡처 | actual target consume 값 |
| 방향 projectile/미확정 area의 target, resolved-target aggregate | target set이 release에 확정되지 않으면 publication에서 UnsupportedMoment | impact/tick의 실제 유한 targets로 평가 |
| result | release 이전 닫힌 prefix witness만 가능; 선행 없으면 거부 | 현재 effect 이전의 닫힌 prefix witness |
| RemainingTime | 명시 deadline owner가 release에 있을 때 max(deadline−now,0) | 같은 deadline identity로 consume 때 계산 |

`EvalCapture`는 compiled plan의 generation·site·fingerprint, typed captured scalars, sorted aggregate member IDs 및 action/source-copy/delivery identity를 가진 immutable Arc다. DamageSourceSnapshot 또는 공통 released evaluation carrier에 붙여 projectile/area/event-queue가 공유한다. caster 퇴장 후에도 캡처 값은 바뀌지 않으며 새 resource spending capability를 만들지 않는다. Live가 사라진 owner/target을 요구하면 MissingContext/InvalidOwner로 원자적 거부한다. 이미 release된 finite delivery의 취소/owner 사망 시 존속 여부는 기존 R04/R05 규칙을 그대로 따른다. 강제 despawn 정책을 새로 만들지 않는다.

entry/root/finite child가 기존 capability를 획득한 뒤 capture, queue publication 전 budget을 확인한다. root 종료는 existing finite hold를 통해 delivery를 마무리하며 완료/cancel/failure에서 Arc를 drop한다. 새 retained extent/CloneShape·native preflight는 queue/projectile/area/cast 포함 모든 carrier에 적용한다. clone마다 plan/캡처 깊은 복사를 하지 않는다. action/result/lease/capture는 저장하지 않는다. restore는 새 attempt identity를 발급하며 이전 handle은 invalid다.

## D04 — finite aggregate와 grouping

`FiniteSource::AuthoredMembers`는 입력 배열의 각 ordinal(중복 값 허용), `ResolvedTargets`는 executor가 확정한 UnitInstanceId set을 쓴다. duplicate target identity는 canonical set에서 한 번만 세며 새로운 전장 탐색은 하지 않는다. grouping은 명시적 `WholeSet | ByOwner | ByProjectedTile`; 후자는 기존 `UnitBody::projected_tile`/Position을 사용한다. group별 inner Count/Sum/Max와 group-result outer reduction을 **별도** 작성 필드로 구분해 타일 count와 대상 count를 혼동하지 않는다. owner는 실제 side/employee/copy 중 declared domain이어야 하며 서로 다른 domain 혼합은 거부한다.

algorithm: admission 대상 cardinality를 확인→member IDs stable sort→moment frame에서 값·group key 캡처→`BTreeMap<GroupKey, Accumulator>`에 checked fold→group key order로 outer fold. WholeSet의 empty Count/Sum은 0(출력 unit 필수), Max는 EmptyAggregate typed error(Unavailable의 구체 사유). empty group를 발명하지 않는다. input [A(tile0,5),B(tile0,3),C(tile1,2)]에서 flat Sum=10/Max=5, tile별 Max 후 Sum=7, tile별 Count 후 Max=2. 이 fixture가 old C03 일괄 제외를 교체하는 근거다.

node/member/group/work/retained를 각각 charge; nested aggregate의 최악 member product는 checked 곱셈으로 admission하며 VM 실행 중 매 member에서도 cancel/work 확인. 입력 순열이 같은 identity set이면 결과/trace/fingerprint가 같다. 네트워크, 전역 타일 상태 생성, 연결망, R10 새 target/flow, R07 fragment-copy aggregate는 지원하지 않는다. 이들은 현재 R06의 finite target grouping과 다른 기능이다.

## D05 — Attack/SpellPower·무기 성장·cadence

변경 파일: `stats.rs::{StatId,UnitStats,apply_modifier}`; `data/equipment_data.rs::WeaponCombatProfile`와 validation; `data/employee_data.rs`의 profile reader; `battle/types.rs::{UnitCombatProfile,BattleEquipmentBinding,BattleEquipmentEnhancement,BattleUnitDraft}`; `battle/stat_pipeline.rs`; `combat_setup/player_spawns.rs`의 battle/Certified producer; `resources/inventory.rs`, `employee.rs`, `growth.rs`는 기존 persistent owner 재사용/회귀 검증.

`UnitStats.spell_power:u32=0`, weapon numeric contribution `{attack:u32=0,spell_power:u32=0}`를 추가한다. 새 0 필드는 D07처럼 omission. SpellPower Flat/Percent는 기존 Attack modifier의 saturating·integer truncation 의미를 재사용한다; 이는 expression의 checked 수학과 구분한다. 새 weapon base contribution의 u32 합산 범위를 검증하고 overflow는 draft construction error. 기존 Percent들을 재정렬/통합하지 않는다.

employee base→기존 fragment rank/growth profile→기존 consumable battle modifier→기존 behavioral weapon override라는 profile 순서를 보존한다. numeric weapon contribution은 `persistent_stats_for_draft_profile`에서 선택된 실제 equipped weapon 한 번만 더한다→기존 GrowthStack→equipped binding 순서대로 Permanent+해당 사본 enhancement→battle에만 scenario scale/starting HP. Certified는 같은 persistent composition을 사용하되 기존대로 consumable/readiness/current HP/scenario scale 제외. `apply_weapon_profile`은 behavior/cadence만 적용하여 재가산하지 않는다. 기존 여러 weapon 거부 유지. 신규 SpellPower는 명시적 authored 값만 사용하며 Magic type, 기존 공격력 배율/defense override에서 암묵 추론하지 않는다. R06은 기존 scenario Attack scaling에 SpellPower 수치를 새로 연동하지 않는다.

enhancement projection은 `(base_uuid, persistent_copy_uuid:Option<Uuid>, level)`로 identity를 보존한다. persistent binding은 정확히 같은 copy 한 row; duplicate copy, mismatched base, missing owned copy는 InvalidStaticData/InventoryItemNotFound. 장비가 없는 scenario와 legacy scenario-only binding(None)은 강화 row가 없으면 0; None에 nonzero enhancement를 definition 전체에서 가져오는 fallback은 금지. explicit fixture enhancement가 필요하면 runtime binding identity를 갖춘 typed fixture 생성자를 사용한다. 원래 persistent 저장 값은 건드리지 않는다.

oracle: employee40+weapon40+KillStack10+Permanent5+enhancement5=100; weapon 입력만 0이면60, growth만0이면90. 같은 definition의 accessory copy A level1/B level2, per-level +5라면 총+15, 기존 오류 +30이 아님. SpellPower20→35 때 Attack 및 damage type은 동일; SpellPower ref 식만+15. 이중 profile 호출 결과가 누적되지 않아야 한다.

AttackSpeedPoints는 기존 `basic_attack_speed_percent`의 typed alias(100=base cadence), 저장 필드 rename 없음. `scale_attack_interval_ms` 호출 전 base>0, point>0, ceil(base×100/point)≤u64::MAX 검사; 1500/200→750ms,1001/150→668ms,1/200→1ms. 실패는 panic 전에 typed 거부. 결과 interval에 기존 fragment interval modifier를 이전 순서대로 적용. area tick/projectile flight/focus/bombardment cadence는 변화 없음.

## D06 — resource/result·시간·HP read 계약

`resource_runtime.rs::apply_typed_resource_operations`에서 definition→actor→owner→scope 선택을 **read-only 공통 resolver**로 추출한다. input은 `(ResourceId, ResourceOwnerSelector, RuntimeFactSource, target, real ResourceContext)`, output은 기존 `ResourceLease`; `ResourceStore::bind(&self)`→`value(&self)`로 읽는다. 읽기에서 `prepare_runtime_resources`, scope open/retain, mutation/reservation은 호출하지 않는다. runtime entry가 준비하지 않은 required resource context는 오류다. balance/reserved/available를 explicit field로 구별하고 initial sparse value는 catalog 값만 허용. 동일 definition의 다른 copy/employee/lifetime은 결코 합치지 않는다.

HP selector는 Source/Target의 Current/Max/Missing; missing=max−current이며 invariant current≤max를 확인한다. Time selector는 BattleElapsed 또는 명시적 Action/DeliveryDeadline Remaining. deadline 부재는 MissingContext; 시간을 임의로 0이나 무한대로 만들지 않는다. 시간 차의 signed underflow 대신 명시된 remaining=max(deadline−now,0)를 사용하며 expire된 deadline의0은 실제 의미다.

Result input은 authored `EarlierEffect(effect_id, field)` 또는 `CurrentActionPrefix(field)`와 실제 private ActionHandle의 조합으로 한정한다. 다른 action/과거 ordinal 직접 lookup·저장 handle 복원·take_settled_action_result 호출은 금지. field whitelist는 실제 FactSummary의 damage_hp_lost(명시 hp amount), healing_delta(signed amount), resource_change_delta(동일 typed resource로만), damage_strikes/damaging_strikes/count다. 혼합 resource aggregate delta를 한 resource balance로 가장하지 않는다.

D02 실행 owner가 effect 완료 직후 actual settled mutation facts의 immutable prefix witness를 생성한다. `(handle capability, ActionSite, effect index, delivery occurrence, fact frontier)`를 key로 하고 현재 effect 시작 전에 끝난 prefix만 읽는다. CurrentActionPrefix는 마지막 완료 frontier까지의 summary이며 terminal action 전체 완료와 다르다. effect0의 prefix 없음, 읽을 이전 effect 미완료, future/same effect self-dependency는 각각 NotSettled/FutureResult. compile 때 static later/self 참조를 거부하고 runtime에도 exact frontier·same-attempt 검사를 한다. valid zero 값은 완료된 witness가 실제 존재할 때만 반환한다.

witness는 해당 action/delivery의 bounded side table이며 authored 최대 effect 수로 admission, 종료 후 R04 retention과 함께 해제한다. full fact history를 보존하지 않고 요청된 field projection만 저장한다. deferred delivery는 자기 발생 이전에 완료된 prefix를 캡처해 소유하며 후속 live prefix를 요구할 경우 actual same-action frontier만 읽는다. 마지막 결과 삭제/eviction은 Unavailable이고 최근 다른 action 값으로 대체하지 않는다. 이전 effect damage7 후 현재 selfheal=EarlierEffect.damage_hp_lost이면7; target armor로 actual loss4면4. 시뮬레이션 rollback이면 witness/facts도 함께 복원한다. 이 adapter와 순차 실행이 이번 R06의 실제 구현 delta이며 R10 Flow 전체가 아니다.

## D07 — preview/explain·generation·저장·cache

`authoring/query.rs::{ExplainQuery,explain}`, `authoring/compiled.rs`, `authoring/preview.rs`, `battle/core/target_usefulness.rs`를 공통 compiled evaluator에 연결한다. static Explain Effect는 compiled ops/units/requirements/moment/fingerprint/source를 내고 context 없는 값은 `needs_runtime_context`로 명시한다. contextual 숫자 explain/preview는 existing authoring trial/application의 read-only state에서 exact EvalFrame을 받아 평가한다. future target/result가 필요한 pre-cast usefulness는 `Indeterminate(requirements)`와 existing non-quantitative eligibility를 분리해 미평가를 0 damage/false로 오인하지 않는다. snapshot 없는 pre-release preview는 명시 `prospective` frame이며 실제 committed snapshot과 동일하다고 주장하지 않는다. frozen 동일 frame의 runtime/preview/explain **amount/trace**는 같아야 하고 실제 RNG/mitigation damage는 damage authority에 따로 연결한다. 숫자 preview에 거짓 action/resource capability를 발급하지 않는다.

range_preview 공간 cell/rotation DTO는 유지하고 `effective_combat_profile_for_employee`에서 range를 받는 실제 read route를 회귀 검사한다. full profile이 필요한 quantitative consumer에는 `persistent/effective_stats_for_draft` 결과를 명시적으로 제공하여 draft-only base stats를 final 값으로 표시하지 않는다.

catalog는 GameDataBase의 immutable Arc로 generation과 함께 publish. pointer 주소는 durable identity가 아니며 fingerprint는 canonical typed AST+moment+compiler contract version+EffectSite semantics에 SHA-256; source-file 경로는 trace로 보존하고 값 의미와 구분. same-ID override 변경은 existing resolved content semantic digest와 plan digest를 바꾼다. shared compiled plan에 per-source-copy cache를 넣지 않는다. 우선 runtime result cache는 만들지 않고 frame당 평가한다. 필요 시 `generation digest, plan digest, actual action/delivery occurrence, source copy, target/group IDs, moment snapshot or live read revision/frontier` 전부가 key이며 누락하면 cache 금지.

Certified 기존 weight/model version은 유지. 별도 `CertifiedInputFingerprint` projection은 persistent employee identity/level, effective Attack/SpellPower/cadence, selected weapon numeric/profile, 실제 equipped copy/level, persistent growth/modifier, content semantic digest를 canonical sort/hash한다. live HP/consumable/readiness는 현재 모델의 제외 계약을 따른다. final weight 미정은 `evaluation_model_pending`로 별도 capability 설명; 새 SpellPower input의 fingerprint 변경을 평가 total 지원으로 표시하지 않는다. 현재 cache가 없으므로 재계산+fingerprint면 충분하며 가짜 cache를 만들 필요 없다.

V7 import는 typed deserialize 후 다시 serialize한 checkpoint hash를 비교한다. 따라서 새 persistent/definition zero field는 `#[serde(default, skip_serializing_if = "is_zero")]`(또는 동일한 zero-struct omission)를 적용해 **이전 필드 누락 bytes의 canonical serialization**이 보존되게 한다. UnitStats/weapon profile가 serialize되는 모든 위치와 content canonical serialization을 조사한다. nonzero SpellPower는 실제로 저장/복원되며 checksum에 포함되어야 한다. global ContentContract/version bump만으로 이전 모든 save를 거부하는 편법은 금지. additive capability는 old content canonical bytes를 유지하도록 설계하고, 새 expression/값이 있으면 digest가 실제로 바뀐다. 변경된 content에 old save를 억지로 붙이는 identity bypass도 금지. 이전 build로 생성한 V7 파일을 그대로 새 build에 넣는 fixture가 필수이며 새 코드 round-trip만으로 호환을 입증하지 않는다.

## D08 — 실패·취소·예산과 clone

`with_instant_resource_delivery`, `build_skill_step_commands` callers, area tick/impact, projectile impact와 event dispatch에 `Result`를 전파한다. temporary source/strike/recording context는 성공/실패 모두 복원하되 failure에서는 외부 candidate 전체를 폐기한다. delivery close 오류도 성공으로 묻지 않는다. 기존 control fault latch만으로 arbitrary ValueExpr error를 대신하지 않는다. 실제 최상위 `runtime_transaction/runtime_timestamp_reusing`의 `operation?`까지 모든 branch를 확인한다. native fast path와 controlled authoring path를 모두 시험한다.

예산 초기 제안(technical safety admission, gameplay 수치 아님): expression depth64/node1024, plan stack≤1024, dynamic members4096/group4096, effect prefix witnesses≤compiled effect count, per-frame retained scalar/member buffer1MiB, per-evaluation opcode/member work65536. compile cost는 checked upper bound로 계산하고 existing global execution/metadata/retained/work budget 중 더 작은 한도를 따른다. parser 입력 depth/node, aggregate cross product, repeated area ticks, long-lived carrier 전체 retained도 별도 산정. caller limits를 늘려 기존 fault를 감추지 않는다. zero work/cancelled token은 첫 mutation 전 typed 실패. 예산 숫자는 D10 owner가 actual workload 측정 후 더 엄격하게 조정할 수 있으나 accepted workload를 깨거나 상한을 늘리는 경우 영향·근거를 남겨야 한다.

capture/plan/witness를 clone_shape.rs와 native fact extent/preflight scan에 모두 포함. immutable Arc payload를 tick마다 deep clone하거나 HashMap capacity로 복사량을 과장/누락하지 않는다. full-core/state clone+새 retained 증가와 probe 측정 비용을 함께 기록한다. R05의 원래 concurrent requested-allocation cap 9,030,188 및 실패126을 보존한다; R06 추가 carrier의 실제 비용이 이를 침범하면 구조를 줄이거나 required evidence blocker로 기록하고 cap을 조용히 완화하지 않는다. allocation bytes는 RSS가 아니다.

## D09 — 구현 순서·시험 owner

최종 Phase/Goal owner는 P2G-40/50이다. 권장 하나의 atomic cutover 내 작업 순서: WP01 typed schema/compiler/catalog/diagnostics → WP02 capture/evaluator/resource/result/fallible runtime+quantitative consumers → WP03 stat/copy join/save/fingerprint를 WP01/02 공유 계약과 통합 → WP04 external capability/fixtures·documentation·전체 통합 검증. WP03의 source 조사/설계는 먼저 가능하나 writer 병렬화는 승인하지 않는다. schema가 public supported인데 executor가 없는 중간 상태를 Phase 완료로 제출하지 않는다. literal constant 경로와 새 AST 전체 소비가 연결되고 AT06의 save/projection 검사가 닫힌 뒤만 전체 cutover 완료다.

새 durable tests는 `core_enhanced/tests/value_expr_contract.rs`(public API/pack+runtime), `game/value_expr/*` unit tests, `game/world/tests/equipment.rs` 및 `snapshots_and_start.rs` save integration, `src/authoring/tests.rs` public limits/diagnostic reuse, 필요 fixture `tests/fixtures/content_authoring/value_expr/`를 WP04가 소유한다. crate name은 `game_core`; public CLI positional 인자는 `author_content <validate|describe|explain|simulate> TRUSTED_ROOT SELECTION.json [REQUEST.json] [--trace-output NEW_PATH]`다. 이것은 후속 명령 계획이고 이번 실행 결과가 아니다.

| ID / 요구 | 시험 입력·기대 결과/실패 경계 | 실행 수단·owner |
| --- | --- | --- |
| V01 E01 | Add(Milliseconds,Magnitude), resource A/B Sum→UnitMismatch publication 없음; valid Magnitude Add2+3=5 | WP01 unit, WP04 public negative pack |
| V02 E01 | 5/2와-5/2의 four rounding 및7/2 ties-even; missing rounding/0 denominator/i128 overflow→정확 code/site/node | WP01 unit/fixture, WP04 public |
| V03 E01/E08 | nested const/refs/Clamp; min>max, final Damage negative/u32 overflow, Heal i32 overflow 거부 | WP01/02, value_expr_contract |
| V04 E09 | old Damage -5/0/i32MIN, Heal signed amounts의 기존 actual 결과·counts·trace 의미 동등 | WP02/04 legacy captured fixtures |
| V05 E02 | source Attack100 release→140 impact; Snapshot100/Live140; release time10→consume30의 elapsed10/30 | WP02 immediate/projectile/area runtime |
| V06 E02/E05 | target-known snapshot, unknown directional target snapshot reject; caster removed Snapshot 유지/Live 오류; cancel finite delivery R04 의미 유지 | WP02 runtime, WP04 public |
| V07 E03 | Max5,3,2=5; Sum=10; empty Count/Sum0, Max error; permutation 동일 | WP01/02 unit/public |
| V08 E03/T27 | tile0:A5/B3,tile1:C2 → grouped Max then Sum7, grouped Count then Max2; owner/tile 혼합 거부 | WP02 runtime grouping |
| V09 E04 | 40+40+10+5+5=100, 각 입력 제거 delta; 두 copy level1/2 +5→15, duplicate/mismatch 거부 | WP03 stat/draft + World route |
| V10 E04/T18 | SpellPower20→35, Attack·type 불변; 획득→장착→강화→entry 실제 copy, Certified input digest만 필요한대로 변화 | WP03/04 world/equipment+public |
| V11 E06 |1500@200→750,1001@150→668,1@200→1;0/overflow reject; interval만 변하고 area/projectile/focus cadence 동일 | WP03 unit/runtime |
| V12 E05 | same resource ID different copy/lifetime 값11/23 분리, sparse catalog initial, balance/revision/lease count 무변경; stale/wrong owner reject | WP02 R05 store/runtime |
| V13 E05 | earlier damage7→heal7; armor actual4→heal4; not-settled/future/same effect/other attempt/evicted reject; missing prefix≠0 | WP02 effect-order runtime/public |
| V14 E08 | effect0 success 뒤 effect1 divide0; candidate HP/resources/queue/RNG/UUID/log/prefix/state/time 전부 시작값, prior committed prefix 보존 | WP02/04 native+controlled fault injection |
| V15 E07/E09 | old-build raw V7 checksum/import unchanged, nonzero SP/growth/enhancement exact fresh-process round-trip; tampered checksum/identity reject unchanged store | WP03/04 active_run public API |
| V16 E07/T18 | same ID override amount/type/moment/weapon 변화→digest 변화; identical semantics ordering→동일; old zero omitted content/save digest 유지 | WP01/03/04 public pack |
| V17 E07 | frozen exact context runtime/preview/explain amount+trace 동등; no context explicit needs_runtime_context; range cells/rotation 불변 | WP02/04 trial+CLI |
| V18 E08/T27 | public positive new RON validate/describe/explain/simulate nonzero effect, unknown opcode/context/unit explicit failure; AoE damage +1 selfheal의 ActionSite/delivery 공유·effect 구분 | WP04 external fixtures |
| V19 E08 | depth/node/member/work 각각 limit−1/limit/limit+1, cancelled acquisition/compile/capture/impact, no token/partial state; 다음 정상 retry 성공 | WP01/02/04 public+runtime |
| V20 E08 | many casts/repeated area/copy-isolated shared plans, stable bounded retained/clone counts, native/controlled 동일 results; long clock failure 후 bounded retry | WP04 existing cost harness와 새 workload |
| V21 E09 | unchanged literal official pack, R01/R04/R05 direct regressions, 전체 core suites·feature/build, required fresh independent audit | WP04 이후 별도 GEX 요청 owner |

## D10 — 구현 전 준비 점검·미결·다음 owner

계획 완료는 구현 준비 완료가 아니다. 후속 명시적 구현 요청의 owner는 mutation 전에: (1) planning-close manifest와 현재 branch/HEAD/status/per-file hash를 비교하고 모든 relevant dirty/untracked bytes를 새 implementation baseline에 보존, (2) R01/R04/R05 직접 producer·현재 audit/closure와 source drift 영향 확인, (3) D01–D08 타입·site mapping·fallible callback·checkpoint zero omission을 실제 interfaces와 대조, (4) old build V7/literal fixture를 **새 schema 변경 전** 생성·고정, (5) shared contract review 및 test-selector discovery/nonzero baseline, (6) 구현 순서 확정·단일 writer 확인을 기록한다. 별도 GEX 절차를 이번에 실행하지 않는다.

아직 실제 관찰이 필요한 항목과 owner:

- WP01: RON 0.5.1의 bounded Deserialize adapter 구체적 hook·최대 입력 allocation. source dependency는 Cargo.lock; 구현 전 작은 fixture/hostile depth probe로 확인. permissive recursive parser를 먼저 publish하지 않는다.
- WP02: result witness의 actual fact frontier 연결과 delayed caller의 Result signature propagation. 구현된 interface에 대한 call graph/negative test가 필요한 준비 항목이며 기존 query를 settled로 취급하지 않는다. 정책/상태 수명이 바뀌면 P2G owner 반환.
- WP03: 모든 zero field nested serialization이 old content/V7 SHA를 유지하는지 실제 old bytes로 확인. 실패하면 schema omission/canonical compatibility 구조를 교정하고 저장 손실/자동 변환안은 사용자 승인 없이 선택하지 않는다.
- WP04: 실제 expression worst-case benchmark 및 기존 source003 guard 재사용 영향. 현재 목표 장비·공식 새 성능 threshold는 미확정; 동일 환경 기존 guard를 유지하고 신규 workload의 operation/retained limit와 실제 측정치를 기록한다. 위 safety 예산은 공식 밸런스가 아니다.
- 구현 owner: 정확한 Rust symbol 이름, 모듈 내 private helper/borrow 구조는 위 meaning/order/owner를 지키며 조정 가능. 핵심 알고리즘·수용 기대값·save 의미·scope는 재량이 아니다.
- 외부 server/Unity/배포는 요청 제외. workspace server source의 Core DTO 소비는 read-only 조사 대상으로 연결하되 외부 호환 PASS나 변경 권한을 주장하지 않는다. 기존 public API shape 변경이 외부 구현 변경을 필수로 만들면 그 경계를 명시하고 별도 owner 요청; core-side additive optional observation으로 우선 설계한다.

Canonical documentation owner: core_runtime_contract.md(식/moment/settlement/failure), data_loading_contract.md(compile/publication/capability), skill_target_contract.md(effect vs target/delivery), game_rulebook.md(Attack/SpellPower·기존 Percent/cadence 보존), authoring 관련 architecture 문서에 가장 가까운 현행 owner를 연결한다. 새 지침·P2G/GEX/validator/AGENTS를 고치지 않는다. 기존 실패·audit·immutable artifact는 이력으로 그대로 보존한다.
