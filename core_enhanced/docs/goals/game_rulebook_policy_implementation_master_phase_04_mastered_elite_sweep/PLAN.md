# Game Rulebook Policy Implementation Master Phase 04 — Mastered Elite Sweep

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes; limited to this atomic Phase contract
Master Goal: `../game_rulebook_policy_implementation_master/`
Phase: P04
Package: `GRB-WP-004`
Primary Finding: `GRB-M-007`
Policy Decisions: `GRB-PD-003`, `GRB-PD-004`, `GRB-PD-010`
Activated: 2026-08-12
Completed: 2026-08-12
Repository Root: `/mnt/f/work/simulator`
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
Direct Dependencies: P02 and P03 `Complete / High`; stable mastery/source ledger, exact Active Run V4 and
PlayerProfile V2 boundary
Consumer Scope: Core + production live RON if validation requires + game_server; Unity explicitly excluded under
current user direction

## Objective

Endless에서 현재 배정된 숙달 Elite node에만 direct battle과 typed sweep 선택을 제공한다. Sweep은 battle
setup/simulation/history를 만들지 않는 synthetic victory이며, base reward, exclusive fragment
reacquire/repeat branch, living-roster XP, active consumable duration, repeat-pool cursor와 node/session progression을
한 transaction으로 commit한다. 마지막 fragment 분쇄 후 같은 identity를 반복 재획득하는 승인 cycle을
mastery/source/upgrade/awakening 손실 없이 보존한다.

## User/Contract-Visible Problem

P02는 Elite mastery, explicit reward-source completion과 mastered repeat assignment를 구현했지만 실제
숙달 소탕 command/capability는 없다. 현재 직접 전투 경로를 재사용해 억지로 승리시키면 condition,
damage, trauma, discovery, event log, result stats와 battle record 같은 금지 side effect가 생길 수 있고,
fragment 보유 0 재획득 분기와 repeat reward를 동시에 지급할 위험도 있다. P04는 battle lifecycle과
분리된 typed vertical slice로 이 마지막 정책 gap을 닫아야 한다.

## Audit Finding ID

- M007 mastered Elite typed sweep vertical slice

## Current Runtime And Live Data Evidence

Activation 전 승인 evidence이며 production 조사에서 다시 확인한다.

- P02 owns run-local Elite identity assignment, mastery/victory count, stable reward-source completion, fragment
  first/repeat branch, mastered pool cycle and exact 29 source rows.
- P03 keeps this graph in exact Active Run V4 and established that no sweep state belongs in permanent
  PlayerProfile V2.
- Rulebook approves unlimited dismantle→mastered-sweep reacquisition when source is complete and owned count is 0.
- Master contract requires a typed command/capability/result with no special item/cost and no battle simulation.
- Exact current NodeConfirm projection, reward/XP/duration reducers, repeat cursor and server producer mapping are
  discovery pending.
- P03 exit current-tree baselines are Core 1065 and game_server 71 passing, 0 failed/ignored; P04 will run focused
  pre-mutation characterization after exact ownership inventory.
- Target paths overlap a broad inherited dirty worktree; hunk ownership and safe merge are required before mutation.

## Source Of Truth And Ownership

- Gameplay meaning: current `game_rulebook.md`, Master P04 contract and `GRB-PD-003/004/010`.
- Eligibility and source/mastery identity: P02's run-local encounter progression and current assigned Elite node.
- Capability/command/result, synthetic planner and atomic mutation: Core typed world/node state.
- Reward, XP and duration semantics: existing authoritative reward/grant and run-duration reducers, composed without
  creating a battle.
- Repeat-pool cursor, node/session and checkpoint: existing run-local state in exact Active Run V4.
- Public producer/mapping: Core DTO plus game_server typed serde/mapping; no client-side eligibility inference.
- Permanent PlayerProfile is read-only to this feature and receives no sweep field or record.

## Invariants

- Sweep exists only in Endless for the currently assigned Elite identity whose mastery threshold and source
  completion are already committed. Standard, unmastered Elite, NormalBoss, FinalBoss and ordinary Combat reject.
- NodeConfirm exposes explicit direct-battle and sweep choices, including Core-owned disabled reasons, from one
  eligibility evaluator. Sweep additionally requires at least one living, combat-available, armed employee; command
  execution revalidates the same authority and stale/ineligible input changes nothing.
- Direct battle is unchanged. Sweep creates no `ActiveBattle`, battle setup/update, combat result screen, event log,
  result stats, battle record or synthetic participant/death/discovery event.
- Sweep is a synthetic victory only for encounter/node/pool progression. It does not add a victory toward initial
  mastery and cannot complete an unfinished reward source.
- Base encounter reward is applied exactly once through the existing grant transaction.
- If source completion is true and owned fragment count is 0, sweep grants exactly one same `fragment_id` and no
  repeat reward. If count is at least 1, it grants only the authored `repeat_reward` and no fragment.
- Reacquisition never resets mastery, source completion, source identity, fragment upgrade or awakening progress.
  Last-copy dismantle→sweep reacquisition may repeat without an artificial counter; assignment/cycle availability is
  the only frequency limiter.
