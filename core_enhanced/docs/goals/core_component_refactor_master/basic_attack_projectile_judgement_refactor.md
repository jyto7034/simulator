# 기본 공격/투사체/공격 판정 시스템 리팩토링 감사

- 기준 문서: `docs/refactor_preparation_plan.md`
- 컴포넌트: 기본 공격/투사체/공격 판정 시스템
- 상태: Initial audit documented

## 읽은 범위

- Runtime code:
  - `src/game/battle/core/basic_attack.rs`
  - `src/game/battle/core/commands.rs`
  - `src/game/battle/core/targeting.rs`
  - `src/game/battle/core/target_usefulness.rs`
  - `src/game/battle/timeline.rs`
  - `src/game/battle/validation/attacks.rs`
  - `src/game/battle/validation/deaths.rs`
- Data/schema:
  - `src/game/data/abnormality_data.rs`
  - `src/game/data/equipment_data.rs`
  - `src/game/data/corroded_employee_data.rs`
  - `src/game/ability.rs`
  - `../game_resources/data/abnormalities/base.ron`
  - `../game_resources/data/equipments/base.ron`
  - `../game_resources/data/enemies/corroded_employees.ron`
- Policy/contract:
  - `docs/skill_target_contract.md`
  - `docs/refactor_preparation_plan.md`
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- Prior goal notes used only as historical context:
  - `docs/goals/basic_attack_lifecycle_refactor/*`
  - `docs/goals/basic_attack_projectile_target_only_collision/*`
  - `docs/goals/tile_based_attack_delivery_contract/*`

## 현재 구조 요약

기본 공격 lifecycle은 `src/game/battle/core/basic_attack.rs`가 담당한다. `try_start_pending_basic_attacks()`는 pending auto attack을 스캔하고, `handle_basic_attack_start_event()`는 readiness/action lock/reposition/target selection을 확인한 뒤 `AttackStart`를 기록하고 `AttackResolve`를 예약한다. `handle_basic_attack_resolve_event()`는 `AttackResolve`를 기록한 뒤 `resolve_basic_attack()`에 피해 또는 projectile launch를 위임하고, 실패하면 `AttackMiss`를 기록한다.

타겟 선택과 range eligibility는 `src/game/battle/core/targeting.rs`에 모여 있다. `BasicAttackRangePolicy`는 runtime `BasicAttackDef`에서 `delivery`, `range_policy`, resolved `TileRangePattern`, `air_capable`을 뽑아 만든다. `is_basic_attack_target_in_range()`는 `range_units`가 아니라 `TileRangePolicy::WholeFieldValidTiles` 또는 `TileRangePattern::contains_target_tile()`로 판정한다.

target usefulness는 `src/game/battle/core/target_usefulness.rs`가 담당한다. 기본 공격은 미리 계산한 최종 피해가 0보다 크거나, 공격자의 `OnAttack` trigger가 hostile target에게 적용 가능한 적대 효과를 가지면 useful target으로 본다.

기본 공격 피해와 projectile runtime은 아직 `src/game/battle/core/commands.rs` 안에 있다. Instant 기본 공격은 release 시점에 즉시 피해를 적용하고, Projectile 기본 공격은 release 시점에 `ProjectileLaunch`와 `ProjectileRecord`를 만들고 이후 `BasicAttackProjectileAdvance` 이벤트에서 locked target body와 continuous sweep으로 impact/miss를 판정한다.

## Source Of Truth 판단

기본 공격 range eligibility의 source of truth는 runtime unit의 최종 `basic_attack.range_policy`와 `basic_attack.defense_tile_range`다. `docs/skill_target_contract.md:80`은 기본 공격 대상 선택 가능 여부가 `range_units`나 연속좌표 거리가 아니라 최종 타일 범위에만 의존한다고 고정한다. 코드도 `src/game/battle/core/targeting.rs:48`에서 `BasicAttackRangePolicy`를 만들고, `src/game/battle/core/targeting.rs:88`에서 WholeField valid tile 또는 tile pattern으로 판정한다.

`range_units`는 기본 공격 eligibility의 source of truth가 아니다. 다만 이동 planner/engine의 접근 거리 힌트로는 여전히 사용된다. 이 컴포넌트에서는 `range_units`를 공격 판정에서 제거하는 방향이 이미 적용된 것으로 보며, 이동 정책 변경은 6번 이동/저지 시스템 또는 별도 정책 감사 범위다.

