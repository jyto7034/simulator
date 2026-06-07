# Combat Preview Threat Warnings Goal

이 goal은 전투 노드 미리보기에서 세부 적 스탯 숫자 대신 위협 경고를 제공하고, 재진입 시 false rumor를 취소선 처리할 수 있도록 `CombatPreview.threat_warnings` 계약을 추가하는 작업이다.

관련 정책은 `docs/core_policy_decisions_2026_06.md`의 `노드 미리보기와 위협 경고`를 따른다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/combat_preview_threat_warnings/PLAN.md
docs/goals/combat_preview_threat_warnings/EXPERIMENTS.md
docs/goals/combat_preview_threat_warnings/EXPERIMENT_NOTES.md
```

## Engineering Principles

- 코드 수정은 장기적인 방향으로 한다.
- 처음 구조 판단이 어려우면 작은 trial and error를 수행하고 결과를 기록한다.
- 레거시 `threat_warning_tags` 같은 단순 배열이 발목을 잡으면 compatibility layer보다 새 typed DTO로 교체한다.
- 문서를 무조건 신뢰하지 않고 실제 `CombatPreview` 생성 경로와 live RON을 읽는다.
- 테스트가 과거 preview shape를 고정하고 있으면 최신 정책을 검증하는 테스트로 교체한다.
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

- `src/game/combat_preview.rs`: `CombatPreview` DTO.
- `src/game/combat_preview/template.rs`: template 기반 preview 생성.
- `src/game/combat_preview/validation.rs`: preview validation.
- `src/game/world/map_encounters.rs`: node preview/encounter 연결.
- `src/game/world/node_flow.rs`: node confirm 흐름.
- `src/game/behavior.rs`: `NodePreview` result.
- `src/game/world/snapshot.rs`: node_confirm/in_battle snapshot.
- `src/game/data/pve_data.rs`: encounter metadata.
- `src/game/data/corroded_wave_data.rs`: generated enemy wave data.
- `docs/core_policy_decisions_2026_06.md`.
- `F:\unity projects\ark\docs\unity_core_contract.md`.

필요하면 `../game_resources/data/**`의 encounter/wave RON을 확인한다.

## Objective

완료 후 다음이 가능해야 한다.

1. `CombatPreview`가 `threat_warnings` 객체 배열을 내려준다.
2. 각 warning은 `tag`, `status`, `source`를 가진다.
3. `status`는 `Unverified`, `Disproved`다.
4. `source`는 `Briefing`, `Rumor`다.
5. 최초 preview에 낮은 확률의 seed 기반 `Rumor` 오경고를 섞을 수 있다.
6. Unity는 warning을 재계산하지 않고 core가 내려준 항목을 표시한다.

## In Scope

- `ThreatWarning`, `ThreatWarningTag`, `ThreatWarningStatus`, `ThreatWarningSource` typed DTO 추가.
- `CombatPreview`에 `threat_warnings` 추가.
- 기존 단순 warning tag가 있다면 교체.
- 초기 tag 후보 구현:
  - `armored_enemy_possible`
  - `high_magic_resist_enemy_possible`
  - `air_enemy_possible`
  - `hard_to_block_enemy_possible`
  - `shielded_enemy_possible`
  - `regenerating_enemy_possible`
  - `fast_breakthrough_enemy_possible`
- 낮은 확률의 `Rumor` 오경고 후보 추가.
- seed 기반 재현성 보장.
- Unity 계약 문서와 tests 갱신.

## Out Of Scope

- false rumor의 재진입 `Disproved` lifecycle 구현. 이 부분은 별도 false-rumor lifecycle goal에서 처리한다.
- 적 외형/아이콘 UI 구현.
- 세부 적 스탯 도감.
- AD/AP 밸런스 validation.

## Implementation Plan

1. `CombatPreview` 생성 위치와 serialization shape를 확인한다.
2. preview가 encounter metadata, generated wave, concrete enemy entries 중 어느 데이터를 가지고 있는지 확인한다.
3. 수동 briefing warning source를 둘 위치를 정한다. 이미 encounter metadata가 적합하면 확장하고, 없으면 가장 단순한 data field를 추가한다.
4. `ThreatWarning` typed DTO를 만든다.
5. 최초 preview 생성 시 `Briefing` 경고를 채운다.
6. `Rumor` 오경고는 낮은 확률, 최대 1개, seed 기반으로 추가한다.
7. 같은 tag가 중복되면 실제 `Briefing`을 우선하고 `Rumor`는 제거한다.
8. warning 사전은 코드 table 또는 RON 중 어느 쪽이 장기적으로 맞는지 코드 읽고 판단한다. 정책 판단이 필요하면 goal을 종료한다.
9. 테스트와 문서를 갱신한다.

## Test Requirements

- `CombatPreview` JSON에 `threat_warnings`가 포함된다.
- 기본 warning은 `status: Unverified`, `source: Briefing`으로 내려간다.
- seed가 같으면 `Rumor` 오경고 선택이 재현된다.
- `Rumor`는 최대 1개만 추가된다.
- 실제 warning과 같은 tag의 rumor가 중복되지 않는다.
- Unity-facing contract test가 `threat_warning_tags`가 아니라 `threat_warnings`를 검증한다.

검증 후보:

```text
cargo test -p game_core combat_preview -- --nocapture
cargo test -p game_core node_preview -- --nocapture
cargo test -p game_core ron_loading -- --nocapture
cargo check -p game_core
```

## Completion Conditions

- `CombatPreview.threat_warnings`가 typed DTO로 구현된다.
- `threat_warning_tags` 레거시 shape가 남아 있다면 제거되거나 공식 계약에서 제외된다.
- seed 기반 rumor warning 테스트가 있다.
- canonical `F:\unity projects\ark\docs\unity_core_contract.md`, `F:\unity projects\ark\docs\unity_client_implementation_goal.md`가 구현과 일치한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- warning source를 encounter RON에 둘지 generated wave에 둘지 정책 결정이 필요하다.
- rumor 확률 또는 false information 빈도 수치를 확정해야 하는데 코드 근거만으로 정하기 어렵다.
- false rumor의 `Disproved` 전환은 attempt/re-entry state가 필요하므로 이 goal 범위를 넘는다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
