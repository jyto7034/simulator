# Core Enhanced Refactoring Audit Phase 01 — Live Data Loading And Validation Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: phase
Parent Master: `core_enhanced_refactoring_audit_master`
Phase: 1
Execution Mode: investigation_only
Production Mutation Authorized: no
Audit Baseline Date: 2026-08-07
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

현재 working tree에서 `core_enhanced`가 production gameplay RON을 embedded bundle로 구성하고,
typed catalog로 parse·validate·publish하는 authority 경계를 실제 code/live-data/test/canonical evidence로
감사한다. 확인된 중복, fallback, validation gap 또는 변경 증폭 신호를 실행 가능한 후보나 명시적
rejected/deferred/unverified disposition으로 분류한다.

## User/Contract-Visible Problem

Runtime이 읽는 파일 집합, typed schema, cross-record validation과 published database가 서로 다른
authority처럼 drift하면 invalid content가 startup 뒤에 발견되거나 작은 data 변경이 loader/test/document
여러 곳을 산탄 수정하게 만들 수 있다. 반대로 source-count 문구나 구조적 유사성만으로 리팩토링을
확정하면 실제로 독립적인 data contract를 잘못 합칠 수 있다.

## Audit Finding ID

없음. Parent Master CRA-01~CRA-13의 첫 boundary investigation Phase다. Candidate ID는 실제 evidence를
확인한 뒤 `CRA-P01-CNN` 형식으로 부여한다.

## Parent Scope And Direct Dependency

- Master registry: [PLAN.md](../core_enhanced_refactoring_audit_master/PLAN.md)
- Direct predecessor: Phase 0
  [COMPLETION_AUDIT.md](../core_enhanced_refactoring_audit_phase_00_surface_inventory/COMPLETION_AUDIT.md),
  verdict `Complete / High`
- Boundary decision:
  [BOUNDARY_PROPOSAL.md](../core_enhanced_refactoring_audit_phase_00_surface_inventory/BOUNDARY_PROPOSAL.md)
- Activation authority: Parent CRA-13. 사용자는 현재 Phase 0 경계·순서로 시작하고 앞으로의 Child/후속
  Goal 이름을 작업자가 정하도록 위임했다.
- 다음 Phase는 이 Phase가 자체 완료 감사에서 `Complete / High`를 받기 전에는 만들거나 활성화하지 않는다.

## Approved Audit Requirements

| ID | 이 Phase의 책임 |
| --- | --- |
| CRA-01 | primary target을 current working-tree `core_enhanced` data-loading 경계로 한정한다. |
| CRA-02 | frozen `engineering_workflow.md#리팩토링-판단`으로 후보와 readiness를 판정한다. |
| CRA-03 | runtime loader, 실제 live data, durable canonical, test, history 순서로 evidence를 적용한다. |
| CRA-04 | 작성 도구, 문체, 파일 길이 또는 이름 유사성을 후보 evidence로 사용하지 않는다. |
| CRA-05 | 확정 후보마다 code/data/test evidence, 현재/목표 authority와 불변식을 기록한다. |
| CRA-06 | 7개 priority axis와 readiness를 분리한다. |
| CRA-07 | behavior-preserving/correction/policy migration을 구분한다. |
| CRA-08 | `game_resources`, config와 외부 consumer evidence를 confirmed/unverified/out-of-scope로 분류한다. |
| CRA-09 | runtime-consuming data authority 하나만 감사하고 domain behavior는 후속 Phase에 남긴다. |
| CRA-10 | production code/data/test/canonical을 수정하지 않는다. |
| CRA-11 | 초기 dirty/untracked 변경을 사용자 소유로 보존한다. |
| CRA-12 | investigated signal 모두에 candidate/rejected/deferred/unverified disposition을 부여한다. |
| CRA-13 | 위임된 이름과 승인된 순서로 이 Phase를 활성화하되 material scope/policy를 임의 확장하지 않는다. |

## Current Runtime And Live Data Evidence

Phase 0에서 다음 seam만 확인됐다. 세부 ownership과 candidate 여부는 아직 `discovery pending`이다.

- Production startup은 `GameDataBase::load_live_embedded()`를 사용한다.
- `GameplayRonSourceSet::embedded()`가 production source set을 구성한다.
- Phase 0 재계산에서 live data와 loader reference는 각각 23 files로 symmetric difference가 없었다.
- Code/test constant는 manifest 외 22 sources를 가리키지만 current data canonical에는 21이라는 문구가 있다.
- `config/game_balance.toml`은 confirmed workspace consumer가 없어 `unverified`다.

