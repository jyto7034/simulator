# Game Rulebook Policy Implementation Master Goal

Status: complete
Implementation Complete: true
Goal Type: master
Execution Mode: orchestration_only
Production Mutation Authorized: no; production mutation belongs only to one separately activated Phase Goal
Repository Root: `/mnt/f/work/simulator`
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
Created: 2026-08-12
Completed: 2026-08-12
Active Phase: none
Next Phase Candidate: none
Maximum Concurrent Production Phases: 1
Pending Policy Decision: none at Master authoring; consumer scope is rechecked at each public-contract Phase activation

## Overall Objective And Completion Conditions

완료된 Preflight가 승인한 게임 룰북 정책을 네 개의 독립 production Phase로 순차 구현한다. 각 Phase는
자기 source-of-truth, migration 수명주기, target-first 검증과 독립 완료 감사를 소유한다. 이 Master는
정책 coverage, dependency, activation, Phase 사이 불변식과 최종 cross-Phase 감사를 소유하며 production을
직접 변경하지 않는다.

Master는 다음 조건을 모두 만족할 때만 완료된다.

1. `P01`~`P04`가 각각 `Complete / High` 자체 완료 감사를 통과한다.
2. `GRB-PD-001`~`010`과 `GRB-M-001`~`014`가 아래 coverage대로 production에서 검증된다.
3. `GRB-PD-007`/`GRB-M-015`/`GRB-WP-005`가 미래 범위로 명시적으로 남고, 현재의 29개
   `BasicAttackModifier` placeholder가 이번 완료 주장을 막지 않는다.
4. active-run V4 전환, successor profile migration, fixed Standard, encounter progression/cycle,
   BossOmen, mastered Elite sweep과 재획득 사이클이 서로 모순 없이 함께 동작한다.
5. Core, live RON, game_server producer/mapping/store가 최종 broad validation을 통과하고, 승인되지 않은
   research/pending, V3 fallback, dual reader/writer 또는 ignored-field compatibility가 남지 않는다.
6. 최종 canonical이 실제 구현 authority와 일치하며 Goal 문서는 실행 evidence로만 남는다.
7. Master completion audit가 Phase별 통과 여부뿐 아니라 cross-Phase state/data/save/DTO/consumer 경계를
   독립적으로 재검증해 `Complete / High`를 판정한다.

## Approved Policy Package And IDs

### Required evidence inputs

- [Goal authoring and execution guide](../../goal_authoring_and_execution_guide.md)
- [Game Rulebook](../../game_rulebook.md)
- [Readiness decision log](../game_rulebook_policy_implementation_readiness_audit/POLICY_DECISION_LOG.md)
- [Readiness work-package map](../game_rulebook_policy_implementation_readiness_audit/WORK_PACKAGE_MAP.md)
- [Readiness implementation sequence](../game_rulebook_policy_implementation_readiness_audit/IMPLEMENTATION_SEQUENCE.md)
- [Preflight PLAN](../game_rulebook_policy_implementation_preflight/PLAN.md)
- [Preflight matrix](../game_rulebook_policy_implementation_preflight/PREFLIGHT_MATRIX.md)
- [Preflight baseline report](../game_rulebook_policy_implementation_preflight/BASELINE_AND_RUNNER_REPORT.md)
- [Preflight completion audit](../game_rulebook_policy_implementation_preflight/COMPLETION_AUDIT.md)

Completed Preflight verdict는 `Complete / High`, lifecycle gate는 `ready_to_activate`, 첫 package handoff는
`ready_for_wp002`다. 이 verdict는 production mutation이나 Phase activation을 자동 승인하지 않는다.

| Package | Approved responsibility | Master disposition |
| --- | --- | --- |
| `GRB-WP-002` | Monolith Stage 2/3 stat tuning과 safe derivation | `P01`, first ready Phase |
| `GRB-WP-001` | encounter progression, reward-source ledger, cycles, BossOmen, active-run V4 atomic cutover | `P02`, after P01 audit |
| `GRB-WP-003` | fixed Standard, successor permanent profile, Endless gate/records | `P03`, after P02 audit |
| `GRB-WP-004` | mastered Elite typed sweep vertical slice | `P04`, after P03 audit |
| `GRB-WP-005` | dedicated fragment skills | future/dormant; excluded from this Master |

