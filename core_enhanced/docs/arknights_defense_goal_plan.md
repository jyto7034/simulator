# 명일방주식 Defense 전투 Goal 계획

이 문서는 `using-codex-goals-effectively-ko.md`의 원칙을 현재 프로젝트에 맞게 적용한 goal 문서다.

목표는 새 전투 코어를 만드는 것이 아니라, 기존 `BattleScenario -> TacticalPlan -> MovementPlanner -> BattleCore -> map node result` 흐름 안에 명일방주식 Defense 전투를 안전하게 흡수하는 것이다.

## Goal 원칙

- 종료 조건이 명확해야 한다.
- 각 단계는 빠르게 검증할 수 있는 테스트 또는 검색 명령을 가져야 한다.
- 현재 게임 흐름을 더 읽기 쉽고 안전하게 만드는 변경만 한다.
- 미래 가능성만으로 과한 추상화, trait 계층, 별도 전투 모드를 만들지 않는다.
- `Defense` 정책이 모호해지는 지점은 코드 수정 전에 사용자와 의논한다.
- 현재 공식 플레이 흐름에 없는 레거시 방어 계약은 adapter로 감싸지 않고 제거한다.
- 테스트는 내부 추상화 모양보다 실제 플레이 계약을 고정해야 한다.
- 현재 UI는 완성본이 아니다. Unity 리소스, 타일 이미지, 배치 UI 위치, route 표시 방식은 이후 언제든 바뀔 수 있으므로 core 계약은 특정 UI 배치나 임시 리소스 이름에 의존하지 않는다.
- 코드 수정은 임시방편이나 최소 땜질이 아니라 장기 방향으로 진행한다. 단, 처음부터 정답을 안다고 가정하지 않고 작은 실험과 검증을 반복하며 목적에 맞는 구조를 찾는다.
- trial-and-error 과정은 실패한 접근까지 문서의 `실험 기록`에 남긴다. 같은 실패를 반복하지 않는 것이 goal mode의 핵심 안전장치다.
- 레거시는 과감하게 제거한다. 현재 공식 플레이 흐름에 필요 없는 compatibility layer, adapter, legacy test는 보존보다 삭제를 우선한다.
- 정책이 모호하거나 게임 감각을 바꿀 수 있는 결정은 구현 전에 사용자에게 질문한다.

## 최종 목표

`CombatNodeType::Defense`를 명일방주식 고정 방어 전투로 전환한다.

완료된 Defense 전투는 아래 흐름을 만족해야 한다.

```text
전투 노드 선택
-> Defense 전장 preview
-> terrain + route overlay + deployment/platform 정보 표시
-> 직원 배치 확정
-> 전투 시작
-> 아군은 배치 위치에 고정
-> 적은 wave별 route_id를 따라 이동
-> 지상 직원은 block_radius 안에서 block_capacity만큼 적을 저지
-> 저지 초과 적은 통과
-> route 끝에 도달한 적은 DefenseObject 공격
-> 모든 wave 종료 + 필수 적 전멸 + DefenseObject 생존이면 성공
-> DefenseObject 파괴 또는 후퇴면 실패
-> 실패는 노드를 소비하지만 즉시 런 실패는 아님
```

## 범위

포함한다.

- Defense route overlay RON 계약.
- route overlay parser/validator.
- 직원 배치 허용 타입.
- 지상/플랫폼 배치 검증.
- `block_radius`, `block_capacity`, `blockable` 기반 저지 런타임.
- Defense용 고정 아군 이동 정책.
- 적 route 끝 도달 후 DefenseObject 공격 전환.
- Defense 후퇴 정책.
- live Defense 조우 1개가 새 계약을 통과하도록 갱신.
- 대표 run-flow timeline/export 또는 world smoke test.

포함하지 않는다.

- 보스 패턴 AI.
- 조건부/분기형 route.
- 전투 중 재배치.
- 명일방주식 라이프 누수 모델.
- 복수 DefenseObject.
- 고급 포메이션 재배치.
- 밸런스 수치 미세 조정.
- Unity 표현/애니메이션 구현.

## 현재 구현 상태

현재 코드는 Defense의 일부 기반만 갖고 있다.

- `BattleScenario`와 `TacticalPlan`은 존재한다.
- `EnemyMovementPlan::PathToPoint`, `PathAlongPath`는 런타임 구현이 있다.
- `DefenseObject` 역할 유닛은 존재한다.
- `WinCondition::ProtectUnit`은 보호 대상 파괴를 실패로 처리하고, 필수 적이 전멸하면 성공할 수 있다.
- 현재 `CombatNodeType::Defense` 기본값은 `FixedDefense + PathAlongPath + ProtectUnit`이다.

