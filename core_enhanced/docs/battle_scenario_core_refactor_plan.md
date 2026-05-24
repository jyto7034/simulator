# BattleScenario 기반 전투 코어 리팩토링 계획

이 문서는 `BattleCore`를 덱 기반 사전 배치 모델에서 `BattleScenario` 기반 런타임 스폰 모델로 전환한 설계 기록과 후속 확장 계획이다.

현재 구현 상태: 덱 호환 레이어는 제거됐다. 전투 시작 입력은 `BattleScenario -> BattleCore::new_from_scenario(...)` 하나로 고정한다. 과거 덱 기반 입력 타입/호환 생성자는 다시 만들지 않는다.

## 목적

전투 코어는 노드별 전장, 웨이브, 증원, 조건부 이벤트를 직접 표현해야 한다. 따라서 전투에 참가하는 유닛은 덱을 한 번에 펼친 결과가 아니라, 시나리오 이벤트가 특정 시점에 스폰한 런타임 객체로 다룬다.

필요한 전투 모델:

- 전투 시작 시 모든 적이 존재하지 않아도 된다.
- 일반 전투는 침식된 전 탐사 직원 웨이브를 중심으로 구성한다.
- 환상체는 정예, 보스, 특수 조우, 보상 원천으로 희소하게 사용한다.
- 매복, 증원, 보스 체력 조건, 소환, 필드 이벤트를 같은 구조로 확장할 수 있어야 한다.
- `CombatPreview`가 보여준 전장/스폰/웨이브 정보와 실제 `BattleCore` 입력이 어긋나면 안 된다.

핵심 원칙:

```text
전투에 유닛이 존재하는 이유는 Deck 때문이 아니라 ScenarioAction 때문이다.
```

## 해결한 구조적 문제

현재 전투 시작 경로는 다음 문제를 해결한 상태다.

- 전투 시작 시 모든 적을 미리 생성하지 않아도 된다.
- 초기 배치 유닛과 지연 웨이브가 같은 `SpawnGroup` 경로로 생성된다.
- 예정된 필수 웨이브가 남아 있으면 현재 적이 비어도 조기 승리하지 않는다.
- `CombatPreview.spawn_waves`가 실제 `BattleScenario` enemy group/event로 변환된다.
- 표시용 `EnemyBriefing`과 실제 스폰 계약 `SpawnWave.enemy_entries`가 분리되어 있다.

## 목표 구조

`BattleScenario`는 전투 코어의 메인 입력이다.

```text
BattleScenario
├─ battlefield
│  ├─ width / height
│  ├─ valid_tiles
│  ├─ obstacles
│  └─ zone definitions
│
├─ actors
│  ├─ player employee spawn definitions
│  ├─ enemy spawn definitions
│  └─ optional summon definitions
│
├─ events
│  ├─ AtBattleStart -> SpawnGroup(player_initial)
│  ├─ AtBattleStart -> SpawnGroup(enemy_wave_0)
│  ├─ AtTimeMs(8000) -> SpawnGroup(enemy_wave_1)
│  └─ future: UnitHpBelow / UnitDefeated / WaveCleared
│
├─ win_condition
│  ├─ player all dead -> lose
│  ├─ required enemy groups cleared -> win
│  └─ future: boss killed / survive until time
│
├─ tactical_plan
│  ├─ objective
│  ├─ player movement policy
│  └─ enemy movement policy
│
└─ metadata
   ├─ encounter_id
   ├─ node_id
   └─ seed context
```

`BattleScenario`는 실행 로그가 아니다. 전투 시작 전 확정된 선언형 전투 설계도다.

구분:

```text
BattleScenario = 입력 설계도
BattleEvent    = 런타임 큐에서 처리되는 내부 명령
BattleRuntime  = 현재 전장 상태
Timeline       = 클라이언트/검증/리플레이용 출력 로그
```

## 신규 타입 설계안

구현 위치는 `src/game/battle/scenario.rs`다.

```rust
pub struct BattleScenario {
    pub battlefield: BattleFieldSpec,
    pub groups: Vec<ScenarioSpawnGroup>,
    pub events: Vec<ScenarioEvent>,
    pub win_condition: WinCondition,
    pub tactical_plan: TacticalPlan,
}
```

```rust
pub struct BattleFieldSpec {
    pub width: u8,
    pub height: u8,
    pub obstacles: Vec<Position>,
}
```

```rust
pub struct ScenarioSpawnGroup {
    pub id: ScenarioGroupId,
    pub side: Side,
    pub required_for_victory: bool,
    pub spawns: Vec<ScenarioUnitSpawn>,
}
```

```rust
pub struct ScenarioUnitSpawn {
    pub unit_ref: ScenarioUnitRef,
    pub owned_uuid: Uuid,
    pub source: BattleUnitSource,
    pub level: Tier,
    pub growth_stacks: GrowthStack,
    pub equipped_items: Vec<Uuid>,
    pub position: Position,
}
```

`position`은 진영별 배치 슬롯이 아니라 전장 타일 좌표다. `BattleCore`는 이 값을 `WorldVec2::from_tile_center(position)`으로 변환해 런타임 위치와 `tactical_anchor`를 만든다. 같은 좌표계가 `valid_tiles`, `TacticalPoint`, 장애물, 배치 가능 타일, 적 출현 구역에 사용된다. `valid_tiles`가 비어 있으면 기존 직사각형 전장 전체를 유효 타일로 본다. 비어 있지 않으면 ASCII 템플릿 기반 비정형 전장으로 보고, 포함되지 않은 좌표는 전장 밖이다. 전장 밖 좌표는 `Battlefield::in_bounds`에서 거부되며, 연속 이동 입력에는 `MovementStaticObstacle::void_tile`로 포함되어 Rapier 정적 충돌체로 동기화된다.

### BattleScenario 검증 계약

`BattleScenario`는 전투 코어의 공식 입력이므로, 전투 시작 전에 `BattleScenario::validate()`를 반드시 통과해야 한다. `BattleCore::run_battle(...)`은 런타임 상태를 만들기 전에 시나리오를 검증하고, 잘못된 전투 설계도는 `InvalidStaticData`로 즉시 실패한다.

