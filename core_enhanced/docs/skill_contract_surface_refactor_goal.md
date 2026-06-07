# Skill Contract Surface Refactor Goal

이 goal은 DefenseRoute 단일 전투 정책 이후 스킬 표시/발동/데이터 계약의 source of truth를 정리하기 위한 리팩토링 계획서다.

핵심 목적은 새 스킬 시스템을 만드는 것이 아니라, Unity-facing 스킬 계약을 타입으로 고정하고, 수동 스킬 버튼과 대상 선택 흐름을 명확히 분리하며, 반복되는 RON range 데이터와 buff 데이터 로딩 구조의 기술 부채를 줄이는 것이다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/skill_contract_surface_refactor/PLAN.md
docs/goals/skill_contract_surface_refactor/EXPERIMENTS.md
docs/goals/skill_contract_surface_refactor/EXPERIMENT_NOTES.md
```

각 파일의 역할:

- `PLAN.md`: 구현 순서, 현재 판단, 남은 체크리스트, 완료 조건을 기록한다.
- `EXPERIMENTS.md`: 시도한 접근, 실패/성공 결과, 테스트 실패 원인과 해결을 기록한다.
- `EXPERIMENT_NOTES.md`: 작업 중 발견한 의심, 정책 질문, 기술 부채 후보를 시간순으로 기록한다.

이 파일들은 최종 정책 문서가 아니라 goal 실행 중의 작업 기억장치다. goal 완료 후 유지해야 할 내용만 `unity_core_contract.md`, `skill_target_contract.md`, `game_rulebook.md`, `refactor_preparation_plan.md`로 옮긴다.

## Engineering Principles

- 코드 수정은 장기적인 방향으로 한다. 임시방편, 최소 수정, 특정 테스트만 맞추는 패치는 피한다.
- 처음에는 어떤 구조가 장기적인 방향인지 확실하지 않을 수 있다. 작은 단위로 trial and error를 수행하고, 실패한 접근과 이유를 `EXPERIMENTS.md`에 남긴다.
- 레거시는 과감하게 제거한다. 제거 가능한 구형 스킬 표시/발동 계약을 adapter나 compatibility layer로 감싸서 보존하지 않는다.
- 문서를 무조건 신뢰하지 않는다. 실제 코드, live RON, snapshot JSON, server mapping을 읽으면서 더 나은 개선안이 있으면 근거를 기록하고 적용한다.
- 문서에 적힌 내용보다 코드에서 더 단순하고 안전한 개선안이 보이면, 왜 더 나은지 기록한 뒤 적용한다. 단, 정책 판단이 필요한 경우 사용자에게 질문하고 goal을 종료한다.
- 새 추상화는 실제 반복이 확인된 곳에만 만든다. 이번 goal의 DTO는 Unity-facing 계약을 컴파일 단계에서 고정하기 위한 타입이며, 일반적인 future-proof 추상화가 아니다.
- 테스트는 내부 구조보다 실제 snapshot 계약, live skill readiness 계약, RON 로딩 실패/성공, Unity가 소비하는 JSON shape를 고정한다.
- 정책 변경으로 실패하는 레거시 테스트는 ignored 처리하지 않는다. 공식 흐름과 맞지 않으면 삭제하거나 최신 계약 테스트로 교체한다.

## Source Of Truth

우선 읽을 파일:

- `src/game/world/snapshot.rs`: `skill_catalog` snapshot 생성.
- `src/game/behavior.rs`: Unity-facing DTO와 `LiveBattleSkillReadinessDto`.
- `src/game/battle/core/sim.rs`: `live_skill_readiness`, 수동 스킬 발동 검증.
- `src/game/ability.rs`: `SkillDef`, `SkillStepDef`, `DeliveryDef`, `SkillActivationMode`, tile range 타입.
- `src/game/data/skill_data.rs`: skill RON validation.
- `src/game/battle/buffs.rs`: buff RON 로딩과 static registry.
- `src/game/data/mod.rs`: `GameDatabase` 구성과 데이터 source of truth.
- `src/game/world/tests.rs`, `tests/ron_loading.rs`, `tests/skill_refactor_validation.rs`, `tests/skill_test/**`: snapshot/skill 계약 테스트.
- `../game_resources/data/skills/base.ron`: live skill source of truth.
- `../game_resources/data/buffs/base.ron`: live buff source of truth.
- `docs/unity_core_contract.md`: Unity-facing snapshot/command 계약.
- `docs/skill_target_contract.md`: DefenseRoute tile range 계약.
- `docs/game_rulebook.md`: 게임 정책.
- `docs/refactor_preparation_plan.md`: 리팩토링 판단 기준.

필요하면 `../game_server`의 `/game` request/result/push mapping도 확인한다. snapshot shape나 behavior DTO가 server serialization에 영향을 주면 server 테스트와 문서도 같이 갱신한다.

## Current Findings

현재 확인된 기술 부채:

- `skill_catalog`가 `src/game/world/snapshot.rs`에서 `json!`로 직접 조립된다. Unity-facing 계약인데도 필드명, serde casing, 문서 예시 불일치를 컴파일 단계에서 잡지 못한다.
- `live_skill_readiness`가 target이 없다는 이유로 곧바로 `manual_activation_allowed=false`를 반환한다. 명일방주식 `스킬 버튼 클릭 -> 타일/대상 선택 -> ActivateSkill` 흐름에서는 버튼 사용 가능 여부와 현재 target 보유 여부가 분리되어야 한다.
- `../game_resources/data/skills/base.ron`에 같은 `defense_tile_range` 패턴이 반복된다. 스킬 수가 늘면 범위 수정과 복사 실수 가능성이 커진다.
- `BuffDatabase`만 `include_str! + static REGISTRY`를 사용한다. 다른 게임 데이터가 `GameDatabase`로 묶이는 흐름과 달라 source of truth가 분산된다.

## Objective

완료 후 다음이 가능해야 한다.

1. Unity가 소비하는 `skill_catalog` JSON이 typed DTO에서 생성된다.
2. `skill_catalog` field name, serde casing, 문서 예시가 코드 타입과 일치한다.
3. 수동 스킬 readiness는 “발동 버튼을 누를 수 있는가”와 “지금 즉시 자동 target이 있는가”를 분리해서 내려준다.
4. Unity는 manual skill button을 열어둔 채 target 선택 UI를 띄울 수 있다.
5. 반복되는 tile range 패턴은 preset 참조 또는 명확한 데이터 구조로 분리되어 live RON의 복사 실수를 줄인다.
6. buff 데이터는 가능하면 `GameDatabase` 흐름에 편입되어 source of truth가 줄어든다.

## In Scope

### 1. Typed Skill Catalog DTO

- `SkillCatalogDto`
- `SkillCatalogSkillDto`
- `SkillCatalogStepDto`
- `SkillCatalogCastTargetDto`
- `SkillCatalogTileAreaDto`
- 필요한 경우 `SkillCatalogDeliveryKind`

계약 규칙:

- DTO는 `Serialize`, 필요하면 `Deserialize`, `Clone`, `Debug`, `PartialEq`를 가진다.
- 기존 JSON shape를 임의로 바꾸지 않는다. 바꿔야 할 이유가 있으면 문서와 테스트를 함께 갱신한다.
- `snapshot.rs`는 `json!` 조립 대신 DTO를 생성한 뒤 `serde_json::to_value` 또는 snapshot 구조체 직렬화를 사용한다.
- `delivery`는 공식 값인 `instant`, `projectile`, `tile_area`만 노출한다.

### 2. Manual Skill Readiness Contract

`LiveBattleSkillReadinessDto`를 확장한다.

권장 필드:

```rust
pub struct LiveBattleSkillReadinessDto {
    pub skill_id: Option<SkillId>,
    pub activation_mode: SkillActivationMode,
    pub resonance_current: u32,
    pub resonance_max: u32,
    pub manual_activation_allowed: bool,
    pub target_required: bool,
    pub target_available: bool,
    pub can_activate_reason: Option<String>,
    pub target_block_reason: Option<String>,
}
```

계약 규칙:

- `manual_activation_allowed`는 버튼을 눌러 스킬 사용 흐름을 시작할 수 있는지를 의미한다.
- `target_required`는 스킬이 타일/대상 선택을 요구하는지를 의미한다.
- `target_available`은 현재 자동 검증 기준으로 즉시 사용할 target이 존재하는지를 의미한다.
- target이 없다는 이유만으로 `manual_activation_allowed=false`가 되면 안 된다.
- Silence, action lock, resonance 부족, unit unavailable, no skill, manual mode 아님 등 caster 자체가 발동 불가능한 조건은 `manual_activation_allowed=false`로 유지한다.
- `ActivateSkill` command는 서버가 최종 검증한다.

### 3. Skill Range Preset RON

반복되는 `defense_tile_range` 패턴을 줄이기 위한 데이터 계약을 설계하고 구현한다.

권장 방향:

- 별도 RON 데이터에 range preset을 둔다.
- `SkillStepDef`는 기존 inline `defense_tile_range`와 preset reference 중 하나를 사용할 수 있게 한다.
- live RON migration은 한 번에 모든 스킬을 바꾸기보다 대표 반복 패턴부터 preset으로 이동한다.
- 최종 source of truth가 둘로 갈라지지 않게 validation에서 inline과 preset의 동시 지정은 금지한다.

예시 방향:

```ron
SkillRangePresetDatabase(
    presets: [
        SkillRangePresetDef(
            id: "front_3x2",
            range: (include_anchor_tile: false, rows: [".XXX.", ".XXX.", "..@..", ".....", "....."]),
        ),
    ],
)
```

정확한 Rust 타입명과 RON shape는 실제 코드 구조를 읽고 결정한다.

### 4. BuffDatabase Source Of Truth Integration

`BuffDatabase`를 가능하면 `GameDatabase` 구성 흐름에 포함한다.

권장 방향:

- `include_str! + static REGISTRY`를 제거하거나, live runtime 조회가 필요한 경우에도 `GameDatabase`에서 생성된 registry를 source of truth로 삼는다.
- `buffs::get()` 같은 global accessor가 live path에 깊게 퍼져 있다면 한 번에 무리하게 바꾸지 말고, 호출부를 분류한 뒤 BattleCore/World 생성 시 명시 주입하는 방향을 우선 검토한다.
- global registry 제거가 scope를 과도하게 키우면 goal을 종료하고 별도 BuffDatabase integration goal로 분리할지 사용자에게 보고한다.

## Out Of Scope

- 신규 스킬 컨텐츠 추가.
- 스킬 수치 밸런스 조정.
- 새로운 skill runtime/effect runtime 작성.
- Unity 클라이언트 UI 구현.
- 조건부 BattleScenario 이벤트 구현.
- DefenseRoute 외 전투 모드 부활.
- 소모품/장비 효과 추가.
- geometric AoE 복구.

## Implementation Plan

1. 작업 기억장치 생성

- `docs/goals/skill_contract_surface_refactor/PLAN.md`
- `docs/goals/skill_contract_surface_refactor/EXPERIMENTS.md`
- `docs/goals/skill_contract_surface_refactor/EXPERIMENT_NOTES.md`

2. 현재 계약 수집

- 실제 snapshot JSON을 생성하는 테스트 또는 기존 snapshot 테스트를 확인한다.
- `skill_catalog`의 현재 JSON shape를 문서 예시와 대조한다.
- `LiveBattleSkillReadinessDto`가 snapshot/server를 통해 어떻게 내려가는지 확인한다.
- `BuffDatabase` 호출부를 `rg "buffs::get|BuffDatabase|BuffId::from_name"`로 분류한다.

3. `skill_catalog` DTO 도입

- DTO 타입을 적절한 모듈에 추가한다. 우선 `snapshot.rs` 내부 private DTO로 시작해도 된다.
- DTO가 다른 모듈/서버에서 재사용될 근거가 있으면 `behavior.rs` 또는 별도 DTO 모듈로 이동한다.
- 기존 JSON shape를 보존하는 focused test를 추가한다.
- 문서 예시와 serde casing을 실제 DTO 결과에 맞춘다.

4. `live_skill_readiness` 분리

- target 없이도 버튼을 열 수 있는 manual skill과 caster 상태상 아예 발동 불가능한 skill을 구분한다.
- `resolve_manual_skill_cast_request`를 무리하게 재사용해 버튼 가능 여부를 판단하지 않는다. 필요하면 target availability용 helper를 따로 둔다.
- `target_required`, `target_available`, `target_block_reason`을 snapshot에 노출한다.
- Unity 계약 문서를 갱신한다.

5. range preset 설계/적용

- 반복되는 live RON 패턴을 집계한다.
- preset schema를 작게 도입한다.
- inline range와 preset reference가 동시에 지정되면 validation error가 나게 한다.
- 대표 반복 패턴을 preset으로 이동한다.
- RON loading focused test로 migration을 검증한다.

6. BuffDatabase 통합 가능성 판단

- global `buffs::get()` 호출부가 BattleCore 내부에만 가까운지, world/effect/data validation까지 퍼져 있는지 분류한다.
- 작게 끝낼 수 있으면 `GameDatabase`에 포함하고 BattleCore가 명시적으로 참조하게 한다.
- scope가 지나치게 커지면 goal을 종료하고 “BuffDatabase integration 별도 goal 필요”로 보고한다.

7. 문서 갱신

- `unity_core_contract.md`: skill catalog DTO shape, readiness field 의미.
- `skill_target_contract.md`: range preset이 도입되면 inline/preset 관계.
- `game_rulebook.md`: 필요 시 수동 스킬 target 선택 정책.
- `refactor_preparation_plan.md`: 남은 buff/range 후속 과제가 있으면 최신 기준만 간단히 반영.

## Test Requirements

focused test를 먼저 작성/갱신한다.

- `skill_catalog` snapshot이 기존 필드와 serde casing을 유지한다.
- `skill_catalog.skills[*].steps[*].delivery`는 `instant`, `projectile`, `tile_area`만 가진다.
- `manual_activation_allowed=true`, `target_required=true`, `target_available=false`가 가능한 상태를 검증한다.
- resonance 부족, silence, action lock, dead/unavailable 상태에서는 `manual_activation_allowed=false`를 유지한다.
- `ActivateSkill` command는 target이 없거나 잘못되면 여전히 서버에서 실패한다.
- range preset reference가 실제 `defense_tile_range`로 해석된다.
- inline range와 preset reference 동시 지정은 validation에서 실패한다.
- buff database를 통합했다면 기존 buff effect 테스트가 동일하게 통과한다.

검증 순서:

```text
cargo test -p game_core skill_catalog -- --nocapture
cargo test -p game_core live_skill_readiness -- --nocapture
cargo test -p game_core ron_loading -- --nocapture
cargo test -p game_core skill_refactor_validation -- --nocapture
cargo test -p game_core -- --nocapture
cargo check -p game_core
cargo check -p game_server
```

server mapping을 변경했다면 다음도 수행한다.

```text
cargo test -p game_server -- --nocapture
```

## Stop Conditions

다음 경우 임의 확정하지 말고 goal을 종료하고 질문 목록을 보고한다.

- `skill_catalog` JSON shape 변경이 Unity 클라이언트 구현에 영향을 주는데 문서상 최신 계약과 충돌한다.
- manual skill readiness에서 target 선택 UX 정책이 불명확하다.
- `target_required`, `target_available`, `manual_activation_allowed` 외에 추가 상태가 필요하다고 판단된다.
- range preset RON schema가 기존 live RON 작성성을 크게 해친다.
- BuffDatabase 통합이 BattleCore 생성자, effect runtime, validation, tests 전반을 광범위하게 바꿔야 해서 별도 goal로 분리하는 편이 안전하다.
- live path에서 global buff registry가 필요한 명확한 이유가 발견된다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Completion Criteria

1. `skill_catalog`가 typed DTO에서 생성된다.
2. `skill_catalog` 문서 예시와 실제 snapshot JSON이 일치한다.
3. 수동 스킬 readiness가 `manual_activation_allowed`와 target 상태를 분리해 노출한다.
4. target이 없다는 이유만으로 수동 스킬 버튼이 비활성화되지 않는다.
5. 반복되는 live skill range 패턴이 preset 또는 명확한 데이터 구조로 분리된다.
6. inline range와 preset reference의 source of truth 충돌이 validation으로 차단된다.
7. BuffDatabase를 `GameDatabase` 흐름에 통합했거나, scope가 큰 경우 별도 goal 필요성을 `EXPERIMENTS.md`와 최종 보고에 명확히 남겼다.
8. 레거시 테스트는 ignored 처리 없이 삭제 또는 최신 계약 테스트로 교체된다.
9. 필요한 문서가 최신 계약으로 갱신된다.
10. focused test와 필요한 `cargo test`/`cargo check`가 통과한다.
11. 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Goal Command

```text
/goal docs/skill_contract_surface_refactor_goal.md를 기준으로, DefenseRoute 단일 전투 정책 이후 남은 스킬 계약 표면의 기술 부채를 정리하라. 먼저 docs/goals/skill_contract_surface_refactor/PLAN.md, EXPERIMENTS.md, EXPERIMENT_NOTES.md를 만들고, src/game/world/snapshot.rs, src/game/behavior.rs, src/game/battle/core/sim.rs, src/game/ability.rs, src/game/data/skill_data.rs, src/game/battle/buffs.rs, src/game/data/mod.rs, src/game/world/tests.rs, tests/ron_loading.rs, tests/skill_refactor_validation.rs, tests/skill_test/**, ../game_resources/data/skills/base.ron, ../game_resources/data/buffs/base.ron, docs/unity_core_contract.md, docs/skill_target_contract.md, docs/game_rulebook.md, docs/refactor_preparation_plan.md를 실제 코드 기준으로 읽고 현재 계약을 확인하라. skill_catalog는 json! 직접 조립을 제거하고 SkillCatalogDto, SkillCatalogSkillDto, SkillCatalogStepDto, SkillCatalogTileAreaDto 등 typed DTO에서 생성되게 하라. live_skill_readiness는 manual_activation_allowed와 target_required/target_available/target_block_reason을 분리해, 타겟이 없다는 이유만으로 수동 스킬 버튼이 비활성화되지 않게 하라. 반복되는 defense_tile_range RON 패턴은 range preset 또는 동등하게 명확한 데이터 구조로 분리하고, inline range와 preset reference가 동시에 지정되는 source of truth 충돌은 validation으로 차단하라. BuffDatabase만 include_str! + static registry로 남은 source of truth 불일치를 검토하고, 작은 범위로 가능하면 GameDatabase 흐름에 통합하라. 단 BuffDatabase 통합 범위가 BattleCore/effect runtime/validation 전반을 크게 흔들면 별도 goal로 분리할지 사용자와 의논하기 위해 goal을 종료하라. 코드 수정은 장기적인 방향으로 하고 임시방편, 최소 수정, 특정 테스트만 맞추는 패치를 피하라. 처음 구조 판단이 어려우면 작은 trial and error를 수행하고 PLAN/EXPERIMENTS/EXPERIMENT_NOTES에 계획, 실험, 생각을 기록하라. 레거시는 과감하게 제거하고 compatibility layer로 감싸지 말라. 문서를 무조건 신뢰하지 말고 실제 코드와 live RON/API를 읽으면서 더 나은 개선안이 보이면 근거를 기록하고 적용하라. 단 skill_catalog JSON shape 변경, manual skill target 선택 UX, range preset RON schema, BuffDatabase 통합 범위처럼 사용자와 의논해야 할 정책이 발견되면 goal을 종료하고 질문 목록을 보고하라. 테스트는 내부 구조가 아니라 Unity-facing snapshot 계약, 수동 스킬 readiness 계약, RON 로딩/validation, 실제 DefenseRoute 스킬 사용 흐름을 고정하도록 갱신하라. 검증은 cargo test -p game_core skill_catalog -- --nocapture, cargo test -p game_core live_skill_readiness -- --nocapture, cargo test -p game_core ron_loading -- --nocapture, cargo test -p game_core skill_refactor_validation -- --nocapture, cargo test -p game_core -- --nocapture, cargo check -p game_core, cargo check -p game_server 순서로 수행하고, server mapping을 바꿨다면 cargo test -p game_server -- --nocapture도 수행하라. 완료 조건은 1) skill_catalog typed DTO화, 2) 실제 snapshot과 unity_core_contract.md 일치, 3) manual_activation_allowed와 target 상태 분리, 4) target 없음만으로 수동 스킬 버튼이 비활성화되지 않음, 5) range preset 또는 명확한 반복 제거 구조 도입, 6) range source of truth 충돌 validation, 7) BuffDatabase 통합 또는 별도 goal 필요성 기록, 8) 레거시 테스트 ignored 없이 삭제/교체, 9) 필요한 문서 갱신, 10) 검증 명령 통과, 11) 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료함이다.
```
