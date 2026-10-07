# WP02 D04 — 실제 코드에 따른 구현 작업 지시

2026-09-27 · 코드 조사에 근거한 기술 지시 보완. 제품 구현은 별도 대화에서 수행한다.

## 0. 구현 담당자에게 전달할 요청

> 이 문서 순서대로 R06 WP02 D04의 유닛 관계·필터·장착 슬롯 전달을 구현해줘.
> 아래에 선택한 자료구조와 변경 경로를 사용하고, 정책이나 범위를 새로 설계하지 마.
> 기존 구현·동결 증거를 보존하고 실제 시험을 수행한 뒤 단계검토 target을 동결해줘.
> “부분 구현·자체 검증 종료 / 단계검토 타깃 동결 / 독립 검토 요청 대기”에서 중단해줘.

먼저 [기존 구현 지시서](../../../successors/r06_scope_20260927_001/IMPLEMENTATION_REQUEST.ko.md)의
U01–U04 적용 범위·O01–O15 검증·중단 경계를 읽는다. 이 문서는 그 지시서의 기술적 선택을 구체화한다.
[정책과 전체 계획](../../../successors/r06_scope_20260927_001/REVIEW_AND_PLAN.ko.md),
[계획 인계](../../../successors/r06_scope_20260927_001/PLAN_HANDOFF.ko.md)는 보존한다.
정책 변경이나 R06 전체 재계획이 아니다. 아래 새 타입/함수는 **만들도록 제안하는 이름**이며 현재 존재하는 API와 구분한다.

이후 설명의 소스 경로는 `core_enhanced/` 기준이다. [조사 증거](WP02_D04_OWNER_CODE_EVIDENCE_20260927.json)에
읽은 파일의 hash와 기존 target 비교를 기록한다. 이번 문서 작업에서는 Rust 코드를 수정하거나 제품 시험을 실행하지 않았다.

## 1. 코드에서 확인한 빈 곳

| 현재 파일·symbol | 확인한 상태 | 이번 조치 |
| --- | --- | --- |
| `src/game/value_expr/mod.rs`: FiniteSource, Grouping, EvalFrame | 집합은 AuthoredMembers/ResolvedTargets, owner 기준에는 구체적 관계가 없음. frame은 resolved 목록 한 개 | 필터를 가진 새 source와 명시적 관계 grouping 추가. aggregate node별 frame view 추가 |
| `value_expr/parse.rs`: WireFiniteSource, BoundedMembers, enter_node | 수동 bounded 역직렬화. 단순 derive만 붙이면 새 Vec가 기존 할당 제한을 우회할 수 있음 | predicate 목록도 할당 전 제한을 검사하는 visitor로 읽음 |
| `value_expr/compile.rs`: validate_aggregate_capabilities, admit_finite_source, normalize_source, put_op | 기존 source/grouping 전제와 수동 fingerprint encoding 사용 | 새 경우의 capability·작업/메모리 계량·정규화·encoding을 함께 변경 |
| `value_expr/eval.rs`: members, group_key, finish_aggregate | 모든 resolved aggregate가 같은 frame 목록을 읽음. owner는 member마다 하나 | node별로 이미 준비된 목록을 선택한 뒤 기존 축약기를 재사용 |
| `battle/core/value_expr_runtime.rs`: prepare_resolved_target_members_with_base | 모든 aggregate의 member ref와 tile 필요 여부를 합쳐 한 목록 생성. ByOwner는 명시적 domain 부재로 거부 | aggregate별 필요한 ref/조건/관계로 서로 다른 view 준비 |
| `resources/item_slot.rs`: ItemSlot::iter | ByType의 빈 슬롯을 생략. Any3은 compact Vec | 주소를 포함한 iterator 추가. 기존 iterator와 저장 구조 유지 |
| `combat_setup/player_spawns.rs`: 두 장비 projection | base UUID와 actual copy UUID만 넘김 | 실제 layout과 슬롯 주소 보존 |
| `battle/core/build.rs`: spawn_scenario_group / spawn_scenario_group_member | 두 생성 경로에서 RuntimeUnit/RuntimeItem 생성. 장비 Vec index로 runtime item ID 생성 | 두 경로에서 동일한 장착 snapshot 생성. 기존 runtime item ID 생성 순서 유지 |
| `battle/core/types.rs`: RuntimeUnit | source_identity와 owner는 있지만 장착 layout/slot 조회용 정보 없음 | 읽기 전용 장착 snapshot 전달. scratch 재사용·clone도 함께 수정 |