검증 대상:

- 전장 크기는 0보다 커야 한다.
- 유효 타일, 장애물, 스폰 위치, 전술 포인트는 전장 범위 안에 있어야 한다.
- `valid_tiles`가 비어 있지 않으면 장애물, 스폰 위치, 전술 포인트는 반드시 유효 타일 위에 있어야 한다.
- 같은 장애물 좌표, 같은 시나리오 그룹 id, 같은 유닛 ref, 같은 이벤트 id, 같은 전술 그룹 id, 같은 전술 포인트 id는 허용하지 않는다.
- 스폰의 `side`는 소속 `ScenarioSpawnGroup.side`와 같아야 한다.
- 스폰 위치는 장애물과 겹치면 안 되며, 같은 그룹 안에서 같은 좌표에 둘 이상의 유닛을 스폰할 수 없다.
- `SpawnGroup` 이벤트는 존재하는 그룹만 참조할 수 있다.
- 승리에 필요한 적 그룹은 적어도 하나의 시나리오 이벤트로 스폰되어야 한다.
- `WinCondition`, `BattleObjective`, `EnemyMovementPlan`, `GroupObjective`는 존재하는 tactical point 또는 unit ref만 참조해야 한다.
- `PathAlongPath`와 `AdvanceAlongPath`는 비어 있을 수 없다.
- 전술 그룹의 반경 값은 유한한 0 이상의 값이어야 하며, `ReconnectToGroup`은 자기 자신을 대상으로 삼을 수 없다.

`BattleScenario::empty(...)`는 코어 내부 수동 fixture용으로 유효한 빈 시나리오다. 일반 게임 흐름에서는 `CombatPreview -> BattleScenario` 변환 단계에서 실제 그룹과 이벤트를 작성해야 한다.

```rust
pub enum ScenarioTrigger {
    AtBattleStart,
    AtTimeMs(u64),
    // 이후 단계에서 추가
    UnitHpBelowPercent {
        unit_ref: ScenarioUnitRef,
        percent: u8,
    },
    UnitDefeated {
        unit_ref: ScenarioUnitRef,
    },
    GroupCleared {
        group_id: ScenarioGroupId,
    },
}
```

```rust
pub enum ScenarioAction {
    SpawnGroup {
        group_id: ScenarioGroupId,
    },
    // 이후 단계에서 추가
    ApplyBuff {
        target: ScenarioTarget,
        buff_id: String,
    },
    EndBattle {
        winner: BattleWinner,
    },
}
```

```rust
pub struct ScenarioEvent {
    pub id: ScenarioEventId,
    pub trigger: ScenarioTrigger,
    pub action: ScenarioAction,
    pub once: bool,
}
```

```rust
pub enum WinCondition {
    AllRequiredEnemyGroupsDefeated,
    DefeatUnit {
        unit_ref: ScenarioUnitRef,
    },
    ProtectUnit {
        unit_ref: ScenarioUnitRef,
    },
    SurviveUntil {
        time_ms: u64,
    },
}
```

현재 구현에서는 `AtBattleStart`, `AtTimeMs`, `SpawnGroup`, `EndBattle`, `AllRequiredEnemyGroupsDefeated`, `DefeatUnit`, `ProtectUnit`, `RecoverHoldAndExtract`, `SurviveUntil`을 주 계약으로 지원한다. 과거 `DefendPoint` 지점 누수 계약과 시간 생존형 보호 방어 계약은 live/runtime/data 계약에서 제거됐다.

조건부 이벤트는 `BattleScenario` 2단계 확장으로 미룬다. 지금은 보스 HP 조건, 특정 유닛 사망, 웨이브 전멸, 지점 도달 같은 조건을 런타임에서 평가해 새 이벤트를 발생시키지 않는다. 현재 우선순위는 시간 기반 웨이브, 전장/배치/스폰 정보 일치, 실제 조우 데이터 확장, 전투 흐름 안정화다.

2단계 조건부 이벤트를 구현할 때는 단순 enum 추가가 아니라 아래 계약을 함께 확정해야 한다.

- 조건 평가 시점: 매 이벤트 처리 후, movement tick 후, 피해/사망 처리 후 중 어디에서 평가할지.
- 재발동 정책: `once`, cooldown, 조건이 계속 참일 때 중복 발동 방지.
- 이벤트 우선순위: 같은 `time_ms`에서 조건부 이벤트, 스폰, 투사체, 스킬, 이동의 처리 순서.
- 참조 안정성: 아직 스폰되지 않은 group/unit ref, 이미 사망한 unit ref, optional wave 참조의 처리.
- RON 작성 계약: `BattleScenario`를 직접 저장하지 않는 원칙을 유지하고 `PveEncounter` authoring field로만 노출할지.
- 타임라인 표현: 조건이 만족된 사실과 실제 액션 결과를 어떤 `TimelineEvent`로 노출할지.

## BattleCore 내부 변경 방향

현재:

```rust
pub struct BattleCore {
    event_queue: BinaryHeap<BattleEvent>,
    scenario: BattleScenario,
    scenario_runtime: ScenarioRuntimeState,
    units: HashMap<UnitInstanceId, RuntimeUnit>,
    ...
}
```

`ScenarioRuntimeState`는 시나리오 참조와 실제 런타임 인스턴스 매핑을 관리한다.

```rust
pub struct ScenarioRuntimeState {
    pub spawned_groups: HashSet<ScenarioGroupId>,
    pub resolved_events: HashSet<ScenarioEventId>,
    pub unit_refs: HashMap<ScenarioUnitRef, UnitInstanceId>,
    pub required_groups: HashSet<ScenarioGroupId>,
    pub cleared_required_groups: HashSet<ScenarioGroupId>,
}
```

필요한 메서드:

```rust
spawn_group(group_id, time_ms)
spawn_unit(spawn, side, group_id, spawn_index, time_ms)
schedule_initial_scenario_events()
schedule_attack_for_spawned_unit(unit_instance_id, time_ms)
has_pending_required_enemy_groups()
compute_winner_with_scenario()
```

