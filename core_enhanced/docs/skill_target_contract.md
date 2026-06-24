# Skill Target Contract

작성 목적: DefenseRoute 스킬의 기준점 선택, 표시 범위, 시전 가능 범위, 실제 피격 범위를 하나의 계약으로 유지하기 위한 문서다.

## 공식 전투 기준

현재 공식 전투는 명일방주식 `DefenseRoute`다.

`DefenseRoute`에서 스킬/평타 범위의 내부 authoring source of truth는 RON에 작성된 `TileRangePattern`이다.

- 직원/장비 기본 공격: 무기 `basic_attack.defense_tile_range`
- 침식 직원 기본 공격: 침식 직원 프로필이 참조하는 basic attack range preset
- 스킬: `SkillStepDef.defense_tile_range`

```text
defense_tile_range
  - 시전 가능 대상 후보
  - 타겟팅 가능한 타일
  - 실제 피격 타일
```

Unity-facing range preview의 source of truth는 `range_previews`의 최종 cell DTO다. Unity는 `defense_tile_range`, `effective_weapon_profile`, `effective_basic_attack`, skill catalog range metadata를 조합해 overlay를 계산하지 않는다.

```text
range_previews
  - basic_attack.cells
  - active_skill.cast_cells
  - active_skill.effect_preview_cells
```

core는 현재 유닛 위치, facing, 무기/스킬 파편, fallback 기본 공격 정책, 전장 valid tile 기준을 반영해 최종 cell을 계산한다.

Range preview cell filtering은 이동 가능 여부가 아니라 전장 안의 유효 타일 여부를 기준으로 한다.

```text
range preview 포함:
  - valid tile
  - obstacle/blocked tile

range preview 제외:
  - void tile
  - 전장 밖 좌표
  - ASCII row 공백처럼 valid_tiles에 포함되지 않는 invalid tile
```

장애물과 blocked tile은 지상 이동/배치/pathfinding/충돌에는 영향을 주지만, 공격 또는 스킬 범위 표시에서 기본적으로 제외하지 않는다. 공중 유닛은 장애물과 walkable tile 제약을 받지 않을 수 있고, 장애물 위나 너머의 대상도 공격/스킬의 `air_capable`, target policy, 실제 target validation이 허용하면 유효 대상이 될 수 있다. 따라서 Unity-facing `range_previews`는 `walkable` 여부가 아니라 core가 계산한 valid tile 기준 최종 cell을 표시한다.

따라서 DefenseRoute 공식 live skill은 `DeliveryDef::Area(shape: Circle/Line/Box/Rectangle/Cone)` 같은 연속좌표 geometric AoE를 사용하지 않는다.

## Weapon Targeting Profile

직원의 고정 직업은 타겟팅 source of truth가 아니다. 직원의 현재 기본 공격 역할은 장착 무기가 만든 전투 프로필에서 나온다.

무기 전투 프로필은 장기적으로 다음 축을 가진다.

```text
range_role: Melee | Ranged
weapon_archetype: Sword | Spear | Shield | Bow | Gun | Staff
targeting_profile: DefaultForward | AirFirst | LowDefenseFirst | LowMagicResistFirst | SplashClusterFirst | ...
```

현재 live RON에서 사용하는 기본 무기 아키타입은 `Sword`, `Spear`, `Shield`, `Bow`, `Gun`, `Staff`다. `GrenadeLauncher`는 runtime enum에 남아 있는 미사용/미래 후보이며, 현재 live 장비/파편 설계의 기본 후보로 취급하지 않는다. `Axe`, `Crossbow`, `Shotgun` 같은 추가 아키타입은 별도 구현 goal에서 enum, live RON, Unity 표시 계약을 함께 확정한 뒤 추가한다.

타겟팅은 항상 아래 순서로 처리한다.

