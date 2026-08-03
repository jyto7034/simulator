# Skill Target Contract

작성 목적: DefenseRoute 스킬의 기준점 선택, 표시 범위, 시전 가능 범위, 실제 피격 범위를 하나의 계약으로 유지하기 위한 문서다.

## 문서 책임

| 구분 | 내용 |
| --- | --- |
| 문서 등급 | Combat-domain canonical |
| 소유 범위 | 기본 공격과 스킬의 대상 획득, 타일 범위, 자동 적대 타겟 usefulness, cast/step/interrupt/projectile/TileArea 의미, 최종 range preview cell의 판정 의미 |
| 소유하지 않는 범위 | 전체 전투·런 진행 규칙, 일반 runtime commit/recovery, live loader의 공통 strictness, Unity event의 정확한 JSON shape와 message order |
| 함께 확인할 문서 | [game_rulebook.md](game_rulebook.md), [core_runtime_contract.md](core_runtime_contract.md), [data_loading_contract.md](data_loading_contract.md), 외부 `core_unity_battle_transport_contract.md` |

## Targeting Foundations

이 장은 기본 공격과 스킬이 사용할 후보 범위, 자동 대상 선택과 유효 적대 대상 판정의 공통 기반을
소유한다. 전체 전투 진행과 live loader의 공통 strictness는 인접 canonical을 따른다.

### 공식 전투 기준

DefenseRoute는 서로 다른 typed source로 범위를 결정한다.

- 기본 공격 후보: 무기 `basic_attack.defense_tile_range` 또는 침식 profile의 range preset
- Skill unit acquisition: `SkillCastTargetingDef.reach + target_range`
- Skill area delivery: `DeliveryDef::TileArea.footprint`

Core는 unit position/facing, loadout과 battlefield valid tile을 반영해
`range_previews.basic_attack/active_skill.*.cells`를 최종 계산한다. Unity는 authored metadata로
overlay를 재계산하지 않는다.

Preview는 valid tile과 obstacle/blocked tile을 포함하고 battlefield 밖, void와 invalid row
space를 제외한다. 장애물은 이동·배치·pathfinding에는 영향을 주지만 range에서 자동 제외되지
않으며 실제 대상 가능성은 air/target validation이 판단한다.

공식 live skill은 연속좌표 `DeliveryDef::Area(Circle/Line/Box/Rectangle/Cone)`를 사용하지 않는다.

### Weapon Targeting Profile

직원의 고정 직업이 아니라 장착 무기의 `range_role`, `weapon_archetype`,
`targeting_profile`이 기본 공격 역할을 만든다. 현재 archetype은 Sword, Spear, Shield, Bow,
Gun, Staff다.

대상 선택 순서:

1. 공격/스킬의 eligibility와 tile range로 후보를 필터링한다.
2. 근거리 기본 공격은 저지 중인 적을 먼저 고른다.
3. 기본 공격은 weapon `targeting_profile`, skill `EnemySingle.rule`로 정렬한다.
4. 동률은 deterministic tie-breaker로 해소한다.

Skill은 `cast_targeting.target`을 명시하고 EnemySingle이면 rule도 필수다. Weapon profile을
상속하지 않으므로 무기 변경이 skill target rule을 바꾸지 않는다.

### Basic Attack Range Authoring

Basic attack의 stop/intent/target eligibility는 `range_units`, body radius와 world distance가
아니라 runtime tile과 최종 `TileRangePattern`으로만 판정한다. 저지 중인 적은 같은 logical
tile의 blocker-first 규칙을 적용한다.

- 직원은 weapon range를 사용하고 skill effect가 이를 대체/보정할 수 있다. Source가 없을 때만
  자기 tile과 facing 기준 전방 1칸을 fallback으로 사용한다.
- 침식 직원은 profile이 필수 range preset을 참조하고 loader가 이를 pattern으로 해석한다.
  `range_role`, `range_units` 또는 projectile 여부로 추론하지 않는다.
- 초기 preset은 `melee_front_1`, facing을 쓰지 않는 `ranged_center_3x3`와
  `ranged_center_5x5`다. Profile이 명시적으로 선택하며 직업 하드코딩이 아니다.
