# Skill Target Contract

작성 목적: DefenseRoute 스킬의 기준점 선택, 표시 범위, 시전 가능 범위, 실제 피격 범위를 하나의 계약으로 유지하기 위한 문서다.

## 공식 전투 기준

현재 공식 전투는 명일방주식 `DefenseRoute`다.

`DefenseRoute`에서 스킬/평타 범위의 단일 source of truth는 `SkillStepDef.defense_tile_range`다.

```text
defense_tile_range
  - Unity range preview
  - 시전 가능 대상 후보
  - 타겟팅 가능한 타일
  - 실제 피격 타일
```

따라서 DefenseRoute 공식 live skill은 `DeliveryDef::Area(shape: Circle/Line/Box/Rectangle/Cone)` 같은 연속좌표 geometric AoE를 사용하지 않는다.

## Weapon Targeting Profile

직원의 고정 직업은 타겟팅 source of truth가 아니다. 직원의 현재 기본 공격 역할은 장착 무기가 만든 전투 프로필에서 나온다.

무기 전투 프로필은 장기적으로 다음 축을 가진다.

```text
range_role: Melee | Ranged
weapon_archetype: Sword | Spear | Axe | Shield | Bow | Crossbow | Gun | Shotgun | GrenadeLauncher | Staff | ...
targeting_profile: DefaultForward | AirFirst | LowDefenseFirst | LowMagicResistFirst | SplashClusterFirst | ...
```

타겟팅은 항상 아래 순서로 처리한다.

1. 전투 모드와 공격/스킬이 정의한 유효 후보를 먼저 필터링한다.
2. `DefenseRoute` 플레이어 유닛은 `defense_tile_range` 안에 있는 대상만 후보로 남긴다.
3. 후보가 없으면 타겟 없음으로 처리한다.
4. 후보가 있으면 무기 또는 스킬의 `targeting_profile`로 정렬한다.
5. 동률은 deterministic tie-breaker를 사용한다.

근거리 기본 공격은 저지 중인 적을 최우선으로 한다. 저지 중인 적이 없을 때만 무기 `targeting_profile`을 적용한다.

원거리 기본 공격은 무기 `targeting_profile`을 기본 source of truth로 삼는다.

스킬은 스킬 데이터가 별도 타겟팅을 명시하면 스킬 전용 타겟팅을 사용한다. 명시하지 않으면 장착 무기의 기본 `targeting_profile`을 따른다.

초기 타겟팅 프로필:

| profile | 정렬 규칙 |
| --- | --- |
| `DefaultForward` | 방어 목표/route 종료 지점까지 남은 route가 가장 짧은 적 우선 |
| `AirFirst` | 공중 적 우선, 없으면 `DefaultForward` |
| `LowDefenseFirst` | 방어력이 가장 낮은 적 우선, 동률이면 `DefaultForward` |
| `LowMagicResistFirst` | 마법 저항이 가장 낮은 적 우선, 동률이면 `DefaultForward` |
| `SplashClusterFirst` | 직접 대상 주변에 함께 맞는 유효 적 수 또는 예상 피해 기대값이 큰 대상 우선, 동률이면 `DefaultForward` |

`DefaultForward`의 route 기준은 단순 직선거리가 아니라 적이 지정 route를 따라 얼마나 진행했는지다. 같은 route 안에서는 route progress가 큰 적이 우선이다. 여러 route가 섞이면 각 적의 현재 route에서 종료 지점까지 남은 진행 거리를 비교한다. 이 값이 동률이거나 계산 불가능하면 먼저 spawn된 적, 그다음 unit id 순으로 처리한다.

`SplashClusterFirst`는 범위 안의 모든 적을 실제로 맞추는 `TileArea`와 다르다. 이 프로필은 “직접 대상 하나를 고르는 규칙”이며, 해당 직접 대상 주변 splash 기대값을 고려할 뿐이다. 실제 다중 피격 여부는 스킬 delivery와 `defense_tile_range`가 결정한다.

추후 필요하면 `EliteFirst`, `LowestHpFirst`, `HighestBlockWeightFirst`, `BossFirst`, `ClosestFirst` 같은 프로필을 추가할 수 있다. 단 새 프로필은 실제 무기/스킬 파편 수요가 확인된 뒤 추가한다.

## SkillTarget

`SkillTarget`은 범위가 아니라 기준 대상을 결정한다.

```rust
pub enum SkillTarget {
    SelfUnit,
    EnemySingle { rule: UnitTargetRule },
    CastTarget,
}
```

- `SelfUnit`: 시전자 자신을 기준으로 한다.
- `EnemySingle`: `defense_tile_range` 안의 유효 적 중 하나를 선택한다.
- `CastTarget`: 이전 step 또는 cast-level 기준점을 재사용한다.

`EnemySingle`과 `CastTarget` step은 `defense_tile_range`를 가져야 한다.

## TileArea

다중 피격은 DefenseRoute에서 `DeliveryDef::TileArea`를 사용한다.

