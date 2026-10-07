# Game Rulebook Policy Implementation Master Phase 03 — Fixed Standard And Profile

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes; limited to this atomic Phase contract
Master Goal: `../game_rulebook_policy_implementation_master/`
Phase: P03
Package: `GRB-WP-003`
Primary Findings: `GRB-M-001`, `GRB-M-004`
Policy Decisions: `GRB-PD-002`, `GRB-PD-006`
Activated: 2026-08-12
Completed: 2026-08-12
Completion Audit: `Complete / High`; see `COMPLETION_AUDIT.md`
Repository Root: `/mnt/f/work/simulator`
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
Direct Dependency: P02 `Complete / High`; exact Active Run V4 and encounter progression seam
Consumer Scope: Core + live RON + game_server; Unity explicitly excluded under current user direction

## Objective

Approved minimum `ST1-NT60` Standard tutorial을 세 개의 authored fixed Floor로 구현하고, F3의 reduced
Nothing There NormalBoss 승리로 Standard run completion과 영구 tutorial 완료를 원자적으로 확정한다.
기존 encounter compendium V1의 four same-semantic sets를 보존하는 successor permanent profile/store,
bounded one-shot migration, Endless admission gate와 monotonic records를 Core/server projection까지 함께 닫는다.

## User/Contract-Visible Problem

Preflight 당시 Standard는 generated/adaptive map과 FinalBoss terminal을 사용했고 영구 저장은 encounter
compendium V1 sets만 소유했다. 승인된 target은 fixed three-Floor/no-adaptive tutorial, F3 Nothing There
NormalBoss completion, tutorial 완료 뒤 Endless 해금과 rollback되지 않는 개인 기록이다. Run terminal과
profile flag를 별도 publication으로 처리하면 crash/restart 뒤 한쪽만 보일 수 있으므로 P03는 fixed data,
Core transition, successor store/migration과 server response를 하나의 runnable boundary로 닫아야 한다.

## Audit Finding ID

- M001 fixed three-Floor Standard and F3 NormalBoss completion
- M004 monotonic successor profile, tutorial gate and records

## Current Runtime And Live Data Evidence

Completed Preflight evidence, to be revalidated before production mutation:

- Standard currently uses generated/adaptive facility assignment and a final-Floor FinalBoss path.
- Existing reusable ST1 topology/content IDs are present; the three exact tutorial reward-pool IDs are absent.
- Nothing There encounter/abnormality exist, but boss-actor-only exact `720/53/36/33` override is absent.
- Permanent persistence is encounter-compendium envelope/profile V1 with four discovered/defeated sets.
- Core/game_server current package baselines were green at P02 exit: 1055 and 63 tests respectively.
- Relevant facility/map/profile/store/server paths are broadly dirty with user work and require path/hunk
  classification before mutation.

## Source Of Truth And Ownership

- Gameplay meaning and fixed bootstrap: current `game_rulebook.md`, `GRB-PD-006` and selected
  `CONTENT_INPUT_CANDIDATES.md` `ST1-NT60` table.
- Fixed Standard topology, content, terminal, reward pools and actor-only tuning: strict production live RON plus
  loader/cross-catalog validation.
- Standard run progression, admission and terminal result: Core typed state/behavior; Unity does not infer them.
- Permanent profile reducer and projection: one Core profile state containing the four compendium sets plus tutorial
  and monotonic record fields.
- Successor profile bytes/current/backup/migration/reset/crash recovery: one game_server host store.
- Active Run V4 remains P02's exact run-local authority; permanent profile is outside checkpoint rollback.

## Invariants

- Standard always uses the three authored ST1 topology versions and fixed slot/content/tuning. No procedural
  assignment, certified-power anchor, adaptive role multiplier, Endless pool/cycle, BossOmen or Monolith applies.
- Floor 1 has the approved Normal core encounter, Floor 2 the approved Porcubbus Elite and Floor 3 reduced Nothing
  There as `NormalBoss`; no Standard `FinalBoss` is authored or selected.
