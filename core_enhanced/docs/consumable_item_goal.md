# Consumable Item Goal

이 goal은 Safezone의 `Item Use` 장면에서 사용할 섭취 아이템 시스템을 game_core/game_server 계약에 추가하기 위한 구현 계획서다.

섭취 아이템은 더 쉬운 클리어를 위한 단순 도핑이 아니라, 다음 노드에서 직원 손실과 트라우마 리스크를 줄이는 준비 자원이다. 공격력 증가 같은 직접 전투력 상승 효과는 가장 희귀하게 두고, 기본 방향은 트라우마 감소, 전투불능 피해 완화, 1회 사망 방지 같은 리스크 완화로 둔다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/consumable_item/PLAN.md
docs/goals/consumable_item/EXPERIMENTS.md
docs/goals/consumable_item/EXPERIMENT_NOTES.md
```

각 파일의 역할:

- `PLAN.md`: 구현 순서, 현재 판단, 남은 체크리스트, 완료 조건을 기록한다.
- `EXPERIMENTS.md`: 시도한 구현 접근, 실패/성공 결과, 테스트 실패 원인과 해결을 기록한다.
- `EXPERIMENT_NOTES.md`: 작업 중 떠오른 의심, 정책 질문, 부채 후보, 사용자 확인이 필요한 내용을 시간순으로 기록한다.

이 파일들은 최종 정책 문서가 아니라 goal 실행 중의 작업 기억장치다. goal 완료 후 유지해야 할 내용만 `game_rulebook.md`와 `unity_core_contract.md`로 옮긴다.

## Engineering Principles

- 코드 수정은 장기적인 방향으로 한다. 임시방편, 최소한의 수정, 특정 UI만 겨우 맞추는 패치는 피한다.
- 처음에는 어떤 구조가 장기적인 방향인지 확실하지 않을 수 있다. 작은 단위로 trial and error를 수행하고, 실패한 접근과 이유를 `EXPERIMENTS.md`에 남긴다.
- 레거시는 과감하게 제거한다. 과거 아이템/아티팩트/장비 구조에 억지로 끼워 넣어 compatibility layer를 만들지 않는다.
- 문서를 무조건 신뢰하지 않는다. 실제 코드와 live RON/API를 읽으면서 더 나은 개선안이 있으면 근거를 기록하고 적용한다.
- 단, 게임 정책, Unity-facing DTO, RON schema, 서버 command 계약처럼 사용자 결정이 필요한 내용은 임의로 확정하지 않고 goal을 종료한 뒤 질문 목록을 보고한다.
- 새 추상화는 실제 반복이 확인된 뒤 만든다. 단순 enum/table/function으로 충분한 곳에 trait, strategy, resolver 계층을 먼저 만들지 않는다.
- 테스트는 내부 추상화 모양보다 실제 Safezone 사용 흐름과 Unity-facing 계약을 고정한다.

## Source Of Truth

우선 읽을 파일:

- `src/game/behavior.rs`: `ActionKind`, `PlayerBehavior`, `BehaviorResult`, `GameError`.
- `src/game/world.rs`: command dispatch.
- `src/game/managers/action_scheduler.rs`: state별 `allowed_actions`.
- `src/game/world/helpers.rs`: action validation과 payload validation.
- `src/game/world/maintenance.rs`: 장비/파편 loadout 처리와 현재 `UnEquipItem` 정책.
- `src/game/resources/inventory.rs`: inventory DTO, owned equipment/artifact 구조.
- `src/game/data/mod.rs`: `Item`, `ItemRef`, `ItemRegistry`.
- `src/game/data/artifact_data.rs`, `src/game/data/equipment_data.rs`, `src/game/data/shop_data.rs`: RON item data loading pattern.
- `src/game/world/snapshot.rs`: Unity-facing inventory/roster snapshot.
- `docs/game_rulebook.md`: Safezone, Safe Node, 섭취 아이템 정책.
- `docs/unity_core_contract.md`: command/result/snapshot 계약.
- `docs/unity_client_implementation_goal.md`: Safezone `Node Exploration`, `Item Use`, `Loadout` UX 계약.

필요하면 `../game_server`의 `/game` request/result mapping도 확인한다. game_server가 workspace 바깥에 있으면 현재 경로 기준 실제 위치를 확인하고, 필요한 경우 사용자 승인 없이 파괴적 명령을 사용하지 않는다.

## Current State

현재 코드 기준:

- `EquipItem`, `UnEquipItem`, `EquipSkillFragment`, `UnequipSkillFragment` command는 이미 있다.
- `UnEquipItem` command는 Safezone 준비 상태에서 `bound: false` 장비를 해제하고, `bound: true` 장비는 거부한다.
- `Item` enum과 item registry는 `Consumable`을 인식한다.
- `ConsumableMetadata`, `ConsumableDatabase`, owned consumable inventory, RON loading이 존재한다.
- `UseConsumableItem` command와 `ConsumableItemUsed` result가 존재한다.
- `ViewingMap`과 `NodeConfirm` 같은 Safezone 성격 준비 구간에서 `UseConsumableItem`이 allowed action으로 노출된다.
- `inventory.consumables`와 `roster.employees[*].active_consumable_modifier`가 snapshot에 노출된다.
- `active_consumable_modifier`는 직원 1명당 1개만 유지되며, 새 consumable 사용 시 기존 modifier를 덮어쓴다.
- consumable은 사망자가 아닌 직원에게 사용할 수 있다. 사망자는 거부한다.
- 현재 남은 부채는 직원별 effective deploy cost DTO, percent validation 확장, live 대표 아이템 보강, 문서 동기화다.
- Safezone 내부 장면 전환은 Unity UX 상태이므로 core command가 아니다.

## Objective

Safezone의 `Item Use` 장면에서 직원에게 섭취 아이템을 적용할 수 있게 한다.

완료 후 다음이 가능해야 한다.

1. RON/data에서 섭취 아이템을 정의한다.
2. 인벤토리 snapshot에서 섭취 아이템을 Unity가 구분해 표시한다.
3. `ViewingMap`과 `NodeConfirm` 같은 Safezone 성격 준비 구간에서 `UseConsumableItem`이 allowed action으로 노출된다.
4. Unity가 `{ type: "use_consumable_item", item_uuid, target_employee_uuid }` command를 보내면 core가 검증하고 적용한다.
5. 적용된 섭취 버프는 직원 snapshot에 노출된다.
6. 섭취 버프는 다음 노드 또는 지정된 node count 동안 유지되고, node transition/result 시점에 duration이 감소한다.
7. 전투 중, 노드 결과 처리 중, Maintenance 내부 작업 중에는 섭취 아이템을 사용할 수 없다.
8. `UnEquipItem`은 Loadout 장면에 맞게 실제 해제 가능 정책으로 정리한다.

## Design Policy

섭취 아이템의 기본 역할:

- 클리어를 쉽게 만드는 공격 도핑보다 리스크 완화가 중심이다.
- `Common`, `Uncommon`은 손실 완화 중심이다.
- `Rare`, `Critical`은 다음 노드의 실패/사망 리스크를 크게 줄일 수 있다.
- 직접 공격력 증가와 사망 방지는 고가치 효과로 취급한다.
- `Forbidden`은 강한 효과와 부작용을 함께 가지는 후보지만, 부작용/침식/신뢰도 정책이 준비되기 전까지 live drop은 보류한다.

기본 사용 규칙:

- 사용 가능 시점: Safezone 성격의 `ViewingMap`과 `NodeConfirm`.
- 전투 중 사용: 금지.
- 노드 내부 사용: 금지. 단, 향후 특정 Safe Node가 사용 아이템을 다루도록 바꿀 경우 별도 정책으로 결정한다.
- 기본 대상: 직원 1명.
- 기본 지속: 다음 전투/보스 노드 1회 또는 `duration_combat_nodes` 기반.
- 지속 시간은 전투/보스 노드에서만 소모된다. Safe Node, Shop, Reward, HeadquartersContact, Maintenance에서는 duration이 감소하지 않는다.
- 기본 중첩: 직원 1명당 활성 섭취 효과는 1개다.
- 같은 직원에게 새 섭취 아이템을 사용하면 기존 효과를 덮어쓴다.
- 덮어쓰기 시 기존 효과는 즉시 제거되며 남은 duration, 효과량, 아이템은 환불하지 않는다.
- 사용 확정: Unity drag-and-drop이 성공하면 즉시 아이템을 소비하고 버프를 적용한다. `ConfirmEnterNode` 전 취소/되돌리기 정책은 구현하지 않는다. 되돌리기가 필요하면 별도 command와 UI 정책을 사용자와 의논한다.
- 섭취 아이템은 사망자가 아닌 직원에게 사용할 수 있다. 사망자인 직원에게는 사용할 수 없다.

효과 적용 우선순위:

1. `DeathPrevent`: 트라우마 임계 초과, 사망 판정, 장기 손실 판정 직전에 먼저 확인한다. 발동하면 해당 사망/손실을 1회 막고 효과를 즉시 소모한다.
2. `TraumaMitigation`: 사망 방지가 필요 없거나 이미 처리된 뒤 실제 증가할 트라우마 수치를 줄인다.
3. `RunHpLossMitigation`: 전투불능이나 실패 후 Run HP 감소량을 줄인다. 장기 생존 핵심은 트라우마이므로 트라우마 완화보다 후순위다.
4. `BattleHpSetup`: 전투 시작 시 Battle HP를 보정한다.
5. `DeployCostReduction`: 해당 직원의 배치 코스트를 줄인다. 전투 시작 안정화치 증가, 안정화치 회복량 증가, 재배치 비용 완화는 포함하지 않는다.
6. `DefenseMitigation`: 전투 중 받는 피해 감소, 방어력 증가를 처리한다.
7. `InitialSkillCharge`: 전투 시작/배치 시 해당 직원의 스킬 게이지(`resonance`)를 퍼센트 기반으로 충전한다. 효과 강도에 따라 희귀도가 달라진다.
8. `OffenseBoost`: 공격력 증가, 스킬 피해 증가를 처리한다. 단순 클리어 도핑이 되지 않도록 `Rare` 이상으로 제한한다.

`InitialSkillCharge` 기본 상한:

- `Common`: 최대 20%.
- `Uncommon`: 최대 40%.
- `Rare`: 최대 70%.
- `Critical`: 최대 100%.
- `Forbidden`: 최대 100%, 단 부작용 정책 확정 전까지 live pool 제외.

## Consumable Tier Table

초기 티어:

| Tier | 역할 | 기본 획득처 | 설계 기준 |
|---|---|---|---|
| `Common` | 작은 손상 완화 | 본사 보급, 저가 상점, 일반 보상 | 클리어를 쉽게 하기보다 손실을 줄임 |
| `Uncommon` | 특정 리스크 대응 | 본사 보급 상위, 상점, 전투 보상 | 다음 노드 위험을 보고 선택 |
| `Rare` | 실패 방지 보조 | 희귀 상점, 정예/보스 전 보상 | 전투 결과를 크게 흔들 수 있음 |
| `Critical` | 런 보존 장치 | 매우 희귀, 이벤트성, 고가 상점 | 사망/붕괴/보스전 실패 위험 완화 |
| `Forbidden` | 강력하지만 위험 | 특수 상점, 환상체/고위험 보상 | 강한 효과와 부작용을 함께 가짐 |

초기 아이템 후보:

| Tier | Item | 효과 방향 | 비고 |
|---|---|---|---|
| `Common` | `안정제 앰플` | 다음 노드 트라우마 획득 소폭 감소 | 기본 구호품 |
| `Common` | `응급 영양팩` | 다음 전투 Battle HP 소폭 증가 | Run HP 회복 아님 |
| `Common` | `진정 패치` | 전투불능 시 트라우마 증가량 소폭 감소 | 초반 손실 완화 |
| `Common` | `저농도 엔케팔린 정제` | 다음 전투 해당 직원 배치 코스트 소폭 감소 | 배치 부담 완화 |
| `Common` | `통증 억제제` | 다음 전투 받는 피해 소폭 감소 | 방어 계열 |
| `Uncommon` | `기억 고정제` | 트라우마 증가 1회 중폭 감소 | 고위험 노드 대비 |
| `Uncommon` | `응급 보호 혈청` | 전투불능 시 Run HP 감소량 감소 | 지속 피해 완화 |
| `Uncommon` | `공간 안정화 캡슐` | 다음 전투 해당 직원 배치 코스트 중폭 감소 | 배치 템포 보조 |
| `Uncommon` | `반응 지연제` | 첫 전투불능 트라우마 증가 지연/완화 | 사망 방지는 아님 |
| `Uncommon` | `정신 보호 테이프` | 위험 노드 반응/트라우마 완화 후보 | 신뢰도 연동 후보 |
| `Rare` | `관리자용 진정 주사` | 다음 노드 트라우마 획득 대폭 감소 | 강한 리스크 완화 |
| `Rare` | `전투 각성제` | 다음 전투 공격력 증가 | 직접 딜 증가라 희귀 |
| `Rare` | `격리 보호막 앰플` | 다음 전투 Battle HP 대폭 증가 | 핵심 직원 보호 |
| `Rare` | `응급 복귀 키트` | 전투불능 1회 시 Run HP 손실 크게 감소 | 사망 방지는 아님 |
| `Rare` | `안정화 촉매제` | 다음 전투 해당 직원 배치 코스트 대폭 감소 | 실시간 배치 보조 |
| `Critical` | `자아 고정 혈청` | 트라우마 임계 초과 사망 1회 방지 | 최고가치 생존템 |
| `Critical` | `블랙박스 보호 프로토콜` | 전투불능 1회 시 트라우마/Run HP 손실 대폭 감소 | 핵심 직원 보호 |
| `Critical` | `E.G.O 동조 촉진제` | 수동 스킬 준비/충전 보정 | 빌드 핵심 보조 |
| `Critical` | `관리자 특제 진정제` | 다음 노드 트라우마 획득 대부분 감소 | 보스/정예 전용급 |
| `Critical` | `완전 안정화 정제` | 다음 전투 핵심 직원 배치 코스트 크게 감소 | 배치 전략 확장 |
| `Forbidden` | `붕괴 촉진 앰플` | 공격력 대폭 증가 | 전투 후 트라우마 증가 후보 |
| `Forbidden` | `불완전 E.G.O 정제` | 스킬 위력/충전 대폭 증가 | 신뢰도/침식 부작용 후보 |
| `Forbidden` | `과잉 안정화 약물` | 해당 직원 배치 코스트를 극단적으로 낮춤 | 전투불능 시 손실 증가 후보 |
| `Forbidden` | `감각 차단제` | 피해/트라우마 감소 | 기억/신뢰도 페널티 후보 |
| `Forbidden` | `관리 금지 혈청` | 사망 방지 1회 + 공격 보정 | 큰 트라우마/장기 부작용 후보 |

초기 live 데이터 권장 범위:

- `Common` 5개.
- `Uncommon` 5개.
- `Rare` 3개.
- `Critical` 1~2개.
- `Forbidden`은 데이터 schema 검증용으로만 두거나 아예 live pool에서 제외한다.

획득처 기본 정책:

- `HeadquartersContact`: `Common`, 낮은 확률 `Uncommon`.
- `Shop`: `Uncommon`, 낮은 확률 `Rare`.
- 희귀 상점 또는 고위험 보상: `Rare`, 낮은 확률 `Critical`.
- `Forbidden`: live pool 제외. 데이터/문서 후보로만 유지한다.

## Required Contract Changes

### Action

새 `ActionKind`:

```rust
UseConsumableItem
```

새 `PlayerBehavior`:

```rust
UseConsumableItem {
    item_uuid: Uuid,
    target_employee_uuid: Uuid,
}
```

Unity JSON 예시:

```json
{
  "type": "command",
  "request_id": "use-consumable-1",
  "behavior": {
    "type": "use_consumable_item",
    "item_uuid": "owned-item-uuid",
    "target_employee_uuid": "employee-uuid"
  }
}
```

### Result

새 `BehaviorResult`:

```rust
ConsumableItemUsed {
    target_employee_uuid: Uuid,
    item_uuid: Uuid,
    applied_effects: Vec<...>,
    inventory_diff: InventoryDiffDto,
}
```

결과 DTO는 처음부터 과도하게 복잡하게 만들지 않는다. Unity가 결과 toast와 직원 상태 갱신을 보여줄 수 있을 정도만 제공한다. 자세한 계산 결과는 뒤따르는 `state_snapshot`을 source of truth로 둔다.

권장 result payload:

- `item_uuid`.
- `target_employee_uuid`.
- `replaced_modifier`: 덮어쓴 기존 효과가 있으면 요약 정보.
- `applied_modifier`: 새로 적용된 효과 요약 정보.
- `inventory_diff`.

Unity는 result로 즉시 toast/피드백을 보여줄 수 있지만, 최종 UI 상태는 항상 뒤따르는 `state_snapshot`을 기준으로 한다.

### Item Data

`Item` 계열에 `Consumable`을 추가한다.

권장 metadata:

```rust
ConsumableMetadata {
    id: String,
    uuid: Uuid,
    name: String,
    description: String,
    tier: ConsumableTier,
    price: u32,
    target_policy: ConsumableTargetPolicy,
    duration_policy: ConsumableDurationPolicy,
    effect: ConsumableEffect,
}
```

초기 enum 후보:

```rust
ConsumableTier {
    Common,
    Uncommon,
    Rare,
    Critical,
    Forbidden,
}

