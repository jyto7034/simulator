# Game Rulebook Policy Implementation Readiness Audit Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: single
Execution Mode: investigation_only
Production Mutation Authorized: no
Implementation Readiness Verdict: ready_for_implementation
Open Bootstrap Inputs: none
Final Content Authority: deferred_to_separate_content_goals
Simulator Start Commit: f318414fb0156a67e04c5479000c2a2683fc0c09
Unity Start Commit: 7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf

## Objective

현재 working tree의 실제 Core runtime, production live RON, game_server, persistence와 접근 가능한
Unity consumer를 읽어 기존 룰북 정합성 감사 finding GRB-M-001부터 GRB-M-015까지 최신 목표 정책으로
바꾸는 데 필요한 구현 경계, 구조적 난이도, 위험, 의존 관계, 추가 정책 결정과 안전한 구현 순서를
근거 기반으로 평가한다.

이 Goal은 정책을 구현하지 않는다. 완료 결과는 사용자가 정책 결정을 마치고 후속 구현 Goal의 경계와
순서를 선택할 수 있는 implementation-readiness evidence다. 여기서 고른 concrete ID, encounter, reward
quantity와 tutorial slot은 **게임 구동·계약 검증용 최소 bootstrap**이며 최종 콘텐츠를 확정하지 않는다.

## User/Contract-Visible Problem

기존 감사는 룰북과 현재 구현 사이의 차이를 확인했지만 다음 질문에는 답하지 않는다.

- 각 finding이 최신 정책으로 실제 구현되어야 하는지, 문서가 다시 정정되어야 하는지
- 기존 primitive를 재사용할 수 있는지, 새 authoritative state와 schema가 필요한지
- Core 수정만으로 닫히는지, live data·save·server·Unity까지 원자적으로 바꿔야 하는지
- 여러 finding이 같은 기반 작업을 공유하는지, 어떤 순서로 구현해야 중복 migration을 피하는지
- 구현 전에 gameplay·balance·save·wire·compatibility 결정을 추가로 받아야 하는지
- 개별 finding의 marginal difficulty와 완전한 work package difficulty가 각각 얼마인지

코드를 읽지 않고 단일 숫자만 붙이거나 변경 파일 수를 공수로 간주하면 상태 수명주기,
producer-consumer cutover와 회귀 검증 범위를 과소평가할 수 있다.

## Audit Finding ID

입력 finding은 완료된
[Game Rulebook Runtime Alignment Audit](../game_rulebook_runtime_alignment_audit/PLAN.md)의
[MISMATCH_CATALOG.md](../game_rulebook_runtime_alignment_audit/MISMATCH_CATALOG.md)에 기록된
GRB-M-001부터 GRB-M-015까지다.

이 Goal은 기존 ID를 유지한다. 조사 중 별도 결함을 발견해도 자동으로 새 correction finding이나
구현 Goal을 만들지 않고 범위 밖 후속 후보로만 기록한다.

## Goal Boundary Choice

이 Goal은 single investigation Goal로 수행한다.

- 산출물은 15개 finding 전체에 대한 하나의 implementation-readiness 판정이다.
- 각 finding은 서로 다른 gameplay 경계를 포함하지만 production mutation과 migration을 수행하지
  않으므로 구현 Phase처럼 원자 배포 단위로 나눌 필요가 없다.
- 실제 구현은 이 Goal이 제안하는 work package 경계에 따라 single Goal 또는 승인된 Master/Phase로
  별도 작성한다.
- 조사 중 하나의 평가 체계로 안전하게 다룰 수 없는 독립 campaign이나 별도 우선순위 결정이
  필요하면 scope_decision_required로 전환하며 사용자 승인 없이 Master/Phase를 생성하지 않는다.

## Approved Policy And Evidence

| ID | 승인 내용 | 근거 |
| --- | --- | --- |
| GRB-IRA-SCOPE-001 | 실제 코드를 읽고 최신 정책 구현의 작업량과 난이도를 평가한다. | 2026-08-12 사용자 요청 |
| GRB-IRA-SCOPE-002 | 추가로 결정해야 하는 정책을 식별하고 문서화한다. | 2026-08-12 사용자 요청 |
| GRB-IRA-SCOPE-003 | Goal 작성과 실행은 goal_authoring_and_execution_guide.md를 따른다. | 2026-08-12 사용자 요청 |
| GRB-IRA-SCOPE-004 | 이번 단계에서는 production을 수정하지 않는다. | 선행 정합성 감사와 이번 단계의 검토 목적 |
| GRB-IRA-SCOPE-005 | 구현 준비도 평가 방법은 policy_implementation_readiness_review_guide.md를 따른다. | 2026-08-12 재사용 가이드 정립 요청 |
| GRB-IRA-BOOTSTRAP-006 | 최소 encounter-acquisition 구동 fixture는 existing 29 source mapping과 Elite 5, NormalBoss 10, FinalBoss 20 fragment dust를 사용한다. | 2026-08-12 사용자 승인 (`R1-continuity`), 이후 최종 콘텐츠 아님을 명시 |
| GRB-IRA-BOOTSTRAP-007 | 최소 Fixed Standard fixture는 `ST1-NT60`: 기존 3개 topology, F2 Porcubbus, F3 Nothing There boss-only HP/ATK/DEF/MR exact override 720/53/36/33(base 60%), scripted fragment 없음과 나머지 ST1 값을 사용한다. | 2026-08-12 사용자 승인 및 이후 최종 콘텐츠 아님을 명시 |
| GRB-IRA-POLICY-008 | Elite slot은 미숙달 후보를 먼저 사용하고, 같은 Floor에 배정 가능한 서로 다른 미숙달 후보가 부족할 때 남은 slot을 숙달 repeat pool로 채운다. | 2026-08-12 사용자 최종 정책 검토 |
| GRB-IRA-TUNING-009 | Monolith Stage 2/3 초기 stat multiplier는 150%/200%이며 개발 playtest 중 RON에서 조절할 수 있다. | 2026-08-12 사용자 최종 정책 검토 |
| GRB-IRA-BOOTSTRAP-010 | Current one-step WhiteNight chain은 generic BossOmen과 all-FinalBoss record 검증용 유일 live bootstrap이며 최종 WhiteNight 콘텐츠가 아니다. | 2026-08-12 사용자 최종 정책 검토 |
| GRB-IRA-POLICY-011 | 숙달 Elite 소탕은 source fragment count가 0이면 repeat reward 대신 파편을 재지급하며 분쇄→재획득 cycle을 횟수 제한 없이 허용한다. | 2026-08-12 사용자 최종 정책 검토 |