- The only new content is three exact single-entry tutorial reward pools:
  `tutorial_f1_training_budget`, `tutorial_f3_salvage`, `tutorial_f3_budget`. No final content is invented.
- F3 Nothing There changes only its boss actor's HP/ATK/DEF/MR to exact `720/53/36/33`. Other actors, other Nothing
  There occurrences, node/wave/reward values and its deliberately easy damage-meter pattern remain unchanged.
- Standard F3 defeat uses NormalBoss attempts; retreat restores pre-entry. Neither writes tutorial completion.
- F3 victory exposes Standard `RunComplete` and `standard_tutorial_completed: true` only through one crash-recoverable
  host publication boundary. Repeated tutorial completion adds no new global reward.
- Endless start is visible but rejected with a typed requirement while tutorial completion is false; true admits it.
- Successor profile preserves the four V1 discovered/defeated sets exactly, including dormant IDs. New fields default
  to false/0 only during the bounded V1 migration.
- V1 support exists only as one-shot migration input. After durable successor publication, current writer/reader is
  successor-only; no permanent dual read/write, alias or empty fallback remains.
- Highest Floor and longest deathless Floor streak are max reducers. All-live-FinalBoss completion is a monotonic
  boolean; current one-chain WhiteNight membership is non-vacuous.
- Profile writes publish immediately and are idempotent. Checkpoint load, battle retreat, active-run rollback,
  run failure, new run and process restart never lower/delete committed profile state.
- Only an explicit full-profile reset removes all profile fields. Individual record reset and partial compendium
  reset are not compatibility paths.
- P02's exact V4 run graph, 29 source semantics and research removal remain unchanged.

## Approved Policy And Evidence

- Master P03 contract, migration strategy and completion gate.
- Preflight `ST-01`~`04`, `PR-01`~`03`, `MIG-04`~`06`, `BOOT-08`~`10`, `CON-01`~`03`.
- `GRB-PD-002`: four same-semantic V1 sets preserved; new tutorial/record fields false/0.
- `GRB-PD-006`: modified `ST1-NT60`, no scripted fragment, F3 boss-actor-only `720/53/36/33`.
- P02 completion audit `Complete / High` and exact V4/profile-outside-checkpoint seam.

## Open Policy Decisions

없음 at activation. Concrete successor version/name, transaction journal shape and internal reducer layout are
technical choices only if they preserve all approved migration/publication semantics. Any need for partial V1
support, a different tutorial topology/content, different F3 tuning, a client compatibility field or separate
run/profile visibility requires stopping the affected mutation.

## In Scope

- Current Standard generator/terminal/adaptive authority, profile/store/mapping and dirty ownership inventory
- Exact fixed `ST1-NT60` three-Floor live authoring and strict referential/shape validation
- Three exact single-entry tutorial reward pools
- F3 Nothing There boss-actor-only exact four-axis override and isolation validation
- Core fixed Standard creation/advance/completion, NormalBoss defeat/retreat and Endless tutorial admission
- Unified successor permanent profile/reducer/projection with tutorial and monotonic record fields
- Bounded V1 set-preserving migration, successor current/backup/restart/corruption/reset behavior
- Crash-recoverable Standard completion + tutorial flag publication and persistence-before-response
- Target-first transition, migration, failpoint, restart, checkpoint/rollback and reconnect tests
- Required current canonical updates and independent `Complete / High` Phase audit

## Out Of Scope

- Mastered Elite sweep/reacquisition UI/runtime (P04)
- Dedicated fragment skills (WP005), scripted tutorial fragment and final tutorial content/polish/balance
- Changes to Nothing There's pattern, transformation, regeneration or other actor/content values
- Additional FinalBoss/BossOmen chains or changes to P02 source/reward/cycle semantics
- Unity source/runner/client migration under current explicit exclusion
- Public release/deployment gate, cloud/account profile, multi-install merge and unrelated dirty cleanup

## Implementation Scope

