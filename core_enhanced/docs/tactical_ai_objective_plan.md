# 전투 목적 기반 AI 정책 설계

이 문서는 `BattleScenario`가 전투 목적과 이동 정책을 선언하고, `BattleCore`의 movement planner가 이를 해석하도록 만들기 위한 장기 설계다.

현재 전투는 양측 유닛이 가장 가까운 적을 향해 이동해 교전하는 `FreeEngage` 방식에 가깝다. 하지만 앞으로 전장은 `OpenHall`, `Corridor`, `ChokePoint`, `Ambush`, `Surrounded`, `SplitRoom`, `ObstacleRoom`, `BossArena`처럼 배치 의미가 다른 형태를 가진다. 이 중 `Ambush`는 전투 목적이 아니라 매복 지형/스폰 변칙이다.

따라서 전투 방식을 하나로 고정하지 않는다. 각 전투는 `BattleScenario` 안에 목적과 이동 정책을 선언하고, movement planner는 그 정책에 따라 유닛의 이동 목표를 생성한다.

## 핵심 원칙

- 전투 목적은 전장 아키타입과 조우 데이터가 정한다.
- movement planner는 하드코딩된 “가장 가까운 적 추격”만 사용하지 않는다.
- 기존 전투 동작은 `SuppressAll + FreeEngage + AssaultPlayer`로 보존한다.
- 방어형 전투에서는 유저가 세심하게 배치한 의미를 깨지 않는다.
- 방어형 아군도 완전 고정 포탑이 아니라 anchor 주변의 제한 반경 안에서만 움직인다.
- 단체 이동은 개별 유닛 AI가 아니라 분대/포메이션 목적 계층에서 처리한다.
- 1차 구현은 최소 기반만 넣고, 경로 기반 방어/저지/포메이션은 후속으로 미룬다.

## 목표 구조

```text
BattleScenario
├─ battlefield
├─ groups
├─ events
├─ win_condition
└─ tactical_plan
   ├─ objective
   ├─ player_plan
   ├─ enemy_plan
   └─ group_plans
```

`tactical_plan`은 전투 전체의 의도를 설명한다. 각 유닛은 이 정책을 바탕으로 런타임 이동 목표를 얻는다.

```text
BattleScenario.tactical_plan
-> RuntimeUnit spawn anchor 저장
-> movement planner가 plan 해석
-> MovementGoal 생성
-> movement engine 실행
```

## 전투 목적

```rust
pub enum BattleObjective {
    SuppressAll,
    ProtectUnit { unit_ref: ScenarioUnitRef },
    ProtectUnitForDuration { unit_ref: ScenarioUnitRef, time_ms: u64 },
    HoldArea { area_id: TacticalAreaId, duration_ms: Option<u64> },
    AdvanceToPoint { point_id: TacticalPointId },
    DefeatBoss { boss_ref: ScenarioUnitRef },
    Survive { duration_ms: u64 },
}
```

목적은 승리 조건과 비슷하지만 완전히 같지 않다.

- `WinCondition`은 전투 종료 조건이다.
- `BattleObjective`는 유닛들이 왜 움직이는지를 설명한다.

예를 들어 `DefeatBoss` 전투도 증원 적은 방어 거점으로 접근할 수 있고, 아군은 보스와 증원 대응을 섞어 움직일 수 있다. 그러므로 objective는 movement policy가 읽는 전술 의도이고, win condition은 결과 판정이다.

## 아군 이동 정책

```rust
pub enum PlayerMovementPlan {
    FreeEngage,
    HoldDeployment {
        guard_radius: f32,
        leash_radius: f32,
        chase_radius: f32,
        return_to_anchor: bool,
    },
    CautiousEngage {
        leash_radius: f32,
        chase_radius: f32,
    },
    HoldLine {
        line_id: TacticalLineId,
        forward_limit: f32,
        backward_limit: f32,
    },
    SquadLocalHold {
        default_guard_radius: f32,
        default_leash_radius: f32,
    },
    GroupAdvance {
        group_plan_id: TacticalGroupPlanId,
    },
}
```

### FreeEngage

현재 전투 방식이다.

```text
가장 가까운 적 선택
-> 적이 사거리 밖이면 접근
-> 사거리 안이면 공격
```