ConsumableTargetPolicy {
    SingleEmployee,
}

ConsumableDurationPolicy {
    NextCombatNode,
    CombatNodes(u32),
}
```

효과 표현은 처음부터 trait/strategy 계층을 만들지 않는다. `ConsumableEffect` enum과 작은 적용 함수/table로 충분한지 먼저 검증한다.

### Snapshot

Unity가 추론하지 않도록 다음을 노출한다.

- inventory item이 `consumable`인지 구분 가능해야 한다.
- consumable metadata에는 tier, target policy, duration, 설명용 effect summary가 있어야 한다.
- 직원 snapshot에는 현재 활성 `active_consumable_modifier`가 있어야 한다.
- active modifier에는 source item id/name, 남은 duration, 효과 요약이 있어야 한다.

권장 JSON 예시:

```json
{
  "inventory": {
    "items": [
      {
        "type": "consumable",
        "item_uuid": "...",
        "id": "stabilizer_ampoule",
        "name": "안정제 앰플",
        "tier": "common",
        "duration": { "type": "next_combat_node" },
        "effect_summary": "다음 노드 트라우마 획득 소폭 감소"
      }
    ]
  },
  "roster": {
    "employees": [
      {
        "uuid": "...",
        "active_consumable_modifier": {
          "source_item_uuid": "...",
          "definition_id": "stabilizing_ampoule",
          "name": "Stabilizing Ampoule",
          "remaining_combat_nodes": 1,
          "effect": { "TraumaMitigation": { "percent": 20 } }
        }
      }
    ]
  }
}
```

실제 JSON shape는 현재 snapshot 구조를 읽고 더 일관된 위치가 있으면 그쪽을 우선한다.

### Allowed Actions

`UseConsumableItem`을 허용할 상태:

- `ViewingMap`.
- `NodeConfirm`.

허용하지 않을 상태:

- `InBattle`.
- `CombatResult`.
- `InNode` 기본.
- `InShop`.
- `InReward`.
- `GameOver`, `RunComplete`, `RunFailed`.

`InNode`에서 사용 아이템이 필요해지는 정책은 별도 논의 전까지 추가하지 않는다.

## UnEquipItem Policy

`Loadout` 장면에서는 장비 장착/해제가 가능해야 한다.

현재 `UnEquipItem` command는 존재하지만 일반 해제를 거부한다. 최신 정책에서는 Safezone Loadout에서 장비 해제가 가능하다.

확정 정책:

- 일반 장비는 Safezone에서 해제 가능하다.
- 전투 중, 결과 처리 중, Maintenance 작업 결과를 보고 즉석 해제는 불가능하다.
- 귀속은 획득 인스턴스가 아니라 장비 종류 metadata에 붙는 개념이다.
- 같은 장비 id는 획득 경로에 따라 귀속/비귀속으로 갈라지지 않는다.
- metadata에 `bound: true`가 붙은 장비는 Safezone에서도 해제할 수 없다.
- metadata에 `bound: false`인 장비는 Safezone에서 자유롭게 해제 가능하다.
- 특수 이벤트로 귀속 장비가 필요하면 같은 장비의 instance flag를 추가하지 말고 별도 장비 id를 만든다.
- Unity-facing snapshot에는 최종 표시용 `can_unequip`과 필요 시 `cannot_unequip_reason`을 내려 Unity가 장비 metadata를 재계산하지 않게 한다.

## In Scope

- `Consumable` item data type과 RON loading.
- `UseConsumableItem` action/result/error.
- Safezone 상태에서만 사용 가능한 action gate.
- inventory snapshot에 consumable 노출.
- 직원 snapshot에 active consumable modifier 노출.
- consumable buff 적용과 duration 감소.
- `UnEquipItem`을 Safezone Loadout 정책에 맞게 실제 구현.
- `docs/game_rulebook.md`, `docs/unity_core_contract.md` 갱신.
- focused tests와 cargo check/test.

## Out Of Scope

- Unity 클라이언트 구현.
- 아이템 최종 수치 밸런싱.
- 최종 아이콘/아트/연출.
- 전투 중 포션 사용.
- 아이템 사용 취소/되돌리기.
- `Forbidden` 부작용, 침식 게이지, 신뢰도 페널티의 실제 적용.
- 아이템 제작 시스템.
- 여러 직원 대상 아이템.
- 직원 1명당 여러 consumable buff 슬롯.

## Implementation Plan

1. 현재 `Item`, `Inventory`, shop/reward, snapshot 구조를 읽고 `Consumable`을 어느 계층에 넣는 것이 가장 단순한지 결정한다.
2. `ConsumableMetadata`, `ConsumableTier`, duration/target/effect enum을 추가한다.
3. RON loading/data registry/shop item registry가 consumable을 인식하게 한다.
4. inventory DTO와 snapshot에 consumable item shape를 추가한다.
5. 직원 상태에 active_consumable_modifier 저장 위치를 추가한다.
6. `ActionKind::UseConsumableItem`, `PlayerBehavior::UseConsumableItem`, `BehaviorResult::ConsumableItemUsed`를 추가한다.
7. `ActionScheduler`와 validator에서 `ViewingMap`/`NodeConfirm`만 허용한다.
8. `GameCore`에 `handle_use_consumable_item`을 추가한다.
9. 소비 시 inventory에서 item을 제거하고 직원 active_consumable_modifier에 적용한다.
10. 직원 1명당 active_consumable_modifier 1개와 덮어쓰기 정책을 검증한다.
11. 전투/보스 노드 완료 시 duration 감소 source of truth를 한 곳으로 정한다. Safe Node 완료로는 duration을 줄이지 않는다.
12. 전투 시작 시 active_consumable_modifier가 BattleScenario/unit draft 또는 battle setup에 필요한 보정을 반영하게 한다.
13. 전투불능, 트라우마 증가, Run HP 감소, 배치 코스트 감소, 공격력 증가 같은 효과 적용 위치를 현재 코드에서 확인해 각 효과를 올바른 source of truth에 연결한다.
14. `UnEquipItem`의 일반 해제 정책을 Safezone Loadout 기준으로 구현한다.
15. `docs/game_rulebook.md`, `docs/unity_core_contract.md`를 실제 JSON/command shape에 맞게 갱신한다.
16. focused tests를 작성하고 전체 검증을 수행한다.

## Test Requirements

최소 focused tests:

- `UseConsumableItem`은 `ViewingMap`/`NodeConfirm`에서 allowed action에 포함된다.
- `UseConsumableItem`은 `InBattle`, `CombatResult`, `InNode`에서 거부된다.
- consumable 사용 시 inventory에서 해당 owned item이 제거된다.
- consumable 사용 시 대상 직원 active_consumable_modifier가 snapshot에 노출된다.
- 사망자인 직원에게 consumable 사용이 거부된다. 사망자가 아닌 직원은 전투 배치 가능 여부와 별개로 사용할 수 있다.
- 직원 1명당 active_consumable_modifier는 1개만 유지되며, 새 consumable 사용 시 기존 modifier가 덮어써진다.
- 덮어쓰기 시 기존 buff는 즉시 제거되고 환불되지 않는다.
- `NextCombatNode` duration buff는 다음 전투/보스 노드 해결 후 감소/제거된다.
- Safe Node, Shop, Reward, HeadquartersContact, Maintenance 완료로는 consumable duration이 감소하지 않는다.
- 트라우마 감소 consumable이 전투불능 또는 전투 결과 처리의 트라우마 증가량을 줄인다.
- Battle HP 증가 consumable이 전투 시작 Battle HP에 반영된다.
- 사망 방지 consumable은 트라우마 임계 초과 사망 1회를 막고 소비된다.
- 공격력 증가 consumable은 data tier상 rare 이상 fixture에만 존재한다.
- result는 toast에 필요한 최소 정보만 반환하고, 최종 상태는 뒤따르는 snapshot과 일치한다.
- `UnEquipItem`은 Safezone 준비 상태에서 `bound: false` 장비를 해제하고 inventory/snapshot을 갱신한다.
- `UnEquipItem`은 `bound: true` 장비를 거부하고 snapshot/result에서 해제 불가 사유를 표현할 수 있다.
- Maintenance 전용 작업은 Safezone에서 노출되지 않는다.

검증 순서:

```text
cargo test -p game_core consumable -- --nocapture
cargo test -p game_core use_consumable -- --nocapture
cargo test -p game_core unequip_item -- --nocapture
cargo test -p game_core -- --nocapture
cargo check -p game_core
```

game_server mapping을 변경했다면 추가로:

```text
cargo test -p game_server consumable -- --nocapture
cargo check -p game_server
```

## Stop Conditions

아래 상황에서는 임의 확정하지 말고 goal을 종료하고 질문 목록을 보고한다.

- 섭취 아이템 사용 확정 시점을 “드롭 성공 즉시 소모”가 아니라 “ConfirmEnterNode 후 소모”로 바꿔야 한다는 정책 충돌이 생긴다.
- duration 감소 시점이 전투/보스 노드 완료 외의 시점이어야 한다는 정책 판단이 필요하다.
- 직원 1명당 active_consumable_modifier 1개와 덮어쓰기 허용 정책이 게임 디자인상 부적절해 보인다.
- 사망 방지 효과가 기존 사망/트라우마 처리 흐름과 충돌해 우선순위 정책이 필요하다.
- consumable을 `Artifact`에 얹는 편이 나은지, 독립 `Consumable` 타입이 나은지 코드 구조상 큰 선택지가 생긴다.
- 장비 귀속을 metadata가 아니라 owned instance에 둬야 할 명확한 코드/정책 이유가 발견된다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Completion Criteria

1. `Consumable` item data와 RON loading이 추가된다.
2. `UseConsumableItem` command/result가 core와 game_server 계약에 연결된다.
3. Safezone 준비 상태에서만 consumable use가 허용된다.
4. inventory와 roster snapshot에서 Unity가 consumable과 active_consumable_modifier를 추론 없이 표시할 수 있다.
5. 트라우마 감소, Battle HP 증가, 사망 방지, 배치 코스트 감소, 시작 스킬 게이지 충전, 공격력 증가의 대표 효과가 테스트된다.
6. `UnEquipItem`이 Safezone Loadout 정책에 맞게 동작한다.
7. `docs/game_rulebook.md`, `docs/unity_core_contract.md`가 최신 계약과 정책으로 갱신된다.
8. focused tests 후 `cargo test -p game_core -- --nocapture`, `cargo check -p game_core`가 통과한다.
9. game_server를 수정했다면 `cargo test -p game_server -- --nocapture`, `cargo check -p game_server`가 통과한다.
10. 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Goal Command

```text
/goal docs/consumable_item_goal.md를 기준으로, Safezone의 Item Use 장면에서 직원에게 섭취 아이템을 사용할 수 있도록 game_core/game_server 계약과 데이터를 구현하라.
먼저 src/game/behavior.rs, src/game/world.rs, src/game/managers/action_scheduler.rs, src/game/world/helpers.rs, src/game/world/maintenance.rs, src/game/resources/inventory.rs, src/game/data/mod.rs, shop/reward data loading, src/game/world/snapshot.rs, docs/game_rulebook.md, docs/unity_core_contract.md를 꼼꼼히 읽고 현재 아이템/인벤토리/allowed_actions/source of truth를 확인하라.
섭취 아이템은 단순 클리어 도핑이 아니라 리스크 완화 준비 자원으로 설계하고, Common/Uncommon은 트라우마/전투불능/피해 완화 중심, Rare/Critical은 강한 방지 효과와 제한적 공격력 증가, Forbidden은 부작용 정책 전까지 live pool 보류로 처리하라.
Item enum/data registry/RON loading에 Consumable을 추가하고, UseConsumableItem ActionKind/PlayerBehavior/BehaviorResult를 추가하라. 사용 payload는 item_uuid와 target_employee_uuid를 기본으로 하고, ViewingMap과 NodeConfirm에서만 allowed action으로 노출하라. InBattle, CombatResult, InNode, Shop, Reward에서는 사용을 거부하라.
Unity drag-and-drop 성공 즉시 owned consumable을 inventory에서 제거하고 대상 직원 active_consumable_modifier에 적용하라. consumable은 사망자가 아닌 직원에게 사용할 수 있으며, 사망자인 직원에게는 거부하라.
직원 1명당 active_consumable_modifier는 1개만 유지하며, 새 consumable 사용 시 기존 modifier를 덮어쓴다. 덮어쓴 기존 modifier의 남은 duration/효과량/아이템은 환불하지 마라.
효과 타입은 단일 효과 중심으로 유지한다. `DeployCostReduction`은 배치 코스트 감소만 의미한다. `InitialSkillCharge`는 전투 시작/배치 시 스킬 게이지 충전만 의미한다. 최소 대표 효과로 트라우마 감소, Battle HP 증가, 트라우마 임계 초과 사망 1회 방지, rare 이상 공격력 증가, 배치 비용 감소, 초기 스킬 충전을 구현/테스트하라.
duration은 NextCombatNode 또는 CombatNodes(n) 데이터로 관리하고, 전투/보스 노드 완료 시에만 감소시키며 Safe Node, Shop, Reward, HeadquartersContact, Maintenance로는 감소시키지 마라.
inventory snapshot은 consumable item을 구분 가능하게 노출하고, roster snapshot은 active_consumable_modifier를 노출해 Unity가 추론하지 않게 하라. UseConsumableItem result는 item_uuid, target_employee_uuid, replaced_modifier, applied_modifier, inventory_diff 같은 toast용 최소 정보만 반환하고 최종 상태는 뒤따르는 state_snapshot을 source of truth로 둔다.
기존 UnEquipItem command는 Safezone Loadout 정책에 맞게 실제 장비 해제가 가능하도록 정리하라. 귀속은 owned instance가 아니라 장비 metadata의 bound flag로 표현하고, bound=false 장비는 Safezone에서 자유롭게 해제 가능하며 bound=true 장비는 해제 불가로 처리하라. 같은 장비 id가 획득 경로에 따라 귀속/비귀속으로 갈라지게 하지 말고, 특수 귀속 장비가 필요하면 별도 장비 id를 사용하라. Unity-facing snapshot에는 can_unequip과 필요 시 cannot_unequip_reason을 노출하라.
획득처 기본 정책은 HeadquartersContact에서 Common 및 낮은 확률 Uncommon, Shop에서 Uncommon 및 낮은 확률 Rare, 희귀 상점/고위험 보상에서 Rare 및 낮은 확률 Critical, Forbidden은 live pool 제외다.
Safezone 내부 장면 전환(Node Exploration/Item Use/Loadout)은 Unity UX 상태이므로 core command로 만들지 마라. 문서를 무조건 신뢰하지 말고 실제 코드와 live RON/API를 기준으로 더 단순하고 안전한 개선안이 있으면 근거를 기록하고 적용하라. 레거시는 compatibility layer로 감싸지 말고 제거하라.
작업 중 docs/goals/consumable_item/PLAN.md, EXPERIMENTS.md, EXPERIMENT_NOTES.md를 만들고 계획/실험/판단을 기록하라.
검증은 focused consumable/use_consumable/unequip_item tests 후 cargo test -p game_core -- --nocapture, cargo check -p game_core 순서로 수행하고, game_server mapping을 수정했다면 cargo test -p game_server -- --nocapture, cargo check -p game_server도 수행하라.
완료 조건은 1) Consumable data/RON loading 추가, 2) UseConsumableItem core/server 계약 연결, 3) Safezone 준비 상태에서만 사용 허용, 4) snapshot에서 consumable과 active_consumable_modifier 노출, 5) 대표 효과 테스트 통과, 6) UnEquipItem Loadout 정책과 metadata bound 정책 정리, 7) docs/game_rulebook.md와 Unity/core 계약 문서 갱신, 8) 검증 명령 통과, 9) 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료함이다.
```