- Every living roster employee receives the approved sweep XP amount once. Dead employees do not. No quotient,
  participant selection or deployed-unit inference is used.
- Existing active consumable run-duration progression advances exactly once by the approved sweep unit. Sweep does
  not invent a dedicated consumable, token, energy, currency cost or cooldown.
- Base reward, exclusive fragment branch, XP, duration, repeat cursor/assignment release, node/session completion and
  resulting navigation commit together or not at all. Overflow/validation/late failure leaves every root unchanged.
- Sweep does not modify employee HP/condition/physical damage/trauma, equipment, combat bonus, compendium discovery,
  BossOmen, monolith, FinalBoss record, result stats or debug combat trace except effects explicitly contained in the
  approved base/repeat reward transaction.
- P01 150/200, P02 exact source/cycle/BossOmen/V4 and P03 fixed Standard/PlayerProfile/journal remain unchanged.

## Approved Policy And Evidence

- Master P04 contract and `GRB-WP-004` disposition.
- Preflight mastered-sweep rows and dependency map, to be copied by stable IDs after activation read.
- `GRB-PD-003`: explicit stable reward-source identity.
- `GRB-PD-004`: exact 29 source rows and Elite repeat dust policy.
- `GRB-PD-010`: unlimited dismantle/reacquire cycle.
- P02 and P03 completion audits `Complete / High`.

## Open Policy Decisions

없음 at activation. Command/result type names, internal planner layout and whether existing reward/duration helpers can
be directly composed are technical choices only if every approved visible/state invariant above remains exact.
Discovery that the approved XP amount/duration unit or direct-vs-sweep projection is not fixed by existing policy
requires stopping the affected mutation rather than inventing a value.

## In Scope

- Current mastery/source/node/session/reward/XP/duration/cycle/DTO/server authority and dirty ownership inventory
- One shared sweep eligibility/projection/revalidation evaluator
- Typed direct-versus-sweep capability and a two-step mastered Elite sweep command/result flow
- No-battle synthetic victory plan and atomic commit
- Base reward plus exclusive reacquire/repeat reward branching
- Living-roster XP and active-consumable duration advancement
- Repeat cursor/assignment release, node completion/navigation and checkpoint/continue persistence
- Dismantle→reacquire cycle with upgrade/awakening/mastery/source preservation
- Invalid/stale/late-failure/idempotence/reconnect tests, canonical update and independent `Complete / High` audit

## Out Of Scope

- Standard sweep, unmastered Elite sweep, NormalBoss/FinalBoss/BossOmen/Monolith sweep
- Dedicated sweep item, charge, token, Enkephalin/currency cost, cooldown or daily limit
- Battle setup/simulation/result/history/stat/discovery generation
- New encounter/reward/fragment content or changes to exact 29 source rows/dust values
- Dedicated fragment skills `GRB-WP-005`, final content/balance/polish
- PlayerProfile schema/version/migration or new permanent record
- Unity source/runner/client migration under current explicit exclusion
- Public deployment gate and unrelated dirty cleanup/refactor

## Implementation Scope

The Phase exit must provide one connected path:

```text
current Endless mastered Elite NodeConfirm
  -> one eligibility evaluator
      -> typed direct/sweep projection with disabled reasons
          -> SweepMasteredElite command revalidation
              -> synthetic result plan on cloned world roots, with no authoritative mutation
                  -> stable staged result projection across reconnect
                      -> CompleteMasteredEliteSweep command revalidation
                          -> one atomic commit of reward/branch/XP/duration/cursor/node/session/navigation
                              -> typed completion result + exact V4 checkpoint continuation
```

No placeholder P04 state may be added outside this path. Existing battle victory helpers may supply pure planning
sub-operations only if forbidden battle side effects cannot enter the staged world.

## DTO/Data/Save Migration

- DTO: additive typed capability/context/command/result within the current Core/game_server development boundary.
  Producer and server serialized tests change together; no alias, unknown-action fallback or client inference.
- Data: no new content is expected. Existing reward source, base reward and RunPolicy values remain authority. Any
  strict validation needed for sweep-reachable sources must reuse current production RON.
- Active Run: remains exact V4. Existing progression/node/session/cycle state must be sufficient; an additive field or
  version change requires stopping for policy/version review.
- Permanent profile: no change. Sweep and its cycle are run-local and checkpoint-rollbackable.

## Deployment And Compatibility

This is a development-only Core/live-RON/game_server atomic boundary with no public rollout gate. Unity is not
inspected or changed, and no ignored field/dual command/compatibility fallback is added. Existing current V4/V2
readers and writers remain exact.

## Focused Validation

- Eligibility matrix: mode/class/mastery/source/assignment/current-node/session/state, at least one living
  combat-available armed employee, disabled reason and stale command