`GRB-IRA-BOOTSTRAP-006/007/010`은 runtime을 빈 참조 없이 실행하고 acceptance test를 작성하기 위한 초기
fixture다. Source roster/pairing, repeat balance, tutorial layout/encounter/tuning/narrative/assets의 최종
content authority가 아니며 별도 content Goal이 policy invariant를 지키면서 교체할 수 있다. 교체 시
final content row 누락 금지, explicit per-source first/repeat authoring, Standard 3-Floor fixed/no-adaptive 및
F3 completion transaction 같은 system invariant는 유지한다. 그 밖의 gameplay 정책은 이 표가 새로
승인하지 않으며 current rulebook과 durable decision record를 조사해 finding별 목표 상태를 판정한다.

## Input Finding Registry

아래 요약은 조사 시작점이며 현재 working tree에서 재검증하기 전 최종 구현 결론으로 사용하지 않는다.

| Finding | 기존 감사 요약 | readiness에서 확인할 핵심 경계 |
| --- | --- | --- |
| GRB-M-001 | Standard가 고정 튜토리얼이 아니라 생성·공유 난이도와 FinalBoss를 사용 | map authoring, generator, difficulty, tutorial completion과 mode transition |
| GRB-M-002 | 제거 대상 research와 pending fragment delivery가 end-to-end로 live | RunState, reward/result, persistence, server, Unity와 legacy cutover |
| GRB-M-003 | 모든 response 완료가 Endless RunComplete를 발생 | completion authority, old ledger dependency와 terminal result |
| GRB-M-004 | tutorial unlock과 영구 개인 기록 authority 부재 | global profile, persistence, mode gate, snapshot과 Unity menu |
| GRB-M-005 | Elite natural/mastered pool이 새 cycle 대신 구 exclusion/weighting 사용 | encounter assignment, run/checkpoint ledger와 mastered identity |
| GRB-M-006 | Monolith Stage 2/3 배율이 룰북 수치와 다름 | policy authority, RON tuning, validation과 balance decision |
| GRB-M-007 | mastered-Elite sweep production path 부재 | action/capability, synthetic result commit, reward/XP와 consumer |
| GRB-M-008 | encounter-grade fragment/repeat-reward ledger 부재 | PVE schema, result transaction, inventory, save와 legacy research removal |
| GRB-M-009 | 일반 Reward node가 fragment를 직접 지급 | reward pool authoring, resolver validation과 source eligibility |
| GRB-M-010 | NormalBoss consume/return/refill cycle 부재 | assignment lifecycle, checkpoint state와 return/resolve semantics |
| GRB-M-011 | BossOmen이 chain별 조건 대신 global awakened-fragment gate 사용 | chain schema, eligibility reason, data validation과 projection |
| GRB-M-012 | FinalBoss chain completion/cycle ledger 부재 | BossOmen run state, forced-boss commit, checkpoint와 records |
| GRB-M-013 | Event/BossOmen source defeat가 generic retry/completion을 사용 | combat source identity, failure transaction, requeue/retain semantics |
| GRB-M-014 | exhausted Monolith failure가 영구 실패가 아님 | attempt exhaustion, progress ledger, later eligibility와 checkpoint |
| GRB-M-015 | 29개 fragment active skill이 direct-attack placeholder | content design, skill/fragment data, runtime delivery와 Unity UX |

## Current Runtime And Live Data Evidence

아래 내용은 선행 alignment audit가 같은 simulator start commit의 당시 working tree에서 확인한
preliminary input이다. 이번 Goal의 최종 evidence로 재사용하지 않고 current path를 다시 읽는다.

- Standard는 generated map/shared difficulty authority와 FinalBoss endpoint를 사용한 것으로 보고됐다.
- abnormality research, pending fragment delivery와 Endless auto-completion legacy가 Core부터 접근 가능한
  Unity consumer까지 남아 있는 것으로 보고됐다.
- tutorial unlock/permanent record, mastered-Elite sweep, encounter-grade fragment/repeat ledger와
  NormalBoss/FinalBoss cycle state는 부재한 것으로 보고됐다.
