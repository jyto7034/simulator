# Codex Goal Command

이 문서는 새 goal을 Codex에게 맡길 때 붙여 넣는 표준 명령어다.

아래 블록의 `<goal_name>`, `<goal_document_path>`, `<objective>`만 해당 작업에 맞게 바꿔 사용한다.

```text
/goal <objective>

목표 문서:
- <goal_document_path>

Goal working directory:
- docs/goals/<goal_name>/

시작 시 반드시 아래 작업 기억장치를 만든 뒤 계속 갱신하라.
- docs/goals/<goal_name>/PLAN.md
- docs/goals/<goal_name>/EXPERIMENTS.md
- docs/goals/<goal_name>/EXPERIMENT_NOTES.md

PLAN.md에는 계획, 범위, 완료 조건, 중단 조건, 검증 명령을 기록하라.
EXPERIMENTS.md에는 시도/실패/성공 결과, 테스트 실패 원인, 수정 내용, 재검증 결과를 기록하라.
EXPERIMENT_NOTES.md에는 작업 중 판단, source-of-truth 충돌, 정책 질문, 후속 후보를 기록하라.

코드 수정은 장기적인 방향으로 하라.
임시방편, 최소한의 수정, compatibility layer, fallback path, dual schema, 특정 테스트만 통과시키는 패치를 기본적으로 피하라.
정말 필요하면 이유, 제거 조건, 테스트 범위를 PLAN.md에 기록하고 사용자 확인을 받아라.

처음에는 어떤 구조가 장기적인 방향인지 판단하기 어려울 수 있다.
작은 trial and error를 통해 확인하고, 실패한 접근과 이유를 EXPERIMENTS.md에 남겨 같은 실수를 반복하지 마라.

source of truth는 아래 순서로 확인하라.
1. 실제 runtime code
2. live RON/data
3. Unity-facing snapshot/command 계약
4. 최신 정책 문서

문서를 무조건 신뢰하지 마라.
실제 코드와 live data를 읽으면서 문서보다 더 나은 개선안이 보이면 근거를 EXPERIMENT_NOTES.md에 기록하고 적용하라.
단, 정책 판단이 필요한 경우 임의로 정하지 말고 goal을 종료한 뒤 사용자에게 질문 목록을 보고하라.

레거시는 과감하게 제거하라.
레거시 동작을 살리기 위한 compatibility layer, fallback path, dual schema는 기본적으로 만들지 마라.
새 정책과 충돌하는 테스트는 기대값만 바꾸지 말고, 그 테스트가 보호하던 사용자-visible behavior를 확인한 뒤 삭제하거나 최신 정책 테스트로 교체하라.
ignored test로 레거시를 보존하지 마라.

테스트는 내부 구현 모양보다 아래 계약을 고정하라.
- 사용자-visible behavior
- Unity-facing DTO
- data validation
- live RON loading
- 실제 gameplay flow

작은 변경 단위마다 빠른 focused test 또는 cargo check를 돌리고, 마지막에는 넓은 테스트를 돌려라.
테스트 실패는 EXPERIMENTS.md에 실패 원인, 수정 내용, 재검증 결과를 기록하라.

goal 범위를 벗어난 개선안은 EXPERIMENT_NOTES.md에 후속 후보로 기록하라.
현재 goal 완료에 필수이거나 blast radius를 줄이는 경우에만 적용하라.

Unity-facing 계약/구현 문서의 canonical 위치는 이 저장소가 아니라 아래 외부 Unity 프로젝트다.
- F:\unity projects\ark\docs
- /mnt/f/unity projects/ark/docs

특히 아래 문서는 외부 canonical 위치를 기준으로 확인하고 갱신하라.
- F:\unity projects\ark\docs\unity_core_contract.md
- F:\unity projects\ark\docs\unity_client_implementation_goal.md

이 저장소 안의 docs/unity_core_contract.md, docs/unity_client_implementation_goal.md는 stale copy일 수 있다.

다음 항목은 사용자 결정이 필요한 정책으로 간주하고 임의로 확정하지 마라.
- Unity-facing DTO shape 변경
- live RON schema 변경
- 저장 데이터 migration
- UX 의미 변화
- 밸런스 기준
- 기존 콘텐츠 삭제/대체
- 실패/보상/소비 시점 변화
- 게임 룰, 노드 흐름, 전투 성공/실패 판정

git/file safety:
- git restore, git reset 등 git file modifier 명령은 수행하지 않는다.
- git의 과거 내역 코드를 현재 파일에 cp하지 않는다.
- 파일을 과거 상태로 돌려야 한다고 판단하면 이유를 설명하고 사용자 허락을 먼저 구한다.
- worktree가 dirty여도 사용자 변경으로 간주하고 되돌리지 않는다.

완료 시 아래 내용을 보고하라.
- 변경 요약
- 제거한 레거시
- 새로 고정한 계약
- 갱신한 테스트
- 남은 위험
- 실행한 검증 명령
- 사용자와 의논해야 할 정책 질문이 있었는지

완료 조건에는 반드시 다음 문장을 포함하라.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
```

## Goal Creation Checklist

새 goal 문서를 작성할 때는 위 명령어와 함께 아래 항목을 구체화한다.

- objective: 한 문장으로 끝나는 구체적 목표.
- source of truth: 먼저 읽을 runtime/data/DTO/docs 경로.
- in scope: 이번 goal에서 실제로 바꿀 범위.
- out of scope: 이번 goal에서 손대지 않을 범위.
- completion conditions: 완료 여부를 판단할 체크리스트.
- stop conditions: 사용자와 의논해야 해서 멈출 조건.
- verification commands: focused test, `cargo check`, 필요 시 broader test.

## Recommended Goal Skeleton

```text
# <Title> Goal

## Objective

## Goal Mode Working Method

## Engineering Principles

## Source Of Truth

## In Scope

## Out Of Scope

## Implementation Plan

## Test Requirements

## Completion Conditions

- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
```
