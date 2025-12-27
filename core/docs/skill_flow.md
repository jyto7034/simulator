# 스킬 명세(초안) (Tick-Based, Deterministic)

이 문서는 전투 중 유닛의 **스킬 시전(공명 기반)** 을 1ms 고정 tick 기반으로 결정론적으로 정의한다.

참조:
- 이동/예약/결정론 기본 규칙: `core/docs/movement_flow.md`
- 원거리 기본 공격(투사체/즉발) 초안: `core/docs/ranged_attack_flow.md`

---

## 0) 범위(Scope)

포함:
- 공명(resonance) 최대 조건 만족 시 스킬 발동 흐름
- 집중 시간(캐스팅) 동안의 행동 제한
- 지정(타겟) 스킬 / 미지정(범위) 스킬
- 즉발 / 투사체 형태(지연 타격 포함)
- 취소 규칙(타겟 사망/집중 방해/사거리 이탈 등)
- 결정론을 위한 ID/정렬 규칙

미포함(후속 분리 권장):
- 스킬별 구체 효과(디버프/버프/넉백/소환 등)
- 공명 획득 규칙(어떤 행동이 공명을 쌓는지)
- 데미지 공식 및 방어/저항

---

## 1) Time & Determinism

- tick: `1 tick = 1ms`
- 모든 처리 순서/타이브레이커는 `core/docs/movement_flow.md`의 결정을 따른다.
  - 같은 tick 내 유닛 처리: `uuid` 오름차순
  - 동률: `거리 → (y,x) → uuid`

스킬 인스턴스 ID:
- `cast_id`는 결정론적으로 생성돼야 한다.
  - 예: `(run_seed, battle_seq, caster_uuid, cast_seq)` 기반
- 투사체/지연 타격이 있다면 `projectile_id`도 결정론적으로 생성한다.

---

## 2) 용어

- `공명(resonance)`: 유닛별 누적 게이지. 최대치 도달 시 스킬 시전 기회가 생김.
- `집중 시간(focus_time_ms)`: 스킬 “시전 준비”에 필요한 시간. 0일 수 있음.
- `지정 스킬(targeted skill)`: 특정 적 유닛을 타겟으로 시전.
- `미지정 스킬(untargeted skill)`: 타겟 지정 없이 범위/지점을 대상으로 시전.
- `사거리(range_tiles)`: `dist_cheb(caster_tile, 대상)`이 이 값 이내면 “사거리 내”.

---

## 3) RON 파라미터(가정)

스킬은 아래 파라미터를 가진다고 가정한다. (정확한 필드는 RON 스키마에 맞게 매핑)

공통:
- `range_tiles: u8`
- `focus_time_ms: u32` (0 가능)
- `cast_delay_after_resonance_full_ms: u32 = 10`
- `delivery`:
  - `Instant`
  - `Projectile { speed_units_per_ms: u32 }`

지정 스킬 전용:
- `targeting_rule`: 기본은 “사거리 내 가장 가까운 적(chebyshev), 동률 uuid”

미지정 스킬 전용:
- `shape`: 예) 원형/부채꼴/직선 등 (초안에서는 “범위 공격 가능” 수준으로만 규정)
- `placement_rule`: 예) caster 중심 / 타겟 타일 중심 등

---

## 4) 유닛 스킬 상태(상태기계)

유닛은 스킬 관점에서 아래 상태를 가진다.

- `SkillIdle`
  - 공명이 최대가 아니거나, 최대여도 아직 시전 트리거가 발동되지 않은 상태
- `SkillArmed(ready_at_ms)`
  - 공명이 최대가 되었고, `ready_at_ms = now_ms + 10ms`에 시전 시도를 해야 함
- `SkillFocusing(until_ms, cast_id, kind)`
  - 집중 시간 동안 아무 행동도 하지 않는 상태
- `SkillExecuting(cast_id)`
  - 스킬 실행(즉발이면 즉시 효과 적용, 투사체면 발사 및 명중 이벤트 스케줄)

규칙:
- `SkillFocusing` 동안 유닛은 **이동/기본공격/다른 스킬**을 수행하지 않는다.
- `SkillFocusing` 또는 `SkillExecuting` 중 하드 CC가 걸리면 즉시 취소 처리한다. (아래 7장)

---

## 5) 발동 트리거(공명 → 10ms 대기)

공명이 최대(`resonance_current == resonance_max`)가 되는 순간:

1. `SkillArmed(ready_at_ms = now_ms + 10ms)`로 전환한다.
2. `ready_at_ms` 전까지는 기존 행동(이동/공격 등)을 유지할 수 있다.
3. `ready_at_ms`에 도달하면 “시전 가능 여부”를 평가하고, 가능하면 `SkillFocusing` 또는 `SkillExecuting`으로 진입한다.

결정론:
- `ready_at_ms`는 고정값(10ms)이며 jitter를 두지 않는다.

---

## 6) 시전 가능 여부(Armed → Focus/Execute)

`now_ms >= ready_at_ms`가 되면 아래 순서로 처리한다.

### 6.1 지정 스킬(Targeted)

1. 사거리 내 적 후보를 수집한다:
   - `dist_cheb(caster_tile, enemy_tile) <= range_tiles`
