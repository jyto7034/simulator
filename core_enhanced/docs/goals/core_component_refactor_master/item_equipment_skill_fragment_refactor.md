# 아이템/장비/스킬 파편 경제 시스템 리팩토링 감사

- 컴포넌트: 아이템/장비/스킬 파편 경제 시스템
- 기준 문서: `docs/refactor_preparation_plan.md`
- 상태: Initial audit documented

## 읽은 범위

- Runtime code:
  - `src/game/resources/inventory.rs`
  - `src/game/resources/item_slot.rs`
  - `src/game/skill_fragment.rs`
  - `src/game/world/maintenance.rs`
  - `src/game/world/support.rs`
  - `src/game/world/helpers.rs`
  - `src/game/world/snapshot.rs`
  - `src/game/employee.rs`
- Data/live data:
  - `src/game/data/mod.rs`
  - `src/game/data/equipment_data.rs`
  - `src/game/data/skill_fragment_data.rs`
  - `../game_resources/data/equipments/base.ron`
  - `../game_resources/data/skill_fragments/base.ron`
- Tests/contracts:
  - `src/game/world/tests/equipment.rs`
  - `tests/ron_loading.rs`
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`

## 현재 구조 요약

`Item` enum은 현재 `Equipment`, `Artifact`, `Consumable`만 포함한다(`src/game/data/mod.rs:390`). 예전 문서에 있던 `Item::Abnormality` 인벤토리 진입 문제는 현재 코드 기준으로는 이미 제거된 상태다.

Inventory는 장비, 장비 재료, 소모품, 아티팩트를 별도 저장소로 나눈다(`src/game/resources/inventory.rs:255`). Equipment/Consumable은 owned instance UUID를 사용하고, Artifact는 `meta.uuid`를 그대로 소유 id처럼 사용하며 generic remove path에서 제거할 수 없다(`src/game/resources/inventory.rs:313`, `src/game/resources/inventory.rs:333`). 이 아티팩트 정책은 현재 주석과 테스트가 맞물려 있어 이전 TODO성 후보는 해소된 것으로 본다.

장비 장착 상태는 `Employee.loadout.item_slot`의 `EquippedRef`와 `OwnedEquipment.equipped_to`에 동시에 저장된다(`src/game/resources/item_slot.rs:15`, `src/game/resources/inventory.rs:523`). 장착/해제/조합/분쇄 handler가 두 상태를 수동으로 맞춘다(`src/game/world/maintenance.rs:85`, `src/game/world/maintenance.rs:201`, `src/game/world/maintenance.rs:502`).

스킬 파편은 run-wide `SkillFragmentInventory`에 stack/progress/pending research/dust를 저장하고, 직원별 `SkillFragmentLoadout`은 baseline fragment 목록과 active fragment id 하나만 저장한다(`src/game/skill_fragment.rs:19`, `src/game/skill_fragment.rs:700`). 장착 검증은 보유 여부와 현재 effective combat profile compatibility를 본다(`src/game/world/helpers.rs:326`, `src/game/world/helpers.rs:363`).

Maintenance는 Unity-facing `maintenance_options` typed operation preview를 제공한다(`src/game/behavior.rs:93`). Projection은 현재 `src/game/world/support.rs`에 있고, command handler는 `src/game/world/maintenance.rs`, payload validation은 `src/game/world/helpers.rs`에 나뉘어 있다(`src/game/world/support.rs:116`, `src/game/world/maintenance.rs:380`, `src/game/world/helpers.rs:453`). Unity 계약은 `maintenance_options.operations.*`를 비용/가능 여부/source of truth로 사용하라고 명시한다(`/mnt/f/unity projects/ark/docs/unity_core_contract.md:1564`).

## Source-of-truth 판단

- Inventory item ownership source of truth는 `Inventory`다.
- 장착 상태 source of truth는 현재 `ItemSlot`과 `OwnedEquipment.equipped_to`가 이중으로 나뉜다. 이것은 리팩토링 후보다.
- 전투용 장비 효과 source of truth는 `BattleUnitDraft.equipped_items`/`equipped_item_enhancements`와 `stat_pipeline`이다.
- 스킬 파편 보유/강화/개화 source of truth는 `SkillFragmentInventory`다. 직원별 active fragment는 `Employee.skill_fragments.active_fragment_id`에 저장된다.
- Maintenance UI 가능 여부 source of truth는 계약상 `maintenance_options`여야 하지만, 실제 command 가능 여부는 helpers validator가 다시 계산한다.

## 리팩토링 후보

### 1. 장비 장착 상태가 `ItemSlot`과 `OwnedEquipment.equipped_to`에 이중 저장된다

`ItemSlot`은 employee loadout 안에 장착 instance/base/type을 들고 있고, `OwnedEquipment`도 `equipped_to: Option<Uuid>`를 들고 있다(`src/game/resources/item_slot.rs:15`, `src/game/resources/inventory.rs:523`). `handle_equip_item()`은 item slot에 equip한 뒤 inventory item의 `equipped_to`를 세팅한다(`src/game/world/maintenance.rs:201`, `src/game/world/maintenance.rs:218`). `handle_unequip_item()`과 `handle_dismantle_equipment()`도 두 상태를 직접 갱신한다(`src/game/world/maintenance.rs:502`).

두 상태가 어긋나면 snapshot, validation, battle draft가 서로 다른 장착 상태를 볼 수 있다. 현재 테스트는 주요 경로를 덮지만, source-of-truth가 둘이라는 구조 자체는 리팩토링 대상이다.

판단: 높은 우선순위 source-of-truth 후보.

가능한 방향:

- `ItemSlot`만 장착 source of truth로 두고 `equipped_to`는 snapshot/projection에서 역산한다.
- 또는 `Inventory`에 장착 transaction helper를 만들어 item slot과 owned equipment를 한 함수에서만 변경하게 한다.

필요 검증:

- equip/unequip/combine/dismantle 후 `ItemSlot`과 `OwnedEquipment.equipped_to` invariant를 확인하는 focused test.
- battle draft 생성과 roster snapshot이 같은 장착 상태를 보는지 확인.

### 2. 스킬 파편 count와 직원 loadout의 소유 의미가 불명확하다

`SkillFragmentInventory.count(fragment_id) > 0`이면 어떤 직원이든 해당 active fragment를 장착할 수 있다(`src/game/world/helpers.rs:338`). 직원 loadout은 fragment id만 저장하고 특정 copy를 예약하지 않는다(`src/game/skill_fragment.rs:700`). 따라서 스킬 파편이 run-wide unlock이면 자연스럽지만, copy 수가 개별 장착 수를 뜻한다면 현재 구조는 source-of-truth가 없다.

이 애매함은 분쇄에서 더 드러난다. `handle_dismantle_skill_fragment()`는 해당 fragment를 active로 장착한 모든 직원을 먼저 unequip한 뒤 stack count를 1 줄인다(`src/game/world/maintenance.rs:418`). 테스트도 count가 2에서 1로 남는 경우에도 장착 직원이 unequip되는 것을 고정한다(`src/game/world/tests/equipment.rs:1291`).

판단: 높은 우선순위 정책 후보. 스킬 파편이 shared unlock인지 per-copy 장착 자원인지 결정해야 한다. 사용자와 정책 논의 필요.

가능한 방향:

- shared unlock이면 count는 "분쇄 가능한 여분/중복 수"로 명시하고, active loadout은 id 참조를 유지한다. 이 경우 count가 1 이상 남을 때 분쇄가 장착을 해제해야 하는지 재검토한다.
- per-copy 장착이면 loadout이 fragment copy/instance를 참조하거나, 장착 예약 수를 inventory가 추적해야 한다.

### 3. Maintenance preview/validation/handler가 같은 작업 규칙을 세 곳에서 계산한다

Maintenance option projection은 `support.rs`의 `maintenance_options()`와 helper preview 함수들이 계산한다(`src/game/world/support.rs:116`). Command validation은 `world/helpers.rs`에서 다시 계산하고, 실제 적용은 `world/maintenance.rs`에서 세 번째로 recipe/material/progress를 읽는다(`src/game/world/helpers.rs:453`, `src/game/world/maintenance.rs:380`).

현재 구조는 동작하지만, Unity-facing source of truth인 `maintenance_options.operations.*.can_execute`와 실제 command validator가 drift할 수 있다. 예를 들어 preview disabled reason은 여러 실패 원인을 `"not_enough_fragment_dust"` 또는 `"cannot_awaken_skill_fragment"`로 뭉뚱그리지만, validator는 missing static data, max level, insufficient material 등을 다른 에러로 낸다(`src/game/world/support.rs:433`, `src/game/world/support.rs:490`, `src/game/world/helpers.rs:453`).

판단: 리팩토링 후보. 각 operation별 pure evaluator를 만들고, preview/validation/handler가 같은 evaluator 결과를 사용하도록 합치는 것이 좋다.

필요 검증:

- maintenance_options의 `can_execute=false`인 작업 command가 동일한 이유 계열로 거부되는지 테스트.
- preview의 costs/gains/after가 command result/inventory diff와 일치하는지 테스트.

### 4. Maintenance projection이 `support.rs`에 있다

Maintenance는 `SupportState`가 아니라 독립 node로 계약에 명시되어 있다(`/mnt/f/unity projects/ark/docs/unity_core_contract.md:1456`). 하지만 `maintenance_options()`, material snapshot, item operation preview는 `src/game/world/support.rs`에 있다(`src/game/world/support.rs:104`, `src/game/world/support.rs:116`). 실제 command handlers는 `src/game/world/maintenance.rs`에 있다.

판단: 모듈 경계 리팩토링 후보. Maintenance projection/preview 코드는 `world/maintenance.rs` 또는 별도 `world/maintenance_projection.rs`로 이동하는 것이 장기 구조와 맞다.

### 5. `fragment_dust`/`equipment_dust` material id가 문자열로 흩어져 있다

Maintenance material snapshot과 preview는 `"fragment_dust"`와 `"equipment_dust"` 문자열을 직접 사용한다(`src/game/world/support.rs:167`, `src/game/world/support.rs:402`, `src/game/world/support.rs:463`). Snapshot도 `fragment_dust` top-level field를 별도로 가진다(`src/game/world/snapshot.rs:431`). Live equipment data에도 `equipment_dust` material id가 있다(`../game_resources/data/equipments/base.ron`).

판단: 소형 source-of-truth 후보. canonical constant를 두거나, dust를 material database와 같은 경로로 표현할지 결정해야 한다. `fragment_dust`는 equipment material DB에 없으므로 장비 재료와 같은 저장소로 합칠지는 정책 문제다. 사용자와 정책 논의 필요.

### 6. `SkillFragmentPolicy`의 dormant variant와 default-policy public methods

`SkillFragmentStackingPolicy::ConvertAdditionalCopiesToResource`는 존재하지만 현재 구현은 `NotImplemented`를 반환한다(`src/game/skill_fragment.rs:189`). `SkillFragmentInventory`에는 `add()`, `upgrade_with_dust()`, `dismantle()`처럼 `SkillFragmentPolicy::default()`를 쓰는 public convenience methods가 있고, runtime handler는 별도로 `self.state.skill_fragment_policy`를 넘긴다(`src/game/skill_fragment.rs:152`, `src/game/world/maintenance.rs:385`).

판단: 정책 surface 정리 후보. live runtime에서 쓰지 않는 stacking variant는 제거하거나 구현해야 한다. runtime 정책은 `GameCoreState.skill_fragment_policy`가 source of truth이므로, 공식 runtime path가 default convenience를 실수로 쓰지 않게 경계를 좁히는 것이 좋다. 사용자와 정책 논의 필요.

### 7. Bound 장비와 Maintenance dismantle 정책

Safezone unequip validator는 bound equipment를 해제할 수 없게 막는다(`src/game/world/helpers.rs:271`). Unity 계약도 bound 장비는 해제할 수 없다고 말한다(`/mnt/f/unity projects/ark/docs/unity_core_contract.md:1844`). 반면 Maintenance dismantle validator/handler는 bound 여부를 확인하지 않고, 장착 중 장비도 자동 해제 후 분쇄할 수 있게 한다(`src/game/world/helpers.rs:507`, `src/game/world/maintenance.rs:455`). 계약의 Maintenance 정책은 장착 중 장비 분쇄/자동 해제를 허용하지만, bound 장비가 예외인지 명시하지 않는다.

판단: 정책 후보. bound가 "일반 해제 금지"인지 "분쇄도 금지"인지 결정해야 한다. 사용자와 정책 논의 필요.

## 기존 기능 조합으로 단순화 가능한 후보

- Maintenance command validation은 preview evaluator를 재사용하면 UI와 command의 동일 정책을 한 source에서 만들 수 있다.
- 장착 상태는 `ItemSlot`에서 inventory projection을 파생하거나, 반대로 inventory transaction helper를 통해 양쪽 갱신을 한 함수로 캡슐화할 수 있다.
- 스킬 파편 분쇄는 "남은 count"와 "장착 상태"를 조합해 자동 unequip 여부를 계산해야 한다. 현재는 count가 남아도 일괄 unequip한다.

## 레거시/fallback/dual schema 제거 후보

- `Item::Abnormality`는 현재 제거되어 있으므로 추가 작업 없음.
- list-only legacy maintenance options는 현재 계약상 제거된 상태이며, typed `MaintenanceOptionsDto`가 사용된다. 유지 후보가 아니라 현재 구조의 좋은 점이다.
- Artifact generic remove TODO는 현재 `InventoryRemoveError::NotRemovableArtifact`와 테스트로 정책화되어 있어 제거 후보가 아니다.

## 하지 않거나 보류한 항목

- 장비/소모품/아티팩트를 하나의 owned item wrapper로 통합하지 않는다. 장비는 장착/강화, 소모품은 사용 즉시 제거와 active modifier, 아티팩트는 영구 귀속 효과라 수명주기가 다르다.
- active consumable modifier와 battle buff는 통합하지 않는다. `docs/refactor_preparation_plan.md`와 Unity 계약 모두 별도 수명주기를 전제한다.
- Maintenance 장착 교체 자체는 제거하지 않는다. 계약상 Maintenance 내부 장착 교체는 부가 기능으로 허용되어 있다.

## 필요한 테스트와 검증 명령

구현 리팩토링 착수 시 필요한 focused test:

- `cargo test -p game_core resources::inventory`
- `cargo test -p game_core resources::item_slot`
- `cargo test -p game_core skill_fragment::`
- `cargo test -p game_core world::tests::equipment`
- `cargo test -p game_core ron_loading::`

이번 감사 단계에서는 코드 변경이 없으므로 Rust test는 실행하지 않았다.

## 사용자와 정책 논의 필요

- 스킬 파편이 shared unlock인지, per-copy 장착 자원인지: 사용자와 정책 논의 필요.
- 스킬 파편 count가 2 이상 남을 때 분쇄가 장착 중인 직원을 자동 unequip해야 하는지: 사용자와 정책 논의 필요.
- `fragment_dust`를 독립 resource로 둘지, equipment material과 같은 material DB/schema로 합칠지: 사용자와 정책 논의 필요.
- `SkillFragmentStackingPolicy::ConvertAdditionalCopiesToResource`를 구현할지 제거할지: 사용자와 정책 논의 필요.
- bound 장비가 Maintenance dismantle 대상이 될 수 있는지: 사용자와 정책 논의 필요.
