# Game Rulebook Policy Implementation Master Phase 03 — Experiments

## 2026-08-12 — Phase Activation And Document Initialization

### Hypothesis

P02 `Complete / High` closes P03's only production prerequisite, so the continuing Master execution request permits
P03 activation as the sole production Phase after its required documents are created.

### Performed work

- Validated P02 complete/true status, `Complete / High` audit and exact V4/source handoff.
- Read the Master P03 boundary, migration strategy, activation rules and Preflight ST/PR/MIG/BOOT/CON rows.
- Read the complete Goal authoring/execution guide before initializing this child.
- Confirmed the P03 child path did not exist and created its three required documents.
- Changed the Master registry from P02 active/P03 pending to P02 complete/P03 active.

### Exact commands

```text
read P02 PLAN/EXPERIMENTS/EXPERIMENT_NOTES/COMPLETION_AUDIT and Master registry
read goal_authoring_and_execution_guide.md completely
test ! -e <P03 child directory>
mkdir -p <P03 child directory>
```

### Result

- P02 dependency: complete/true, audit `Complete / High`.
- P03 path collision: none.
- Tests/probes: 0; pass/fail/skip 0/0/0 in this activation unit.
- Production changes: 0; Child/Master documents only.

### Interpretation

P03 is active. Production mutation remains gated by current dirty ownership, current Standard/profile authority,
exact ST1 references, profile V1 bytes/store graph and current Core/server baselines.

## 2026-08-12 — Activation Ownership And Authority Inventory

### Hypothesis

The approved fixed Standard can reuse the three live ST1 topology templates while replacing only their assignment
authority, and the existing compendium V1 store can be migrated into one successor profile plus a bounded terminal
transaction journal without changing Active Run V4 meaning.

### Performed work

- Inspected the Standard schedule, facility template/profile schemas, generator, encounter assignment, terminal
  progression and `RunComplete` commit path.
- Verified the exact approved ST1 slot/content table against all three live topology IDs.
- Inspected reward metadata/pools and confirmed that all three reward IDs exist while the three approved tutorial
  pools do not.
- Inspected Nothing There metadata/encounter and confirmed base `1200/88/60/55`; the existing three-axis global
  scale cannot express actor-only exact magic resistance.
- Inspected Core compendium state/projection, V1 host envelope/current/backup/reset, active-run V4 invalidation and
  player-actor persistence ordering.
- Classified the broad target-path dirty state as inherited Master/P01/P02 work plus pre-existing user work; no
  reset, stash, restore or unrelated mutation was performed.

### Exact evidence

- Standard schedule: Floor 1/2/3 resolve `standard_intro_a`, `standard_branching_a`, `standard_final_a` through
  weighted `NodeAssignmentProfile` assignment.
- Current F3 terminal: `RunFinalBoss`; current encounter class selection upgrades any final Standard Boss to
  `FinalBoss`.
- Existing profile: four compendium sets only; host version/file set is
  `encounter_compendium.json`/backup at version 1.
- Current server order: terminal active-run invalidation and independent compendium publication, with no shared
  commit marker.
- P02 exit baselines are reused as the P03 activation baseline because no production file changed between the P02
  full runs and this read-only inventory: Core 1055 passed; game_server 63 passed; 0 failed/ignored.

### Result

- Reused ST1 references: present and exact.
- New tutorial pools: absent as expected; underlying reward entries present.
- Required production design seams: fixed slot payload authoring, Standard-only difficulty/actor override,
  successor profile reducer/projection, V1-to-successor migration and a recoverable profile/active-run terminal
  journal.
- Stop conditions triggered: none.

### Interpretation

Production mutation is authorized. Standard must bypass weighted category/content assignment and adaptive combat
difficulty entirely; Endless keeps those paths. The Nothing There reduction must be carried by the authored F3 map
node into battle setup and applied only to the matching boss actor. The successor profile transaction must recover
before sessions are served so restart never exposes only one side of Standard completion.

## 2026-08-12 — Fixed Standard And Successor Profile Implementation

### Hypothesis

One strict fixed-assignment branch plus one monotonic successor profile can implement the approved bootstrap without
changing Endless assignment or P02 encounter-progression ownership. A durable Standard-completion intent journal can
close the otherwise separate profile and Active Run aggregates.

### Performed work

- Added mutually exclusive `fixed_slot_assignments` authoring with exact kind/payload and optional authored combat
  rewards/actor overrides; fixed Standard no longer runs the adaptive category solver or random content selection.
- Authored the exact three ST1 Floors, `StandardCompletion` F3 terminal, `tutorial_fixed_v1` 100% tuning, three
  single-entry reward pools and the F3 Nothing There actor-only `720/53/36/33` override.
