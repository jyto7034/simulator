# Game Rulebook Policy Implementation Master Phase 02 — Encounter Progression V4

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes; limited to this atomic Phase contract
Master Goal: `../game_rulebook_policy_implementation_master/`
Phase: P02
Package: `GRB-WP-001`
Primary Findings: `GRB-M-002`, `003`, `005`, `008`, `009`, `010`, `011`, `012`, `013`, `014`
Policy Decisions: `GRB-PD-001`, `003`, `004`, `005`, `008`, `010`
Activated: 2026-08-12
Repository Root: `/mnt/f/work/simulator`
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
Direct Dependency: P01 `Complete / High`; Preflight `ready_after_wp002_completion_audit`
Consumer Scope: Core + live RON + game_server; Unity explicitly excluded under current user direction

## Objective

기존 run-local abnormality research/pending-delivery authority를 제거하고, explicit authored reward-source
identity를 기반으로 Elite/Monolith/NormalBoss/FinalBoss cycle, first/repeat reward, BossOmen chain과 checkpoint를
하나의 encounter progression authority로 구현한다. 이 graph를 exact Active Run V4 payload, game_server
store/mapping과 함께 원자적으로 전환하며 V3 이하를 strict reject한다.

## User/Contract-Visible Problem

현재 runtime은 research/pending delivery, generated encounter weighting, global BossOmen eligibility와 V3
active-run graph를 사용한다. 승인된 target은 특정 encounter source의 first/repeat 보상, unmastered/mastered
cycles, 실패/미방문/재시도, active FinalBoss retention과 permanent run-local failure를 하나의 명시적 ledger로
소유해야 한다. 일부만 바꾸면 Core state, live data, save, server projection이 서로 다른 authority를 갖게
되므로 P02는 fully runnable atomic cutover로 닫아야 한다.

## Audit Finding ID

- M002 research/pending removal
- M003 Endless non-completion
- M005 Elite assignment/mastered cycle
- M008 first/repeat source ledger
- M009 ordinary Reward exclusion
- M010 NormalBoss cycle
- M011 BossOmen per-chain conditions/DAG
- M012 FinalBoss chain/cycle
- M013 provisional requeue versus active-step retention
- M014 exhausted Monolith remanifestation failure

## Current Runtime And Live Data Evidence

Completed Preflight evidence, to be revalidated before mutation:

- Current Core/server active-run version is V3.
- Checkpoint/DTO/result/store still expose abnormality research and pending delivery.
- Current encounter selection and BossOmen use legacy weighting/global eligibility rather than target cycles/DAG.
- Explicit `reward_source_id`, `fragment_id` and repeat authoring are absent from production encounters.
- All 29 approved encounter/abnormality/fragment identities exist; one live one-step WhiteNight chain exists.
- Core 1063/server 63 were green at Preflight; P01 final Core is 1065 green.
- Relevant Core/RON/server tree is broadly dirty with user work and must be path/hunk classified.

## Source Of Truth And Ownership

- Gameplay meaning: current `game_rulebook.md`, closed decisions and Preflight transition matrix.
- Encounter source/class/repeat data: production PVE/BossOmen/reward live RON and strict loaders.
- Run-local progression/cycle/source/mastery/checkpoint authority: one Core state graph.
- Result planning/atomic commit: Core world transaction; DTO is projection, not secondary state.
- Active-run V4 bytes/checksum/current-backup publication: game_server store, exact Core V4 payload.
- Core DTO producer: Core typed behavior/action/result/snapshot.
- Transport mapping/order/durability: game_server actor state/handlers.
- Unity is not a P02 verified consumer and no ignored-field/fallback is allowed for it.

## Invariants

- Run-local assignment, attempt, source completion, victory/mastery, reward/inventory, cycle, BossOmen, node/session
  and checkpoint participants commit or roll back as one approved transaction.
- Direct source first victory grants exactly one authored fragment; subsequent direct victory grants exactly the
  authored repeat reward. Direct combat never uses sweep reacquisition.
- Ordinary Reward node/pool cannot grant target fragments or research; invalid authoring fails publication.
- Elite manifestation fills distinct unmastered identities first; mastered shortage uses a deterministic distinct
  cycle. Unvisited identities return to the current candidate set and same-Floor duplicates are forbidden.
- Natural Elite victory is count 1; Monolith Stage 2 and 3 are counts 2 and 3. Exhausted Stage 2/3 failure is
  run-local permanent for that identity and never unlocks mastery/reward/sweep.
- P01's 150/200 checked stat contract remains unchanged.
- NormalBoss uses an authored deterministic cycle. Unvisited returns; exhausted defeat consumes current-cycle
  membership without victory reward and can return after refill.
- BossOmen evaluates per-chain min Floor, optional exact fragment/awakening and dependency DAG with no global
  awakened fallback. Unknown/cyclic/unreachable data rejects publication.
