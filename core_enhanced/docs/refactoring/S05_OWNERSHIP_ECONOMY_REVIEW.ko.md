# S05 사본 취득·소비·귀환 경계 조사와 유지 판단

작성·갱신일: 2026-10-17

현재 단계: 대표 흐름 조사·유지 판단·선택 테스트 19개 검증 완료. 후속 [S02/S11 조사](S02_S11_CONTENT_PUBLICATION_REVIEW.ko.md)도 마쳤으며 최신 다음 행동은 [출발점](README.md)을 따른다.

## 1 범위와 기준

S03/S06에서 확인한 상점 지급을 출발점으로 장비·파편·유물의 취득, 판매/분쇄, 성공 귀환·실패/포기,
다음 출발 대여를 추적했다. 강화·개화는 실제 투자액과 분쇄 환급을 설명하는 범위만 연결했다.
이번 종료점은 대표 조사·유지/개선 판정·필요 검증·후속 계획이다. 모든 성장·장착·사망·경제 활동의 전수 감사는 아니다.

기준은 [귀환과 대여](../game_rules/results-and-save.md#원정-귀환과-회수품-대여),
[재료 회수](../game_rules/results-and-save.md#재료-회수와-공용-창고),
[분쇄와 투자 환급](../game_rules/equipment-and-fragments.md#분쇄와-투자-재료-환급),
[유물 판매·분쇄](../game_rules/relics.md#판매와-분쇄), [runtime 계약](../core_runtime_contract.md)의 U12/U15 절이다.
최대 2사본 대여, 파츠 합계 2·부활 촉매 1의 선택 회수, 실제 투자 재료별 합계의 절반 내림 환급을 보존한다.
테스트용 수량과 recipe는 이 규칙을 검증하는 입력이며 공식 balance 값으로 채택하지 않는다.

직전 문서 4개와 S03/S06 선택 입력 28개가 그대로임을 먼저 확인했다.
[S01](S01_APPLICATION_BOUNDARY_REVIEW.ko.md)의 draft/commit,
[S04](S04_PERSISTENCE_BOUNDARY_REVIEW.ko.md)의 Continue·SavePoint·창고 소비 복원과 paired publication,
[S07](S07_COMBAT_BOUNDARY_REVIEW.ko.md)의 보상 수량,
[S03/S06](S03_S06_NODE_ACTIVITY_REVIEW.ko.md)의 상점 자격·quote·session은 재사용했다.

## 2 상태와 실제 생산·소비

| 상태 | 권위와 identity | 취득·소비·종료 |
| --- | --- | --- |
| 장비 사본 | `OwnedEquipment.instance_uuid`; metadata UUID는 정의 identity | `GrantExecutor`가 instance UUID를 생성하고 inventory에 추가. 판매/분쇄는 요청한 instance만 제거. 귀환은 보유 중인 사본을 같은 UUID로 vault에 이동 |
| 파편 사본 | `OwnedFragment.copy_uuid`, 개화 forms·선택 form·투자 원장 | grant가 새 사본 생성. 분쇄는 선택 사본과 장착 연결을 제거. `VaultFragment`는 form과 실제 투자 원장을 반납·대여 사이에 보존 |
| 유물 사본 | `RelicInventory`의 copy와 직원별 slot | 정상 variant로 지급. 분쇄 확인 후 해제·제거하고 가루 지급. 일반 상점 매각 불가. 성공/실패 terminal에서 원정 보유·slot은 정리하며 vault 대여 대상에 넣지 않음 |
| 영구 회수품 | `profile.lifecycle.vault.equipment/fragments` | 기록에 따른 재발급이 아닌 실제 사본 이동. 현재 정의가 없는 dormant 사본도 보관하되 대여는 거절 |
| 재료·투자 | 소지 재료, `RunCraftingResources`의 가루, vault 재료, 사본별 `MaterialInvestment` | 출처별 결제와 실제 누적액을 기록. 분쇄 환급은 소지분에만 지급. 성공 귀환 때 허용 재료를 명시 선택해 창고에 합산 |
| 원정 identity | player·checked sequence로 만든 `ExpeditionInstance`; active marker | 출발 시 새로운 instance·UUID stream, terminal 시 정확한 active instance 확인 후 marker와 소비 원장 종료 |

근거: [공통 지급](../../src/game/reward.rs), [inventory](../../src/game/resources/inventory.rs),
[유물 사본](../../src/game/relic.rs), [영구 회수품](../../src/game/expedition_lifecycle.rs),
[원정 전이](../../src/game/world/expedition_lifecycle.rs), [재료 원장](../../src/game/material_economy.rs).

### 취득부터 분쇄까지

1. 상점/보상은 기존 `EconomyGrantState`에서 자원·사본·UUID 변경을 준비한다. `GrantExecutor` 자체는
   ordered effect 실행기이며 그 함수 하나가 모든 호출자의 트랜잭션을 자동 보장한다고 해석하지 않는다.
   상점 구매 이력과 공개 command commit은 해당 owner가 결합한다.
2. 장비 판매는 S03/S06에서 확인한 귀속·장착·수량·quote 검사를 통과한 instance를 제거한다.
   장비 분쇄는 `EquipmentLoadoutTransition::remove_for_dismantle`로 inventory와 직원 slot의 양방향 binding을
   함께 정리한다. 파편은 해당 사본의 장착 해제와 삭제를 같은 staged grant 상태에서 수행한다.
3. 장착 장비/파편은 명시 확인을 요구한다. 분쇄 효과는 기본 recipe와 **그 사본의 실제 투자액 환급**을 합산한다.
   누적 9를 지불했다면 환급은 4이며 각 결제 3마다 1씩 계산하지 않는다. 무료 강화 단계로 투자액을 합성하지 않는다.
   엔케팔린 투자비는 환급하지 않는다. recipe 자체의 기본 보상이 통화를 지급하는 경우와는 별개다.
4. 지급 성공 뒤 staged 자원·사본을 게시한다. 유물 분쇄는 handler 안에서 가루 지급 후 사본 제거·조회가 이어지지만,
   그 전체는 outer application draft 안에 있다. 후속 실패 가능성만으로 부분 공개 결함으로 판단하지 않는다.
5. 마지막 공개 command 검증은 `reconcile_run_resources`와 `validate_profile_active_ownership` 등을 거친 뒤
   Continue를 갱신한다. vault와 active의 장비/파편 합집합에 같은 UUID가 있으면 후보 전체를 거절한다.
   이 검사는 dormant 사본과 다른 사본 종류 사이의 충돌도 포함한다.

근거: [정비](../../src/game/world/maintenance.rs), [장비 연결 전이](../../src/game/resources/equipment_loadout.rs),
[분쇄 자격](../../src/game/world/helpers.rs), [유물 분쇄](../../src/game/world/relic_runtime.rs),
[공개 후보 검증](../../src/game/world/application.rs), [grant 후보](../../src/game/world/grant_state.rs).

### 귀환·실패·다음 출발

- `ReturnFromExpedition`은 귀환 자격, 중복/0 수량, 소지량, 허용 재료 종류, 회수 한도와 창고 overflow를
  먼저 검사한다. 선택 재료의 새 창고 잔액을 준비하고 성공 귀환을 처리한 뒤 반영한다. 같은 명령의 outer draft에 속한다.
- `settle_expedition_terminal(true)`는 현재 inventory·fragment에 **남아 있는 사본**을 열거한다.
  판매·분쇄한 사본을 구매 이력이나 도감에서 재생성하는 경로는 없다. vault/active 소유권과 원정 instance가 맞아야 한다.
- 성공 귀환 시 일반 무기는 강화 0으로 `VaultEquipment`에 변환한다. 이 형식에는 무기 투자 원장을 저장하지 않으며
  재대여의 `OwnedEquipment::new`도 빈 투자 원장을 만든다. 파편은 form과 `investment`를 그대로 이동한다.
  실패/포기는 active 자산을 비우되 대여하지 않은 vault 자산·미사용 창고 재료를 유지한다.
- terminal은 active marker, 원정 창고 소비 원장, 직원·inventory·파편·유물·crafting 자원,
  checkpoint·Continue·전투 결과 전달 상태를 정리한다. node/phase의 정리는 호출 owner가 처리한다.
  `settle_expedition_terminal` 하나를 모든 terminal 전이의 독립 API로 간주하지 않는다.
- 대여는 중복 없는 합계 0–2사본을 먼저 확인하고 vault에서 제거해 같은 UUID로 active에 넣는다.
  중간 오류가 나면 공개 출발 명령 전체가 commit되지 않는다. 새 출발은 무조건 기본 예산을 새로 설정한다.
  terminal의 통화 필드를 즉시 0으로 만든다는 주장은 하지 않는다. 확인한 계약은 vault 보관/다음 원정 잔액 계승이 없다는 것이다.
- 성공 귀환 뒤 active marker가 닫히므로 같은 귀환 명령을 반복해 다시 적립할 수 없다. first-return story는 기존
  별도 receipt로 관리한다. 저장 호스트의 원자적 게시·재시작 복구 의미는 S01/S04 근거를 재사용한다.

근거: [출발 명령](../../src/game/world.rs)의 `handle_select_starter_employees`,
[반납·대여](../../src/game/world/expedition_lifecycle.rs), [vault 변환·검증](../../src/game/expedition_lifecycle.rs).

## 3 구조 평가와 후보 판정

| 기준 | 대표 흐름에서 확인한 사실·판정 |
| --- | --- |
| 책임·도메인 경계 | 활동 owner가 취득/소비 사유를 결정하고 grant·material·loadout·lifecycle owner에 위임. 유지 |
| 상태 소유권 | 정의 ID와 사본 UUID 구분, vault→active→vault 실제 이동, 재료별 단일 잔액과 실제 투자 원장 확인 |
| 전이·수명 | 무기 투자 초기화와 파편 투자 보존, 유물의 원정 수명, 미사용 vault 보존이 서로 다른 정책 |
| 실패·commit | 국소 후보와 outer draft의 범위가 다름. 지급 충돌·환급 overflow·대여 거절을 전체 명령이 보호 |
| 규칙 중복 | 환급은 `MaterialInvestment::refunds`, 출처 결제는 공통 helper 사용. 현재 recipe 가격으로 환급을 다시 계산하지 않음 |
| 의존·공개 범위 | 사본/경제 연산에 host I/O 의존 없음. 공개 command 마지막 소유권 검증이 임시 후보의 충돌을 차단 |
| 변경 용이성 | 장비/파편 정책 차이를 보존하며 공통 지급·투자 helper 사용. 단일 asset registry 필요 미확인 |
| 데이터·외부 계약 | vault는 exact identity와 필요한 지속 상태만 저장. dormant 정의 보존과 현재 정의 대여 자격 구분 |
| 결정론·복원 | instance별 UUID와 사본 투자 원장을 보존. 창고 소비 차이 복원은 S04 재사용, 이번 반복 SavePoint 테스트로 결합 확인 |
| 오류·관측 | preview와 실행의 환급/자격 일치, 취소·미확인·overflow 후 원본 보존 확인. telemetry 별도 변경 없음 |
| 검증 가능성 | 기존 내부 setup 테스트로 소유·환급 경계를 관찰하고 공개 application에서 포기·재출발 확인. 도달성 한계는 아래 명시 |
| 성능·자원 | vault/active 합집합 검사와 staged 복사가 존재. 이번 비용 측정 없음. 성장 한도나 최적화 필요성을 테스트 통과만으로 결론 내리지 않음 |

**선택한 대표 흐름은 유지한다.** 확정한 실행 결함이나 이번 범위에서 착수할 구조 변경 후보는 없다.
active/vault를 공통 영구 asset 저장소로 합치거나 모든 지급·반환을 새 transaction 추상화로 교체하는 안은 선택하지 않는다.
이미 공통 helper가 있고, 반출 시 초기화·보존·소실의 정책 차이를 없애면 의미가 바뀐다.

재검토 조건은 정상 경로의 소유권 불일치 재현, 새 지급/소비 기능에서 실제 중복 규칙 발생,
지원 규모에서 복사·소유권 검사 비용 초과가 확인되는 경우다. 그때 보존 의미와 작은 변경 범위를 먼저 정한다.
이번 단계에는 구현 후 완료 감사를 따로 만들지 않고 조사·검증을 확정한 뒤 데이터 게시 경계로 이동한다.

## 4 검증과 한계

선택한 소스·계약·build·테스트 입력 32개는 [baseline.json](evidence/s05_001/baseline.json),
명령·환경은 [commands.json](evidence/s05_001/commands.json)에 기록했다. 전체 의존성 freeze는 아니다.
최종 입력·문서 확인은 [verification.json](evidence/s05_001/verification.json)에 기록한다.

| 실제 실행 | 결과 | 시간·근거 |
| --- | --- | --- |
| Core lib: U12 대여/반환/충돌 8, U15 결제/환급/반납 6, R07 사본 지급/소비 2, 유물 분쇄 2 | 18 통과, 실패·무시 0 | build 38.16초, test 6.74초. [core001.log](evidence/s05_001/core001.log) |
| 공개 application: 포기 → 새 원정 → 과거 사본 거절·새 Continue 복원 | 1 통과, 실패·무시 0 | build 31.25초, test 10.08초. [public001.log](evidence/s05_001/public001.log) |

합계 **19개 통과, 실패·무시 0**이다. 테스트 내부의 대여 수량·교차 종류·반복 복원 입력은 별도 테스트 수로 세지 않았다.
기존 Core 경고 33개(lib test)·36개(lib)는 남아 있다.

- U12는 profile/vault·map depth를 내부에서 구성하고 `GameCore::execute`를 호출한다.
  helper 이름 `core_command`는 JSON을 `PlayerBehavior`로 변환하는 것이며 `CoreApplication` 통합 테스트가 아니다.
  future UUID와 dormant vault 충돌을 만든 테스트는 방어 경계의 근거이고 자연 발생 충돌 재현이 아니다.
- U15는 유효 custom recipe·정의, 입장 전 재료·창고 준비와 강제 node 구성을 사용한다.
  실제 결제 9→환급 4, 파츠 5→환급 2 같은 독립 기대값을 검증한다. 무료 강화·상한값은 일부 내부 setup이다.
  반환 후 recipe 가격을 바꾼 다음 generation에서 재대여·분쇄해 과거 실제 투자액 보존을 확인한다.
- R07은 동일 정의의 다른 사본과 잘못된 후속 grant를 사용해 정확한 소비·실패 후 UUID 보존을 검사한다.
  누락 정의를 넣은 batch 실패는 내부 fault fixture이며 현재 유효 content의 오류를 재현한 것이 아니다.
- 유물 단위 테스트는 admin 자원 지급과 내부 정비 node 설정을 포함한다. 공개 U08 포기·재출발 테스트도
  external pack loader와 `CoreApplication`을 통과하지만 사전 사본·자원 지급에 admin을 사용한다.
- 판매·상점 취득과 미완료/완료 Continue는 S03/S06, 창고 소비의 Continue 복원은 S04의 기존 근거를 재사용한다.
  이전 테스트 수를 이번 실행 수에 더하지 않는다. 취득부터 성공 귀환·재대여까지 모든 단계를 한 번의
  공식 content 자연 플레이로 검증했다고 주장하지 않는다.
- 전체 suite/release/Unity, host crash·승인 A/B matrix, 모든 성장/장착·사망 경계, 큰 vault 성능은 미실행이다.

실행 종료 후 코드의 실제 소유자·거절 지점과 기록의 주장 범위를 다시 대조했다. 이는 같은 작성자의 조사 결과 검토이며
외부 독립 감사가 아니다. 제품 코드·계약·테스트를 변경하지 않았고 선택 입력 32개가 그대로임을 확인했다.

## 5 다음 재개 위치

다음은 **S02/S11 데이터 검증·게시·승인 대표 흐름 조사와 개선 계획**이다.
장비·파편·유물 정의가 실행에서 소비되기 전의 경계를 선택한다. 시작은
`game/data/content_pack/mod.rs::load` → `game/data/mod.rs::ValidatedGameData`와 내부 `ContentGeneration`이다.
실제 loader 검증·immutable generation 생성과 application/승인 host의 소비를 연결하되,
S01/S04에서 확인한 실행 중 generation·저장 identity 경계는 재사용한다.

유효 입력 게시, invalid input 거절, 기존 generation 유지, 후속 원정에 새 generation을 쓰는 경계를 먼저 확인한다.
실제 필요가 확인된 작성·Trial·승인 경로만 추가 추적하고 모든 제작 도구나 schema 전수 감사로 넓히지 않는다.
종료점은 대표 흐름의 조사·후보 판정·필요한 개선/검증 계획·진행 현황 갱신이다.