1. 전투 모드와 공격/스킬이 정의한 유효 후보를 먼저 필터링한다.
2. `DefenseRoute` 플레이어 유닛은 `defense_tile_range` 안에 있는 대상만 후보로 남긴다.
3. 후보가 없으면 타겟 없음으로 처리한다.
4. 후보가 있으면 무기 또는 스킬의 `targeting_profile`로 정렬한다.
5. 동률은 deterministic tie-breaker를 사용한다.

근거리 기본 공격은 저지 중인 적을 최우선으로 한다. 저지 중인 적이 없을 때만 무기 `targeting_profile`을 적용한다.

원거리 기본 공격은 무기 `targeting_profile`을 기본 source of truth로 삼는다.

스킬은 스킬 데이터가 별도 타겟팅을 명시하면 스킬 전용 타겟팅을 사용한다. 명시하지 않으면 장착 무기의 기본 `targeting_profile`을 따른다.

## Basic Attack Range Authoring

기본 공격 범위는 런타임에서 `range_units`나 연속좌표 거리로 추론하지 않는다. 기본 공격이 대상을 고를 수 있는지는 최종 타일 범위에만 의존한다.

직원 기본 공격 범위:

- 장착 무기가 있으면 무기 RON의 `basic_attack.defense_tile_range`가 source of truth다.
- 시전 중인 스킬 파편이 기본 공격을 대체하거나 보정하면 해당 스킬/효과 RON이 최종 범위를 만든다.
- 아무 무기/스킬 범위 source가 없을 때만 core 기본값을 사용한다.
- 직원 기본값은 자기 타일과 facing 기준 전방 1칸이다.

침식 직원 기본 공격 범위:

- 침식 직원 프로필 RON이 basic attack range preset pool에서 하나의 range id를 참조한다.
- range preset pool은 반복되는 `TileRangePattern`을 이름으로 관리하는 authoring 편의 계층이다.
- RON 로딩/검증 단계에서 profile의 range preset은 실제 `TileRangePattern`으로 해석된다.
- 전투 런타임, Unity-facing DTO, range preview는 preset id를 재계산 source로 사용하지 않고, 이미 해석된 최종 타일 범위를 사용한다.
- 침식 직원은 `range_role`, `range_units`, projectile 여부만 보고 자동으로 공격 범위를 추론하지 않는다.
- live 침식 직원 프로필은 반드시 명시적인 basic attack range preset을 가져야 한다.

초기 침식 직원 range preset:

| preset | 의미 |
| --- | --- |
| `melee_front_1` | 자기 타일 + facing 기준 전방 1칸 |
| `ranged_center_3x3` | 자기 위치 타일 중심 3x3 정사각형. facing을 사용하지 않는다. |
| `ranged_center_5x5` | 자기 위치 타일 중심 5x5 정사각형. facing을 사용하지 않는다. |

근거리 침식 직원은 `melee_front_1`을 사용할 수 있고, 원거리 침식 직원은 `ranged_center_3x3` 또는 `ranged_center_5x5`를 사용할 수 있다. 그러나 이들은 하드코딩된 직업 규칙이 아니라 RON profile이 선택한 preset이다.

원거리 침식 직원 range preset은 자기 위치 타일을 기준으로 한다. 공격 후보 타일은 전장 `valid_tiles`와 교차한 결과만 사용하며, 전장 밖/void/invalid tile은 제외한다. 장애물/blocked tile은 공격 후보 타일에서 제외하지 않는다. 장애물은 이동/배치/pathfinding/충돌에만 영향을 주며, 실제 피격 가능성은 공격/스킬의 target validation, 공중 대상 가능 여부, 유효 hostile target 규칙이 판단한다.

## Enemy Ranged Route Attack Policy

적군 원거리 route 유닛은 제자리 포탑이 아니다. 기본 이동 정책은 route를 따라 강제 전진하는 것이며, 공격은 이 전진 흐름 사이에 끼어드는 행동이다.

기본 흐름:

