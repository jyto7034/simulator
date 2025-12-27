# Movement & Targeting Flow (Tick-Based, Deterministic)

이 문서는 전투 중 유닛의 **타겟 선정**, **이동**, **타일 점유/예약**, **동시성 처리**를 1ms 고정 tick 기반으로 결정론적으로 정의한다.

## Goals

- 리플레이 시 매번 동일한 결과(결정론)
- `예약(Reserved)`과 `점유(Occupied)`는 이동/경로탐색에서 **장애물**
- 이동 중 **아군끼리만** 일시적인 겹침/통과 허용, **아군-적군 겹침 불가**
- 유닛 스왑(서로 자리 바꿈)은 허용하지 않음

## 1. Time & Units

- 시간 단위: `ms`
- tick: `1 tick = 1ms`
- 거리 단위(공통): `tile_units` (고정소수점)
  - `1 tile = 1_000_000 tile_units`
  - 모든 속도는 `tile_units_per_ms`로 표현 (유닛 이동 속도, 투사체 속도 등)
  - 소요 시간 계산은 부동소수점 없이: `time_ms = ceil(distance_units / speed_units_per_ms)`

## 2. Field Model

- 필드는 직사각형 격자 `width x height` 이다.
  - 최소 크기: `min(width, height) >= 3` 그리고 `max(width, height) >= 4` (즉 최소 3x4)
  - 라운드별 크기 변경 없음(매치 중 고정)
- 좌표계:
  - `x ∈ [0, width-1]`, `y ∈ [0, height-1]`
  - 한 좌표를 “타일(tile)”이라 부른다.

## 3. Distances

### 3.1 Chebyshev Distance (사거리/시간 기본)

- `dist_cheb(a, b) = max(|ax-bx|, |ay-by|)`
- 공격 사거리 및 “사거리 내” 판정은 `dist_cheb`를 사용한다.
- 이동/투사체의 시간 계산에서 기본 거리로 `dist_cheb * 1 tile`을 사용한다.
  - 대각 이동도 1 tile로 취급한다.

### 3.2 BFS Path Distance (추적/이동 목표 선택)

- “가장 가까운 적(추적 대상)”은 **BFS 경로 길이**로 정의한다.
- BFS는 8방향 이웃을 사용하며, **코너컷을 허용**한다.
  - 상하좌우가 막혀 있어도 대각 이동 가능

## 4. Determinism Rules (필수)

### 4.1 Global Order

- 같은 tick에서 유닛 처리 순서는 `uuid` 오름차순이다.
  - `uuid` 비교는 `Uuid::as_bytes()`의 lexicographic 오름차순 기준으로 정의한다.

### 4.2 Tile Order

- 타일 정렬은 `(y, x)` 오름차순(위→아래, 좌→우)이다.

### 4.3 BFS Neighbor Order

BFS에서 이웃 확장 순서는 아래 고정 순서를 사용한다.

1. `NW (-1,-1)`
2. `N  ( 0,-1)`
3. `NE ( 1,-1)`
4. `W  (-1, 0)`
5. `E  ( 1, 0)`
6. `SW (-1, 1)`
7. `S  ( 0, 1)`
8. `SE ( 1, 1)`

### 4.4 Tie-Breakers (동률 깨기)

명시되지 않은 모든 동률은 아래 우선순위를 적용한다.

1. 최단 거리(해당 규칙이 사용하는 거리: `dist_cheb` 또는 BFS 길이)
2. 타일 정렬 `(y, x)` 오름차순
3. `uuid` 오름차순

## 5. Tile States (점유/예약)

타일은 아래 상태 중 하나로 해석한다.

- `Empty`: 비어있음
- `Occupied(unit, side)`: 유닛이 **고정** 상태로 점유
- `Reserved(unit)`: 특정 유닛의 **목적지**로 예약됨 (장애물)
- `SoftOccupied(unit, side, until_ms)`: “이동 중 적을 포착해 멈춘 직후”의 임시 점유
  - `until_ms` 이전:
    - 같은 진영 유닛만 “통과(겹침)” 가능
    - 예약/정착 목적지는 불가
  - `until_ms` 이후: `Occupied`로 승격

