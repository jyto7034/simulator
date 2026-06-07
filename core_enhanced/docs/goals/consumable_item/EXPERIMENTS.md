# Consumable Item Experiments

## 2026-06-02 Initial Reading

### Evidence

- `ActionKind`와 `PlayerBehavior`에는 `UseConsumableItem`이 없다.
- `InventoryItemDto`는 `Equipment`와 `Artifact`만 표현한다.
- `Item` enum은 `Equipment`, `Artifact`, `Abnormality`만 가진다.
- `UnEquipItem`은 command가 있으나 현재 구현은 `InvalidAction`으로 일반 해제를 거부한다.

### Decision

소모품을 `Artifact`에 얹지 않고 독립 `Consumable` 타입으로 추가한다. 섭취 아이템은 소모/대상/duration/active buff가 있으므로 아티팩트와 수명주기가 다르다. 기존 타입에 compatibility layer로 끼워 넣으면 Unity snapshot과 reward/shop 계약이 더 불명확해진다.

## 2026-06-02 Implementation Pass

### Successes

- `ConsumableDatabase`와 live `game_resources/data/consumables/base.ron`을 추가했다.
- `UseConsumableItem` action/result가 `ViewingMap`/`NodeConfirm`에서 동작한다.
- `inventory.consumables`와 `roster.employees[*].active_consumable_modifier` snapshot을 추가했다.
- 대표 효과 focused tests가 통과했다.
- `UnEquipItem`이 metadata `bound` 정책을 기준으로 실제 해제/거부된다.

### Issues and Fixes

- 기존 `EquipmentMetadata` Rust fixture들이 새 `bound`/`cannot_unequip_reason` 필드를 빠뜨려 컴파일이 실패했다.
  - 모든 fixture를 기본 비귀속 값으로 보정했다.
- 공격력 증가 테스트가 기본 starter skill fragment 적용 후 스탯을 고려하지 않았다.
  - 고정 수치 대신 baseline profile 대비 증가량을 검증하도록 수정했다.
- 첫 RON focused test 실행이 sandbox의 target lock read-only 오류로 실패했다.
  - 동일 명령을 승인된 cargo test 실행으로 재시도해 통과를 확인했다.
