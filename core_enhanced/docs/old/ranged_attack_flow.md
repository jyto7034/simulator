# 원거리 공격 명세 (Tick-Based, Deterministic)

이 문서는 전투 중 **원거리 기본 공격(평타)** 및 **투사체 기반 타격**을 1ms 고정 tick 기반으로 결정론적으로 정의한다.

참조:
- 이동/예약/결정론 기본 규칙: `core/docs/movement_flow.md`

---

## 0) 범위(Scope)

포함:
- 원거리 유닛의 “기본 공격” 흐름(타겟 선정 → 준비(윈드업) → 발사 → 타격)
- 투사체/즉발(Delayed-hit 포함) 처리 규칙
- 동시성/결정론 정렬 규칙

미포함(후속 문서로 분리 권장):
- 스킬(지정/미지정), 공명/집중(캐스팅) 상세
- 방어막/회피/치명타/저항 등 데미지 공식
- 관통/탄환 충돌/지형 LOS 등 특수 투사체

---

## 1) Time & Units

- tick: `1 tick = 1ms`
- 공통 거리 단위: `tile_units`
  - `1 tile = 1_000_000 tile_units`
- 공통 속도 단위: `tile_units_per_ms`
  - 유닛 이동, 투사체 이동 모두 동일 단위를 사용한다.

시간 계산(부동소수점 금지):
- `time_ms = ceil(distance_units / speed_units_per_ms)`
- `ceil_div(a, b) = (a + b - 1) / b`

---

## 2) Distance & Range

- 공격 사거리(기본 공격 사거리)는 `dist_cheb(attacker_tile, target_tile) <= range_tiles`로 판정한다.
- 원거리 유닛의 기본 공격은 `range_tiles >= 2`를 전제한다.
- line-of-sight(LOS) 차단은 없다.
  - 진영 상관 없이 유닛 너머로 공격 가능

---

## 3) Determinism (필수)

원거리 공격 처리도 `core/docs/movement_flow.md`의 결정론 규칙을 그대로 따른다.

- 같은 tick에서 “행동 처리 순서”는 `uuid` 오름차순
- 타이브레이커는 문서의 `거리 → (y,x) → uuid` 규칙을 적용

추가로 원거리 공격에서 필요한 ID 규칙:
- 투사체(또는 지연 타격 이벤트)는 결정론적 `projectile_id`를 갖는다.
  - 예: `(run_seed, battle_seq, shooter_uuid, shot_seq)` 기반으로 생성
  - 동일 입력이면 리플레이에서 동일한 `projectile_id`가 생성돼야 한다.

---

## 4) Data Model (RON에서 정의되는 값)

원거리 기본 공격은 아래 파라미터를 가진다고 가정한다. (정확한 필드는 RON 스키마에 맞게 매핑)

- `attack_range_tiles: u8` (>=2)
- `attack_interval_ms: u32` (다음 공격 가능까지의 주기)
- `attack_windup_ms: u32` (공격 준비 시간; 0 가능)
- `attack_damage` (데미지 값/스케일; 본 문서에서는 “타격 시 데미지 적용”만 규정)
- `attack_mode`:
  - `Instant`: 즉발 타격(윈드업 후 바로 데미지 적용)
  - `Projectile { speed_units_per_ms: u32 }`: 투사체가 날아가며, 도착 시 타격

권장:
- `attack_interval_ms`에는 최소값을 둔다(예: 1ms 이상) — 무한 루프/동시성 폭발 방지.

---

## 5) Unit Attack States (기본 공격 관점)

유닛의 기본 공격은 아래 상태로 표현한다.

- `AttackReady`: 공격 가능(쿨다운 완료)
- `Windup(until_ms)`: 준비(공격 모션/윈드업)
- `Fired(cooldown_until_ms)`: 발사 완료, 다음 공격까지 대기

주의:
- 투사체가 날아가는 동안에도 다음 공격은 가능할 수 있다.
  - 즉, “발사”와 “명중”은 분리된 이벤트다.

---

## 6) Target Selection (원거리 기본 공격)

### 6.1 후보 집합

공격자 `A`의 현재 타일을 기준으로:
- `dist_cheb(A, enemy) <= attack_range_tiles` 인 적 유닛만 후보

### 6.2 선택 규칙

후보가 1개 이상이면 다음 1명을 타겟으로 선택한다.

1. `dist_cheb(A, enemy)` 최소
2. 동률이면 `enemy_uuid` 오름차순

### 6.3 타겟 재평가 시점(중요)