## BattleEvent 변경

초기 구현에 필요한 이벤트:

```rust
BattleEvent::SpawnGroup {
    time_ms: u64,
    group_id: ScenarioGroupId,
    cause: TimelineCause,
}
```

우선순위는 다음처럼 두는 것이 좋다.

```text
같은 time_ms 기준:
1. SpawnGroup
2. projectile advance / impact
3. buff / skill / attack
4. movement tick
```

이유:

- `time_ms = 0`에서 스폰된 유닛이 같은 시각의 전투 초기 이벤트에 정상 참여해야 한다.
- `AtTimeMs(8000)` 증원이 먼저 생성되고, 이후 같은 시간대의 타겟팅/이동/공격이 새 유닛을 인식할 수 있어야 한다.

## Timeline 규칙

`ScenarioEvent`와 `TimelineEvent`는 섞지 않는다.

```text
ScenarioEvent:
8000ms에 enemy_wave_1을 스폰한다.

TimelineEvent:
8000ms에 실제 unit_instance_id A/B/C가 어떤 좌표에 생성됐다.
```

이미 `TimelineEvent::UnitSpawned`가 있으므로, 런타임 스폰도 같은 이벤트를 기록한다.

다만 향후 클라이언트 연출을 위해 아래 메타 이벤트를 추가할 수 있다.

```rust
TimelineEvent::WaveStarted {
    wave_id: String,
}
```

이 이벤트는 필수는 아니다. 1차 구현에서는 `UnitSpawned`만으로도 충분하다.

## 승패 판정 변경

현재 `compute_winner()`는 현재 살아있는 유닛만 본다.

새 규칙:

```text
플레이어 생존 유닛 없음
-> 패배

현재 적 생존 유닛 없음
AND required enemy group이 모두 스폰됨
AND required enemy group의 모든 유닛이 사망함
AND pending required spawn event가 없음
-> 승리

그 외
-> 전투 계속
```

주의:

- `time_ms = 0` 웨이브가 아직 처리되기 전에는 승리/패배 판정을 하면 안 된다.
- 선택적 증원은 승리 조건에 포함하지 않을 수 있어야 한다.
- 보스전은 `DefeatUnit { boss_main }` 같은 승리 조건으로 확장 가능해야 한다.

## 인스턴스 ID 생성

현재 유닛 인스턴스 ID는 덱 index 기반이다.

```rust
make_instance_id(base_uuid, side, idx)
```

시나리오 모델에서는 group과 spawn index가 필요하다.

```text
instance_id = deterministic(seed, side, group_id, unit_ref, spawn_index)
```

요구사항:

- 같은 `BattleScenario`와 seed는 항상 같은 `UnitInstanceId`를 만든다.
- 같은 base_uuid가 여러 번 스폰되어도 충돌하지 않는다.
- `ScenarioUnitRef`가 있는 주요 유닛은 참조 가능해야 한다.
- 잡몹처럼 개별 참조가 중요하지 않은 유닛도 deterministic id를 가져야 한다.

## CombatPreview와의 연결

현재 `CombatPreview`는 이미 다음 정보를 가진다.

- `width`
- `height`
- `deployment_zones`
- `spawn_zones`
- `spawn_waves`
- `obstacles`
- `enemy_briefing`

목표 연결:

```text
CombatPreview + player deployment + encounter data
-> BattleScenario
-> BattleCore
```

`CombatExecutor`의 역할은 전투 노드의 미리보기/배치/조우 데이터를 `BattleScenario`로 확정하는 어댑터가 된다.

```text
CombatExecutor
├─ player deployment -> player_initial spawn group
├─ CombatPreview.spawn_waves -> enemy spawn groups/events
├─ PveEncounter/enemy data -> ScenarioUnitSpawn source/profile
└─ BattleScenario 생성
```

## 단계별 구현 계획

현재 상태:

- 1단계의 기반 타입은 구현됐다.
- `src/game/battle/scenario.rs`에 `BattleScenario`, `ScenarioSpawnGroup`, `ScenarioEvent`, `ScenarioAction`, `ScenarioTrigger`, `WinCondition`이 추가됐다.
- 과거 덱 호환 레이어는 제거됐다. 전투 시작 경로는 `BattleScenario` 하나다.
- 덱이 필요 없는 코어 내부 테스트 fixture는 `BattleScenario::empty(...)`와 `BattleCore::new_from_scenario(...)`를 사용한다.
- 스킬 테스트 공통 harness는 더 이상 player/opponent deck을 만들지 않고 `BattleScenario`를 직접 만든다.
- `skill_refactor_validation`은 직접 `BattleScenario` fixture를 만들고, 런타임 유닛 조작이 필요한 테스트는 `run_battle_with_post_spawn_setup(...)`로 시나리오 스폰 이후 상태를 수정한다.
- replay validation 테스트는 `TimelineExpectedCounts::from_scenario(...)`를 사용해 시나리오 기준 기대 스폰 수를 검증한다.
- ranged/projectile/basic attack integration 테스트는 더 이상 deck helper를 만들지 않고 `BattleScenario`와 `BattleCore::new_from_scenario(...)`를 직접 사용한다.
- live item skill activation 테스트는 `ScenarioArtifact`와 `ScenarioUnitSpawn.draft.equipped_items`로 장비/아티팩트 loadout을 표현한다.
- movement timeline export 테스트는 export 계약은 유지하되 전투 시작 입력을 `BattleScenario`로 전환했다.
- Rapier movement 테스트는 `BattleScenario`로 전환했고, 정적 장애물은 setup hook이 아니라 `BattleFieldSpec.obstacles`에 작성한다.
- Unity timeline contract export 테스트는 출력 계약을 유지하되 입력 fixture를 `BattleScenario`로 전환했다. 스폰 이후 상태 검사가 필요한 배치 테스트는 `run_battle_with_post_spawn_setup(...)`를 사용한다.
- `CombatExecutor`도 플레이어 배치를 직접 `ScenarioSpawnGroup`과 `ScenarioArtifact`로 변환한다.
- `BattleCore`는 더 이상 `player_info`/`opponent_info`를 직접 보관하지 않고 `scenario`와 `scenario_runtime`을 보관한다.
- `BattleEvent::SpawnGroup`이 도입됐다. 전투 시작 그룹과 지연 웨이브는 모두 이벤트 큐에서 같은 경로로 스폰된다.
- `ScenarioTrigger::AtBattleStart`, `ScenarioTrigger::AtTimeMs`, `ScenarioAction::SpawnGroup`, `ScenarioAction::EndBattle`의 런타임 연결이 있다.
- `UnitSpawned`와 장착 아이템 `ItemSpawned`는 실제 스폰 시점에 기록된다.
- 새로 스폰된 유닛은 해당 시점 기준으로 기본 공격 이벤트를 예약한다.
- `WinCondition::AllRequiredEnemyGroupsDefeated`, `DefeatUnit`, `ProtectUnit`, `RecoverHoldAndExtract`, `SurviveUntil`이 `compute_winner`에 반영됐다. 과거 `DefendPoint`와 시간 생존형 보호 방어 호환 경로는 제거됐다.
- 첫 필수 웨이브를 처치해도 아직 스폰되지 않은 필수 웨이브가 남아 있으면 조기 승리하지 않는 테스트가 추가됐다.
- `CombatExecutor`가 `CombatPreview.spawn_waves`를 직접 `BattleScenario`의 enemy spawn group/event로 변환한다.
- map/node combat은 `BattleCore::new_from_scenario(...)`를 사용한다.
- `EnemyBriefing`과 실제 스폰 계약이 분리됐다. `EnemyBriefing`은 표시/정찰 정보이고, `SpawnWave.enemy_entries`가 실제 스폰 수량/종류/티어를 담당한다.
- `PveEncounter.waves`가 실제 조우 작성 계약이다. authored wave composition은 `PveWaveData { id, time_ms, enemies }`로 작성하고, `BattlefieldGenerator`는 이를 `SpawnWave.enemy_entries`로 변환한다.
- live RON의 legacy `units` 기반 encounter와 `PveEncounter.units` 호환 필드는 제거됐다.