1. route를 따라 이동한다.
2. 공격 가능한 타겟을 포착하면 이동을 멈추고 공격한다.
3. 공격 windup/release/피해 또는 투사체 생성 후, 명시된 재이동 시간 동안 route 이동을 시도한다.
4. 재이동 후 기존 타겟을 재검증한다.
5. 기존 타겟이 살아 있고, 타겟팅 가능하며, 공격 타일 범위 안에 있고, 피해 또는 적용 가능한 적대 효과가 있으면 새 타겟 탐색 없이 기존 타겟을 다시 공격한다.
6. 기존 타겟이 유효하지 않으면 현재 타겟팅 규칙으로 새 타겟을 찾는다.

재이동 시간은 attack release 시점, 즉 피해 적용 또는 투사체 생성 직후부터 시작한다. 공격 windup 시간은 재이동 시간에 포함하지 않는다.

기본 재이동 시간은 `ranged_reposition_ms = 1000`이다. basic attack range preset pool은 공용 authoring pool이며, 침식 직원 profile은 그중 하나를 참조한다. `ranged_reposition_ms`도 profile/RON에서 유닛별로 명시적으로 바꿀 수 있어야 한다. 명시값은 0 이상이어야 하며, 0이면 공격 후 재이동 없이 즉시 기존 타겟을 재검증한다.

`ranged_reposition_ms`는 무조건 이동을 보장하는 시간이 아니라 최소 route 재이동 시도 시간이다. 재이동 중에는 새 원거리 타겟 탐색을 하지 않는다. 단, 저지, 사망, route end, 전투 종료, hard movement lock 같은 더 강한 상태가 발생하면 즉시 중단한다.

저지 상태는 원거리 재이동, 기존 원거리 타겟, 방어 목표/보호 오브젝트 공격보다 우선한다. 자신을 저지하는 유닛이 있으면 blocker를 우선 공격한다. blocked 상태에서는 `ranged_reposition_ms`를 이유로 block을 무시하고 이동하지 않는다.

기본 공격 tile eligibility는 core battle time의 runtime tile 기준으로 판정한다. `AttackStart`에서 공격을 시작할 수 있는지 확인하고, `AttackResolve`/release 시점의 tile eligibility가 피해 적용 또는 투사체 생성의 최종 판정이다.

기본 공격 투사체는 target-locked delivery다. `AttackResolve`/release 시점에 타일 기반 eligibility를 통과한 원래 대상만 hit 후보이며, 투사체 경로상의 다른 유닛, blocker, bystander는 충돌 후보가 아니다. 발사 후 impact 시점에는 공격 범위, `range_units`, target usefulness, 우선순위/재타겟팅을 다시 계산하지 않는다. 투사체 평타의 gameplay impact는 `BasicAttackProjectileLaunched.expected_impact_time_ms`에 고정되지 않고, finite travel window 안에서 원래 locked target body와 continuous sweep으로 접촉한 첫 시점이다. `expected_impact_time_ms`는 Unity presentation용 초기 예상값이다.

원래 locked target이 impact 전 사망, 소멸, 아군화, 공중 대상 불가 같은 non-range targetability 실패 상태가 되면 projectile은 miss다. 공격자가 발사 후 사망해도 launch 시점의 owner snapshot으로 적대성을 판정하며, live attacker record 존재 여부 때문에 target hostility가 흔들려서는 안 된다.

보스, 정예, 환상체, 이스터에그 직원, 특수 소환체는 일반 침식 직원 preset에 강제로 맞추지 않는다. 이들은 각자의 RON에서 고유 기본 공격 범위 또는 스킬 범위를 명시해야 한다. 의도한 범위가 데이터에 없으면 core는 임의 fallback으로 대체하지 않고 데이터 오류로 처리한다.

## Range Policy

