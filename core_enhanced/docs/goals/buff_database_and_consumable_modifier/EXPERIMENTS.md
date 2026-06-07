# BuffDatabase And Consumable Modifier Experiments

## Experiment Log

### E1. Current Contract Survey

- Status: complete
- Attempt: `buffs::get`, `contains_name`, `ActiveConsumableModifier`, `GameDataBuilder::empty`, live RON loader, timeline validator 호출부를 검색했다.
- Result:
  - BattleCore runtime은 `game_data`를 이미 들고 있어 source of truth 이동이 가능하다.
  - Timeline validation은 현재 독립 validator라 `BuffDatabase` 주입이 필요하다.
  - consumable modifier는 이미 직원 상태에 저장되고 전투 시작/배치/전투불능 후처리에 일부 연결되어 있다.

### E2. Builder Default Buff Policy

- Status: decided
- Attempt: `GameDataBuilder::empty()` 호출부를 확인했다.
- Result: 많은 unit/focused test가 empty builder로 만든 `GameDataBase`에서도 `poison/stun/silence` 같은 기본 buff가 존재한다고 암묵 가정한다.
- Decision: `GameDataBuilder::empty()`는 live 기본 buff RON을 포함한다. 특정 테스트가 빈 buff DB를 원하면 `with_buff_data(Arc::new(BuffDatabase::new(vec![])))`로 명시한다.

### E3. BuffDatabase GameDataBase Integration

- Status: applied
- Attempt: `BuffDatabase`에 `get`, `contains_name`, `validate_indexes`, `live_default`를 추가하고 `GameDataBase`/`GameDataBuilder`/`GameDataBaseParts`에 `buff_data`를 포함했다.
- Result: BattleCore runtime은 `self.game_data.buff_data`를 사용한다. `SkillDatabase`는 shape validation만 수행하고, buff id 교차검증은 `GameDataBase` 구성 시점에 `skill_data.validate_buff_references(&buff_data)`로 수행한다.

### E4. Timeline Validator Buff Source

- Status: applied
- Attempt: `TimelineValidator`가 `Arc<BuffDatabase>`를 들도록 하고, `with_buff_data` 생성자를 추가했다.
- Result: `timeline_validator_uses_injected_buff_database` focused test로 validator가 주입된 buff DB를 사용함을 고정했다.

### E5. Consumable Modifier Naming

- Status: applied
- Attempt: `ActiveConsumableBuff`/`active_consumable_buff`/`replaced_buff`/`applied_buff`를 `ActiveConsumableModifier`/`active_consumable_modifier`/`replaced_modifier`/`applied_modifier`로 정리했다.
- Result: core snapshot, behavior result, game_server result mapping, Unity contract 문서가 같은 명칭을 사용한다.

### E6. Verification

- Status: complete
- Result:
  - `cargo test -p game_core buffs -- --nocapture` 통과.
  - `cargo test -p game_core consumable -- --nocapture` 통과.
  - `cargo test -p game_core --test ron_loading -- --nocapture` 통과.
  - `cargo test -p game_core --test skill_refactor_validation -- --nocapture` 통과.
  - `cargo test -p game_core --test skill_test_suite -- --nocapture` 통과.
  - `cargo test -p game_core -- --nocapture` 통과.
  - `cargo test -p game_server -- --nocapture` 통과.
  - `cargo check -p game_core` 통과.
  - `cargo check -p game_server` 통과.