새 목표와의 차이:

- Defense 아군 고정 이동 정책과 logical 저지 런타임은 구현됐다.
- route는 terrain과 1:1인 ASCII overlay로 작성되고 preview/static data에서 검증된다. 기본 Defense runtime은 battlefield routes를 tactical point path로 변환해 `PathAlongPath`로 소비한다.
- 여러 Defense route는 하나의 DefenseObject endpoint로 수렴해야 한다. DefenseObject가 단일 보호 대상이기 때문에, endpoint가 갈라지는 구조는 별도 다중 목표 방어 정책이 정해지기 전까지 validation error다.
- `PathAlongPath` route 끝에서 보호 오브젝트를 공격 대상으로 전환하는 흐름은 구현됐다.
- 저지 초과 적은 logical movement goal 기준으로 통과하지만, 좁은 route에서 시각적 충돌/겹침 감각은 클라이언트 관찰 단계에서 추가 확인해야 한다.
- Defense 성공 조건은 시간 생존이 아니라 모든 wave 종료, 필수 적 전멸, DefenseObject 생존이어야 한다.
- 후퇴 액션은 구현됐다. 비보스 전투는 후퇴 가능하고, Boss 전투는 후퇴 불가다.

## 정책 확정 사항

- `Defense`는 명일방주식 고정 방어 전투다.
- `Encirclement`는 제한 반경 자동전투형 포위 생존/난전으로 남긴다.
- Defense에서 아군은 전투 중 이동하지 않는다.
- 전투 중 재배치는 불가하다.
- 직원은 `GroundOnly`, `PlatformOnly`, `Any` 배치 허용 타입을 가진다.
- 지상 배치 직원만 저지할 수 있다.
- 플랫폼 직원은 저지하지 않고 공격/스킬만 수행한다.
- 저지는 `block_radius` 기반이다.
- 저지 우선순위는 거리 가까운 순, 동률이면 unit id 순이다.
- `block_capacity` 초과 적은 통과한다.
- 적끼리 겹침은 최대한 허용한다.
- `blockable` 적만 저지된다.
- 추후 엘리트/보스 조정을 위해 `block_weight`를 추가할 수 있지만, 1차 구현에서는 필수로 보지 않는다.
- route 하나는 단일 시작점, 단일 종료점, 무분기 선형 경로다.
- wave는 사용할 `route_id`를 명시한다.
- route 끝은 누수 지점이 아니라 DefenseObject 접근/공격 지점이다.
- Defense 실패 조건은 DefenseObject 파괴다.
- Defense 성공 조건은 모든 wave 종료, 필수 적 전멸, DefenseObject 생존이다.
- 비보스 전투는 후퇴 가능하다.
- 후퇴는 노드 실패 소비, 핵심 보상/연구 진행도 없음, 직원 전투불능 없음으로 처리한다.
- 보스 전투와 강제 이벤트 전투는 기본적으로 후퇴 불가다.

## G1. Route Overlay 데이터 계약

### 목표

`battlefield_templates.ron`에 Defense route overlay를 작성할 수 있게 한다.

### 설계 방향

terrain은 실제 맵 source of truth다. route는 terrain과 같은 크기의 overlay로 작성한다.

```ron
terrain: [
    "###########",
    "#PP..#...X#",
    "#PP..#....#",
    "#....#....#",
    "#A........#",
    "###########",
],

routes: [
    (
        id: "main_breach",
        overlay: [
            "           ",
            "      >>>X ",
            "      ^    ",
            "      ^    ",
            " A>>>>^    ",
            "           ",
        ],
    ),
]
```

### 종료 조건

- route overlay row 수가 terrain row 수와 다르면 validation error.
- route overlay column 수가 terrain width와 다르면 validation error.
- terrain 공백이 아닌 곳만 route marker를 가질 수 있다.
- terrain 장애물 위에 route 화살표가 있으면 validation error.
- route 하나는 시작 marker 하나와 종료 marker 하나만 가진다.
- route는 끊기지 않은 단일 경로로 파싱된다.
- route가 분기되면 validation error.
- wave가 알 수 없는 `route_id`를 참조하면 validation error.

### 빠른 검증

```bash
cargo test -p game_core pve_data
cargo test -p game_core ron_loading
```

### 기록

- 성공/실패한 route parser 설계는 이 문서의 `실험 기록`에 남긴다.

