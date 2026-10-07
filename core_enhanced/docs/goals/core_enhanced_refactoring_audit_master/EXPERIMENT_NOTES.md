# Core Enhanced Refactoring Audit Master — Experiment Notes

Status: complete

## 초기 구조 판단

- 감사 대상은 하나의 기능 계약이 아니라 현재 `core_enhanced` 프로젝트 전체다.
- 서로 다른 state, runtime lifecycle, DTO, persistence, data-loading과 test 수명주기가 존재할 수 있으므로
  하나의 단일 Goal에서 모든 evidence를 혼합하지 않고 Master/Phase 구조를 사용한다.
- Master는 순서, coverage, 공통 판정 기준과 최종 catalog만 조정한다.
- 실제 read-only code/data/test 조사는 승인되어 활성화된 Phase Goal만 소유한다.
- 후속 Phase를 디렉터리나 파일 크기만으로 미리 나누지 않는다. Phase 0에서 실제 source-of-truth
  경계를 inventory한 뒤 사용자에게 분할·이름 승인을 받는다.

## 승인된 실행 경계

- Master Goal: `core_enhanced_refactoring_audit_master`
- Primary scope: current working tree의 `core_enhanced`
- Audit standard: current working-tree
  `core_enhanced/docs/engineering_workflow.md#리팩토링-판단`
- Goal authoring/execution standard:
  `core_enhanced/docs/goal_authoring_and_execution_guide.md`
- Master mode: `orchestration_only`
- Child mode: `investigation_only`
- Production mutation: `no`
- Unity: 명시적 비범위
- `game_resources`/`game_server`: Core 후보 판정에 필수인 경우 supporting evidence만 read-only 확인하며
  그 자체의 리팩토링 후보는 만들지 않음

## Source-of-truth 판단

- `engineering_workflow.md`는 후보 발굴·작업 유형·priority/readiness와 실행 판정 방법을 소유한다.
- 현재 runtime 사실은 `core_enhanced` code와 실제 authoritative mutation이 우선한다.
- Production live data가 runtime 의미에 필요하면 supporting evidence로 확인한다.
- 계약 의미는 `README.md`가 지정하는 가장 가까운 durable canonical에서 확인한다.
- Goal/experiment/archive는 current runtime 또는 정책 source of truth가 아니다.
- 현재 동작, current canonical과 사용자 승인 정책이 다르면 어느 하나를 자동 정답으로 간주하지 않는다.

## 초기 repository와 dirty 상태

- Repository root: `/mnt/f/work/simulator`
- Start commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- Baseline: 2026-08-07 current dirty working tree
- Goal path와 필수 문서 3개는 초기화 전에 모두 존재하지 않았다.

초기 `git status --short`에는 다음 사용자 소유 변경이 함께 존재했다.

- `core_enhanced/docs`: canonical, workflow/review guide와 skill note 수정, 작성 지침·decision·이전 Goal untracked
- `core_enhanced/src`: battle/world/data/employee/behavior 등 다수 수정과 tactical/field-work 파일 untracked
- `core_enhanced/tests`: RON과 skill-runtime test 수정
- `game_resources`: consumable/employee/equipment/reward/PvE/run/skill live RON 수정
- `game_server`: protocol, fixture generator와 player-game actor 수정
- 이전 `defense_route_tactical_command_master`와 Phase 0~9 Goal directory가 untracked
- `core_enhanced/docs/tmp/`가 untracked

이 Master는 위 변경을 만들었다고 주장하지 않고 되돌리거나 정리하지 않는다. Phase별 evidence는 이
dirty baseline 위의 현재 동작으로 판정하되, candidate의 변경 빈도를 Git history가 대표하지 못하면
`unknown`으로 기록한다.

## Audit standard freeze

- Master 초기화 시점의 current working-tree `engineering_workflow.md`를 적용한다.
- 감사 중 이 문서를 후보에 유리하게 수정해 기준을 이동하지 않는다.
- 외부 작업으로 기준이 바뀌면 새 diff를 사용자 소유로 보존하고 영향 Candidate를 재평가할지 기록한다.
- AI가 작성했다는 추정, 문체 불일치와 파일 길이는 단독 evidence가 아니다.

## 적용하지 않은 대안

### 전체 프로젝트를 하나의 단일 조사 Goal로 실행

적용하지 않는다. 독립 source-of-truth 경계의 evidence, priority와 policy question이 섞이고 하나의
완료 감사로 누락 여부를 판정하기 어렵다.

### 폴더 또는 계층별 Phase를 즉시 확정

적용하지 않는다. 동일 계약이 runtime/data/DTO/test를 가로지를 수 있으므로 source-of-truth 경계를
확인하기 전 `src`, `data`, `tests` 같은 모양으로 분할하면 동일 후보가 중복된다.

### Clean worktree 또는 과거 commit을 감사 기준으로 요구

적용하지 않는다. 사용자는 현재 누적 코드베이스를 감사하려 한다. Current dirty tree를 명시적
baseline으로 고정하고 기존 변경을 보존한다.