위 항목은 loader correctness나 refactor candidate 존재를 확정하지 않는다.

## Source Of Truth And Ownership

1. 실제 production startup이 호출하는 `core_enhanced` loader와 publication mutation
2. loader가 포함하는 `game_resources/data/**` production files
3. typed RON schema, parse/validation과 `GameDataBase` consumer entry
4. `docs/README.md`가 지정하는 가장 가까운 current data canonical
5. focused/integration tests
6. Goal, experiment, archive는 supporting history

Phase 1은 loading/validation/publication authority만 소유한다. 개별 run, economy, battle policy의 소비
계산은 해당 후속 Phase가 소유한다.

## Invariants

- production source set, serialization shape, authored content와 startup failure semantics를 변경하지 않는다.
- strict parse/validation, default/fallback/silent recovery 존재 여부를 실제 호출 경로에서 확인한다.
- source count나 canonical drift만으로 runtime defect를 추정하지 않는다.
- manifest, typed catalog와 published database의 authority를 구분한다.
- consumer domain이 data를 어떻게 해석하는지는 필요한 seam까지만 읽고 정책 후보를 이 Phase에 흡수하지 않는다.
- `game_resources`는 Core가 실제 읽는 live evidence일 때만 확인하며 그 repository 자체 후보는 만들지 않는다.
- Unity와 다른 외부 consumer는 비범위/미검증으로 유지한다.
- 기존 dirty/untracked 파일을 restore, format, 이동, 삭제 또는 정리하지 않는다.

## Refactor Candidate And Behavior-Preservation Boundary

확정 후보는 `cleanup`, `behavior_preserving_refactor`, `correctness_correction`,
`policy_or_feature_migration` 중 하나로 분류한다. 각 후보는 다음을 포함한다.

- 실제 loader→parse→validate→publication→consumer evidence
- 현재 authority와 목표 authority
- 제거·통합하려는 중복/fallback/legacy 범위
- 보존할 file set, RON/typed shape, ordering과 failure semantics
- 7개 priority axis evidence와 별도 readiness
- 먼저 필요한 characterization/focused test
- 정책·외부 계약 결정 여부와 disposition

Serialized shape, accepted content 또는 startup failure 의미를 바꿔야 하면 behavior-preserving refactor로
확정하지 않는다.

## Open Policy Decisions

- 현재 없음.
- Authored RON 의미, validation strictness 또는 compatibility를 변경해야만 후보가 성립하면
  `policy_or_feature_migration` 또는 `deferred`로 기록하고 이 Phase에서 결정하지 않는다.

## In Scope

- production embedded source-set construction과 file inclusion
- RON schema/typed catalog parse, validation, cross-record reference와 publication path
- `GameDataBase::load_live_embedded()` 및 직접 startup/consumer seam
- loader가 실제 읽는 `game_resources/data/**`의 read-only shape/count/supporting evidence
- data canonical과 source-count/document ownership 정합성
- loader/validation을 직접 보호하는 Core source/integration tests
- `config/game_balance.toml` consumer evidence의 confirmed/unverified disposition
- 후보 catalog, priority/readiness, safety-net과 policy/external-contract 기록

## Out Of Scope

- production code, RON, test, canonical, config와 server mutation
- 개별 run/map/economy/event/combat 정책 계산의 독립 refactor 후보
- balance/content correction 또는 RON schema migration 구현
- `game_resources` 자체 architecture와 authoring tooling의 독립 감사
- Unity/runtime/UI/test와 다른 external consumer
- formatter, commit, stash, cleanup 또는 완료 Goal correction

## Investigation Scope

1. exact Core source/test/canonical와 live-data surface를 재현 가능한 file inventory로 고정한다.
2. production startup에서 `load_live_embedded`까지의 호출과 publication lifetime을 추적한다.
3. embedded source-set의 manifest/non-manifest file 목록을 live file set과 대조한다.
4. parse, per-record/cross-record validation, default/fallback/error aggregation과 publication 단계를 추적한다.
5. typed catalog와 `GameDataBase` field별 authority, duplicate representation과 direct consumer seam을 조사한다.
6. focused/integration test가 실제 failure/validation/file-set invariant를 보호하는지 확인한다.
7. current canonical과 runtime/live data의 일치·drift를 분류한다.
8. 모든 signal을 candidate/rejected/deferred/unverified로 disposition하고 candidate evidence contract를 채운다.
9. 별도 completion pass에서 coverage, candidate completeness, no-production-mutation과 dirty preservation을 감사한다.