## G2. 전투 프로필 배치/저지 스탯

### 목표

직원과 적 전투 프로필에 Defense에 필요한 최소 스탯을 추가한다.

### 필요한 데이터

```rust
pub enum DeploymentAffinity {
    GroundOnly,
    PlatformOnly,
    Any,
}
```

후보 필드:

```text
employee/combat profile:
- deployment_affinity
- block_capacity

enemy/combat profile:
- blockable
- block_weight optional
```

### 종료 조건

- 기존 직원/침식 직원/환상체 데이터가 기본값으로 역호환 로드된다.
- 기본 직원은 지상 배치 가능이며 기본 저지력 1을 가진다.
- 원거리/플랫폼 전용 직원은 저지력 0 또는 저지 불가로 표현할 수 있다.
- 일반 침식 직원은 `blockable = true`다.
- DefenseObject는 배치/저지 대상이 아니다.
- live RON validation이 누락 기본값을 일관되게 처리한다.

### 빠른 검증

```bash
cargo test -p game_core employee_data
cargo test -p game_core corroded_employee
cargo test -p game_core ron_loading
```

## G3. Deployment Zone 지상/플랫폼 검증

### 목표

전투 시작 전 배치에서 직원의 배치 허용 타입과 전장 타일 타입을 검증한다.

### 정책

- `GroundOnly` 직원은 지상 배치칸에만 배치 가능.
- `PlatformOnly` 직원은 플랫폼 배치칸에만 배치 가능.
- `Any` 직원은 양쪽 모두 가능.
- DefenseObject는 플레이어가 배치하거나 이동할 수 없다.
- 전투 중 재배치는 불가하다.

### 종료 조건

- `NodeConfirm`의 `MoveUnit`이 배치 타입을 검증한다.
- stale deployment 재검증도 같은 규칙을 사용한다.
- invalid placement는 노드를 소비하지 않고 `NodeConfirm`에 머문다.
- snapshot/preview가 클라이언트에게 지상/플랫폼 배치 가능 타일을 구분해 줄 수 있다.

### 빠른 검증

```bash
cargo test -p game_core game::world::tests::placement::
cargo test -p game_core game::world::tests::combat::
```

## G4. FixedDefense 아군 정책

### 목표

Defense에서 아군이 이동하지 않고 공격/스킬/저지만 수행하게 한다.

### 설계 방향

`PlayerMovementPlan::FixedDefense`를 추가하거나 동일 의미의 명확한 정책명을 사용한다. 이 정책은 이동/접근 goal을 만들지 않는다.

단, 공격 대상 선택은 유지해야 한다.

우선순위:

1. 자신이 저지 중인 적.
2. 사거리 안의 적.
3. 기존 deterministic target tie-break.

### 종료 조건

- Defense 아군에게 `MoveToPoint` goal이 생성되지 않는다.
- Defense 아군은 사거리 안 적을 공격한다.
- Defense 아군은 스킬 자동 발동/쿨다운 흐름을 유지한다.
- Encirclement의 `HoldDeployment`는 유지된다.

### 빠른 검증

```bash
cargo test -p game_core movement::planner
cargo test -p game_core battle_rapier_movement
```

### 구현 결과

- `PlayerMovementPlan::FixedDefense`를 추가했다.
- `CombatNodeType::Defense` 기본 아군 이동 정책을 `FixedDefense`로 변경했다.
- `FixedDefense`는 플레이어 그룹 목표보다 먼저 적용된다. 따라서 Defense 아군은 `AdvanceToPoint`, `HoldArea` 같은 본대 이동 목표가 있어도 배치 위치를 이탈하지 않는다.
- 사거리 안의 적에게는 `AttackUnit` 목표를 만든다. 이때 `approach_point`는 `None`이므로 사거리 밖 적을 향해 접근하지 않는다.
- `Encirclement`의 `HoldDeployment`는 유지했다.

### 검증 결과

```bash
cargo test -p game_core fixed_defense -- --nocapture
cargo test -p game_core default_black_box -- --nocapture
cargo test -p game_core authored_protect_unit -- --nocapture
cargo test -p game_core encirclement_node_type -- --nocapture
```

## G5. 저지 런타임

### 목표

명일방주식 저지를 전투 규칙으로 구현한다.

### 설계 방향

저지는 물리 충돌 결과가 아니라 `BattleCore`의 명시적 런타임 상태여야 한다.

후보 상태:

```text
BlockState
- blocker_id
- blocked_enemy_ids

EnemyBlockState
- blocked_by
```

