# A04 — 유물·장비 효과 선택과 합성 알고리즘

**권장 알고리즘(R)**이다. 공식 gameplay 값·미정 교체 우선순위를 확정하는 문서가 아니다. CA12에서 current code와 대조한 상세 설계와 PG01/PG02의 해당 결정을 남긴다.

## 1. 채택안: 원본 변경이 아닌 최종 계획 생성

```text
정의 세대 + 사본/형태/장비 + 장착 유물 + 규칙 profile
 → BaseEffectInventory
 → 부여 가능한 정의의 closure
 → typed 구조 변환
 → property별 수치·조건 보정
 → 결과 graph 재검증
 → EffectivePlanTemplate + BindingTable + ExplainReceipt
```

정적 합성은 장착·형태·장비 변경 때 수행한다. 실제 HP·공유 잔량·이번 타격 횟수는 runtime에 둔다. 조건부 modifier는 현재 조건을 평가할 predicate와 참조를 계획에 남긴다. HP가 바뀔 때마다 전체 스킬을 복제하지 않는다.

## 2. selector 계약

`EffectSelector`는 원콘텐츠·형태·effect role/ID, 같은 effect의 태그 all/any/not, property capability와 source 제한을 가진다. 전용 유물은 전용 콘텐츠 조건 **AND** 해당 효과 조건을 통과해야 한다. 다른 유물이 일반 태그를 만들어도 전용 identity 조건을 대신 충족하지 않는다.

한 효과 안에서 `all_tags=[X,Y]`를 요구하면 그 한 효과가 두 태그를 가져야 한다. “보유한 어떤 효과 X와 다른 어떤 효과 Y가 함께 존재”는 별도 집합 predicate다. selector가 대상 0개를 찾았을 때는 장착을 잃지 않고 해당 유물 효과가 휴면이다. 단, 명시적 incompatible와 copy 독점 위반은 장착 오류다.

## 3. 생성 효과의 안정적인 주소

원효과는 `(definition, form, authored_effect_id)`로 식별한다. 생성 효과는 `(provider_copy, provider_effect_site, matched_input_site, output_local_id)` 등의 안정적인 논리 주소로 생성한다. 구체적인 storage index는 컴파일 결과일 뿐 identity가 아니다.

Grant는 런타임 객체를 미리 생성하지 않는다. **발동할 수 있는 효과 정의·binding**을 추가한다. “폭발 부여”로 폭발 definition이 생기면 “폭발 강화”가 정적으로 활성화할 수 있다. 실제 폭발 피해는 해당 사건이 생겼을 때만 실행한다.

무한 Grant 확대를 막기 위해 compiled grant site별 적용 key와 ancestry를 추적한다. 자신의 산출물을 입력으로 삼아 같은 provider가 무한히 새 정의를 만드는 확장은 거부하거나 명시적 유한 반복 construct로만 표현한다. runtime 연쇄 제한과 별도의 compile-time 유한성 검사다.

## 4. 정적 활성 알고리즘

권장 시작점은 base effects에서 출발하는 **최소 안정 집합**이다. 긍정적 태그·capability 부여는 dependency graph의 positive component에서 worklist로 확장한다. 외부 근거 없이 A가 B만 요구하고 B가 A만 요구하면 둘 다 휴면이다.

NOT·태그 제거·교체로 자격을 없애는 변환은 positive closure에 섞어 무한 재계산하지 않는다. signed dependency를 만들고 층을 나누어 하위 층의 완료 결과만 읽게 하거나 충돌을 진단한다. 음의 순환은 명시적으로 거부하는 것을 기본안으로 한다. “마지막 적용 순서에서 우연히 남은 값”을 선택하지 않는다.

고정 3단계라는 이유만으로 변환 후 새 기능을 영구히 활성 판정에서 제외하지 않는다. 각 modifier가 어떤 입력 층을 읽고 무엇을 생성/제거하는지 선언하고 dependency 순서로 평가한다. downstream modifier가 변환된 효과를 읽을 수는 있지만, upstream 자격을 다시 바꾸는 feedback은 검증된 유한·층화 규칙이 필요하다. 최종 graph에서 참조와 활성 근거를 재검사한다.

## 5. 변환 연산의 계약

권장 `TransformSpec`:

```text
selector
required_input_capabilities
read_set / write_set
output_ports (원래 의미의 어느 출력을 대체하는가)
preserve_set (source/resource owner/lifetime/다른 효과 등)
operation sequence (엔진이 제공하는 typed edit만)
preconditions / postconditions
```

`SetProperty`, `AddPropertyModifier`, `GrantEffect`, `ReplaceSubflow`, `ConnectResult`, `RestrictRecipients`, `SelectFormVariant` 같은 공통 연산의 지원 범위를 좁고 명확하게 정의한다. 이는 임의 AST rewrite 언어가 아니다. 교체 subflow의 input/output과 완료/취소 계약이 맞지 않으면 컴파일 오류다.

특정 콘텐츠를 선택하는 RON recipe는 허용한다. 그러나 compiler에 `if content == Porccubus` 같은 행동 분기를 추가하지 않는다. 범용 유물은 지원하는 capability에 대한 typed transform recipe를 사용하고 특수한 수치 mapping이 있으면 원문 근거가 있는 data binding으로 표현한다.

## 6. 합성과 충돌

| 변경 종류 | 권장 처리 |
| --- | --- |
| 동일 종류 피해 증폭 가산 | 승인 정책대로 합산. 별도 공통 밸런스 상한 신설 금지 |
| 숫자 Add/Scale/Replace | property descriptor가 단위·수식·연산 단계 제공. 모든 Percent를 동일 연산으로 취급하지 않음 |
| 동일 값의 중복 Replace | 같음을 검증할 수 있는 경우 idempotent 합성 가능 |
| 서로 다른 Replace | 선언된 의미상 우선순위/명시 선택이 없으면 충돌 오류 |
| 공유 recipient/source 변경 | 관련 effect group과 공유 reference를 검사. 다른 효과에 암묵 전파하지 않음 |
| subflow 교체·연결 | input/output·result·cancel·lifetime 검증 후 재컴파일 |
| 원효과 삭제 | 아직 참조하는 후속 조건·유물·자원이 있으면 오류 또는 명시적인 함께 교체 필요 |

UUID 정렬은 결정론적 동률 처리에 쓸 수 있으나, 의미가 충돌하는 Replace의 승자를 고르는 정책이 아니다. 팩 단위 정의 override와 전투 loadout modifier 우선순위도 서로 다른 단계다.

## 7. 두 실제 설계의 적용

**풍운승려:** bite damage의 footprint만 patch한다. self heal effect의 target·reference HP·발동 횟수·출처는 보존한다. 같은 action carrier를 공유하는 경우 A02의 scope 규칙을 적용한다.

**단일 지원 전환:** 포큐버스는 pool owner와 lifetime은 그대로 두고 수혜자 집합을 단일로 제한하며 초기량을 일반 상태 `2S`로 변경한다. 태아는 channel 제약을 유지하면서 수혜자와 basic damage/speed modifier를 각각 원문 교체값으로 바꾼다. 태아의 모든 값·시간을 곱하거나 시전자의 채널을 제거하지 않는다. 두 효과에 공통인 것은 typed 대상 전환 interface이며 모든 property를 같은 배율로 바꾸는 알고리즘이 아니다.

## 8. 설명과 캐시

ExplainReceipt는 base value/expression, selector match, 생성·변환 제공자, 적용/미적용 이유, conflict, 최종 plan ID를 기록한다. 활성·휴면 설명과 runtime의 “이번 사건에서는 발동 조건 불충족”을 별도로 표현한다.

Compiler/Composer 결과 캐시는 generation·형태·rule profile·구조 관련 loadout 입력에 결합한다. template과 copy binding을 분리하지 못하는 구현이면 copy identity도 키에 넣는다. 서로 다른 두 사본이 캐시 때문에 자원·방문 이력을 공유하는 시험을 반드시 실패시켜야 한다.

## 9. 구현자 상세 설계의 필수 산출

CA12의 DG에는 selector AST, typed patch opcode, activation dependency graph와 worklist 종료 근거, conflict matrix, 적용 key·산출 effect ID 예시, cache invalidation 표, 2개 실제 사례의 전후 plan diff가 있어야 한다. “태그를 보고 변경한다”라는 목적 설명만으로 구현 준비 완료가 아니다.