### 감사 중 안전해 보이는 refactor를 즉시 구현

적용하지 않는다. 이번 campaign은 candidate discovery와 prioritization만 승인됐으며 실제 production
변경은 별도 implement-and-verify Goal과 사용자 승인이 필요하다.

## 외부 시스템과 소비자

- Unity runtime/contract/UI/tests는 비범위이며 읽거나 검증하지 않는다.
- `game_server`와 `game_resources`는 `core_enhanced` authority 또는 consumer/data 의미를 확정하는 데
  반드시 필요한 경우만 supporting evidence로 읽는다.
- Supporting boundary를 읽지 않았거나 접근할 수 없으면 candidate evidence에 `unverified`를 남긴다.
- 범위 밖 component의 독립 결함이나 리팩토링 후보를 이 Master에 흡수하지 않는다.

## Test-code audit policy

- 사용자가 제공한 current baseline은 `core_enhanced` 전체 test 통과다. 이를 확인하기 위한 반복 runner는
  기본적으로 수행하지 않는다.
- Green count는 기존 안전망이 실행된다는 역사적 사실일 뿐, assertion 의미·경계 적합성·중복 여부 또는
  누락된 negative invariant를 증명하지 않는다.
- 각 Phase는 배정된 test source를 production code와 같은 중요도로 읽고 runtime/live data/canonical과
  대조한다.
- Test family disposition은 `retain`, `strengthen`, `merge`, `remove`, `unverified` 다섯 가지다.
- Happy path는 실제 성공 계약, 조립, authoritative state transition과 결과 shape를 고유하게 보호하면
  유지한다. 단순 `Ok`, non-empty 또는 fixture가 생성한 값을 그대로 되받는 assertion은 고유 안전망이
  없으면 강화·병합·제거 후보다.
- Production에서 도달할 수 없는 test-only 경계, 더 강한 test의 약한 반복, private 구현 순서/shape를
  behavior보다 강하게 고정하는 test, 제거된 legacy/fallback을 authority로 보호하는 test를 찾는다.
- `remove` 판정은 해당 test에 고유 invariant가 없고 병합/대체 뒤 동일하거나 더 강한 안전망이 남는다는
  evidence를 요구한다. Test 수 감소 자체는 목적이 아니다.
- 이번 campaign은 investigation-only이므로 test file을 실제로 수정·삭제하지 않고 후속 cleanup/refactor
  후보와 readiness로 기록한다.
- 정적 대조로 해결할 수 없는 구체적인 runtime evidence gap이 있을 때만 좁은 runner/probe를 실행한다.
- CRA-14 도입 전 완료된 Phase 0~3은 다시 열지 않는다. 해당 test surface의 family disposition은 Final
  consolidation M-11에서 보완하며, Phase 4 이후 child는 자체 evidence matrix에 CRA-14를 적용한다.

## 생략한 조사와 검증

Master 초기화 전에는 지침에 따라 repository-wide code search, runtime/data 조사, test, probe,
formatter와 diff 기반 후보 분석을 수행하지 않았다. 따라서 아직 다음을 주장하지 않는다.

- 실제 source-of-truth 경계 수와 Phase 수
- 리팩토링 후보 존재 여부 또는 severity
- baseline test 상태
- Core와 supporting data/consumer 정합성
- 특정 module의 ownership, lifecycle 또는 abstraction 문제

## 현재 정책 질문

Gameplay/runtime 정책 질문은 아직 발견되지 않았다. Phase 0 child 이름은 2026-08-07 사용자의
Master `PLAN.md` 수행 요청에 따라 제안 이름으로 활성화했다. 사용자는 같은 날 현재 Phase 0
경계·순서로 시작하고 앞으로의 Child/후속 Goal 이름은 작업자가 정하도록 명시적으로 위임했다.
Material scope, authority boundary 수 또는 사용자-visible 계약 변경은 이 명명 위임에 포함되지 않는다.

## 남은 위험

- 큰 dirty tree 때문에 과거 commit 기반 change frequency와 blame이 현재 누적 변경을 대표하지 않을 수 있다.
- Supporting live data/server를 비범위로 유지하면 일부 후보의 authority 또는 consumer 판정이
  `unverified`로 남을 수 있다.
- 기준 문서 자체가 current uncommitted change이므로 감사 중 변경되면 판정 기준 drift가 생길 수 있다.
- Project-wide surface inventory가 불완전하면 경계별 Phase와 최종 catalog에 coverage gap이 생긴다.

## 다음 안전한 작업

1. Phase 1 필수 문서 3개를 초기화한다.
2. live data loading/validation authority 경계를 조사하고 candidate/disposition을 기록한다.
3. Phase 1부터 각 boundary investigation을 `Complete / High`로 순차 종료한다.
4. Final catalog consolidation과 Master 자체 완료 감사를 수행한다.

