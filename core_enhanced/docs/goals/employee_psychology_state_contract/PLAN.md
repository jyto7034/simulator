# Employee Psychology State Contract

Status: complete

Implementation complete: true

Created: 2026-07-18

Last updated: 2026-07-18

Goal type: `employee_domain_implementation_master` Phase 4 Child Goal

## 목표

직원의 authored personality는 Phase 1 Profile에만 두고, run-local trust score/band와 의미 기반 memory를 하나의 Psychology aggregate로 complete cutover한다. UUID에서 personality를 추론하거나 opaque cue를 중복 저장하지 않으며 checkpoint, DTO/server/Unity와 canonical 문서가 같은 계약을 사용하게 한다.

## 선행 계약

- Phase 1 `employee_definition_profile_contract`: complete
- Phase 2 `employee_recruitment_queue_ownership`: complete
- Phase 3 `employee_condition_lifecycle_migration`: complete
- Phase 3 review: `docs/goals/employee_condition_lifecycle_migration/COMPLETION_REVIEW.md`
- Master: `docs/goals/employee_domain_implementation_master/PLAN.md`

## 대상 저장소와 경로

- Core employee trust/psychology, world/battle/reward/checkpoint/DTO/tests/canonical: `/mnt/f/work/simulator/core_enhanced`
- game_server transport/fixtures/tests: `/mnt/f/work/simulator/game_server`
- Unity roster/event/result/presentation consumers/tests: `/mnt/f/unity projects/ark`
- live policy/data가 실제로 trust를 authoring하면 `/mnt/f/work/simulator/game_resources`

## 목표 Source of Truth

- authored personality: `EmployeeDefinition.profile.personality`
- run-local Psychology: employee roster instance의 trust score와 semantic merged memories
- trust band/eligibility/display state: Psychology에서 파생
- mutation: typed trust event resolver 하나
- checkpoint/retreat: 기존 roster/entry snapshot clone
- public projection: typed Core DTO; server transport; Unity consumer

## 승인된 불변식

- personality를 UUID, trust score 또는 memory에서 생성하지 않는다.
- trust score는 run-local이며 승인 범위로 clamp한다.
- trust band는 score에서 파생하고 mutable 중복 저장하지 않는다.
- memory는 의미 identity로 merge하며 동일 의미 cue를 무한히 append하지 않는다.
- condition/death/recruitment identity는 선행 Phase source를 그대로 사용한다.
- high-risk refusal, combat/Event modifier, dialogue/reaction 콘텐츠는 보류한다.
- compatibility, fallback, alias, dual read/write를 허용하지 않는다.

## 구현 범위

1. current trust/personality/memory mutation과 DTO/Unity/checkpoint consumer를 최신 tree에서 조사한다.
2. Psychology aggregate, score bounds, derived band와 semantic memory identity/merge를 고정한다.
3. UUID-derived personality와 trust 내부 personality authority를 제거한다.
4. typed trust event mutation이 condition/battle/support lifecycle과 원자적으로 합성되게 한다.
5. checkpoint, retreat, recruitment/new employee 초기화와 run carry-forward를 검증한다.
6. Core DTO/server/Unity reader/presenter/fixture/canonical을 complete cutover한다.
7. focused/broad tests, legacy 검색과 completion review를 수행한다.

## 명시적 비범위

- 새 high-risk/refusal gameplay
- trust 기반 전투/Event 수치 modifier
- 새 dialogue/reaction content
- account memorial/legacy
- personality/profile 재authoring

## DTO Migration

완료. roster의 구형 `trust { score, band, memories, recent_reactions }`를 `psychology { trust_score, trust_band, memories, recent_events }`로 complete cutover했다. memory는 typed subject/source, occurrence, intensity, first/last source와 optional expiry를 투영한다. Unity는 필수 shape와 band 일치를 strict 검증하고 legacy `trust`를 거부한다.

## Data 및 Live RON Migration

production trust authoring은 없었다. dormant 정책을 live data로 승격하지 않았고 RON migration은 수행하지 않았다.

## Save 및 Checkpoint Migration

완료. Psychology는 roster instance가 소유하고 checkpoint와 battle-entry snapshot의 기존 exact roster clone으로 복원된다. 사망 asset recovery는 roster record를 유지하므로 Psychology도 보존된다. 외부 durable save는 발견되지 않았다.

## 필수 Focused 검증

- score clamp와 derived band 경계
- authored Profile personality가 UUID와 무관하게 보존됨
- semantic memory merge/dedupe, intensity/expiry update
- typed event one-shot/idempotence가 필요한 lifecycle
- recruitment 초기 Psychology와 roster ownership
- battle result commit/retreat rollback
- Rest/checkpoint/Floor carry-forward
- exact DTO/server/Unity projection과 legacy rejection

## 필수 Broad 검증

- Core `cargo test --all-targets`
- game_server `cargo test --all-targets`
- Unity full EditMode와 ordinary PlayMode
- relevant Rust fixture exact match
- formatter, diff check와 legacy symbol/schema/mock/document search

## 생략할 검증과 근거

- live RON migration: production Psychology authoring이 없어 생략했다.
- 전체 explicit PlayMode live-server 4개: 외부 server가 필요한 별도 통합 흐름이다. 같은 최종 Unity 코드의 ordinary PlayMode 57개를 실행했다.

## 중단 조건

- 승인되지 않은 refusal/modifier/dialogue 수치가 완료에 필수임
- 외부 durable save migration이 필수임
- 선행 Profile/condition/recruitment source를 깨지 않고 Psychology를 분리할 수 없음
- dirty 사용자 변경과 안전하게 병합할 수 없음

## 완료 조건

완료. authored personality와 run-local Psychology 책임이 분리되고, trust score/band/memory mutation·rollback·projection이 하나의 계약을 사용하며 legacy/fallback과 dormant public surface가 제거됐다.
