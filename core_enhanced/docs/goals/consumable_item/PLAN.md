# Consumable Item Implementation Plan

## Objective

Safezone의 Item Use 장면에서 직원에게 섭취 아이템을 사용할 수 있도록 `game_core`와 필요 시 `game_server` 계약을 구현한다.

## Current Plan

1. 현재 아이템 source of truth를 확인한다.
   - `Item`/`ItemRegistry`/`GameDataBase`에 `Consumable`이 없음.
   - `Inventory`는 장비/장비 재료/아티팩트만 보관함.
   - `UseConsumableItem` action/result가 없음.
   - `UnEquipItem`은 command만 있고 일반 해제를 거부함.
2. `ConsumableMetadata`와 consumable database를 추가한다.
   - `Common`/`Uncommon`/`Rare`/`Critical`/`Forbidden`.
   - `NextCombatNode`/`CombatNodes(n)` duration.
   - 대표 효과: trauma mitigation, battle HP setup, death prevent, rare+ offense boost.
3. inventory와 reward/shop registry에 consumable을 연결한다.
   - owned consumable은 instance uuid를 가진다.
   - snapshot은 consumable을 별도 타입으로 노출한다.
4. employee active consumable buff를 추가한다.
   - 직원당 1개만 유지.
   - 새 아이템 사용 시 기존 buff는 환불 없이 덮어씀.
   - 전투/보스 노드 완료 시에만 duration 감소.
5. `UseConsumableItem` command를 추가한다.
   - `ViewingMap`/`NodeConfirm`에서만 allowed.
   - alive/available employee만 대상.
   - drag-and-drop 성공 즉시 inventory에서 제거.
6. `UnEquipItem`을 Safezone Loadout 정책에 맞게 구현한다.
   - 장비 metadata `bound` flag가 source of truth.
   - `bound=false` 장비는 Safezone에서 해제 가능.
   - `bound=true` 장비는 해제 불가 사유를 snapshot에 노출.
7. 문서와 테스트를 최신 계약으로 갱신한다.

## Implementation Notes

- `Consumable`은 `Artifact`에 얹지 않고 독립 `Item` variant로 추가했다.
- `ItemRegistry`가 consumable uuid도 조회하므로 Shop visible/hidden item과 Reward 지급이 같은 source of truth를 쓴다.
- 직원 active consumable modifier는 `Employee.active_consumable_modifier`에 저장한다.
- Battle HP/Offense는 `combat_profile_for_battle`에서 적용한다.
- Trauma/Run HP/DeathPrevent는 `apply_incapacitation`에서 적용한다.
- Duration 감소는 전투/보스 노드 처리 경계(`handle_complete_combat_result`, retreat node completion)에서만 호출한다.
- `UnEquipItem`은 `bound=false` 장비를 실제 해제하고, `bound=true` 장비는 `InvalidAction`으로 거부한다.

## Completion Checklist

- [x] Consumable data/RON loading 추가.
- [x] UseConsumableItem core 계약 추가.
- [x] Safezone 준비 상태에서만 사용 허용.
- [x] Inventory snapshot에 consumables 노출.
- [x] Roster snapshot에 active consumable buff 노출.
- [x] 대표 효과 테스트 추가 및 통과.
- [x] UnEquipItem bound 정책 구현.
- [x] `docs/game_rulebook.md`, `docs/unity_core_contract.md` 갱신.
- [ ] focused tests, `cargo test -p game_core -- --nocapture`, `cargo check -p game_core` 통과.

## Stop Conditions

- 사용자와 의논해야 할 게임 정책이 발견되면 임의 확정하지 않고 goal을 종료한다.
- live RON 또는 game_server 위치가 workspace 외부라 수정 승인이 필요한 경우, 코드 진행 가능한 범위를 먼저 끝낸 뒤 승인 필요 항목을 분리한다.
