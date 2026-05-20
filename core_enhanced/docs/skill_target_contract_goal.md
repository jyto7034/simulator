# Skill Target Contract Migration

작성 목적: 스킬 시스템에서 기준점 선택과 실제 피격 범위를 분리한 C5 리팩토링 결과를 기록한다. 이 문서는 더 이상 임시 목표가 아니라 현재 유지해야 할 스킬 데이터 계약이다.

## 최종 목표

스킬은 다음 두 책임을 분리한다.

```text
SkillTarget
  - 스킬 단계가 어떤 기준 대상을 사용할지 결정한다.
  - 범위나 다중 피격 의미를 갖지 않는다.

DeliveryDef::Area
  - 실제 공간 범위와 피격 대상을 결정한다.
  - shape + anchor + hit_targets + include_caster가 다중 피격의 source of truth다.
```

## 현재 계약

`SkillTarget`은 범위를 들지 않는다.

```rust
pub enum SkillTarget {
    SelfUnit,
    EnemySingle { rule: UnitTargetRule },
    CastTarget,
}
```

각 variant 의미:

- `SelfUnit`: 시전자 자신을 대상으로 한다. 자기 중심 버프, 회복, 자가 효과에 사용한다.
- `EnemySingle`: 단일 적을 선택한다. 최초 적 기준점이 필요한 공격/광역 스킬에 사용한다.
- `CastTarget`: cast-level target 또는 저장된 cast anchor를 재사용한다. 지연 광역, 후속 폭발, 같은 지점을 여러 번 때리는 스킬에 사용한다.

## Area 피격 계약

다중 피격은 `DeliveryDef::Area`만 사용한다.

```ron
delivery: Area(area: (
    shape: Circle(radius_units: 2000000),
    anchor: CastTarget,
    hit_targets: Enemies,
    include_caster: false,
))
```

주요 anchor 의미:

- `CastTarget`: 최초 선택한 대상 또는 타일 기준점에 광역을 생성한다.
- `Caster`: 시전자 중심 광역을 생성한다.
- `ImpactContext`: 직전 projectile/area 충돌 지점에 후속 광역을 생성한다.
- `ImpactContextStart`: 직전 충돌 지점에서 시작하는 선형/방향형 광역을 생성한다.

## RON 작성 예시

적 기준 광역:

```ron
target: EnemySingle(rule: Nearest),
delivery: Area(area: (
    shape: Circle(radius_units: 2000000),
    anchor: CastTarget,
    hit_targets: Enemies,
))
```

자기 중심 회복 광역:

```ron
target: SelfUnit,
delivery: Area(area: (
    shape: Circle(radius_units: 3000000),
    anchor: Caster,
    hit_targets: Allies,
    include_caster: true,
))
```

같은 기준점에 지연 후속타:

```ron
target: CastTarget,
targeting: ReuseCastTarget,
delivery: Area(area: (
    shape: Line(length_units: 5000000),
    anchor: CastTarget,
    hit_targets: Enemies,
))
```

투사체 명중 지점 후속 폭발:

```ron
target: CastTarget,
targeting: ReuseCastTarget,
when: IfPreviousStepDealtDamage,
delivery: Area(area: (
    shape: Circle(radius_units: 1500000),
    anchor: ImpactContext,
    hit_targets: Enemies,
))
```

## 제거된 레거시

다음 계약은 제거되었고 다시 도입하지 않는다.

- `SkillTarget` 안의 범위형 아군/적 variant
- 타일 기반 스킬 범위 타입
- 스킬 타겟 해석 단계의 타일 기반 반경/라인 피격 판정
- `target.area`와 `delivery.area.shape`가 동시에 범위를 말하던 이중 source of truth

## 구현 메모

- `SkillTarget::CastTarget`은 실제 범위가 아니다. 지연 area step이 기존 cast target/anchor를 계속 참조하기 위한 단일 대상 계약이다.
- `DeliveryDef::Instant`는 `SelfUnit`, `EnemySingle`, 제한적인 `CastTarget` 단일 대상에만 사용한다.
- `DeliveryDef::Area`의 실제 피격은 `resolve_instant_area_targets`, persistent area tick, spatial query 계층에서 처리한다.
- cluster 최적화, best-target area selection 같은 미래 기능은 실제 스킬 요구가 생기기 전까지 추가하지 않는다.

## 완료 검증

다음 조건을 유지한다.

- live RON 스킬 데이터는 범위형 `SkillTarget`을 사용하지 않는다.
- source/test/live skill data에 제거된 타일 기반 스킬 범위 키워드가 남지 않는다.
- RON 로딩 테스트와 스킬 회귀 테스트가 통과한다.

권장 검증 명령:

```bash
rg -n "SkillTarget::(Enemies|Allies)|SkillArea::|\\bSkillArea\\b|RadiusChebyshev|length_tiles|radius_tiles|target: (Allies|Enemies)\\(area:|target:(Allies|Enemies)\\(area:" src tests ../game_resources/data/skills
cargo test -p game_core --test ron_loading
cargo test -p game_core --test skill_refactor_validation
cargo test -p game_core --test skill_test_suite
cargo test -p game_core
```
