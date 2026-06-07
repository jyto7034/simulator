# Post Policy Technical Debt Review Goal

이 goal은 2026-06 core policy implementation 이후 추가/수정된 코드가 장기적으로 유지 가능한 구조인지 검토하는 audit 작업이다.

이 goal의 기본 산출물은 코드 수정이 아니라 기술부채 보고서다. 명백히 작은 문서 오탈자나 기록 누락을 제외하고, runtime/core/server 코드 수정은 사용자 승인 전에는 수행하지 않는다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/post_policy_technical_debt_review/PLAN.md
docs/goals/post_policy_technical_debt_review/EXPERIMENTS.md
docs/goals/post_policy_technical_debt_review/EXPERIMENT_NOTES.md
docs/goals/post_policy_technical_debt_review/REPORT.md
```

## Engineering Principles

- 이 goal은 review/audit goal이다. 바로 코드 수정에 들어가지 않는다.
- 문서를 무조건 신뢰하지 않고 실제 runtime code, live RON/data, Unity-facing snapshot/command 계약을 읽는다.
- 장기적인 방향을 기준으로 평가한다. 임시방편, compatibility layer, hidden fallback, dual source of truth를 부채로 본다.
- 레거시를 과감히 제거해야 하는 후보는 보고서에 기록하되, 이 goal 안에서 제거하지 않는다.
- 테스트가 예전 정책을 고정하는지, 최신 사용자-visible behavior를 고정하는지 분리해서 판단한다.
- 사용자와 의논하여 정해야 할 정책이 발견되면 goal을 종료하고 질문 목록을 보고한다.

## Goal Execution Contract

- source of truth 우선순위는 실제 runtime code, live RON/data, Unity-facing snapshot/command 계약, 최신 정책 문서 순서로 본다.
- 문서와 코드가 충돌하면 코드를 우선하고, 충돌 내용을 `EXPERIMENT_NOTES.md`와 `REPORT.md`에 기록한다.
- 이 goal은 기본적으로 코드 변경을 하지 않는다. 변경이 필요하면 `REPORT.md`에 권장 수정 goal을 분리해서 제안한다.
- 단순 format, lint, 테스트 실행 결과도 수정하지 말고 관찰 결과로 기록한다.
- 위험도는 `Critical`, `High`, `Medium`, `Low`, `Follow-up` 중 하나로 분류한다.
- 각 finding은 파일/함수/데이터 경로, 왜 부채인지, 사용자-visible 위험, 권장 해결 방향, 필요한 정책 결정 여부를 포함한다.
- 발견한 부채가 현재 master goal 완료 상태를 무효화한다고 판단되면 즉시 사용자에게 보고한다.
- 완료 시 “바로 고칠 항목”, “별도 goal로 분리할 항목”, “유보할 항목”, “정책 논의 필요 항목”을 분리해서 보고한다.

## Git And File Safety

- `git restore`, `git reset` 등 git file modifier 명령은 수행하지 않는다.
- git의 과거 내역 코드를 복사하여 현재 파일에 `cp`하지 않는다. 만약 해야 한다면 사용자의 허락을 무조건 받는다.
- 파일을 과거로 돌리는 경우 이유를 설명하고 사용자의 허락을 무조건 구한다.
- worktree가 dirty여도 사용자 변경으로 간주하고 되돌리지 않는다.

## Canonical Unity Docs

Unity-facing 계약/구현 문서의 canonical 위치는 `F:\unity projects\ark\docs`다. Linux/WSL 경로로는 `/mnt/f/unity projects/ark/docs`다.

리뷰 중 Unity-facing 계약이 관련되면 아래 외부 canonical 문서를 기준으로 확인한다.

- `F:\unity projects\ark\docs\unity_core_contract.md`
- `F:\unity projects\ark\docs\unity_client_implementation_goal.md`

이 저장소 안의 같은 이름 문서는 stale copy일 수 있다.

## Source Of Truth

우선 읽을 문서:

- `docs/core_policy_decisions_2026_06.md`
- `docs/goals/core_policy_implementation_master/PLAN.md`
- `docs/goals/core_policy_implementation_master/EXPERIMENTS.md`
- `docs/goals/core_policy_implementation_master/EXPERIMENT_NOTES.md`
- 완료된 하위 goal들의 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`

우선 읽을 코드 영역:

- `src/game/battle/damage.rs`
- `src/game/combat_balance.rs`
- `src/game/combat_preview.rs`
- `src/game/data/mod.rs`
- `src/game/battle/core/movement/**`
- `src/game/battle/core/targeting.rs`
- `src/game/battle/core/sim.rs`
- `src/game/battle/core/commands.rs`
- `src/game/combat_player_spawns.rs`
- `src/game/data/equipment_data.rs`
- `src/game/data/skill_fragment_data.rs`
- `src/game/world/helpers.rs`
- `src/game/world/maintenance.rs`
- `src/game/world/snapshot.rs`
- `src/game/behavior.rs`
- `../game_server/src/game/player_game_actor/state.rs`
- `../game_resources/data/**`

