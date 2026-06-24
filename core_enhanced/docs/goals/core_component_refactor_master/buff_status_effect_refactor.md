# 버프/상태이상 시스템 리팩토링 감사

- 컴포넌트: 버프/상태이상 시스템
- 기준 문서: `docs/refactor_preparation_plan.md`
- 상태: Initial audit documented

## 읽은 범위

- Runtime code:
  - `src/game/battle/buffs.rs`
  - `src/game/battle/core/sim.rs`
  - `src/game/battle/core/types.rs`
  - `src/game/battle/core/commands.rs`
  - `src/game/battle/enums.rs`
  - `src/game/battle/timeline.rs`
- Validation/test code:
  - `src/game/battle/validation/buffs.rs`
  - `src/game/battle/validation/validator.rs`
  - `src/game/battle/core/mod.rs`
  - `tests/ron_loading.rs`
  - `tests/live_item_skill_activation.rs`
- Live data:
  - `../game_resources/data/buffs/base.ron`
- Policy/contract docs:
  - `docs/refactor_preparation_plan.md`
  - `docs/skill_target_contract.md`
  - `docs/component_design_review.md`
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`

## 현재 구조 요약

`BuffDatabase`는 live RON의 `BuffMetadata` 목록을 읽고 `BuffId::from_name()`으로 deterministic id를 만든 뒤 `OnceLock<HashMap<BuffId, BuffDef>>` 인덱스를 제공한다(`src/game/battle/buffs.rs:9`, `src/game/battle/buffs.rs:58`, `src/game/battle/buffs.rs:84`, `src/game/battle/buffs.rs:91`). 현재 live 데이터에는 `poison`, `stun`, `freeze`, `silence`가 있고 poison만 periodic damage다(`../game_resources/data/buffs/base.ron`).

Runtime active buff state는 caster/target/buff id를 키로 삼는다(`src/game/battle/core/types.rs:442`). `BattleEvent::ApplyBuff`는 `game_data.buff_data`에서 정의를 찾고, duration 0 또는 사망 대상이면 무시한다(`src/game/battle/core/sim.rs:2577`, `src/game/battle/core/sim.rs:2581`, `src/game/battle/core/sim.rs:2585`). 적용 성공 시 `BuffApplied` timeline을 기록하고, reapply policy에 따라 stack/expiry/tick cadence를 갱신한 뒤 `BuffTick`과 `BuffExpire`를 예약한다(`src/game/battle/core/sim.rs:2592`, `src/game/battle/core/sim.rs:2641`, `src/game/battle/core/sim.rs:2654`, `src/game/battle/core/sim.rs:2668`).

`Stun`과 `Freeze`는 hard CC로 취급되어 같은 대상의 기존 hard CC를 active buff map에서 제거한다(`src/game/battle/core/sim.rs:2612`, `src/game/battle/core/sim.rs:2618`). hard CC 적용 시 `next_action_time`과 `ActionLocks`를 expiry+1까지 max 방식으로 잠그고 movement를 interrupt한다(`src/game/battle/core/sim.rs:2676`). `Silence`는 hard CC replacement 대상이 아니지만 manual/auto skill start를 막는 상태로 쓰인다(`src/game/battle/core/sim.rs:475`, `src/game/battle/core/sim.rs:566`). 정책 문서도 `Silence`는 새 시전 시작을 막고 이미 시작된 pending cast를 취소하지 않는다고 분리한다(`docs/skill_target_contract.md`).

Timeline validator는 runtime과 별도의 `BuffInstanceKey`, `ActiveBuff`, hard CC predicate, reapply/tick/expire 모델을 갖고 `BuffApplied`, `BuffTick`, `BuffExpired` 순서를 검증한다(`src/game/battle/validation/buffs.rs:11`, `src/game/battle/validation/buffs.rs:25`, `src/game/battle/validation/buffs.rs:29`).

## Source-of-truth 판단

- Buff definition source of truth는 `GameDataBase.buff_data`다. `docs/refactor_preparation_plan.md`도 BuffDatabase가 `GameDataBase` source of truth로 통합되는 방향을 명시하고, battle status buff와 consumable modifier는 수명주기가 다르므로 합치지 말라고 한다.
- Buff runtime state source of truth는 `BattleCore.buffs`의 `ActiveBuff`다. 그러나 hard CC의 행동 제한은 `ActiveBuff`와 별도로 `RuntimeUnit.next_action_time`/`ActionLocks`에도 저장된다.
- Timeline presentation source of truth는 `BuffApplied`, `BuffTick`, `BuffExpired`다. Unity 계약은 이 세 이벤트를 버프 표시 기준으로 적고 있다(`/mnt/f/unity projects/ark/docs/unity_core_contract.md:1147`).
- Timeline validator는 runtime 상태를 직접 읽지 않는 독립 검증 모델이다. 독립 모델 자체는 유효하지만, hard CC/reapply/tick 정책이 runtime과 validator에 복제되어 drift 위험이 있다.

## 리팩토링 후보

### 1. Hard CC active buff와 action lock의 source-of-truth split

`ApplyBuff`는 Stun/Freeze 적용 시 기존 hard CC active buff를 제거한다(`src/game/battle/core/sim.rs:2618`). 하지만 행동 제한은 `next_action_time.max(lock_until)`과 `ActionLocks::lock_*_until(lock_until)`로만 연장된다(`src/game/battle/core/sim.rs:2676`). `ActionLocks` 자체도 더 짧은 lock 입력을 무시하는 max 방식이다(`src/game/battle/core/types.rs:460`).

이 때문에 긴 stun 뒤 짧은 freeze가 들어오면 active buff source of truth는 새 freeze의 짧은 expiry를 가리키지만, 행동 lock은 이전 긴 stun의 lock_until을 유지할 수 있다. 반대로 "hard CC는 기존 CC를 덮어씌운다"는 주석과 validator 모델은 active buff replacement만 표현한다(`src/game/battle/buffs.rs:30`, `src/game/battle/validation/buffs.rs:70`).

판단: 높은 우선순위 리팩토링 후보. hard CC의 공식 행동 제한 source가 active buff인지, action lock인지, 또는 "교체해도 더 긴 lock은 유지" 정책인지 불명확하다. 사용자와 정책 논의 필요.

가능한 방향:

- active hard CC 상태에서 action lock을 파생하거나, hard CC 전용 lock source를 따로 두고 replacement 시 재계산한다.
- replacement가 buff 표시에만 적용되고 행동 lock은 max 유지라는 정책이라면 문서와 테스트로 고정한다.

필요 검증:

- 긴 Stun 적용 후 짧은 Freeze 적용 시 movement/basic/resonance lock 해제 시점을 고정하는 battle core test.
- Timeline validator가 active buff replacement와 행동 lock 정책을 둘 다 설명할 수 있는지 확인.

### 2. Hard CC replacement가 `BuffExpired`를 기록하지 않는다

Runtime은 새 hard CC 적용 시 기존 hard CC를 `self.buffs.retain()`으로 제거하지만 `BuffExpired` timeline을 기록하지 않는다(`src/game/battle/core/sim.rs:2618`). Validator도 같은 방식으로 기존 hard CC를 active set에서 제거하고, 기존 hard CC의 만료 이벤트가 나중에 오면 invalid로 본다(`src/game/battle/validation/buffs.rs:70`).

내부 상태 모델로는 일관적이지만 Unity 표시 source of truth가 `BuffApplied`/`BuffExpired`라면, 클라이언트가 새 hard CC `BuffApplied`만 보고 기존 hard CC 아이콘 제거를 추론해야 한다.

판단: Unity-facing presentation 계약 후보. replacement를 `BuffApplied`가 암시하는 것으로 둘지, 제거된 buff에 대해 별도 `BuffExpired`/replacement event를 낼지 정책이 필요하다. 사용자와 정책 논의 필요.

필요 검증:

- hard CC replacement 시 timeline event sequence를 고정하는 validator/core test.
- Unity 계약 문서에 "Stun/Freeze BuffApplied는 동일 대상 기존 hard CC 표시를 대체한다" 또는 별도 만료 이벤트 정책을 반영.

### 3. 사망 대상의 active buff 생존과 post-death `BuffTick`

`ApplyBuff`는 사망 대상에게 새 buff를 걸지 않는다(`src/game/battle/core/sim.rs:2585`). 하지만 `finalize_unit_death()`는 target의 active buffs를 제거하지 않는다(`src/game/battle/core/commands.rs:340`). `BuffTick` 처리도 active buff와 tick cadence가 맞으면 먼저 `BuffTick` timeline을 기록하고 그 다음 damage command를 처리한다(`src/game/battle/core/sim.rs:2715`, `src/game/battle/core/sim.rs:2725`, `src/game/battle/core/sim.rs:2744`).

따라서 poison 등 periodic buff가 남아 있는 유닛이 다른 원인으로 죽으면, scheduled tick이 도착했을 때 사망 대상에 대한 `BuffTick` 표시가 먼저 기록될 가능성이 있다. damage 적용 자체는 dead target path에서 무시될 수 있지만, Unity-facing timeline에는 post-death tick 표시가 남을 수 있다.

판단: lifecycle source-of-truth 후보. 죽음이 buff를 정리해야 하는지, 아니면 buff는 자연 만료까지 남고 tick event만 조건부로 suppress해야 하는지 정책이 필요하다. 사용자와 정책 논의 필요.

필요 검증:

- periodic buff가 걸린 유닛이 tick 전에 사망했을 때 `BuffTick`/`BuffExpired`가 어떻게 기록되는지 고정하는 battle core test.
- validator의 death-aware timeline 규칙과 presentation 계약 확인.

### 4. Runtime과 validator의 buff 정책 복제

Runtime과 validator는 각각 `BuffInstanceKey`, `ActiveBuff`, hard CC predicate, reapply policy, tick cadence를 별도로 구현한다(`src/game/battle/core/types.rs:442`, `src/game/battle/validation/buffs.rs:11`, `src/game/battle/validation/buffs.rs:25`, `src/game/battle/core/sim.rs:2641`, `src/game/battle/validation/buffs.rs:87`).

검증기가 runtime mutable state를 공유하지 않는 것은 좋다. 하지만 hard CC 대상이 Stun/Freeze인지, RefreshDuration이 stack을 1로 유지하는지, StackRefreshDurationKeepCadence가 expiry max를 쓰는지 같은 순수 정책은 drift가 나기 쉽다.

판단: 순수 정책 helper 추출 후보. runtime state와 validator state를 합치지는 말고, `is_exclusive_hard_cc`, `apply_reapply_policy`, `first_tick_after_apply` 같은 pure helper만 공유하면 source-of-truth 중복을 줄일 수 있다.

필요 검증:

- 기존 validator tests 유지.
- core hard CC/reapply/tick tests 유지.

### 5. `BuffDatabase::live_default()` 직접 include 사용 범위 정리

`BuffDatabase::live_default()`는 `game_resources/data/buffs/base.ron`을 직접 include한다(`src/game/battle/buffs.rs:84`). 공식 runtime path는 `GameDataBuilder::live_defaults()`가 이 값을 `GameDataBase.buff_data`로 넣고, runtime은 `self.game_data.buff_data`만 본다. 이 구조는 현재 기준과 대체로 맞다.

주의할 점은 `TimelineValidator::with_live_buff_data()` 같은 convenience가 검증 입력의 실제 `GameDataBase.buff_data`와 다른 live default를 암묵적으로 쓰는 경우다. custom/test data timeline을 검증할 때 hidden source-of-truth가 될 수 있다.

판단: 당장 제거 후보라기보다는 사용 범위 명시 후보. 공식 검증 path는 가능한 한 검증 대상 `GameDataBase.buff_data`를 주입해야 한다. live default helper는 builder/live fixture 전용으로 제한하는 것이 좋다.

### 6. `BuffId` hash collision 검증 부재

`BuffDatabase::build_registry()`는 duplicate name과 일부 metadata 조건만 검증한다(`src/game/battle/buffs.rs:91`). `BuffId::from_name()`은 deterministic hash이고, HashMap collect 과정에서 이론적 collision이 있으면 뒤 항목이 앞 항목을 덮을 수 있다(`src/game/battle/buffs.rs:13`, `src/game/battle/buffs.rs:113`).

가능성은 낮지만 source-of-truth 인덱스 검증 관점에서는 duplicate generated id를 assert하는 편이 낫다.

판단: 낮은 위험도의 validation 보강 후보. 정책 결정 없이 적용 가능하다.

### 7. Hard CC/상태이상 metadata invariant가 live test에만 묶여 있다

`build_registry()`는 `max_stacks > 0`과 periodic damage의 `tick_interval_ms > 0`만 검증한다(`src/game/battle/buffs.rs:101`, `src/game/battle/buffs.rs:106`). Live test는 stun/freeze/silence가 max stack 1이라고 확인하지만(`src/game/battle/buffs.rs:157`), 일반 metadata validation 규칙은 아니다.

Runtime에서 RefreshDuration은 `max_stacks.min(1)`을 쓰므로(`src/game/battle/core/sim.rs:2646`), Stun/Freeze/Silence에 `max_stacks > 1`이 들어와도 실제 stack은 1로 눌린다. 데이터에는 max stack이 3이라고 쓰여 있는데 runtime은 1처럼 동작하는 식의 혼란이 가능하다.

판단: metadata invariant 강화 후보. 다만 Silence가 향후 stackable silence 같은 디자인으로 바뀔 수 있는지와 연결되므로 사용자와 정책 논의 필요.

## 기존 기능 조합으로 단순화 가능한 후보

- Hard CC replacement와 action locking은 현재 active buff map, action lock, movement interrupt가 각각 상태를 들고 있다. 공식 정책이 "현재 active hard CC가 행동 제한을 결정한다"라면 action lock 저장을 별도 source로 유지하지 않고 active hard CC expiry에서 파생하는 방향을 검토할 수 있다.
- `Silence`는 이미 cast interrupt와 분리되어 있으며, 별도의 pending cast cancel 역할로 확장하지 않는 것이 기존 정책 조합에 맞다. 이 항목은 리팩토링 후보가 아니라 유지 판단이다.

## 레거시/fallback/dual schema 제거 후보

- runtime과 validator의 상태 타입 복제는 dual schema라기보다 독립 검증 모델이다. 제거 대상은 아니지만 pure policy 복제는 줄일 수 있다.
- `BuffDatabase::live_default()`는 live fixture/builder 편의 기능으로 남길 수 있으나, 공식 runtime/validator path에서 hidden fallback처럼 쓰이지 않게 사용 범위를 좁혀야 한다.

## 하지 않거나 보류한 항목

- Battle buff와 `ActiveConsumableModifier` 통합은 하지 않는다. `docs/refactor_preparation_plan.md`가 두 수명주기가 다르므로 합치지 말라고 명시한다.
- Silence를 hard CC replacement 대상에 넣는 변경은 하지 않는다. 현재 runtime, validator, 정책 문서가 Silence를 "새 skill start blocker"로 일관되게 다루며, pending cast interrupt와도 분리되어 있다.
- 이번 goal에서는 구현을 변경하지 않는다. 현재 산출물은 컴포넌트 감사 문서이며, 정책 논의 항목은 문서에 `사용자와 정책 논의 필요`로 남긴다.

## 필요한 테스트와 검증 명령

구현 리팩토링 착수 시 필요한 focused test:

- `cargo test -p game_core battle::core::` 중 hard CC replacement/lock 해제 시점 테스트.
- `cargo test -p game_core battle::validation::validator::buff_validator_models_exclusive_hard_cc_replacement`
- periodic buff target death 후 `BuffTick`/`BuffExpired` behavior를 고정하는 신규 test.
- `cargo test -p game_core ron_loading::` 또는 live buff loading 관련 test.

이번 감사 단계에서는 코드 변경이 없으므로 Rust test는 실행하지 않았다.

## 사용자와 정책 논의 필요

- Hard CC replacement가 행동 lock을 짧게 만들 수 있어야 하는가, 아니면 active buff만 교체되고 기존 긴 lock은 유지되는가: 사용자와 정책 논의 필요.
- Hard CC replacement 시 기존 hard CC에 대해 `BuffExpired` 또는 별도 replacement event를 기록해야 하는가: 사용자와 정책 논의 필요.
- 사망 시 active buff를 즉시 제거해야 하는가, 자연 만료까지 유지하되 tick timeline만 suppress해야 하는가: 사용자와 정책 논의 필요.
- Stun/Freeze/Silence 같은 non-periodic status의 `max_stacks == 1`을 data validation invariant로 고정할 것인가: 사용자와 정책 논의 필요.