특히 다음 세 가지를 임의로 해결하면 안 된다.

1. `EvalMember.owner` 하나를 전역으로 채우면 같은 식 안의 Side/Employee/다른 슬롯 집계가 서로 오염된다.
2. 장비 binding에 slot만 추가하면 **장비가 하나도 없는 정상 layout**과 **layout 정보 자체가 없는 fixture**를 구분할 수 없다.
3. 기존 `draft()` 시험 helper는 `BattleUnitSource::TestFixture`를 만든다. owned UUID가 있어도 실제 직원 긍정 시험이 아니다.

## 2. 사용할 표현 구조

기존 RON의 Aggregate 필드와 기존 enum variant는 유지한다. 다음 항목을 **추가**하는 방식으로 진행한다.

```rust
// resources/item_slot.rs: 새 공통 주소. Ord/Hash/Serde 등 사용처에 필요한 derive.
enum EquipmentSlotAddress { Weapon, Armor, Accessory, Any3(u8) }

// value_expr/mod.rs: 새 타입
enum ResolvedOwnerRelation { Side, ActualEmployee, EquippedSlot(EquipmentSlotAddress) }
enum ResolvedTargetPredicate { IsActualEmployee, HasEquippedSlot(EquipmentSlotAddress) }

// 기존 variants 유지 + 아래 variants 추가
FiniteSource::ResolvedTargetsWhere(Vec<ResolvedTargetPredicate>)
Grouping::ByResolvedOwner(ResolvedOwnerRelation)
```

필터 목록의 의미는 **모든 조건을 만족하는 member만 포함**하는 conjunction이다.
이번에는 재귀적인 And/Or/Not AST·임의 비교식·사용자 함수·새 공간 조회를 만들지 않는다.
필요한 공개 조건은 위 두 가지다. 추가 조건이 실제로 필요해지면 그때 확장한다.

- Any3 주소는 0, 1, 2만 유효하다. 3 이상은 compile/parse 진단이며 “빈 슬롯”이 아니다.
- `ByResolvedOwner`는 두 resolved source에서만 허용한다. AuthoredMembers와 조합하면 typed 오류다.
- 기존 authored `ByOwner`의 의미는 그대로 유지한다. 기존 domain 없는 `ResolvedTargets + ByOwner`의 거부도 유지한다.
  신규 명시적 관계를 암묵적으로 그 variant에 끼워 넣지 않는다.
- WholeSet에는 기존처럼 outer reduction을 넣지 않는다. 새 그룹형 관계에는 outer reduction이 필요하다.
- 필터 목록은 bounded admission 후 고정 순서로 정렬·중복 제거한다. IsActualEmployee를 먼저, 슬롯 조건은 주소 순서로 정렬한다.
  빈 목록은 기존 ResolvedTargets로 정규화한다. 의미가 같은 조건 순열의 compiled fingerprint는 같아야 한다.
- 정적 잘못된 주소는 전체를 먼저 검증한다. 런타임에서는 정상 predicate false 뒤 불필요한 속성 조회를 계속하지 않는다.
  예: 실제 직원만 요청했으면 소환물의 장착 layout까지 조회할 필요가 없다. action/capture/대상 identity 검증은 필터보다 먼저 한다.

아래는 구현 후 만들 RON 시험의 모양이다. **현재 parser에서 실행되는 예시가 아니다.**

```text
Aggregate(
    source: ResolvedTargetsWhere([IsActualEmployee]),
    grouping: ByResolvedOwner(ActualEmployee),
    reduction: Max,
    outer_reduction: Some(Sum),
    member: Read(Stat(subject: AggregateMember, stat: Attack)),
)
```

이 예시는 grouping 동작 시험용이다. 일반 스킬 피해를 타겟 공격력 기반으로 바꾸지 않는다.

## 3. 장착 정보 전달을 먼저 연결

### 3.1 ItemSlot → BattleUnitDraft