```ron
SkillStepDef(
    id: "example_sweep",
    range_units: 2,
    defense_tile_range: Some((
        include_anchor_tile: false,
        rows: [".XXX.", ".XXX.", "..@..", ".....", "....."],
    )),
    target: EnemySingle(rule: Nearest),
    delivery: TileArea(area: (
        anchor: CastTarget,
        tile_origin: Caster,
        hit_targets: Enemies,
        include_caster: false,
    )),
    effects: [
        Damage(amount: 35, damage_type: Magic),
    ],
)
```

`TileArea`는 별도 shape를 갖지 않는다. 어떤 타일이 맞는지는 항상 `defense_tile_range`로 계산한다.

`TileArea.area`가 담당하는 것은 범위 모양이 아니라 다음 보조 정책이다. 실제 피격 타일은 `tile_origin`이 선택한 기준 타일과 시전자 facing에 투영한 `defense_tile_range`에서 나온다.

- `anchor`: timeline center, impact context, VFX 기준점, 또는 `tile_origin: Anchor`가 참조할 위치를 결정한다.
- `tile_origin`: `defense_tile_range`의 `@`를 실제 전장 어디에 놓을지 결정한다. 기본값은 `Caster`이며, `Anchor`를 쓰면 `anchor`가 가리키는 타일을 기준으로 한다. 모르가나 W 같은 지정 지점 장판은 `anchor: CastTarget`, `tile_origin: Anchor`, `tracking: GroundFixed`로 표현한다.
- `tracking`: 지속 영역이 시전자/대상/지면 중 무엇을 따라갈지 결정한다.
- `hit_targets`: 범위 안에서 아군, 적, 전체 중 어떤 유닛을 맞출지 결정한다.
- `include_caster`: 시전자가 같은 타일 필터에 걸릴 때 포함할지 결정한다.
- `tick_policy`, `duration_ms`, `tick_interval_ms`: 지속 범위 스킬의 tick 정책을 결정한다.

`ImpactContext`/`ImpactContextStart` 앵커는 이전 projectile 또는 tile area delivery가 만든 impact context를 재사용한다. 이전 spatial delivery가 없는 step에서 사용하면 잘못된 RON으로 간주하고 로딩 검증에서 실패시킨다. 전투 중 대상이 사라져 런타임 anchor를 해석하지 못하는 경우는 합법적인 전투 상태일 수 있으므로, 정적 데이터 오류와 분리해서 처리한다.

기본 공격은 `TileArea`를 사용하지 않는다. 기본 공격의 `defense_tile_range`는 DefenseRoute에서 단일 공격 대상 후보를 찾는 사거리 패턴이며, 실제 다중 피격은 스킬 step의 `DeliveryDef::TileArea`만 사용한다.

## Timeline / Unity 표시 계약

DefenseRoute에서 `SkillAreaDeclared.shape`는 우선 `tile_pattern`을 사용한다.

```json
{
  "type": "tile_pattern",
  "affected_tiles": [
    { "x": 4, "y": 2 },
    { "x": 5, "y": 2 }
  ]
}
```

Unity는 `affected_tiles`를 그대로 범위 경고/효과 타일로 표시한다. Circle/Line/Box/Rectangle/Cone을 재해석해서 DefenseRoute 범위를 추론하지 않는다.

## 레거시

다음 계약은 DefenseRoute 공식 live path에서 제거됐다.

- `DeliveryDef::Area(shape: Circle/Line/Box/Rectangle/Cone)` 기반 공식 스킬 판정.
- `SkillTarget` 안에 범위형 아군/적 variant를 두는 방식.
- `target.area`와 `delivery.area.shape`가 동시에 범위를 말하던 이중 source of truth.
- geometric AoE 동작을 고정하는 legacy compatibility test.

`DeliveryDef::Area`, `SkillAreaDeliveryDef`, `SkillAreaShapeDef`는 schema/runtime에서 제거 대상이다. 공식 스킬 범위를 막는 방식은 validation adapter가 아니라 `TileArea`와 `defense_tile_range`만 남기는 구조 제거다.

## 작성 규칙

- 단일 타겟 스킬: `defense_tile_range` 안의 유효 대상 중 하나를 선택한다.
- 범위 스킬: `defense_tile_range`가 산출한 타일 안의 유효 대상 전체에게 효과를 적용한다.
- live RON에서는 반복되는 `defense_tile_range`를 `SkillDatabase.range_presets`와 `defense_tile_range_preset`으로 작성할 수 있다. preset은 RON 로딩 단계에서 실제 `defense_tile_range`로 해석되며, 전투 런타임은 preset 개념을 모른다.
- `defense_tile_range`와 `defense_tile_range_preset`을 같은 step 또는 explicit cast target에 동시에 작성하면 source of truth 충돌로 간주하고 로딩 검증에서 실패시킨다.
- `ModifyDamage`는 같은 step의 `Damage`에만 적용되는 step-local modifier다. `Damage` 없는 step에는 작성하지 않는다.
- 새로운 범위 모양이 필요하면 geometric shape를 되살리지 말고 `TileRangePattern.rows`를 추가한다.
- 시나리오/RON이 맵의 고정 좌표에 직접 장판을 생성하는 기능은 아직 공식 계약이 아니다. 필요해지면 BattleScenario 조건부 이벤트 확장에서 별도 target payload나 tactical point anchor로 추가한다.
