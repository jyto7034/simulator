# A02 — 작성 스키마·중간 표현·컴파일 권장안

**제안 구조**다. 아래 이름과 RON 형태는 아직 실제 parser나 compiler에서 검증되지 않았다. 실행 가능한 문법은 각 작업의 코드 조사·설계 점검 후 구현된 최신 schema 하나가 소유한다.

## 1. 작성 모델의 선택

외부 제작 데이터는 **타입화된 선언과 구조화된 flow**로 표현한다. 스킬 내부는 자유로운 임의 jump보다 `Sequence / Branch / Parallel / Repeat / Wait / Invoke`를 기본으로 삼는다. 반복은 횟수·시간·외부 종료와 예산 계약이 명시되어야 한다. 지속 상태는 무한한 스킬 루프가 아니라 별도 activation과 event subscription으로 모델링한다.

보스의 장기 제어는 별도의 phase/behavior/pattern/state/task graph를 유지한다. 두 모델은 TargetQuery·ValueExpr·EffectProgramRef·ActionHandle 같은 작은 계약을 공유하며, 서로의 internal node나 mutable state에 직접 접근하지 않는다.

## 2. 작성 단위

| 단위 | 필수 의미 | 대표 검증 |
| --- | --- | --- |
| `DefinitionRef<K>` | namespace·kind·local ID | Skill을 Buff 위치에 참조하면 오류 |
| `EffectSite` | 콘텐츠/형태 안의 stable effect ID, semantic role, tags, operation | 단일 effect의 태그 AND를 다른 effect 합집합으로 만족시키지 않음 |
| `TargetQuery` | Unit/Tile/Object 종류, faction/trait/source filter, 정렬, 수 제한 | PlayerUnit·아군·재선정 허용은 consumer 계약으로 검사 |
| `SpatialScope` | acquisition/search/footprint/anchor/orientation/tracking | 단순 range 하나로 합치지 않음 |
| `ValueExpr<T>` | typed constants/references/operators/evaluation moment | 시간+HP, 가능성 있는 0 나눗셈, 미정 reference 금지 |
| `ResourceSpec/Ref` | owner·lifetime·initial/cap·변화·소비 | 수혜자에게 원천 pool을 복제하지 않음 |
| `ActionFlow` | sequence/branch/parallel/repeat·완료 기준·result slot | 미완료 result 읽기, 합류 shape, 반복 상한 검사 |
| `PersistentSpec` | state/aura/channel/override·종료 원인·중첩 정책 | cleanup 누락·다른 source 해제 금지 |
| `TransformSpec` | selector, typed operation, read/write set, preserve set | 비지원 property·공유 scope 변경·참조 손상 금지 |
| `EventBinding` | event kind, recognition key, predicate, invocation, causal policy | 정의되지 않은 trigger 의미·root reset 금지 |

태그는 사용자 팩에서 추가할 수 있다. 하지만 태그를 붙인다고 행동 capability가 생기지는 않는다. `[shield]`로 표시한 피해 효과에 shield initial capacity를 patch할 수 없다. 실제 capability는 opcode·schema와 compiler 검증이 결정한다.

## 3. 세 단계 표현과 런타임 binding

```text
Authoring AST
  문자열 ID·typed 데이터·원문 위치
    ↓ resolve/validate
Resolved Definition Graph
  팩 합성 후 참조·effect site·표현식 dependency
    ↓ compile
Compiled Templates
  stable index·typed opcode·result slot·property handle·SourceMap
    ↓ compose + revalidate
EffectivePlanTemplate + SourceBindingTable
    ↓ 실행에 필요한 입력 snapshot/live reference 바인딩
Activation / Delivery / State instances
```

**선택:** flow의 구조화된 AST를 인덱스 기반 immutable node 배열과 explicit continuation/result slot으로 낮춘다. op별 payload는 타입이 있는 enum/struct다. 최적화용 bytecode는 필요성이 입증된 ValueExpr 등 좁은 경계에서만 검토한다. 모든 gameplay를 하나의 bytecode VM으로 바꾸지 않는다.

Compiler는 다음 두 수준에서 실행한다. 팩 게시 전에는 정의·관계·지원 기능·기본 실행 계획을 검증한다. 장착 조합이나 형태 변경 때에는 실제 변형된 계획을 다시 검증한다. 개별 팩이 유효하다고 모든 loadout 조합이 유효한 것은 아니다.

## 4. 선택된 효과와 공유 전달의 관계

