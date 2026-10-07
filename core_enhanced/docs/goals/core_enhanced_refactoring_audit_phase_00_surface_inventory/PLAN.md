# Core Enhanced Refactoring Audit Phase 00 — Surface Inventory Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: phase
Parent Master: `core_enhanced_refactoring_audit_master`
Phase: 0
Execution Mode: investigation_only
Production Mutation Authorized: no
Audit Baseline Date: 2026-08-07
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

현재 working tree의 `core_enhanced` 전체 surface, 실제 source-of-truth와 consumer/data seam을
누락 없이 inventory하고, 파일 크기나 폴더 모양이 아니라 독립 authority 경계를 기준으로 후속
read-only 후보 감사 Phase 분할안을 만든다.

## User/Contract-Visible Problem

Master 감사의 후속 Phase를 실제 ownership 경계보다 디렉터리 모양으로 먼저 나누면 동일 계약을
여러 번 감사하거나 authority seam을 누락할 수 있다. 전체 surface와 canonical/data/test/consumer
경계를 먼저 식별해야 이후 후보 catalog의 coverage와 우선순위가 신뢰할 수 있다.

## Audit Finding ID

없음. Parent Master의 CRA-01~CRA-12를 위한 inventory Phase다.

## Parent Scope And Direct Dependency

- Master: [PLAN.md](../core_enhanced_refactoring_audit_master/PLAN.md)
- Direct predecessor: 없음. Master initialization이 유일한 dependency다.
- Activation evidence: 2026-08-07 사용자의 Master `PLAN.md` 수행 요청을, Master에 기록된 제안
  Phase 0 이름과 investigation-only 범위의 실행 승인으로 적용했다.
- 다음 Phase는 이 Phase가 `Complete / High`를 받고 사용자가 boundary 분할·이름을 승인하기 전에는
  만들거나 활성화하지 않는다.

## Approved Audit Requirements

| ID | 이 Phase의 책임 |
| --- | --- |
| CRA-01 | 현재 working-tree `core_enhanced`를 primary inventory 대상으로 고정한다. |
| CRA-02 | 현재 `engineering_workflow.md#리팩토링-판단`의 판정 축과 필수 evidence seam을 inventory에 반영한다. |
| CRA-03 | runtime/live data/가장 가까운 durable canonical 순서로 authority 경계를 찾는다. |
| CRA-04 | 작성 도구, 문체, 파일 길이와 취향을 boundary 또는 후보 evidence로 사용하지 않는다. |
| CRA-05 | 후속 후보가 갖춰야 할 authority/producer/consumer/test evidence를 어느 Phase가 수집할지 정의한다. |
| CRA-06 | priority evidence와 readiness safety-net surface를 각각 inventory한다. |
| CRA-07 | behavior-preserving, correction, migration을 분리할 수 있는 contract seam을 표시한다. |
| CRA-08 | supporting data/consumer를 confirmed/unverified/out-of-scope로 분류한다. |
| CRA-09 | 독립 source-of-truth 경계와 cross-boundary seam을 제안한다. |
| CRA-10 | 조사와 Goal 문서 기록만 수행하고 production을 수정하지 않는다. |
| CRA-11 | 초기 dirty/untracked 변경을 사용자 소유로 보존한다. |
| CRA-12 | 후속 Phase가 priority/deferred/rejected/unverified disposition을 모두 만들 수 있도록 coverage를 정의한다. |

## Current Runtime And Live Data Evidence

Phase 초기화 시점에는 discovery pending이다. Master 초기화 기록 외에 runtime module, live loader,
producer/consumer 또는 test coverage를 검증한 것으로 주장하지 않는다.

## Source Of Truth And Ownership