- Monolith stage tuning과 permanent failure, BossOmen eligibility/source failure, ordinary Reward fragment
  source가 룰북 목표와 다르게 동작한 것으로 보고됐다.
- 29개 formal fragment active skill이 direct BasicAttackModifier placeholder로 남은 것으로 보고됐다.
- 선행 감사에서는 Rust test 1,126개가 통과했고 Unity source/assertion은 읽었으나 Unity runner는
  사용할 수 없었다. 이 결과는 관련 tree나 환경이 달라졌다면 재실행 또는 재판정한다.

Goal 작성 단계에서는 production runtime을 새로 조사하거나 test/probe를 실행하지 않았다.

## Baseline And Evidence Freeze

실행 시작 시 다음을 기록한다.

- simulator, 외부 Unity repository와 필요한 인접 repository의 start commit
- 각 repository의 git status --short와 기존 dirty/untracked 상태
- game_rulebook.md, 입력 MISMATCH_CATALOG.md와 RULEBOOK_CLAIM_MATRIX.md의 hash
- 접근 가능한 consumer, 알려졌지만 접근 불가능한 consumer와 runner 상태
- current runtime canonical, target decision record와 과거 Goal evidence의 문서 등급
- 이전 감사 뒤 code, live data, canonical 또는 consumer가 바뀌었는지와 재감사 범위

현재 worktree가 광범위하게 dirty하므로 HEAD가 아니라 조사 시점의 working tree를 authority로 읽는다.
이 Goal의 새 변경은 이 Goal 디렉터리의 문서로만 제한한다.

## Source Of Truth And Ownership

### Evidence precedence

1. 조사 시점 working tree의 실제 reachable runtime과 authoritative state mutation
2. 그 runtime이 실제 로드하는 production live RON/data
3. 실제 serializer, persistence, game_server mapping과 접근 가능한 consumer
4. 가장 가까운 현재 durable canonical
5. 사용자가 승인한 target decision record 또는 명시적 정책 승인 기록
6. focused/integration test의 setup, action과 assertion
7. 완료 Goal과 과거 감사는 재검증할 evidence pointer

runtime이 최신 목표 정책이라는 뜻은 아니다. finding마다 다음을 분리한다.

- Current actual behavior
- Current durable canonical
- Approved target policy
- Proposed implementation approach

Goal 문서나 평가자의 추천을 승인된 정책으로 승격하지 않는다.

### Ownership boundaries to inspect

- Core world/run state와 command-to-authoritative-mutation 경로
- encounter/map/reward/BossOmen/Monolith/fragment live data와 loader/validation
- checkpoint, active-run store와 global profile persistence lifetime
- Core DTO/result/snapshot producer와 game_server mapping/order
- 접근 가능한 Unity command, parser, store와 presentation gate
- 기존 test가 보호하는 current contract와 새 정책에 필요한 test-first safety net

모든 finding에 모든 계층을 기계적으로 요구하지 않는다. 관련 없는 계층은 이유와 근거를 기록한다.

## Invariants

| ID | Invariant |
| --- | --- |
| GRB-IRA-I01 | production code, live data, schema/DTO, save, fixture/test, canonical, server와 Unity를 수정하지 않는다. |
| GRB-IRA-I02 | 기존 dirty/untracked 사용자 변경을 이동·삭제·정리·되돌리지 않는다. |
| GRB-IRA-I03 | current behavior, current canonical, approved target과 proposed design을 혼합하지 않는다. |
| GRB-IRA-I04 | symbol이나 field 존재가 아니라 command/input부터 authoritative mutation과 consumer까지 실제 path를 읽는다. |
| GRB-IRA-I05 | schema/test fixture와 production live data, dead branch와 reachable branch를 구분한다. |
| GRB-IRA-I06 | 난이도는 구조적 복잡도이며 인일, 일정, 변경 파일 수 또는 코드 줄 수가 아니다. |
| GRB-IRA-I07 | 정책 불확실성을 구현 난이도에 숨기지 않고 별도 decision으로 기록한다. |
| GRB-IRA-I08 | individual finding 점수를 합산해 전체 공수로 표현하지 않고 shared work package를 별도 평가한다. |
| GRB-IRA-I09 | compatibility, fallback, dual-read/write와 migration을 기본 해법으로 가정하지 않는다. |
| GRB-IRA-I10 | 접근하거나 실행하지 않은 consumer/test를 검증한 것으로 표현하지 않는다. |
| GRB-IRA-I11 | 미래 콘텐츠를 위해 placeholder state/schema/DTO를 선행 구현하도록 추천하지 않는다. |
| GRB-IRA-I12 | finding과 결정 질문은 bounded investigation unit 직후 Goal artifact에 기록한다. |
| GRB-IRA-I13 | policy/finding ID, 문서 순서, severity와 difficulty를 구현 순서로 사용하지 않는다. |
| GRB-IRA-I14 | 각 구현 stage 종료 상태가 code/data/save/server/consumer 관점에서 유효해야 하며 다음 stage를 위한 깨진 임시 계약을 허용하지 않는다. |

## Target Policy Disposition

각 finding은 구현 난이도 평가 전에 목표 정책 상태를 다음 중 하나로 판정한다.

