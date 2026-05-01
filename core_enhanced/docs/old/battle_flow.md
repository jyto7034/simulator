# Battle 시스템 설계/흐름 (game_core)

참조 코드:
- 전투 코어/루프: `core/src/game/battle/mod.rs:202`, `core/src/game/battle/mod.rs:357`
- 이벤트 정렬(결정성): `core/src/game/battle/enums.rs:10`, `core/src/game/battle/enums.rs:60`
- 데미지/커맨드 모델: `core/src/game/battle/damage.rs:45`
- 어빌리티 실행기: `core/src/game/battle/ability_executor.rs:41`, `core/src/game/battle/ability_executor.rs:156`
- 타겟 탐색(결정성 tie-break): `core/src/ecs/resources/mod.rs:169`
- 공격 주기 최소값 보장: `core/src/game/stats.rs:152`

---

## 1) 핵심 아이디어(아키텍처)

이 전투는 **이벤트 큐 기반 시뮬레이션**이다.

- 시간축(ms)을 가지는 `BattleEvent`를 `BinaryHeap`에 넣고, 가장 이른 이벤트부터 처리한다. (`core/src/game/battle/enums.rs:10`, `core/src/game/battle/enums.rs:60`)
- 이벤트가 발생하면 “직접 상태를 바꾸기”보다, 먼저 **계산 → 커맨드 생성 → 커맨드 실행** 순서로 진행한다.
  - 계산 단계: `calculate_damage()`가 `DamageResult`와 `triggered_commands`를 만든다. (`core/src/game/battle/damage.rs:90`)
  - 실행 단계: `process_commands()`가 커맨드를 처리하며, 필요 시 추가 이벤트를 스케줄한다. (`core/src/game/battle/mod.rs:534`)

장점:
- 스킬/버프/사망 연쇄 같은 복잡한 흐름을 `BattleCommand`로 일관되게 표현 가능
- 결정성(리플레이/서버 동기화)을 강화하기 쉬움(정렬/타이브레이크 지점이 명확)

---

## 2) 데이터 모델(런타임/정적)

### 덱(전투 시작 전 입력)
- `PlayerDeckInfo`: 유닛/아티팩트/배치정보
  - `units: Vec<OwnedUnit>`: base 메타데이터 UUID + 성장/장비
  - `artifacts: Vec<OwnedArtifact>`: 전투에 적용할 아티팩트 base UUID 목록
  - `positions: HashMap<Uuid, Position>`: **base_uuid 기준** 배치 (`core/src/game/battle/mod.rs:35`)

### 런타임(전투 중)
- `RuntimeUnit`: 전투용 인스턴스(= `instance_id`)를 갖는 유닛 (`core/src/game/battle/mod.rs:163`)
- `RuntimeArtifact` / `RuntimeItem`: 트리거 수집용 런타임 인스턴스
- `graveyard`: 제거된 유닛의 마지막 `UnitSnapshot` 저장소 (사망 후 어빌리티 등 후처리용) (`core/src/game/battle/mod.rs:212`, `core/src/game/battle/mod.rs:625`)

중요 구분:
- `base_uuid`: 데이터베이스 조회용(정적)
- `instance_id`: 전투 중 “대상 식별”용(동적). 이벤트/커맨드의 ID는 전부 `instance_id` 기준.

---

## 3) 전투 시작(초기화 단계)

진입점: `BattleCore::run_battle()` (`core/src/game/battle/mod.rs:357`)

1. 런타임 상태 초기화
   - `units/artifacts/items/graveyard` 초기화
   - `DeathHandler` 상태 초기화
   - `AbilityExecutor` 쿨다운 초기화

2. 덱 → 런타임 유닛/아티팩트/장비 구성
   - `build_runtime_units_from_decks(side)`에서 **덱의 artifacts만 해당 side에 등록** (월드 `Inventory`를 읽지 않음)
   - `OwnedUnit::effective_stats()`가 base 스탯 + 성장 + 장비 + 아티팩트의 Permanent 트리거를 합쳐 `UnitStats`를 만든다. (`core/src/game/battle/mod.rs:96`)

3. `Field` 구성
   - 유닛 `instance_id`를 격자에 배치 (`core/src/game/battle/mod.rs:329`)

4. 초기 이벤트 스케줄
   - 각 유닛에 대해 첫 공격 이벤트를 `attack_interval_ms` 시점에 등록 (`core/src/game/battle/mod.rs:413`)

---

## 4) 메인 루프(이벤트 처리)

루프: `while let Some(event) = event_queue.pop()` (`core/src/game/battle/mod.rs:357`)
- 이벤트의 `time_ms`가 `MAX_BATTLE_TIME_MS`를 넘으면 무승부로 종료
- 매 이벤트 처리 후 생존 유닛 수로 승패 판정

### Attack 이벤트 처리 흐름
핸들러: `process_event()` (`core/src/game/battle/mod.rs:737`)

