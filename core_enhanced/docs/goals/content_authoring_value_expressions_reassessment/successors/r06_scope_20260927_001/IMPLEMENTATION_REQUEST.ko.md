# WP02 D04 — 유닛 관계·명시적 필터 구현 지시서

2026-09-27 · 별도 구현 대화에 전달할 요청문. 이 문서를 작성한 대화에서는 제품을 수정하지 않는다.

> 이 디렉터리의 `REVIEW_AND_PLAN.ko.md`와 successor 계획 인계를 읽고 R06 WP02의 다음 단계검토 범위를 구현해줘.
> 이미 구현한 결과 누적·release Snapshot·WholeSet/타일 집계·D04-F01 교정을 보존하고,
> 실제 유닛의 진영·실제 직원·명시한 착용 슬롯의 실제 장비 사본별 집계와 명시적 제외 조건을 연결해줘.
> 자체 검증과 단계검토 target 동결 후 독립 검토 요청에서 중단해줘. D07·WP03·WP04 전체나 후속 R로 자동 진행하지 마.

## 1. 범위·근거·시작점

[검토와 정책](REVIEW_AND_PLAN.ko.md)의 U01–U04가 이번 의미를 정한다.
[원 상세 설계](../../CODE_REVIEW_003.ko.md)의 변경되지 않은 계약과
[비용 판정 변경](../../execution/single_goal/handoffs/R06_COST_POLICY_UPDATE_20260927.ko.md)을 함께 적용한다.

이전 target은 [D04-F01 manifest](../../execution/single_goal/corrections/wp02_d04_f01_001/target/d04_f01_correction_001/MANIFEST.json),
SHA-256 `e4f0d77b1bd3066493d605271605d7a4b6e16317a6d540d303909b076b480104`다.
구현 직전 현재 source/dirty/untracked와 비교하고 구현 기준선을 보존한다. 계획 당시 333개 일치 결과를 새 currentness로 대신하지 않는다.
다른 작업자의 변경을 덮어쓰지 않는다. 기존 코드·시험·동결 인계는 재사용하되 현재 변경 영향에 맞춰 회귀를 검증한다.

다음 구간의 완료는 **D04 유닛 관계 구간의 구현·자체 검증 완료**다.
WP02/R06 전체 완료·production publication 지원·독립 감사 PASS가 아니다.

## 2. 입력과 의미

집계 입력은 전달 경로가 이미 확정한 `ResolvedTargets`다. 새 공간 query/상태 조회/범용 Flow를 만들지 않는다.
기존 AuthoredMembers의 명시적 scalar/owner와 WholeSet/ByProjectedTile 의미를 유지한다.

| 관계 | 입력·키 | 거부 조건 |
| --- | --- | --- |
| Side | 현재 frame의 실제 유닛 진영 | 유효한 유닛/관측 문맥 부재. id hash 등으로 진영을 추정하지 않음 |
| ActualEmployee | 실제 Employee source identity의 employee UUID | 적·소환물·다른 source variant. `source_owned_uuid != nil`만으로 허용하지 않음 |
| EquippedSlot(address) | 명시적 장착 layout/slot에 결속된 실제 equipment-copy UUID | 미지원 주소/layout, 빈 슬롯, 실제 사본 identity 부재/불일치. base definition/첫 장비로 대체하지 않음 |

유닛에 장비가 많다는 이유로 하나를 임의 선택하지 않는다. 실제 직원이 아닌 유닛도
명시적으로 지원하는 장착 관계와 실제 사본을 가진다면 그 관계 계약에 따라 판단한다. 직원 조건과 장착 조건은 별개다.
효과가 A의 장비에서 유래했더라도 대상 B의 착용 슬롯 조회는 B의 실제 장비를 뜻한다.
효과·상태의 출처 장비별 집계는 이 구간에서 미지원으로 진단한다. 단건 source read의 기존 지원을 제거하지 않는다.

`AggregateMember`의 값은 명시적 stat/HP ref로 선택한다. 유닛 자체를 scalar로 쓰거나 단위만 지정해 Attack을 기본값으로 만들지 않는다.
지원되지 않은 SpellPower/AttackSpeed ref를 이번 구간에서 0 또는 Attack으로 대신하지 않는다.