| Disposition | 의미 |
| --- | --- |
| approved_target | 최신 룰북 또는 승인 기록이 구현할 목표 의미를 충분히 결정함 |
| clarification_required | 상위 방향은 승인됐지만 acceptance criteria 일부가 결정되지 않음 |
| policy_decision_required | 선택에 따라 사용자-visible 의미나 migration이 달라져 사용자 결정이 필요함 |
| document_correction_candidate | current runtime을 유지하고 룰북을 고칠 가능성을 배제할 수 없음 |
| future_or_dormant | 현재 implementation wave가 아닌 후속 콘텐츠 또는 authoring |
| superseded_or_not_applicable | 더 최신 승인 정책이 finding의 필요성을 제거함 |

approved_target이 아닌 항목은 구현을 전제로 한 확정 점수를 부여하지 않는다. 선택지별 조건부
난이도 또는 범위를 기록하고 어떤 결정이 점수를 바꾸는지 명시한다.

## Difficulty Evaluation Model

### Assessment units

두 수준을 각각 평가한다.

1. Individual finding difficulty: 필요한 shared foundation이 이미 있다고 가정한 추가 구현 복잡도
2. Work package difficulty: 실제 end-to-end cutover와 공통 기반을 포함한 후속 Goal 단위 복잡도

최종 계획에는 work package difficulty를 사용한다. individual score는 비교와 누락 확인용이며 서로
더하지 않는다.

### Dimension scoring

각 단위는 아래 여섯 축을 0부터 3까지 evidence와 함께 평가한다.

| 축 | 0 | 1 | 2 | 3 |
| --- | --- | --- | --- | --- |
| A. Authority/state delta | 변경 없음 | 기존 state/branch의 국소 변경 | 새 typed state 또는 여러 transition | 새 durable authority나 핵심 lifecycle/state machine |
| B. Runtime coupling | 없음 | 한 모듈의 기존 primitive 재사용 | 여러 Core 모듈·transaction 결합 | 여러 authoritative subsystem의 원자적 재구성 |
| C. Persistence/cutover | 없음 | 비영속 값 또는 strict data 교체 | checkpoint/profile/schema 중 하나 변경 | save/version migration, rollback/reconnect 또는 다중 lifetime cutover |
| D. Cross-layer contract | 없음 | 내부 projection만 변경 | Core와 server 또는 단일 consumer 변경 | Core·server·복수 consumer의 coordinated contract 변경 |
| E. Live data/content | 없음 | 기존 schema의 값·membership 변경 | 새 field/reference와 authored migration | 새 catalog/content family 또는 대량 개별 authoring/design |
| F. Verification/regression | 문서·정적 검사 | focused test 한 계층 | 통합·loading·shape test 여러 family | end-to-end, migration/reconnect와 광범위 회귀 |

### Overall difficulty

기본 합계와 등급은 다음과 같다.

| 합계 | Difficulty | 해석 |
| --- | --- | --- |
| 0–3 | 1 | 기존 구조 안의 국소 data/logic 변경 |
| 4–6 | 2 | 한 authority 중심의 제한된 변경 |
| 7–10 | 3 | 여러 모듈 또는 새 state/schema와 통합 검증 |
| 11–14 | 4 | cross-layer 계약이나 durable state cutover가 필요한 변경 |
| 15–18 | 5 | 핵심 lifecycle 재설계, 다중 migration 또는 대규모 content/cross-component 검증 |

다음 minimum rule을 함께 적용한다.

- public wire 계약 또는 기존 save/profile migration이 필요하면 최소 4다.
- 여러 durable lifetime의 atomic cutover와 핵심 run state machine 재설계가 함께 필요하면 5다.
- 정책 미결정이나 필수 consumer 미확인으로 축을 확정할 수 없으면 단일 점수 대신 provisional range를 쓴다.
- minimum rule로 등급이 올라가면 원래 합계와 override 근거를 모두 기록한다.

### Separate risk and confidence

Difficulty와 별개로 다음을 기록한다.

- Implementation risk: low / medium / high / critical
- Confidence: high / medium / low
- Unknowns: 점수를 바꿀 수 있는 미확인 path, consumer, migration 또는 정책

Risk는 데이터 손실, 부분 commit, save 호환성, 사용자 progression과 배포 실패 영향으로 판정한다.
Confidence가 low인 4~5 또는 범위가 두 단계 이상인 항목은 후속 구현 전 별도 read-only 확인이나
승인된 spike가 필요한 것으로 표시한다.

## Finding Assessment Record

IMPLEMENTATION_ASSESSMENT.md에서 각 GRB-M ID는 정확히 한 primary row와 상세 section을 가진다.

필수 필드:

- finding ID와 현재 유효성
- rulebook claim과 target-policy disposition
- observable acceptance criteria
- current command/input→authority→result path
- production live data identity와 loader/assignment
- DTO, server, persistence와 consumer 영향
- existing primitive와 제거할 legacy/fallback
- 예상 production 변경 경계와 work type
- DTO/data/save migration과 rollout 가정
- 필요한 test-first safety net과 broad regression
- A~F dimension score, overall difficulty와 override
- implementation risk, confidence와 unknowns
- linked policy-decision ID
- linked work-package ID와 prerequisites
- item-level readiness disposition

item-level readiness disposition은 ready, ready_with_prerequisite, decision_required,
external_evidence_required, future_or_dormant, not_applicable 중 하나를 사용한다. 이는 Goal의 primary
Status와 구분한다.

## Policy Decision Record

POLICY_DECISION_LOG.md의 각 미결정은 GRB-PD-001 형식의 stable ID를 사용하고 다음을 기록한다.