P03는 fixed Standard data/runtime와 permanent profile store/migration을 partial release 없이 함께 구현한다.
Internal work may use bounded target-first units, but Phase exit must provide this graph:

```text
strict fixed ST1 live data
  -> Core fixed Standard progression and F3 NormalBoss result
      -> staged RunComplete + successor profile update
          -> one crash-recoverable host publication
              -> typed result/snapshot and restart/reconnect
```

Future P04 fields or placeholder commands are not authored. If P03 needs to add a permanent projection to a shared
DTO, Core producer and game_server mapping/store change in the same unit.

## DTO/Data/Save Migration

- Data: exact fixed templates/assignments/terminal/tuning and three reward pools are strict required production RON.
- Active Run: remains exact V4. Additive current-version change is allowed only if the approved fixed Standard state
  cannot be represented by existing map/facility fields and exact backward meaning is retained; otherwise stop for
  a version decision.
- Permanent profile: successor schema contains V1's four sets plus tutorial and monotonic records with deny-unknown
  strictness. Writer-produced V1 current/backup are bounded migration inputs, not ongoing profile readers.
- Migration preserves all four sets exactly and initializes new fields false/0, then durably publishes successor
  before retiring V1 source. Current, backup, corrupt pair, interrupted write and restart are direct tests.
- Standard terminal/profile publication uses a combined store or recovery journal with a proven commit marker so
  restart exposes both completion and flag or neither. The exact mechanism is discovery pending.

## Deployment And Compatibility

This is a development-only project with no public rollout gate. Core, production live RON and game_server are one
supported P03 boundary. Unity is not inspected or changed and receives no alias, ignored field or fake tutorial
completion. V1 compatibility is limited to the approved one-shot local migration and has an explicit successor
commit termination condition.

## Focused Validation

- `ST-01`~`04`: exact fixed Floor topology/content/tuning, no adaptive/FinalBoss, F3 victory/defeat/retreat and gate
- `PR-01`~`03`: highest Floor, deathless streak, all-live-FinalBoss and rollback/profile separation
- `MIG-04`~`06`: exact V1 set preservation/defaults, current/backup/corrupt/write/restart and full reset only
- `BOOT-08`~`10`: reused IDs, exact new pools, boss-actor-only `720/53/36/33` isolation
- `CON-01`~`03`: typed profile producer/server mapping and durability-before-success
- Standard completion/profile late-failure and crash/restart failpoints with exact two-sided visibility
- Exact commands and actual counts are recorded after current runner discovery.

## Broad Validation

- `cargo test -p game_core -- --quiet`
- `cargo test -p game_server -- --quiet`
- Production live-RON integration and successor profile store suites within those packages
- `cargo fmt --all -- --check`
- Scoped `git diff --check`, legacy/adaptive/FinalBoss/V1/static authority searches and exact ST1 reference validator

## Omitted Validation

- Unity source, EditMode/PlayMode and live-client probe: explicitly excluded because the adjacent development tree is
  materially drifted. P03 claims only Core/live-RON/game_server and adds no old-client fallback.
- Final tutorial content, art, pacing and balance: minimum approved `ST1-NT60` bootstrap only.
- Public deployment rehearsal, cloud/account merge and multi-install profile: development-only/local-install scope.

## Completion Conditions

