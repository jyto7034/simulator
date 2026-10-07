# Core Enhanced Refactoring Post-Boss Runtime Reaudit Phase 00 Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: phase
Execution Mode: investigation_only
Production Mutation Authorized: no
Master Goal: `../core_enhanced_refactoring_post_boss_runtime_reaudit_master/PLAN.md`
Master Phase: 0 — current dirty baseline, 51-candidate/Boss delta inventory와 최종 boundary 확인
Direct Predecessor: 없음 (`Master initialized`가 유일한 activation dependency)
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
Baseline Date: 2026-08-09

## Objective

현재 dirty working tree를 재현 가능한 문서 evidence로 고정하고, 2026-08-07 최종 51개 후보와 source
candidate, Boss Runtime v1 교차 19개 및 current Boss 변경 surface를 누락 없이 Phase 1~10 경계에 배정해
후속 read-only 재감사의 실행 입력을 확정한다.

## User/Contract-Visible Problem

Start commit만으로는 현재 untracked Boss runtime과 기존 사용자 변경을 재현할 수 없고, 과거 51개 catalog와
Boss 완료 감사의 disposition은 current source proof가 아니다. 정확한 baseline과 identity map 없이 후속
Phase를 시작하면 기존 후보를 누락·중복하거나 Boss-specific subset을 원래 후보 전체 완료로 오판할 수 있다.

이 Phase의 contract-visible 결과는 runtime 동작 변경이 아니라 다음 Phase들이 같은 현재 tree, 같은 51개
identity, 같은 consumer/test boundary를 감사하도록 만드는 durable inventory다.

## Audit Finding ID

없음. Master가 승인한 post-implementation read-only reaudit의 Phase 0이다.

## Current Runtime And Live Data Evidence

초기화 시 확인한 evidence는 다음으로 제한한다.

- repository HEAD는 `f318414fb0156a67e04c5479000c2a2683fc0c09`다.
- current `git status --short`에는 Core source/test/canonical, production live data, server source의 modified
  파일과 Boss/tactical source 및 기존 Goal 문서의 untracked 파일이 함께 존재한다.
- Phase 경로는 초기화 직전 존재하지 않았다.
- runtime reachability, live loader consumption, DTO/consumer, test assertion 의미와 과거 candidate closure는
  아직 `discovery pending`이다.

현재 동작은 Goal 문서가 아니라 current runtime mutation과 production loader가 읽는 live data로 후속
조사한다. 과거 catalog와 Boss completion audit은 identity와 비교 기준인 historical input으로만 사용한다.

## Source Of Truth And Ownership

Evidence precedence와 공통 불변식은 Master의 `Common Source Of Truth And Invariants`를 그대로 따른다.

1. current `core_enhanced` runtime과 authoritative mutation
2. production loader가 실제 소비하는 `game_resources` live data
3. `core_enhanced/docs/README.md`가 지정하는 current canonical
4. 실제 producer type과 production path를 사용하는 test
5. 필요한 `game_server` 및 접근 가능한 외부 Unity consumer
6. 기존 refactoring/Boss Goal과 experiment는 historical support

P00은 최종 candidate disposition을 소유하지 않는다. P00은 identity, baseline, 조사 surface와 Phase owner를
고정하고 P01~P10이 current evidence로 disposition할 수 있게 한다. Final은 cross-boundary dedupe와 최종
identity를 소유한다.

## Invariants

- Master RBA-01~RBA-14 전체 coverage를 축소하지 않는다.
- 기존 최종 51개 후보를 정확히 한 번씩 inventory하고 source ID traceability를 보존한다.
- Boss 관련 19개를 historical disposition 그대로 승인하지 않고 deep-review owner만 배정한다.
- current source/data/test/canonical/server/Unity를 수정, format, 정리, 이동 또는 삭제하지 않는다.
- 기존 dirty/untracked 사용자 변경과 unrelated `game_server/logs/app.log.2026-08-09`를 보존한다.
- search hit, 새 module 존재, 파일 크기나 이름 유사성만으로 신규 후보를 만들지 않는다.
- 신규 후보 ID namespace는 기존 `CRA-*`, Boss `BP-*` 및 다른 current Goal ID와 충돌하지 않는다.
- 접근하지 않은 consumer와 실행하지 않은 test를 verified로 표현하지 않는다.