- 연결된 finding/work package
- 결정할 질문과 결정을 미루면 막히는 범위
- current runtime/live data와 current canonical
- rulebook/decision record가 이미 결정한 부분
- 상호 배타적인 선택지
- 선택지별 사용자-visible 의미
- Core/data/save/wire/Unity와 rollout 영향
- 선택지별 difficulty 변화와 필요한 검증
- 권장안이 있다면 code evidence와 trade-off
- 필요한 사용자 입력과 결정 상태

평가자는 권장안을 제시할 수 있지만 사용자 결정을 대신하지 않는다. 질문을 하나의 거대한
승인 요청으로 합치지 않고 독립적인 사용자-visible 선택 단위로 분리한다.

## Work Package And Dependency Model

WORK_PACKAGE_MAP.md는 finding을 룰북 section이 아니라 같은 authority, migration lifetime과 원자적
완료 조건을 기준으로 묶는다.

각 package는 다음을 가진다.

- package ID와 연결 finding/decision
- 사용자-visible 또는 contract-visible 완료 결과
- current→target authority
- shared foundation과 제거할 legacy
- prerequisite / coimplement / independent / defer 관계
- 포함할 producer, live data, persistence와 consumer
- package-level A~F difficulty, risk와 confidence
- strict cutover 또는 승인 필요한 compatibility
- test-first safety net과 completion audit gate
- 권장 single Goal 또는 Master/Phase 경계

개별 점수 합계로 package 점수를 만들지 않는다. 하나의 기반 변경으로 여러 finding이 함께 닫히는지,
반대로 하나의 finding이 여러 독립 content/contract package로 분리되는지 실제 ownership으로 판단한다.

## Implementation Sequence Audit

IMPLEMENTATION_SEQUENCE.md는 WORK_PACKAGE_MAP.md가 확정한 package를 node로 사용해 안전한 구현 순서를
별도 감사한다.

필수 절차:

1. 모든 current-scope package를 dependency graph에 정확히 한 번 등록한다.
2. policy, authority, schema/data, persistence, producer-consumer, validation, legacy-removal과 external
   edge를 source path와 failure 근거로 기록한다.
3. strongly connected component를 검사해 같은 transaction/cutover package는 coimplement로 합치고,
   정책 또는 scope 선택이 필요한 cycle은 blocker로 분류한다.
4. cycle disposition 뒤 graph를 topological layer로 정렬한다.
5. 같은 layer의 package가 shared authority, schema, save version, live catalog와 consumer를 공유하지
   않는지 확인한 뒤에만 parallel-safe로 판정한다.
6. 각 stage의 activation gate, completion gate, 다음 stage seam과 intermediate-state safety를 기록한다.

각 stage는 다음을 포함한다.

- sequence number와 package ID
- prerequisite와 edge type
- 같은 stage에서 원자적으로 함께 구현할 producer/data/save/consumer
- activation 조건과 사용자-visible completion result
- focused/broad 검증과 completion audit gate
- 다음 stage가 사용할 stable seam
- 병렬 가능 여부와 독립성 근거
- stage 종료 시 code/data/save/server/Unity 계약이 유효하다는 근거
- blocker, rollout/cutover와 재평가 조건

정책 나열 순서, GRB-M 번호, difficulty 또는 severity만으로 sequence를 정하지 않는다. 순환 의존을
placeholder, dual schema, fallback과 temporary duplicate authority로 억지로 순차화하지 않는다.

## In Scope

- GRB-M-001부터 GRB-M-015의 current-tree 유효성 재확인
- current game_rulebook.md 전체와 finding 관련 승인/decision evidence 확인
- command/input부터 authoritative state mutation, live data, persistence와 projection 추적
- 실제 producer type, game_server mapping과 접근 가능한 Unity consumer 확인
- 관련 existing test의 setup/action/assertion 및 production reachability 검토
- finding별 target-policy disposition과 observable acceptance criteria 작성
- individual/work-package difficulty, risk, confidence와 unknown 평가
- 추가 정책 질문, 선택지, 영향과 권장안 작성
- shared foundation, evidence-backed dependency graph, cycle/coimplementation과 후속 Goal boundary 제안
- topological implementation sequence, stage activation/completion gate와 parallel-safety 감사
- 각 stage 종료의 code/data/save/server/consumer intermediate-state safety 검토
- migration, rollout, compatibility와 필요한 test-first safety net 평가
- 독립 completion audit

## Out Of Scope

- production Rust, live RON, schema/DTO, save, server, Unity와 canonical 수정
- gameplay·balance·UX·content 정책의 사용자 승인 대행
- correction, migration, compatibility layer, fixture 생성과 구현 테스트 추가
- 후속 Goal 또는 Master/Phase 자동 생성
- 인일, 달력 일정, 담당자별 속도와 파일/코드 줄 기반 공수 추정
- 각 fragment skill의 구체적인 기믹·수치·art/VFX/animation 제작
- hidden-boss future content처럼 기존 감사에서 명시적으로 제외한 범위의 재도입
- repository-wide 일반 리팩터링 감사와 unrelated dirty file 정리

## Investigation And Artifact Plan

### 0. Durable incremental evidence

- 한 finding, 한 source/consumer path, 한 policy question 또는 한 test family를 bounded unit으로 조사한다.
- 새 evidence와 미결 gap은 다음 unit으로 넘어가기 전에 Goal artifact에 기록한다.
- 명령이나 probe는 정확한 command, 목적, 실제 결과와 한계를 EXPERIMENTS.md에 즉시 기록한다.
- 대화, model context와 과거 요약만 존재하는 관찰은 완료 evidence로 사용하지 않는다.