1. 공격자 조회(없으면 무시)
2. 타겟 선정
   - 현재 타겟이 없으면 `Field::find_nearest_enemy()`로 가까운 적을 찾음
   - 동거리면 UUID tie-break로 결정성 유지 (`core/src/ecs/resources/mod.rs:169`)
3. `apply_attack(attacker_id, time_ms)` 실행 (`core/src/game/battle/mod.rs:473`)
4. 다음 Attack 이벤트 스케줄(최소 1ms 보장) (`core/src/game/battle/mod.rs:766`)

---

## 5) 공격/데미지/트리거 처리

### 트리거 수집
- `collect_all_triggers(unit_instance_id, trigger)`가
  - 같은 side의 아티팩트 트리거 + 해당 유닛의 아이템 트리거를 합친다. (`core/src/game/battle/mod.rs:461`)
- 수집 순서를 `instance_id`로 정렬해서 결정성 유지

### 데미지 계산
- `calculate_damage(request, ctx)` (`core/src/game/battle/damage.rs:90`)
  - 기본: `(attack - defense).max(1)`
  - OnAttack/OnHit 효과를 훑으면서
    - BonusDamage는 즉시 damage를 증감
    - Ability는 `ExecuteAbility` 커맨드 생성
    - Heal은 `ApplyHeal` 커맨드 생성(`source_id` 포함)
  - 최종 데미지로 대상 HP 감소 → 사망이면 `UnitDied` 커맨드 생성

### 커맨드 실행
- `process_commands(commands, now)` (`core/src/game/battle/mod.rs:534`)
  - `ExecuteAbility`: `AbilityExecutor` 실행 결과 커맨드를 다시 처리 (`core/src/game/battle/mod.rs:614`)
    - caster가 이미 제거된 경우 `graveyard` 스냅샷으로도 실행 가능 (`core/src/game/battle/mod.rs:625`)
  - `ApplyHeal`: 음수면 데미지로 처리, 사망하면 `killer_id`를 `source_id`로 기록 (OnKill 트리거 연동)
  - `UnitDied`: `DeathHandler` 큐에 등록
  - `ScheduleAttack`: 추가 공격 이벤트 스케줄

---

## 6) 사망 처리(연쇄 트리거)

- 사망은 즉시 제거하지 않고 `DeathHandler`에 적재 → `process_pending_deaths()`에서 일괄 처리 (`core/src/game/battle/mod.rs:670`)
- `DeathHandler::process_all_deaths()`는 다음 트리거를 지원 (`core/src/game/battle/death.rs:63`)
  - OnDeath (죽은 유닛)
  - OnKill (킬러)
  - OnAllyDeath (같은 편 유닛들)
- 처리 결과:
  - 제거 대상 유닛 리스트 + 추가 커맨드 리스트
- 제거 전에 `graveyard`에 마지막 스냅샷 저장(사망 후 어빌리티 실행 등) (`core/src/game/battle/mod.rs:717`)
- 타겟이 제거되면 다른 유닛의 `current_target`을 정리

---

## 7) 어빌리티 시스템(데이터 드리븐)

- `AbilityExecutor`는 `AbilityId -> SkillDef` 테이블 기반으로 커맨드를 만든다. (`core/src/game/battle/ability_executor.rs:41`)
- 쿨다운은 `(unit_id, ability_id) -> next_ready_time`로 관리 (`core/src/game/battle/ability_executor.rs:47`)
- 타겟 해석은 `TargetScope` 기반이며, EnemySingle에서 tie-break를 넣어 결정성 유지 (`core/src/game/battle/ability_executor.rs:255`)

---

## 8) 결정성(Determinism) 보장 포인트

현재 전투는 아래 지점에서 “정렬/타이브레이크”를 통해 흔들림을 줄인다.
- 이벤트 큐 정렬: time → priority → id tie-break (`core/src/game/battle/enums.rs:60`)
- 타겟 탐색: 최단거리 동률이면 UUID 비교 (`core/src/ecs/resources/mod.rs:169`)
- 트리거 수집 순서: 아티팩트/아이템 `instance_id`로 정렬
- 유닛 스냅샷 목록 정렬(어빌리티 타겟팅에 영향): `unit_snapshots.sort_by(...)` (`core/src/game/battle/mod.rs:614` 근처)

---

## 9) 현재 한계/확장 TODO

- 버프 이벤트(`ApplyBuff/BuffTick/BuffExpire`)는 이벤트 타입만 있고 로직은 TODO 상태 (`core/src/game/battle/enums.rs:10`, `core/src/game/battle/mod.rs:737`)
- `Timeline`은 구조만 있고 기록이 없음 (`core/src/game/battle/mod.rs:49`)
- 덱/배치/instance 설계는 “동일 `base_uuid` 유닛 복수 등장”을 충분히 표현하지 못할 수 있음(`positions`가 `base_uuid` 키 기반).
- 사망 후 어빌리티는 `graveyard`로 실행되지만, Self 대상 커맨드는 실제 `units`에 없으면 무시될 수 있음(의도 설계가 필요).