## DTO/Data/Save Migration

모두 `not_applicable`이다. 이 Phase는 DTO, RON schema/content, save/checkpoint 또는 compatibility를 변경하지
않는다. 후보가 migration을 요구하면 별도 구현 Goal 필요사항만 기록한다.

## Deployment And Compatibility

`not_applicable`. Current startup/file bundle lifetime과 consumer compatibility evidence는 readiness에 기록할
수 있지만 rollout을 설계하거나 구현하지 않는다.

## Focused Validation

- embedded reference set과 production live file set의 exact sorted-set comparison
- loader/startup/publication call-path source trace
- parse/validation/default/fallback/error path source trace
- data-focused test inventory와, 필요할 경우 정확한 filtered runner 실행
- canonical count/shape statement와 runtime source evidence comparison
- candidate별 authority/consumer/safety-net evidence 확인

## Broad Validation

- Phase 1 primary/shared surface 전체 disposition 재검토
- 모든 investigated signal의 candidate/rejected/deferred/unverified completeness
- Candidate ID, work type, 7축 점수/readiness와 preserved invariant 교차 검토
- initial/final scoped dirty set과 production no-mutation 비교

## Omitted Validation

- Domain behavior 전체 test suite: loading/validation candidate evidence에 직접 필요하지 않으면 생략한다.
- Unity/external runtime validation: 사용자 승인 비범위다.
- Formatter: read-only audit이므로 실행하지 않는다.

생략·실행 여부와 실제 test 수는 `EXPERIMENTS.md`에 기록한다.

## Completion Conditions

- production startup→loader→source set→parse/validate→publication path가 evidence로 연결된다.
- embedded reference set과 production live data가 exact disposition을 가진다.
- typed catalog, validation, fallback/default/error와 publication authority가 coverage table에 포함된다.
- relevant canonical과 test safety net이 current runtime과 구분되어 평가된다.
- 조사한 모든 signal이 candidate/rejected/deferred/unverified 중 하나로 disposition된다.
- 확정 후보가 CRA-05/06/07 evidence contract를 모두 가진다.
- domain behavior와 external consumer 비범위가 과대 검증되지 않는다.
- production mutation 없이 Goal 문서와 audit artifacts만 갱신된다.
- Phase completion audit가 `Complete / High`다.

## Completion Audit Evidence Matrix

| ID | Completion condition or invariant | Required evidence | Status |
| --- | --- | --- | --- |
| P1-01 | exact surface coverage | source/test/canonical/live-data inventory | pass |
| P1-02 | startup and loader trace | production call/publication evidence | pass |
| P1-03 | embedded/live file reconciliation | sorted set and count comparison | pass |
| P1-04 | parse/validation authority trace | schema, validator, error/default/fallback evidence | pass |
| P1-05 | typed catalog/publication ownership | field and direct consumer map | pass |
| P1-06 | canonical/test safety-net audit | current canonical and focused test evidence | pass |
| P1-07 | complete signal disposition | candidate/rejected/deferred/unverified catalog | pass |
| P1-08 | candidate evidence quality | work type, authority, invariants, axes/readiness | pass |
| P1-09 | dirty preservation/no mutation | initial/final scoped status comparison | pass |
| P1-10 | completion audit verdict | `COMPLETION_AUDIT.md` Complete / High | pass |

## Stop Conditions

- Runtime-consuming production file set을 접근하거나 재현할 수 없음
- Authored RON/validation/failure/compatibility 정책 없이는 후보 분류가 편향됨
- Phase 2 이상 domain behavior를 material하게 조사해야만 Phase 1 경계를 완결할 수 있음
- 외부 consumer를 확인하지 않고 verified compatibility를 주장해야 함
- read-only command가 production state나 사용자 파일을 변경할 위험이 있음
- frozen audit standard가 외부 변경으로 drift함
- 초기 dirty change와 audit artifact를 안전하게 구분할 수 없음
- 다음 Phase activation: Phase 1 자체 `Complete / High` 전에는 중단