### 1단계: 타입 추가와 전투 시작 경로 단일화

목표:

- `BattleScenario` 타입을 추가한다.
- 새 테스트와 새 전투 시작 경로는 `BattleScenario`를 직접 만든다.
- 시나리오는 전투 시작 전에 `BattleScenario::validate()`로 검증한다.

작업:

- `src/game/battle/scenario.rs` 추가.
- `BattleScenario::empty(...)` 추가. 내부 테스트/수동 fixture는 빈 시나리오를 사용한다.
- `BattleCore::new_from_scenario(...)` 추가.
- `run_battle_with_hooks(...)` 시작 시점에 `scenario.validate()?`를 호출한다.
- `run_battle_with_post_spawn_setup(...)` 추가. 시나리오의 초기 스폰이 완료된 뒤, 전투 시작 훅이 실행되기 전에 테스트/시나리오 준비 상태를 주입할 수 있다.
- `TimelineExpectedCounts::from_scenario(...)` 추가. replay/validation 테스트는 덱이 아니라 시나리오를 기준으로 기대 스폰 수를 계산한다.
- replay/validation 테스트도 `TimelineExpectedCounts::from_scenario(...)`를 사용한다.

완료 기준:

- 전투 테스트가 모두 `BattleScenario` 기반으로 통과한다.
- 전투 시작 타입/함수는 시나리오 기반으로만 유지한다.
- 잘못 작성된 시나리오는 런타임 상태 생성 전에 실패한다.

### 2단계: 스폰 API 분리

목표:

- 초기 유닛과 런타임 웨이브 유닛이 같은 스폰 경로를 사용한다.
- 유닛 생성 로직은 `spawn_unit_from_scenario` 중심으로 유지한다.

작업:

- `spawn_unit(...)` 메서드 추가.
- `RuntimeUnit` 생성, `battlefield.place`, 아이템 생성, 타임라인 `UnitSpawned` 기록을 하나의 경로로 묶는다.
- 전투 시작 유닛도 이 경로로 생성한다.

완료 기준:

- 초기 유닛 생성과 런타임 유닛 생성이 같은 함수를 사용한다.
- `UnitSpawned`는 생성 시점에 기록된다.

상태: 완료. `spawn_scenario_group(...)`가 `ScenarioSpawnGroup`을 런타임 유닛/아이템/필드 배치로 변환하고, `SpawnGroup` 이벤트 처리부가 타임라인 기록과 공격 스케줄링을 담당한다.

### 3단계: `SpawnGroup` 이벤트 도입

목표:

- `AtBattleStart`와 `AtTimeMs` 기반 스폰을 이벤트 큐로 처리한다.

작업:

- `BattleEvent::SpawnGroup` 추가.
- `BattleEvent::time_ms`, `priority`, `Ord` tie-breaker에 반영.
- `process_event`에 `SpawnGroup` 처리 추가.
- `run_battle_with_setup`에서 scenario events를 큐에 등록한다.
- `init_initial_events`는 “현재 존재하는 유닛 전체”가 아니라 “스폰된 유닛 각각에게 AttackStart 예약”으로 바꾼다.

완료 기준:

- `time_ms = 0` 스폰이 기존 초기 전투와 동일하게 동작한다.
- `time_ms > 0` 스폰 테스트가 통과한다.
- 새로 스폰된 유닛이 타겟팅, 이동, 기본 공격, 스킬 타겟팅에 포함된다.

상태: 완료. `BattleEvent::SpawnGroup`이 이벤트 큐 최우선 순위로 처리되며, `AtTimeMs` 지연 스폰 테스트가 통과한다.

### 4단계: 승패 판정 시나리오화

목표:

- 예정된 필수 웨이브가 남아 있으면 조기 승리하지 않는다.

작업:

- `compute_winner()`를 `compute_winner_with_scenario()`로 교체 또는 내부 확장한다.
- required group 상태를 추적한다.
- 선택적 group은 승리 조건에 포함하지 않는다.

완료 기준:

- 현재 적이 모두 죽어도 5초 뒤 필수 웨이브가 남아 있으면 전투가 계속된다.
- 마지막 필수 웨이브까지 정리하면 승리한다.
- 플레이어 전멸은 예정 웨이브와 무관하게 패배한다.

상태: 1차 완료. 필수 enemy group 기반 승리, 특정 유닛 처치 승리, 생존 시간 승리 조건을 지원한다. 조건 트리거 기반 동적 이벤트는 아직 후속 단계다.

### 5단계: `CombatExecutor` 전환

목표:

- `CombatExecutor`가 전투 시작용 `BattleScenario`를 만든다.

작업:

- `CombatPreview.spawn_waves`와 encounter enemy data를 `ScenarioSpawnGroup`으로 변환.
- player deployment를 `player_initial` group으로 변환.
- `BattleCore::new_from_scenario` 사용.

완료 기준:

- `CombatPreview`의 field size, obstacles, spawn waves가 실제 전투 입력의 source of truth가 된다.
- 플레이어 시작 배치는 `ScenarioSpawnGroup`으로 직접 변환된다.

상태: 완료. `CombatExecutor`는 player deployment를 `player_initial` group으로, `CombatPreview.spawn_waves`를 enemy scenario groups/events로 변환한다. `BattleCore::new_from_scenario(...)`를 사용하며, 지연 wave가 실제 `UnitSpawned` 시점으로 기록되는 테스트가 추가됐다.

남은 정리:

- 웨이브 작성 계약은 `PveEncounter.waves -> CombatPreview.spawn_waves -> BattleScenario` 단일 흐름으로 고정됐다.
- `EnemyBriefing`은 표시용 정보로만 유지하고, 실제 스폰 수량/종류/티어를 다시 추론하는 경로를 만들지 않는다.
- `PveEncounter`는 최소 하나의 wave를 반드시 가져야 한다.
- `CombatExecutor`는 `CombatPreview.spawn_waves`가 비어 있을 때 `PveEncounter.waves`에서 몰래 첫 웨이브를 복원하지 않는다. 이 경우 작성 오류로 보고 전투 시작을 실패시킨다.
- 전투 미리보기와 실제 전투 입력이 어긋나면 fallback을 추가하지 말고, `BattlefieldGenerator` 또는 조우 데이터 작성 계약을 수정한다.

### 5.5단계: PveEncounter 시나리오 작성 오버라이드

상태: 완료.

`BattleScenario`를 RON에 직접 저장하지 않는다. `BattleScenario`는 여전히 전투 시작 직전에 만들어지는 런타임 입력이고, RON 작성자는 `PveEncounter`에 조우 의도를 작성한다. 변환 흐름은 다음 하나로 고정한다.

```text
PveEncounter authored data
-> BattlefieldGenerator / CombatPreview
-> CombatExecutor
-> BattleScenario::validate()
-> BattleCore
```

추가된 작성 필드:

- `PveEncounter.node_type`: 전투 노드 목적 분류. 전장 형태가 아니라 왜 이 전투를 수행하는지를 표현한다.
- `PveEncounter.battlefield`: 전장 아키타입, 크기 등급, width/height 오버라이드.
- `PveEncounter.tactical_plan.points`: 전술 포인트 목록.
- `PveEncounter.tactical_plan.objective`: 전투 목적 오버라이드.
- `PveEncounter.tactical_plan.enemy_plan`: 적 이동 계획 오버라이드.
- `PveEncounter.win_condition`: 승리/패배 조건 오버라이드.
- `PveWaveData.spawn_zone_ids`: 웨이브별 출현 구역 고정.
- `PveWaveData.required_for_victory`: 해당 웨이브가 승리 조건에 필요한지 여부.
- `PveWaveData.source`: 향후 웨이브 작성 source. 수동 작성은 `Manual(Vec<PveWaveEnemyData>)`, 침식 직원 생성은 `GeneratedCorroded { preset_id, budget_override, seed_salt }`를 사용한다. 현재 구현은 기존 `enemies` 직접 작성이 `Manual`과 같은 의미다.
- `PveWaveEnemyData.kind`: 적의 세계관/전술 분류. 현재 `CorrodedEmployee`, `Abnormality`, `FacilityEntity`를 표현할 수 있고, `FacilityEntity`는 아직 실제 구현 대상이 아니다.
- `PveWaveEnemyData.profile_id`: `CorrodedEmployee`일 때 사용하는 침식 직원 전투 프로필 id.

기본값:

- `battlefield`가 없으면 기존 map category/seed 기반 `BattlefieldGenerator` 추천값을 사용한다.
- `node_type`이 없으면 map category와 전장 아키타입으로 기본 목적을 추론한다. 예를 들어 `Corridor`는 `Frontline`, `ChokePoint`는 `Defense`, `Surrounded`는 `Encirclement`, `Ambush` 지형은 기본 `Suppression`, boss category는 `Boss`가 된다.
- 노드 전투가 시작되면 `CombatPreview.node_type`을 `CombatBattleState.node_type`으로 복사한다. 전투 시작 후 스냅샷, 보상 처리, 후속 이벤트는 조우 데이터를 다시 추론하지 않고 활성 전투 상태의 목적 타입을 기준으로 삼는다.
- `tactical_plan`이 없으면 `CombatPreview.archetype` 기반 기본 전술 계획을 사용한다.
- `win_condition`이 없으면 `AllRequiredEnemyGroupsDefeated`를 사용한다.
- `spawn_zone_ids`가 비어 있으면 아키타입별 기본 출현 구역 추천값을 사용한다.
- `required_for_victory`는 기본 `true`다.
- `kind`는 현재 호환성을 위해 기본 `Abnormality`다. 일반 웨이브를 침식 직원 중심으로 바꾸는 데이터에서는 `CorrodedEmployee`를 명시하고 `profile_id`를 반드시 작성한다.
- 현재 라이브 데이터의 기본 침식 직원 프로필은 guard/rusher/marksman/bruiser/medic/veteran 역할로 나뉜다. 새 일반 웨이브는 먼저 이 프로필 조합으로 표현하고, 환상체 본체는 정예/보스/특수 위협으로 유지한다.
- 침식 직원의 seed 기반 변이는 외형 표현 전용이다. 스탯은 seed로 roll하지 않고 `profile_id`, 난이도, 전투 프로필 데이터로 확정한다.
- `SpawnWaveEnemyEntry.appearance_seeds`는 `CorrodedEmployee` 엔트리에서 count만큼 생성되는 표시 전용 seed 목록이다. `Abnormality`와 아직 미구현인 `FacilityEntity` 엔트리에서는 비워 둔다.
- 향후 클라이언트 외형 조합이 더 구체화되면 `appearance_pool_id`, `resolved_part_ids` 같은 표시 전용 필드를 preview 계층에 추가할 수 있다. 이 필드는 전투 결과를 바꾸면 안 된다.