## 3. 명시적 제외 조건

최소 typed predicate는 실제 직원 여부와 지정 슬롯의 장착 존재 여부를 표현할 수 있어야 한다.
복합 조건은 유한·bounded 형태로 제한한다. 새 사용자 함수/일반 query language는 필요 없다.
정확한 Rust/RON 이름은 구현 결정으로 남기되 다음 의미를 보존한다.

```text
ResolvedTargets
  → 명시한 조건으로 집계 member 제한
  → member의 명시한 값 읽기
  → 지정 관계의 실제 key로 grouping
  → inner reduction → outer reduction
```

- 필터가 없으면 적용 불가능한 정상 member도 입력에 남는다. 관계 해석 실패를 typed 오류로 반환한다.
- `IsActualEmployee`가 false인 소환물/적, `HasEquippedSlot(address)`가 false인 정상 빈 슬롯은 명시적으로 제외할 수 있다.
- 잘못된/미지원 slot address는 빈 슬롯과 다르다. stale identity·잘못된 action/capture·권한 불일치는 false로 감추지 않는다.
- 필터는 집계에만 적용한다. 타격 목록/타격 횟수/완료 처리 대상을 바꾸지 않는다.
- 필터 뒤 빈 Count/Sum은 0, 빈 Max는 기존 EmptyAggregate 오류다. 암묵적 fallback/자동 cast 취소 없음.
- 원 입력 스캔·필터 실행·관계 조회·value read·정렬/그룹 축약에 실제 작업량을 계산한다.
  필터로 전부 지워지는 큰 입력도 cardinality/작업 예산을 우회하지 못한다.

## 4. 설계 방향과 동시 수정 경계

### 4.1 typed 표현·compiled plan

resolved grouping에 명시적 관계 선택자를 넣는다. 기존 `ByOwner`만으로 어떤 domain인지 추정하지 않는다.
authored owner의 기존 RON 의미는 유지하며, 새 표현은 parser→validation→compiler→evaluator까지 연결한다.
해당 조합이 어느 source에서 가능한지 compile/describe capability가 정확히 설명해야 한다.
조건·관계·slot address의 경로를 오류에 남겨 제작자가 원인을 찾을 수 있게 한다.

한 식에서 Side 집계와 Employee 또는 서로 다른 슬롯 집계를 함께 할 수 있다.
EvalMember의 단일 owner 필드를 첫 aggregate 기준으로 채워 뒤 aggregate가 재사용하는 설계는 피한다.
권장 구조는 relation별 typed key 조회 또는 aggregate별 준비된 읽기 전용 member view다.
공유 immutable frame을 써도 각 aggregate의 조건/키가 독립적으로 평가되어야 한다.
추가 cache가 있다면 source generation·frame/moment·관계/필터/slot 등 의미 입력을 포함하고, 미지원 캐시 공유를 하지 않는다.

### 4.2 슬롯 identity의 생산 경로

`resources/item_slot.rs`의 실제 layout과 주소를 보존하는 read-only projection을 추가하고
`combat_setup/player_spawns.rs`의 전투 draft와 Certified profile 양쪽 projection을 확인한다.
`BattleEquipmentBinding` 또는 같은 책임의 별도 binding에 actual copy와 typed slot을 함께 전달한다.

- ByType은 Weapon/Armor/Accessory 주소를 구분한다. 빈 슬롯 생략 후 Vec index로 주소를 재구성하지 않는다.
- Any3을 지원하는 경로는 명시적 Any3 주소로 표현하고 타입별 슬롯과 섞지 않는다.
  기존 layout이 compact Vec라면 관측 frame의 명시적 index임을 설명하고 영구 슬롯 UUID인 것처럼 사용하지 않는다.
- 같은 종류 장비 중 '첫 것'을 선택하거나 서로 다른 layout 주소를 암묵적으로 변환하지 않는다.
- legacy/test binding에 슬롯 정보가 없으면 기존 비슬롯 경로는 유지하고, 슬롯 관계 요청 때 정보 부족을 명시한다.
  허구의 슬롯을 채우지 않는다. native/controlled clone과 모든 binding 생산자도 확인한다.