Evidence precedence는 Parent Master와 [Docs Index](../../README.md#source-of-truth-순서)를 따른다.

1. `core_enhanced` runtime code와 authoritative state mutation
2. 해당 runtime이 실제 읽는 production live data
3. 실제 DTO/serialization producer와 접근 가능한 direct consumer seam
4. README가 지정하는 가장 가까운 durable canonical
5. focused/integration test
6. Goal, experiment와 archive는 supporting history

`engineering_workflow.md`는 감사 판정 방법을 소유하지만 gameplay/runtime/data 계약을 소유하지 않는다.

## Invariants

- production code, test, live data, canonical과 external consumer를 수정하지 않는다.
- read-only inventory 결과를 곧바로 결함 또는 refactor candidate verdict로 확대하지 않는다.
- 모든 surface는 후속 boundary Phase, cross-boundary final pass, explicit out-of-scope 또는
  unverified 중 하나로 disposition한다.
- 동일 authority가 여러 디렉터리를 가로지르면 하나의 경계로 추적하고, 독립 계약을 외형이
  비슷하다는 이유로 합치지 않는다.
- 초기 dirty/untracked 파일을 이동, 삭제, restore, format 또는 정리하지 않는다.
- Unity는 사용자 승인 비범위이며 읽거나 검증하지 않는다.
- `game_resources`와 `game_server`는 Core authority/data/consumer seam 확인에 직접 필요한 범위만 읽는다.

## Refactor Candidate And Behavior-Preservation Boundary

이 Phase는 후보를 확정하거나 점수화하지 않는다. 후보처럼 보이는 신호가 발견되면 실제
authority→producer→consumer 경로의 심층 확인을 후속 boundary Phase에 배정하거나, 경계를 확정할
수 없는 경우 `unverified` inventory note로 남긴다.

## Open Policy Decisions

- Gameplay/runtime/data/wire 정책 결정은 현재 없음.
- Phase 0 결과로 제안할 후속 Phase 수, 이름, 범위와 순서는 사용자 승인이 필요하다.

## In Scope

- `core_enhanced/Cargo.toml`, workspace/package/dependency와 crate entry surface
- `core_enhanced/src/**` module, public API, command/input, authoritative mutation과 projection surface
- `core_enhanced/tests/**` 및 `src/**/tests/**`의 contract/safety-net surface
- `core_enhanced/docs/**`의 index, current canonical, workflow, goal/archive/tmp 분류
- Core가 실제 소비하는 live data loader와 `game_resources` production path seam의 필요한 read-only 확인
- Core DTO/command/result를 직접 전달하는 `game_server` seam의 필요한 read-only 확인
- 독립 source-of-truth boundary와 cross-boundary seam, 후속 Phase 분할·순서 제안

## Out Of Scope

- 모든 production refactor, cleanup, correction, migration과 formatter
- candidate의 최종 work type, 7축 점수, readiness 또는 disposition 판정
- `game_resources`나 `game_server` 자체의 독립 리팩토링 감사
- Unity repository/runtime/contract/UI/test
- gameplay, UX, balance, DTO/RON/save/compatibility 신규 정책
- 완료 Goal의 correction 또는 historical document를 current policy로 승격하는 작업
- test/probe 실행으로 behavior를 새로 입증하는 작업. Inventory에 필수인 read-only source 확인만 한다.

## Investigation Scope

1. 현재 dirty baseline과 in-scope path classification을 기록한다.
2. crate entrypoint, top-level module, command/input와 authoritative state owner surface를 inventory한다.
3. runtime이 읽는 live data loader/validation과 production file family seam을 inventory한다.
4. DTO, serialization, projection과 direct server consumer seam을 inventory한다.
5. README가 지정한 canonical owner와 goal/archive/tmp의 비권위 경계를 inventory한다.
6. focused/integration/broad test surface와 명백한 coverage owner를 inventory한다.
7. surface마다 제안 boundary, cross-boundary final, explicit out-of-scope 또는 unverified disposition을 부여한다.
8. 독립 authority 경계에 근거한 후속 Phase 수, 이름, dependency와 조사 순서를 제안한다.
9. 별도 완료 감사에서 coverage와 no-production-mutation을 재검증한다.

## DTO/Data/Save Migration

모두 `not_applicable`이다. 이 Phase는 shape, content, persistence를 변경하지 않는다.

## Deployment And Compatibility

`not_applicable`. Producer/consumer version 또는 compatibility seam은 inventory할 수 있지만 결정하거나
변경하지 않는다.

## Focused Validation

- file/module inventory count와 classification의 재현 가능한 read-only 명령
- crate entrypoint와 module export 연결 확인
- live loader가 참조하는 production path family 확인
- DTO/producer와 direct server seam의 symbol/path 연결 확인
- canonical index의 owner mapping 확인
- test target/module surface와 inventory mapping 확인

정확한 명령과 결과 수치는 조사 중 `EXPERIMENTS.md`에 기록한다.

## Broad Validation

- `core_enhanced` in-scope path 전체가 inventory disposition을 갖는지 독립 재검토
- 제안 boundary 사이 coverage gap, duplicate ownership과 cross-boundary seam 재검토
- initial/final scoped status와 production no-mutation 비교

## Omitted Validation

- `cargo test`, `cargo check`, runtime probe: 이 Phase는 behavior correctness를 주장하지 않고 surface와
  authority seam만 조사하므로 실행하지 않는다. 후속 candidate Phase가 필요한 characterization/focused
  runner를 정한다.
- Unity: 사용자 승인 비범위다.

## Completion Conditions

- 모든 in-scope `core_enhanced` source/test/doc/manifest surface가 inventory에 포함된다.
- crate/runtime authority, live-data, DTO/server, canonical과 test safety-net seam이 구분된다.
- supporting data/consumer는 confirmed/unverified/out-of-scope 중 하나로 분류된다.
- 모든 surface가 후속 boundary Phase, final cross-boundary pass, explicit out-of-scope 또는 unverified
  disposition을 갖는다.
- 후속 Phase 제안은 독립 source-of-truth 경계와 dependency에 근거하고 폴더 크기만으로 나뉘지 않는다.
- production mutation 없이 Goal 문서와 감사 산출물만 갱신된다.
- Phase completion audit가 `Complete / High`다.
- 다음 Phase는 사용자 분할·이름 승인 전 활성화되지 않는다.

## Completion Audit Evidence Matrix

| ID | Completion condition or invariant | Required evidence | Status |
| --- | --- | --- | --- |
| P0-01 | 전체 source surface coverage | source/module inventory와 path count reconciliation | pass |
| P0-02 | 전체 test surface coverage | test target/module inventory와 classification | pass |
| P0-03 | docs/manifest surface coverage | canonical/noncanonical/goal/archive/tmp inventory | pass |
| P0-04 | runtime authority boundary map | entrypoint/state owner/projection evidence | pass |
| P0-05 | live data seam map | loader/validation/production path evidence | pass |
| P0-06 | DTO/direct consumer seam map | producer type와 server supporting evidence | pass |
| P0-07 | complete disposition | surface→boundary/out-of-scope/unverified matrix | pass |
| P0-08 | executable Phase proposal | boundary, dependency, order와 completion output | pass |
| P0-09 | dirty preservation/no mutation | initial/final scoped status and diff comparison | pass |
| P0-10 | completion audit verdict | `COMPLETION_AUDIT.md` Complete / High | pass |

## Stop Conditions

- 실제 inventory가 Parent Master의 승인 범위를 material하게 넓혀야 함
- 사용자 정책 또는 외부 shape 결정 없이는 authority boundary를 편향 없이 나눌 수 없음
- 필요한 supporting data/server seam에 접근할 수 없어 boundary를 확정할 수 없음
- read-only 명령이 production state나 사용자 파일을 변경할 위험이 있음
- current `engineering_workflow.md` 판정 기준이 외부 작업으로 바뀜
- 초기 dirty 변경과 Phase 감사 산출물을 안전하게 구분할 수 없음
- 후속 Phase activation: Phase 0 `Complete / High`와 사용자 승인 전에는 중단