기존 테스트와 현재 전투 감각을 유지하기 위한 기본값이다.

### HoldDeployment

방어형 전투의 기본 정책이다.

```text
스폰 위치를 anchor로 기억
-> 사거리 내 적 공격
-> 사거리 밖 적은 무한 추격하지 않음
-> guard_radius 안에서만 짧게 반응 이동
-> leash_radius 밖으로 나가려 하면 추격 중단
-> 타겟이 없으면 anchor 근처로 복귀
```

의도:

- 배치의 의미를 보존한다.
- 전투가 완전히 정적이지 않도록 작은 움직임을 허용한다.
- 포위형, 병목, 매복 지형에서 유저의 배치 판단이 유지된다.

권장 초기값:

```text
guard_radius = 1.5
leash_radius = 2.5
chase_radius = 0.75
return_to_anchor = true
```

### CautiousEngage

교전형이지만 추격을 제한한다.

```text
적을 추격
-> anchor 기준 leash_radius 밖으로 나가지 않음
-> 지나치게 멀어진 적은 포기
-> 다음 가까운 적 또는 anchor 복귀
```

장애물이 많은 전장이나 배치 의미를 어느 정도 보존해야 하는 전장에 적합하다.

### HoldLine

복도형 전장의 전선 유지 정책이다.

```text
전선 line을 기준으로 이동
-> 앞쪽 forward_limit까지만 압박
-> 뒤쪽 backward_limit까지만 후퇴
-> 라인을 크게 벗어나지 않음
```

초기 구현 대상은 아니다. `Corridor` 목적이 구체화된 뒤 추가한다.

### SquadLocalHold

분리된 방이나 소규모 분대 전투를 위한 정책이다.

```text
분대별 local anchor 또는 room anchor 보유
-> 각 분대는 자기 방어 반경 안에서 교전
-> 합류 조건이 발생하면 GroupAdvanceToReconnect로 전환 가능
```

초기 구현 대상은 아니다.

### GroupAdvance

단체 이동 정책이다. 개별 유닛이 각각 적을 추격하지 않고, 분대 중심과 포메이션 슬롯을 따라 움직인다.

```text
분대 중심점 이동
-> 각 유닛은 formation slot으로 이동
-> 사거리 내 적 공격
-> engage_radius 밖 적은 추격하지 않음
-> cohesion_radius 밖으로 벗어나면 분대/slot으로 복귀
```

초기 구현 대상은 아니다. 다만 1차 설계부터 타입 확장 여지를 남긴다.

## 적 이동 정책

```rust
pub enum EnemyMovementPlan {
    AssaultPlayer,
    PathToPoint { point_id: TacticalPointId },
    PathAlongPath { point_ids: Vec<TacticalPointId> },
    PathToArea { area_id: TacticalAreaId },
    SurroundAndCollapse { area_id: TacticalAreaId },
    LocalAssault,
    BossPattern,
}
```

### AssaultPlayer

현재 적 이동 방식이다.

```text
가장 가까운 아군 선택
-> 접근
-> 교전
```

기본값으로 유지한다.

### PathToPoint / PathToArea

명일방주식 방어 전투의 기반이다.

```text
적은 목적지나 경로를 따라 이동
-> 사거리 내 아군 공격 또는 블로킹 대응
-> 목적지 도달 시 별도 결과 처리
```

아직 구현하지 않는다. 방어 거점, 경로, 저지/블로킹 규칙이 확정된 뒤 추가한다.

### SurroundAndCollapse

포위형 전투에서 여러 출현 구역의 적이 중앙 또는 지정 구역으로 압박하는 정책이다.

### LocalAssault

분리된 방 단위로 자기 방 안의 아군을 공격하는 정책이다.

### BossPattern

보스 전용 정책이다. 보스는 일반 추격 AI가 아니라 패턴/위상/스킬 이벤트를 따른다.

## 런타임 상태

`tactical_plan`을 읽기 위해 런타임 유닛은 최소한 spawn anchor를 기억해야 한다.

```rust
pub struct RuntimeUnit {
    pub body: UnitBody,
    pub tactical_anchor: Option<WorldVec2>,
    pub tactical_group_id: Option<TacticalGroupId>,
    ...
}
```

초기 구현에서는 `tactical_anchor`만 필요하다.

스폰 시점:

```text
ScenarioUnitSpawn.position
-> WorldVec2::from_tile_center(position)
-> RuntimeUnit.body.position
-> RuntimeUnit.tactical_anchor = same world position
```

`ScenarioUnitSpawn.position`, `TacticalPoint.position`, 장애물 좌표, 배치 가능 타일은 모두 같은 전장 타일 좌표계를 사용한다. 과거 TFT/hex `PlacementBoard` 좌표 변환은 전투 시나리오 스폰 경로에서 제거됐다.

추후 확장:

- 분대 anchor
- 방/구역 anchor
- 전선 line anchor
- 목적지 waypoint

`BattleScenario.tactical_plan.points`는 1차 목적지 표현으로 사용한다. `PathToPoint` 적은 해당 전술 포인트의 타일 중심으로 이동하고, `PathAlongPath` 적은 이미 도달한 waypoint를 건너뛰며 첫 미도달 waypoint로 이동한다.

## Movement Planner 변경 방향

현재 기본 흐름:

```text
build_continuous_attack_goals()
-> 모든 움직일 수 있는 유닛
-> nearest enemy 또는 sticky target 선택
-> MovementGoal::AttackUnit 생성
```

목표 흐름:

```text
build_continuous_attack_goals()
-> unit side 확인
-> tactical_plan에서 side policy 확인
-> policy별 goal 생성
```

예시:

```text
FreeEngage
-> 기존 nearest enemy 추격

HoldDeployment
-> anchor/leash 계산
-> 사거리 내 적 우선 공격
-> guard_radius 안에서 접근 가능한 적만 짧게 추격
-> anchor에서 멀면 MoveToPoint(anchor)
-> 목표 없으면 NoGoal 또는 anchor 복귀

AssaultPlayer
-> 기존 nearest enemy 추격
```

`MovementGoal` 자체는 당장 크게 바꾸지 않아도 된다.

```rust
pub enum MovementGoal {
    AttackUnit {
        target_id: UnitInstanceId,
        desired_range: f32,
        approach_point: Option<WorldVec2>,
    },
    MoveToPoint {
        point: WorldVec2,
        stop_radius: f32,
    },
}
```

`HoldDeployment`는 `AttackUnit`과 `MoveToPoint(anchor)` 조합으로 1차 구현 가능하다.

## GroupPlan 확장 설계

단체 이동은 개별 유닛 AI의 합으로 처리하지 않는다. 분대 목적과 포메이션 슬롯을 별도 계층으로 둔다.

```rust
pub struct TacticalGroupPlan {
    pub id: TacticalGroupPlanId,
    pub side: Side,
    pub members: TacticalGroupMembers,
    pub objective: GroupObjective,
    pub formation: FormationKind,
    pub cohesion_radius: f32,
    pub engage_radius: f32,
}
```

```rust
pub enum TacticalGroupMembers {
    SpawnGroup(ScenarioGroupId),
    ExplicitUnits(Vec<ScenarioUnitRef>),
    SideAll(Side),
}
```

```rust
pub enum GroupObjective {
    AdvanceToPoint { point_id: TacticalPointId },
    AdvanceAlongPath { point_ids: Vec<TacticalPointId> },
    HoldArea { area_id: TacticalAreaId },
    ReconnectToGroup { target_group_id: TacticalGroupPlanId },
}
```

```rust
pub enum FormationKind {
    Column,
    Line,
    Wedge,
    Loose,
}
```

GroupPlan 처리 흐름:

```text
group center 계산
-> objective 목적지 방향으로 center 이동 목표 생성
-> formation slot 계산
-> 각 유닛에게 MoveToPoint(slot) 또는 제한적 AttackUnit 부여
-> cohesion_radius 밖 유닛은 복귀 우선
```

기본 전투, 포위형 전투, TFT식 전선 전투처럼 아군 전체가 하나의 본대로 움직이는 경우 `SideAll(Player)` 기반 `player_main` 그룹을 사용한다. 분리 방, 호위, 다중 거점 방어처럼 목적이 갈라지는 전투에서만 여러 `TacticalGroupPlan`으로 나눈다.