Phase 0 `core_enhanced_refactoring_audit_phase_00_surface_inventory`는 `Complete / High`다. 제안된 다음
Phase 이름 `core_enhanced_refactoring_audit_phase_01_live_data_loading_and_validation`을 확정했고 CRA-13에
따라 활성화한다.

## Phase 1 handoff

- Phase 1 is `Complete / High` with three confirmed candidates.
- `CRA-P01-C02` is a priority correctness candidate: startup preview validation covers baseline difficulty only
  while current canonical requires every authored stage representative.
- `CRA-P01-C01` and C03 are behavior-preserving ownership/change-amplification candidates.
- Exact current data inclusion is confirmed at 23/23; `config/game_balance.toml` remains unverified.
- Phase 2 is now the only active child and consumes published typed data without reopening Phase 1 ownership.

## Phase 2 handoff

- Phase 2 is `Complete / High` with four confirmed candidates.
- `CRA-P02-C01` and C02 are priority test-first correctness candidates for host monotonic current publish and
  checkpoint identity/progression validation.
- `CRA-P02-C03` and C04 are behavior-preserving mapping/progression-encapsulation candidates.
- Strict Core/host V3, fixed map persistence, explicit backup/tombstone behavior and 3-load semantics are confirmed.
- Cross-player singleton-run ownership and admin durability remain policy-deferred; Unity remains unverified.
- Phase 3 is now the only active child. It consumes the confirmed checkpoint carrier but independently audits
  roster/inventory/economy semantics.

## Phase 3 handoff

- Phase 3 is `Complete / High` with five confirmed candidates and fourteen disposed signals.
- `CRA-P03-C01` and C02 are test-first correctness candidates for checkpoint economy-graph validation and atomic
  enhancement material consumption.
- C03~C05 are behavior-preserving transition/grant/HQ transaction consolidation candidates.
- Retained runner counts are historical Phase 3 baseline evidence only; future Phases apply CRA-14 and audit test
  meaning without default reruns.
- Phase 4 is not yet created or active.

## Phase 4 activation handoff

- Phase 4 is the only active child and audits Event/Facility/node sessions plus VN trigger/session lifecycle.
- Phase 2 run/checkpoint and Phase 3 persistent asset staging are consumed as carriers/handoffs, not reopened.
- CRA-14 applies directly: every assigned test family needs an invariant/reachability/assertion/duplication
  disposition, without default runner repetition.
- Phase 5 remains pending until Phase 4 obtains its own `Complete / High` audit.

## Phase 4 completion handoff

- Phase 4 is `Complete / High` with four confirmed candidates and nine disposed signals.
- C01 is the priority publication correction for runtime-invalid VN repeat/source pairs; C02~C04 consolidate HQ
  transaction, Event/Shop paired commits and VN session assembly/queue placement.
- Event/Shop run caches, heterogeneous facility variants and top-level VN checkpoint behavior are retained
  authorities, not generic merge targets.
- CRA-14 is satisfied for Phase 4; one exact Event unit example is mergeable and four missing characterization
  families are recorded as implementation prerequisites.
- Phase 5 activation is now permitted and must independently audit combat preview/setup/handoff.

## Phase 5 activation handoff

- Phase 5 is the only active child.
- It consumes Phase 1 publication, Phase 2 manifested difficulty, Phase 3 roster/economy and Phase 4 Event-combat
  session handoffs without reopening them.
- Phase 1 C02 remains the existing baseline-only startup-preview candidate and must not be duplicated.
- Phase 6 remains pending until Phase 5 is `Complete / High`.

## Phase 5 completion handoff

- Phase 5 is `Complete / High` with five confirmed candidates and twelve disposed signals.
- C01 is the priority correction for serde-lossy preview cache reuse after durable active-run Continue; C02 adds
  publication-time full setup feasibility after Phase 1 C02's stage/seed matrix.
- C03/C04 consolidate preview resolution and typed setup planning; C05 removes no-op mission policy/test shapes.
- Manifested difficulty, preview-fixed waves, armed eligibility and current live setup remain retained authorities.
- Phase 6 is now the only active child and consumes a successfully validated `ActiveBattleSession` handoff without
  reopening preview/setup ownership.

## Final consolidation handoff

- Phase 0~10 are all `Complete / High`; Final is the only active child.
- Phase catalogs contain 53 confirmed candidates before dedupe. Final must preserve source Candidate IDs and merge
  only proven shared root causes.
- CRA-14 for Phase 1~3 is completed here from their assigned source/test families; Phase 4~10 test audits remain
  authoritative inputs and are cross-deduped, not reopened by green counts.
- Candidate score and readiness remain separate. Policy/external gates cannot be promoted to implementation-ready.
- Final may write only Final/Master audit documents.

## Campaign close

- Final consolidation is `Complete / High`; no active child remains.
- The executable backlog is `FINAL_CANDIDATE_CATALOG.md` in the Final child, with dependencies and policy/external
  gates. The test backlog is `TEST_CLEANUP_CATALOG.md`.
- This campaign remains investigation-only. Any implementation requires a separately authorized Goal and must use
  the candidate-specific safety net and compatibility decision.