- RON/schema field name은 `range_policy`다.
- 기본값은 `pattern`이며, 기존 `defense_tile_range`를 facing 기준으로 투영한다.
- 전장 전체 valid tile 후보 범위는 `range_policy: whole_field_valid_tiles`로 명시한다.
- `range_policy: whole_field_valid_tiles`와 `defense_tile_range`를 동시에 작성하면 source of truth 충돌이므로 validation failure다.
- Unity-facing skill catalog DTO는 cast target과 step에 `range_policy`를 포함한다. Unity는 이 값을 참고할 수 있지만, range overlay의 최종 source of truth는 여전히 runtime `range_previews` cell DTO다.
- `WholeFieldValidTiles`는 기본 공격과 스킬이 모두 사용할 수 있는 공용 range policy다.
- `WholeFieldValidTiles`는 전장 전체 valid tile을 후보 타일 범위로 사용한다.
- 이 정책은 route 유무와 직접 결합되지 않는다. route-following 유닛도 유닛/profile/skill이 명시적으로 허용하면 `WholeFieldValidTiles`를 사용할 수 있다.
- 보스, 특수 침식 직원, 저격수 같은 특수 유닛은 route 유무와 무관하게 `WholeFieldValidTiles`를 사용할 수 있다.
- 일반 `profile_role: Normal` 침식 직원은 route-following 여부와 무관하게 `WholeFieldValidTiles`를 사용하지 않는다.
- 전장 전체 범위는 후보 타일 범위만 전체라는 뜻이다. 실제 공격 여부는 alive/hostile/air_capable/untargetable/유효 hostile target usefulness와 targeting rule을 따른다.
- 기본 공격이 `WholeFieldValidTiles`를 쓰면 유닛 또는 basic attack의 `targeting_profile`이 대상 선택 source of truth다.
- 스킬이 `WholeFieldValidTiles`를 쓰면 해당 skill data는 명시적인 targeting rule을 가져야 한다.
- `WholeFieldValidTiles` 스킬에 명시 targeting rule이 없으면 validation failure로 처리한다. 조용히 fallback하지 않는다.
- `WholeFieldValidTiles`와 `TileArea` 조합은 전장 광역 스킬을 표현하기 위해 허용한다. 단 skill data가 명시적으로 전장 광역 스킬임을 드러내야 하며, 명시 targeting/effect policy 없이 암묵적으로 전장 전체 피격을 만들지 않는다.
- 적군의 공격 범위 preview는 기본 Unity-facing DTO로 매번 내려보내지 않는다. 플레이어가 조작하는 배치/선택/스킬 preview만 `range_previews` 최종 cell DTO로 내려보낸다.
- 적군 범위가 시각적 경고/telegraph/debug에 필요하면 모든 후보 cell을 보내지 않고, 해당 경고나 스킬로 실제 피해/효과가 적용될 타일만 별도 event/DTO로 내려보낸다.
- Unity는 route-less 여부나 enemy type을 보고 범위를 추론하지 않는다. core가 preview/telegraph용으로 명시해 내려준 최종 cell만 표시한다.

침식 직원 profile role:

- 침식 직원 프로필은 `profile_role`을 가져야 한다.
- `profile_role`은 웨이브 생성용 role mix가 아니라, 해당 침식 직원 프로필의 전투/검증 역할을 나타내는 metadata다.
- 초기 값:
  - `Normal`: 일반 wave 침식 직원. `WholeFieldValidTiles` 금지.
  - `Special`: 저격수, 특수 침식 직원 등. route 유무와 무관하게 `WholeFieldValidTiles` 허용 가능.
  - `LegacyEcho`: 죽은 직원 이스터에그/잔향 계열 정예 침식 직원. 고유 범위/스킬/targeting 허용.
- 일반 침식 직원은 공식 wave 유닛으로 사용할 수 있지만, `WholeFieldValidTiles` 권한은 없다.

## Automatic Hostile Target Usefulness

자동 적대 타겟팅은 피해 또는 적대적 대상 효과 중 하나라도 기대할 수 있는 대상만 고른다.