침식 직원 웨이브 생성기 구현 상태:

- `CorrodedEmployeeWaveGenerator`는 완전한 조우 생성기가 아니라 침식 직원 웨이브 구성 정책 레이어다.
- source of truth는 별도 `CorrodedWavePreset` RON이다. preset은 `id`, `difficulty`, `pressure`, `role_mix`, `budget`, `count_range`, optional constraints를 가진다. 현재 live 파일은 `game_resources/data/enemies/corroded_wave_presets.ron`이다.
- `PveWaveSource::GeneratedCorroded`는 preset id와 override만 들고, 조우 RON 안에 긴 역할군 가중치와 예산값을 반복하지 않는다.
- 현재 생성 입력은 preset, preview seed, wave index, `seed_salt`, `budget_override`다. 이후 `node_type`, `difficulty`, `risk_level`, `battlefield_archetype`, act/depth, `player_squad_power`를 추가 입력으로 확장할 수 있다.
- 생성 출력은 preview 단계의 `SpawnWave.enemy_entries`다. 이후 전투 시작 경로는 변하지 않는다.
- 생성은 `CombatPreview` 생성 시점에 확정한다. `BattleScenario`는 확정된 `SpawnWave.enemy_entries`만 받으며 전투 중 generator를 다시 호출하지 않는다.
- 같은 입력과 seed는 항상 같은 웨이브를 생성해야 한다. 재현성, 리플레이, 클라이언트 preview 일치를 우선한다.
- profile stats는 seed로 흔들지 않는다. generator는 역할군 선택/수량/외형 seed/스폰 변주만 결정하고, 전투 스탯은 `CorrodedEmployeeProfileDatabase`가 결정한다.
- 도입 방식은 하이브리드다. 기존 `PveWaveData.enemies`는 하위 호환 수동 source로 유지하고, 명시적 `PveWaveSource::Manual`도 사용할 수 있다. 새 live 조우 또는 반복이 심한 조우부터 `GeneratedCorroded`로 전환한다.
- 현재 `recover_black_box_archive`는 `archive_sentry_pair`, `breach_response_light` preset을 참조해 생성형 침식 직원 웨이브를 사용한다.
- 현재 `defend_black_box_relay`는 `black_box_breach_probe`, `black_box_breach_pressure` preset을 참조해 생성형 침식 직원 웨이브를 사용한다. 이 조우는 live RON에서 `Defense` 기본 전술 계약을 검증하는 대표 데이터다.

작성 검증:

- `PveEncounterDatabase::validate_indexes()`는 중복 wave id, 빈 wave enemy 목록, 중복 tactical point id, 존재하지 않는 tactical point 참조를 로딩 단계에서 잡는다.
- 라이브 RON 조우 데이터는 `node_type`을 반드시 명시한다. 테스트/생성용 데이터에서는 기본 추론을 사용할 수 있지만, 실제 작성 데이터는 목적 타입을 문서화된 계약으로 남긴다.
- live 작성 가능 전투 목적은 `CombatMissionPolicy::LIVE_SUPPORTED_NODE_TYPES`로 중앙화한다. 현재 허용값은 `Suppression`, `Defense`, `Frontline`, `Encirclement`, `Recovery`, `Boss`다.
- `SplitOperation`은 추후 분대 분리/다중 목표/UI가 준비된 뒤 활성화할 deferred 타입이다. enum은 남기지만 live RON 검증은 이를 거부한다.
- 전장 범위, 스폰/장애물 겹침, 존재하지 않는 런타임 유닛 ref 등 최종 계약은 `BattleScenario::validate()`가 전투 시작 전에 다시 잡는다.

전장 아키타입과 전투 노드 목적은 분리한다.