- Direct battle remains exact and sweep never creates battle lifecycle artifacts
- Base reward exactly once; exclusive owned-0 reacquire versus owned-1+ repeat branch
- Living-only roster XP and active consumable duration exactly once
- Atomic failure corpus across reward overflow/invalid source/late grant/cursor/session mutation
- Repeat assignment/cursor/node/navigation transition and checkpoint/continue round trip
- Dismantle/reacquire repeated cycle preserving mastery/source/upgrade/awakening
- Forbidden side-effect before/after fingerprints including profile/compendium/BossOmen/monolith/battle trace
- Core producer/game_server serialization, reconnect idempotence and old command/placeholder static zero
- Exact commands and actual counts are recorded after current runner/test discovery.

## Broad Validation

- `cargo test -p game_core -- --quiet`
- `cargo test -p game_server -- --quiet`
- Production live-RON suite if the reachable-source validator is changed
- `cargo fmt --all -- --check`
- Scoped `git diff --check` and legacy/fake-battle/profile/V4/static authority searches

## Omitted Validation

- Unity source, EditMode/PlayMode and live-client probe: explicitly excluded because the adjacent development tree is
  materially drifted. Completion claims only Core/live-RON/game_server and adds no client compatibility fallback.
- Final content/balance/playtesting: policy vertical slice and existing bootstrap sources only.
- Public rollout/migration rehearsal: development-only exact current versions.

## Completion Conditions

1. Current dirty overlap and authoritative command→mutation path are classified; unrelated user changes preserved.
2. One evaluator projects and revalidates sweep only for the current Endless mastered Elite/source-complete node with
   at least one living combat-available armed employee, and Core owns the disabled reason.
3. Direct battle remains available/unchanged. One typed command stages a stable battle-independent result without
   authoritative mutation; a second typed confirmation command atomically commits it through Core and game_server.
4. Sweep creates no battle setup/simulation/result/history/event/stat/discovery artifact.
5. Base reward applies exactly once through the existing atomic grant semantics.
6. Source complete + owned 0 grants exactly one source fragment and no repeat reward; owned 1+ grants repeat only.
7. Living roster XP and active consumable duration advance exactly once with approved existing values.
8. Reward/branch/XP/duration/repeat cursor/assignment/node/session/navigation are one all-or-nothing transaction.
9. Failure, invalid and stale commands leave every relevant root unchanged; reconnect before confirmation reprojects
   the same staged result without replanning or duplication, and reconnect cannot replay a committed sweep.
10. Dismantle→reacquire can repeat without a new counter while preserving mastery/source/upgrade/awakening progress.
11. Condition/damage/trauma/bonus/compendium/BossOmen/monolith/profile/result stats/battle trace remain unchanged.
12. Checkpoint load rolls back the run-local sweep/cycle normally; exact V4/V2 shape/version and P02/P03 semantics stay.
13. No item/cost/cooldown, new content, profile field, fake battle path, compatibility alias or future WP005 state exists.
14. Focused Core/server producer tests and full non-zero package suites pass with actual counts.
15. Closest canonical, scoped diff/legacy/consumer audit and independent completion audit receive `Complete / High`.
16. Master records P04 completion and proceeds only to its orchestration-only final cross-Phase audit.

## Completion Audit Evidence Matrix

| Condition | Required evidence | Current verdict |
| --- | --- | --- |
| C1 dirty/authority | activation inventory + scoped final diff | pass |
| C2 eligibility | complete matrix and shared evaluator | pass |
| C3 typed producer/server | command/capability/result serialization | pass |
| C4 no battle | artifact fingerprint and message sequence | pass |
| C5 base reward | exact-once grant test | pass |
| C6 exclusive branch | owned 0/1+ tests | pass |
| C7 XP/duration | living/dead and active duration tests | pass |
| C8 atomic commit | staged roots + late-failure corpus | pass |
| C9 stale/reconnect | no-mutation/idempotence tests | pass |
| C10 reacquire cycle | repeated dismantle/reacquire preservation | pass |
| C11 forbidden effects | before/after full fingerprint | pass |
| C12 persistence | checkpoint/V4/V2 preservation | pass |
| C13 exclusions/legacy | static zero and scope review | pass |
| C14 validation | focused/full exact counts | pass |
| C15 audit | canonical + `COMPLETION_AUDIT.md` | pass — `Complete / High` |
| C16 Master handoff | registry/final-audit readiness | pass |

## Stop Conditions

- Dirty overlap cannot preserve an existing user change or current sweep ownership is ambiguous.
- Eligibility, XP amount, duration unit, base reward or node completion meaning is not fixed by approved policy/current
  authority and would require inventing user-visible behavior.
- Sweep cannot avoid battle state/history/discovery/condition side effects without a competing progression authority.
- Exclusive reacquire/repeat cannot be planned atomically with reward/XP/duration/cursor/node/session.
- Existing V4 state cannot represent committed/reconnect/checkpoint continuation without a version/additive decision.
- A dedicated item/cost, new content, profile field, compatibility shim, ignored command or Unity change is required.
- P01/P02/P03 invariant regresses or a required Core/server runner has unexplained baseline failures.
- Work expands into WP005 skills, final content, Unity repair or unrelated refactor.