이 필터는 기본 공격, 스킬 cast target 선택, `RetargetOnStep` 재타겟팅, `TileArea` 시전 가능성 판단에 동일하게 적용한다. 후보 필터 이후의 정렬은 기존 `targeting_profile`, 거리, route progress, 위협도, deterministic tie-breaker를 그대로 사용한다. 데미지 가능 대상과 효과만 가능한 대상 사이에 별도 우선순위 규칙을 추가하지 않는다.

유효 후보 규칙:

1. deterministic expected final damage가 0보다 크면 유효 타겟이다.
2. deterministic expected final damage가 0이어도, 선택한 적 대상 또는 AoE 안의 적 대상에게 적용되는 적대적 target-applied effect가 최소 1개 있으면 유효 타겟이다.
3. deterministic expected final damage가 0이고 적대적 target-applied effect도 없으면 후보에서 제외한다.
4. AoE 스킬은 범위 안에 위 조건을 만족하는 적 대상이 최소 1명 있을 때만 자동 시전 후보가 된다.
5. 후보가 없으면 해당 공격 또는 공격 스킬은 사용하지 않는다.

`deterministic expected final damage`는 core가 현재 알고 있는 피해 타입, 방어/마법 저항, 면역/무효화, 고정 피해, deterministic modifier를 반영한 최종 예상 피해다. 치명타, 명중 실패, 랜덤 편차처럼 실행 시점 확률 결과는 타겟 후보 필터에 사용하지 않는다.

적대적 target-applied effect란 선택된 적 유닛 또는 AoE 안의 적 유닛에게 상태 변화를 적용 시도하는 효과다.

포함:

- debuff
- control/status ailment
- forced movement
- damage-over-time
- stat/defense/resistance reduction
- vulnerability
- mark/targeting modifier
- aggro/threat manipulation
- beneficial-effect removal

제외:

- caster self buff
- ally buff/heal/shield
- resource gain
- cooldown reduction
- visual/audio-only effect
- projectile spawn itself
- summon/spawn effect itself
- global battle state change only
- target에게 아무 상태 변화도 남기지 않는 trigger

`InterruptCast`는 별도 적대적 target-applied effect다. 단, 자동 타겟팅에서 `InterruptCast`만 가진 스킬은 대상에게 현재 pending skill cast가 있을 때만 유효 후보로 본다. 시전 중이 아닌 적에게는 취소할 상태가 없으므로 피해 0, 효과 없음과 같게 처리한다.

타겟팅 단계에서는 해당 효과가 이미 적용되어 있는지, 갱신 가능한지, 중첩 가능한지, 저항되는지, 면역되는지는 따지지 않는다. 단, 그런 제한이 스킬의 target filter에 이미 표현되어 있다면 그 필터는 따른다.

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

## Step Targeting Mode

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
- `SkillTarget::EnemySingle`: cast target이 살아 있는 적이고 해당 step의 `air_capable` 조건을 만족하면 재사용한다. 대상이 죽었거나 더 이상 유효하지 않으면 해당 step은 target 없음으로 처리된다.
- `SkillTarget::CastTarget`: cast target이 unit이면 unit, tile이면 tile을 그대로 재사용한다. cast target이 없고 cast anchor tile이 기록되어 있으면 그 tile을 사용한다.

`RetargetOnStep`은 예약된 step이 실행되는 시점에 대상을 다시 고른다. 공식 DefenseRoute 단일 타겟 스킬에서 사용하는 range source of truth는 해당 step의 `target`, `defense_tile_range`, `air_capable`이다. `SkillStepDef.range_units`는 장기 제거 대상이며, 전염/체인/접촉/오라/명시적 연속 충돌 스킬 같은 별도 메커니즘에도 재사용하지 않는다. 해당 기능에 거리 제한이 필요하면 targeting/delivery 정책 안에 목적별 명시 필드를 추가한다.

- 후속 타격이 첫 타격 대상에 고정되어야 하면 `ReuseCastTarget`을 사용한다.
- 후속 타격이 현재 전장의 유효 대상 중 새 대상을 찾아야 하면 `RetargetOnStep`을 사용한다.
- `RetargetOnStep`은 이전 cast target을 보정하는 모드가 아니라, step-local targeting을 다시 수행하는 모드다.

