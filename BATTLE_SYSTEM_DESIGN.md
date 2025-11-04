# 전투 시스템 설계 문서

> **작성일**: 2025-10-23
> **버전**: 4.0
> **목적**: Lobotomy Corp 환상체 기반 실시간 오토 배틀 시스템 설계 (전투 + 장비 + 아티팩트)

---

## 📋 목차

1. [개요](#개요)
2. [핵심 설계 원칙](#핵심-설계-원칙)
3. [실시간 전투 시스템](#실시간-전투-시스템)
4. [포지션 시스템](#포지션-시스템)
5. [타게팅 규칙](#타게팅-규칙)
6. [승리 조건](#승리-조건)
7. [자동 전투 AI](#자동-전투-ai)
8. [환상체 역할 분류](#환상체-역할-분류)
9. [능력 설계 원칙](#능력-설계-원칙)
10. [장비 시스템](#장비-시스템)
11. [아티팩트 시스템](#아티팩트-시스템)
12. [구현 우선순위](#구현-우선순위)
13. [전투 예시](#전투-예시)
14. [향후 확장](#향후-확장)

---

## 개요

### 게임 흐름

**Day 기반 로그라이크:**

```
Day 1 시작 (6 Hours)
├─ Hour 0: 이벤트/상점 (랜덤 발생)
├─ Hour 1: 이벤트/상점 (랜덤 발생)
├─ Hour 2: PvE 전투 (Monster - 3개 중 1개 선택)
├─ Hour 3: 이벤트/상점 (랜덤 발생)
├─ Hour 4: 이벤트/상점 (랜덤 발생)
└─ Hour 5: PvP 전투 (플레이어 Ghost 매칭)
     ↓
Day 2 시작 (6 Hours)
├─ Hour 0: 이벤트/상점
├─ Hour 1: 이벤트/상점
├─ Hour 2: PvE 전투
├─ Hour 3: 이벤트/상점
├─ Hour 4: 이벤트/상점
└─ Hour 5: PvP 전투
     ↓
반복...
```

**이벤트/상점 종류 (Hour 0, 1, 3, 4):**
- 상점 입장 (장비/아티팩트/환상체 구매)
- 골드 얻기
- 환상체 얻기
- 퀘스트 얻기
- 무료 옵션 (최대 HP, 재생, 경험치 등)
- 그외 등 (장비 강화, 휴식 등)

**PvE 전투 (Hour 2 - 고정):**
- 3개 Monster 중 1개 선택 전투
- Monster 등급: Bronze / Silver / Gold / Diamond / Legendary
- 항상 Bronze 1개, Silver 1개, Gold+ 1개 제공

**PvP 전투 (Hour 5 - 고정):**
- 비동기 PvP (다른 플레이어의 Ghost 빌드와 자동 전투)
- 플레이어 스냅샷 기반 (실시간 매칭 아님)

**레벨업 시스템:**
- 특정 행동 또는 시간 경과로 경험치 획득
- 경험치 일정량 누적 → 레벨업
- 레벨업 시 보상 (스탯 포인트, 스킬 포인트 등)

### 전투 특징
- **실시간 전투** (TFT, Auto Chess 방식)
- **완전 자동 전투** (플레이어는 배치만 결정)
- **정보 비대칭 환경** (상대 구성 사전 확인 불가)
- **비동기 PvP** (배치 후 서버에서 자동 시뮬레이션)
- **결정론적 재현** (같은 초기 조건 → 같은 결과)

### 환상체 사용
- Lobotomy Corporation IP 기반
- 저등급(ZAYIN/TETH)에서 시작 → 고등급(ALEPH) 성장
- 환상체 = 기물 (유닛)
- Day마다 성장 (장비 획득, 스타 강화, 레벨업)

---

## 핵심 설계 원칙

### 1. 단순하고 명확한 규칙
- 누구나 이해 가능한 타게팅
- "앞에서부터 순서대로 공격"

### 2. 정보 비대칭 환경 대응
- 상대 구성을 모르는 상태에서도 공정
- 범용성 높은 능력 중심
- 특화 카운터 도구 최소화

### 3. 기물 섬멸 = 승리
- 넥서스 없음
- 모든 적 기물 제거 시 승리
- Front든 Back이든 어차피 다 죽여야 함
- → 특화 도구(관통, 위치변경) 필수 아님

### 4. 전략 깊이
- 포지셔닝 (Front/Mid/Back 배치)
- 기물 조합 (시너지)
- 성장 경로 (업그레이드, 스타 강화)
- 속도 차별화 (빠른 기물이 여러 번 공격)

### 5. 결정론적 재현
- 모든 전투는 완벽히 재현 가능
- 랜덤 요소 없음 (크리티컬, 회피 등)
- 리플레이 시스템 지원

---

## 실시간 전투 시스템

### 시간 기반 시뮬레이션

**핵심 개념:**
- 전투는 실시간(ms 단위)으로 진행
- 각 기물은 **공격 속도(Attack Speed)** 스탯을 가짐
- 공격 속도에 따라 독립적으로 공격

**공격 속도 예시:**
- 빠른 기물 (Der Freischütz): 500ms마다 공격
- 중간 기물 (Scarecrow): 1000ms마다 공격
- 느린 기물 (One Sin): 2000ms마다 공격

→ **Der Freischütz는 One Sin보다 4배 많이 공격**

### 시뮬레이션 구조

```rust
pub struct BattleSimulation {
    current_time_ms: u64,      // 현재 시간 (밀리초)
    player_field: Field,
    enemy_field: Field,
    battle_log: Vec<BattleEvent>,
}

pub struct Piece {
    id: String,
    hp: u32,
    attack: u32,
    attack_interval_ms: u64,   // 공격 주기 (예: 500ms)
    next_attack_time_ms: u64,  // 다음 공격 시간
}

impl BattleSimulation {
    pub fn step(&mut self, delta_ms: u64) {
        self.current_time_ms += delta_ms;

        // 모든 기물의 공격 시간 체크
        for piece in self.all_alive_pieces_mut() {
            if self.current_time_ms >= piece.next_attack_time_ms {
                let target = select_target(piece, &self.enemy_field);
                self.execute_attack(piece, target);

                // 다음 공격 시간 예약
                piece.next_attack_time_ms = self.current_time_ms + piece.attack_interval_ms;
            }
        }
    }
}
```

### 재현 가능성 (Deterministic)

**모든 요소가 결정론적:**
1. 초기 배치 고정
2. 공격 속도 고정
3. 타게팅 규칙 고정 (Front → Mid → Back)
4. 데미지 고정 (크리티컬 없음)
5. 시간 단위 정수 (부동소수점 오차 없음)

**리플레이 구현:**
```rust
pub fn replay_battle(initial_state: BattleState) -> BattleResult {
    let mut sim = BattleSimulation::new(initial_state);

    // 1ms씩 시뮬레이션 (또는 이벤트 기반 점프)
    while !sim.is_finished() && sim.current_time_ms < MAX_BATTLE_TIME_MS {
        sim.step(1);
    }

    sim.get_result()
}
```

→ **같은 초기 상태 → 항상 같은 전투 로그 → 같은 결과**

### Event Queue 방식 (권장)

**핵심: 시간 점프 (Time Jump)**

1ms씩 증가하는 대신, **다음 이벤트 시간으로 바로 점프**

**초기 상태:**
```
Der Freischütz: attack_interval = 500ms
Scarecrow: attack_interval = 1000ms, skill_interval = 3000ms
Mountain: attack_interval = 2500ms, heal_interval = 3000ms

Event Queue 초기화:
[
  (500ms, Attack { attacker: "Der" }),
  (1000ms, Attack { attacker: "Scarecrow" }),
  (2500ms, Attack { attacker: "Mountain" }),
  (3000ms, Skill { caster: "Scarecrow", type: Debuff }),
  (3000ms, Skill { caster: "Mountain", type: Heal }),
]
```

**처리 흐름:**
```
current_time = 0ms

[1] Queue.pop() → (500ms, Der 공격)
    current_time = 500ms로 점프 ⚡

    execute_attack(Der)
    timeline.push(Attack { time: 500ms, ... })

    Queue.push((500 + 500 = 1000ms, Der 공격))

[2] Queue.pop() → (1000ms, Scarecrow 공격)
    current_time = 1000ms로 점프 ⚡

    execute_attack(Scarecrow)
    timeline.push(...)

    Queue.push((2000ms, Scarecrow 공격))

[3] Queue.pop() → (1000ms, Der 공격)  // 동일 시간
    current_time = 1000ms (유지)

    execute_attack(Der)
    Queue.push((1500ms, Der 공격))

...계속
```

**장점:**
- ✅ 효율적: 필요한 시점만 계산 (501ms, 502ms 같은 빈 시간 스킵)
- ✅ 정확함: 모든 이벤트가 정확한 ms에 발동
- ✅ 타임라인 자동 정렬: Queue에서 나온 순서 = 시간순
- ✅ 결정론적: Priority Queue 순서만 명확하면 완벽 재현

### 이벤트 타입

```rust
pub enum BattleEvent {
    // 공격
    Attack {
        time_ms: u64,
        attacker_id: String,
        target_id: String,
        damage: u32,
    },

    // 스킬 발동
    Skill {
        time_ms: u64,
        caster_id: String,
        skill_type: SkillType,
        targets: Vec<String>,
    },

    // 버프/디버프 틱
    BuffTick {
        time_ms: u64,
        piece_id: String,
        buff_type: BuffType,
        effect: i32,  // 양수: 힐, 음수: 독 데미지
    },

    // 사망
    Death {
        time_ms: u64,
        piece_id: String,
    },

    // 재배치 (특수 스킬 전용 - 예: "적 Back 기물을 Front로 이동")
    // 자동 재배치는 없음! 타게팅 로직으로 자동 처리
    Reposition {
        time_ms: u64,
        piece_id: String,
        from: Lane,
        to: Lane,
    },

    // 소환
    Summon {
        time_ms: u64,
        summoner_id: String,
        summoned_id: String,
        lane: Lane,
    },
}
```

### 버프/디버프/스킬 처리

#### 1. 주기적 스킬 (재생, 디버프)

**예시: Scarecrow 디버프 (3초마다)**
```
초기화:
  Queue.push((3000ms, Skill { caster: "Scarecrow", type: Debuff }))

[3000ms] 처리:
  - 적 전체에 디버프 적용 (공격력 -20%)
  - timeline.push(Skill { time: 3000ms, ... })
  - 다음 스킬 예약: Queue.push((6000ms, Skill {...}))

[6000ms] 처리:
  - 또 디버프 적용
  - Queue.push((9000ms, Skill {...}))

...계속
```

#### 2. 지속 효과 (독, 재생, 버프)

**예시: 독 데미지 (5초간, 초당 10 데미지)**
```
피격 시점 (2000ms):
  piece.active_buffs.push(Buff {
    type: Poison,
    damage_per_tick: 10,
    tick_interval: 1000ms,
    duration: 5000ms,
    started_at: 2000ms,
  })

  // 틱 이벤트 예약
  Queue.push((3000ms, BuffTick { piece_id, damage: 10 }))
  Queue.push((4000ms, BuffTick { piece_id, damage: 10 }))
  Queue.push((5000ms, BuffTick { piece_id, damage: 10 }))
  Queue.push((6000ms, BuffTick { piece_id, damage: 10 }))
  Queue.push((7000ms, BuffTick { piece_id, damage: 10 }))
  Queue.push((7000ms, BuffExpired { piece_id, buff_id }))

[3000ms] BuffTick 처리:
  - piece.hp -= 10
  - timeline.push(BuffTick {...})
  - 사망 체크
```

#### 3. 광역 스킬

**예시: Silent Orchestra 광역 공격**
```
[800ms] Attack { attacker: "Silent Orchestra", type: AreaOfEffect }

처리:
  for target in enemy_field.all_lanes():
    apply_damage(target, 120)
    if target.hp <= 0:
      death_list.push(target)

  // 사망 일괄 처리
  process_all_deaths(death_list)

  // 승리 조건 체크
  check_victory()
```

#### 4. 소환 스킬

**예시: WhiteNight 사도 소환 (10초마다)**
```
초기화:
  Queue.push((10000ms, Summon { caster: "WhiteNight" }))

[10000ms] 처리:
  - 사도 기물 생성 (Piece)
  - 필드에 추가
  - timeline.push(Summon {...})

  // 사도의 첫 공격 예약
  Queue.push((10000 + 사도공격속도, Attack { attacker: "사도" }))

  // 다음 소환 예약
  Queue.push((20000ms, Summon {...}))
```

### 사망 처리

**핵심: 죽은 기물의 미래 이벤트 제거**

```
[3000ms] Mountain 공격 → Der Freischütz에게 300 데미지

execute_attack():
  1. 데미지 적용
     Der.hp = 150 - 300 = -150

  2. HP <= 0 체크 → 사망 처리
     - piece.is_alive = false
     - timeline.push(Death { time: 3000ms, piece_id: "Der" })

     - Queue에서 해당 기물 이벤트 전부 제거
       queue.retain(|event| event.piece_id != "Der")

       제거되는 이벤트:
       (3500ms, Der 공격)  ← 제거
       (4000ms, Der 공격)  ← 제거
       (4500ms, Der 공격)  ← 제거
       ...

  3. 승리 조건 체크
     if enemy_field.all_dead():
       return BattleResult { winner: "player1", time: 3000ms }

  4. Mountain 다음 공격 예약
     Queue.push((3000 + 2500 = 5500ms, Mountain 공격))
```

**타게팅 자동 변경:**
```
Front 사망 전:
Front: [One Sin]
Mid:   [Scarecrow]
Back:  [Der Freischütz]

→ 적의 공격 타겟: Front (One Sin)

Front 사망 후:
Front: []
Mid:   [Scarecrow]  ← 위치 그대로
Back:  [Der Freischütz]  ← 위치 그대로

→ 적의 공격 타겟: 자동으로 Mid (Scarecrow)
→ 물리적 이동 없이 타게팅만 변경
→ 레인별 아티팩트 효과 유지!
```

**레인별 시너지 유지 (중요):**
```
아티팩트: "Mid 레인 공격력 +30%"
Mid: Scarecrow (공격력 80 + 30% = 104)

Front 사망 후:
Scarecrow: Mid 위치 유지
→ Mid 레인 아티팩트 효과 유지
→ 공격력 104 유지하며 적의 공격 받음
```

**동시 사망 처리:**
```
광역 공격으로 3000ms에 여러 기물 사망:

1. 모든 대상에 데미지 적용
2. 사망자 리스트 수집
3. 일괄 사망 처리:
   - Death 이벤트 기록
   - Queue에서 이벤트 제거
4. 승리 조건 체크
```

### 동시 이벤트 우선순위

**같은 시간(예: 3000ms)에 여러 이벤트 발생 시:**

```
Queue at 3000ms:
[
  (3000ms, BuffTick { type: Heal }),      // 회복
  (3000ms, Attack { attacker: "Der" }),   // 공격
  (3000ms, Skill { type: Debuff }),       // 디버프
]
```

**우선순위 규칙 (선택):**

**방식 1: 이벤트 타입 우선순위**
```
1. Buff/Heal (버프/회복 먼저)
2. Attack (공격)
3. Debuff (디버프)
4. Death (사망 처리 마지막)
```

**방식 2: 등록 순서 (FIFO)**
```
Queue에 먼저 들어간 순서대로 처리
```

**방식 3: piece_id 순서**
```
사전순 정렬 (일관성 보장)
"Der" → "Mountain" → "Scarecrow"
```

**중요:** 어떤 규칙이든 **일관되게** 적용하면 결정론적 재현 보장

**권장: 이벤트 타입 우선순위 + 등록 순서**
```rust
impl Ord for BattleEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // 1. 시간 비교
        match self.time_ms.cmp(&other.time_ms) {
            Ordering::Equal => {
                // 2. 이벤트 타입 우선순위
                self.event_priority().cmp(&other.event_priority())
            }
            other => other,
        }
    }
}

impl BattleEvent {
    fn event_priority(&self) -> u8 {
        match self {
            BattleEvent::BuffTick { .. } => 1,  // 버프 먼저
            BattleEvent::Attack { .. } => 2,
            BattleEvent::Skill { .. } => 3,
            BattleEvent::Death { .. } => 4,     // 사망 마지막
            _ => 5,
        }
    }
}
```

### 구현 예시

```rust
use std::collections::BinaryHeap;

pub fn simulate_battle(
    player1_deck: Deck,
    player2_deck: Deck,
) -> BattleResult {
    let mut state = BattleState::new(player1_deck, player2_deck);
    let mut event_queue = BinaryHeap::new();
    let mut timeline = Vec::new();
    let mut current_time = 0u64;

    // 초기 이벤트 삽입
    for piece in state.all_pieces() {
        // 첫 공격 예약
        event_queue.push(BattleEvent::Attack {
            time_ms: piece.attack_interval,
            attacker_id: piece.id.clone(),
            ..
        });

        // 주기적 스킬 예약
        if let Some(skill) = piece.periodic_skill {
            event_queue.push(BattleEvent::Skill {
                time_ms: skill.first_cast_time,
                caster_id: piece.id.clone(),
                ..
            });
        }
    }

    // 이벤트 루프
    while let Some(event) = event_queue.pop() {
        // 시간 점프
        current_time = event.time_ms;

        // 최대 시간 체크
        if current_time > MAX_BATTLE_TIME_MS {
            return BattleResult::draw(timeline);
        }

        // 이벤트 처리
        match event {
            BattleEvent::Attack { attacker_id, .. } => {
                if let Some(result) = process_attack(
                    &mut state,
                    &attacker_id,
                    current_time,
                    &mut event_queue,
                ) {
                    timeline.push(result);
                }
            }
            BattleEvent::Skill { caster_id, skill_type, .. } => {
                process_skill(&mut state, &caster_id, skill_type, current_time);
            }
            // ... 다른 이벤트 처리
        }

        // 승리 조건 체크
        if let Some(winner) = check_victory(&state) {
            return BattleResult {
                winner_id: winner,
                time_ms: current_time,
                timeline,
            };
        }
    }

    BattleResult::draw(timeline)
}
```

---

## 포지션 시스템

### 3개 레인 구조

```
[아군 진형]
┌───────┬───────┬───────┐
│ Front │  Mid  │ Back  │
└───────┴───────┴───────┘

[적 진형]
┌───────┬───────┬───────┐
│ Front │  Mid  │ Back  │
└───────┴───────┴───────┘
```

### 레인별 특징

**Front (전방):**
- 역할: 탱커, 방어벽
- 특징: 높은 HP, 느린 공격 속도, 낮은 공격력
- 환상체: One Sin (2000ms), Mountain of Smiling Bodies (2500ms)

**Mid (중앙):**
- 역할: 서포터, 유틸리티
- 특징: 중간 HP, 중간 속도, 버프/디버프
- 환상체: Scarecrow (1000ms), Funeral Butterfly (1200ms)

**Back (후방):**
- 역할: 딜러, 주 공격수
- 특징: 낮은 HP, 빠른 공격 속도, 높은 공격력
- 환상체: Der Freischütz (500ms), Silent Orchestra (800ms)

---

## 타게팅 규칙

### 기본 규칙: Front → Mid → Back 순서

**모든 기물이 동일한 규칙 적용:**
1. 적 Front에 기물 있음 → Front 공격
2. 적 Front 없음 → Mid 공격
3. 적 Front/Mid 없음 → Back 공격

```rust
fn select_target(attacker: &Piece, enemy_field: &Field) -> Option<TargetId> {
    if !enemy_field.front.is_empty() {
        Some(enemy_field.front[0].id)  // Front 첫 번째 기물
    } else if !enemy_field.mid.is_empty() {
        Some(enemy_field.mid[0].id)    // Mid 첫 번째 기물
    } else if !enemy_field.back.is_empty() {
        Some(enemy_field.back[0].id)   // Back 첫 번째 기물
    } else {
        None  // 적 전멸
    }
}
```

### 장점

1. **극도로 단순**
   - 플레이어가 즉시 이해 가능
   - 전투 결과 예측 가능

2. **Front의 중요성**
   - Front = 방어벽 역할 확실
   - Lobotomy Corp "격리실 방어선" 느낌

3. **드라마틱한 전투 흐름**
   - Front 붕괴 → Mid 노출 → Back 위험
   - 방어선 돌파 느낌

4. **자연스러운 밸런스**
   - Front 약하면 금방 뚫림 → 딜러 위험
   - Front 강하면 오래 버팀 → 딜러 안전
   - Back 딜러만으로는 부족 (Front 필수)

5. **속도 차별화**
   - 빠른 딜러가 느린 탱커보다 2-4배 많이 공격
   - 하지만 Front가 막아주지 않으면 즉시 노출

---

## 승리 조건

### 기물 섬멸 (Last Stand)

**승리:**
- 적 기물 전멸 (Front, Mid, Back 모두 0)

**패배:**
- 아군 기물 전멸

**무승부:**
- 최대 시간(예: 60초) 도달
- 양쪽 남은 기물 HP 합산 비교

```rust
pub enum BattleResult {
    Victory { winner: PlayerId, time_ms: u64 },
    Draw { time_ms: u64 },
}

pub fn check_victory_condition(state: &BattleState) -> Option<BattleResult> {
    let player_alive = state.player_field.count_alive() > 0;
    let enemy_alive = state.enemy_field.count_alive() > 0;

    if !enemy_alive {
        return Some(BattleResult::Victory {
            winner: PlayerId::Player,
            time_ms: state.current_time_ms
        });
    }

    if !player_alive {
        return Some(BattleResult::Victory {
            winner: PlayerId::Enemy,
            time_ms: state.current_time_ms
        });
    }

    if state.current_time_ms >= MAX_BATTLE_TIME_MS {
        return Some(BattleResult::Draw {
            time_ms: state.current_time_ms
        });
    }

    None  // 전투 계속
}
```

### 기물 섬멸의 장점

1. **정보 비대칭 문제 해결**
   - Front든 Back이든 어차피 다 죽여야 함
   - 특화 도구(관통) 없어도 게임 성립

2. **범용 능력 중심**
   - 높은 데미지 → 빠른 섬멸
   - 높은 HP → 오래 생존
   - 회복 → 지속력
   - 빠른 속도 → 많은 공격
   - 모두 상대 구성 무관하게 유용

3. **단순하고 직관적**
   - "적 다 죽이기"

---

## 자동 전투 AI

### 완전 자동 전투

**플레이어 제어:**
- 배치만 결정 (어떤 환상체를 어느 레인에)
- 전투는 완전 자동

**AI 행동:**
- 타게팅: Front → Mid → Back 규칙 (자동)
- 공격 타이밍: 공격 속도 기반 (자동)
- 스킬 사용: 조건 만족 시 자동 발동
- 이동: 없음 (배치 후 고정)

### PvP 전투 흐름

```
[매칭 성사]
플레이어 A: 환상체 배치 완료
플레이어 B: 환상체 배치 완료

[서버에서 실시간 전투 시뮬레이션]
0ms: 전투 시작
500ms: Der Freischütz 첫 공격
800ms: 보스 첫 공격
1000ms: Der Freischütz 두 번째 공격
...
15000ms: 전투 종료 (적 전멸)

[결과 전송]
플레이어 A: 승리 (15초 소요)
플레이어 B: 패배
+ 전투 로그 (리플레이용)
```

---

## 환상체 역할 분류

### ALEPH (최고 위험등급)

**탱커:**
- **Mountain of Smiling Bodies (T-01-75)**
  - HP: 1000, ATK: 100, Speed: 2500ms
  - 능력: 3초마다 HP 50 재생

**딜러:**
- **WhiteNight (T-03-46)**
  - HP: 300, ATK: 250, Speed: 600ms
  - 능력: 사도 소환 (10초마다)

- **Apocalypse Bird (O-02-40)**
  - HP: 400, ATK: 200, Speed: 800ms
  - 능력: 광역 공격 (모든 레인 동시)

- **Nothing There (O-06-20)**
  - HP: 350, ATK: 180, Speed: 700ms
  - 능력: 변신 (죽은 적 모방)

### WAW (높은 위험등급)

**탱커:**
- **Big Bird (O-02-56)**
  - HP: 700, ATK: 80, Speed: 2000ms
  - 능력: 보호막 (5초마다)

**딜러:**
- **Der Freischütz (F-01-69)**
  - HP: 150, ATK: 150, Speed: 500ms
  - 능력: 높은 단일 데미지

- **Silent Orchestra (T-01-31)**
  - HP: 200, ATK: 120, Speed: 800ms
  - 능력: 광역 데미지

**서포터:**
- **Scarecrow Searching for Wisdom (F-01-87)**
  - HP: 200, ATK: 80, Speed: 1000ms
  - 능력: 적 공격력 -20%

- **Army in Black (D-01-106)**
  - HP: 250, ATK: 90, Speed: 1200ms
  - 능력: 병사 소환 (8초마다)

### HE (중간 위험등급)

**서포터:**
- **Funeral of the Dead Butterflies (T-01-68)**
  - HP: 180, ATK: 60, Speed: 1200ms
  - 능력: 아군 회복 (3초마다 50 HP)

- **Skin Prophecy (T-09-80)**
  - HP: 150, ATK: 70, Speed: 1100ms
  - 능력: 디버프 (적 방어력 -30%)

### TETH/ZAYIN (낮은 위험등급)

**탱커:**
- **One Sin and Hundreds of Good Deeds (O-03-03)**
  - HP: 500, ATK: 50, Speed: 2000ms
  - 능력: 기본 탱커

- **Beauty and the Beast (F-02-44)**
  - HP: 400, ATK: 60, Speed: 1800ms
  - 능력: 변신 (HP 50% 이하 시)

**딜러:**
- **Red Shoes (O-04-08)**
  - HP: 120, ATK: 100, Speed: 800ms
  - 능력: 기본 딜러

- **1.76 MHz (T-02-36)**
  - HP: 100, ATK: 90, Speed: 600ms
  - 능력: 속도형 딜러

---

## 능력 설계 원칙

### 범용 능력 중심 (Phase 1)

**공격형:**
- **높은 데미지**: 단순히 빠르게 적 제거
- **빠른 공격 속도**: 더 자주 공격
- **광역 공격**: 모든 레인 동시 타격
- **지속 데미지**: 독, 출혈 (초당 데미지)

**방어형:**
- **높은 HP**: 오래 생존
- **재생**: 초당 HP 회복
- **보호막**: 일회성 데미지 흡수
- **느린 공격 속도**: 탱커 특성

**서포터형:**
- **아군 버프**: 공격력/공격속도 증가
- **적 디버프**: 공격력/공격속도 감소
- **회복**: 아군 HP 회복 (주기적)

**유틸리티:**
- **소환**: 추가 기물 생성 (쿨다운 있음)
- **부활**: 1회 사망 시 부활

### 특화 능력 (Phase 2 - 선택적)

**관통 공격:**
- 효과: 적 Back에게 50% 추가 데미지
- 용도: Back 딜러 메타 카운터
- 필수 아님 (어차피 Front 죽이고 Back 공격 가능)

**위치 변경:**
- 효과: 적 Back 1명을 Front로 이동 (1회)
- 용도: 우선순위 조정
- 필수 아님 (기물 수 우위 효과 정도)

**고정 데미지:**
- 효과: 방어력 무시 데미지
- 용도: 탱커 메타 카운터
- 필수 아님 (시간 들여서 죽이면 됨)

**둔화:**
- 효과: 적 공격 속도 +50% (느려짐)
- 용도: 속도 메타 카운터

→ **이런 특화 능력은 메타가 고착되면 조정 도구로만 추가**

---

## 장비 시스템

### 혼합 방식 개요

**2개 레이어 강화 시스템:**
1. **장비 (Equipment)** - 개별 장착
2. **아티팩트 (Artifact)** - 덱 전체 효과

### 슬롯 구조

**환상체당 3개 슬롯:**

```
환상체 (Der Freischütz)
├─ 무기 슬롯      [마탄의 사수 총]
├─ 방어구 슬롯    [강철 갑옷]
└─ 악세서리 슬롯  [시간 가속 장치]
```

### 장비 등급

| 등급 | 스탯 보너스 | 특수 효과 | 드랍률 |
|------|------------|----------|--------|
| Common (회색) | +10% | 없음 | 50% |
| Rare (파랑) | +20% | 없음 | 30% |
| Epic (보라) | +35% | 약한 효과 | 15% |
| Legendary (주황) | +50% | 강력한 효과 | 5% |

### 무기 (Weapon)

**공격력 강화 중심**

**Common/Rare:**
- 단검: 공격력 +20 / +40
- 권총: 공격력 +30 / +60
- 소총: 공격력 +40 / +80

**Legendary 예시:**
```
마탄의 사수 (Der Freischütz E.G.O)
- 공격력 +225 (50% 보너스)
- 특수: 첫 공격 치명타 (2배 데미지)
- 특수: 적 처치 시 공격력 +10 (최대 5스택)

심판새의 저울 (Apocalypse Bird E.G.O)
- 공격력 +225
- 특수: 공격마다 적 전체에게 데미지 20% 분산
- 특수: 전투 시간 10초마다 공격력 +50
```

### 방어구 (Armor)

**HP 강화 중심**

**Common/Rare:**
- 가죽 갑옷: HP +100 / +200
- 사슬 갑옷: HP +150 / +300
- 판금 갑옷: HP +200 / +400

**Legendary 예시:**
```
미소 짓는 시체들의 껍질 (Mountain of Smiling Bodies E.G.O)
- HP +1500 (50% 보너스)
- 특수: 3초마다 HP +100 회복
- 특수: HP 30% 이하 시 방어력 +50%

큰 새의 깃털 (Big Bird E.G.O)
- HP +1200
- 특수: 5초마다 보호막 200 (3초간)
- 특수: Front 레인 배치 시 모든 아군 방어력 +10%
```

### 악세서리 (Accessory)

**공격 속도 & 유틸리티 중심**

**Common/Rare:**
- 시계: 공격 속도 -50ms / -100ms
- 나침반: 공격 속도 -75ms / -150ms
- 수정: 공격 속도 -100ms / -200ms

**Legendary 예시:**
```
시간을 먹는 시계 (1.76 MHz E.G.O)
- 공격 속도 -500ms (50% 보너스)
- 특수: 공격마다 5% 확률로 추가 공격
- 특수: 적 처치 시 3초간 공격 속도 -200ms 추가

나비의 시간 (Funeral of Dead Butterflies E.G.O)
- 공격 속도 -400ms
- 특수: 공격 시 30% 확률로 아군 1명 HP 50 회복
- 특수: 3초마다 랜덤 아군 HP 100 회복
```

### 장비 효과 적용

```rust
pub struct EquippedPiece {
    pub base_piece: Piece,
    pub weapon: Option<Equipment>,
    pub armor: Option<Equipment>,
    pub accessory: Option<Equipment>,
}

impl EquippedPiece {
    pub fn calculate_final_stats(&self) -> PieceStats {
        let mut stats = self.base_piece.stats.clone();

        // 장비 스탯 적용
        if let Some(weapon) = &self.weapon {
            stats.attack += weapon.stats.attack_bonus;
        }
        if let Some(armor) = &self.armor {
            stats.hp += armor.stats.hp_bonus;
        }
        if let Some(accessory) = &self.accessory {
            stats.attack_interval_ms += accessory.stats.speed_bonus_ms;
        }

        stats
    }
}
```

---

## 아티팩트 시스템

### 덱 전체 효과

**소지 제한:**
- 덱당 5-7개 장착 가능
- 조건 중복 가능 (중첩 효과)

### 아티팩트 카테고리

#### 1. 타입 기반 (환상체 등급)

```
"ALEPH의 위엄" (Epic)
- 효과: ALEPH 등급 환상체 모든 스탯 +15%

"WAW 특화 훈련" (Rare)
- 효과: WAW 등급 환상체 공격력 +30%

"혼합 부대" (Epic)
- 효과: 서로 다른 등급 환상체 3개 이상 시 모든 스탯 +10%
```

#### 2. 위치 기반 (Front/Mid/Back)

```
"전방 방어선" (Rare)
- 효과: Front 레인 HP +200

"후방 지원" (Rare)
- 효과: Back 레인 공격력 +40

"중앙 조율" (Rare)
- 효과: Mid 레인 스킬 쿨다운 -20%

"삼위일체" (Epic)
- 효과: Front/Mid/Back 각각 1개 이상 배치 시 모든 환상체 공격력 +25
```

#### 3. 전투 기반

```
"선공의 이점" (Rare)
- 효과: 전투 시작 후 첫 5초간 모든 환상체 공격력 +50%

"역전의 여신" (Epic)
- 효과: 아군 환상체 HP 50% 이하일 때 공격 속도 -30%

"장기전 대비" (Rare)
- 효과: 전투 10초 경과 시 모든 환상체 스탯 +20%

"복수의 칼날" (Epic)
- 효과: 아군 환상체 사망 시 남은 환상체 공격력 +40% (누적)
```

#### 4. 능력 기반 (범용)

```
"재생의 힘" (Rare)
- 효과: 3초마다 모든 환상체 HP +20 회복

"공격 집중" (Common)
- 효과: 모든 환상체 공격력 +15

"속도 증폭" (Common)
- 효과: 모든 환상체 공격 속도 -5%

"흡혈" (Epic)
- 효과: 공격 시 데미지의 10% HP 회복
```

#### 5. 시너지 기반

```
"딜러 서포트" (Rare)
- 효과: Back 레인에 환상체 있고 Mid 레인에 환상체 있을 때 Back 공격력 +35%

"탱커 라인" (Rare)
- 효과: Front 레인 환상체 2개 이상 시 모두 HP +300

"속도 시너지" (Epic)
- 효과: 공격 속도 500ms 이하 환상체 2개 이상 시 모두 공격 속도 -100ms
```

### Legendary 아티팩트

```
"격리실 붕괴" (Legendary)
- 효과: ALEPH 환상체 3개 이상 시:
  - 모든 환상체 공격력 +50%
  - 모든 환상체 공격 속도 -20%
  - 전투 시작 시 적 전체에 200 데미지

"완벽한 방어" (Legendary)
- 효과: Front 레인 HP 2000 이상 시:
  - Front 레인 피격 데미지 50% 감소
  - 3초마다 Front HP 200 회복
  - Back/Mid 레인 공격력 +60%

"시간의 지배자" (Legendary)
- 효과: 공격 속도 500ms 이하 환상체 3개 이상 시:
  - 모든 환상체 공격 속도 -30%
  - 전투 시작 시 3초간 적 시간 정지
  - 공격마다 5% 확률로 2배 공격
```

### 아티팩트 효과 적용

```rust
pub struct Artifact {
    pub condition: ArtifactCondition,
    pub effect: ArtifactEffect,
}

pub enum ArtifactCondition {
    HasAbnormalityTier { tier: AbnormalityTier, min_count: u8 },
    HasPieceInLane { lane: Lane, min_count: u8 },
    BattleTimeElapsed { min_time_ms: u64 },
    AllyHpBelow { percent: u8 },
    Always,
}

pub enum ArtifactEffect {
    StatBonus {
        target: EffectTarget,
        attack_percent: i8,
        hp_percent: i8,
        speed_percent: i8,
    },
    PeriodicHeal {
        target: EffectTarget,
        amount: u32,
        interval_ms: u64,
    },
    DamageReduction {
        target: EffectTarget,
        percent: u8,
    },
}
```

### 장비 + 아티팩트 시너지 예시

**속도 딜러 덱:**
```
Der Freischütz
- 무기: 마탄의 사수 (+225 공격력)
- 악세서리: 시간 왜곡 장치 (-350ms)

아티팩트:
- "속도 증폭" (-5%)
- "속도 시너지" (-100ms)
- "선공의 이점" (첫 5초 +50% 공격력)

결과: 초고속 연속 공격 + 첫 공격 치명타
```

**탱커 중심 덱:**
```
Mountain of Smiling Bodies
- 방어구: 미소 짓는 시체들의 껍질 (+1500 HP, 재생)

아티팩트:
- "전방 방어선" (+200 HP)
- "탱커 라인" (+300 HP)
- "재생의 힘" (+20 HP/3초)
- "완벽한 방어" (피격 -50%, Back/Mid 공격력 +60%)

결과: HP 3000, 초당 120 HP 회복, 무너지지 않는 방어선
```

### 획득 방법

**Day 구조 (6 Hours):**

#### Hour 0, 1, 3, 4 - 이벤트/상점 선택

**랜덤 발생 이벤트/상점:**
```
- 상점 입장
  · 골드로 장비 구매
  · 골드로 아티팩트 구매
  · 골드로 환상체 구매

- 골드 얻기
  · 소량 골드 (100-200)
  · 중량 골드 (300-500)
  · 대량 골드 (600-1000)

- 환상체 얻기
  · Common 환상체 (70%)
  · Rare 환상체 (25%)
  · Epic 환상체 (5%)

- 퀘스트 얻기
  · 환상체 수집 → Epic 장비 보상
  · PvP 승리 → Rare 아티팩트 보상

- 무료 옵션
  · 최대 HP 증가
  · HP 재생 버프
  · 경험치 획득

- 그외 등
  · 장비 강화
  · 휴식 (HP 회복, 버프)
```

#### Hour 2 - PvE 전투 (고정)

**Monster 선택 전투:**
```
3개 Monster 중 1개 선택:
- Bronze 등급 Monster (1개, 쉬움)
- Silver 등급 Monster (1개, 보통)
- Gold/Diamond/Legendary 등급 Monster (1개, 어려움)

승리 보상:
- 골드 (100-300)
- 경험치
- Common/Rare 장비 드랍 (50% 확률)
- Monster 등급에 따라 보상 증가
```

#### Hour 5 - PvP 전투 (고정)

**플레이어 Ghost 매칭:**
```
- 진행 상황 비슷한 플레이어 Ghost와 매칭
- Ghost = 다른 플레이어의 스냅샷된 빌드
- 배치 후 자동 전투 (비동기)

승리 보상:
- 골드 대량 (500-1000)
- 경험치 대량
- Rare 아티팩트 (50% 확률)
- Epic 아티팩트 (10% 확률)

패배 보상:
- 골드 소량 (100-200)
- 경험치 소량
- Common 아티팩트 (30% 확률)
```

**Legendary 획득 (특수):**
- Day 10, 20, 30... 기념 보상
- 특수 퀘스트 완료
- PvP 연승 (5연승, 10연승)
- 보스 PvE 처치 (Day 10마다 보스 등장)

---

## 구현 우선순위

### Phase 1: 기본 실시간 전투 시스템 (2-3주)

**목표:** 프로토타입 완성

**구현 항목:**
1. 시간 기반 시뮬레이션 (ms 단위)
2. Front/Mid/Back 포지션 구조
3. Front → Mid → Back 타게팅
4. 공격 속도(Attack Speed) 시스템
5. 기본 능력 (데미지, HP, 속도)
6. 기물 섬멸 승리 조건
7. 전투 로그 (리플레이용)
8. Event Queue 기반 시뮬레이션

**환상체 (최소):**
- Front: One Sin (TETH) - 2000ms
- Mid: Funeral Butterfly (HE) - 1200ms
- Back: Der Freischütz (WAW) - 500ms

### Phase 2: 능력 확장 (1-2주)

**목표:** 다양성 추가

**구현 항목:**
1. 광역 공격 (Silent Orchestra)
2. 소환 (Army in Black) - 쿨다운 시스템
3. 재생 (Mountain of Smiling Bodies) - 주기적 회복
4. 디버프 (Scarecrow) - 공격력/속도 감소
5. 버프 능력
6. 스킬 쿨다운 시스템

**환상체 추가:**
- 5-10개 추가

### Phase 3: 고급 메커니즘 (1주)

**목표:** 전략 깊이 추가

**구현 항목:**
1. 시너지 시스템 (같은 타입 환상체 조합)
2. 스타 강화 (3개 → 1개 업그레이드)
3. 부활/불사 메커니즘
4. 지속 데미지 (독, 출혈)

### Phase 4: 밸런스 및 특화 능력 (계속)

**목표:** 메타 조정

**구현 항목:**
1. 데이터 수집 (승률, 사용률, 평균 전투 시간)
2. 메타 분석 (탱커 메타? 딜러 메타? 속도 메타?)
3. 필요시 특화 능력 추가 (관통, 위치변경, 둔화 등)

---

## 전투 예시

### 초기 배치

```
[플레이어 A]
Front: One Sin (HP 500, ATK 50, Speed 2000ms)
Mid:   Scarecrow (HP 200, ATK 80, Speed 1000ms, 스킬: 적 공격력 -20%)
Back:  Der Freischütz (HP 150, ATK 150, Speed 500ms)

[플레이어 B - AI 적]
Front: 병사 A (HP 100, ATK 60, Speed 1500ms)
Mid:   병사 B (HP 100, ATK 60, Speed 1500ms)
Back:  보스 (HP 300, ATK 100, Speed 800ms)
```

### 실시간 전투 로그

```
[0ms] 전투 시작
  - 모든 기물 next_attack_time 초기화

[500ms] Der Freischütz 공격
  - 타겟: 적 Front (병사 A)
  - 데미지: 150
  - 결과: 병사 A 사망 ✝
  - Next: 1000ms
  - 적 Front 사망 → 다음 타겟은 Mid (병사 B)

[800ms] 보스 공격
  - 타겟: 플레이어 Front (One Sin)
  - 데미지: 100
  - 결과: One Sin HP 400/500
  - Next: 1600ms

[1000ms] Der Freischütz 공격
  - 타겟: 적 Mid (병사 B)  ← Front 없으므로 Mid 타겟
  - 데미지: 150
  - 결과: 병사 B 사망 ✝
  - Next: 1500ms
  - 적 Front/Mid 사망 → 다음 타겟은 Back (보스)

[1000ms] Scarecrow 공격
  - 타겟: 적 Back (보스)  ← Front/Mid 없으므로 Back 타겟
  - 데미지: 80
  - 스킬: 보스 공격력 -20% (100 → 80)
  - 결과: 보스 HP 220/300
  - Next: 2000ms

[1500ms] Der Freischütz 공격
  - 타겟: 적 Back (보스)
  - 데미지: 150
  - 결과: 보스 HP 70/300
  - Next: 2000ms

[1600ms] 보스 공격
  - 타겟: 플레이어 Front (One Sin)
  - 데미지: 80 (디버프 적용)
  - 결과: One Sin HP 320/500
  - Next: 2400ms

[2000ms] Der Freischütz 공격
  - 타겟: 적 Back (보스)
  - 데미지: 150
  - 결과: 보스 사망 ✝
  - Next: 2500ms

[2000ms] One Sin 공격
  - 타겟: 없음 (적 전멸)

[2000ms] 전투 종료
```

**전투 결과:**
- **승자**: 플레이어 A
- **시간**: 2000ms (2초)
- **남은 기물**: 3개 (One Sin 320 HP, Scarecrow 200 HP, Der Freischütz 150 HP)
- **적 기물**: 0개

**타게팅 자동 변경:**
- 병사 A 사망 후 → 자동으로 Mid (병사 B) 타겟
- 병사 B 사망 후 → 자동으로 Back (보스) 타겟
- 물리적 이동 없음, 레인 유지

**속도 차별화 효과:**
- Der Freischütz (500ms): 4번 공격
- Scarecrow (1000ms): 2번 공격
- One Sin (2000ms): 1번 공격
- 보스 (800ms): 2번 공격

---

## 향후 확장

### Day 진행 시스템 (우선순위: 높음)

**1. 이벤트 시스템**
- 랜덤 이벤트 생성
- 이벤트 타입:
  - 상점 (장비/아티팩트/환상체 구매)
  - 골드 얻기
  - 환상체 얻기
  - 퀘스트 얻기 (조건 달성 시 보상)
  - 장비 강화
  - 휴식 (HP 회복, 버프)
  - 그외 등 (향후 확장)

**2. 레벨업 시스템**
```rust
pub struct Player {
    pub level: u32,
    pub exp: u32,
    pub exp_to_next_level: u32,
}

pub enum ExpSource {
    TimeElapsed { minutes: u32 },     // 시간 경과
    EventCompleted { event_id: String },  // 이벤트 완료
    PvEVictory,                       // PvE 승리
    PvPVictory,                       // PvP 승리
    QuestCompleted,                   // 퀘스트 완료
}

pub struct LevelUpReward {
    pub stat_points: u32,      // 스탯 포인트 (HP, 공격력 등에 투자)
    pub skill_points: u32,     // 스킬 포인트 (특수 능력 해금)
    pub gold: u32,             // 골드 보상
}
```

**3. PvE 전투 시스템**
- NPC/AI 상대 전투
- Day 난이도 증가 (Day 1: 쉬움, Day 10: 어려움)
- 보스 PvE (Day 10, 20, 30...)
- 보상: 골드, 경험치, 장비

**4. 퀘스트 시스템**
```rust
pub enum QuestType {
    CollectAbnormality { tier: AbnormalityTier, count: u8 },
    WinPvP { count: u8 },
    WinPvE { count: u8 },
    ReachDay { day: u32 },
    EquipLegendary { count: u8 },
}

pub struct Quest {
    pub quest_type: QuestType,
    pub reward: QuestReward,
}
```

### 전투 고급 메커니즘

**1. 스킬 쿨다운 시스템**
- 현재: 기본 공격만
- 확장: 강력한 스킬 (5초마다 1번 등)
- 예: "마탄의 사수" (5초 쿨다운, 300 데미지)

**2. 지형 효과**
- 특정 Day에서 "Front 방어력 +20%" 같은 버프
- 전투 시간 +10초 연장 등

**3. 환상체 합성**
- Apocalypse Bird (심판새 + 징벌새 + 큰 새)

**4. E.G.O 장비 (이미 구현됨)**
- Lobotomy Corp 무기/방어구 시스템
- 환상체에서 추출한 장비 착용
- 공격 속도 +20%, 데미지 +50 등

**5. 지속 효과 (DoT/HoT)**
- 독: 초당 10 데미지 (5초간)
- 출혈: 초당 20 데미지 (3초간)
- 재생: 초당 30 HP 회복 (영구)

**6. 버프/디버프 스택**
- "분노" 스택: 공격마다 공격력 +5% (최대 10스택)
- "약화" 스택: 피격마다 방어력 -3% (최대 5스택)

### 메타 대응 도구

**탱커 메타가 지배하면:**
- 고정 데미지 환상체 추가
- %HP 기반 데미지 추가
- 초당 %HP 데미지 (True Damage)

**딜러 메타가 지배하면:**
- 관통 공격 환상체 추가
- 암살 능력 추가 (Back 우선 타겟)
- 반사 데미지 탱커

**속도 메타가 지배하면:**
- 둔화 능력 추가 (공격 속도 +50%)
- 빙결 능력 추가 (3초간 공격 불가)
- 공격 속도 상한선 설정 (최소 200ms)

→ **메타 분석 후 점진적 추가**

---

## 기술 스택

### simulator_core (게임 로직 라이브러리)

**순수 함수 구현:**
```rust
// 실시간 전투 시뮬레이션
pub mod battle {
    pub fn step(state: &mut BattleState, delta_ms: u64);
    pub fn select_target(attacker: &Piece, enemy_field: &Field) -> Option<TargetId>;
    pub fn execute_attack(attacker: &Piece, target: &mut Piece) -> AttackResult;
    pub fn check_victory_condition(state: &BattleState) -> Option<BattleResult>;
}

pub struct BattleState {
    pub current_time_ms: u64,      // 현재 시간
    pub player_field: Field,
    pub enemy_field: Field,
    pub battle_log: Vec<BattleEvent>,
}

pub struct Field {
    pub front: Vec<Piece>,
    pub mid: Vec<Piece>,
    pub back: Vec<Piece>,
}

pub struct Piece {
    pub id: String,
    pub name: String,
    pub hp: u32,
    pub max_hp: u32,
    pub attack: u32,
    pub attack_interval_ms: u64,   // 공격 주기 (예: 500ms)
    pub next_attack_time_ms: u64,  // 다음 공격 시간
    pub abilities: Vec<Ability>,
}

pub enum BattleEvent {
    Attack {
        time_ms: u64,
        attacker_id: String,
        target_id: String,
        damage: u32,
    },
    Death {
        time_ms: u64,
        piece_id: String,
    },
    Reposition {
        time_ms: u64,
        piece_id: String,
        from: Lane,
        to: Lane,
    },
    SkillActivated {
        time_ms: u64,
        piece_id: String,
        skill_name: String,
    },
}
```

### game_server (인프라)

**BattleActor 호출:**
```rust
// game_server/src/game/battle_actor/mod.rs
pub async fn execute_battle(
    player1_field: Field,
    player2_field: Field,
) -> BattleResult {
    use simulator_core::battle;

    let mut state = BattleState::new(player1_field, player2_field);

    // 실시간 시뮬레이션 (1ms 단위)
    while state.current_time_ms < MAX_BATTLE_TIME_MS {
        battle::step(&mut state, 1);  // 1ms 진행

        if let Some(result) = battle::check_victory_condition(&state) {
            return BattleResult {
                winner: result.winner,
                time_ms: state.current_time_ms,
                battle_log: state.battle_log,
            };
        }
    }

    // 시간 초과 무승부
    BattleResult::draw(state.current_time_ms, state.battle_log)
}
```

**이벤트 기반 최적화 (선택적):**
```rust
// 1ms씩 증가 대신, 다음 이벤트로 점프
pub fn step_optimized(state: &mut BattleState) {
    let next_event_time = find_next_event_time(state);
    let delta = next_event_time - state.current_time_ms;

    state.current_time_ms = next_event_time;

    // 해당 시간에 발생하는 모든 공격 처리
    process_attacks_at_current_time(state);
}
```

---

## 리플레이 시스템

### 전투 로그 저장

```rust
pub struct BattleReplay {
    pub initial_state: BattleState,
    pub events: Vec<BattleEvent>,
    pub final_state: BattleState,
}

impl BattleReplay {
    pub fn replay(&self) -> BattleResult {
        let mut state = self.initial_state.clone();

        // 이벤트 순서대로 재현
        for event in &self.events {
            apply_event(&mut state, event);
        }

        // 결과는 항상 동일
        assert_eq!(state, self.final_state);

        BattleResult::from_state(&state)
    }
}
```

### 클라이언트 재생

```
Unity Client:
1. 서버에서 BattleReplay 다운로드
2. 이벤트를 시간순으로 재생
3. 애니메이션, 이펙트 표시

[500ms] Der Freischütz 공격 애니메이션
[800ms] 보스 공격 애니메이션
[1000ms] 병사 B 사망 이펙트
...
```

---

## 참고 문서

- **ARCHITECTURE_STATUS.md** - 전체 아키텍처 현황
- **EQUIPMENT_ARTIFACT_DESIGN.md** - 장비/아티팩트 시스템 상세 (통합됨)
- **BATTLE_ACTOR_REFACTORING_PLAN.md** - Battle Actor 리팩토링 계획 (완료)
- **TRYMATCH_REFACTORING_PLAN.md** - TryMatch 리팩토링 계획 (완료)

---

## 변경 이력

| 날짜 | 버전 | 변경 사항 |
|------|------|--------------|
| 2025-10-23 | 1.0 | 초안 작성 (포지션 기반 전투 시스템) |
| 2025-10-23 | 2.0 | 실시간 전투 방식으로 수정 (ms 단위 시뮬레이션) |
| 2025-10-23 | 3.0 | 장비 시스템 + 아티팩트 시스템 추가 (혼합 방식) |
| 2025-10-23 | 3.1 | Day 진행 구조 수정 (이벤트 3개 → PvE → 이벤트 → PvP), 레벨업 시스템 추가 |
| 2025-10-23 | 3.2 | Event Queue 방식 상세 설명 추가 (시간 점프, 버프/스킬 처리, 사망 처리, 우선순위 규칙) |
| 2025-10-23 | 3.3 | 자동 재배치 제거 (타게팅 로직으로 자동 처리, 레인별 아티팩트 효과 유지) |
| 2025-10-23 | 3.4 | 이벤트 시스템 표현 수정 ("3개 중 1개 선택" → "랜덤 발생") |
| 2025-10-26 | 4.0 | Day 구조 The Bazaar 방식으로 수정 (5단계 → 6 Hours, Hour 2 PvE 고정, Hour 5 PvP 고정) |

---

**작성자**: Development Team
**최종 수정**: 2025-10-26