`ItemSlot::iter_with_address()`를 추가해 `(EquipmentSlotAddress, &EquippedRef)`를 반환한다.
기존 `iter()`는 그대로 둔다. 두 iterator의 실제 장비 순서는 같아야 한다.

- ByType: Weapon, Armor, Accessory를 고정 주소로 반환한다. 빈 칸을 생략해도 뒤 장비의 주소는 바뀌지 않는다.
- Any3: 기존 Vec index를 `Any3(index)`로 반환한다. 이 주소는 그 관측 시점의 배열 위치이며 영구 슬롯 UUID가 아니다.
- ItemSlot의 저장 schema와 equip/unequip 동작을 변경하지 않는다.

`BattleEquipmentBinding`에 `slot: Option<EquipmentSlotAddress>`를 추가하고,
`BattleUnitDraft`에 `equipment_layout: Option<SlotLayoutKind>`를 추가한다.
이 둘을 추가하는 이유는 slot이 없다는 정보와 layout을 모른다는 정보를 분리하기 위해서다.

`player_spawns.rs`의 `certified_combat_profile_for_employee`와
`battle_unit_draft_for_employee_from_item_slot` 두 projection을 모두 변경한다.
ItemSlot에서 생성하는 정상 draft는 장비가 0개여도 `Some(item_slot.layout_kind())`를 전달하고,
각 binding은 실제 주소와 `persistent_copy_uuid`를 전달한다.
기존 출처 불명의 fixture/legacy draft는 `None`을 유지할 수 있다. 첫 장비를 Weapon이라고 추정해 채우지 않는다.

### 3.2 Draft → RuntimeUnit

새 `BattleEquipmentSlots`를 layout과 길이 3의 고정 슬롯 배열로 구성한다.
배열 값은 `Option<BattleEquipmentBinding>`이며, binding 존재와 실제 copy UUID의 존재를 구별한다.
RuntimeUnit에는 `equipment_slots: Option<Arc<BattleEquipmentSlots>>`를 추가한다.
`None`은 layout 미제공, `Some(...빈 배열...)`은 알려진 정상 빈 layout이다.
장착 정보는 해당 배치의 읽기 전용 projection으로만 사용하고 새 mutable 장비 소유 저장소로 만들지 않는다.

새 생성 helper를 하나 만들어 `build.rs`의 **두 spawn 경로**에서 호출한다.

1. draft의 layout/binding 구조를 확인한다. layout이 알려진 경우 중복 슬롯·layout과 다른 주소·범위 초과를 거부한다.
2. 명시된 실제 사본의 nil UUID·중복 UUID 같은 잘못된 입력을 거부한다. actual copy가 미제공인 legacy binding과 구별한다.
3. 최대 3칸의 배열을 만든다. 기존 `equipped_items`의 순서를 재배열하지 않는다.
4. 새 snapshot의 보존 메모리를 기존 admission 경로로 확인한 후 Arc를 생성한다.
5. RuntimeUnit에 넣는다. 장비가 없는 legacy/fixture마다 새 Arc를 할당하지 않는다.

기존 `make_item_instance_id(..., index)` 및 `runtime_facts.register_provider`의 의미를 유지한다.
새 snapshot은 같은 draft에서 생긴 RuntimeItem의 base/copy와 일치해야 한다.
현재 전투 중 장비 교체 기능을 이 작업으로 추가하지 않는다. 재배치 때에는 새 draft에서 다시 projection한다.
Side는 `unit.owner`, Employee는 `unit.source_identity`의 **Employee variant 안의 employee UUID**를 사용한다.

### 3.3 빠뜨리면 안 되는 소비자

- `battle/types.rs`: 두 struct의 `clone_shape::struct_shape!` 선언과 모든 struct literal.
- `battle/core/types.rs`: RuntimeUnit의 clone shape, `reuse_timestamp_value()`의 전체 destructuring·필드 복사.
- `battle/core/build.rs`: 두 RuntimeUnit 생성 경로 및 player deployment 연결.
- `battle/core/mod.rs`: scenario binding/RuntimeUnit 메모리 산정 경계.
- profile/stat pipeline 및 시험의 binding 생성자: 기존 값 그대로, 새 정보의 유무만 정확히 명시.