## Common Source Of Truth And Invariants

### Evidence and authority order

- Target gameplay meaning: current `game_rulebook.md` and closed `GRB-PD-001`~`010`.
- Current implementation fact: activated Phase가 확인한 runtime code, live RON, DTO producer, server mapping과
  persistence mutation path.
- Execution boundary: completed Preflight matrix/baseline and this Master registry.
- Validation evidence: activated Phase의 exact test results and completion audit.
- Durable authority after implementation: closest runtime/data/save/wire canonical. Master와 Phase 문서는
  durable gameplay authority를 대신하지 않는다.

Current code가 target과 다르면 구현 gap으로 취급한다. Current fact와 target policy가 material하게 바뀌면
해당 Phase를 시작하지 않고 change control에 따라 policy/readiness/preflight owner로 되돌린다.

### Cross-Phase invariants

- Run-local encounter assignment, attempts, source completion, mastery, reward, inventory, cycle, BossOmen chain,
  node/session과 checkpoint state는 각 command의 승인된 원자 경계 안에서 함께 commit/rollback한다.
- Permanent profile의 tutorial flag와 monotonic records는 checkpoint load, active-run rollback, failure 또는
  new run으로 감소하지 않는다. `loads_used`도 이전 값으로 rollback하지 않는다.
- Direct eligible victory의 source 미완료 분기는 정확히 한 fragment, 완료 분기는 정확히 repeat reward다.
  두 분기가 동시에 실행되거나 암묵적 fallback으로 대체되지 않는다.
- Mastered sweep에서 source 완료 + 보유 수량 0이면 같은 fragment 하나를 재획득하고, 수량 1 이상이면
  repeat reward만 적용한다. 마지막 복사본 분쇄와 재획득의 반복 사이클은 허용되며 mastery, source,
  upgrade와 awakening progress는 보존된다.
- Elite는 같은 Floor에서 identity를 중복하지 않고 unmastered를 우선한다. 부족한 슬롯만 mastered cycle로
  채우며 unvisited/failed 결과가 임의의 Normal identity로 downgrade되지 않는다.
- NormalBoss와 FinalBoss는 authored cycle과 source identity를 사용한다. Ordinary Reward node는 target
  fragment나 제거 대상 research를 지급하지 않는다.
- Active FinalBoss chain은 포기, 교체 또는 pool 반환이 불가능하다. 후속 step 실패는 같은 step을 다음
  Floor에도 유지하며, FinalBoss defeat는 공통 run-failure 경계로 간다.
- BossOmen은 per-chain 조건, dependency DAG, cycle과 fail-closed validation을 사용한다. sole live chain은
  개발용 one-step WhiteNight bootstrap이며 synthetic chains는 test-only다.
- Standard는 fixed `ST1-NT60` 3-Floor bootstrap이다. F3의 reduced Nothing There는 `NormalBoss`이고, 승리
  completion과 `standard_tutorial_completed: true`는 crash-recoverable host boundary에서 함께 publish된다.
- Endless는 tutorial flag가 true일 때만 시작하며 ledger, FinalBoss 또는 hidden-boss completion만으로
  `RunComplete`가 되지 않는다.
- Initial Monolith Stage 2/3 stat multiplier는 live RON `150/200`이다. 이는 개발 중 조정 가능한 값이나,
  positive/ordered/exactly representable/no-saturation validation을 항상 만족해야 한다. Reward multiplier,
  wave budget, extra wave와 rounding 의미는 P01에서 변경하지 않는다.
- Active-run은 V4로 한 번에 전환하고 V3와 그 이전 payload를 strict reject한다. Profile V1은 같은 의미의
  네 set만 보존하며 successor의 새 flag/record는 false/0으로 시작한다.
- 승인 없는 dual-read/write, semantic transform, ignored-field fallback, compatibility shim과 placeholder
  production state/schema 선행 구현은 금지한다.