저지 판정은 movement tick 전후의 deterministic 단계에서 수행한다.

### 종료 조건

- `block_radius` 안의 blockable 적이 가장 가까운 저지 가능 직원에게 잡힌다.
- 동률은 unit id 순으로 해결된다.
- 직원의 `block_capacity`를 넘는 적은 저지되지 않는다.
- 저지된 적은 이동하지 않는다.
- 저지된 적은 저지자를 우선 공격한다.
- 저지자는 저지 중인 적을 우선 공격한다.
- 적 사망, 직원 사망/전투불능, 강제 이동/넉백, 반경 이탈 시 저지가 해제된다.
- 저지 초과 적은 통과한다.

### 빠른 검증

```bash
cargo test -p game_core movement::planner
cargo test -p game_core battle_rapier_movement
cargo test -p game_core game::battle::core
```

### 주의점

- 유닛 간 충돌이 저지 초과 적 통과를 막으면 안 된다.
- Rapier collision 정책 변경은 Defense에 필요한 최소 범위로 제한한다.
- 적끼리 겹침 허용을 위해 전체 전투의 충돌 감각을 망가뜨리지 않는다. 필요하면 Defense 전용 unit collision policy로 제한한다.

### 구현 결과

- `RuntimeUnit`에 `block_capacity`, `block_radius_units`, `blockable`을 내려보낸다.
- `PlatformOnly` 프로필은 런타임 생성 시 `block_capacity = 0`, `block_radius_units = 0`으로 정규화한다. 플랫폼 직원은 데이터 실수로 block capacity가 들어와도 저지자가 되지 않는다.
- `BattleCore`에 명시적인 `BlockRuntimeState`를 추가했다. 이 상태는 물리 충돌 결과가 아니라 movement goal 생성 전 deterministic하게 갱신된다.
- `FixedDefense`에서만 저지를 활성화한다. 일반 전선형, 회수형, 포위형 전투의 기존 교전 감각을 바꾸지 않는다.
- blockable 적은 `block_radius_units` 안에 들어온 가장 가까운 player blocker에게 배정된다. 동률은 unit id 순이다.
- blocker의 `block_capacity`가 가득 차면 남은 적은 저지되지 않는다.
- 저지된 적은 movement input에서 이동 불가가 되며, 저지자를 `current_target`으로 잡는다.
- 저지자는 자신이 저지 중인 첫 적을 우선 target으로 잡는다.
- 적이 사망하거나, 저지자가 사망하거나, 반경 밖으로 이동하면 다음 refresh에서 저지가 해제된다. 강제 이동/넉백도 위치가 반경 밖으로 바뀌면 같은 방식으로 해제된다.
- `FixedDefense`의 route 적은 저지되지 않은 상태라면 주변 직원을 우발 교전하지 않고 route 진행을 우선한다. 이로써 저지 용량 초과 적은 “통과”할 수 있다.

### 검증 결과

```bash
cargo test -p game_core fixed_defense -- --nocapture
cargo test -p game_core movement::planner -- --nocapture
cargo test -p game_core --test battle_rapier_movement -- --nocapture
cargo test -p game_core game::battle::core -- --nocapture
```

### 남은 주의점

- 현재 단계는 logical block runtime이다. 좁은 route에서 시각적으로 적이 어느 정도 겹쳐 지나가는지, 유닛 충돌 보정이 과하게 보이지 않는지는 Unity 타임라인/클라이언트 관찰 단계에서 추가 확인한다.

## G6. Defense Route End와 DefenseObject 공격 전환

### 목표

적이 route 끝에 도달하면 사라지거나 라이프를 깎는 대신 DefenseObject를 공격하게 한다.

### 종료 조건

- route 끝에 도달한 적은 DefenseObject를 target으로 삼는다.
- DefenseObject가 살아 있으면 적은 계속 공격한다.
- DefenseObject가 파괴되면 전투는 opponent 승리로 끝난다.
- 모든 wave가 끝나고 필수 적이 전멸했으며 DefenseObject가 살아 있으면 player 승리다.
- 기존 `DefendPoint` 누수형 계약을 되살리지 않는다.

### 빠른 검증

```bash
cargo test -p game_core game::battle::core
cargo test -p game_core game::events::combat
```

### 구현 결과