현재 `CloneShape for Arc<T>`는 Arc 내부 payload를 전부 방문하는 보증으로 사용하면 안 된다.
새 공유 payload의 실제 allocation/admission과 보존 extent를 명시적으로 확인한다.
원본 저장·literal 결과를 바꾸거나 WP03 강화 합산 문제를 함께 해결하려고 범위를 확대하지 않는다.

## 4. Aggregate node마다 별도의 준비된 view 사용

이 작업에서는 **node_index별 member view** 방식을 선택한다.
EvalMember에 모든 가능한 관계를 넣는 범용 사전이나 전역 owner cache를 만들지 않는다.

새 `PreparedResolvedTargetView`는 `node_index`와 `Arc<Vec<EvalMember>>`를 갖는다.
EvalFrame에 정렬된 view 목록을 공유하는 `Option<Arc<Vec<PreparedResolvedTargetView>>>`를 추가한다.
기존 `resolved_targets` 필드는 기존 단위 시험/호출자 호환용으로 남긴다.
실제 battle adapter가 만든 frame은 모든 resolved aggregate에 대해 node별 view를 제공한다.

| evaluator 조회 | 동작 |
| --- | --- |
| frame에 node별 view 목록이 있음 | 해당 node의 view를 찾음. 없으면 오류. 다른 node의 view로 fallback 금지 |
| view 목록이 없고 기존 ResolvedTargets + WholeSet/ByOwner/ByProjectedTile | 기존 frame 목록 경로 유지 |
| 새 필터 source 또는 ByResolvedOwner인데 node view 없음 | MissingContext. 기존 무필터 목록으로 대체 금지 |
| 존재하는 node view의 members가 빈 목록 | 정상 빈 집합. “준비되지 않음”과 구분 |

`EvalMember.owner`는 **그 view의 관계에 대한 키 한 개**이므로 기존 owner 표현을 재사용할 수 있다.
Side view에는 Side 키, Employee view에는 Employee UUID, 슬롯 view에는 actual copy UUID를 넣는다.
다른 view가 같은 EvalMember를 mutable하게 공유하지 않게 한다. 만들어진 Vec/Arc는 읽기 전용이다.
원 대상 목록은 모든 view에서 같은 유닛 identity 기준으로 출발한다. key가 같다는 이유로 UnitId를 중복 제거하지 않는다.

`ResolvedMemberFrame`은 이 view 목록과 전체 retained_extent를 소유하도록 바꾼다.
`build_value_expr_frame_with_members()`는 목록의 Arc만 frame에 연결한다.
release capture와 각 대상별 frame이 동일한 immutable 준비 결과를 공유하는 기존 방식은 유지한다.
한 대상마다 전체 목록을 다시 복제하거나 relation view를 전역 cache에 남기지 않는다.

## 5. Runtime adapter 처리 순서

`prepare_resolved_target_members_with_base()`를 아래 순서로 바꾼다.
기존 entry point를 유지해 `sim.rs`와 release 경로의 호출 규칙을 재사용한다.

```text
필요한 resolved context 존재 확인
→ compiled segments에서 각 resolved Aggregate의 node/grouping/source/member_segment 수집
→ 원 targets 스캔·identity 정렬/중복 제거에 대한 admission
→ 각 실제 UnitId의 존재·active 확인
→ 각 aggregate에 대해:
    그 member_segment의 직접 member-scoped Read 목록 확보
    원 canonical 유닛 목록을 순회
    명시한 predicate 적용
    통과한 유닛의 관계 키 해석
    그 aggregate에 필요한 명시적 Attack/HP 값만 읽기
    그 aggregate의 EvalMember Vec 생성
→ node_index로 정렬한 immutable view 묶음 반환
```

핵심 사항:

- 전체 frame의 member ref 합집합을 모든 유닛에 읽는 기존 방식을 그대로 사용하지 않는다.
  제외한 member의 HP/stat을 불필요하게 읽어 다른 집계에 오류를 전파하지 않는다.
- 중첩 Aggregate는 자기 node와 member_segment를 가진 별도 view다.
  안쪽 aggregate의 AggregateMember ref를 바깥 member의 ref로 섞지 않는다.