기본 공격 projectile은 target-locked delivery다. `docs/skill_target_contract.md:131`은 `AttackResolve`/release 시점의 tile eligibility가 피해 적용 또는 projectile spawn의 최종 판정이라고 한다. `docs/skill_target_contract.md:133`은 projectile impact 시점에 range, `range_units`, usefulness, 우선순위/retargeting을 다시 계산하지 않는다고 고정한다. Runtime도 `resolve_basic_attack()`이 release 시점 range를 재검증하고 `ProjectileRecord`를 생성하며, `advance_basic_attack_projectile()`은 locked target validity와 body sweep만 확인한다.

Unity-facing basic attack 표현의 source of truth는 timeline event다. `/mnt/f/unity projects/ark/docs/unity_core_contract.md:1140`은 `AttackStart`, `AttackResolve`, `AttackMiss`를 기본 공격 표현으로, `:1141`은 `BasicAttackProjectileLaunched`, `BasicAttackProjectileImpacted`를 투사체 표현으로 둔다. `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md:585` 역시 `AttackStart`부터 `BasicAttackProjectileImpacted`까지 event 이름을 transport 계약에 포함한다.

## 리팩토링 후보

### 1. Same-time `AttackResolve` batch semantics

`src/game/battle/core/basic_attack.rs:347`에 같은 `time_ms`의 `AttackResolve`를 batch로 처리해야 한다는 TODO가 남아 있다. 현재 sequential queue order에서는 첫 lethal resolve가 battle end를 만들면, 이미 시작된 동시 공격의 damage가 적용되기 전에 전투가 끝날 수 있다고 주석이 설명한다.

이것은 단순 코드 정리가 아니라 gameplay timing 정책이다. 동시에 release된 공격이 전투 종료 이후에도 이미 시작된 피해로 적용되어야 하는지, death/battle end event 순서를 어떻게 고정할지 결정해야 한다.

판단: 리팩토링 후보. `사용자와 정책 논의 필요`.

검증 후보:

- 같은 `time_ms`에 양측 `AttackResolve`가 예약되고 한쪽 resolve가 lethal인 테스트.
- expected timeline order: 두 `AttackResolve`가 모두 기록되는지, 두 damage가 모두 적용되는지, `BattleEnd` 위치가 어디인지.

### 2. 공격자 사망 후 projectile hit의 launch owner snapshot 적용 범위

`ProjectileLaunch`와 `ProjectileRecord`는 `attacker_owner_at_launch`를 저장한다. `advance_basic_attack_projectile()`은 target이 launch owner와 같은 편으로 바뀌면 miss 처리한다(`src/game/battle/core/commands.rs:787`). 이는 `docs/skill_target_contract.md:135`의 "launch 시점 owner snapshot으로 적대성을 판정" 기준과 맞는다.

하지만 실제 hit damage path는 더 좁게 보면 불일치 가능성이 있다. `apply_basic_attack_projectile_hit_at()`은 공격자가 impact 전에 죽은 경우 `DamageContext.attacker_side`와 `target_side`를 둘 다 `target_owner`로 둔다(`src/game/battle/core/commands.rs:998`). 이 경로는 trigger를 비우는 의도는 분명하지만, damage modifier가 side 관계를 보거나 "공격자 side"를 필요로 하면 launch owner snapshot이 damage context까지 전달되지 않는다. 현재 테스트 `advance_basic_attack_projectile_uses_launch_side_after_attacker_death`는 projectile이 hit하고 HP가 감소하는지만 확인한다(`src/game/battle/core/commands.rs:1970`). dead-attacker projectile damage context가 launch owner를 쓰는지까지는 고정하지 않는다.

판단: source-of-truth 적용 범위 리팩토링 후보. targetability에는 launch owner snapshot을 쓰고 있으므로 "전체 미사용"이 아니라 "damage context dead-attacker path의 부분 불일치"로 다뤄야 한다. 공격자 사망 후 on_attack/on_hit trigger를 계속 막을지, damage side만 launch owner로 둘지, modifier 의미가 바뀌는지 정책 판단이 필요하다. `사용자와 정책 논의 필요`.

검증 후보:

- 공격자 사망 후 projectile hit에서 side-sensitive damage modifier가 live attacker hit와 같은 hostile context를 받는지 확인하는 focused test.
- dead attacker projectile은 trigger 없이 기본 피해만 적용한다는 정책을 유지한다면, trigger 미발동과 damage context owner를 별도 assertion으로 고정한다.

### 3. Projectile miss event의 이중 source-of-truth