- 기본 Defense 목표/승리 조건을 시간 생존형 계약에서 `ProtectUnit`으로 전환했다.
- `FixedDefense` route 적이 최종 tactical point에 도달하면 보호 대상 `DefenseObject`를 공격 목표로 삼는다.
- route 끝에 도달하기 전의 적은 저지되지 않은 한 주변 직원을 우발적으로 공격하지 않고 route 진행을 우선한다.
- 보호 오브젝트가 파괴되면 기존 `ProtectUnit` 승리 조건에 따라 opponent 승리로 끝난다.
- 필수 적 그룹이 모두 정리되고 보호 오브젝트가 살아 있으면 player 승리로 끝난다.
- `DefendPoint` 누수형 계약은 되살리지 않았다.

### 검증 결과

```bash
cargo test -p game_core fixed_defense -- --nocapture
cargo test -p game_core default_black_box -- --nocapture
cargo test -p game_core authored_protect_unit -- --nocapture
cargo test -p game_core game::battle::core -- --nocapture
cargo test -p game_core game::events::combat -- --nocapture
```

## G7. 후퇴 액션

### 목표

비보스 전투에서 후퇴할 수 있게 한다.

### 정책

- 후퇴 가능: 일반 전투, Defense, Frontline, Encirclement, Recovery.
- 후퇴 불가: Boss, 강제 이벤트 전투.
- 후퇴 결과: 전투 즉시 중단, 노드 실패 소비, 핵심 보상/연구 진행도 없음, 직원 전투불능 없음.

### 종료 조건

- 전투 중 `RetreatCombat` 또는 동등한 액션이 가능하다.
- Boss 전투에서 후퇴하면 `InvalidAction`.
- 후퇴 후 `NodeOutcomeSummary`는 mission failure를 반환한다.
- 후퇴 노드는 소비되고 다음 노드가 열린다.
- 후퇴 자체로 직원 전투불능/트라우마 대량 증가가 발생하지 않는다.
- 후퇴 후 런 실패 판정은 기존 노드 실패 정책과 동일하다.

### 빠른 검증

```bash
cargo test -p game_core game::world::tests::combat::
cargo test -p game_core game::world::tests::map_flow::
```

### 구현 결과

- `PlayerBehavior::RetreatCombat`과 `ActionKind::RetreatCombat`을 추가했다.
- 기본 `InCombatReplay` 상태에서는 후퇴 capability를 제공하되, 현재 전투가 `CombatNodeType::Boss`이면 상태 전환 시 capability에서 제거한다.
- 비보스 전투에서 후퇴하면 현재 노드를 실패로 소비하고 `ViewingMap`으로 복귀한다.
- 후퇴 outcome은 `mission_success: false`, `combat.retreated: true`, `combat.winner: Draw`로 표시한다. 이는 실제 패배 리플레이와 후퇴를 클라이언트가 구분하기 위한 계약이다.
- 후퇴는 보상, 연구 진행도, 직원 전투불능, 트라우마, 경험치 변화를 발생시키지 않는다.
- Boss 전투에서 `RetreatCombat`을 강제로 호출하면 `InvalidAction`이다.

### 검증 결과

```bash
cargo test -p game_core retreat -- --nocapture
```

## G8. Live Defense 조우 전환

### 목표

`defend_black_box_relay`를 새 Defense 계약의 대표 live 조우로 전환한다.

### 종료 조건

- live Defense 조우가 route overlay를 사용한다.
- wave가 `route_id`를 명시한다.
- DefenseObject 접근 지점이 route 끝과 연결된다.
- live RON loading test가 성공한다.
- world smoke/timeline export가 새 Defense 계약을 최소 1회 통과한다.

### 빠른 검증

```bash
cargo test -p game_core ron_loading
cargo test -p game_core map_combat_node_smoke_exports_timeline -- --nocapture
```

### 구현 결과

- `defend_black_box_relay`는 live RON에서 `CombatNodeType::Defense`, `ChokePoint`, `black_box_breach_main` route를 사용한다.
- 해당 조우의 모든 wave는 `route_id: "black_box_breach_main"`을 명시하고 `required_for_victory`를 유지한다.
- RON loading test가 route 시작/끝, route cell의 valid tile 포함 여부, entry spawn zone과 route start 연결, ground deployment zone 존재를 검증한다.
- Defense 전용 world smoke test `defense_combat_node_smoke_exports_timeline`을 추가했다. 이 테스트는 map node preview, route/deployment 계약, node-scoped deployment, `ConfirmEnterNode`, `CombatResolved`, timeline export를 한 번 통과한다.

### 검증 결과

```bash
cargo test -p game_core --test ron_loading -- --nocapture
cargo test -p game_core defense_combat_node_smoke_exports_timeline -- --nocapture
```

## G9. 레거시 정리

### 목표