- bare `ValueRef::AggregateMember { unit }`는 유닛 scalar producer가 없으므로 계속 거부한다.
- 실제 직원 여부 판정은 bool이지만 잘못된 source identity의 nil employee UUID는 오류다.
- slot helper는 `Result<Option<binding>, fault>` 의미로 만든다.
  알려진 정상 빈 슬롯만 `Ok(None)`이다. layout 미제공·주소/layout 불일치·binding 불일치는 오류다.
- `HasEquippedSlot`은 정상 빈 슬롯을 false로 만든다. 장비는 있으나 actual copy가 없는 경우
  grouping에서 실제 copy key를 생산할 수 없으므로 오류다. 장비 존재와 사본 identity의 유효성을 혼동하지 않는다.
- 필터가 없거나 predicate를 통과했는데 관계 키를 만들 수 없으면 typed 오류다. skip/0/첫 장비 fallback 금지.
- static 주소 오류와 action/capture/대상 identity 오류는 필터 false로 숨기지 않는다.
- 실제 값을 읽을 Read의 node_index를 오류에 연결한다. 전체 식에서 동일 ValueRef의 첫 위치만 고르는 것으로
  서로 다른 view의 오류 위치를 잘못 표시하지 않도록 한다.

### Snapshot/Live 연결

release의 `build_value_expr_delivery_capture()`가 만든 Snapshot용 view에는 predicate 판정·owner key·값이 모두 고정된다.
`evaluate_value_expr_capture_with_resolved()`에서 그 view를 Live 입력으로 덮어쓰지 않는다.
Live는 기존 ordered effect 실행 시점에 한 번 준비하고, 같은 effect의 모든 대상 amount 계산에 사용한다.
모든 amount 평가가 성공한 뒤 mutation하는 `sim.rs` 경계를 유지한다.

## 6. Parser·compiler·evaluator 세부 변경

### Parser

`WireFiniteSource`에 새 variant를 추가하고 `BoundedResolvedPredicates`와 visitor를 둔다.
기존 `BoundedMembers`의 **요소를 읽기 전에 제한을 확인하는 방식**을 참고한다.
predicate 항목도 전체 parse node 예산에 포함한다. 무제한 `Vec::deserialize` 후 길이 검사로 대체하지 않는다.
RON 문자열 길이 검사만 믿지 말고 직접 typed loading 경로에서도 제한을 지킨다.
프로그램으로 만든 AST는 compiler 단계에서 같은 제한을 다시 검사한다.

### Compiler

다음 symbol의 새 variant 분기를 함께 처리한다.

- `enter`, `exit_aggregate`: group/outer reduction, member 단위, AST node/work/storage.
- `validate_aggregate_capabilities`: 두 resolved source의 moment/target availability, 명시적 관계, source×group 조합.
- `admit_finite_source`: 새 predicate 목록을 clone/정규화하기 전 compile work와 저장 공간 확인.
- `normalize_source`: 조건 정렬·중복 제거·빈 목록 정규화.
- `operation_work`, `execution_shape`, cost/retained 계산: 동적 집계 view와 predicate 비용 누락 여부.
- `ContextRequirements`: 기존 resolved_targets/owner_grouping 플래그와 실제 요구 관계·조건 설명의 일치.

위 목록을 끝낸 뒤 `rg -n 'FiniteSource::|Grouping::' src`로 전체 참조를 확인한다.
특히 `matches!`, `if let`, `let ... else`는 새 variant가 추가돼도 컴파일이 성공하면서 잘못된 분기로 빠질 수 있다.
`runtime_dynamic_work`, 정적 계산 가능 여부, delivery capability, 비용 산정의 그런 분기를 직접 확인한다.

`put_op`에는 현재 `*grouping as u8`이 있다. payload variant를 추가하면 이 코드는 사용할 수 없다.
`put_grouping()`을 만들어 기존 태그 **WholeSet=0, ByOwner=1, ByProjectedTile=2**를 보존하고
새 ByResolvedOwner에 새 태그와 relation/slot payload를 기록한다.
`put_source()`의 기존 AuthoredMembers=0, ResolvedTargets=1도 유지하고 새 필터 source의 정규화된 조건을 기록한다.
relation/slot/predicate는 각기 명시적 태그로 encoding한다. Debug 문자열이나 enum 메모리 표현을 hash하지 않는다.
이 additive 확장만을 이유로 전체 compiler version을 무작정 올려 기존 literal fingerprint를 바꾸지 않는다.
이전 variant의 digest 보존과 새 의미의 digest 구별을 모두 시험한다.