- 같은 definition의 두 실제 copy를 합치지 않는다. 하나의 copy를 두 직원에게 중복 장착한 비현실 fixture로 정상 증거를 만들지 않는다.

이 metadata 추가가 직렬화/fingerprint/Certified 입력에 영향을 주면 그 직접 소비자까지 같은 구간에서 맞춘다.
기존 literal/old save의 canonical bytes·identity를 무관하게 바꾸지 않는다. WP03의 강화 수준을 definition별로 합산하는 문제는
별도 잔여 작업으로 표시한다. 슬롯 보존을 이유로 장착 규칙·다중 무기·성장 공식까지 바꾸지 않는다.

### 4.3 Snapshot/Live와 실패 경계

필터 판정·관계 key·member value를 같은 관측 frame에서 읽는다.
Snapshot은 release에서 확보한 입력/키/값을 고정하고 이후 Live 값이나 장비 상태와 섞지 않는다.
Live는 해당 effect가 gameplay를 변경하기 전에 목록과 모든 필요한 입력을 확보한다.
release에서 알 수 없는 미래 target 집계는 기존 지원 경계대로 거부한다. 임의 미래 lookup을 'Snapshot'이라고 부르지 않는다.

같은 effect에서 대상별 amount를 모두 준비한 뒤 mutation하는 기존 경계를 보존한다.
한 member 실패로 앞 대상만 피해를 받거나 자원/result/RNG/log/queue가 부분 갱신되지 않아야 한다.
기존 rollback/typed fault 전달을 재사용하며 grouping 실패를 새 성공/skill cancel로 바꾸지 않는다.

F01 유지: 대상 없는 projectile impact에서 amount 평가는 건너뛰되 effect row는 확인하며
후속 effect·step·delivery 완료를 계속한다. 실제 대상이 있으면 capture identity 검사는 그대로 수행한다.

### 4.4 공개 상태·preview 접속

compiled descriptor와 semantic fingerprint에 관계/필터/slot 의미를 포함한다.
정적 explain은 필요한 입력을 설명하고 실제 frame이 없으면 `needs_runtime_context`에 해당하는 명시적 상태를 준다.
공간 preview를 수치 preview 완료로 표시하지 않는다. D07 전체는 후속 구간이다.
`catalog.rs`의 전체 expression production publication 차단은 이 단계에서 해제하지 않는다.
시험 전용 허용 경로로 실제 소비까지 검증하더라도 public 완성 증거와 구별한다.

## 5. 자체 검증 기준

구현을 그대로 복제하는 단위 시험 대신 아래 관측 가능한 경계를 실제 경로에서 검증한다.

