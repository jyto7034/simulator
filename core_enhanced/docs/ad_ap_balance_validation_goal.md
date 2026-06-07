# AD AP Balance Validation Goal

이 goal은 AD/AP가 클리어 가능 여부를 잠그는 하드 게이트가 되지 않도록 live data validation과 경고 태그 일관성을 추가하는 작업이다.

이 goal은 수치 밸런스를 자동으로 완성하려는 goal이 아니다. 처음에는 명확히 금지할 수 있는 레거시/위험 패턴을 validation하는 데 집중한다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/ad_ap_balance_validation/PLAN.md
docs/goals/ad_ap_balance_validation/EXPERIMENTS.md
docs/goals/ad_ap_balance_validation/EXPERIMENT_NOTES.md
```

## Engineering Principles

- 코드 수정은 장기적인 방향으로 한다.
- 자동 밸런서 같은 큰 추상화를 만들지 않는다.
- 문서를 무조건 신뢰하지 않고 실제 enemy stats, damage formula, encounter data를 읽는다.
- 정책 변경으로 깨지는 테스트가 하드 카운터를 고정하고 있으면 최신 정책으로 교체한다.
- 사용자와 의논하여 정해야 할 정책이 발견되면 goal을 종료한다.

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

- `src/game/battle/damage.rs`: mitigation formula.
- `src/game/battle/types.rs`: unit stats.
- `src/game/data/corroded_employee_data.rs`: generated/corroded enemy stats.
- `src/game/data/pve_data.rs`: encounter metadata.
- `src/game/data/corroded_wave_data.rs`: wave generation.
- `src/game/combat_preview.rs`: threat warnings.
- `src/game/combat_preview/validation.rs`: preview validation.
- `src/game/data/mod.rs`: cross-data validation.
- `tests/ron_loading.rs`: live data validation entry.
- `docs/core_policy_decisions_2026_06.md`.

필요하면 `../game_resources/data/**`의 enemy/encounter/wave RON을 확인한다.

## Objective

완료 후 다음이 가능해야 한다.

1. 일반/정예 enemy data가 특정 피해 타입을 사실상 영구 무효화하지 않는다.
2. 보스의 영구 단일 타입 면역은 validation 또는 명시 예외 정책 없이는 허용되지 않는다.
3. warning data와 enemy composition의 큰 불일치를 잡을 수 있다.
4. AD/AP는 효율 차이 축으로 유지된다.

## In Scope

- enemy stats validation.
- encounter/wave warning consistency validation.
- 명확한 금지 패턴부터 검사:
  - 일반/정예 permanent type immunity.
  - 극단적인 defense/magic_resist 값.
  - warning이 전혀 없는 고방어/고마저 핵심 wave.
- boss 예외는 명시 metadata가 있을 때만 허용.
- docs/tests 갱신.

## Out Of Scope

- 자동 난이도 산출기.
- DPS 시뮬레이터.
- 모든 노드 클리어 가능성 증명.
- 보스 기믹 상세 설계.
- Unity UI 구현.

## Implementation Plan

1. 현재 damage formula에서 resistance 값이 어느 정도면 사실상 무효인지 작은 계산 실험을 한다.
2. live enemy stats와 generated corroded stats 범위를 확인한다.
3. 일반/정예/boss 구분이 data에서 어떻게 표현되는지 확인한다.
4. 처음에는 hard validation보다 warning/fail 기준을 작게 잡는다.
5. `threat_warnings`와 실제 enemy stats의 큰 불일치를 validation한다.
6. false positive가 많으면 `EXPERIMENTS.md`에 기록하고 기준을 조정한다.
7. 정책 수치가 필요하면 goal을 종료하고 질문한다.

## Test Requirements

- 일반 enemy가 physical permanent immunity를 가지면 validation 실패.
- 일반 enemy가 magic permanent immunity를 가지면 validation 실패.
- boss immunity는 명시 예외 metadata 없으면 validation 실패.
- high armor enemy가 있는데 `armored_enemy_possible` warning이 누락된 encounter는 validation warning 또는 failure를 낸다.
- high magic resist enemy가 있는데 `high_magic_resist_enemy_possible` warning이 누락된 encounter는 validation warning 또는 failure를 낸다.
- 정상 live RON은 validation을 통과한다.

검증 후보:

```text
cargo test -p game_core ron_loading -- --nocapture
cargo test -p game_core data -- --nocapture
cargo test -p game_core combat_preview -- --nocapture
cargo check -p game_core
```

## Completion Conditions

- AD/AP 하드 카운터 금지 정책의 최소 validation이 구현된다.
- warning consistency validation 또는 audit가 구현된다.
- live RON이 최신 validation을 통과한다.
- 자동 밸런스 판정이 아닌 명확한 금지 패턴 중심으로 유지된다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- “사실상 무효” 수치를 코드 근거만으로 정하기 어렵다.
- boss 예외 metadata shape가 정책 결정을 요구한다.
- validation이 실제 live data에서 너무 많은 false positive를 만든다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