- `BattlefieldArchetype`은 필드 형태, 배치 구역, 출현 구역, 장애물 배치를 설명한다.
- `CombatNodeType`은 진압형, 방어형, 전선형, 포위형, 회수형, 보스형 같은 임무 목적을 설명한다. 분리형 `SplitOperation`은 타입만 남겨두고 현재 live 조우/기본 목적에서는 사용하지 않는 추후 구현 대상이다.
- 매복은 전투 목적이 아니라 전장/스폰/웨이브 변칙이다. 따라서 `BattlefieldArchetype::Ambush`와 `SpawnZoneKind::Ambush`는 유지하지만 `CombatNodeType`으로는 표현하지 않는다.
- 같은 `ChokePoint` 전장도 `Defense` 블랙박스 방어전이 될 수 있고, `Recovery` 회수전이 될 수도 있다.
- 클라이언트와 보상 정책은 가능한 한 `CombatPreview.node_type`을 기준으로 목적/보상 정체성을 표현하고, 세부 배치는 `archetype`과 전장 데이터에서 읽는다.
- 전투 진입 후 클라이언트는 selected event snapshot의 `node_type`을 읽어 현재 전투의 목적을 표시할 수 있다. 이 값은 preview에서 확정된 값과 동일해야 한다.
- 현재 `CombatExecutor`는 `CombatNodeType::Defense` 조우가 별도 `tactical_plan`과 `win_condition`을 작성하지 않아도 기본 블랙박스 방어 흐름을 만든다. 기본 지점 id는 `black_box_recovery`이고, 주 배치 구역 근처에 `black_box_recovery_device` 보호 오브젝트를 시나리오가 주입한다. 아군은 `FixedDefense`, 적군은 transitional `PathToPoint`, 승패는 `ProtectUnit(black_box_recovery_device)`를 사용한다.
- 보호 오브젝트는 플레이어가 임의로 옮길 수 없고, 이동/공격/스킬 사용을 하지 않는 `BattleUnitRole::DefenseObject` 역할 유닛이다. 이 유닛이 파괴되면 상대 승리다. 기본 Defense는 시간 생존형이 아니라 필수 적 그룹이 모두 정리되고 보호 오브젝트가 살아 있으면 승리한다.
- `CombatMissionRisk`는 적 종류 tier가 아니라 임무 위험도다. 현재 기본값은 조우 `RiskLevel`에서 추론한다. `ZAYIN`/`TETH`는 `Controlled`, `HE`/`WAW`는 `Unstable`, `ALEPH`는 `Collapse`다.
- `TimelineEvent::UnitSpawned`는 `role`을 포함한다. 클라이언트는 `DefenseObject`를 배치 가능한 아군이 아니라 고정 방어 목표로 표시해야 한다.
- 과거 `DefendPoint`/지점 도달 누수 계약은 제거됐다. 관문/침투 저지처럼 “지점 도달 누수”가 목적일 때는 기존 방어 오브젝트 계약을 재사용하지 말고 새 leak-runner 임무 계약으로 설계한다.
- live RON의 첫 방어형 조우는 `defend_black_box_relay`다. 이 조우는 작성 전술/승리 조건을 일부러 비워 core 기본 `Defense` 계약을 검증하고, ChokePoint 전장에서 생성형 침식 직원 웨이브를 보호 오브젝트 방향으로 압박시킨다.
- 현재 `CombatExecutor`는 `CombatNodeType::Recovery` 조우가 별도 `tactical_plan`과 `win_condition`을 작성하지 않아도 기본 회수 흐름을 만든다. 기본 지점 id는 `recovery_target`, `extraction_point`이고, 아군 본대는 `AdvanceAlongPath([recovery_target, extraction_point])`로 돌파 후 복귀한다. 승패는 `RecoverHoldAndExtract(target_radius: 0.75, extraction_radius: 0.75, hold_duration_ms)`를 사용한다.
- 회수전에서 `RecoverHoldAndExtract`는 필수 적 그룹이 정리된 뒤 회수 지점 반경에 들어오면 `RecoveryTargetSecured`를 기록한다. 이후 같은 지점에서 지정 시간 동안 버틴 뒤 탈출 지점 반경에 들어오면 `ExtractionCompleted`를 기록하고 플레이어 승리로 끝난다.
- live RON의 첫 회수형 조우는 `recover_black_box_archive`다. 이 조우는 작성 전술/승리 조건을 일부러 비워 core 기본 `Recovery` 계약을 검증하는 대표 데이터로 사용한다.
- 현재 `CombatExecutor`는 `CombatNodeType::Frontline` 조우가 별도 `tactical_plan`과 `win_condition`을 작성하지 않아도 기본 전선 교전 흐름을 만든다. 아군은 `CautiousEngage(leash_radius: 5.0, chase_radius: 1.5)`로 제한적 전진을 하고, 승패는 `AllRequiredEnemyGroupsDefeated`를 사용한다.
- 현재 `CombatExecutor`는 `CombatNodeType::Encirclement` 조우가 별도 `tactical_plan`과 `win_condition`을 작성하지 않아도 기본 포위 생존 흐름을 만든다. 기본 지점 id는 `survival_anchor`이고, 아군 본대는 `HoldArea`로 anchor 주변을 지키며, 승패는 `SurviveUntil(45_000ms)`를 사용한다.
- 작성자가 `tactical_plan` 또는 `win_condition`을 명시하면 작성 데이터가 우선한다. 따라서 특수 방어전/회수전/전선전/포위전은 RON에서 별도 지점, 경로, 허용 누수, 생존 시간, 회수/탈출 반경으로 확장할 수 있다.
- `SplitOperation`은 추후 구현 대상이다. 실제 도입 전에는 split-squad 배치 UI, 여러 `TacticalGroupPlan`, 다중 목표 상태 추적, 동시 점령/저지 조건을 함께 설계해야 한다. 현재 `SplitRoom` 전장은 존재하지만, 미작성 fallback 목적은 `Suppression`으로 둔다.
- 보상 정책은 `RewardMetadata.tags`를 기준으로 확장한다. 예를 들어 `Defense`는 `ResearchProgress`, `Recovery`는 `Equipment`/`Narrative`, `Suppression`은 `SkillFragment`/`Equipment` 보상 후보를 우선할 수 있다. 태그는 의미 분류이고, 실제 지급 처리는 `RewardEffect`가 담당한다.
- 실제 노드 전투의 보상 해석은 활성 전투의 `CombatNodeType`과 작성된 `PveEncounter.node_type`이 일치해야 한다. 불일치하면 정적 데이터 오류로 간주한다.
- `CombatRewardPolicy`는 이 연결을 검증하는 전용 정책 레이어다. 전투 보상 후보가 비어 있지 않다면 해당 전투 목적의 featured tag를 최소 하나 포함해야 하며, `Forbidden` 같은 비전투 보상 태그는 허용하지 않는다.
- `GrantSkillFragmentResearch`는 `ResearchProgress` 태그의 첫 실제 지급 효과다. 방어전/블랙박스 회수 보상은 이 효과를 통해 특정 파편 연구도를 올리고, 파편 소유권 지급은 별도 효과/정책으로 유지한다.

RON 예시:

```ron
PveEncounter(
    id: "defense_route",
    abnormality_id: "enemy",
    difficulty: 2,
    risk_level: HE,
    battlefield: Some((
        archetype: Some(ChokePoint),
        size_class: Some(Small),
    )),
    tactical_plan: Some((
        points: [
            (id: "black_box_recovery", position: (x: 3, y: 6)),
        ],
        objective: Some(ProtectUnit(unit_ref: "black_box")),
        enemy_plan: Some(PathToPoint(point_id: "black_box_recovery")),
    )),
    win_condition: Some(ProtectUnit(unit_ref: "black_box")),
    waves: [
        (
            id: "wave_0",
            time_ms: 0,
            spawn_zone_ids: ["north_entry"],
            required_for_victory: false,
            enemies: [
                (abnormality_id: "enemy", tier: I, count: 1),
            ],
        ),
    ],
)
```

