# Skill System Refactor Plan

## Objective

`docs/skill_system_refactor_goal.md`를 기준으로 스킬 시스템을 DefenseRoute tile-based 계약에 맞춘다.

핵심 목표:

- `SkillStep.defense_tile_range`를 DefenseRoute 스킬의 표시/시전/타겟/피격 범위 단일 source of truth로 만든다.
- geometric `Area(shape: Circle/Line/Box/Rectangle/Cone)`가 DefenseRoute 공식 live skill 판정을 고정하지 않게 정리한다.
- skill id 참조와 active fragment skill range 계약을 data load 단계에서 검증한다.
- `ModifyDamage`의 step-local 의미를 validation으로 고정한다.
- Unity가 수동 스킬 UI를 추론 없이 만들 수 있도록 skill catalog + snapshot readiness 계약을 보강한다.
- buff registry를 RON 기반으로 옮긴다.

## Current Strategy

한 번에 전체 스킬 런타임을 교체하지 않는다. 먼저 현재 코드와 live RON의 source of truth를 확인한 뒤, DefenseRoute 공식 경로에 필요한 계약부터 작게 바꾼다.

작업 순서:

1. 현재 schema/runtime/RON 로드 경로 재확인.
2. `defense_tile_range` 기반 범위 대상 수집을 추가하고 geometric area 테스트를 교체.
3. live RON을 geometric area 의존에서 tile range 기반 delivery로 마이그레이션.
4. 데이터 검증 강화.
5. Unity-facing skill metadata/readiness 보강.
6. buff registry RON화.
7. 문서와 검증 정리.

## Active Findings

- 현재 `DeliveryDef::Area`는 실제 피격을 `SkillAreaShapeDef` 연속좌표 shape로 수집한다.
- `TimelineEvent::SkillAreaDeclared`도 `TimelineSkillAreaShape`를 노출하므로 runtime만 바꾸면 Unity 계약이 어긋난다.
- live `../game_resources/data/skills/base.ron`에도 geometric area가 남아 있다.
- 기존 테스트 다수가 Circle/Line/Box/Rectangle/Cone area semantics를 고정한다. 정책 변경에 충돌하면 삭제 또는 최신 tile-range focused test로 교체한다.

## Stop Conditions

- `DeliveryDef` schema 이름/구조가 기존 client/RON과 크게 충돌한다.
- persistent tile area의 anchor/facing/tracking 해석이 코드 근거만으로 결정되지 않는다.
- geometric area가 DefenseRoute 외 live 경로에서 반드시 필요하다는 근거가 발견된다.
- 사용자와 의논하여 정해야 할 정책이 생긴다.