2. 후보가 없다면:
   - 시전을 보류하고 `SkillArmed`를 유지한다. (다음 tick에 재평가)
3. 후보가 있다면 타겟을 선택한다:
   - `dist_cheb` 최소, 동률 `enemy_uuid` 오름차순
4. `focus_time_ms > 0`:
   - `SkillFocusing(until_ms = now_ms + focus_time_ms, cast_id, Targeted(target_uuid))`
5. `focus_time_ms == 0`:
   - 즉시 `SkillExecuting(cast_id)`로 진입(타겟은 cast context로 저장)

### 6.2 미지정 스킬(Untargeted)

1. “시전 가능한가”의 최소 조건:
   - 스킬 정의에 따라 다르지만, 초안에서는 **사거리 조건을 만족하는 목표 지점/영역이 존재해야 함**으로 정의한다.
   - 간단히: “적이 사거리 내에 1명 이상 존재”를 시전 조건으로 둔다.
2. 조건을 만족하지 않으면:
   - `SkillArmed` 유지(다음 tick 재평가)
3. 조건을 만족하면:
   - `focus_time_ms > 0`이면 `SkillFocusing` 진입
   - 아니면 `SkillExecuting` 진입

---

## 7) 취소 규칙(Cancellation)

### 7.1 공통 취소(하드 CC)

`SkillFocusing` 중 “집중 방해(인터럽트)” 또는 하드 CC(빙결/기절 등) 발생 시:

- 즉시 시전을 취소한다.
- 공명 처리:
  - **공명은 소모된다.**
- 상태:
  - `SkillIdle`로 전환한다.

### 7.2 지정 스킬: 타겟 사망

`SkillFocusing` 중 타겟이 사망하면:

- 시전은 취소된다.
- 공명 처리:
  - **공명은 유지된다.**
- 상태:
  - `SkillArmed(ready_at_ms = now_ms)`로 즉시 재시도 가능하도록 전환한다.

### 7.3 지정 스킬: 사거리 이탈

`SkillFocusing` 중 타겟이 사거리 밖으로 이동하더라도:

- 시전은 **취소되지 않는다.**
- 집중 종료 시점에 정상 발사/적용한다.

---

## 8) 실행 규칙(Execute)

`SkillExecuting`은 “효과를 실제로 발생”시키는 단계다.

### 8.1 즉발(Instant)

- 실행 tick에서 즉시 효과를 적용한다.
- 지정 스킬:
  - 타겟이 실행 시점에 생존이면 적용, 사망이면 무효(아무 일도 없음)
- 미지정 스킬:
  - 영역 내 대상 판정 및 적용(초안에서는 상세 생략)

### 8.2 투사체(Projectile)

- `projectile_id`를 생성하고, 비행/명중을 스케줄한다.
- 비행 시간:
  - 발사 시점의 스냅샷을 사용한다(결정론):
    - `origin_tile = caster_tile_at_fire`
    - 지정 스킬이면 `target_tile_snapshot = target_tile_at_fire`
  - `distance_units = dist_cheb(origin_tile, target_tile_snapshot) * 1 tile_units`
  - `flight_time_ms = ceil_div(distance_units, projectile_speed_units_per_ms)`
  - `impact_ms = fire_ms + flight_time_ms`

명중 처리(초안 기본):
- `impact_ms`에 타겟이 생존이면 적용, 사망이면 무효
- 투사체는 유닛/지형에 막히지 않는다(충돌/LOS 없음)
- 타겟이 이동해도 “지연 타격”으로 처리(생존이면 맞음)

---

## 9) 공명 소모 타이밍

초안 규칙:
- 집중 성공 → 스킬 실행이 시작되는 순간 공명을 소모한다.
- 예외:
  - 지정 스킬이 “타겟 사망”으로 취소된 경우 공명 유지
  - 집중 방해(인터럽트/하드 CC)로 취소된 경우 공명 소모

---

## 10) Tick Loop 통합(권장 순서)

리플레이 일관성을 위해 tick 처리 순서를 고정한다.

1. 만료/상태 갱신(사망, CC, soft state 만료)
2. `SkillFocusing` 만료 처리(= 실행 진입)
3. `SkillArmed`의 `ready_at_ms` 도달 처리(= focus/execute 진입)
4. 스킬 `Impact`(투사체/지연 타격) 처리
5. 기본 공격/이동 처리

정렬:
- 같은 tick 내에서 (2)~(4) 다수 발생 시 `uuid`/`cast_id`/`projectile_id` 오름차순으로 처리한다.

---

## 11) 남은 결정(TBD)

아래는 구현 전에 추가로 확정하면 좋은 항목들이다.

- 공명이 최대가 되는 순간이 이동 중일 때:
  - (A) `ready_at_ms`까지는 이동 유지, 이후 시전 시도 시 “이동을 즉시 중단”할지
  - (B) “현재 예약한 목적지 도착 후에만” 시전 시도할지
- 미지정 스킬의 “시전 가능 조건”:
  - 적 1명 이상 사거리 내 vs 빈 지점 시전 허용 여부
- 스킬이 기본 공격을 대체하는지(공명 최대 시 무조건 스킬 우선인지), 또는 우선순위 규칙