Projectile miss는 `record_basic_attack_projectile_miss()`에서 `BasicAttackProjectileImpacted { hit: false }`와 `ProjectileMiss`를 모두 기록한다(`src/game/battle/core/commands.rs:911`). `TimelineEvent`에도 두 이벤트가 별도 variant로 존재한다(`src/game/battle/timeline.rs:294`, `:308`). 과거 goal 문서는 `BasicAttackProjectileImpacted.hit = false`가 locked target invalid/expired를 뜻한다고 설명하지만, Unity-facing 계약 문서는 `ProjectileMiss`를 주요 event 목록에 별도로 고정하지는 않고 transport/death validation만 참조한다.

이 구조는 "투사체가 끝난 위치와 hit 여부"와 "miss라는 의미 이벤트"를 나눈 것일 수 있다. 하지만 동일한 miss를 두 이벤트가 동시에 말하면 Unity/client가 어느 쪽을 authoritative miss signal로 삼아야 하는지 애매하다.

판단: source-of-truth 명확화 후보. 이벤트 제거 또는 계약 변경은 Unity-facing DTO 의미 변화이므로 `사용자와 정책 논의 필요`.

검증 후보:

- Unity transport contract에서 `ProjectileMiss`의 역할을 `BasicAttackProjectileImpacted(hit=false)`와 구분하거나, 한쪽을 legacy로 제거하는 계획을 세운다.
- 제거한다면 validation code(`src/game/battle/validation/deaths.rs`, `src/game/battle/validation/spawns.rs`)와 Unity consumer를 함께 확인한다.

### 4. Projectile speed zero fallback

`projectile_flight_ms()`는 `speed_units_per_ms == 0`이면 panic/overflow 대신 0ms instant로 처리한다(`src/game/battle/core/commands.rs:78`). 이 동작은 테스트 `projectile_flight_ms_is_zero_when_speed_is_zero`로 고정되어 있다(`src/game/battle/core/commands.rs:1604`).

반면 live RON 검색상 basic attack projectile speed는 모두 0보다 큰 값이다. `BasicAttackDef::validate_runtime_contract()`는 `TileArea`와 `DirectionalCollision`은 금지하지만 projectile speed > 0은 검증하지 않는다(`src/game/data/abnormality_data.rs:168`). `WeaponCombatProfile::validate_runtime_contract()`도 `range_units`, `interval_ms`, tile range, `TileArea`만 확인한다(`src/game/data/equipment_data.rs:513`). `DeliveryDef::Projectile.speed_units_per_ms`는 plain `u32`로 정의되어 nonzero deserialize guard가 없다(`src/game/ability.rs:157`).

리팩토링 기준상 invalid authoring data를 runtime fallback으로 "instant projectile"처럼 해석하는 것은 제거 후보에 가깝다.

판단: validation 강화 후보. live data에 0 speed가 없으므로 `DeliveryDef::Projectile` speed > 0 검증을 data loading 단계로 올리고, runtime zero-speed fallback test를 삭제하거나 "distance zero only 0ms" 테스트로 교체하는 방향을 검토한다. 다만 skill projectile과 basic attack projectile 모두 같은 `DeliveryDef`를 쓰므로 적용 범위는 14번 데이터/RON 검증 시스템과도 연결된다.

검증 후보:

- projectile speed 0 RON fixture가 `InvalidStaticData` 또는 panic validation으로 실패하는 테스트.
- live RON loading test.

### 5. `basic_attack_range_units()` / `basic_attack_delivery()` public helper 표면

`src/game/battle/core/targeting.rs:391`의 `basic_attack_range_units()`와 `:400`의 `basic_attack_delivery()`는 repo 내부 검색상 호출처가 없다. 특히 `basic_attack_range_units()`는 abnormality base uuid에서 `range_units`를 읽어 `1.0` fallback을 반환한다. 현재 정책에서는 `range_units`가 기본 공격 eligibility, preview, Unity overlay source of truth가 아니므로 이 public helper는 새 코드가 잘못된 source를 다시 잡는 길이 될 수 있다.

판단: 제거 후보. 외부 crate/API 노출이 실제로 필요한지 확인한 뒤, 호출처가 없다면 삭제한다. 만약 Unity-facing catalog에 delivery 표시가 필요하다면 `range_previews`/runtime unit snapshot 또는 explicit DTO에서 읽도록 해야 한다.

검증 후보:

- `rg "basic_attack_range_units\\(|basic_attack_delivery\\(" src tests`로 호출처 없음 확인.
- 삭제 후 `cargo check`.

### 6. Targeting profile hard-coded policy와 `SplashClusterFirst`

`TargetingProfile::SplashClusterFirst`는 hard-coded `SPLASH_CLUSTER_RADIUS_UNITS = 1.5`를 쓴다(`src/game/battle/core/targeting.rs:19`). 후보 target 자체는 `is_basic_attack_useful_target()`으로 필터링하지만(`src/game/battle/core/targeting.rs:159`), cluster score의 주변 적 카운트는 alive/enemy/air-capable/body 거리만 보며 주변 적 각각의 usefulness는 다시 확인하지 않는다(`src/game/battle/core/targeting.rs:275`).