`ActionSite`는 release/전달/직접 타격의 주체, `EffectSite`는 그 결과에 적용할 효과 단위다. 하나의 투사체가 피해와 독 부여를 함께 전달할 수 있고 두 effect는 공통 impact context를 참조할 수 있다.

범위 확대가 피해 effect의 footprint만 바꾸는 경우에는 독 부여의 scope가 자동으로 바뀌지 않게 한다. 반대로 투사체 속도·발사 수처럼 공통 carrier 속성을 바꾸려면 **ActionSite 또는 명시한 effect group**을 선택해야 한다. 독립 효과를 선택한 것처럼 위장해 공유 carrier를 변경하지 않는다.

선택적 수정이 현재 공유 구조에서 표현되지 않으면 compiler는 원인·공유 참조를 진단한다. 별도 effect scope를 추가하거나 명시적인 변환 recipe로 분리하되, 조용히 투사체를 복제해 타격·출처·발동 횟수를 바꾸지 않는다.

## 5. PropertyDescriptor와 opcode 연결

제안 descriptor는 `operation_id`, 허용 consumer, typed input/output, property 목록, units, state requirement, capability read/write, finite completion 가능성, 진단 설명을 가진다. 런타임 함수 포인터를 RON에서 받지 않는다. engine registry가 operation 구현을 연결한다.

property는 임의의 Rust/JSON 경로가 아니라 `Damage.amount`, `Effect.footprint`, `Shield.initial_capacity`, `Channel.max_duration`, `BasicAttack.speed_bonus_points` 같은 **공개 의미 식별자**를 해석한 typed handle로 접근한다. 배열 위치나 메모리 주소를 제작 API로 노출하지 않는다.

Registry를 유일한 모든 코드 생성기로 만들 필요는 없다. 대신 registry에 runtime-supported로 공개된 op마다 parser·validation·compiler·executor·descriptor·negative test가 존재하는 일치 검사를 둔다. 계획 vocabulary는 지원 목록과 분리한다.

## 6. 컴파일 검사 순서

1. source snapshot의 syntax·field·kind·단위·required field 검사.
2. pack 의존·교체 결정을 적용하고 최종 definition 집합 확정.
3. 최종 집합에서 참조 해석, stable ID·owner·target type 검사.
4. flow result의 선행 생산과 branch/join·반복·capability 충돌 검사.
5. 계산식·자원 수명·Action completion·취소·동적 조건 입력 검사.
6. 변형 write conflict·지원되지 않는 carrier/scope 조합 검사.
7. compile resource budget·생성 가능한 구조 상한 및 domain setup 검증.
8. immutable output + source mapping + receipt 생성. 모두 성공해야 게시.

Override로 제거된 loser 정의의 죽은 참조를 최종 참조 검증에 포함해 유효한 replacement를 막지 않는다. 다만 선택된 팩의 원문 타입 자체가 손상되면 거부한다. 원문과 최종 graph 검증의 책임을 구분한다.

## 7. 스칼라·공식 규칙

수식은 유한한 typed AST를 채택한다. 정수/고정 정밀도 비율과 checked intermediate를 기본안으로 한다. 정확한 percent 단위·반올림·범위를 DG에서 승인 계약과 대조한다. `Min/Max/Clamp`, Count/Sum/Max와 GroupByTile은 별도 연산이다. 부재와 0, 상한 없음과 미정을 구분한다.

Attack/SpellPower는 스탯 입력이고 Physical/Magic/True는 피해 해석 종류다. 공격 속도 증가가 준비·반복·작업·전술 자원 회복 시간을 일괄 가속하지 않게 한다. typed `EvaluationMoment`로 CastStart/Release/Impact/EachTick 등을 구분하며 허용 조합만 노출한다.

## 8. CA01의 종료 범위

CA01은 이 계약을 current code에 연결할 수 있는 최소 schema/compiled shape와 실제 Damage/Heal/Instant slice를 고정한다. 모든 future op를 빈 stub로 등록하지 않는다. CA02는 그 최소 실행 slice로 최신 외부 pack을 게시한다. CA04 이후 새 기능마다 같은 compiler 경계에 추가하며 구형 실행기를 따로 유지하지 않는다.

예제와 type sketch는 [A08](08_WORKED_EXAMPLES.ko.md)을 따른다. 최종 구현에서 예제의 문법이 바뀌면 schema 문서와 해당 예제를 함께 고친다.