### 6단계: 조건부 이벤트 확장

이 단계는 1차 구현 대상이 아니다.

현재 결정: 구현하지 않고 설계 슬롯만 확보한다. 조건부 이벤트는 보스전/특수 조우를 실제로 작성하면서 필요성이 검증된 뒤 구현한다. 그 전까지는 `AtBattleStart`와 `AtTimeMs` 기반 스폰, `WinCondition` 기반 승패 판정, `TacticalPlan` 기반 이동 목적을 안정화한다.

구현 진입 조건:

- 라이브 조우 데이터가 시간 기반 웨이브만으로 표현하기 어려운 보스/특수 조우를 요구한다.
- 조건 평가 주기와 재발동 정책을 문서와 테스트로 고정할 수 있다.
- `CombatPreview`가 조건부 위협을 플레이어에게 어떻게 예고할지 결정되어 있다.
- 조건부 이벤트가 없으면 전투 흐름을 테스트하거나 데이터셋을 만들기 어려운 상태다.

후속 확장:

- `UnitHpBelowPercent`
- `UnitDefeated`
- `GroupCleared`
- `TimeSinceGroupSpawned`
- `PointBreached`
- `UnitEnteredArea`
- `RequiredWaveCleared`
- 보스 패턴
- 소환 스킬
- `BattlefieldHazard` 활성화

이 단계에서 `ScenarioAction`을 `SpawnGroup` 외 액션으로 확장한다.

2단계 구현 방향:

```rust
pub enum ScenarioTrigger {
    AtBattleStart,
    AtTimeMs(u64),
    When(ScenarioCondition),
}

pub enum ScenarioCondition {
    UnitHpBelowPercent {
        unit_ref: ScenarioUnitRef,
        percent: u8,
    },
    UnitDefeated {
        unit_ref: ScenarioUnitRef,
    },
    GroupCleared {
        group_id: ScenarioGroupId,
    },
    PointBreached {
        point_id: TacticalPointId,
        leaks_at_least: u32,
    },
}
```

조건 평가기는 `BattleCore` 내부의 승패 판정과 분리한다. 승패 조건은 전투 종료 여부를 결정하고, 조건부 시나리오 이벤트는 새 `BattleEvent`를 큐에 넣는 역할만 한다.

## 유지 대상

시나리오 기반 전투 코어에서 계속 유지할 대상:

- `BattleUnitDraft`: `ScenarioUnitSpawn` 내부의 유닛 정의로 재사용 가능하다.
- `TimelineEvent::UnitSpawned`: 계속 사용한다.

## 테스트 계획

필수 테스트:

- 전투 시작 fixture와 노드 전투가 `BattleScenario`를 직접 사용한다.
- `AtBattleStart` player/enemy group이 스폰된다.
- `AtTimeMs(5000)` enemy group이 늦게 스폰된다.
- 늦게 스폰된 enemy가 `UnitSpawned` 타임라인에 5000ms로 기록된다.
- 늦게 스폰된 enemy가 공격/이동/타겟팅 대상에 포함된다.
- 첫 enemy group이 전멸해도 pending required group이 있으면 승리하지 않는다.
- 마지막 required group 전멸 후 승리한다.
- optional group은 승리 조건을 막지 않는다.
- player 전멸은 pending enemy wave와 무관하게 패배한다.
- 같은 scenario와 seed는 같은 unit instance id를 만든다.

권장 테스트 명:

```text
scenario_spawn_group_at_battle_start_matches_legacy_deck_start
scenario_delayed_enemy_wave_spawns_at_scheduled_time
scenario_pending_required_wave_blocks_early_victory
scenario_optional_wave_does_not_block_victory
scenario_spawned_unit_receives_attack_schedule
scenario_unit_refs_map_to_runtime_instance_ids
```

## 구현 시 주의점

- `ScenarioEvent`는 입력이고 `TimelineEvent`는 출력이다. 둘을 섞지 않는다.
- 스폰은 항상 `BattleEvent` 큐를 통해 처리한다.
- `run_battle_with_setup`의 테스트용 setup hook은 유지하되, 시나리오 스폰 이후/이전 중 어느 시점에 실행되는지 명확히 해야 한다.
- 스폰된 유닛의 `AttackStart` 예약을 빼먹으면 유닛이 가만히 서 있는 버그가 생긴다.
- 유닛 생성 시 movement backend, spatial backend, battlefield occupancy가 모두 같은 상태를 보도록 해야 한다.
- `participant_results`는 전투 중 스폰된 모든 유닛을 포함해야 한다.
- graveyard에 들어간 유닛도 `ScenarioUnitRef` 해석이 가능해야 보스 사망 조건을 판정할 수 있다.
- 같은 좌표 스폰 충돌 정책을 명확히 해야 한다. 1차는 중복 좌표를 static data 오류로 처리하는 것이 안전하다.

## 1차 구현 범위 권장안

첫 구현에서 할 것:

- `BattleScenario` 타입 추가.
- 기존 deck 입력을 scenario로 변환.
- `SpawnGroup` 이벤트 추가.
- `AtBattleStart`, `AtTimeMs`만 지원.
- required group 기반 승패 판정.
- `CombatExecutor`는 아직 완전 전환하지 않아도 되지만, 새 scenario 경로 테스트를 추가한다.

첫 구현에서 하지 말 것:

- 보스 HP 조건 트리거.
- 유닛 사망 조건 트리거.
- 스킬로 소환물 생성.
- 필드 위험요소.
- 데이터 RON 스키마 대규모 변경.

이 순서가 안전한 이유는 기존 전투 테스트를 유지하면서도 전투 코어의 핵심 가정을 먼저 바꿀 수 있기 때문이다.