live RON의 `RetargetOnStep` 사용 예는 Big Bird의 `peck_flurry`, Judgement Bird의 `final_verdict`, Funeral of the Dead Butterflies의 `fragment_quiet_lament`처럼 후속/조건부 타격이 실행 시점의 유효 대상을 다시 잡아야 하는 경우다.

`SkillDef.cast_targeting`과 `SkillStepDef.targeting`의 책임은 다르다.

- `cast_targeting`: 스킬 발동 가능 여부, 최초 cast anchor, Unity event log의 cast target 기준을 정한다.
- `target`: step이 원하는 대상 종류를 정한다.
- `targeting`: 그 step이 cast target을 재사용할지, step 실행 시 다시 target을 고를지 정한다.
- `defense_tile_range`: DefenseRoute에서 후보 타일/범위의 단일 source of truth다.
- `DeliveryDef::TileArea`: 이미 정해진 기준과 `defense_tile_range`를 사용해 실제 피격 타일과 hit target filter를 적용한다. 별도 범위 shape를 만들지 않는다.

`SkillCastTargetingDef::FirstStepTarget` 같은 첫 step 기반 cast target 추론은 장기 제거 대상이다. Live RON뿐 아니라 internal tests/fixtures도 cast-level target을 명시해야 한다. 작성 편의 helper는 허용하지만, helper는 step 목록을 읽어 cast target을 추론하지 않고 caller가 target/range/range_policy/air capability를 직접 넘기는 explicit constructor여야 한다.

Unity 표시용 range overlay는 위 내부 source를 직접 읽지 않고 core가 계산한 `range_previews` 최종 cell만 사용한다.

## Skill Cast Interrupt

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

## Projectile Target Lifecycle

기본 공격 projectile은 locked target을 가진다. impact 시점에 locked target이 죽었거나, 철수했거나, 더 이상 적대적 유효 대상이 아니거나, 공중 타격 조건을 만족하지 못하면 `BasicAttackProjectileImpacted { hit: false }`를 기록하고 damage를 적용하지 않는다. 별도 `ProjectileMiss` event는 만들지 않는다.

스킬 projectile은 step/delivery 정책에 따라 대상 또는 타일을 가진다. impact 시 유효한 첫 피격 유닛이 없으면 `SkillProjectileImpacted { first_hit_unit_id: null }`로 끝난다. 이 이벤트가 스킬 projectile의 no-hit/miss 표현이며, Unity는 별도 miss event를 기대하지 않는다.

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

- `anchor`: event-log center, impact context, VFX 기준점, 또는 `tile_origin: Anchor`가 참조할 위치를 결정한다.
- `tile_origin`: `defense_tile_range`의 `@`를 실제 전장 어디에 놓을지 결정한다. 기본값은 `Caster`이며, `Anchor`를 쓰면 `anchor`가 가리키는 타일을 기준으로 한다. 모르가나 W 같은 지정 지점 장판은 `anchor: CastTarget`, `tile_origin: Anchor`, `tracking: GroundFixed`로 표현한다.
- `tracking`: 지속 영역이 시전자/대상/지면 중 무엇을 따라갈지 결정한다.
- `hit_targets`: 범위 안에서 아군, 적, 전체 중 어떤 유닛을 맞출지 결정한다.
- `include_caster`: 시전자가 같은 타일 필터에 걸릴 때 포함할지 결정한다.
- `tick_policy`, `duration_ms`, `tick_interval_ms`: 지속 범위 스킬의 tick 정책을 결정한다.