1차 기반에서는 `TacticalGroupPlan` 데이터 계약, 런타임 멤버십 추적, 단일 tactical point를 목적지로 하는 최소 formation slot 이동, `AdvanceToPoint`의 제한 거리 기반 group center 전진, `AdvanceAlongPath`의 순차 waypoint 전진, `engage_radius` 제한 교전, `cohesion_radius` 기반 슬롯 복귀 우선순위, 대상 그룹의 현재 중심을 향한 최소 합류 목적까지 구현한다. 고급 동적 포메이션 재배치, 그룹 분리/합류 전환 트리거는 후속 실행 정책으로 남긴다.

## 아키타입별 권장 기본 정책

```text
Archetype      Objective           Player Plan              Enemy Plan
---------------------------------------------------------------------------
OpenHall       SuppressAll         FreeEngage / GroupAdvance AssaultPlayer
Corridor       PushOrHoldLine      HoldLine / GroupAdvance   PathToPoint
ChokePoint     ProtectUnit         HoldDeployment            PathToPoint
Ambush         SuppressAll         CautiousEngage            AssaultWithFlankSpawns
Surrounded     HoldArea            HoldDeployment            PathToArea
SplitRoom      SuppressAll now     Cautious/Hold fallback    AssaultPlayer
ObstacleRoom   SuppressAll         CautiousEngage            AssaultPlayer
BossArena      DefeatBoss          MixedPlan                 BossPattern
```

현재 세부 정책은 확정하지 않는다. 아키타입은 기본 정책 추천값만 가진다. `SplitRoom`의 진짜 목적이 될 `SplitOperation`은 추후 구현 대상이므로, 지금은 분리형 전술을 기본값으로 쓰지 않는다.

## 1차 구현 범위

지금 게임이 돌아가는 것을 우선한다. 따라서 1차 구현은 작게 유지한다.

구현 상태:

- 완료: `BattleScenario.tactical_plan` 추가.
- 완료: 기본값은 기존 동작과 같은 `SuppressAll + FreeEngage + AssaultPlayer`.
- 완료: `RuntimeUnit.tactical_anchor`에 스폰 위치를 저장.
- 완료: movement planner가 `tactical_plan`을 읽는 구조로 변경.
- 완료: `PlayerMovementPlan::HoldDeployment` 최소 구현.
- 완료: `PlayerMovementPlan::CautiousEngage`를 별도 제한 추격 정책으로 분리.
- 완료: `BattlefieldArchetype`에서 기본 `tactical_plan` 추천값을 생성하고, `CombatPreview` 기반 map combat scenario에 연결.
- 완료: `TacticalPoint`, `EnemyMovementPlan::PathToPoint`, `EnemyMovementPlan::PathAlongPath` 이동 목표를 추가.
- 완료: `TacticalGroupPlan` 데이터 계약을 추가.
- 완료: 기본 `TacticalPlan`은 `SideAll(Player)` 본대 그룹인 `player_main`을 가진다.
- 완료: scenario spawn group, 명시 유닛, side 전체 기준으로 런타임 유닛의 `tactical_group_id`를 해석하고 `ScenarioRuntimeState.tactical_groups`에 멤버십을 기록.
- 완료: 명시적인 spawn group/unit 그룹은 `SideAll` catch-all 그룹보다 우선한다.
- 완료: `GroupObjective::AdvanceToPoint`와 `HoldArea`는 단일 tactical point를 본대/분대 목적지로 해석하고, 멤버별 formation slot으로 `MoveToPoint` 목표를 만든다.
- 완료: `AdvanceToPoint`는 최종 목적지 슬롯으로 즉시 흩어지지 않고, 현재 그룹 중심을 목표 방향으로 `cohesion_radius`만큼 전진시킨 이동 중심을 사용한다.
- 완료: `AdvanceAlongPath`는 현재 그룹 중심에서 이미 도달한 waypoint를 건너뛰고 첫 미도달 waypoint를 향해 제한 거리 기반으로 전진한다.
- 완료: 비어 있거나 깨진 group path는 기존 `FreeEngage`로 fallback하지 않는다.
- 완료: `HoldArea`는 전진 이동이 아니라 지정 tactical point 자체를 방어/집결 중심으로 유지한다.
- 완료: group objective 이동 중에도 즉시 공격 가능한 적이 있으면 자기방어 교전을 우선한다.
- 완료: group objective가 적용된 유닛은 멀리 있는 적 때문에 기존 `FreeEngage`로 fallback하지 않는다.
- 완료: `engage_radius` 안의 적만 제한 교전 대상으로 삼고, 접근점은 `cohesion_radius` 안으로 제한한다.
- 완료: formation slot에서 `cohesion_radius`보다 멀어진 유닛은 교전보다 슬롯 복귀를 우선한다.
- 완료: `GroupObjective::ReconnectToGroup`은 대상 그룹의 살아있는 멤버 중심으로 합류 목표를 만든다.
- 완료: 합류 대상 그룹이 없거나 전멸한 경우 기존 `FreeEngage`로 fallback하지 않는다.
- 완료: 기본 `FreeEngage`는 기존처럼 적에게 접근한다.
- 완료: `HoldDeployment` 아군은 leash 밖 적을 무한 추격하지 않는다.
- 완료: `HoldDeployment` 아군은 leash 밖으로 밀려난 경우 anchor로 복귀한다.
- 완료: `CautiousEngage` 아군은 가까운 적만 짧게 추격하고, 먼 적 추격과 leash 이탈을 피한다.
- 완료: `PathToPoint` 적은 멀리 있는 아군을 무한 추격하지 않고 전술 포인트로 이동하되, 사거리 안의 아군은 공격한다.
- 완료: `PathAlongPath` 적은 이미 도달한 waypoint를 건너뛰고 첫 미도달 waypoint로 이동한다.
- 완료: 빈 적 path나 깨진 적 path는 기존 `FreeEngage`로 fallback하지 않는다.
- 완료: `WinCondition::ProtectUnitForDuration`은 보호 대상 유닛이 파괴되면 상대 승리로 전투를 종료한다. `Controlled` 방어는 지정 시간 생존 시 즉시 승리하고, `Unstable`/`Collapse` 방어는 지정 시간 생존 후 필수 적 그룹 섬멸까지 요구한다.
- 제거됨: 과거 `WinCondition::DefendPoint`/지점 누수 계약은 live/runtime/data 계약에서 제거됐다. runner/leak AI가 필요해지면 별도 임무 계약으로 새로 설계한다.
- 완료: `WinCondition::RecoverHoldAndExtract`는 필수 적 그룹 정리 후 회수 지점 반경에 들어오면 `RecoveryTargetSecured`를 기록하고, 지정 시간 사수 후 탈출 지점 반경에 들어오면 `ExtractionCompleted`를 기록한 뒤 플레이어 승리로 전투를 종료한다.
- 완료: `CombatNodeType::Recovery` 기본 전술은 `recovery_target -> extraction_point` 순서의 `AdvanceAlongPath`를 사용한다. 이 흐름은 “적이 포진한 지점까지 돌파, 회수 지점 사수, 탈출 지점 복귀”를 표현하기 위한 최소 구현이다.

아직 구현하지 않는다:

- 방어 거점 HP.
- 블로킹/저지.
- 조건부/분기형 waypoint path.
- 고급 formation slot 재배치.
- 그룹 분리/합류 전환 트리거.
- 아키타입별 세부 수치 확정.
- 보스 패턴 AI.

## 후속 구현 순서

1. 적 path를 조건부/분기형 waypoint로 확장.
2. 목적지 도달 결과를 HP/누적 침입/복수 목표 등으로 확장.
3. 저지/블로킹 규칙 논의 후 구현.
4. `TacticalGroupPlan`을 조건부 path, 고급 포메이션 재배치, 그룹 전환 트리거까지 확장.
5. BossArena용 `BossPattern`과 조건부 시나리오 이벤트 연결.

## 설계상 주의점

- `tactical_plan`은 전투를 실행하는 정책이지, 클라이언트 표시 전용 데이터가 아니다.
- 클라이언트는 같은 데이터를 읽어 배치 UI와 브리핑을 보여줄 수 있지만, 실제 이동 판단은 core가 한다.
- 기본값이 기존 동작을 보존해야 리팩토링 중 테스트와 플레이 흐름이 깨지지 않는다.
- 방어형 정책은 유저 배치 의미를 최우선으로 보존한다.
- 단체 이동은 나중에 반드시 별도 계층으로 처리한다. 개별 nearest-target AI에 약간의 보정만 붙이는 방식은 장기적으로 분대 이동을 망가뜨린다.