## Refactor Candidate And Behavior-Preservation Boundary

P00은 후보 구현 Goal이 아니라 inventory Phase다. 기존 51개의 원래 work type, priority와 invariant를
그대로 trace하되 current work type/disposition을 선판정하지 않는다. 신규 Boss surface도 evidence owner와
candidate-creation rule만 정하고 실제 후보 확정은 배정 Phase와 Final이 수행한다.

P00 산출물에 최소한 다음 column을 둔다.

- Final/source Candidate ID와 원본 artifact
- 원래 work type, priority/readiness와 preserved invariant
- Boss historical relation 여부와 disposition 주장
- current-tree changed/unchanged/unknown surface
- primary Phase와 필요한 cross-Phase contribution
- expected runtime/data/test/canonical/consumer evidence
- current disposition owner와 P00에서 미확정인 이유

## Approved Policy And Evidence

- Master RBA-01: current dirty baseline
- Master RBA-02: exact 51-candidate mapping
- Master RBA-04: 19 deep + 32 impact review owner 배정
- Master RBA-05: Boss surface inventory와 증거 기반 신규 후보 규칙
- Master RBA-06: evidence precedence
- Master RBA-09: external/consumer classification
- Master RBA-10: test family/invariant inventory
- Master RBA-11~RBA-12: read-only와 dirty preservation
- Master RBA-14: historical preservation

승인 근거는 Master 필수 문서 3개와 current
`goal_authoring_and_execution_guide.md`, `engineering_workflow.md`,
`goal_completion_review_guide.md`다.

## Open Policy Decisions

초기화 시점에는 없다. P00이 Phase boundary를 확정하기 위해 gameplay, DTO, live RON, save/checkpoint,
compatibility, balance 또는 rollout 의미를 결정해야 하면 해당 결론을 확정하지 않고 Status를
`policy_decision_required` 또는 실제 blocker에 맞게 기록한다.

## In Scope

- exact `git status --short`, HEAD, repository/category/count와 relevant untracked content baseline
- 기존 final 51개와 Phase 1~10 source candidate의 exact identity/merge trace
- Boss completion audit이 추적한 19개 relation과 원래 candidate 전체 범위 trace
- current Boss catalog/setup/runtime/data/test/server/public-private surface의 inventory
- current Core source/test/canonical과 supporting live data/server/accessible Unity surface의 Phase 배정
- 기존 test family의 runner location, setup/action/assertion owner와 재검토 Phase 배정
- 충돌 없는 신규 reaudit candidate namespace와 source-phase ID 규칙
- Phase 1~10 registry가 current boundary를 누락 없이 다루는지 확인
- P00 Goal 문서, inventory artifact와 `COMPLETION_AUDIT.md` 갱신

## Out Of Scope

- production code/data/test/canonical/server/Unity 수정 또는 formatter
- 기존 51개나 신규 후보의 최종 current disposition, 구현 우선순위와 wave 확정
- 실제 cleanup/refactor/correction/migration 구현
- gameplay/UX/balance/content/DTO/RON/save/compatibility 정책 결정
- 범위 밖 `game_resources`, `game_server`, Unity 자체 리팩토링 후보 발굴
- 기존 refactoring/Boss Goal history 수정
- broad suite 재실행을 source inspection의 대체로 사용

## Investigation And Artifact Scope

P00은 다음 artifact를 작성한다.

- `CURRENT_TREE_BASELINE.md`: HEAD, exact status, category/count, relevant content identity와 preservation rule
- `ORIGINAL_CANDIDATE_INVENTORY.md`: final 51↔source candidate exact map, Boss 19 relation, Phase owner
- `BOSS_DELTA_SURFACE_INVENTORY.md`: current Boss authority/data/test/consumer seams와 후속 evidence owner
- `PHASE_BOUNDARY_REVIEW.md`: Phase 1~10 coverage, dependency, cross-contribution과 신규 ID rule
- `COMPLETION_AUDIT.md`: P00 자체 `Complete / High` evidence

Production mutation은 `none`이다.

## DTO/Data/Save Migration

모두 `not_applicable`이다. 현재 shape/lifetime/consumer가 inventory evidence로 필요하면 read-only로만
기록하고 migration 또는 compatibility 정책을 만들지 않는다.

## Deployment And Compatibility