- Added strict cross-catalog validation for fixed slots, payload discriminants, reward IDs and override actor
  membership; battle setup applies exact stats only to the matching abnormality spawn.
- Added `PlayerProfileState`, typed Endless admission, Endless-only max/OR records, checkpoint-scoped deathless
  progress and exact current Active Run V4 `record_progress` serialization.
- Replaced the current host compendium store with strict PlayerProfile V2 while preserving V1's four sets through a
  bounded one-shot migration and full-reset-only removal.
- Added a durable Standard completion journal. Profile publication creates a fail-closed
  `completion_recovery_pending` state until Active Run invalidation and journal cleanup finish; startup and typed
  retry replay the same intent.
- Updated the closest gameplay/runtime/data canonical. Unity files and runners were not inspected or changed.

### Target-first corrections

- The first broad Core run found shared tests selecting the first combat-difficulty profile; adding
  `tutorial_fixed_v1` exposed that hidden ordering assumption. Production and tests now resolve
  `default_facility` by stable ID.
- The first all-live-FinalBoss reducer used cumulative compendium defeats. It was corrected to the current run's
  `BossOmenRunState.completed_chain_ids` against the non-empty current live chain set.
- Independent ST-03 review found only composed defeat/retreat evidence. Added a direct
  `StandardCompletion + NormalBoss` test covering retryable defeat, re-entry, retreat rollback and tutorial-flag
  invariance.
- Failpoint review found that restart recovery was safe but same-process actors could observe a stale continue
  source after Active Run invalidation failure. Added the explicit pending state, command fail-closed behavior and
  an in-process typed retry integration test.

### Result

- Fixed Standard, actor-only tuning, profile V2/migration, Endless gate/records and completion journal are connected
  through production Core/live-RON/game_server paths.
- Active Run remains exact V4; the development-only additive required `record_progress` field deliberately rejects
  earlier writer V4 shapes rather than defaulting them.
- No partial profile reset, public old compendium command, research/pending authority or V3 reader was introduced.

## 2026-08-12 — P03 Validation And Exit Candidate

### Focused validation

```text
cargo test -p game_core standard -- --quiet
  13 passed, then direct ST-03 addition:
cargo test -p game_core standard_completion_normal_boss_defeat_and_retreat_never_complete_tutorial -- --quiet
  1 passed
cargo test -p game_core profile -- --quiet
  44 passed across lib/live-RON binaries
cargo test -p game_core deathless -- --quiet
  1 passed
cargo test -p game_core permanent_records -- --quiet
  1 passed
cargo test -p game_core all_live_final_boss -- --quiet
  1 passed
cargo test -p game_core current_v4 -- --quiet
  7 passed
cargo test -p game_core encounter_progression -- --quiet
  6 passed
cargo test -p game_core boss_omen -- --quiet
  15 passed
cargo test -p game_server player_profile_store::tests -- --quiet
  11 passed
cargo test -p game_server pending_standard_completion_blocks_continue_and_retries_in_process -- --quiet
  1 passed
cargo test -p game_server active_run_store::tests -- --quiet
  12 passed
```

### Broad validation

```text
cargo test -p game_core -- --quiet
  1065 passed = 1014 lib + 4 live item + 4 live skill catalog + 25 RON loading
                + 14 skill runtime + 4 skill suite; 0 failed/ignored
cargo test -p game_server -- --quiet
  71 passed; 0 failed/ignored
cargo fmt --all -- --check
  exit 0
scoped git diff --check over P03 canonical/Core/live-RON/server paths
  exit 0
```

### Static and scope audit

- Current production command/store names `RetryEncounterCompendium`, `ResetEncounterCompendium`,
  `EncounterCompendiumPersistence`, `EncounterCompendiumStore` and `encounter_compendium_persistence`: zero.
- Active Run V3 reader/version authority: zero. Research/pending matches are negative rejection/projection tests only.
- Legacy compendium V1 filenames occur only in the bounded migration constants, migration code and its tests.
- Live Standard schedules exactly three fixed profiles and the F3 topology owns `StandardCompletion`; remaining
  `RunFinalBoss` symbols belong to Endless/general terminal support and test fixtures.
- The worktree remains broadly dirty from user and earlier campaign work. No reset/stash/restore or unrelated file
  deletion was performed; formatting was restricted to P03-touched Rust files.

### Interpretation

P03 is a `Complete / High` audit candidate. Unity remains an explicit omitted consumer and no Unity integration
claim is made. The next production mutation may begin only after `COMPLETION_AUDIT.md` independently records that
verdict and the Master registry activates P04.