- Pattern은 valid tile과 교차하고 battlefield 밖/void를 제외하되 obstacle/blocked tile은
  제외하지 않는다.

### Enemy Ranged Route Attack Policy

원거리 route enemy는 전진하다 target을 포착하면 멈춰 공격하고 release 뒤
`ranged_reposition_ms` 동안 이동을 시도한 다음 기존 target을 재검증한다. 기본값은 1000ms이며
0이면 재이동 없이 즉시 재검증한다.

- 재이동 중 새 target을 찾지 않지만 block, death, route end, battle end와 hard movement lock이
  이를 중단한다. Blocker attack은 재이동과 기존 target보다 우선한다.
- `AttackStart`와 release에서 runtime tile eligibility를 검사하며 release 판정이 damage/projectile
  생성의 최종 gate다.
- Basic projectile은 release 때 확정한 target만 continuous sweep hit 후보로 삼고 bystander에
  충돌하거나 range/usefulness를 재평가하지 않는다.
- Target이 impact 전 죽거나 철수·아군화·air-ineligible이 되면 miss다. Attacker가 죽어도 launch
  owner snapshot으로 hostility를 유지한다.
- 정식 Boss/Elite/특수 unit은 일반 preset fallback 대신 자체 authored range/skill을 가져야 한다.
  현재 live abnormality 30개의 generic missing-pattern 범위는 dedicated range 설계 전 temporary
  placeholder이며 이 정식 authoring 정책의 완료 증거가 아니다. 현재 placeholder 수치는 이번
  정책 정리에서 변경하지 않는다.

### Basic-Attack Range And Skill Reach

Basic attack은 `TileRangePolicy/defense_tile_range`, skill cast는 별도 `reach`를 사용한다.

- Live skill은 `reach: pattern|global`과 `air_capable`을 명시한다.
- Pattern EnemySingle은 `target_range` 또는 preset이 필수고 global은 competing range를
  허용하지 않는다.
- Global은 unit acquisition만 전역으로 만들며 effect area는 `TileArea.footprint`가 소유한다.
  Step-level range/air field와 `WholeFieldValidTiles + TileArea` 혼합은 제거됐다.
- `WholeFieldValidTiles`는 route와 무관하지만 일반 `profile_role: Normal` 침식 직원은 사용할
  수 없다. Special/LegacyEcho와 명시된 특수 unit만 허용한다.
- 전장 전체 후보여도 alive/hostile/air/usefulness와 targeting rule을 적용한다. Skill은 weapon
  profile이나 area 최대화가 아니라 명시 `UnitTargetRule`을 사용한다.
- 일반 DTO는 player preview만 운반한다. Enemy telegraph가 필요하면 실제 피해/효과 tile을 별도
  event/DTO로 보낸다.

### Automatic Hostile Target Usefulness

자동 기본 공격, skill cast, step retarget과 TileArea 시전 가능성은 deterministic expected final
damage가 양수이거나 적에게 실제 hostile state change를 적용할 수 있을 때만 target을 고른다.
AoE는 범위 안에 이런 적이 하나 이상 있어야 하며 damage 가능 대상과 effect-only 대상 사이에
추가 우선순위를 두지 않는다.

Expected damage는 현재 defense/resistance, immunity, fixed damage와 deterministic modifier를
반영하되 crit/miss/random variance는 포함하지 않는다.

Hostile target-applied effect에는 debuff, control, forced movement, DoT, stat/resistance 감소,
vulnerability, mark, aggro 조작과 beneficial-effect 제거가 포함된다. Self/ally buff·heal·shield,
resource/cooldown 변화, presentation-only, projectile/summon 생성 자체와 target state를 바꾸지
않는 global trigger는 포함하지 않는다. `InterruptCast`만 가진 skill은 target이 실제 pending
cast를 가질 때만 유효하다.

Targeting 단계는 이미 적용된 effect의 refresh/stack/resist를 미리 계산하지 않지만 authored
target filter는 따른다.

초기 profile:

- `DefaultForward`: route end까지 남은 진행 거리가 짧은 적 우선
- `AirFirst`: air 우선 후 DefaultForward
- `LowDefenseFirst`: 낮은 defense 후 DefaultForward
- `LowMagicResistFirst`: 낮은 magic resistance 후 DefaultForward