감사 문서 외 배포는 없다. Producer-consumer topology나 compatibility는 inventory/gate로만 기록한다.
접근하지 못한 public Rust, Unity, standalone 또는 deployed consumer는 `unverified`로 남긴다.

## Focused Validation

- exact 51 count와 duplicate/missing ID를 deterministic text query로 검증
- final↔source map과 Boss 19 set의 cardinality 및 membership 검증
- current status artifact가 captured raw status와 일치하는지 검증
- Phase 1~10 각각에 기존 candidate/test/data/consumer owner가 배정됐는지 검증
- 신규 prefix가 current repository ID와 충돌하지 않는지 검증
- artifact link/path와 trailing whitespace 검증

Runtime test는 static evidence로 닫히지 않는 구체적 P00 inventory gap이 있을 때만 좁게 실행한다. 실행 시
정확한 runner와 pass/fail/skip 수를 기록한다.

## Broad Validation

P00은 production behavior를 변경하지 않으므로 repository-wide test는 기본 요구사항이 아니다. Completion
audit에서 inventory의 exactness와 current dirty preservation을 독립 재검증한다.

## Omitted Validation

- formatter: production/contract 문서를 변경하지 않고 historical text artifact를 작성하므로 생략
- broad Core/server suite: P00 candidate inventory의 identity와 boundary를 직접 증명하지 않아 생략
- live server/WebSocket/Unity build: public contract를 변경하지 않고 consumer surface만 분류하므로 생략
- performance/load/balance/art/VFX: P00 완료 조건과 무관

## Completion Conditions

- P00 시작 tree를 재현할 exact status/category/content baseline이 있다.
- 기존 final 51개가 누락·중복 없이 source ID, 원래 invariant/work type과 Phase owner에 연결된다.
- Boss 교차 19개가 exact membership과 원래 전체 범위, deep-review owner에 연결된다.
- 나머지 32개의 current-tree impact-review owner가 명시된다.
- current Boss runtime/data/test/consumer seam이 current source evidence와 함께 inventory된다.
- 신규 candidate namespace/source-phase 규칙이 충돌 없이 확정된다.
- Phase 1~10 registry가 approved coverage를 유지하거나, material mismatch가 있으면 확정하지 않고 정확한
  stop condition을 기록한다.
- P00 artifact 외 production 및 historical Goal mutation이 없고 초기 dirty 변경이 보존된다.
- 자체 completion audit의 primary verdict와 long-term fit이 `Complete / High`다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료하지 않는다.

## Completion Audit Evidence Matrix

| ID | Completion condition or invariant | Required evidence | Status |
| --- | --- | --- | --- |
| P00-C01 | exact current dirty baseline이 재현 가능함 | `CURRENT_TREE_BASELINE.md`, current status comparison | pass |
| P00-C02 | original final 51가 정확히 한 번 mapping됨 | deterministic count/membership check, inventory | pass |
| P00-C03 | Boss-linked 19와 other 32가 분리되고 owner를 가짐 | set/cardinality check, inventory | pass |
| P00-C04 | Boss delta authority/data/test/consumer surface가 배정됨 | Boss surface inventory와 source pointers | pass |
| P00-C05 | Phase 1~10 boundary가 approved coverage를 유지함 | boundary review와 Master registry comparison | pass |
| P00-C06 | 신규 ID 규칙이 충돌하지 않음 | repository ID search와 rule | pass |
| P00-C07 | test/consumer 범위가 verified/unverified/excluded로 정직하게 분류됨 | inventory matrices | pass |
| P00-C08 | production/historical mutation 없이 dirty state가 보존됨 | start/end scoped status and artifact audit | pass |
| P00-C09 | P00 독립 완료 감사가 Complete / High임 | `COMPLETION_AUDIT.md` | pass |

## Stop Conditions

- original final/source catalog 또는 Boss 19 identity를 추적할 수 없어 exact reconciliation이 불가능함
- current Phase 1~10 구조가 material surface를 소유할 수 없고 승인되지 않은 Master 재구조화가 필요함
- 필수 external repository/consumer 없이는 boundary 자체를 판정할 수 없음
- policy/compatibility 의미를 정해야만 inventory identity나 owner를 확정할 수 있음
- production mutation, formatter 또는 historical Goal rewrite가 필요함
- 기존 사용자 변경과 P00 artifact를 안전하게 구분할 수 없음