### 5.1 Occupancy Rules

- **한 타일에는 오직 하나의 고정 유닛만** 존재할 수 있다.
- 적진 타일로 이동은 가능하지만, **적 유닛이 점유 중인 타일을 점유할 수는 없다.**
- `Reserved`는 점유와 동일하게 취급되는 장애물이다.
- “이동 중 겹침”은 **아군끼리만** 허용하며, **아군-적군 겹침은 불가**이다.
- 유닛 스왑(서로의 위치를 교환)은 허용하지 않는다.

## 6. Unit States (이동 관점)

- `Acquire`: 공격 가능 여부 확인 및 이동 계획 수립
- `Moving`: 목적지 예약 + 경로를 따라 이동 중
- `WaitRepath(until_ms)`: 목적지/경로를 찾지 못해 재탐색 대기
- `CCLocked`: 빙결/기절 등 하드 CC로 모든 행동 정지 (즉시 점유로 전환)
- `Dead`

## 7. Targeting

### 7.1 Attackable Set

유닛 `U`의 공격 사거리 `range`는 `dist_cheb(U.pos, enemy.pos) <= range`로 판정한다.

- 근거리(사거리 1): 인접 8칸에 적이 있을 때만 공격 가능
- 원거리(사거리 2 이상): 사거리 내 적을 공격 가능
  - 지형/유닛에 의한 line-of-sight 차단 없음(유닛 너머로 공격 가능)

### 7.2 Choose Attack Target

공격 가능한 적이 있다면 다음 규칙으로 1명을 선택한다.

1. `dist_cheb` 최소
2. 동률이면 `enemy_uuid` 오름차순

## 8. Movement Planning (Acquire 단계)

공격 가능한 적이 없다면, 추적 대상과 목적지를 아래처럼 선택한다.

### 8.1 Choose Chase Target (가장 가까운 적)

- BFS 경로 길이가 최소인 적을 추적 대상으로 선택한다.
- BFS에서 장애물:
  - `Occupied` / `Reserved` / 적군의 현재 타일(이동 중 포함)은 장애물
  - 아군의 현재 타일:
    - `Occupied`는 장애물
    - 이동 중인 아군의 “현재 타일”은 **통과 가능**(겹침 허용)
- 동률이면 `enemy_uuid` 오름차순

### 8.2 Choose Destination Tile (사거리 걸치는 지점)

추적 대상의 위치 `E`에 대해 목적지 후보 `T`는:

- `dist_cheb(T, E) <= range` 를 만족
- `T`가 `Empty` 여야 함
- `T`가 `Occupied / Reserved / SoftOccupied` 이면 제외
- `U.pos`에서 `T`까지 BFS로 도달 가능해야 함

후보 중 다음으로 1개를 선택한다.

1. `U.pos -> T` BFS 길이 최소
2. 동률이면 `(y, x)` 오름차순

### 8.3 Reserve Destination

- 선택된 목적지 타일은 `Reserved(U)`로 예약한다.
- 예약 충돌 시(이미 `Reserved/Occupied/SoftOccupied`):
  - 같은 BFS 결과로 생성 가능한 다음 후보를 순서대로 시도한다.
  - 후보가 모두 실패하면 `WaitRepath`로 전환한다.

## 9. Movement Execution (Moving 단계)

### 9.1 Progress Model

유닛은 매 tick `move_speed_units_per_ms`만큼 이동 게이지를 누적한다.

- `move_progress_units += move_speed_units_per_ms`
- `move_progress_units >= 1 tile` 이면, 경로의 다음 타일로 1스텝 이동을 시도한다.
  - 한 tick에 최대 1스텝만 수행한다. (초과분은 carry)

### 9.2 Step Validity (겹침/충돌)