- Provisional first source may requeue. Once first source succeeds, one active chain cannot be abandoned/swapped;
  later source failure retains the same step on the next Floor.
- FinalBoss victory commits source branch, chain completion/cycle and Gate visibility without advancing Floor or
  completing Endless. FinalBoss defeat enters common run-failure; retreat restores pre-entry.
- Sole live membership is the one-step WhiteNight development bootstrap. Dormant bosses do not enter cycle/all-boss
  denominator; synthetic multi-chain fixtures are test-only.
- Endless never emits `RunComplete` from ledger, all normal FinalBoss or hidden-boss completion.
- Checkpoint load restores the complete V4 run-local graph before mutation and increments monotonic `loads_used`
  exactly once; permanent profile remains outside rollback.
- V4 is one exact Core/server reader/writer. V2/V3/unknown, missing/extra/research/pending payloads reject without
  mutation; no transform, dual reader/write, alias or ignored field.
- Result/response publication occurs only after required durable state succeeds; reconnect cannot duplicate or
  invent reward/cursor/node state.

## Approved Policy And Evidence

- Master P02 contract and migration table.
- Preflight `EL-01`~`08`, `MON-01`~`05`, `NB-01`~`07`, `BO-01`~`12`, `EN-01`, `AT-01`~`04`,
  `RW-01`, `TX-01`, `CP-01`.
- Preflight atomicity rows for direct victory, Monolith Stage 3, FinalBoss, checkpoint and host response.
- `BOOT-01`~`07`, `MIG-01`~`03`, `CON-01`~`05` with Unity disposition above.
- P01 completion audit `Complete / High` and stable 150/200 checked-stat seam.

## Open Policy Decisions

없음 at activation. Implementation이 source identity, reward amount, failure/retreat semantics, V4 migration,
BossOmen eligibility 또는 consumer compatibility에 새 의미를 요구하면 관련 mutation을 중단한다.

## In Scope

- Current Core/RON/server authority, dirty ownership, baseline and exact 29-row bootstrap revalidation
- One encounter progression/cycle/source/mastery graph and typed transition/result projection
- Research/pending state, reward, DTO, store and live-data path removal
- Exact 29 source authoring with Elite 5 / NormalBoss 10 / FinalBoss 20 dust repeat rewards
- Unmastered-first Elite assignment and deterministic mastered refill cycle
- Natural/Monolith progression and exhausted failure semantics, preserving P01 tuning
- Authored NormalBoss cycle
- Generic per-chain BossOmen conditions/DAG/cycle/failure plus sole live WhiteNight bootstrap
- Endless non-completion rule
- Active Run V4 Core payload + game_server envelope/store/mapping one-step cutover
- Target-first transition, failpoint, checkpoint, migration, reconnect and strict live-data tests
- Required current canonical updates and independent `Complete / High` Phase audit

## Out Of Scope

- Fixed Standard and successor permanent profile (P03)
- User-facing mastered Elite sweep command/planner/result (P04); P02 provides ledger/branch primitives only
- Dedicated fragment skills (WP005), final content/balance and extra live BossOmen chains
- Unity source/runner/client migration under current explicit exclusion
- Special sweep item/cost, final tutorial content, unrelated dirty refactor/cleanup

## Implementation Scope

P02는 Core state/data/save/DTO와 server store/mapping을 partial-release 없이 함께 구현한다. 내부 작업은
test-first bounded units로 나눌 수 있으나 Phase completion 시에는 다음 vertical graph가 하나여야 한다.

```text
strict live encounter/BossOmen data
  -> deterministic manifestation/cycle/source state
      -> staged result and atomic Core commit
          -> exact V4 checkpoint payload
              -> game_server durable V4 store
                  -> typed snapshot/result response and reconnect
```

Temporary dual authority, compatibility path 또는 research를 숨긴 채 남기는 중간 완료는 허용하지 않는다.

## DTO/Data/Save Migration

- Active Run V4는 complete encounter source/cycle/chain/mastery/inventory/economy/node graph를 exact restore한다.
- Core payload와 server envelope version을 함께 4로 올리고 writer-produced V3 current/backup을 incompatible로
  reject한다. 기존 bytes는 explicit reset/new run 전까지 손상시키지 않는다.
- V2/V3/unknown, missing/extra field, research/pending field와 one-edge graph corruption은 first mutation 전에
  실패한다.
- Removed research/pending DTO/result/snapshot fields have no alias/default/ignored compatibility.
- game_server mappings are exhaustive; required persistence succeeds before success publication.

## Deployment And Compatibility

Development-only project라 public release gate는 없다. P02의 supported development boundary는 Core, production
live RON과 game_server다. 이 세 component는 하나의 commit에서 V4/current DTO를 사용해야 한다. Unity는
사용자의 current instruction에 따라 검사·수정하지 않으며, 이 omission은 compatibility를 승인하지 않는다.
No server fallback or ignored field is added for an old Unity consumer.