DefaultForward 동률은 먼저 spawn된 unit, 그다음 unit id 순서다.

## Skill Execution Lifecycle

이 장은 cast target 캡처부터 step retarget, cast interrupt와 projectile impact까지 대상 identity가
언제 선택·유지·무효화되는지를 소유한다.

### SkillTarget

`SkillTarget`은 범위가 아니라 기준 대상을 결정한다.

```rust
pub enum SkillTarget {
    SelfUnit,
    EnemySingle { rule: UnitTargetRule },
    CastTarget,
}
```

- `SelfUnit`: 시전자 자신을 기준으로 한다.
- `EnemySingle`: cast의 `reach`/`target_range`와 air eligibility를 통과한 유효 적 중 명시 rule로 하나를 선택한다.
- `CastTarget`: 이전 step 또는 cast-level 기준점을 재사용한다.
- `EnemySingle.rule`은 authoring 필수값이다. `Nearest`도 의도적으로 작성해야 하며 default,
  weapon-profile inheritance, unknown dual-authority field는 허용하지 않는다.

Step은 cast acquisition range를 다시 작성하지 않는다. Area step은 delivery 안에 concrete
`footprint`을 반드시 가진다.

### Step Targeting Mode

`SkillStepDef.targeting`은 step 실행 시점의 대상 해석 방식을 결정한다.

```rust
pub enum StepTargetingMode {
    ReuseCastTarget,
    RetargetOnStep,
}
```

기본값은 `ReuseCastTarget`이다.

`ReuseCastTarget`은 스킬 시전 시 확정된 cast-level target을 step 실행에도 재사용한다.

- `SkillTarget::SelfUnit`: 항상 시전자 자신을 step target으로 사용한다.
- `SkillTarget::EnemySingle`: direct unit effect는 cast range를 다시 검사하지 않지만 실행 시점에
  대상이 살아 있는 적이고 cast의 air eligibility를 만족하며 해당 step effect가 여전히 유효해야
  한다. 이 조건을 잃으면 direct target 없이 끝난다. `TileArea`는 선택 시 캡처한 target tile을
  재사용하므로 대상이 죽거나 withdraw되어도 그 tile에서 실행된다.
- `SkillTarget::CastTarget`: cast target이 unit이면 unit, tile이면 tile을 그대로 재사용한다. cast target이 없고 cast anchor tile이 기록되어 있으면 그 tile을 사용한다.

`RetargetOnStep`은 예약된 step이 실행되는 시점에 대상을 다시 고르되 같은 skill cast의
`reach`, `target_range`, `air_capable`을 사용한다. Step-local range/air authority는 없다.

- 후속 타격이 첫 타격 대상에 고정되어야 하면 `ReuseCastTarget`을 사용한다.
- 후속 타격이 현재 전장의 유효 대상 중 새 대상을 찾아야 하면 `RetargetOnStep`을 사용한다.
- `RetargetOnStep`은 이전 cast target을 보정하는 모드가 아니라, step-local targeting을 다시 수행하는 모드다.

현재 live RON의 `RetargetOnStep` 사용은 다음 네 step이다.

- `little_red_hunt_the_prey.hunt_finish`
- `nothing_there_goodbye.goodbye_finish`
- `funeral_butterfly_eulogy_volley.black_funeral`
- `fragment_funeral_butterfly_eulogy.fragment_quiet_lament`

모두 후속/조건부 타격이 실행 시점의 유효 대상을 같은 cast-level reach/range/air 계약 안에서
다시 고르는 경우다.

`SkillDef.cast_targeting`과 `SkillStepDef.targeting`의 책임은 다르다.

- `cast_targeting`: target kind/rule, `reach`, `target_range`, `air_capable`, 최초 캡처 tile을 정한다.
- `target`: step이 원하는 대상 종류를 정한다.
- `targeting`: 그 step이 cast target을 재사용할지, step 실행 시 다시 target을 고를지 정한다.
- `DeliveryDef::TileArea.footprint`: 캡처/impact/caster anchor에서 실제 피격 타일을 계산하는 유일한 footprint다.