- 이 프로젝트는 개발 중이며 public release gate는 두지 않는다. 그렇더라도 Phase 내부 atomicity,
  runnable baseline과 contract evidence는 생략하지 않는다.
- Final content, balance polish, 다단계 WhiteNight content와 전용 fragment skill은 이 Master의 완료 조건이
  아니다. 승인된 최소 bootstrap만 구현한다.

## Master In Scope / Out Of Scope

### In scope

- 네 production Phase의 이름, dependency, status와 activation/termination 기록
- 10개 policy decision, 14개 현재 finding과 네 package의 exact Phase coverage
- 공통 state lifetime, atomicity, version/migration과 compatibility 금지 규칙
- Phase별 baseline, target-first test, focused/broad validation과 독립 완료 감사 gate
- 모든 Phase 뒤 Core/live-RON/server/canonical cross-Phase validation과 Master completion audit
- Phase 활성화 시 public consumer boundary 재확인 및 미검증 범위의 명시적 기록

### Out of scope

- Master 문서에서 직접 수행하는 production code/data/test/canonical/server/Unity 변경
- 최종 콘텐츠 roster, 밸런스/플레이테스트, tutorial polish와 art/VFX/animation
- `GRB-WP-005` 전용 fragment skill 구현
- 현재 Unity 개발 트리의 정비, Preflight에서 생략된 Unity source/runner 재검사
- unrelated dirty worktree cleanup/refactor
- 특별 sweep item/cost 또는 승인되지 않은 신규 gameplay rule
- public release gate와 과거 완료 Goal 문서의 재작성

Unity 제외는 compatibility 승인이나 consumer correctness 주장이 아니다. Public DTO/wire를 바꾸는 Phase는
활성화 시 자신의 consumer 범위를 다시 기록한다. Unity가 계속 제외되면 Core/live-RON/game_server 개발
경계만 완료했다고 한정하고 Unity 통합을 주장하지 않으며, ignored field나 fallback을 추가하지 않는다.

## Policy Coverage Matrix

### Decision coverage

| Policy ID | Approved meaning | Production Phase | Completion owner |
| --- | --- | --- | --- |
| `GRB-PD-001` | active-run V3 strict reject; no transform/dual reader | P02 | P02 audit + Master migration audit |
| `GRB-PD-002` | Profile V1 same-semantic sets preserved; new fields false/0 | P03 | P03 audit + Master migration audit |
| `GRB-PD-003` | explicit stable reward-source identity | P02 definition/runtime; P04 sweep consumption | P02/P04 audits |
| `GRB-PD-004` | exact 29-row fixture and dust 5/10/20 | P02 authoring/runtime; P04 sweep branch | P02/P04 audits |
| `GRB-PD-005` | sole live one-step WhiteNight bootstrap | P02 | P02 audit |
| `GRB-PD-006` | fixed `ST1-NT60` minimum tutorial bootstrap | P03 | P03 audit |
| `GRB-PD-007` | dedicated fragment skills deferred | no current Phase | Master exclusion audit |
| `GRB-PD-008` | unmastered-first mixed Elite fill | P02 | P02 audit |
| `GRB-PD-009` | initial RON Monolith stats `150/200` | P01 | P01 audit |
| `GRB-PD-010` | unlimited mastered-sweep dismantle/reacquire cycle | P02 ledger; P04 sweep | P02/P04 audits |

### Finding coverage

| Finding ID | Target result | Primary Phase |
| --- | --- | --- |
| `GRB-M-001` | fixed 3-Floor Standard, F3 NormalBoss completion | P03 |
| `GRB-M-002` | research/pending path removed end-to-end | P02 |
| `GRB-M-003` | Endless never auto-completes from ledger/boss completion | P02 |
| `GRB-M-004` | monotonic successor profile, tutorial gate and records | P03 |
| `GRB-M-005` | Elite unmastered/mastered assignment cycles | P02 |
| `GRB-M-006` | initial Monolith `150/200` and safe derivation | P01 |
| `GRB-M-007` | mastered Elite sweep vertical slice | P04 |
| `GRB-M-008` | explicit first/repeat source ledger | P02 |
| `GRB-M-009` | ordinary Reward cannot grant target fragment/research | P02 |
| `GRB-M-010` | NormalBoss authored cycle | P02 |
| `GRB-M-011` | per-chain BossOmen conditions and fail-closed DAG | P02 |
| `GRB-M-012` | FinalBoss chain completion/cycle | P02 |
| `GRB-M-013` | provisional requeue versus active-step retention | P02 |
| `GRB-M-014` | exhausted remanifestation failure is run-local permanent | P02 |
| `GRB-M-015` | 29 placeholder conversion remains deferred | no current Phase; future `WP005` |