## Review Axes

### Source Of Truth Drift

- 같은 정책 값이나 판정이 여러 모듈에서 재계산되는지 확인한다.
- 특히 damage feedback, threat warning, targeting, mobility, fragment compatibility, effective combat profile 경로를 본다.
- `combat_balance`, `effective_combat_profile_for_employee`, `mobility_kind`, `weapon_profile` 같은 새 source of truth가 우회되는 곳이 있는지 확인한다.

### Runtime And Snapshot Alignment

- Unity snapshot이 실제 battle runtime profile과 같은 경로를 사용하는지 확인한다.
- `effective_weapon_profile`, `skill_fragment compatibility`, `mobility_kind`, `air_capable`, `feedback_tags`, `threat_warnings`가 runtime 기준과 일치하는지 확인한다.
- game_server mapping이 typed result/error와 field naming을 약하게 연결하는 부분이 있는지 확인한다.

### Validation Placement And Cost

- `GameDataBase::new`에 들어간 cross-data validation과 preview generation validation이 적절한 위치와 비용인지 확인한다.
- static validation, live RON validation, runtime preview generation 검증의 경계가 섞였는지 확인한다.
- validation이 테스트 데이터 작성 비용을 과도하게 올렸는지 확인한다.

### Movement Backend Boundaries

- Rapier가 다시 gameplay source of truth처럼 쓰이는 곳이 없는지 확인한다.
- board clamp, static obstacle correction, unit overlap, block state, route progress 책임이 분리되어 있는지 확인한다.
- Direct/Rapier 동등성 테스트가 실제 정책-visible behavior를 고정하는지 확인한다.

### Policy Leakage

- Unity가 core 정책을 재계산해야만 하는 DTO 누락이 있는지 확인한다.
- live RON schema와 docs가 실제 serde shape와 맞는지 확인한다.
- policy docs에는 있는데 구현이 아직 없는 내용을 “구현된 것처럼” 보이게 하는 코드/DTO가 있는지 확인한다.

### Test Debt

- 테스트 fixture가 최신 policy source of truth를 우회하는지 확인한다.
- ignored test, 0개 실행 filter, 구현 내부 모양만 고정하는 brittle test를 찾는다.
- full test와 focused test가 서로 다른 계약을 말하지 않는지 확인한다.

## In Scope

- 코드 읽기와 구조 리뷰.
- live RON/data schema 읽기.
- Unity-facing canonical docs와 실제 DTO shape 비교.
- 테스트 구조와 실행 명령 검토.
- 기술부채 finding 작성.
- 후속 개선 goal 후보 작성.

## Out Of Scope

- runtime/core/server 코드 수정.
- live RON 수정.
- Unity 구현 수정.
- 새 정책 설계 확정.
- 대규모 리팩터링 수행.
- 레거시 제거 수행.

## Implementation Plan

1. 작업 기억장치 4개 파일을 만든다.
2. master notes와 완료된 하위 goal notes를 읽고 변경 축을 목록화한다.
3. `git diff --stat`와 관련 코드 검색으로 새로 추가/수정된 주요 영역을 파악한다. git file modifier 명령은 사용하지 않는다.
4. source of truth drift를 먼저 검토한다.
5. runtime/snapshot/game_server alignment를 검토한다.
6. validation 위치와 비용을 검토한다.
7. movement backend와 airborne/targeting 경계를 검토한다.
8. 테스트 debt는 별도 `post_policy_test_verification_goal.md`와 중복되지 않게 구조적 finding만 기록한다.
9. `REPORT.md`에 우선순위별 finding과 추천 후속 goal을 정리한다.
10. 사용자에게 수정 착수 여부를 묻고 goal을 종료한다.

## Report Format

`REPORT.md`는 아래 구조를 따른다.

```text
# Post Policy Technical Debt Review Report

## Executive Summary

## Critical Findings

## High Findings

## Medium Findings

## Low Findings

## Follow-up Candidates

## Policy Questions For User

## Suggested Next Goals

## Verification Commands Run
```

각 finding은 아래 필드를 포함한다.

```text
- Severity:
- Area:
- Files:
- Problem:
- Evidence:
- User-visible Risk:
- Recommended Direction:
- Needs User Policy Decision:
```

## Completion Conditions

- 작업 기억장치가 생성되고 최신 상태로 갱신된다.
- 주요 추가/수정 영역을 실제 코드 기준으로 읽었다.
- 기술부채 finding이 위험도별로 정리된다.
- 바로 수정하지 않고 후속 goal 후보로 분리된다.
- 정책 논의가 필요한 항목이 있으면 질문 목록이 작성된다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- 리뷰 중 master goal 완료 상태를 무효화할 수 있는 Critical 문제가 발견된다.
- 코드 수정 없이는 더 이상 판단할 수 없는 문제가 발견된다.
- 정책 결정 없이는 finding severity나 해결 방향을 정할 수 없다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