제거된 `SkillCastTargetingDef::FirstStepTarget` 같은 첫-step 기반 cast target 추론은 legacy다.
Live RON과 internal tests/fixtures는 cast-level target을 명시한다. 작성 편의 helper는 step 목록을
읽어 추론하지 않고 caller가 target, reach, target range, air capability를 직접 넘기는 explicit
constructor여야 한다.

### Skill Cast Interrupt

스킬 시전 방해는 `Silence`와 분리한다.

- `Silence`: 새 수동/자동 스킬 시전 시작을 막는다. 이미 시작된 pending cast를 취소하지 않는다.
- `InterruptCast`: 대상의 현재 pending skill cast를 취소한다.

cast start event-log `seq`가 cast epoch/token이다. core는 `ManualCastStart` 또는 `AutoCastStart`를 기록한 뒤 그 `seq`를 pending cast에 저장한다. 예약된 `ManualCastEnd` 또는 `AutoCastEnd`는 자신의 `cause: Parent { seq }`가 현재 pending cast의 start seq와 같을 때만 스킬을 invoke한다.

`InterruptCast`가 성공하면 대상의 pending cast를 제거하고 `SkillCastInterrupted` event를 기록한다. 이후 예전에 예약된 cast-end event가 도착해도 start seq가 맞지 않으므로 stale event로 무시한다.

시전자 자신이 철수해 lifecycle이 `Withdrawn`으로 바뀌는 경우는 외부 방해 효과가 아니므로 `SkillCastInterrupted`를 쓰지 않는다. core는 pending manual/auto cast 또는 이미 시작된 active skill runtime을 제거하고 `SkillCastCancelled { reason: Withdrawn }` event를 기록한다.

정책:

- 성공한 interrupt는 `interrupted_cast_seq`로 취소된 `ManualCastStart` 또는 `AutoCastStart`를 가리킨다.
- interrupt는 ability invoke 전 focus/cast 단계만 취소한다.
- 이미 생성된 projectile, tile area, persistent area, scheduled skill step, buff tick, triggered effect는 취소하지 않는다.
- 이미 소비된 resonance/focus/cooldown/resource는 환불하지 않는다.
- interrupt 대상에게 pending cast가 없으면 no-op이며 `SkillCastInterrupted` event를 만들지 않는다.
- interrupted cast는 `ManualCastEnd` 또는 `AutoCastEnd` 완료 event를 만들지 않는다.
- withdrawn으로 취소된 cast는 `SkillCastCancelled`를 만들고, 해당 cast에 대한 `ManualCastEnd` 또는 `AutoCastEnd` 완료 event를 만들지 않는다.
- withdrawn caster가 만든 active skill projectile/area runtime은 제거된다. 이미 event log에 기록된 과거 event는 수정하지 않는다.

### Projectile Target Lifecycle

기본 공격 projectile은 locked target을 가진다. impact 시점에 locked target이 죽었거나, 철수했거나, 더 이상 적대적 유효 대상이 아니거나, 공중 타격 조건을 만족하지 못하면 `BasicAttackProjectileImpacted { hit: false }`를 기록하고 damage를 적용하지 않는다. 별도 `ProjectileMiss` event는 만들지 않는다.

스킬 projectile은 step/delivery 정책에 따라 대상 또는 타일을 가진다. impact 시 유효한 첫 피격 유닛이 없으면 `SkillProjectileImpacted { first_hit_unit_id: null }`로 끝난다. 이 이벤트가 스킬 projectile의 no-hit/miss 표현이며, Unity는 별도 miss event를 기대하지 않는다.

## Area Delivery

이 장은 unit acquisition과 분리된 `TileArea` footprint, anchor, affected tile과 deferred area 정책을
소유한다.

### TileArea

다중 피격은 DefenseRoute에서 `DeliveryDef::TileArea`를 사용한다.

```ron
SkillStepDef(
    id: "example_sweep",
    target: EnemySingle(rule: Nearest),
    delivery: TileArea(
        footprint: (include_anchor_tile: true, rows: ["XXX", "X@X", "XXX"]),
        area: (
            anchor: CastTarget,
            tile_origin: Anchor,
            hit_targets: Enemies,
            include_caster: false,
        ),
    ),
    effects: [
        Damage(amount: 35, damage_type: Magic),
    ],
)
```

`TileArea.footprint`이 actual affected-tile shape다. Global reach와 footprint는 서로 대체하지 않는다.

