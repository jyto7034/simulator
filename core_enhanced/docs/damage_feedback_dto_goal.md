# Damage Feedback DTO Goal

이 goal은 전투 피해 결과를 Unity가 추론 없이 표시할 수 있도록 `HpChanged.feedback_tags` 계약을 추가하는 작업이다.

관련 정책은 `docs/core_policy_decisions_2026_06.md`의 `피해 피드백`, `피해 타입과 AD/AP`를 따른다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/damage_feedback_dto/PLAN.md
docs/goals/damage_feedback_dto/EXPERIMENTS.md
docs/goals/damage_feedback_dto/EXPERIMENT_NOTES.md
```

- `PLAN.md`: 구현 순서, 현재 판단, 완료 체크리스트를 기록한다.
- `EXPERIMENTS.md`: 시도한 접근, 실패/성공 결과, 테스트 실패 원인을 기록한다.
- `EXPERIMENT_NOTES.md`: 작업 중 발견한 정책 질문, 기술 부채, 더 나은 개선안을 기록한다.

## Engineering Principles

- 코드 수정은 장기적인 방향으로 한다. 임시방편, 최소한의 수정, 특정 테스트만 맞추는 패치를 피한다.
- 처음 구조가 애매하면 작은 trial and error를 수행하고 결과를 `EXPERIMENTS.md`에 남긴다.
- 레거시는 과감하게 제거한다. 구형 damage event shape를 compatibility layer로 길게 끌고 가지 않는다.
- 문서를 무조건 신뢰하지 않는다. 실제 code path와 Unity-facing serialization을 읽고 더 나은 개선안이 보이면 기록하고 적용한다.
- 테스트가 이전 정책을 고정하고 있으면 ignored 처리보다 삭제/교체를 우선한다.
- 사용자와 의논하여 정해야 할 정책이 발견되면 임의로 확정하지 않고 goal을 종료한다.

## Goal Execution Contract

- source of truth 우선순위는 실제 runtime code, live RON/data, Unity-facing snapshot/command 계약, 최신 정책 문서 순서로 본다.
- 문서와 코드가 충돌하면 코드를 먼저 읽고, 최신 정책과 코드의 차이를 `EXPERIMENT_NOTES.md`에 기록한 뒤 수정 방향을 정한다.
- compatibility layer, fallback path, dual schema는 기본적으로 만들지 않는다. 정말 필요하면 이유, 제거 예정 조건, 테스트 범위를 `PLAN.md`에 기록하고 사용자 확인을 받는다.
- 테스트는 내부 구현 모양보다 사용자-visible behavior, Unity-facing DTO, data validation, live RON loading, 실제 gameplay flow를 고정한다.
- 새 정책과 충돌하는 테스트는 기대값만 바꾸지 않는다. 그 테스트가 무엇을 보호하던 것인지 확인한 뒤 삭제하거나 최신 정책 테스트로 교체한다.
- ignored test로 레거시를 보존하지 않는다.
- goal 범위를 벗어난 개선안은 바로 구현하지 말고 `EXPERIMENT_NOTES.md`에 후속 후보로 기록한다. 현재 goal 완료에 필수이거나 blast radius를 줄이는 경우에만 적용한다.
- Unity-facing DTO shape, live RON schema, 저장 데이터 migration, UX 의미 변화, 밸런스 기준, 기존 콘텐츠 삭제/대체, 실패/보상/소비 시점 변화처럼 사용자 결정이 필요한 정책이 발견되면 임의로 결정하지 않고 goal을 종료하고 질문 목록을 보고한다.
- 작은 변경 단위마다 빠른 focused test 또는 `cargo check`를 먼저 돌리고, 마지막에 넓은 테스트를 돌린다.
- 테스트 실패는 `EXPERIMENTS.md`에 실패 원인, 수정 내용, 재검증 결과를 기록한다.
- 완료 시 변경 요약, 제거한 레거시, 새로 고정한 계약, 갱신한 테스트, 남은 위험, 실행한 검증 명령을 보고한다.

## Canonical Unity Docs

Unity-facing 계약/구현 문서의 canonical 위치는 `F:\unity projects\ark\docs`다. Linux/WSL 경로로는 `/mnt/f/unity projects/ark/docs`다.

아래 파일은 외부 canonical 문서를 기준으로 확인하고 갱신한다.

- `F:\unity projects\ark\docs\unity_core_contract.md`
- `F:\unity projects\ark\docs\unity_client_implementation_goal.md`

이 저장소 안의 같은 이름 문서는 stale copy일 수 있다.

## Source Of Truth

우선 읽을 파일:

- `src/game/battle/damage.rs`: `DamageResult`, `DamageBreakdown`, `DamageModifiers`, `calculate_damage`.
- `src/game/battle/timeline.rs`: `TimelineEvent::HpChanged`.
- `src/game/battle/core/commands.rs`: damage 적용과 `HpChanged` 기록.
- `src/game/battle/validation/state.rs`: HP 변화 validation.
- `src/game/battle/validation/deaths.rs`: death validation과 `HpChanged` 의존.
- `docs/core_policy_decisions_2026_06.md`: 새 피해 피드백 정책.
- `F:\unity projects\ark\docs\unity_core_contract.md`: Unity-facing event 계약 canonical 문서.

필요하면 `../game_server`의 event mapping을 확인한다.

## Objective

완료 후 다음이 가능해야 한다.

1. `HpChanged` event에 `feedback_tags`가 포함된다.
2. `feedback_tags`는 core가 계산한다.
3. Unity는 방어력/마법 저항/치명타 여부를 재계산하지 않아도 된다.
4. `Physical`/`Magic`/`True` 피해 타입 정보와 `feedback_tags`가 함께 내려간다.
5. 기존 timeline validation이 새 필드를 무시하지 않고 계약을 검증할 수 있다.

## In Scope

- `DamageFeedbackTag` 또는 동등한 typed enum 추가.
- `DamageResult` 또는 `HpChanged` 생성 단계에서 `feedback_tags` 계산.
- 초기 태그 구현:
  - `critical`
  - `mitigated`
  - `fixed_damage`
  - `immune`
  - `piercing`
  - `shield`
  - `blocked`
  - `resisted_status`
- 현재 런타임에 실제 의미가 없는 태그는 계산하지 않고 schema만 future-proof로 둘지 판단한다.
- `weakness`는 초기 상단 태그 계약에서 제외한다.
- canonical `unity_core_contract.md`와 focused tests 갱신.

## Out Of Scope

- 실제 shield/block/status immunity runtime 새 구현.
- AD/AP 밸런스 수치 조정.
- Unity 피해 숫자 UI 구현.
- 무기 아키타입/타겟팅 시스템 도입.

## Implementation Plan

1. 현재 `calculate_damage`가 raw/final/critical/penetration 정보를 어디까지 보존하는지 확인한다.
2. `feedback_tags`를 `DamageResult`에 저장할지, `TimelineEvent::HpChanged` 생성 시 계산할지 비교한다.
3. 가장 중앙화된 지점을 선택해 basic attack과 skill damage 모두 같은 계산을 타게 한다.
4. `mitigated` 기준은 `raw_damage > 0`, `final_damage > 0`, `final_damage <= raw_damage * 60%`로 시작한다.
5. `fixed_damage`는 주 피해 타입이 `True`일 때 붙인다.
6. `immune`은 damage path에서 최종 피해 0을 표현할 수 있는지 확인한다. 현재 minimum damage가 있어 일반 basic attack에서 불가능하면 skill/status path만 먼저 지원한다.
7. `piercing`은 관통 modifier가 실제 resistance를 낮췄을 때만 붙인다. 현재 결과에 adjusted resistance 정보가 없다면 작게 실험하고 기록한다.
8. `HpChanged` serialization과 validation/test를 갱신한다.

## Test Requirements

- critical hit damage event에 `critical` tag가 포함된다.
- raw 대비 final 피해가 60% 이하이면 `mitigated` tag가 포함된다.
- `DamageType::True` 피해 event에 `fixed_damage` tag가 포함된다.
- 적용되지 않은 태그는 빈 배열 또는 누락 없는 안정된 shape로 내려간다.
- basic attack과 skill damage 모두 `feedback_tags`를 가진다.
- legacy expectation이 `HpChanged` shape를 고정하고 있으면 최신 계약으로 교체한다.

검증 후보:

```text
cargo test -p game_core damage -- --nocapture
cargo test -p game_core battle -- --nocapture
cargo test -p game_core -- --nocapture
cargo check -p game_core
```

## Completion Conditions

- `HpChanged.feedback_tags`가 typed contract로 구현된다.
- focused tests가 새 피해 피드백 계약을 검증한다.
- canonical `F:\unity projects\ark\docs\unity_core_contract.md`와 `docs/core_policy_decisions_2026_06.md`가 구현과 일치한다.
- 레거시 피해 event shape를 고정하는 테스트는 최신 정책으로 교체되었다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- `immune`, `shield`, `blocked`, `resisted_status`의 의미가 현재 runtime에 없어 dummy tag로만 남아야 하는지 정책 결정이 필요하다.
- `feedback_tags` 계산 위치가 basic attack과 skill damage에서 크게 갈라져 중앙화 설계가 필요하다.
- Unity-facing JSON casing을 바꿔야 하는 상황이 발견된다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