### 1. Baseline and input revalidation

- repository commit/dirty 상태, 문서 hash, 접근 가능한 consumer와 runner를 고정한다.
- 기존 15개 finding의 code/data anchor가 현재 worktree에서도 유효한지 확인한다.
- finding이 수정·supersede·분할됐다면 원래 ID와 현재 disposition을 보존한다.

### 2. Target-policy and acceptance-criteria pass

- current rulebook을 EOF까지 다시 읽고 finding 문맥과 다른 절의 교차 불변식을 확인한다.
- durable decision record와 사용자 승인 기록을 대조한다.
- 각 finding을 target-policy disposition으로 분류하고 성공·실패·retry·reward·persistence 관점의
  observable acceptance criteria를 작성한다.
- 결정되지 않은 의미는 GRB-PD ID로 즉시 분리한다.

### 3. End-to-end implementation boundary trace

- 실제 current path를 command/data entry부터 authority, result, save와 consumer까지 추적한다.
- 기존 primitive 재사용, 새 state/schema, legacy 제거와 competing authority를 구분한다.
- 관련 없는 계층과 접근 불가능한 consumer도 이유를 기록한다.
- 결과를 IMPLEMENTATION_BOUNDARY_AUDIT.md에 누적한다.

### 4. Difficulty, risk and confidence assessment

- 각 finding에 A~F evidence score와 individual difficulty를 부여한다.
- 결정 또는 외부 evidence가 부족하면 provisional range와 unknown을 기록한다.
- test-first safety net, migration/cutover와 broad regression을 평가한다.
- 결과를 IMPLEMENTATION_ASSESSMENT.md에 기록한다.

### 5. Policy decision ledger

- 미결정 질문을 독립 decision 단위로 정리한다.
- 선택지별 사용자 의미, migration, rollout, difficulty와 검증 차이를 작성한다.
- 이미 승인된 의미를 다시 질문하거나 구현 세부를 gameplay 정책 질문으로 과장하지 않는다.

### 6. Work-package clustering

- 같은 authority와 lifetime을 공유하는 finding을 package로 묶는다.
- prerequisite, coimplementation, 독립 package와 future content를 구분한다.
- package-level difficulty/risk를 새로 평가하고 권장 Goal 경계를 작성한다.
- 기존 15개 finding을 15개 독립 구현 Goal로 기계적으로 변환하지 않는다.

### 7. Safe implementation sequence audit

- work package 전체를 typed dependency graph node로 등록하고 edge 근거를 작성한다.
- strongly connected component를 coimplement package 또는 policy/scope blocker로 disposition한다.
- cycle이 닫힌 graph를 topological layer로 정렬하고 안전한 구현 sequence를 작성한다.
- 같은 layer의 병렬 후보가 authority/schema/save/catalog/consumer 경계를 공유하지 않는지 검토한다.
- 각 stage의 activation/completion gate, 다음 stage seam과 intermediate-state safety를
  IMPLEMENTATION_SEQUENCE.md에 기록한다.

### 8. Coverage and independent completion audit

- 15개 ID, 상세 section, difficulty/readiness/decision/package/sequence 링크의 중복·누락을 검산한다.
- initial/final hash와 scoped status로 production 무변경과 dirty 보존을 확인한다.
- goal_completion_review_guide.md를 investigation Goal에 맞게 적용한다.
- COMPLETION_AUDIT.md에 evidence matrix, cross-component finding, long-term fit, primary verdict와
  Status transition을 기록한다.

## Required Goal Artifacts

- PLAN.md
- EXPERIMENTS.md
- EXPERIMENT_NOTES.md
- IMPLEMENTATION_BOUNDARY_AUDIT.md
- IMPLEMENTATION_ASSESSMENT.md
- POLICY_DECISION_LOG.md
- CONTENT_INPUT_CANDIDATES.md (supplemental minimum-bootstrap manifest; final content authority 아님)
- WORK_PACKAGE_MAP.md
- IMPLEMENTATION_SEQUENCE.md
- COMPLETION_AUDIT.md

긴 code listing, 전체 test log와 JSON dump는 붙이지 않는다. 정확한 source path/symbol, 재현 가능한
명령, 핵심 결과와 artifact 경로를 기록한다.

## Open Policy Decisions

Goal 초기화 시점에는 미확정이다.

이번 Goal의 목적 중 하나가 추가 정책 결정을 찾는 것이므로 정책과 독립적인 read-only 조사는 끝까지
수행한다. material한 질문이 남으면 assessment artifact는 보존하되 Goal을 complete로 표시하지 않고
policy_decision_required로 전환한다. 사용자가 결정한 뒤 영향받는 점수, work package와 validation을
다시 평가해야 complete가 될 수 있다.

다음 유형을 우선 확인한다.

- 룰북의 구현 대기 문구가 현재도 승인된 target인지
- Standard tutorial의 exact authored map/content와 전환 의미
- permanent record의 identity, scope와 persistence lifetime
- encounter fragment/repeat reward의 exact trigger, quantity와 dedupe
- cycle의 identity membership, return/reset와 checkpoint 의미
- sweep의 synthetic victory가 reward/XP/condition/battle record에 미치는 exact 의미
- Event/BossOmen failure의 retry, source retention/requeue와 attempt 소비
- Monolith tuning 수치가 확정 balance인지 data correction candidate인지
- placeholder skill의 완료 단위가 runtime framework인지 29개 개별 content authoring인지
- strict cutover가 가능한지 또는 기존 save/consumer compatibility가 필요한지