### Evaluator

`members()` 시작 시 사용할 member slice를 한 번 선택하고, raw_count·스캔·canonical 목록 생성 모두 같은 slice를 사용한다.
일부 분기만 고치고 나머지가 `frame.resolved_targets()`를 읽게 남겨두지 않는다.
`group_key()`는 ByResolvedOwner를 해당 view의 owner 키로 처리하되 예상 domain과 실제 key domain을 확인한다.
`grouping == ByOwner`로 owner 정렬/키 bytes를 계산하는 곳은 새 관계도 포함하는 helper로 바꾼다.
GroupKey/축약/checked arithmetic/결정론을 재사용하고 두 번째 evaluator를 만들지 않는다.

**빈 Max의 실제 현재 오류 코드는 `ValueExprFaultCode::Unavailable`이다.**
기존 문서의 EmptyAggregate는 빈 Max 오류 의미를 설명한 표현이다. 존재하지 않는 enum variant를 구현 전제로 삼지 않는다.
전체 집합의 Max(empty)는 오류, Count/Sum(empty)는 0이라는 계약을 유지한다.
그룹이 0개일 때에는 기존 `finish_aggregate()`의 outer reduction 규칙을 따른다.
예를 들어 그룹별 Max 뒤 outer Sum이면 만들어진 그룹이 없어서 Sum(empty)=0이다.
inner Max를 실행하지 않은 경우까지 임의로 빈 Max 오류로 바꾸지 않는다.

## 7. 메모리·작업량·clone

준비 단계의 **모든 view 합계**를 bounded하게 만든다. view마다 한도를 새로 받아 전체를 무제한 늘리지 않는다.
기존 ExprLimits/authoring control/native retained admission을 재사용한다.

계량 대상은 원 targets 스캔·canonical 정렬, op/ref 스캔, predicate 검사, 관계 조회,
각 view의 member/value/key 저장, 정렬용 임시 목록, Arc/Vec 및 lookup 목록이다.
checked 산술로 상한을 계산하고 해당 단계의 allocation 전에 admission한다.
필터가 모두 false여도 원 targets와 predicate의 작업량은 발생한다.
단계별로 이미 제한된 기존 prepare/evaluate meter를 임의로 합치거나 제거하지 말고,
준비 전체의 비용을 정확히 합산하고 evaluator의 별도 제한도 유지한다.

`retained_extent`, `capture_extent`, `frame_storage_bound`, delivery capture의 row 합계,
native/controlled clone, scratch 재사용·drop 경로를 함께 대조한다.
실제 공유하는 payload의 메모리를 누락하지 않고, target마다 같은 공유 목록을 실제로 다시 할당하지 않는다.
새 슬롯 projection과 view 때문에 다른 비용 guard가 실패하면 그 원인과 변경을 보고한다.
기존 [비용 정책](R06_COST_POLICY_UPDATE_20260927.ko.md)의 특정 concurrent peak 예외를 다른 실패에 적용하지 않는다.

## 8. 작은 작업 단위와 시험 순서

| 순서 | 끝낼 변경 | 바로 확인할 것 |
| --- | --- | --- |
| 1 | 주소 타입·ItemSlot iterator·draft metadata·두 projection | 빈 앞 슬롯 뒤 장비 주소, Any3 주소, 기존 iter 순서 보존 |
| 2 | 두 spawn 경로·RuntimeUnit 장착 snapshot·clone/scratch/extent | known-empty/unknown 구별, copy 보존, legacy 비슬롯 경로 |
| 3 | 새 source/grouping·bounded parser·compiler·fingerprint | 같은 조건 순열 정규화, invalid 조합/주소, 기존 digest 유지 |
| 4 | node별 view 및 evaluator 선택 | 같은 식의 Side+Employee+서로 다른 slot view 격리, missing/empty 구별 |
| 5 | 실제 adapter·Snapshot/Live·effect 사전 평가 연결 | 실제 producer 값으로 O01–O15, 실패 시 부분 mutation 없음 |
| 6 | 관련 회귀·비용·전체 시험·증거 동결 | 기존 지시서의 명령과 단계검토 종료 경계 |