이것은 버그라고 단정하기 어렵다. cluster profile이 "실제 피해 가능한 주변 적 수"를 뜻하는지, "밀집도"를 뜻하는지 정책이 필요하다. 또한 radius를 RON으로 옮기는 일은 밸런스/data schema 변경이다.

판단: 정책 명확화 후보. `사용자와 정책 논의 필요`.

검증 후보:

- 0 damage/immune 주변 적이 cluster score를 올려도 되는지 정책 결정.
- 정책 결정 후 targeting profile test를 보강한다.

## 기존 기능 조합으로 단순화 가능한 후보

- 기본 공격 range eligibility는 이미 `TileRangePolicy`와 `TileRangePattern` 조합으로 단순화되어 있다. `range_units`나 projectile 여부로 range를 재추론하는 runtime path는 찾지 못했다.
- 기본 공격 projectile은 skill의 directional/collision projectile 기능으로 합칠 대상이 아니다. 기본 공격은 target-locked이고, skill projectile은 target-locked와 directional collision을 모두 다룬다. 다만 flight time 계산, 1ms reevaluation cadence, moving body sweep 같은 낮은 수준 helper는 공유 가능성이 있다. 이것은 "행동 의미 통합"이 아니라 중복 helper 추출 후보로만 다뤄야 한다.
- `AttackStart`/`AttackResolve` lifecycle 분리는 이미 `basic_attack.rs`로 옮겨져 있다. 더 큰 통합보다는 같은-time resolve batch나 event contract 정리가 우선이다.

## 레거시/fallback/dual schema 제거 후보

- `projectile_flight_ms(speed=0) => 0` fallback과 이를 고정하는 테스트는 data validation으로 대체할 수 있는 레거시 방어 경로 후보.
- `basic_attack_range_units()`는 `range_units`를 기본 공격 공식 source처럼 보이게 하는 unused public helper 후보.
- `ProjectileMiss`와 `BasicAttackProjectileImpacted(hit=false)`는 둘 중 하나가 legacy 의미 이벤트인지 계약 확인이 필요한 dual event 후보.

## 하지 않거나 보류한 항목

- `range_units` 필드 자체 삭제는 하지 않는다. 이동 planner가 approach/stop distance로 사용하고, 기존 정책 문서도 `range_units`를 비공식 공격 eligibility가 아닌 다른 연속좌표 메커니즘 용도로 남긴다.
- 기본 공격 projectile runtime을 skill projectile runtime으로 곧장 합치지 않는다. 두 delivery의 target 후보, collision 후보, pierce/kill stop policy가 다르므로 겉모양만 보고 통합하면 source-of-truth가 흐려진다.
- `AttackMiss`와 projectile miss 이벤트 관계는 바로 삭제하지 않는다. Unity-facing event 계약과 presentation timing을 같이 확인해야 한다.

## 필요한 테스트와 검증 명령

문서 감사만 수행했으므로 이번 단계에서 테스트는 실행하지 않았다.

후속 구현 시 우선 검증:

```bash
cargo test -p game_core advance_basic_attack_projectile
cargo test -p game_core whole_field_basic_attack
cargo test -p game_core basic_attack_targeting
cargo test -p game_core ron_loading
cargo check -p game_core
```

정책이 확정된 뒤 추가할 테스트:

- same-time `AttackResolve` batch behavior.
- dead-attacker projectile damage context owner snapshot.
- projectile speed 0 validation failure.
- projectile miss event single-source contract.
- `SplashClusterFirst` 주변 적 usefulness 반영 여부.

## 사용자와 정책 논의 필요

- `AttackResolve`가 같은 `time_ms`에 여러 개 있을 때, 이미 시작된 공격 피해를 battle end 이후에도 batch로 적용할지 여부. 사용자와 정책 논의 필요.
- 공격자가 projectile 발사 후 사망한 경우, trigger는 막더라도 damage context의 `attacker_side`를 launch owner snapshot으로 유지할지 여부. 사용자와 정책 논의 필요.
- `ProjectileMiss`와 `BasicAttackProjectileImpacted(hit=false)` 중 어느 이벤트를 Unity-facing miss source of truth로 둘지 여부. 사용자와 정책 논의 필요.
- `SplashClusterFirst`가 주변 적의 usefulness까지 고려해야 하는지, cluster radius를 data로 빼야 하는지 여부. 사용자와 정책 논의 필요.

