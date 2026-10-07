# Core Enhanced Refactoring Audit Master Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: master
Execution Mode: orchestration_only
Production Mutation Authorized: no
Audit Baseline Date: 2026-08-07
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Overall Objective And Completion Conditions

현재 working tree의 `core_enhanced` 프로젝트를 대상으로 실제 code/data/test evidence에 근거한
리팩토링 후보를 발굴하고, ownership·correctness·lifecycle·변경 비용·검증 안전망을 기준으로
분류·우선순위화한 실행 가능한 후보 catalog를 만든다.

이 Master는 리팩토링을 구현하지 않는다. 완료 결과는 다음을 모두 포함하는 read-only 감사
산출물이다.

- `core_enhanced`의 감사 대상 surface와 source-of-truth 경계 inventory
- 증거가 확인된 후보별 Candidate ID, 작업 유형과 disposition
- 현재 authority/producer/consumer와 목표 authority
- 보존할 사용자-visible 또는 contract-visible 불변식
- 제거·통합할 중복, fallback, legacy 또는 parallel path
- `engineering_workflow.md`의 7개 우선순위 축과 실행 준비도
- 필요한 characterization/focused test, 기존 test의 의미·중복·경계 품질 disposition과 정책·외부 계약 결정
- 중복 후보를 합친 cross-boundary 최종 우선순위 목록
- 기각·보류·미검증 후보와 그 근거

Master 완료는 production refactor 완료를 뜻하지 않는다. 실제 correction, cleanup, refactor 또는
policy migration은 사용자 승인을 받은 별도 구현 Goal이 소유한다.

## User/Contract-Visible Problem

여러 AI와 작업자가 누적한 현재 코드에서 ownership 중복, 부분 mutation 위험, lifecycle 분산,
fallback과 과도한 추상화 후보를 일관된 기준으로 판단하고 실행 순서를 정할 근거가 아직 없다.
파일 크기나 작성 도구가 아니라 실제 runtime 계약과 변경 위험을 기준으로 후보 backlog를 만들어야
후속 리팩토링이 gameplay 또는 serialized contract를 우발적으로 바꾸지 않는다.

## Audit Finding ID

없음. 기존 완료 감사에서 파생된 correction Goal이 아니라 사용자가 승인한 신규 조사 campaign이다.

## Approved Policy Package And IDs