`TileArea.area`는 anchor/filter/tick 정책을 담당한다. `CastTarget` anchor는 선택 순간 캡처한
tile을 사용하고 `tile_origin: Anchor`가 필수다. Target/impact-centered footprint는 explicit
orientation 정책이 아직 없으므로 rotation-invariant여야 하며 facing 없는 stationary caster도
동일한 타일을 만든다.

- `anchor`: event-log center, impact context, VFX 기준점, 또는 `tile_origin: Anchor`가 참조할 위치를 결정한다.
- `tile_origin`: `footprint`의 `@`를 caster 또는 resolved anchor 중 어디에 놓을지 결정한다.
- `tracking`: 지속 영역이 시전자/대상/지면 중 무엇을 따라갈지 결정한다.
- `hit_targets`: 범위 안에서 아군, 적, 전체 중 어떤 유닛을 맞출지 결정한다.
- `include_caster`: 시전자가 같은 타일 필터에 걸릴 때 포함할지 결정한다.
- `tick_policy`, `duration_ms`, `tick_interval_ms`: 지속 범위 스킬의 tick 정책을 결정한다.

`ImpactContext`/`ImpactContextStart` 앵커는 이전 projectile 또는 tile area delivery가 만든 impact context를 재사용한다. 이전 spatial delivery가 없는 step에서 사용하면 잘못된 RON으로 간주한다. Cast target tile은 selection 때 저장되므로 대상 소멸은 area anchor 소멸이 아니다.

기본 공격은 `TileArea`를 사용하지 않는다. 기본 공격의 `defense_tile_range`는 DefenseRoute에서 단일 공격 대상 후보를 찾는 사거리 패턴이며, 실제 다중 피격은 스킬 step의 `DeliveryDef::TileArea`만 사용한다.

## Projection And Authoring Boundaries

이 장은 targeting 결과를 event/preview로 전달할 때의 의미, 제거된 schema와 새 range 작성 규칙을
소유한다. 정확한 JSON shape와 message order는 외부 Unity canonical을 따른다.

### Event Log / Unity 표시 계약

DefenseRoute에서 runtime `SkillAreaDeclared` event는 core가 확정한 `tile_pattern.affected_tiles`를 전달한다. Unity는 이 타일을 범위 경고/효과 표시에 사용하고, 배치/선택/스킬 버튼 preview는 `range_previews`의 최종 cell DTO를 사용한다. Circle/Line/Box/Rectangle/Cone을 재해석하거나 authored `defense_tile_range`에서 표시 타일을 추론하지 않는다.

정확한 event JSON shape와 message ordering은 외부 canonical `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`가 소유한다. 이 문서는 어떤 타일이 판정 대상이 되는지와 range/targeting 의미만 소유한다.

### 레거시

다음 계약은 DefenseRoute 공식 live path에서 제거됐다.

- `DeliveryDef::Area(shape: Circle/Line/Box/Rectangle/Cone)` 기반 공식 스킬 판정.
- `SkillTarget` 안에 범위형 아군/적 variant를 두는 방식.
- `target.area`와 `delivery.area.shape`가 동시에 범위를 말하던 이중 source of truth.
- geometric AoE 동작을 고정하는 legacy compatibility test.

`DeliveryDef::Area`, `SkillAreaDeliveryDef`, `SkillAreaShapeDef`, step range fields, 그리고
`WholeFieldValidTiles + TileArea` 혼합 계약은 schema/runtime에 없다.

### 작성 규칙

- 단일 타겟 스킬: core는 cast `reach`/`target_range` 안의 유효 대상 중 하나를 선택한다.
- 범위 스킬: core는 `TileArea.footprint`가 산출한 valid tile 안의 authored hit-filter 대상에게 적용한다.
- live RON cast pattern은 `target_range` 또는 `target_range_preset`을 사용하며 둘을 동시에
  작성하면 실패한다. TileArea footprint는 delivery 안에 concrete pattern으로 존재한다.
- `ModifyDamage`는 같은 step의 `Damage`에만 적용되는 step-local modifier다. `Damage` 없는 step에는 작성하지 않는다.
- 새로운 범위 모양이 필요하면 geometric shape를 되살리지 말고 `TileRangePattern.rows`를 추가한다.