Shared decision IDs are consumed contracts, not duplicate primary implementation ownership. Every current finding has
one primary Phase; future `M015` is deliberately outside the campaign.

## Phase Dependency And Status Table

```text
Completed Preflight / ready_for_wp002
  -> P01 Monolith tuning (WP002)
      -> P02 encounter progression and active-run V4 (WP001)
          -> P03 fixed Standard and successor profile (WP003)
              -> P04 mastered Elite sweep (WP004)
                  -> Master cross-Phase completion audit

Future/dormant: WP005 dedicated fragment skills
```

| Phase | Child Goal name | Package / findings | Hard prerequisite | Coordination status | Activation condition |
| --- | --- | --- | --- | --- | --- |
| P01 | `game_rulebook_policy_implementation_master_phase_01_monolith_tuning` | WP002 / M006 | Preflight `Complete / High`, `ready_for_wp002` | `complete` — 2026-08-12, `Complete / High` | satisfied |
| P02 | `game_rulebook_policy_implementation_master_phase_02_encounter_progression_v4` | WP001 / M002, M003, M005, M008~M014 | P01 `Complete / High` | `complete` — 2026-08-12, `Complete / High` | satisfied; exact V4/source graph seam available |
| P03 | `game_rulebook_policy_implementation_master_phase_03_fixed_standard_and_profile` | WP003 / M001, M004 | P02 `Complete / High` | `complete` — 2026-08-12, `Complete / High` | satisfied; fixed ST1, PlayerProfile V2 and completion journal seam available |
| P04 | `game_rulebook_policy_implementation_master_phase_04_mastered_elite_sweep` | WP004 / M007 | P02 semantic foundation and P03 `Complete / High` serialization gate | `complete` — 2026-08-12, `Complete / High` | satisfied; typed battle-independent sweep/reacquire seam available |

P04의 semantic prerequisite는 P02지만, Preflight가 확인한 shared public-file/current ownership 충돌을 피하기
위해 P03 뒤에 직렬화한다. 두 production Phase를 병렬 활성화하지 않는다. Child directory는 Phase가
실제로 활성화될 때만 필수 문서 3개와 함께 만든다.

### P01 contract boundary

- Initial stat 값을 `150/200`으로 바꾸고 loader가 zero/reversed/non-representable/saturating composition을
  reject하도록 한다.
- Every applied Monolith stat derivation과 live bundle load를 검증한다.
- Reward multiplier, generated wave budget, extra waves, rounding과 public DTO/save를 변경하지 않는다.

### P02 contract boundary

- Core/data/active-run V4/store/server producer mapping을 하나의 runnable cutover로 구현한다.
- Research/pending authority를 제거하고 29 source rows, Elite/NormalBoss/FinalBoss cycles, first/repeat ledger,
  Monolith progression, per-chain BossOmen DAG/failure와 WhiteNight bootstrap을 함께 닫는다.
- `EL`, `MON`, `NB`, `BO`, `AT`, `TX`, `CP`, `MIG-01`~`03` matrix를 target-first로 검증한다.
- V3 transform, dual authority 또는 package 일부만 runnable한 중간 완료를 허용하지 않는다.

### P03 contract boundary

- Fixed `ST1-NT60`, 세 tutorial reward pool과 F3 boss-actor-only reduced Nothing There를 author한다.
- Successor permanent profile/store, bounded V1 migration, Endless tutorial gate와 monotonic records를
  Core/server projection까지 함께 구현한다.