원거리 기본 공격은 아래 “2단계”로 타겟을 평가한다.

1. `AttackReady`에서 공격 시작을 결정할 때(윈드업 진입 여부 판단)
2. `Windup` 종료 시점(실제 발사 직전)

규칙:
- `Windup` 도중 타겟이 사망하면, 발사 직전(2)에서 다시 후보를 평가한다.
- 발사 직전(2)에도 후보가 없으면:
  - 발사를 취소하고 이동/탐색(`Acquire`)로 돌아간다.

의도:
- 타겟 사망으로 인한 “헛스윙 시간 낭비”를 최소화하면서도 결정론 유지.

---

## 7) Fire & Hit Resolution

### 7.1 발사 이벤트(Fire)

발사 시점 `t_fire_ms`는:
- `Windup` 종료 tick과 동일하다.

발사 시점에 아래를 스냅샷으로 남긴다.
- `shooter_uuid`
- `target_uuid` (발사 직전에 선정된 타겟)
- `shooter_tile` (발사 순간 위치)
- `target_tile` (발사 순간 타겟 위치)
- `shot_seq` (해당 유닛의 발사 카운터, 결정론적)

### 7.2 즉발(Instant)

`attack_mode == Instant`이면:
- `t_fire_ms`에 즉시 타격을 수행한다.
- 타격 시점에 `target`이 생존이면 데미지를 적용한다.
- 타겟이 사망/소멸 상태면 아무 일도 하지 않는다(즉발 취소).

### 7.3 투사체(Projectile)

`attack_mode == Projectile`이면:
- `distance_units = dist_cheb(shooter_tile, target_tile) * 1 tile_units`
- `flight_time_ms = ceil_div(distance_units, projectile_speed_units_per_ms)`
- `t_impact_ms = t_fire_ms + flight_time_ms`

투사체는 아래 속성을 가진다.
- `projectile_id` (결정론적으로 생성)
- `t_fire_ms`, `t_impact_ms`
- `shooter_uuid`, `target_uuid`
- `target_snapshot_tile` (발사 순간 타겟 타일)

### 7.4 명중 처리(Impact)

명중 시점 `t_impact_ms`에:
- `target_uuid`가 생존이면 데미지를 적용한다.
- 타겟이 사망/소멸이면 투사체는 소멸하며 아무 효과도 없다.

기본 규칙(초안):
- 투사체는 유닛/지형에 의해 막히지 않는다(충돌 없음).
- 투사체는 “타겟드”이며, 명중 시 타겟의 현재 위치를 고려하지 않는다.
  - 즉, “발사 후 타겟이 이동해도(생존이면) 맞는다”는 의미의 **지연 타격** 모델이다.

확장 지점(추후):
- `homing: bool` 또는 `hit_requires_same_tile: bool` 같은 옵션으로 “회피/빗나감” 모델 확장 가능
- `pierce`, `aoe_radius_tiles`, `on_hit_effects` 등

---

## 8) Tick Integration (권장 처리 순서)

본 문서의 공격 처리는 tick 루프에서 아래 우선순위를 권장한다.

1. 만료 처리(사망/CC/상태 만료)
2. `AttackReady` 유닛의 공격 시작 판정(후보 존재 시 `Windup`)
3. `Windup` 만료 유닛의 발사 처리(`Fire` 생성)
4. 투사체 `Impact` 만료 처리(데미지 적용)
5. 이동/예약 처리(`core/docs/movement_flow.md`의 순서와 통합 필요)

결정론을 위해:
- 같은 tick 내에서 (2)~(4) 처리 대상이 여러 개면 `uuid`/`projectile_id` 오름차순으로 처리한다.

---

## 9) Edge Cases (명세로 고정)

- 동일 tick에 여러 투사체가 같은 타겟에 명중할 수 있다.
  - 처리 순서는 `t_impact_ms` → `projectile_id` 오름차순.
- 발사 직전 타겟이 범위 밖으로 나간 경우:
  - 발사 직전(타겟 재평가) 규칙을 적용하여, 범위 내 타겟이 없으면 취소.
- 발사 후 타겟이 사망하면:
  - 명중 시 아무 효과 없음.

---

## 10) Deterministic Test Checklist

- 동일 배치/seed로 리플레이 시:
  - 같은 tick에 동일 타겟을 선택하는가(거리/uuid tie-break)
  - 투사체의 `t_impact_ms`가 항상 동일한가(고정소수점/ceil_div)
  - 동일 tick 명중이 여러 개일 때 처리 순서가 동일한가(projectile_id 정렬)