이 목록은 질문이 존재한다고 미리 판정하는 것이 아니라 필수 확인축이다.

## Implementation Scope

Production 구현은 없다.

허용되는 write는 이 Goal 디렉터리의 조사·실험·평가·감사 문서뿐이다. 어떤 readiness verdict도
별도 승인된 구현 Goal 없이 production 변경 권한을 부여하지 않는다.

## DTO, Data And Save Migration

- DTO/wire migration: 수행하지 않으며 finding/package별 예상 필요성과 cutover를 평가한다.
- Data/live RON migration: 수행하지 않으며 schema/value/catalog/content migration을 구분한다.
- Save/checkpoint/profile migration: 수행하지 않으며 lifetime, version, strict rejection와 기존
  progression 영향을 평가한다.
- Unity asset/presentation migration: 수행하지 않으며 command/reader/store/UI 영향만 확인한다.

migration 없음은 조사 결과여야 한다. 단순히 이 Goal이 read-only라는 이유로 후속 구현의 migration
비용을 0으로 두지 않는다.

## Deployment And Compatibility

이 Goal에는 배포가 없다.

후속 package마다 다음을 판정한다.

- Core, live data, game_server와 Unity의 atomic deployment 가능 여부
- 기존 active run, checkpoint, global profile와 consumer version 지원 필요 여부
- strict cutover와 명시적 rejection이 가능한지
- compatibility가 필요하다면 대상 버전, 기간, 종료 조건과 전용 test

승인되지 않은 compatibility, fallback, alias와 dual-read/write를 기본 계획에 포함하지 않는다.

## Focused Validation

필수 검증은 readiness 주장의 traceability다.

- GRB-M-001부터 GRB-M-015까지 exact coverage와 ID 중복/누락 검사
- finding별 rulebook target, current runtime/live data, persistence와 consumer path 대조
- A~F dimension 값, 합계, overall difficulty와 override 규칙의 기계적 재계산
- provisional score가 policy decision/unknown과 연결됐는지 검사
- decision ID가 관련 finding/package와 양방향 연결됐는지 검사
- work package가 모든 implementable finding을 포함하고 중복 소유하지 않는지 검사
- dependency graph가 모든 current-scope package를 포함하고 각 edge에 type/evidence가 있는지 검사
- cycle/SCC disposition과 topological layer의 재현 가능성을 검사
- stage activation/completion gate와 intermediate-state safety를 검사
- parallel 후보가 shared authority/schema/save/catalog/consumer를 갖지 않는지 검사
- existing test family별 보호 invariant와 새 test-first gap 확인
- 입력 문서 hash와 initial/final scoped status 비교

Static source만으로 concrete runtime fact를 닫을 수 없고 verdict에 필요한 경우에만 가장 작은 기존
focused test 또는 read-only probe를 실행한다. 정확한 명령과 runner가 보고한 pass/fail/skip 수를
EXPERIMENTS.md에 기록한다.

## Broad Validation

Production 변경이 없는 investigation Goal이므로 전체 cargo test, Unity full suite와 live server
end-to-end probe는 기본 완료 조건이 아니다.

Broad validation은 다음 read-only completeness pass다.

- Core, game_resources, game_server와 접근 가능한 Unity에서 15개 finding 관련 identity/legacy 검색
- current rulebook 전체 heading/문맥과 입력 audit artifact의 stale 여부 검토
- persistence, compatibility/fallback, DTO/consumer와 test gap 독립 검색
- artifact 간 finding, decision, package, sequence와 completion condition traceability 검사
- 제안된 각 stage 종료 상태의 code/data/save/server/consumer 계약 일치와 legacy cutover 검사
- Goal directory 외 production/canonical이 이 Goal로 인해 바뀌지 않았는지 scoped diff 확인

cross-component fact가 static/test-source evidence만으로 불확실하고 completion verdict에 필수면 가장
작은 충분한 package/integration runner를 선택한다. green test 수만으로 구현 난이도나 target readiness를
증명하지 않는다.

## Omitted Validation

- before/after regression suite: production diff가 없으므로 생략한다.
- 새 target-policy behavior test: 아직 구현되지 않았으므로 실행 대상이 아니며 후속 package의
  test-first 요구로 기록한다.
- Unity PlayMode visual/art validation: presentation 구현과 asset 품질은 비범위다.
- balance simulation: 명시적 수치가 current data와 다른지는 확인하지만 적절한 수치를 평가하지 않는다.
- 인일/일정 검증: difficulty는 구조적 복잡도이며 일정 약속이 아니다.

생략 근거가 특정 readiness 판정을 약화하면 그 항목의 confidence를 낮추거나
external_evidence_required로 남긴다.

## Completion Conditions

1. GRB-M-001부터 GRB-M-015가 각각 정확히 한 primary row와 상세 assessment에 연결된다.
2. 각 finding의 current validity, current behavior, current canonical, target-policy disposition과
   observable acceptance criteria가 구분된다.
3. 관련 command/data entry부터 authoritative mutation, live data, persistence, server와 접근 가능한
   consumer까지 실제 경로가 확인된다.