## Focused Validation

- New transition suites for every owned EL/MON/NB/BO/EN/AT/RW/TX/CP row
- Strict 29-row source/repeat data and WhiteNight DAG/bootstrap loader suites
- Active-run V4 exact round-trip and V2/V3/unknown/missing/extra/corrupt/research payload rejection
- Result rollback failpoints and reconnect idempotence
- Core DTO exact serialization and game_server exhaustive mapping/store tests
- Exact commands and actual counts are recorded after current runner discovery.

## Broad Validation

- `cargo test -p game_core -- --quiet`
- `cargo test -p game_server -- --quiet`
- Production live-RON integration and current/backup store suites within those runners
- `cargo fmt --all -- --check`
- Scoped `git diff --check`, legacy/static authority searches and exact bootstrap reference validator

## Omitted Validation

- Unity source, EditMode/PlayMode and live-client probe: explicitly excluded by user because the adjacent development
  tree is materially drifted. P02 completion will claim only Core/live-RON/game_server and record future client risk.
- Final content/playability/balance: minimum 29-source + WhiteNight bootstrap only.
- Public release/deployment rehearsal: development-only project.

## Completion Conditions

1. Current dirty overlap is classified and P02-owned changes preserve unrelated user work.
2. One Core encounter progression graph replaces research/pending state and DTO/store authority end-to-end.
3. Exact 29 source rows, class assignment and 5/10/20 repeat reward authoring load fail-closed.
4. EL-01~08 and MON-01~05 pass, including unvisited/refill/distinct/exhausted-failure and P01 seam preservation.
5. NB-01~07 authored cycle passes for first/repeat/unvisited/defeat/refill.
6. BO-01~12 passes for per-chain conditions/DAG, provisional/active failure, capstone, victory/defeat and refill with
   only WhiteNight live.
7. AT-01/02 direct branch and shared AT-03/04 ledger primitives are exclusive and atomic; ordinary Reward exclusion
   passes. P04 user-facing sweep remains absent.
8. EN-01 prevents ledger/boss completion from ending Endless.
9. TX-01/CP-01 prove late rollback and exact checkpoint restore/monotonic load semantics.
10. Core/server Active Run V4 round-trips exact graph and strict-rejects all unapproved versions/shapes without
    dual path or mutation.
11. Core DTO and game_server mapping/store publish only after required durability and reconnect is idempotent.
12. Removed research/pending live-data/code/DTO/store/tests/canonical authority has no production remainder.
13. Focused non-zero and full Core/server/live-RON/store validation pass with exact counts.
14. Canonical, scoped diff, compatibility/consumer and independent completion audit receive `Complete / High`.
15. Master records P02 completion and makes only P03 ready.

## Completion Audit Evidence Matrix

| Condition | Required evidence | Current verdict |
| --- | --- | --- |
| C1 dirty preservation | activation ownership map + final scoped diff | pass |
| C2 authority replacement | state/runtime/DTO/store path + legacy zero | pass |
| C3 live sources | exact manifest validator + strict loader | pass |
| C4 Elite/Monolith | EL/MON target-first suites | pass |
| C5 NormalBoss | NB target-first suites | pass |
| C6 BossOmen | BO/DAG/bootstrap suites | pass |
| C7 reward branches | AT/RW/failpoint suites | pass |
| C8 Endless | EN suite | pass |
| C9 transaction/checkpoint | TX/CP suites | pass |
| C10 V4 migration | writer/store corruption corpus | pass |
| C11 producer/server | DTO/mapping/durability/reconnect | pass |
| C12 legacy removal | static + runtime/data evidence | pass |
| C13 runners | focused/full actual counts | pass |
| C14 audit | `COMPLETION_AUDIT.md` | pass — `Complete / High` |
| C15 Master handoff | Master registry/experiments | pass — P03 activated |

## Stop Conditions

- Dirty overlap cannot preserve an existing user change or target ownership is ambiguous.
- Any approved 29-row encounter/abnormality/fragment reference is missing or requires substitution.
- P01 checked 150/200 seam must be weakened or reward/wave semantics changed without policy.
- WhiteNight bootstrap cannot be reachable without a global fallback or new live final content.
- V4 requires V3 transformation, dual read/write, ignored field or partial Core/server rollout.
- Package cannot reach a fully runnable Core/live-RON/game_server state in one Phase.
- Public consumer support requires a new compatibility policy; Unity exclusion itself is not a blocker.
- Source/reward/cycle/attempt/node state cannot be rolled back atomically.
- Required non-zero Core/server/live-data/store runner is unavailable or has unexplained baseline failures.
- Work expands into P03 profile/Standard, P04 sweep UX or WP005 skill content.