아래 ID는 gameplay 정책 ID가 아니라 이 감사 campaign의 승인된 조사·판정 요구사항이다. 승인 근거는
2026-08-07 사용자 대화와 현재 working-tree의
[engineering_workflow.md](../../engineering_workflow.md#리팩토링-판단)다.

| ID | 승인된 감사 요구사항 |
| --- | --- |
| CRA-01 | Primary audit target은 현재 working tree의 `core_enhanced`다. |
| CRA-02 | 리팩토링 판단 기준은 현재 working-tree `engineering_workflow.md`의 `리팩토링 판단` 절이다. |
| CRA-03 | 실제 runtime/live data와 가장 가까운 durable canonical을 역사 문서보다 우선한다. |
| CRA-04 | AI 작성 여부, 문체, 파일 길이와 취향만으로 후보를 만들지 않는다. |
| CRA-05 | 모든 확정 후보는 Candidate ID, 작업 유형, code/data/test evidence, 현재/목표 authority와 불변식을 가진다. |
| CRA-06 | 7개 우선순위 축과 실행 준비도를 분리하고, evidence가 부족한 점수를 자동 verdict로 사용하지 않는다. |
| CRA-07 | correctness correction과 policy/feature migration을 behavior-preserving refactor로 숨기지 않는다. |
| CRA-08 | 범위 밖 data/consumer는 증거 유형을 명시하고 확인하지 않은 경계를 검증한 것으로 표현하지 않는다. |
| CRA-09 | 독립 source-of-truth 경계별 감사를 거친 뒤 competing authority와 중복 후보를 교차 정리한다. |
| CRA-10 | 이 Master와 모든 child는 read-only이며 production code/data/test/canonical을 수정하지 않는다. |
| CRA-11 | 초기 dirty 변경과 이전 Goal 산출물을 사용자 소유로 보존한다. |
| CRA-12 | 최종 catalog에는 우선 후보뿐 아니라 보류·기각·미검증 disposition과 후속 승인 필요사항도 포함한다. |
| CRA-13 | 2026-08-07 사용자 지시에 따라 현재 Phase 0 경계·순서 제안을 실행하고, 이후 Child/후속 Goal 이름은 작업자가 범위·의존성에 맞게 정한다. 이름 결정만으로 재승인을 요구하지 않으며 material scope나 사용자-visible 정책 변경은 별도 승인을 받는다. |
| CRA-14 | 전체 test runner 실행은 기본 요구사항이 아니다. 각 Phase는 배정된 test code를 읽고 실제 runtime/live-data/canonical 계약과 대조하여 테스트의 고유 invariant, 도달 가능한 경계, assertion 의미와 중복을 감사하고 `retain`/`strengthen`/`merge`/`remove`/`unverified` disposition을 남긴다. 통과 수나 happy-path 여부만으로 품질 또는 제거를 판정하지 않는다. |

## Common Source Of Truth And Invariants

### Evidence precedence

1. 현재 `core_enhanced` runtime code와 authoritative state mutation
2. 해당 runtime이 실제 소비하는 production live data
3. [README.md](../../README.md)가 지정하는 가장 가까운 durable canonical
4. 실제 producer type을 사용하는 focused/integration test code의 주장·assertion·fixture 경계와, 정말
   필요한 경우에만 수행한 read-only probe
5. 완료 Goal, experiment와 archived audit는 역사적 supporting evidence만 제공

`engineering_workflow.md`는 감사 방법과 리팩토링 판정 기준을 소유하지만 gameplay/runtime/data
계약의 source of truth는 아니다.

### Frozen audit standard

- 감사 기준은 Master 초기화 시점 current working-tree의
  [engineering_workflow.md](../../engineering_workflow.md#리팩토링-판단)다.
- 감사 중 기준 문서를 임의로 수정하지 않는다.
- 기준 변경이 별도로 발생하면 변경 시점, 영향받는 Candidate ID와 재평가 여부를 기록한다.

### Common invariants

- production mutation, formatter, cleanup, refactor implementation과 canonical 갱신을 수행하지 않는다.
- 기존 dirty/untracked 파일을 되돌리거나 이동·삭제·정리하지 않는다.
- 검색 hit, 이름 유사성이나 정적 모양만으로 결함 또는 후보를 확정하지 않는다.
- 후보는 실제 authority→producer→consumer 또는 runtime/live-data 경로 evidence를 가진다.
- 사용자-visible behavior, serialized shape, timing/order와 실패 의미의 변경 가능성을 명시한다.
- 우선순위가 높다는 사실과 안전하게 구현할 준비가 됐다는 사실을 구분한다.
- 외부 또는 범위 밖 consumer를 읽지 않았으면 `unverified` 또는 명시적 비범위로 기록한다.
- 독립 계약 경계를 외형이 비슷하다는 이유로 한 후보나 Phase에 합치지 않는다.
- 정책 질문이 필요한 후보는 구현안을 확정하지 않고 `policy_or_feature_migration` 또는 보류로 분류한다.
- 모든 test가 통과한다는 baseline이나 test 개수는 test 품질 또는 runtime 정확성의 단독 evidence가 아니다.
- Happy path test는 그 자체로 제거 대상이 아니다. 실제 성공 계약·조립·상태 전이·결과 shape를 고유하게
  보호하면 유지하고, `Ok` 반환이나 fixture 자기복제만 확인하며 고유 invariant가 없으면 제거·통합 후보다.
- Test-only fixture에서만 도달 가능한 경계, 구현 내부 순서/private shape를 과도하게 고정하는 assertion,
  더 강한 기존 test의 약한 중복과 legacy/fallback을 잘못 authority로 만드는 test를 명시적으로 찾는다.
- Test 제거 후보는 해당 test가 고유한 contract를 보호하지 않고, 제거 또는 병합 뒤에도 동일하거나 더
  강한 안전망이 남는다는 evidence를 가져야 한다.

## Master In Scope / Out Of Scope

### In scope

- `core_enhanced/src/**`의 read-only runtime와 ownership 감사
- `core_enhanced/tests/**`와 범위 안 test module의 계약·안전망·fixture 경계·assertion 의미·중복 감사
- Test family별 `retain`/`strengthen`/`merge`/`remove`/`unverified` disposition과 test cleanup 후보 문서화
- `core_enhanced/docs/**`의 current canonical, workflow와 stale/ownership 경계 감사
- `core_enhanced` manifest와 직접 dependency surface의 read-only 확인
- `core_enhanced`가 실제 소비하는 live data 또는 server DTO 경계의 필요한 read-only supporting evidence
- 후보 inventory, scoring, readiness, disposition과 cross-boundary consolidation 문서화
- 이 Master와 승인되어 활성화된 investigation-only Phase Goal 문서·감사 산출물 갱신

### Out of scope

- `core_enhanced`, `game_resources`, `game_server`와 외부 저장소의 production mutation
- 실제 cleanup, bug correction, refactor와 policy/feature migration 구현
- `game_resources`와 `game_server` 자체의 독립 리팩토링 후보 발굴
- Unity runtime, contract, UI, test와 repository
- gameplay, UX, balance, DTO/RON/save 의미의 신규 정책 결정
- 완료된 이전 Goal의 상태·문서·구현 correction
- archive/debug/generated artifact를 현재 source of truth로 승격하는 작업
- commit, stash, branch, format 또는 worktree cleanup

Supporting evidence가 범위 밖 경계에 필요하면 읽은 범위와 이유를 기록한다. 읽지 못했거나 사용자
비범위인 경계 때문에 후보를 확정할 수 없으면 `unverified`로 남긴다.

## Policy Coverage Matrix

| Requirement | Phase 0 surface inventory | Boundary investigation Phase(s) | Final consolidation | Final disposition |
| --- | --- | --- | --- | --- |
| CRA-01 scope | primary | enforce | re-audit | complete |
| CRA-02 audit standard | freeze | apply | re-audit | complete |
| CRA-03 evidence precedence | map | apply | re-audit | complete |
| CRA-04 origin-neutral judgment | record | enforce | re-audit | complete |
| CRA-05 candidate evidence | define inventory seam | primary | completeness audit | complete |
| CRA-06 priority/readiness | identify available evidence | primary | normalize and rank | complete |
| CRA-07 work-type separation | identify policy seams | primary | reclassify drift | complete |
| CRA-08 external boundary classification | map | apply | completeness audit | complete |
| CRA-09 cross-boundary consolidation | propose boundaries | contribute | primary | complete |
| CRA-10 read-only execution | enforce | enforce | enforce | complete |
| CRA-11 dirty preservation | snapshot | enforce | final audit | complete |
| CRA-12 complete disposition set | define catalog shape | contribute | primary | complete |
| CRA-13 delegated naming/activation | proposal | enforce | re-audit | complete |
| CRA-14 test-code meaning and cleanup audit | inventory test surfaces | primary | dedupe and completeness audit | complete |

## Phase Dependency And Status Table

한 번에 하나의 Phase만 활성화한다. 병렬 실행은 승인하지 않았다.

| Phase | Purpose | Child Goal Name | Dependency | Status |
| --- | --- | --- | --- | --- |
| 0 | 감사 surface, canonical/authority inventory와 독립 경계·후속 Phase 제안 | `core_enhanced_refactoring_audit_phase_00_surface_inventory` | Master initialized | complete (`Complete / High`) |
| 1 | embedded live data loading/validation | `core_enhanced_refactoring_audit_phase_01_live_data_loading_and_validation` | Phase 0 complete, CRA-13 | complete (`Complete / High`) |
| 2 | run/map/checkpoint/persistence authority | `core_enhanced_refactoring_audit_phase_02_run_map_and_persistence` | Phase 1 `Complete / High` | complete (`Complete / High`) |
| 3 | roster/inventory/economy authority | `core_enhanced_refactoring_audit_phase_03_roster_inventory_and_economy` | Phase 2 `Complete / High` | complete (`Complete / High`) |
| 4 | interactive node/VN lifecycle | `core_enhanced_refactoring_audit_phase_04_interactive_node_and_visual_novel` | Phase 3 `Complete / High` | complete (`Complete / High`) |
| 5 | combat preview/setup/handoff | `core_enhanced_refactoring_audit_phase_05_combat_preview_and_setup` | Phase 4 `Complete / High` | complete (`Complete / High`) |
| 6 | battle lifecycle/event/result | `core_enhanced_refactoring_audit_phase_06_battle_lifecycle_and_result` | Phase 5 `Complete / High` | complete (`Complete / High`) |
| 7 | battle spatial/tactical authority | `core_enhanced_refactoring_audit_phase_07_battle_spatial_and_tactical` | Phase 6 `Complete / High` | complete (`Complete / High`) |
| 8 | battle effects/stats authority | `core_enhanced_refactoring_audit_phase_08_battle_effects_and_stats` | Phase 7 `Complete / High` | complete (`Complete / High`) |
| 9 | battle targeting/delivery authority | `core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery` | Phase 8 `Complete / High` | complete (`Complete / High`) |
| 10 | command/projection/direct server seam | `core_enhanced_refactoring_audit_phase_10_command_projection_and_server_seam` | Phase 9 `Complete / High` | complete (`Complete / High`) |
| Final | candidate dedupe, competing authority, priority/readiness와 disposition 교차 감사 | `core_enhanced_refactoring_audit_final_catalog_consolidation` | Phase 10 `Complete / High` | complete (`Complete / High`) |

Phase 0~10은 2026-08-07 `Complete / High`로 종료했다. 사용자는 같은 날 현재 Phase 0
`BOUNDARY_PROPOSAL.md`의 경계·순서로 실행을 시작하고 앞으로의 Child/후속 Goal 이름은 작업자가
정하도록 위임했다. Phase 4는 direct dependency와 preflight를 확인한 뒤 활성화했으며 이후 Phase도
직전 Phase의 `Complete / High` 전에는 만들지 않는다.

### Goal naming delegation

- Master의 frozen scope와 dependency 안에서 Child/후속 Goal 이름 선택·정규화는 작업자 권한이다.
- 이름만 새로 정하거나 더 명확하게 바꾸는 일은 사용자 승인 gate가 아니다.
- 현재 registry 이름은 예약 이름이며 실제 activation 때 충돌 여부와 경계를 확인해 확정할 수 있다.
- Phase 수·authority 경계의 material 변경, gameplay/UX/DTO/RON/save/compatibility 정책 결정은 이 위임에
  포함되지 않으며 기존 stop condition과 사용자 승인 절차를 따른다.

## Shared Migration And Version Strategy

이 Master는 investigation-only campaign이므로 DTO, live data, save/checkpoint, event log, protocol,
deployment와 compatibility migration은 모두 `not_applicable`이다.

후보가 migration이나 compatibility를 요구하면 구현하지 않고 다음을 기록한다.

- 영향을 받는 authoritative shape와 producer/consumer
- 현재 version/lifetime evidence
- 정책 또는 rollout 결정 필요 여부
- 후보 작업 유형과 별도 구현 Goal 필요성

## Phase Completion Audit Gate

각 Phase는 [goal_completion_review_guide.md](../../goal_completion_review_guide.md)를 적용한 별도
완료 감사를 가진다. Investigation Phase의 완료는 구현 완료가 아니라 승인된 read-only 조사 범위와
evidence 산출물 완료를 뜻한다.

각 Phase는 최소한 다음을 만족해야 한다.

- 배정된 surface 또는 계약 경계를 누락 없이 disposition함
- 후보마다 실제 code/data/test evidence 또는 미확정 이유를 기록함
- 현재 authority, consumer, canonical과 테스트 안전망을 구분함
- 배정된 test code를 읽고 test family가 보호하는 고유 invariant, production 도달 가능성, assertion의
  관찰력과 fixture 경계를 검토함
- Test family마다 `retain`/`strengthen`/`merge`/`remove`/`unverified` disposition과 근거를 기록함
- 단순 happy-path, 통과 여부, test 수 또는 구현과 함께 움직이는 assertion만으로 test 품질을 주장하지 않음
- work type, priority axes, readiness와 정책 질문을 기록함
- 검색 hit만으로 확정하거나 범위 밖 consumer를 검증한 것으로 표현하지 않음
- production mutation과 기존 dirty 변경 침해가 없음
- Phase evidence matrix와 `Complete / High` verdict가 있음

`Complete / High` 전에는 다음 Phase를 활성화하지 않는다.

## Cross-Phase Validation

정확한 명령은 Phase 0 inventory와 각 boundary 조사 뒤 영향 범위에 맞게 정한다. 이 Master 생성
단계에서는 code search, baseline test, formatter와 runtime probe를 실행하지 않았다.

최종 교차 검증은 최소한 다음을 포함한다.

- Phase surface coverage와 누락 경계 감사
- Candidate ID 중복, 동일 root cause와 competing target authority 병합
- 각 후보의 work type, priority axes와 readiness evidence 일관성
- policy/feature migration이 refactor로 잘못 분류되지 않았는지 확인
- supporting data/consumer의 confirmed/unverified/out-of-scope 분류 확인
- initial dirty state와 production no-mutation 확인
- 최종 catalog의 priority, deferred, rejected와 unverified disposition 완전성

전체 test runner와 broad/focused 재실행은 기본적으로 생략한다. 사용자가 제공한 current baseline은
`core_enhanced` test 전체 통과이며, 이는 역사적 baseline일 뿐 test 의미나 coverage 품질의 증거가 아니다.
Child Phase는 test source를 반드시 읽고 다음을 직접 감사한다.

- Test 이름과 실제 setup/action/assertion이 같은 계약을 검증하는지
- Assertion이 단순 성공 여부가 아니라 결과, authoritative state, 순서, 실패 원인 또는 no-mutation을
  의미 있게 관찰하는지
- Production에서 도달 가능한 입력·경계인지, 아니면 fixture가 만든 인위적 경계를 고정하는지
- 같은 invariant의 더 강한 test가 있어 약한 중복을 병합·제거할 수 있는지
- 구현 내부/private shape에 결합되어 behavior-preserving refactor를 불필요하게 방해하는지
- Runtime/live data/canonical과 충돌하는 legacy/fallback/default를 보호하고 있지 않은지

Runner 또는 read-only probe는 정적 source/data/canonical/test-code 대조만으로 후보를 확정할 수 없는
구체적인 evidence gap이 있을 때만 실행한다. 실행 전 Phase 문서에 필요한 이유를 기록하고, 실행 뒤
정확한 명령과 실제 pass/fail/skip/0-test 수를 남긴다. 0-test filter, compile-only 결과와 단순 green
count는 runtime 계약 또는 test 품질 증거로 사용하지 않는다.

CRA-14는 Phase 3 종료 후 추가됐다. Phase 0~3의 완료 상태나 기존 후보 결론을 runner 재실행 목적으로
되돌리지 않는다. 대신 Final consolidation이 Phase 0~3에 배정됐던 test surface를 다시 읽어 test-family
disposition을 보완하고 M-11을 닫는다. Phase 4 이후 child는 자체 완료 조건으로 CRA-14를 직접 적용한다.

## Master Completion Audit Evidence Matrix

| ID | Completion condition or invariant | Required evidence | Status |
| --- | --- | --- | --- |
| M-01 | 모든 in-scope surface에 최종 disposition이 있음 | Phase coverage와 final inventory | complete |
| M-02 | 모든 확정 후보가 CRA-05 필드를 가짐 | Candidate catalog | complete — source catalogs + merged full fields |
| M-03 | 우선순위와 실행 준비도가 분리됨 | Candidate score/readiness evidence | complete — tier/wave/gate index |
| M-04 | behavior-preserving/correction/migration 분류가 정확함 | Work-type cross-audit | complete — 22/19/10 plus deferred migrations |
| M-05 | competing authority와 중복 후보가 정리됨 | Cross-boundary findings | complete — 53→51, two merges |
| M-06 | 범위 밖 consumer/data가 정확히 분류됨 | Consumer evidence matrix | complete — explicit external/policy gates |
| M-07 | 보류·기각·미검증 후보가 이유와 함께 남음 | Final disposition table | complete — source signal index preserved |
| M-08 | production mutation이 없음 | final scoped status/diff audit | complete — counts/hashes identical |
| M-09 | 초기 사용자 dirty가 보존됨 | initial/final dirty comparison | complete — Phase 0 counts reproduced |
| M-10 | 최종 Master 감사가 `Complete / High`임 | Master `COMPLETION_AUDIT.md` | complete |
| M-11 | 모든 in-scope test family가 의미·중복·경계 품질 disposition을 가짐 | Phase test audit와 final test-cleanup catalog | complete |

## Master Completion Audit

모든 boundary Phase가 `Complete / High`인 뒤 다음을 별도 pass로 감사한다.

- CRA-01~CRA-14 traceability
- Phase 사이 coverage gap과 중복 root cause
- 최종 target authority의 충돌 여부
- 우선순위 점수의 evidence와 critical-risk override 적용
- high-priority지만 safety net이 없어 deferred된 후보의 정확한 표현
- test family별 고유 invariant와 `retain`/`strengthen`/`merge`/`remove`/`unverified` disposition 완전성
- 무의미한 happy-path/인위적 fixture 경계/약한 중복 test 후보가 evidence 없이 과대·과소 분류되지 않았는지
- policy/external-contract 질문과 별도 사용자 승인 필요사항
- production no-mutation, scoped Goal-document diff와 dirty preservation

Master 자체가 `Complete / High`일 때만 Status를 `complete`로 바꾼다. 조사 전용이므로
`Implementation Complete`는 `not_applicable`을 유지한다.

## Stop Conditions

다음 경우 영향을 받는 Phase를 중단하고 실제 상태를 기록한다.

- Phase 0 조사 결과가 승인된 Master 범위를 material하게 바꿔야 함
- 후속 Phase의 material authority 경계·수 또는 사용자-visible 정책 변경에 사용자 승인이 필요함
- gameplay/UX/DTO/RON/save/compatibility 정책 없이는 후보 분류가 편향됨
- 필수 external consumer 또는 live data가 범위 밖이라 후보를 확정할 수 없음
- 현재 dirty 변경과 안전하게 구분할 수 없는 overlapping audit artifact가 발견됨
- read-only 명령이 production state 또는 사용자 파일을 변경할 위험이 있음
- 감사 기준 `engineering_workflow.md`가 변경되어 기존 후보 재평가가 필요함
- 필수 evidence를 얻을 수 없는 환경·권한·dependency blocker가 발생함

범위 밖 독립 문제는 새 Goal을 자동 생성하지 않고 `EXPERIMENT_NOTES.md`의 후속 후보로만 기록한다.
