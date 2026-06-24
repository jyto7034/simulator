# 보상/상점/지원 노드 시스템 리팩토링 감사

- 컴포넌트: 보상/상점/지원 노드 시스템
- 기준 문서: `docs/refactor_preparation_plan.md`
- 상태: Initial audit documented

## 읽은 범위

- Runtime code:
  - `src/game/reward.rs`
  - `src/game/reward_policy.rs`
  - `src/game/world/reward.rs`
  - `src/game/world/shop.rs`
  - `src/game/world/support.rs`
  - `src/game/world/headquarters.rs`
  - `src/game/world/map_content.rs`
  - `src/game/world/node_flow.rs`
  - `src/game/world/helpers.rs`
  - `src/game/resources/selection.rs`
  - `src/game/resources/inventory.rs`
  - `src/game/skill_fragment.rs`
- Data/live data:
  - `src/game/data/reward_data.rs`
  - `src/game/data/shop_data.rs`
  - `src/game/data/validation.rs`
  - `../game_resources/data/events/rewards/base.ron`
  - `../game_resources/data/events/rewards/legacy_random_event_rewards.ron`
  - `../game_resources/data/events/shops/base.ron`
  - `../game_resources/data/map/node_definitions.ron`
  - `../game_resources/data/pve/encounters.ron`