Defense 전환 후 남은 낡은 방어 기본값과 테스트를 정리한다.

### 삭제 또는 전환 대상

- Defense 기본값의 `HoldDeployment`: 제거 완료. Defense는 `FixedDefense`를 사용하고, `HoldDeployment`는 `Encirclement` 전용으로 유지한다.
- Defense 기본값의 `PathToPoint` 단일 목적지 의존: 제거 완료. 기본 Defense는 route cells를 `PathAlongPath` tactical points로 변환한다.
- Defense 기본값의 시간 생존형 계약: 제거 완료. 기본 Defense는 `ProtectUnit`을 사용한다.
- `Controlled`/`Unstable`/`Collapse` 방식의 Defense 성공 분기: 제거 완료. 위험도는 전장/보상/Recovery hold 같은 다른 정책에만 남고 Defense 성공 조건을 바꾸지 않는다.
- Defense를 제한 반경 자동전투로 가정하는 테스트.
- route 없이 Defense가 작동한다고 가정하는 live 조우.

### 유지 대상

- `DefenseObject` 역할 유닛.
- 보호 대상 파괴 시 전투 실패.
- `Encirclement`의 제한 반경 자동전투.
- `PathToPoint`/`PathAlongPath` 자체. Recovery/Frontline/기타 전술에서 계속 사용 가능하다.

### 빠른 검증

```bash
rg -n "Defense.*HoldDeployment|ProtectUnitForDuration|defense_cleanup_required|default_defense_duration_ms" src tests
cargo test -p game_core
```

### 구현 결과

- 과거 시간 생존형 보호 방어 계약을 runtime/data 계약에서 제거했다. Defense 성공/실패의 source of truth는 `ProtectUnit`, `DefenseObject`, 필수 적 그룹 전멸이다.
- `defense_cleanup_required`, `default_defense_duration_ms` API와 해당 테스트를 제거했다. `Controlled`/`Unstable`/`Collapse` 위험도는 Defense 성공 분기를 바꾸지 않는다.
- 기본 Defense tactical plan은 더 이상 단일 `PathToPoint`를 만들지 않는다. battlefield routes의 cells를 tactical points로 변환하고, route 끝을 `black_box_recovery`로 유지한 뒤 대표 route를 scenario-wide `PathAlongPath`로 소비한다.
- 각 wave의 `route_id`는 enemy spawn group의 `enemy_movement_plan`으로 변환되고, 생성된 runtime enemy에 보존된다. Movement planner는 유닛별 plan을 scenario-wide enemy plan보다 우선 사용하므로, wave별 route 선택이 실제 이동에 반영된다.
- route 끝에서 DefenseObject로 공격 전환할 때도 유닛별 route plan을 우선 사용한다. 따라서 wave override가 있는 적은 scenario-wide route 끝이 아니라 자기 route의 끝점 기준으로 보호 대상 공격 전환을 판단한다.
- 기존 `PathToPoint`/`PathAlongPath` 타입 자체는 유지한다. authored tactical plan, Recovery, Frontline, 기타 전술 목적에서 계속 사용할 수 있기 때문이다.

### 검증 결과

```bash
rg -n "Defense.*HoldDeployment|ProtectUnitForDuration|defense_cleanup_required|default_defense_duration_ms" src tests
cargo test -p game_core pve_data -- --nocapture
cargo test -p game_core game::battle::core -- --nocapture
cargo test -p game_core game::events::combat -- --nocapture
cargo test -p game_core --test ron_loading -- --nocapture
cargo check -p game_server
```

## 완료 기준

이 goal은 아래 조건을 모두 만족할 때 완료로 본다.

- Defense terrain + route overlay가 RON에서 작성 가능하다.
- route overlay validation이 잘못된 route를 잡는다.
- wave가 `route_id`를 통해 이동 경로를 선택한다.
- Defense 배치가 지상/플랫폼 타입을 검증한다.
- Defense 아군은 전투 중 움직이지 않는다.
- 지상 직원은 `block_radius`/`block_capacity`로 적을 저지한다.
- 플랫폼 직원은 저지하지 않는다.
- 저지 초과 적은 통과한다.
- route 끝에 도달한 적은 DefenseObject를 공격한다.
- DefenseObject 파괴 시 Defense 실패다.
- 모든 wave 종료, 필수 적 전멸, DefenseObject 생존 시 Defense 성공이다.
- 비보스 전투 후퇴가 가능하고, Boss 후퇴는 불가하다.
- 실패/후퇴는 노드를 소비하지만 즉시 런 실패가 아니다.
- live Defense 조우가 새 계약을 사용한다.
- 레거시 Defense 기본값과 맥락이 맞지 않는 테스트가 정리된다.