중간 순서는 구현 내부 진행 단위다. 별도 승인이나 독립 단계 완료 판정을 요구하지 않는다.
빌드가 깨진 큰 변경을 모두 쌓은 뒤 처음 확인하지 말고, 각 연결 단위에서 관련 compile/focused 시험을 확인한다.
전체 회귀를 의미 없이 매 단계 반복할 필요는 없다. 최종 source에서 요구된 전체 검증을 수행한다.

시험 배치:

- `resources/item_slot.rs`의 시험: 주소·빈 칸·Any3.
- `value_expr/parse.rs`, `value_expr/tests.rs`: parser/compiler/evaluator/fingerprint와 제한 경계.
- `battle/core/value_expr_runtime.rs`의 기존 runtime 시험: 실제 source→frame→amount 소비.
- `combat_setup/player_spawns.rs` 및 관련 world 시험: 실제 직원·inventory copy→draft→runtime projection.

재사용할 기존 runtime 시험:

- `actual_resolved_members_canonicalize_identity_and_read_projected_tiles`
- `empty_resolved_set_is_distinct_from_missing_and_unsupported_context`
- `ordered_effect_uses_one_live_member_set_before_mutation_then_rebuilds_it`
- `released_member_values_and_tiles_remain_snapshot_while_live_advances`
- `resolved_member_preparation_admits_exact_native_extent_before_copy`
- `aggregate_fault_restores_the_current_candidate_and_allows_clean_retry`
- `one_effect_evaluates_every_target_before_any_target_mutates`

기존 `draft()`/`core_with_units_and_equipment()`는 TestFixture source를 만든다는 점에 유의한다.
Employee 긍정 시험은 `BattleUnitSource::Employee`와 정상 장비 metadata/weapon profile을 갖춘 draft 또는 실제 world 경로를 사용한다.
TestFixture는 nonnil owned UUID가 있어도 Employee가 아니어야 한다는 부정 시험으로 활용한다.
같은 definition의 서로 다른 실제 copy를 사용하고, 런타임 필드를 억지로 Employee로 덮어쓴 시험만으로 실제 projection을 입증하지 않는다.

다음 오류·fallback 방지 시험도 넣는다.

1. 같은 node view가 없는데 다른 node view가 존재하는 경우: 오류.
2. 서로 다른 두 slot 조회가 같은 owner를 공유해버리는 경우를 탐지할 key/trace assertion.
3. 명시적 직원 필터가 스킬의 원래 타격 목록을 바꾸지 않음.
4. Snapshot capture 이후 Live view를 준비해도 Snapshot의 key/value/member 집합은 불변.
5. layout 미제공은 빈 슬롯과 다르게 오류; 정상 빈 슬롯은 명시한 predicate로만 제외.
6. 기존 domain 없는 ByOwner 오류와 실제 지원한 ByResolvedOwner 성공을 구별.
7. F01 무대상 projectile에서 amount만 건너뛰고 후속 effect/step/delivery 완료 유지.

## 9. 중단·보고 경계

공개 expression 전체의 production publication 차단은 유지한다. D07 전체, WP03 스탯 공식,
상태/효과 조회, 스킬 자동 취소, 실험실·편집기·20분 시험은 이 구간에 넣지 않는다.
정적 descriptor는 새 요구 조건을 정확히 설명하되, 실제 frame 없는 amount를 0으로 보여주지 않는다.

타입/함수 이름의 작은 조정은 허용하지만 이 문서의 node별 view/슬롯 전달 경로를 다른 구조로 크게 바꿔야 한다면
기존 코드에서 발견한 이유와 대안을 인계서에 남기고, 사용자 정책 변경이 필요한 경우에만 원 대화로 질문을 돌린다.
컴파일 에러를 없애려고 `.ok()`, `unwrap_or(0)`, 첫 장비 선택, 무조건 skip을 넣지 않는다.

최종 동결·복원 검증·실제 test/exit·비용 raw 기록과 중단 문구는 [기존 구현 지시서](../../../successors/r06_scope_20260927_001/IMPLEMENTATION_REQUEST.ko.md) §5–6을 따른다.
이번 구간 완료와 WP02/R06 전체 완료는 구별한다. 독립 검토는 원 대화에서 수행한다.