4. 각 finding에 A~F evidence score, individual difficulty, risk, confidence, unknown과 readiness가 있다.
5. 확정할 수 없는 점수는 provisional range로 남고 원인이 decision 또는 external evidence에 연결된다.
6. material policy question은 stable GRB-PD ID, 선택지, 사용자 영향, migration/rollout/test 영향과
   필요한 사용자 입력을 가진다.
7. implementable finding 전체가 authority/lifetime 기반 work package와 연결되고 dependency와
   coimplementation 관계가 기록된다.
8. 각 work package가 별도 A~F difficulty, risk, confidence, migration/cutover, test-first safety net과
   권장 Goal boundary를 가진다.
9. 모든 current-scope package가 typed, evidence-backed dependency graph에 정확히 한 번 포함된다.
10. 모든 cycle/SCC가 coimplement package, policy blocker 또는 scope blocker로 disposition되고 graph의
    topological implementation sequence가 작성된다.
11. 각 stage에 activation/completion gate, 다음 stage stable seam, parallel-safety와 code/data/save/
    server/consumer intermediate-state safety가 기록된다.
12. individual score를 합산하거나 policy/finding/severity/difficulty 순서를 구현 순서로 오인하지 않는다.
13. 접근하지 못한 consumer, 실행하지 않은 runner와 omitted validation을 과장 없이 기록한다.
14. production/canonical mutation이 없고 초기 dirty/untracked 사용자 상태가 보존된다.
15. 독립 completion audit가 모든 evidence 행을 검토하고 명시적 verdict를 기록한다.
16. material한 정책 질문이 없거나 모두 사용자 결정으로 닫혔을 때만 complete로 전환한다. 질문이
    남으면 조사 산출물이 완성됐더라도 Status는 policy_decision_required다.
17. 모든 finding, policy question, 명령과 disposition이 조사 단위 직후 durable Goal artifact에
    누적되고 대화에만 남은 완료 근거가 없다.

## Completion Audit Evidence Matrix

| ID | Completion condition or invariant | Required evidence | Final status |
| --- | --- | --- | --- |
| GRB-IRA-C01 | 15개 finding exact coverage | IMPLEMENTATION_ASSESSMENT.md ID/row/section 검사 | pass |
| GRB-IRA-C02 | current/canonical/target/acceptance 분리 | boundary audit와 assessment | pass |
| GRB-IRA-C03 | end-to-end actual path | IMPLEMENTATION_BOUNDARY_AUDIT.md | pass |
| GRB-IRA-C04 | evidence-based difficulty/risk/confidence | A~F score 재계산과 source links | pass |
| GRB-IRA-C05 | provisional/unknown 정직한 처리 | assessment와 decision/external link 검사 | pass |
| GRB-IRA-C06 | policy decision completeness | POLICY_DECISION_LOG.md | pass |
| GRB-IRA-C07 | work-package coverage와 dependency | WORK_PACKAGE_MAP.md | pass |
| GRB-IRA-C08 | package-level migration/test/Goal boundary | work package detail sections | pass |
| GRB-IRA-C09 | package graph exact coverage와 typed edge evidence | IMPLEMENTATION_SEQUENCE.md graph audit | pass |
| GRB-IRA-C10 | cycle/SCC disposition과 topological sequence | graph cycle/topological validation | pass |
| GRB-IRA-C11 | stage gate, parallel·intermediate-state safety | sequence stage detail sections | pass |
| GRB-IRA-C12 | no false effort/order inference | methodology and final report review | pass |
| GRB-IRA-C13 | consumer/validation limits disclosed | experiments, notes와 assessment | pass |
| GRB-IRA-C14 | production no-mutation and dirty preservation | initial/final hash·status comparison | pass |
| GRB-IRA-C15 | independent completion review | COMPLETION_AUDIT.md verdict | pass |
| GRB-IRA-C16 | open runtime/bootstrap gate respected | all runtime decisions closed; final content explicitly deferred; Status `complete` | pass |
| GRB-IRA-C17 | incremental evidence durability | per-unit artifact history | pass |

## Stop Conditions

- 목표 policy source가 없거나 충돌하여 implementation delta를 편향 없이 정의할 수 없음
- 사용자-visible gameplay, balance, UX, RON, DTO, save/checkpoint, compatibility 또는 rollout 선택이
  material하게 달라짐
- 하나의 readiness deliverable로 닫을 수 없는 독립 campaign이 발견되어 Master/Phase 재구조화 승인이 필요함
- dependency cycle을 안전한 coimplementation으로 닫을 수 없고 policy 또는 scope 선택이 필요함
- 필수 Core/live data/server/Unity repository, consumer, test source, 권한 또는 환경에 접근할 수 없음
- current dirty change가 핵심 path를 중간 migration 상태로 만들어 current와 target을 구분할 수 없음
- readiness evidence를 얻기 위해 production mutation, fixture regeneration, formatter 또는 destructive
  command가 필요함
- target decision record, rulebook 또는 input audit artifact가 조사 중 material하게 변경됨
- 사용자 소유 dirty 파일을 수정·이동·삭제해야만 진행할 수 있음

정책과 독립적인 read-only 조사는 가능한 범위까지 완료하고 evidence를 보존한다. 이후 실제 원인에
맞춰 policy_decision_required, scope_decision_required 또는 externally_blocked로 전환하며 완료로
표현하지 않는다.

> 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
