# Game Core 테스트 전략

## 목차
1. [초기 제안: Mock Player](#초기-제안-mock-player)
2. [개선 제안](#개선-제안)
3. [Rust 생태계의 테스트 방법론](#rust-생태계의-테스트-방법론)
4. [최종 결정: AIPlayerActor](#최종-결정-aiplayeractor)
5. [구현 가이드](#구현-가이드)

---

## 초기 제안: Mock Player

### 개념
테스트 목적을 가진 Mock Player를 생성하여 다양한 시나리오 테스트

```rust
pub enum BehaviorType {
    /// 정상적인 플레이
    Normal,

    /// 자신이 가지고 있지 않은 아이템 판매 시도
    WrongSellItem,

    /// 현재 페이즈에 적합하지 않은 행동 수행
    WrongPhaseBehavior,
}
```

### 장점
- ✅ 명확한 목적
- ✅ 에지 케이스 커버
- ✅ 예상 결과 검증 가능

### 단점
- ⚠️ 시나리오를 수동으로 정의해야 함
- ⚠️ 예상하지 못한 케이스는 놓칠 수 있음

---

## 개선 제안

### 1. TestScenario + Expectation

```rust
pub enum TestScenario {
    NormalPlay,
    CheatAttempt {
        at_state: GameState,
        forbidden_action: PlayerBehavior,
    },
    InsufficientResources {
        item_price: u32,
        player_gold: u32,
    },
    InvalidItemSale {
        item_uuid: Uuid,
    },
}

pub struct TestExpectation {
    pub should_succeed: bool,
    pub expected_error: Option<GameError>,
    pub expected_state: Option<GameState>,
    pub expected_enkephalin_change: Option<i32>,
}
```

### 2. Scenario Builder

```rust
let scenario = ScenarioBuilder::new()
    .then_start_game()
    .then_request_phase()
    .then_select_shop()
    .then_cheat_with(PlayerBehavior::RequestPhaseData)
    .build();
```

### 3. Snapshot 기반 검증

```rust
pub struct GameSnapshot {
    pub state: GameState,
    pub enkephalin: u32,
    pub phase_events_count: usize,
    pub allowed_actions: Vec<PlayerBehavior>,
}
```

---

## Rust 생태계의 테스트 방법론

### 1. Property-Based Testing

**라이브러리**: `proptest`, `quickcheck`

```rust
proptest! {
    #[test]
    fn test_enkephalin_never_negative(
        initial: u32,
        spend_amount: u32
    ) {
        let mut game = GameCore::new(test_data(), 0);

        if spend_amount > initial {
            assert!(game.spend_enkephalin(spend_amount).is_err());
        } else {
            assert!(game.spend_enkephalin(spend_amount).is_ok());
        }
    }
}
```

**특징**:
- 수백/수천 가지 랜덤 입력 자동 생성
- 버그 발견 시 최소 재현 케이스 자동 생성
- 예상 못한 엣지 케이스 발견

### 2. Snapshot Testing

**라이브러리**: `insta`

```rust
#[test]
fn test_phase_event_generation() {
    let game = GameCore::new(test_data(), 12345);
    let phase_event = game.request_phase_data();

    assert_debug_snapshot!(phase_event);
}
```

**특징**:
- 출력을 파일로 저장하고 비교
- 복잡한 데이터 구조 검증 쉬움
- `cargo insta review`로 변경사항 시각적 확인

### 3. Parameterized Testing

**라이브러리**: `rstest`

```rust
#[rstest]
#[case(GameState::NotStarted, PlayerBehavior::StartNewGame, true)]
#[case(GameState::NotStarted, PlayerBehavior::RequestPhaseData, false)]
#[case(GameState::InShop { .. }, PlayerBehavior::PurchaseItem { .. }, true)]
fn test_allowed_actions(
    #[case] state: GameState,
    #[case] action: PlayerBehavior,
    #[case] should_be_allowed: bool,
) {
    let allowed = ActionScheduler::get_allowed_actions(&state);
    let is_allowed = allowed.iter().any(|a| same_variant(a, &action));

    assert_eq!(is_allowed, should_be_allowed);
}
```

**특징**:
- 테이블 형식으로 읽기 쉬움
- 케이스 추가 간단
- 실패 시 어떤 케이스인지 명확

### 4. State Machine Testing

```rust
#[test]
fn test_state_transitions() {
    let transitions = vec![
        (GameState::NotStarted, PlayerBehavior::StartNewGame,
         GameState::WaitingPhaseRequest),
        (GameState::WaitingPhaseRequest, PlayerBehavior::RequestPhaseData,
         GameState::SelectingEvent),
    ];

    for (initial_state, action, expected_state) in transitions {
        let mut game = setup_game_in_state(initial_state.clone());
        game.execute(player_id, action);

        assert_eq!(game.get_state(), expected_state);
    }
}
```

**특징**:
- FSM(Finite State Machine) 기반
- 상태 전환 검증에 최적
- 게임 로직에 매우 적합

### 5. Deterministic Replay Testing

```rust
#[derive(Debug, Serialize, Deserialize)]
struct GameReplay {
    seed: u64,
    actions: Vec<(Uuid, PlayerBehavior)>,
}

impl GameReplay {
    fn replay(&self, game_data: Arc<GameData>) -> GameCore {
        let mut game = GameCore::new(game_data, self.seed);

        for (player_id, action) in &self.actions {
            game.execute(*player_id, action.clone());
        }

        game
    }
}
```

**특징**:
- 입력 시퀀스 기록
- 같은 seed로 재실행 → 같은 결과
- 버그 재현에 매우 유용

---

## 최종 결정: AIPlayerActor

### 개념

**Seed 기반 자동 플레이 Actor**
- 목적을 가지고 생성되는 것이 아님
- Seed 기반 랜덤 선택으로 매 테스트마다 다른 플레이
- Seed만 기록하면 재현 가능
- Fuzz Testing 효과

```
AIPlayerActor(seed=42)
  ↓
현재 allowed_actions 확인
  ↓
seed 기반 랜덤 선택
  ↓
execute()
  ↓
반복 (GameOver까지)
```

### 핵심 장점

1. **랜덤하지만 결정론적**
   - 같은 seed = 같은 플레이
   - 버그 재현 가능

2. **매 테스트마다 다른 경로**
   - 다양한 플레이 패턴 자동 탐색
   - 예상 못한 버그 발견

3. **버그 재현 간단**
   ```
   Bug found with seed 12345!
   → AIPlayerActor::new(12345, ...) → 정확히 재현
   ```

4. **통계 수집**
   - 승률, 평균 스텝 수
   - 자원 획득량
   - 에러 발생 빈도

5. **CI/CD 통합**
   - 매 커밋마다 자동 실행
   - 병렬 처리로 대량 테스트

---

## 구현 가이드

### 기본 구조

```rust
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

pub struct AIPlayerActor {
    id: Uuid,
    seed: u64,
    rng: StdRng,
    game: GameCore,

    /// 플레이 기록 (버그 재현용)
    history: Vec<ActionRecord>,

    /// 통계
    stats: GameStats,
}

#[derive(Debug, Clone)]
pub struct ActionRecord {
    pub action: PlayerBehavior,
    pub state_before: GameState,
    pub state_after: GameState,
    pub result: Result<BehaviorResult, GameError>,
}

#[derive(Debug, Default)]
pub struct GameStats {
    pub total_actions: usize,
    pub successful_actions: usize,
    pub failed_actions: usize,
    pub phases_completed: usize,
    pub enkephalin_earned: u32,
    pub items_purchased: usize,
}
```

### 핵심 메서드

#### 1. 한 스텝 실행

```rust
impl AIPlayerActor {
    pub fn play_one_step(&mut self) -> Result<BehaviorResult, GameError> {
        // 1. 현재 상태 저장
        let state_before = self.game.get_state().clone();

        // 2. 현재 허용된 행동들 가져오기
        let allowed = self.game.get_allowed_actions();

        if allowed.is_empty() {
            return Err(GameError::InvalidAction);
        }

        // 3. seed 기반 랜덤 선택
        let action = allowed.choose(&mut self.rng).unwrap().clone();

        // 4. 실행
        let result = self.game.execute(self.id, action.clone());

        // 5. 상태 확인 및 기록
        let state_after = self.game.get_state().clone();

        self.history.push(ActionRecord {
            action: action.clone(),
            state_before,
            state_after,
            result: result.clone(),
        });

        // 6. 통계 업데이트
        self.update_stats(&action, &result);

        result
    }
}
```

#### 2. 전체 게임 플레이

```rust
impl AIPlayerActor {
    pub fn play_full_game(&mut self, max_steps: usize) -> GamePlayResult {
        let start_time = std::time::Instant::now();

        for step in 0..max_steps {
            match self.play_one_step() {
                Ok(_) => {
                    if self.game.get_state() == &GameState::GameOver {
                        return GamePlayResult::Victory {
                            seed: self.seed,
                            steps: step + 1,
                            duration: start_time.elapsed(),
                            stats: self.stats.clone(),
                        };
                    }
                }
                Err(GameError::InvalidAction) => {
                    return GamePlayResult::Stuck {
                        seed: self.seed,
                        steps: step,
                        last_state: self.game.get_state().clone(),
                    };
                }
                Err(e) => {
                    return GamePlayResult::Error {
                        seed: self.seed,
                        steps: step,
                        error: e,
                        history: self.history.clone(),
                    };
                }
            }
        }

        GamePlayResult::Timeout {
            seed: self.seed,
            steps: max_steps,
        }
    }
}

#[derive(Debug)]
pub enum GamePlayResult {
    Victory {
        seed: u64,
        steps: usize,
        duration: std::time::Duration,
        stats: GameStats,
    },
    Stuck {
        seed: u64,
        steps: usize,
        last_state: GameState,
    },
    Error {
        seed: u64,
        steps: usize,
        error: GameError,
        history: Vec<ActionRecord>,
    },
    Timeout {
        seed: u64,
        steps: usize,
    },
}
```

### 테스트 예시

#### 1. 기본 스모크 테스트

```rust
#[test]
fn ai_smoke_test() {
    let game_data = test_game_data();

    // 10개 다른 seed로 플레이
    for seed in 0..10 {
        let mut ai = AIPlayerActor::new(seed, game_data.clone());
        let result = ai.play_full_game(100);

        // 에러 없이 완료되어야 함
        assert!(matches!(
            result,
            GamePlayResult::Victory { .. } | GamePlayResult::Timeout { .. }
        ));
    }
}
```

#### 2. 대량 테스트

```rust
#[test]
fn test_ai_plays_100_games() {
    let game_data = test_game_data();
    let mut results = vec![];

    for seed in 0..100 {
        let mut ai = AIPlayerActor::new(seed, game_data.clone());
        let result = ai.play_full_game(1000);
        results.push(result);
    }

    // 통계 분석
    let victories = results.iter()
        .filter(|r| matches!(r, GamePlayResult::Victory { .. }))
        .count();
    let errors = results.iter()
        .filter(|r| matches!(r, GamePlayResult::Error { .. }))
        .count();

    println!("Victories: {}/100", victories);
    println!("Errors: {}/100", errors);

    // 에러 발견 시 seed 출력
    for result in results {
        if let GamePlayResult::Error { seed, error, history, .. } = result {
            println!("\n❌ Bug found with seed {}!", seed);
            println!("Error: {:?}", error);
            println!("History: {:#?}", history);
            panic!("Bug found! Rerun with seed {}", seed);
        }
    }
}
```

#### 3. 버그 재현 테스트

```rust
#[test]
fn test_reproduce_bug() {
    // 버그 발견된 seed로 재현
    let seed = 42;

    let mut ai = AIPlayerActor::new(seed, test_game_data());
    let result = ai.play_full_game(1000);

    // 디버그
    println!("History: {:#?}", ai.history);

    // 버그 수정 후 이 테스트가 통과해야 함
    assert!(matches!(result, GamePlayResult::Victory { .. }));
}
```

#### 4. 병렬 테스트

```rust
#[test]
fn test_parallel_ai_plays() {
    use rayon::prelude::*;

    let game_data = Arc::new(test_game_data());

    // 병렬로 1000개 게임 실행
    let results: Vec<_> = (0..1000)
        .into_par_iter()
        .map(|seed| {
            let mut ai = AIPlayerActor::new(seed as u64, game_data.clone());
            ai.play_full_game(1000)
        })
        .collect();

    // 분석
    let victories: Vec<_> = results.iter()
        .filter_map(|r| {
            if let GamePlayResult::Victory { steps, stats, .. } = r {
                Some((*steps, stats.clone()))
            } else {
                None
            }
        })
        .collect();

    let avg_steps = victories.iter()
        .map(|(s, _)| *s as f64)
        .sum::<f64>() / victories.len() as f64;

    println!("Average steps to victory: {:.2}", avg_steps);
    println!("Win rate: {}/{}", victories.len(), results.len());
}
```

### CI/CD 통합

```toml
# .github/workflows/test.yml
name: AI Player Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      # 빠른 스모크 테스트 (매 커밋)
      - name: Quick AI test
        run: cargo test ai_smoke_test -- --nocapture

      # 주간 대규모 테스트
      - name: Extensive AI test
        if: github.event_name == 'schedule'
        run: cargo test ai_soak_test --release -- --ignored --nocapture
```

---

## 추가 아이디어

### AI 성향 추가

```rust
pub enum AIPersonality {
    /// 완전 랜덤
    Random,

    /// 탐욕적 (골드 최대화)
    Greedy,

    /// 보수적 (안전한 선택)
    Conservative,

    /// 항상 첫 번째 선택
    Deterministic,
}

impl AIPlayerActor {
    pub fn with_personality(
        seed: u64,
        personality: AIPersonality,
        game_data: Arc<GameData>
    ) -> Self {
        // ...
    }

    fn choose_action(&mut self, allowed: &[PlayerBehavior]) -> PlayerBehavior {
        match self.personality {
            AIPersonality::Random => {
                allowed.choose(&mut self.rng).unwrap().clone()
            }
            AIPersonality::Greedy => {
                // 골드를 가장 많이 얻는 선택 우선
                self.choose_greedy(allowed)
            }
            AIPersonality::Conservative => {
                // 위험이 적은 선택 우선
                self.choose_conservative(allowed)
            }
            AIPersonality::Deterministic => {
                allowed.first().unwrap().clone()
            }
        }
    }
}
```

---

## 최종 추천 전략

### 레이어드 접근

1. **CI에서 매번 실행** (빠른 스모크 테스트)
   ```rust
   #[test]
   fn ai_smoke_test() {
       for seed in 0..10 {
           let mut ai = AIPlayerActor::new(seed, test_data());
           let result = ai.play_full_game(100);
           assert!(result.is_ok());
       }
   }
   ```

2. **주기적으로 실행** (광범위한 탐색)
   ```rust
   #[test]
   #[ignore] // cargo test -- --ignored로 실행
   fn ai_soak_test() {
       for seed in 0..1000 {
           let mut ai = AIPlayerActor::new(seed, test_data());
           ai.play_full_game(1000);
       }
   }
   ```

3. **특정 버그 재현**
   ```rust
   #[test]
   fn test_bug_seed_12345() {
       let mut ai = AIPlayerActor::new(12345, test_data());
       let result = ai.play_full_game(1000);
       // 버그 수정 후 이 테스트가 통과해야 함
       assert!(matches!(result, GamePlayResult::Victory { .. }));
   }
   ```

4. **특정 시나리오는 수동으로** (필요시)
   ```rust
   #[test]
   fn test_sell_non_owned_item() {
       let mut game = setup_game();
       let result = game.execute(player_id, PlayerBehavior::Sell {
           item_uuid: Uuid::new_v4()
       });
       assert_eq!(result, Err(GameError::InvalidAction));
   }
   ```

### 보완 도구

필요에 따라 추가:
- `rstest`: 테이블 기반 파라미터 테스트
- `insta`: 스냅샷 테스트 (이벤트 생성 결과 검증)
- `proptest`: 특정 속성 검증 (예: Enkephalin은 항상 음수가 아님)

---

## 구현 우선순위

1. **Phase 1**: AIPlayerActor 기본 구현
   - `play_one_step()`, `play_full_game()`
   - 10개 seed 스모크 테스트

2. **Phase 2**: 통계 수집 및 분석
   - `GameStats`, `ActionRecord`
   - 100개 seed 테스트 + 분석

3. **Phase 3**: CI 통합
   - GitHub Actions 설정
   - 자동 버그 리포트

4. **Phase 4**: 고급 기능 (선택)
   - AI Personality
   - 병렬 실행
   - 스냅샷 비교