다음 타일 `N`으로의 스텝은 아래 조건을 모두 만족해야 한다.

- `N`이 필드 내부
- `N`이 `Reserved`가 아님
- `N`이 `Occupied`가 아님
- `N`이 적 유닛(이동 중 포함)의 현재 타일이 아님
- `N`이 `SoftOccupied`인 경우:
  - 같은 진영이면 통과 가능
  - 상대 진영이면 불가

### 9.3 No Swap Rule (스왑 금지)

같은 tick에서 `A.next == B.cur` 이고 `B.next == A.cur` 인 **상호 교환(swap)** 의도는 금지한다.

- 충돌 해결은 `uuid` 오름차순 우선권을 가진다.
  - 낮은 `uuid`는 예정 스텝을 수행할 수 있다.
  - 높은 `uuid`는 해당 tick의 이동 스텝을 취소하고 다음 tick에 재시도한다.

### 9.4 Arrive at Destination

유닛이 예약한 목적지에 도착하면:

- `Reserved(U)`를 해제하고 목적지 타일을 `Occupied(U)`로 전환한다.

## 10. Repath / Waiting

목적지 후보를 찾지 못했거나(도달 불가 포함), 모든 후보 예약이 실패하면:

- `WaitRepath(now_ms + 100ms + jitter_ms)`로 전환한다.
- `jitter_ms`는 리플레이 일관성을 위해 결정적으로 생성한다.
  - 예: `jitter_ms = hash(run_seed, unit_uuid, repath_counter) % 17`

`WaitRepath` 만료 시 `Acquire`로 돌아가 재탐색한다. (횟수 제한 없음)

## 11. Reservation Cancellation

`Moving` 중이라도 아래 조건이 발생하면 즉시 예약을 취소한다.

### 11.1 Cancellation Causes

- 사망(`Dead`)
- 하드 CC로 행동 불가(`CCLocked`)
- 이동 중 공격 가능한 적을 새로 포착(사거리 내 적 존재)

### 11.2 After Cancellation

- 사망: `Reserved` 해제 후 유닛 관련 자원 정리
- CC: 즉시 정지하며 현재 타일을 `Occupied`로 취급(고정)
- 적 포착: 즉시 정지하며 현재 타일을 `SoftOccupied(until = now_ms + 1000ms)`로 전환

## 12. Tick Loop Integration (권장 순서)

동일한 결과를 위해 tick 처리 순서를 고정한다.

1. `now_ms += 1`
2. 만료 처리:
   - `SoftOccupied` 만료 → `Occupied`
   - 사망/CC 상태 갱신 → 예약/점유 정리
3. 의도 계산(Intent):
   - `uuid` 오름차순으로 각 유닛을 처리
   - `Acquire` 또는 `WaitRepath` 만료 상태의 유닛만 BFS/목적지 계산을 수행
   - 예약은 이 단계에서만 생성/갱신
4. 이동 적용(1스텝):
   - `uuid` 오름차순으로 이동 스텝 적용 (스왑 금지 포함)
5. 공격/스킬(본 문서 범위 밖)

## 13. Deterministic Test Scenarios (체크리스트)

- 근접 유닛이 적을 포위한 상태에서 추적/대기 반복이 결정론적으로 유지되는가
- 동일 거리의 적 2명 존재 시 uuid 정렬로 동일 타겟을 선택하는가
- 목적지 후보가 여러 개인 경우 `(y,x)` 기준으로 동일 후보를 고르는가
- 예약 충돌(여러 유닛이 같은 목적지)을 `uuid` 우선권으로 해결하는가
- 아군 이동 중 겹침은 허용되고, 아군-적군 겹침은 항상 거부되는가
- 스왑 의도(A↔B)를 금지하고 우선권으로 한쪽만 이동하는가
- `Reserved`가 BFS 경로에서 장애물로 동작하는가
- 코너컷 허용(상하좌우 막힘에도 대각 이동)이 실제로 가능하고 결정론적인가