`ImpactContext`/`ImpactContextStart` 앵커는 이전 projectile 또는 tile area delivery가 만든 impact context를 재사용한다. 이전 spatial delivery가 없는 step에서 사용하면 잘못된 RON으로 간주하고 로딩 검증에서 실패시킨다. 전투 중 대상이 사라져 런타임 anchor를 해석하지 못하는 경우는 합법적인 전투 상태일 수 있으므로, 정적 데이터 오류와 분리해서 처리한다.

기본 공격은 `TileArea`를 사용하지 않는다. 기본 공격의 `defense_tile_range`는 DefenseRoute에서 단일 공격 대상 후보를 찾는 사거리 패턴이며, 실제 다중 피격은 스킬 step의 `DeliveryDef::TileArea`만 사용한다.

기본 공격에 무기, 침식 직원 profile range preset, 스킬 파편 같은 range source가 없으면 직원용 core fallback은 시전자 자기 타일과 facing 기준 전방 1칸이다. 이 fallback은 공식 침식 직원 live profile의 범위 source가 아니다. Unity는 어떤 경우에도 fallback이나 preset을 재구현하지 않고 `range_previews.basic_attack.cells`를 그대로 표시한다.

## Event Log / Unity 표시 계약

DefenseRoute에서 runtime event log의 `SkillAreaDeclared.shape`는 우선 `tile_pattern`을 사용한다.

```json
{
  "type": "tile_pattern",
  "affected_tiles": [
    { "x": 4, "y": 2 },
    { "x": 5, "y": 2 }
  ]
}
```

Unity는 runtime event의 `affected_tiles`를 범위 경고/효과 타일로 표시한다. 배치/선택/스킬 버튼 preview overlay는 `range_previews`의 최종 cell DTO를 표시한다. Circle/Line/Box/Rectangle/Cone을 재해석하거나 `defense_tile_range`에서 DefenseRoute 범위를 추론하지 않는다.

## 레거시

다음 계약은 DefenseRoute 공식 live path에서 제거됐다.

- `DeliveryDef::Area(shape: Circle/Line/Box/Rectangle/Cone)` 기반 공식 스킬 판정.
- `SkillTarget` 안에 범위형 아군/적 variant를 두는 방식.
- `target.area`와 `delivery.area.shape`가 동시에 범위를 말하던 이중 source of truth.
- geometric AoE 동작을 고정하는 legacy compatibility test.

`DeliveryDef::Area`, `SkillAreaDeliveryDef`, `SkillAreaShapeDef`는 schema/runtime에서 제거 대상이다. 공식 스킬 범위를 막는 방식은 validation adapter가 아니라 `TileArea`와 `defense_tile_range`만 남기는 구조 제거다.

## 작성 규칙

- 단일 타겟 스킬: core 내부에서는 `defense_tile_range` 안의 유효 대상 중 하나를 선택한다.
- 범위 스킬: core 내부에서는 `defense_tile_range`가 산출한 타일 안의 유효 대상 전체에게 효과를 적용한다.
- Unity-facing preview: core가 계산한 `range_previews` 최종 cell만 표시한다.
- live RON에서는 반복되는 `defense_tile_range`를 `SkillDatabase.range_presets`와 `defense_tile_range_preset`으로 작성할 수 있다. preset은 RON 로딩 단계에서 실제 `defense_tile_range`로 해석되며, 전투 런타임은 preset 개념을 모른다.
- `defense_tile_range`와 `defense_tile_range_preset`을 같은 step 또는 explicit cast target에 동시에 작성하면 source of truth 충돌로 간주하고 로딩 검증에서 실패시킨다.
- `ModifyDamage`는 같은 step의 `Damage`에만 적용되는 step-local modifier다. `Damage` 없는 step에는 작성하지 않는다.
- 새로운 범위 모양이 필요하면 geometric shape를 되살리지 말고 `TileRangePattern.rows`를 추가한다.
- 시나리오/RON이 맵의 고정 좌표에 직접 장판을 생성하는 기능은 아직 공식 계약이 아니다. 필요해지면 BattleScenario 조건부 이벤트 확장에서 별도 target payload나 tactical point anchor로 추가한다.