- Standard completion과 tutorial flag가 crash/restart에도 둘 다 보이거나 둘 다 보이지 않는 publication
  boundary를 증명한다.

### P04 contract boundary

- Mastered Elite에서만 direct/sweep typed capability와 command를 제공한다.
- Battle setup/simulation을 만들지 않는 synthetic victory planner/result를 사용하고 base reward, exclusive
  reacquire/repeat branch, living roster XP, active-consumable duration, repeat cursor와 node/session을 한
  transaction으로 commit한다.
- Condition, damage, trauma, bonus, discovery, event log, stats와 battle record side effect가 없음을 증명한다.
- Dedicated sweep item/cost를 추가하지 않는다.

### Activation and transition rule

1. Master registry에서 선행 Phase와 handoff를 확인한다.
2. Child 경로 collision, current dirty ownership과 Phase별 baseline을 다시 확인한다.
3. Child의 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`를 production 조사 전에 생성하고 Master 입력,
   범위, baseline, omitted validation과 stop condition을 기록한다.
4. Master에서 해당 Phase 하나만 `active`로 전환한다. `ready`는 구현 권한이 아니다.
5. Child가 `Complete / High` audit를 통과하면 실제 결과와 remaining risk를 Master experiments/notes에
   기록하고 다음 Phase만 `ready`로 전환한다.
6. Phase가 block/replan되면 다음 Phase를 활성화하지 않는다.

## Shared Migration And Version Strategy

| Phase | Version/data change | Required strategy | Forbidden strategy |
| --- | --- | --- | --- |
| P01 | live RON tuning only | strict load validation and unchanged adjacent semantics | code fallback/default that hides invalid RON |
| P02 | active-run Core/server V3 → V4 | one atomic writer/reader/store/DTO cutover; V4 exact round trip and corruption rejection | V3 semantic transform, dual reader/writer, ignored research/pending fields |
| P03 | compendium Profile V1 → successor profile | one-shot bounded migration preserving four same-semantic sets; new fields false/0; current/backup/restart proof | permanent V1 reader, partial profile reset, checkpoint rollback of profile |
| P04 | typed sweep action/capability/result | additive producer/server mapping in one Phase; persistence-before-publication and reconnect idempotence | unknown-action fallback, fake battle path, partial producer-consumer publication |

Phase migration evidence는 writer-produced fixture, current/backup/corrupt/interrupted-write/restart, missing/extra
field, one-edge graph corruption과 checkpoint restore를 해당 boundary에 맞게 포함한다. P02가 소유한 V4 state를
P03/P04가 확장해야 한다면 같은 V4 의미 안의 승인된 additive schema인지 먼저 증명한다. 그렇지 않으면
새 version/policy decision 없이 진행하지 않는다.

## Phase Completion Audit Gate

각 Phase는 다음 gate를 모두 통과해야 한다.

- Required child documents 3개가 current execution evidence를 포함한다.
- Activation 직전의 current-tree baseline과 dirty ownership이 기록되고 unexplained failure가 없다.
- Preflight의 해당 transition/bootstrap/migration/consumer rows가 implementation test와 연결된다.
- Focused tests가 non-zero로 실행되고 실제 pass/fail/skip 수와 exact command가 기록된다.
- 필요한 full Core, live RON integration과 game_server broad suite가 실행된다.
- Runtime/data/save/DTO/server producer-consumer가 같은 target authority를 사용하며 stale legacy path가 없다.
- 승인된 omitted validation과 범위 한계, 특히 Unity consumer claim 여부가 명시된다.
- `goal_completion_review_guide.md` 기반 독립 audit가 completion conditions, scoped diff, canonical alignment,
  compatibility와 remaining risk를 검토하고 `Complete / High`를 판정한다.
- Child status가 complete이고 `Implementation Complete: true`일 때만 Master가 다음 Phase를 ready로 만든다.

Preflight의 Core 1063/server 63/focused 17 green 결과는 작성 시점의 실행 가능성 증거일 뿐 Phase activation
baseline이나 완료 증거를 대체하지 않는다.

## Cross-Phase Validation

P04 완료 후 Master는 production 수정 없이 최소 다음을 최종 현재 tree에서 재검증한다.

- Full Core와 game_server suite, production live-RON bundle/integration runner 및 Phase별 focused suite
- Initial `150/200`과 safe derivation, 29-row exact source authoring, dust 5/10/20, one live WhiteNight chain과
  fixed ST1 bootstrap referential integrity
- Research/pending acquisition/state/DTO/store authority, V3 fallback/reader/writer, adaptive/generated Standard,
  global BossOmen fallback과 sweep fake-battle path가 제거됐는지 static/runtime 확인
- V4 save/checkpoint exact restore와 corruption corpus, successor profile V1 migration/recovery, monotonic records,
  Standard completion/tutorial atomic publication
- Direct/repeat/reacquire branch exclusivity, dismantle-reacquire repetition, cycle refill, unvisited/failure/retreat,
  late rollback, checkpoint load와 reconnect idempotence
- Core DTO producer와 game_server mapping/store의 exhaustive shape/order/durability 일치
- Canonical docs와 production runtime/data/version constants의 일치
- Dirty worktree에서 Master/Phase 소유 변경만 attribution되고 unrelated user work가 보존됐는지 확인

Unity source/runner는 이 Master 작성 시 검사하지 않는다. Final audit는 각 public-contract Phase가 기록한
consumer disposition을 모아 Unity를 검증했다고 과장하지 않는다. 사용자가 later Phase에 Unity를 포함하면
그 Phase가 exact runner/probe evidence를 추가하며, 계속 제외하면 별도 future integration risk로 남긴다.

## Master Completion Audit

모든 production Phase가 `Complete / High`일 때 Master는 별도 `COMPLETION_AUDIT.md`를 작성한다. Audit은
Phase status 표를 신뢰하는 데 그치지 않고 다음을 독립적으로 판정한다.

- Policy/finding/package coverage에 누락, 중복 primary owner 또는 future-scope 혼입이 없는가
- Cross-Phase source-of-truth, state lifetime, transaction과 migration이 실제 production에서 일치하는가
- Legacy/compatibility 검색과 target-first/broad runner가 completion claim을 직접 지지하는가
- Bootstrap를 final content로 과장하지 않고 승인된 최소 콘텐츠만 구현했는가
- Unity 및 다른 omitted consumer/validation claim이 실제 evidence와 정확히 일치하는가
- Canonical, scoped diff, dirty preservation과 remaining risk가 정직하게 기록됐는가

모든 항목이 통과하고 correction work가 남지 않을 때만 `Status: complete`,
`Implementation Complete: true`, `Active Phase: none`, `Next Phase Candidate: none`으로 전환한다. Material
correction이 필요하면 Master가 직접 수정하지 않고 owning Phase를 재개하거나 별도 승인된 correction Goal을
사용한다.

## Stop Conditions

- 승인된 rulebook/PD와 current Phase가 material하게 충돌하거나 새 사용자-visible policy가 필요함
- Phase boundary를 지키면 runnable/atomic production state를 만들 수 없고 boundary 변경이 coverage를 바꿈
- Save/profile version, migration 또는 compatibility가 위 승인 전략 밖의 새 결정을 요구함
- Required source/content reference가 사라지거나 silent substitute/default가 필요함
- Current dirty hunk와 Phase target hunk가 겹쳐 사용자 변경을 안전하게 보존할 수 없음
- Public producer/server/consumer를 부분 publish하거나 fallback 없이는 진행할 수 없음
- Required non-zero runner가 승인 범위 안에서 접근 불가능하거나 unexplained baseline failure가 남음
- P02가 WhiteNight bootstrap reachability, per-chain DAG 또는 fully runnable atomic V4 cutover를 증명할 수 없음
- P03가 F3 completion/profile publication을 crash-recoverable하게 만들 수 없음
- P04가 special item/cost, fake battle state 또는 승인되지 않은 side effect를 요구함
- 다음 Phase를 시작하려 하지만 선행 Phase의 독립 `Complete / High` audit가 없음
- Final content, WP005, Unity 정비나 unrelated refactor를 이 Master에 흡수해야만 진행 가능함