| ID | 입력/상황 | 기대 결과 |
| --- | --- | --- |
| O01 | 진영 A 값 3·5, 진영 B 값 7, 그룹별 Max의 Sum | 12. 입력 순서를 바꾸고 같은 UnitId가 중복돼도 12; 다른 UnitId의 같은 값은 보존 |
| O02 | 실제 직원 A/B, A가 만든 소환물, 적 혼합 | 조건 없는 Employee는 typed 오류·부분 mutation 없음. 명시적 직원 필터이면 A/B만 포함; 소환물은 A로 합치지 않음 |
| O03 | 실제 직원이 아닌 source에 nonnil owned UUID | Employee로 잘못 분류하지 않음 |
| O04 | A의 무기 X/갑옷 Y, B의 같은 definition 무기 Z | Weapon key는 X/Z, Armor key는 Y 등 명시 주소. definition 합침/첫 장비 선택 없음. 수치뿐 아니라 실제 key/trace 확인 |
| O05 | 앞 슬롯이 비고 뒤 슬롯만 장착, Any3 및 잘못된 layout 주소 | 빈 슬롯을 다른 장비로 대체하지 않음. 명시적 존재 조건만 정상 빈 슬롯 제외. 잘못된 주소는 오류 |
| O06 | 장착 binding의 copy/slot 부재, stale identity/capture | 부재와 불일치 진단 구분, 권한 오류를 필터 false로 은폐하지 않음. 기존 비슬롯 literal 경로 호환 |
| O07 | 같은 식의 Side+Employee 및 서로 다른 slot aggregate | 각 관계/필터의 결과 독립. AST 평가 순서나 cache 선행 호출로 key가 바뀌지 않음 |
| O08 | 필터 뒤 빈 Count/Sum/Max | 0/0/EmptyAggregate. 원 타격 목록에는 영향 없음 |
| O09 | release 이후 허용된 상태 변화, Live 재평가 | Snapshot의 값/키/필터 판정 유지, Live는 현재 frame 관측. 불가능한 장비 교체 기능을 시험 때문에 제품에 추가하지 않음 |
| O10 | Instant/TileArea/Projectile의 실제 amount 소비 | 지원 delivery/moment 조합마다 정답·source identity. 같은 광역 effect의 평가 도중 앞 타격이 뒤 입력을 바꾸지 않음 |
| O11 | 뒤 member의 관계·overflow·예산 오류 | 같은 effect의 damage/resource/result/RNG/queue/log 부분 변경 없음. 기존 typed fault 경계 유지 |
| O12 | 필터로 대부분/전부 제외하는 큰 입력, 다중 관계, limit 경계 | 원 스캔 포함 실제 비용 반영, 경계 바로 아래 성공/초과 typed 거부, clone/retained 상시 누적 없음 |
| O13 | RON 왕복, 잘못된 source×relation, 미지원 효과·상태 집계 | parser/컴파일 진단과 descriptor 일치, literal/명시적 authored ByOwner 호환. 전체 production publication 차단 유지 |
| O14 | 의미가 다른 filter/slot·같은 의미의 canonical 표현 | 다른 의미는 fingerprint 분리, 동일 의미 결정론. old literal/세이브 영향은 변경 범위에 맞게 검증 |
| O15 | F01 무대상 projectile와 후속 effect/step/delivery | 완료 경로 유지, 존재하는 대상의 capture identity 엄격 검사 유지 |

O01 수치는 집계 연산의 oracle이며 실제 콘텐츠 밸런스가 아니다.
O04/O09는 실제 game 규칙에서 가능한 producer 입력으로 구성한다. 장비 교체가 전투 중 불가능하면
release capture와 이후 별도 frame의 순수 adapter 경계를 검증하고 '인게임 장비 교체 지원'으로 보고하지 않는다.

기본 명령은 `core_enhanced/`에서 실행한다.

```sh
cargo check -p game_core --all-targets --offline --locked
cargo test -p game_core --lib value_expr --offline --locked
cargo test -p game_core --all-targets --offline --locked
cargo clippy -p game_core --all-targets --offline --locked -- -A clippy::never_loop -A clippy::eq_op
```

새 관계·projection 시험이 `value_expr` selector 밖에 있으면 해당 실제 test 이름을 추가로 실행·기록한다.
PC 36조건 및 관련 R05 allocation guard는 기존 정확한 도구/입력으로 회귀 검증한다. R05 기능을 새로 개발하는 작업이 아니다.
특정 concurrent peak 초과만 경고로 분류하고 raw 실패·samples를 보존한다. 다른 비용/결과/도구 실패는 성공으로 낮추지 않는다.
save/Certified/native clone 등 실제 변경된 소비자의 관련 시험은 함께 수행한다. WP04 전체 감사나 20분 시험은 추가하지 않는다.

## 6. 종료와 인계

구현 기준선, 최종 source, 변경 전후 bytes/patch, 실제 명령·선택 test·건수·exit·원 로그,
판정·실패 이력·원본 비용 결과, 요구 O01–O15 대응과 잔여 범위를 하나의 단계검토 target에 고정한다.
manifest digest와 currentness·격리된 양방향 복원 증거를 남기고 기존 frozen target은 보존한다.

인계서에는 D04 유닛 집계 완료 여부와 D07/WP03/WP04 잔여를 분리한다.
후속 효과·상태 조회의 미지원은 사용자 U04에 따른 배정이며, 이번 scope의 미구현을 후속으로 숨기는 근거가 아니다.
마지막 보고는 **“부분 구현·자체 검증 종료 / 단계검토 타깃 동결 / 독립 검토 요청 대기”**로 끝낸다.
독립 검토·WP02/R06 전체 완료 판정은 원 대화에 남긴다.