- Tests/contracts:
  - `src/game/world/tests/map_flow.rs`
  - `src/game/world/tests/node_sessions.rs`
  - `src/game/world/tests/support.rs`
  - `tests/ron_loading.rs`
  - `docs/game_rulebook.md`
  - `docs/refactor_preparation_plan.md`
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`

## 현재 구조 요약

Reward는 `RewardOption.effects`를 `RewardExecutor`가 순차 적용한다(`src/game/reward.rs:83`). `world/reward.rs`는 현재 reward session의 선택 모드에 따라 적용할 reward option을 고르고, `RewardGranted { enkephalin, inventory_diff }`를 반환한다(`src/game/world/reward.rs:54`, `src/game/world/reward.rs:48`). 전투 종료 보상은 같은 `apply_reward_session()`을 호출하지만, 경험치 effect는 별도로 먼저 합산해 battle participant에게 지급한다(`src/game/world/combat.rs:338`, `src/game/world/combat.rs:1374`).

Shop은 `ShopSessionState`의 `visible_items`/`hidden_items`를 들고 있고, 구매/판매/리롤은 `world/shop.rs`에서 처리한다(`src/game/resources/selection.rs:20`, `src/game/world/shop.rs:46`). 구매는 global `game_data.item(item_uuid)`를 통해 표시 아이템을 다시 해석하고, 인벤토리 추가와 `InventoryDiffDto` 생성을 직접 수행한다(`src/game/world/shop.rs:65`, `src/game/world/shop.rs:94`). RewardExecutor도 item reward에 대해 유사한 인벤토리 추가와 diff 생성을 직접 수행한다(`src/game/reward.rs:271`).

Support는 현재 `Medical`과 `Rest`만 `SupportState`를 사용한다. Maintenance는 독립 node로 분리되어 있다. 이 구조는 `docs/game_rulebook.md:382`와 Unity 계약의 `Support Nodes` 기준에 맞다(`/mnt/f/unity projects/ark/docs/unity_core_contract.md:1378`). Medical은 대상 선택과 치료 선택 뒤 `complete_node`에서 적용되고, Rest는 `complete_node`에서 살아있는 직원 전체에게 trauma 회복을 적용한다(`src/game/world/support.rs:577`, `src/game/world/support.rs:622`, `src/game/world/support.rs:669`).

Research delivery는 Safe Node 진입 때 `deliver_research_if_safe_node()`가 처리하고, Support/Shop/Reward/HeadquartersContact state result에 `research_deliveries`를 실어준다(`src/game/world/node_flow.rs:105`, `src/game/world/map_content.rs:221`, `src/game/world/map_content.rs:263`). 정책 문서도 완료된 연구는 전투 노드에서 즉시 지급하지 않고 Safe Node에서 자동 수령한다고 명시한다(`docs/game_rulebook.md:321`, `/mnt/f/unity projects/ark/docs/unity_core_contract.md:2166`).

## Source-of-truth 판단

- Reward definition source of truth는 `GameDataBase.reward_data`와 `RewardOption.effects`다.
- Reward claim runtime source of truth는 `RewardExecutor`와 `GameCore::apply_reward_session()` 조합이다.
- Combat experience reward만은 `RewardExecutor`가 아니라 `world/combat.rs`가 별도 source처럼 effect를 해석한다.
- Shop stock source of truth는 active `ShopSessionState.visible_items`/`hidden_items`다. Snapshot의 full item objects와 uuid lists는 Unity-facing projection이므로 canonical duplication은 아니다.
- Support node state source of truth는 `SupportSessionState`와 `apply_current_support_node_effect()`다.
- Research completion source of truth는 `SkillFragmentInventory.progress`와 `pending_research_deliveries`다. `research_deliveries` result는 Safe Node 진입 때 발생한 projection이다.

## 리팩토링 후보

### 1. `GrantExperience` effect의 실행 source가 RewardExecutor와 전투 종료 경로로 갈라져 있다

`RewardEffect::GrantExperience`는 reward effect enum에 포함되어 있고 live reward data에도 존재한다(`src/game/reward.rs:23`, `../game_resources/data/events/rewards/base.ron:29`). 하지만 `RewardExecutor::grant_effect_with_state()`는 이 effect를 실제 지급하지 않고 로그만 남긴다(`src/game/reward.rs:141`). 실제 경험치 지급은 전투 종료 처리에서 reward session을 다시 순회해 `GrantExperience` 합계를 구한 뒤 participant result에 따라 살아남은 직원에게 지급한다(`src/game/world/combat.rs:338`, `src/game/world/combat.rs:366`, `src/game/world/combat.rs:1374`).

현재 map reward node는 runtime에서 `Experience` tag reward를 필터링한다(`src/game/world/map_content.rs:172`). 그래서 일반 reward node에서 경험치 reward가 우연히 뜨는 경로는 막혀 있지만, effect의 이름과 executor 책임만 보면 "보상 effect는 executor가 지급한다"는 직관과 어긋난다.

판단: source-of-truth 후보. 경험치가 combat-only reward라면 effect 이름이나 executor result를 그 의미에 맞게 좁혀야 한다. 반대로 reward node도 경험치를 줄 수 있어야 한다면 RewardExecutor가 경험치 대상 정책을 반환하거나 적용해야 한다. 경험치 지급 대상과 시점은 보상/성장 정책이므로 사용자와 정책 논의 필요.

### 2. Reward 적용이 원자적이지 않다

`RewardExecutor::grant_reward_with_state()`는 effect를 순차 적용하면서 각 effect가 성공한 직후 runtime state를 변경한다(`src/game/reward.rs:95`). `GameCore::apply_reward_session()`도 reward option을 순서대로 적용한다(`src/game/world/reward.rs:68`). 뒤쪽 effect가 `InventoryFull`, missing static data, `ForbiddenAbnormalityGrant` 등으로 실패하면 앞쪽 `GrantEnkephalin`, material, item, skill fragment 변경은 이미 적용된 상태로 남을 수 있다(`src/game/reward.rs:131`, `src/game/reward.rs:187`, `src/game/reward.rs:262`).

Reward claim은 사용자-visible 보상 지급 시점이고, 실패/보상/소비 시점 변화에 해당한다. 따라서 이 문제는 단순 내부 정리보다 정책성이 있다.

판단: 높은 우선순위 리팩토링 후보. 먼저 전체 reward session을 preflight validation한 뒤 적용하거나, reward application transaction을 만들어 실패 시 partial grant가 남지 않게 해야 한다. 사용자와 정책 논의 필요.

### 3. Skill fragment reward와 research progress reward가 `InventoryDiffDto`/`RewardGranted` 표면에 드러나지 않는다

`InventoryDiffDto`는 item added/updated/removed와 equipment material stacks만 표현한다(`src/game/resources/inventory.rs:195`). `GrantSkillFragment`는 `SkillFragmentInventory`에 파편을 추가하지만 diff에는 아무것도 넣지 않는다(`src/game/reward.rs:226`). `GrantSkillFragmentResearch`도 research progress/pending delivery를 갱신하지만 `InventoryDiffDto`나 `RewardGranted` result에는 progress 변화가 없다(`src/game/reward.rs:239`, `src/game/world/reward.rs:48`).

스냅샷은 `inventory.skill_fragments`, `skill_fragment_progress`, `pending_research_deliveries`를 노출하므로 최종 state source는 존재한다(`src/game/world/snapshot.rs:423`). 또 pending research delivery는 Safe Node 진입 result에 따로 들어간다(`src/game/world/node_flow.rs:105`). 하지만 direct reward claim 순간의 toast/result 표면에서는 "무엇이 지급되었는가"를 Unity가 `RewardOption.effects`나 다음 snapshot diff로 재추론해야 할 수 있다.

판단: Unity-facing DTO 표면 후보. 파편/연구 보상을 `RewardGranted`의 별도 diff로 확장할지, snapshot만 공식 source로 둘지 결정해야 한다. 사용자와 정책 논의 필요.

### 4. Reward와 Shop의 item grant 로직이 중복된다

RewardExecutor는 item reward를 inventory에 추가하기 전에 `inventory.can_add_item()`을 검사하고, owned UUID를 선택한 뒤 `InventoryItemDto`를 만든다(`src/game/reward.rs:271`). Shop 구매도 가격/중복/수용량을 검사한 뒤 owned UUID를 선택하고 같은 종류의 diff를 만든다(`src/game/world/shop.rs:65`, `src/game/world/shop.rs:86`, `src/game/world/shop.rs:94`, `src/game/world/shop.rs:101`).

두 경로의 artifact UUID 정책은 표현이 다르다. Shop은 artifact 구매 시 `item.uuid()`를 owned UUID로 사용하지만, RewardExecutor는 artifact도 `_ => next_owned_equipment()` branch로 들어간다(`src/game/world/shop.rs:94`, `src/game/reward.rs:281`). 현재 `Inventory::add_item_owned()`와 DTO 생성이 artifact metadata UUID를 기준으로 처리해 실제 동작이 크게 깨지지는 않지만, 같은 item grant 정책이 두 곳에 흩어져 있는 신호다.

판단: source-of-truth 리팩토링 후보. `InventoryGrant` 또는 `ItemGrantService` 같은 작은 helper를 만들어 shop/reward/admin grant가 같은 capacity, artifact duplicate, owned UUID, diff 생성 규칙을 사용하게 한다.

### 5. `GrantEquipment { equipment_id: None }`는 전체 장비 DB에서 무작위 선택한다

`GrantEquipment { equipment_id: None }`는 reward pool이나 tag가 아니라 `game_data.equipment_data.items` 전체에서 seed 기반으로 하나를 고른다(`src/game/reward.rs:160`). Validation도 `equipment_id: None`은 허용하고 별도 authoring 제약을 걸지 않는다(`src/game/data/validation.rs:312`). Live reward data와 legacy reward data 모두 이 효과를 사용한다(`../game_resources/data/events/rewards/base.ron:40`, `../game_resources/data/events/rewards/legacy_random_event_rewards.ron:24`).

현재 장비 DB 전체가 보상 가능한 장비만 포함한다면 문제가 작다. 하지만 향후 bound, tutorial-only, generated-only, boss-only 장비가 같은 DB에 들어오면 reward authoring 의도와 runtime 결과가 갈라질 수 있다.

판단: 리팩토링 후보. 전체 DB fallback 대신 equipment reward pool/tag/rarity 같은 authoring source를 두거나, `None` 랜덤 장비 효과를 제거하고 명시 reward로 바꾸는 방향을 검토한다. 기존 콘텐츠 대체와 보상 밸런스에 걸리므로 사용자와 정책 논의 필요.

### 6. Map reward pool 검증과 runtime filtering 책임이 나뉜다

Map reward node는 pool에서 후보를 가져온 뒤 runtime에서 `Forbidden`과 `Experience` tag reward를 제외한다(`src/game/world/map_content.rs:140`, `src/game/world/map_content.rs:172`). Reward data validation은 reward reference와 effect target은 확인하지만, map reward pool이 runtime filter에 의해 비게 되는지, reward node에 부적합한 tag가 섞였는지까지는 별도 정책으로 검증하지 않는다(`src/game/data/validation.rs:244`). 반면 PVE encounter reward는 `CombatRewardPolicy`로 allowed/featured tag를 검증한다(`src/game/data/validation.rs:421`, `src/game/reward_policy.rs:54`).

판단: source-of-truth 후보. "어떤 reward가 map reward node에 들어갈 수 있는가"는 runtime filtering보다 data validation과 reward pool authoring 단계에서 실패시키는 편이 `docs/refactor_preparation_plan.md`의 fallback 제거 기준에 맞다.

### 7. Shop session과 shop metadata가 stock 조작 메서드를 중복 보유한다

`ShopMetadata`와 `ShopSessionState`가 모두 `remove_visible_item()`과 `reroll_items()`를 가진다(`src/game/data/shop_data.rs:33`, `src/game/resources/selection.rs:31`). Runtime은 session 쪽을 쓰고, data 쪽은 deserialization/test helper 성격에 가깝다. 현재 구현은 단순해서 위험은 작지만, 리롤 정책이 "swap" 이상으로 바뀌면 두 타입이 쉽게 벌어진다.

판단: 낮은 우선순위 리팩토링 후보. session-only 조작으로 좁히거나, visible/hidden stock 조작을 작은 shared value helper로 분리할 수 있다.

### 8. Medical trust event 이름과 실제 적용 조건이 어긋날 수 있다

Medical은 대상이 alive이고 HP 손실, trauma, injury 중 하나가 있으면 후보로 올린다(`src/game/world/support.rs:544`). 실제 치료 후에는 치료 종류나 실제 incapacitation 여부와 무관하게 `TrustEventKind::TreatedAfterIncapacitation`을 적용한다(`src/game/world/support.rs:661`). 즉 단순 trauma counseling이나 minor HP 회복도 "incapacitation 이후 치료" memory로 기록될 수 있다.

판단: 정책/명명 후보. trust memory를 의료 지원 전체로 넓힐지, 정말 전투 불능 이후 치료에만 한정할지 결정해야 한다. 사용자와 정책 논의 필요.

### 9. Enkephalin 증가 overflow 정책이 경로마다 다르다

Reward의 `GrantEnkephalin`은 `checked_add` 실패 시 `InvalidAction`을 반환한다(`src/game/reward.rs:131`). Shop sell도 `checked_add`를 사용한다(`src/game/world/shop.rs:143`). Headquarters emergency supplies는 `saturating_add`를 사용한다(`src/game/world/headquarters.rs:58`). 실제 overflow는 드물지만, currency source-of-truth 정책이 경로마다 다르다.

판단: 소형 source-of-truth 후보. Enkephalin 증감은 하나의 helper를 통해 checked/saturating/capped 정책을 통일하는 것이 좋다. 최대치/cap이 UX와 밸런스 의미를 갖는다면 사용자와 정책 논의 필요.

## 기존 기능 조합으로 단순화 가능한 후보

- Reward와 Shop의 item grant는 shared inventory grant helper로 합칠 수 있다.
- Map reward node의 runtime filtering은 reward pool validation으로 이동하면 fallback-style runtime 제거가 가능하다.
- Enkephalin 증감은 reward/shop/headquarters가 공통 currency helper를 쓰도록 단순화할 수 있다.
- Medical target eligibility와 treatment application은 pure evaluator를 두면 `target_candidates`, command validation, effect application이 같은 판단을 공유할 수 있다.

## 레거시/fallback/dual schema 제거 후보

- `ForbiddenAbnormalityGrant`는 legacy abnormality materialization을 막는 guard다(`src/game/reward.rs:262`). live data와 legacy reward data에도 남아 있다(`../game_resources/data/events/rewards/base.ron:51`, `../game_resources/data/events/rewards/legacy_random_event_rewards.ron:35`). 실제 live path에서 계속 필요한 guard인지 확인 후, 더 이상 정규 데이터에 둘 필요가 없다면 reward data에서 제거하고 validation failure로 바꾸는 편이 좋다. 기존 콘텐츠 삭제/대체에 해당하므로 사용자와 정책 논의 필요.
- `GrantEquipment { equipment_id: None }`는 전체 DB 랜덤 fallback처럼 동작한다. 명시 reward pool로 대체 가능하면 제거 후보로 본다.
- Shop snapshot의 `visible_items`와 `visible_item_uuids` 동시 노출은 Unity-facing projection이므로 제거 후보로 보지 않는다. canonical source는 `ShopSessionState.visible_items`다.

## 하지 않거나 보류한 항목

- Research delivery를 RewardExecutor 안에서 즉시 deliver하도록 합치지 않는다. 현재 정책은 research completion과 delivery를 분리하고, Safe Node 진입에서 자동 수령하도록 명시한다.
- Support와 Maintenance를 다시 하나의 `SupportState`로 합치지 않는다. 정책 문서와 Unity 계약 모두 Maintenance를 독립 node로 둔다.
- Reward/Shop/Support state result에 들어가는 `research_deliveries`는 snapshot duplication이 아니라 Safe Node 진입 시점의 presentation result로 본다.
- Shop visible/hidden item UUID와 display item object를 동시에 내려주는 것은 Unity-facing projection으로 본다. 이것만으로는 source-of-truth 중복이 아니다.

## 필요한 테스트와 검증 명령

구현 리팩토링 착수 시 필요한 focused test:

- `cargo test -p game_core world::tests::node_sessions`
- `cargo test -p game_core world::tests::map_flow`
- `cargo test -p game_core world::tests::support`
- `cargo test -p game_core ron_loading::`
- `cargo test -p game_core reward_policy::`

추가로 고정해야 할 테스트:

- reward claim 중 뒤쪽 effect 실패 시 앞쪽 effect가 남지 않는지, 또는 partial grant를 공식 정책으로 고정하는지 테스트.
- `GrantSkillFragment`와 `GrantSkillFragmentResearch` 후 Unity-facing result/snapshot에서 표시해야 하는 source가 명확한지 테스트.
- shop purchase와 reward item grant가 artifact/equipment/consumable의 UUID와 diff를 같은 규칙으로 생성하는지 테스트.
- map reward pool이 runtime filter 후 비지 않는지 live RON validation test.
- Medical treatment가 trust memory를 어떤 조건에서 남기는지 테스트.

이번 감사 단계에서는 코드 변경이 없으므로 Rust test는 실행하지 않았다.

## 사용자와 정책 논의 필요

- `GrantExperience`를 combat-only effect로 좁힐지, RewardExecutor가 경험치 지급 결과를 공식 처리해야 하는지: 사용자와 정책 논의 필요.
- reward claim 실패 시 partial grant를 허용할지, 전체 reward session을 원자적으로 처리할지: 사용자와 정책 논의 필요.
- `RewardGranted`에 skill fragment/research progress diff를 추가할지, snapshot만 공식 표시 source로 둘지: 사용자와 정책 논의 필요.
- `GrantEquipment { equipment_id: None }` 전체 DB 랜덤 장비 보상을 유지할지, 명시 reward pool로 대체할지: 사용자와 정책 논의 필요.
- `ForbiddenAbnormalityGrant`를 정규 reward data guard로 유지할지, live data에서 제거하고 validation failure로 바꿀지: 사용자와 정책 논의 필요.
- Medical trust event를 의료 지원 전체로 볼지, 실제 incapacitation 이후 치료로만 제한할지: 사용자와 정책 논의 필요.
- Enkephalin overflow/cap 정책을 checked error, saturating cap, explicit max cap 중 무엇으로 통일할지: 사용자와 정책 논의 필요.