1. Current dirty overlap is classified and unrelated user changes are preserved.
2. Exact three fixed Standard Floors load from strict production authoring with no generated/adaptive fallback.
3. BOOT-08 exact reused references and all three BOOT-09 single-entry reward pools pass fail-closed validation.
4. BOOT-10 applies exact `720/53/36/33` only to the F3 Nothing There boss actor and preserves every other use/value.
5. ST-01 proves Floor 1 Normal, Floor 2 Porcubbus Elite, Floor 3 Nothing There NormalBoss and no FinalBoss.
6. ST-02 F3 victory makes RunComplete and tutorial flag atomically durable across failure/crash/restart.
7. ST-03 defeat/retreat obey NormalBoss attempt semantics and never grant tutorial completion.
8. ST-04 rejects Endless while false and admits it while true through a typed Core/server contract.
9. One successor profile owns four compendium sets, tutorial flag and all approved monotonic records/projection.
10. PR-01~03 pass across checkpoint load, active-run rollback, failure, new run, duplicate event and restart.
11. MIG-04 preserves all four V1 sets exactly and initializes new fields false/0 with dormant IDs retained.
12. MIG-05 current/backup/corrupt/interrupted-write/restart recovery is fail-closed and crash-safe.
13. MIG-06 removes only through explicit full reset; no permanent V1 reader/dual writer/partial reset remains.
14. Core producer, game_server mapping/store and response order use one current profile contract.
15. P02 V4/source/research-removal invariants and all existing gameplay tests remain green.
16. Focused non-zero and full Core/server/live-RON/store validation pass with exact counts.
17. Current canonical, scoped diff, compatibility/consumer classification and independent completion audit receive
    `Complete / High`.
18. Master records P03 completion and makes only P04 ready.

## Completion Audit Evidence Matrix

| Condition | Required evidence | Current verdict |
| --- | --- | --- |
| C1 dirty preservation | activation ownership map + final scoped diff | pass — no reset/cleanup; scoped diff check 0 |
| C2 fixed authoring | exact three-template data/loader/runtime path | pass — strict fixed branch and live schedules |
| C3 bootstrap refs/pools | BOOT-08/09 validator | pass — exact three single-entry pools |
| C4 actor-only tuning | BOOT-10 isolation suite | pass — exact actor/corroded/outside-F3 test |
| C5 Standard topology | ST-01 end-to-end | pass — fixed seed-independent Normal/Elite/NormalBoss |
| C6 atomic completion | ST-02 failpoint/restart | pass — journal, pending fail-close, startup/typed retry |
| C7 defeat/retreat | ST-03 transitions | pass — direct StandardCompletion NormalBoss test |
| C8 Endless gate | ST-04 typed rejection/admission | pass — false no-mutation reject; true admit |
| C9 profile authority | state/DTO/store path | pass — one PlayerProfile V2 authority |
| C10 monotonic records | PR suites and rollback separation | pass — max/OR and checkpoint separation |
| C11 V1 migration | MIG-04 exact preservation/defaults | pass — all four sets, dormant IDs, false/0 |
| C12 recovery | MIG-05 corruption/write/restart corpus | pass — current/backup/temp/failpoint corpus |
| C13 reset/legacy | MIG-06 + static zero | pass — full reset only; old public names zero |
| C14 producer/server | DTO/mapping/durability/order | pass — Core/server typed path and 71-server suite |
| C15 P02 preservation | V4/source/legacy focused suites | pass — V4 7, progression 6, BossOmen 15, store 12 |
| C16 runners | focused/full actual counts | pass — Core 1065; server 71; 0 failed/ignored |
| C17 audit | `COMPLETION_AUDIT.md` | pass — `Complete / High` |
| C18 Master handoff | Master registry/experiments | pass — handoff restricted to P04 ready |

## Stop Conditions

- Dirty overlap cannot preserve an existing user change or fixed/profile ownership is ambiguous.
- Any reused ST1 reference is missing or requires a substitute; any new content beyond the three approved pools is
  needed for a runnable tutorial.
- Fixed Standard cannot avoid adaptive/generated/Endless authority without weakening shared runtime invariants.
- F3 exact actor-only override cannot be isolated from other actors/encounters or requires pattern redesign.
- Standard completion/tutorial flag cannot be made crash-recoverable with both-or-neither restart visibility.
- Successor migration requires set loss, permanent V1 dual reader/write, ignored field, partial reset or silent empty
  fallback.
- Profile semantics require an unapproved version/default/record/reset/identity or cloud/account policy.
- P02 exact V4 would require a semantic version change outside the approved additive boundary.
- Required Core/server/live-data/profile runner is unavailable or has unexplained baseline failures.
- Work expands into P04 sweep, WP005 skills, final content, Unity repair or unrelated refactor.