## 빠른 피드백 루프

작업 중에는 전체 테스트보다 아래 순서로 빠르게 돌린다.

```bash
cargo test -p game_core pve_data
cargo test -p game_core ron_loading
cargo test -p game_core movement::planner
cargo test -p game_core game::events::combat
cargo test -p game_core game::world::tests::combat::
```

최종 검증:

```bash
cargo test -p game_core
```

## 실험 기록

장시간 goal mode로 진행할 경우 아래 형식을 유지한다.

### Experiment 1. Route Overlay Parser

- 가설: terrain ASCII를 source of truth로 두고, 같은 크기의 얇은 route overlay만 추가하면 맵 형태와 적 이동 경로를 1:1로 읽을 수 있다.
- 변경: `BattlefieldRoute`, template `routes`, wave `route_id`를 추가했다. route overlay는 공백을 “route 없음”으로 해석하고, `> < ^ v` 화살표와 terrain marker와 일치하는 대문자 endpoint만 허용한다.
- 검증 명령: `cargo test -p game_core combat_preview -- --nocapture`, `cargo test -p game_core game::events::combat::tests -- --nocapture`
- 결과: 성공. route row/column 불일치, 장애물 위 route, 누락 route 참조, Defense wave의 route 누락을 validation에서 잡는다. `defend_black_box_relay`는 `black_box_breach_main` route를 명시하도록 전환했다.
- 다음 결정: G2/G3에서 route를 따라오는 적과 고정 배치/지상·플랫폼 배치 타입을 연결한다.

### Experiment 2. Block Runtime

- 사전 결과: G2/G3 프로필·배치 계약을 먼저 분리해 통과시켰다.
- 가설: 저지 런타임을 넣기 전에 `UnitCombatProfile`과 `CombatPreview`가 지상/플랫폼/저지 가능성 데이터를 명확히 노출하면, 이후 G4/G5는 새 전투 모드 없이 기존 시나리오 흐름에서 해당 데이터를 소비할 수 있다.
- 변경: `DeploymentAffinity(GroundOnly/PlatformOnly/Any)`, `block_capacity`, `block_radius_units`, `blockable`을 `UnitCombatProfile`에 추가했다. `P`는 지상 배치, `T`는 플랫폼 배치로 파싱하고, `DeploymentZoneKind`를 preview에 노출했다. `MoveUnit`과 `ConfirmEnterNode` stale validation이 직원 배치 타입을 검증한다.
- 검증 명령: `cargo test -p game_core employee_default_profile_has_ground_blocking_contract -- --nocapture`, `cargo test -p game_core abnormality_profile_defaults_to_blockable_enemy_without_deployment_capacity -- --nocapture`, `cargo test -p game_core corroded_employee_profile_deserializes_and_builds_combat_profile -- --nocapture`, `cargo test -p game_core battlefield_template_exposes_ground_and_platform_deployment_zones -- --nocapture`, `cargo test -p game_core combat_deployment_rejects_employee_on_incompatible_zone_kind -- --nocapture`, `cargo test -p game_core combat_node_confirm_revalidates_stale_deployment_affinity -- --nocapture`
- 결과: 성공. 기본 직원은 ground-only blocker로 고정됐고, 적/방어 오브젝트는 저지 런타임에서 소비할 수 있는 기본값을 갖는다. preview와 NodeConfirm이 같은 배치 타입 계약을 사용한다.
- 다음 결정: 실제 저지 배정/해제와 Defense 고정 이동 정책은 G4/G5에서 처리한다.

### Experiment 3. Defense Flow Smoke

- 가설: 현재 core는 `ConfirmEnterNode`에서 전투 결과를 계산한 뒤 `InCombatReplay`로 들어가므로, 후퇴는 새 전투 코어가 아니라 “리플레이 수락 전 작전 포기” 액션으로 표현하는 것이 현재 구조에 가장 잘 맞는다.
- 변경: `RetreatCombat` 액션을 추가하고, 비보스 전투에서 노드 실패 소비/맵 복귀를 수행한다. Boss 전투는 allowed action에서 숨기고 강제 호출도 거부한다. `CombatOutcomeSummary`에는 `retreated` 플래그를 추가해 실제 전투 패배와 후퇴 실패를 구분한다.
- 검증 명령: `cargo test -p game_core retreat -- --nocapture`
- 결과: 성공. Defense 후퇴는 보상/연구/트라우마/경험치 없이 노드 실패로 소비되고, Boss 후퇴는 `InvalidAction`으로 거부된다.
- 다음 결정: G8에서 live Defense 조우가 route/deployment/DefenseObject 계약을 모두 통과하는지 RON loading과 world smoke 기준으로 확인한다.

### Experiment 4. Live Defense Scenario Contract

- 가설: G8은 새 데이터 필드를 추가하기보다 live Defense 조우가 이미 G1~G7 계약을 실제로 통과하는지 테스트로 고정하는 단계가 맞다.
- 변경: `ron_loading`의 live PVE 계약 테스트가 `defend_black_box_relay`의 route start/end, route valid tile, spawn zone 연결, ground deployment zone, wave `route_id`를 직접 검증하도록 강화했다. 또한 `defense_combat_node_smoke_exports_timeline` world test를 추가해 Defense preview에서 battle entry와 timeline export까지 통과시킨다.
- 검증 명령: `cargo test -p game_core --test ron_loading -- --nocapture`, `cargo test -p game_core defense_combat_node_smoke_exports_timeline -- --nocapture`
- 결과: 성공. live Defense 조우와 test fixture Defense 조우 모두 새 route/deployment 계약을 통과한다.
- 다음 결정: G9에서 남은 Defense 레거시 용어/기본값/테스트를 실제 live 경로 기준으로 정리한다.

### Experiment 5. Defense Legacy Removal

- 가설: 시간 생존형 보호 방어 계약과 Defense duration/cleanup API는 현재 명일방주식 Defense 흐름에 남길 이유가 없고, 보존하면 Defense 성공 조건을 다시 헷갈리게 만든다.
- 변경: 시간 생존형 보호 방어 enum variant와 RON schema variant, BattleCore 승패 분기, Defense duration/cleanup API를 제거했다. 기본 Defense 적 이동은 단일 `PathToPoint` 대신 battlefield route cells를 변환한 `PathAlongPath`를 사용한다.
- 추가 변경: wave `route_id`를 `ScenarioSpawnGroup.enemy_movement_plan`과 `RuntimeUnit.enemy_movement_plan`에 보존하고, movement planner가 유닛별 route plan을 우선 소비하도록 했다. 또한 여러 Defense route는 하나의 DefenseObject endpoint로 수렴하도록 validation을 추가했다.
- 검증 명령: `rg -n "Defense.*HoldDeployment|ProtectUnitForDuration|defense_cleanup_required|default_defense_duration_ms" src tests`, `cargo test -p game_core pve_data -- --nocapture`, `cargo test -p game_core game::battle::core -- --nocapture`, `cargo test -p game_core game::events::combat -- --nocapture`, `cargo test -p game_core --test ron_loading -- --nocapture`, `cargo test -p game_core defense_combat_node_smoke_exports_timeline -- --nocapture`, `cargo test -p game_core fixed_defense_route_enemy_uses_unit_route_override_for_endpoint_targeting -- --nocapture`, `cargo test -p game_core validate_instance_requires_defense_routes_to_share_endpoint -- --nocapture`, `cargo check -p game_server`
- 결과: 성공. `src`/`tests`에는 제거 대상 레거시가 남지 않았고, core battle/events/PVE/server check 및 live Defense route 계약 테스트가 통과했다. 유닛별 route override가 endpoint 공격 전환에도 적용되는 것을 추가 단위 테스트로 고정했다.
- 다음 결정: G10에서 전체 world/map flow를 기준으로 최종 smoke와 전체 테스트를 돌리고, goal 완료 여부를 증거 기반으로 판단한다.

## 진행 메모

- route overlay는 사람이 보기 쉬워야 하며, 좌표 목록을 손으로 나열하는 방식으로 회귀하지 않는다.
- 명일방주식 Defense를 별도 전투 코어로 만들지 않는다.
- `Defense`와 `Encirclement`의 역할을 섞지 않는다.
- 물리 충돌로 우연히 저지되는 구조를 만들지 않는다.
- UI와 아트 리소스는 아직 고정 계약이 아니다. core는 지상/플랫폼/route/목표/배치 가능 영역을 데이터로 명확히 제공하고, 클라이언트 표현은 교체 가능해야 한다.
- 장기 방향인지 확신하기 어려운 구현은 작게 실험하고, 테스트/문서로 결과를 남긴 뒤 확장한다.
- 레거시가 새 계약 이해를 방해하면 호환 유지보다 제거를 우선한다.
- 정책이 애매하면 구현 전에 사용자와 의논한다.
