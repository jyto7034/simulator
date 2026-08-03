# Event Employee Check Dice System Experiments

Status: complete

Started: 2026-07-18

This file records only experiments and validations actually performed for this Goal.

## 2026-07-18 — Goal Initialization

### Hypothesis

The approved Dice System policy can be initialized as one cross-layer Event employee-check contract
Goal without a path collision or an immediate scope/policy decision.

### Performed checks

1. Checked the required document paths:

   ```bash
   for file in PLAN.md EXPERIMENTS.md EXPERIMENT_NOTES.md; do if test -e "core_enhanced/docs/goals/event_employee_check_dice_system/$file"; then echo "$file exists"; else echo "$file missing"; fi; done
   ```

   Actual result: all three files were missing; the Goal is new and the path has no required-document
   collision.

2. Captured the Core/server/live-data repository dirty baseline:

   ```bash
   git status --short
   ```

   Working directory: `/mnt/f/work/simulator`.

   Actual result: broadly dirty tracked and untracked state across Core, live data, game_server,
   canonical documents and many existing Goal directories. Output was longer than the tool display
   budget; no cleanup or mutation was performed.

3. Captured the external Unity repository dirty baseline with the same exact command.

   Working directory: `/mnt/f/unity projects/ark`.

   Actual result: broadly dirty tracked and untracked scripts, tests, assets, documents and unrelated
   files. No cleanup or mutation was performed.

4. Created the new Goal directory:

   ```bash
   mkdir -p core_enhanced/docs/goals/event_employee_check_dice_system
   ```

5. Created `PLAN.md`, `EXPERIMENTS.md` and `EXPERIMENT_NOTES.md` from the approved policy input and the
   user's single-Goal execution rules.

### Production changes

None. Only Goal execution documents were initialized.

### Tests

No tests or formatter were run. Actual test count: 0. This is intentional under the initialization
rule; current code/live-data/consumer discovery and focused baselines come next.

### Failure and correction

No command failed. No change was reverted or corrected.

### Decision

- Status remains `active`.
- The approved scope is one tightly coupled Event employee-check contract.
- Runtime/data/DTO/Unity facts not directly reverified in this initialization are marked discovery
  pending in `PLAN.md`.
- Production investigation or mutation has not started.

## 2026-07-19 — Mandatory Phase Gates Added

### Hypothesis

The large single contract can remain one Goal while gaining explicit internal phases whose mandatory
verification prevents an unverified schema, runtime, server, Unity or content stage from advancing.

### Performed work

- Re-read `PLAN.md`, `EXPERIMENTS.md` and `EXPERIMENT_NOTES.md` completely before editing the existing
  Goal.
- Replaced the implicit ten-step implementation sequence with Phase 0 through Phase 6.
- Added mandatory Phase states, ordering, evidence requirements, invalidation/rerun rules and a
  concrete exit gate for every Phase.
- Added Phase-local required work and validation for discovery, schema/data, Core behavior,
  DTO/server, Unity, live vertical slice and final completion review.
- Updated notes so the resume point starts at Phase 0 and later phases cannot begin early.

### Production changes

None. Only existing Goal execution documents were changed.

### Commands and tests

- Document read command:

  ```bash
  cat core_enhanced/docs/goals/event_employee_check_dice_system/PLAN.md && cat core_enhanced/docs/goals/event_employee_check_dice_system/EXPERIMENTS.md && cat core_enhanced/docs/goals/event_employee_check_dice_system/EXPERIMENT_NOTES.md
  ```

- No runtime test, formatter or production baseline was run. Actual test count: 0. Phase 0 remains
  `not_started`; this change defines its gate rather than claiming to pass it.

### Decision

- Goal Status remains `active`.
- All seven Phase states remain `not_started`.
- Phase 0 is the only Phase authorized to begin next.
- No later Phase may start until the preceding exit gate has actual recorded validation evidence.

## 2026-07-19 — Phase 0 Started

### Hypothesis

The current owner chain and dirty baseline can be enumerated read-only without an unapproved policy,
compatibility requirement, inaccessible mandatory consumer or unsafe overlap that prevents Phase 1.

### Input state

- Goal Status: `active`.
- Phase 0: changed from `not_started` to `in_progress`.
- Phase 1–6: `not_started` and blocked by predecessor gates.
- Both Core/server/data and external Unity repositories remain broadly dirty; exact target-file diffs
  must be inspected before any production mutation.

### Planned Phase 0 evidence

- Current command/state/RNG/checkpoint/effect mutation path
- Production Event/employee/equipment data and strict validation path
- Core DTO serialization, server mapping/order and Unity reader/store/command/presentation path
- Consumer and external persistence classification
- Exact target dirty-diff inventory
- Nonzero Core/live-RON/server/Unity Event baselines and current legacy/payload inventory

No production file has been changed and no Phase 0 validation result is claimed yet.

## 2026-07-19 — Phase 0 Owner Discovery And Gate

### Hypothesis

The approved current baseline is still real, all required owners/consumers are accessible, no durable
external save or deployed compatibility requirement exists, and the dirty worktree has a green
reproducible Event baseline that can support a fail-closed migration.

### Performed investigation

- Read current Core Event authoring, validation, session/checkpoint state, command/mutation, behavior
  DTO, employee profile/eligibility, equipment metadata, live Event/employee/equipment data and Event
  regression paths.
- Read game_server client-message deserialization, generic command-result mapping and snapshot
  wrapper; found no Event-specific JSON repair.
- Read Unity actual fixture, strict Event reader, selection state, command bus, composer and dedicated
  EditMode/PlayMode tests.
- Read the current Event completion review and nearest Core/external canonical to distinguish current
  pure-d20 authority from this Goal's approved target.
- Inspected target-file status/diff stats in both dirty repositories and classified safe patching on
  current contents as the only preservation-safe approach.
- Searched current dependencies/source for external database/save persistence; found only the
  in-memory cloned `RunCheckpointPayload` for run checkpoint state.

### Exact baseline validation and results

1. Core Event-focused baseline:

   ```bash
   cargo test -p game_core event_ --lib --quiet -- --test-threads=1
   ```

   Result: 38 passed, 0 failed, 0 ignored; 703 filtered out.

2. Production live RON baseline:

   ```bash
   cargo test -p game_core --test ron_loading --quiet -- --test-threads=1
   ```

   Result: 21 passed, 0 failed, 0 ignored.

3. game_server generic transport/state baseline:

   ```bash
   cargo test -p game_server --quiet -- --test-threads=1
   ```

   Result: main server test binary 30 passed, 0 failed, 0 ignored; two additional targets executed
   zero tests and are not counted as behavior evidence.

4. Actual Rust fixture comparison:

   ```bash
   cargo run -p game_core --bin generate_event_contract_fixture --quiet
   ```

   Compared with `/mnt/f/unity projects/ark/Assets/Tests/Fixtures/event_employee_check_contract.json`.
   Result: exact equality after trimming the terminal newline; generator exit 0.

5. Unity focused Event EditMode:

   ```text
   run_tests(mode=EditMode, test_names=ArkClient.Tests.EditMode.EventFlowContractTests)
   job d5da567bcc6d4b3287a3cff561b0fe4b
   ```

   Result: 7 passed, 0 failed, 0 skipped.

6. Unity focused Event PlayMode:

   ```text
   run_tests(mode=PlayMode, test_names=ArkClient.Tests.PlayMode.EventFlowRenderTests)
   job 9490821e0de043b894381b176e894fc9
   ```

   Result: 1 passed, 0 failed, 0 skipped.

### Legacy/current inventory

- Core current `required_roll`: nine source/test files.
- Unity current `required_roll`/`RequiredRoll`: five Assets files.
- Live checked Event/roll authoring: zero files.
- game_server Event repair schema: zero files.
- Target Event aptitude/modifier/key/chance symbols: zero current Core/live/Unity files.
- An initial combined inventory command referenced Unity-relative paths from the Core working
  directory and reported missing paths. It was invalid evidence; the Unity count was rerun from the
  correct Unity root and produced the counts above.

### Production changes

None. Phase 0 was read-only apart from Goal execution records.

### Gate decision

- Current/target delta, owner map, consumer classification, persistence result, dirty overlap and all
  mandatory baseline counts are recorded.
- No current checked live content, external durable save, required inaccessible consumer, policy gap,
  compatibility need or unsafe overlapping Dice implementation was found.
- Phase 0 changed from `in_progress` to `gate_passed`.
- Phase 1 is authorized to start; Phase 2–6 remain blocked.

## 2026-07-19 — Phase 1 Started

### Hypothesis

The approved employee/Event/equipment schema can replace the current scene-common check shape
fail-closed, all current fixtures/live data can migrate without compatibility, and WhiteNight can
remain advance-only while the later live checked Event is still absent.

### Input state

- Phase 0: `gate_passed` with green current baselines.
- Phase 1: changed to `in_progress`.
- Phase 2–6: `not_started` and blocked.
- Phase 1 target files are already user-modified/untracked current architecture; exact current
  snippets were read and will be patched in place without restoring historical versions.

### Phase boundary

This Phase may change employee/Event/equipment schema, validation, existing fixtures and base live
data only. It must not implement the modifier evaluator, chance calculation, RNG, committed result,
server DTO migration, Unity migration or live checked Event.

## 2026-07-19 — Phase 1 Schema, Strict Validation And Base Live Data Gate

### Hypothesis

The approved employee, checked-choice and equipment authoring contract can replace the legacy
scene-level roll schema fail-closed, preserve WhiteNight as advance-only, and load the entire current
live bundle without a compatibility alias.

### Changes

- Added the six-value `EventAptitude` and made it a required stored `EmployeeProfile` field copied by
  `Employee::from_definition` and cloned by the existing checkpoint roster payload.
- Authored the approved aptitude for all 12 live employee definitions.
- Replaced scene-level checked `required_roll` with choice-owned `EventChoiceCheckDefinition`
  (`aptitude`, allowed `dc`, optional typed `required_choice_key`).
- Added strict DC allowlist, lower-snake key, maximum-one-reachable-check, existing acyclic graph and
  broken-reference validation.
- Added equipment `event_check_modifiers` and `event_choice_keys`, including amount/duplicate/key
  validation and Event-to-equipment provider cross-catalog validation.
- Added the minimal later vertical-slice provider: `archive_scanner` (Analysis +2,
  `sealed_archive_access`) in `maintenance_tech`'s authored accessory loadout. No checked live Event
  was added; WhiteNight remains advance-only.
- Migrated all current Rust fixtures/constructors to the required profile/equipment/check schema.

### Exact commands and results

1. Initial focused schema checks:

   ```bash
   cargo test -p game_core data::event_data::tests --lib -- --test-threads=1
   cargo test -p game_core data::equipment_data::tests --lib -- --test-threads=1
   cargo test -p game_core employee_profile_requires_authored_event_aptitude --lib -- --test-threads=1
   ```

   Results: Event 6 passed, 0 failed, 0 ignored; equipment 5 passed, 0 failed, 0 ignored;
   employee-required-field 1 passed, 0 failed, 0 ignored. Two additional Event graph tests were
   added afterward and are included in the final 142-test data gate below.

2. Initial production live RON check before the final live provider/audit test was added:

   ```bash
   cargo test -p game_core --test ron_loading -- --test-threads=1
   ```

   Result: 21 passed, 0 failed, 0 ignored.

3. First Core data regression attempt:

   ```bash
   cargo test -p game_core game::data --lib -- --test-threads=1
   ```

   Result: 139 passed, 3 failed, 0 ignored; 611 filtered out. All three failures were legacy
   `employee_data.rs` RON fixtures missing the newly required `event_aptitude`. The parser rejected
   them with `missing field event_aptitude`; the fixtures were migrated and the same command rerun.

4. Final Core data regression rerun:

   ```bash
   cargo test -p game_core game::data --lib -- --test-threads=1
   ```

   Result: 142 passed, 0 failed, 0 ignored; 611 filtered out. This includes 8 Event schema/graph
   tests, 5 equipment local-validation tests and 2 Event-key cross-catalog bundle tests.

5. Profile creation, required import shape and checkpoint restoration:

   ```bash
   cargo test -p game_core employee_from_definition_preserves_authored_profile_and_combat_values --lib -- --test-threads=1
   cargo test -p game_core employee_profile_requires_authored_event_aptitude --lib -- --test-threads=1
   cargo test -p game_core load_run_checkpoint_restores_visible_run_state_but_not_load_count --lib -- --test-threads=1
   ```

   Results: 1/1, 1/1 and 1/1 passed respectively; 0 failed and 0 ignored for every command.

6. Final production-shaped embedded bundle and 12-assignment/provider audit:

   ```bash
   cargo test -p game_core --test ron_loading -- --test-threads=1
   ```

   Result: 22 passed, 0 failed, 0 ignored.

7. Formatting, compilation and scoped diff/legacy checks:

   ```bash
   cargo fmt --all --check
   cargo check -p game_core --all-targets
   git diff --check -- <Phase 1 scoped Core/data/test paths>
   rg -n 'required_roll' core_enhanced/src/game/data/event_data.rs game_resources/data/events/story/base.ron
   rg -n 'EmployeeCheckChoices' game_resources/data/events/story/base.ron
   ```

   Results: formatter exit 0; all-target check exit 0; scoped diff check exit 0. `required_roll`
   remains only inside the negative legacy-rejection test in `event_data.rs`; production Event schema
   and live Event RON contain none. Live Event RON contains no `EmployeeCheckChoices`.

### Gate decision

- Every current live source loads under the new schema; invalid/missing/legacy authoring fails before
  publication.
- Approved 12-profile mapping, profile creation and checkpoint clone authority are directly tested.
- Local and cross-catalog equipment rules, checked-scene count, cycle and broken references are
  directly tested.
- WhiteNight remains advance-only. No live checked Event was introduced.
- Phase 1 changed from `in_progress` to `gate_passed`. Phase 2 is authorized to begin.

## 2026-07-19 — Phase 2 Started

### Hypothesis

One shared Core evaluator can own condition/key eligibility, aptitude and highest-only equipped
modifier selection, exact chance enumeration and resolve-time revalidation while preserving the
existing one-draw atomic Event commit/checkpoint/retry boundary.

### Input state and boundary

- Phase 1: `gate_passed`.
- Phase 2: changed from `not_started` to `in_progress`.
- Phase 3–6 remain blocked.
- This Phase may change Core evaluator, Event session/committed state, RNG and Core tests. It must not
  finalize the outward DTO/server fixture, modify Unity, or add the live checked Event.

## 2026-07-19 — Phase 2 Core Evaluation, RNG, Commit And Persistence Gate

### Changes

- Added one shared Core employee/choice evaluator used by action capability, payload validation,
  staged projection and resolve-time revalidation.
- Implemented stored aptitude +3, highest-only equipped +1/+2, owned-instance UUID tie-break,
  same-performer equipped key evidence, condition-before-key reason priority and exact d20 chance
  enumeration.
- Added natural 1/20 priority over modified totals and persisted the complete evaluation/provenance in
  `EventCommittedCheckState`.
- Preserved node-seeded choice/employee-independent raw d20 generation and atomic grant/combat
  planning; invalid requests commit nothing, exact duplicates and checkpoint/retreat paths reuse the
  full stored result.
- Added `missing_event_choice_key` as a typed Core disabled reason; Phase 3 will expose it in the final
  choice-specific DTO.

### Exact validation and results

1. First post-implementation Event focused attempt:

   ```bash
   cargo test -p game_core event_ --lib -- --test-threads=1
   ```

   Result: 48 passed, 3 failed, 0 ignored; 703 filtered out. Two assertions still expected old test DC
   values and unknown-employee validation temporarily mapped to `content_unavailable`. Expectations
   were migrated and unknown identity restored to `no_eligible_payload`.

2. Second Event attempt after adding evaluator/checkpoint tests:

   Result: 52 passed, 2 failed, 0 ignored; 704 filtered out. One test tried checkpoint load while the
   Event was still active (load is valid only from `ViewingMap`), and one tried a loadout command
   inside Event context. Tests were corrected to complete before load and to mutate the authoritative
   owned/equipped state directly for stale-preview revalidation.

3. Focused final Event suite:

   ```bash
   cargo test -p game_core event_ --lib --quiet -- --test-threads=1
   ```

   Result: 55 passed, 0 failed, 0 ignored; 704 filtered out.

4. Pre-check checkpoint raw-roll identity test:

   ```bash
   cargo test -p game_core precheck_checkpoint_reuses_raw_roll_for_different_valid_choice_and_employee --lib --quiet -- --test-threads=1
   ```

   First attempt: 0 passed, 1 failed because load was attempted while Event was active. Final result:
   1 passed, 0 failed, 0 ignored; 757 filtered out. The corrected test saves at `ViewingMap`, submits an
   invalid choice without commit, resolves one choice/employee, completes, loads, then resolves a
   different valid choice/employee and observes the identical raw d20.

5. Core full library regression:

   ```bash
   cargo test -p game_core --lib --quiet -- --test-threads=1
   ```

   Final result after all Phase 2 edits: 759 passed, 0 failed, 0 ignored.

6. game_server affected-type compilation:

   ```bash
   cargo check -p game_server --all-targets
   ```

   First attempt failed compilation in two server test initializers missing `event_aptitude` and the
   two neutral equipment Event arrays. The fixtures were migrated. Final result: exit 0; `game_core`
   and `game_server` all targets compiled.

7. Formatting and scoped diff checks:

   ```bash
   cargo fmt --all --check
   git diff --check -- <Phase 2 Core/server paths>
   ```

   Results: both exit 0.

### Directly pinned invariants

- 480-case raw-face/DC/modifier outcome matrix plus exact chance table for DC 8/12/16/20 and total
  modifiers 0..5.
- Stored-profile match/mismatch, highest-only, UUID tie, inventory-only exclusion, non-consumed key,
  key-without-implicit-bonus and condition reason priority.
- Projection/resolve evaluator equivalence after equipment changes and resolve-time missing-key
  rejection without commit.
- Invalid choice before a valid resolve does not change the later raw roll; a pre-check checkpoint
  with different valid choice/employee yields the same raw roll.
- Full committed provenance survives exact duplicate, post-commit checkpoint load and combat retreat;
  reward and combat success paths remain single-commit/atomic under existing regressions.

### Gate decision

Phase 2 changed from `in_progress` to `gate_passed`. Core owns every gameplay calculation and stored
result; Phase 3 may finalize only the outward DTO/serialization/server/actual-fixture contract.

## 2026-07-19 — Phase 3 Started

### Hypothesis and boundary

The staged Event projection can be replaced by one exact choice-specific DTO carrying Core-authored
evaluations and committed provenance, while the existing identity-only command and generic server
pass-through/order remain unchanged.

- Phase 3 changed from `not_started` to `in_progress`.
- Phase 4–6 remain blocked.
- Unity files and live checked Event content are out of this Phase.

## 2026-07-19 — Phase 3 DTO, Serialization, Server And Actual Fixture Gate

### Changes

- Removed the staged scene-common `required_roll` and global employee list from the outward Event DTO.
- Added choice-specific aptitude/DC/key/eligibility and per-choice employee evaluations with typed
  modifier components, key evidence and exact chance.
- Expanded committed-result DTO to exact source scene, choice/employee, aptitude/DC, raw roll, typed
  components, key evidence, modifier/total roll and outcome.
- Added strict unknown-field rejection to the Event DTO/evidence shapes. Resolve request remains the
  exact five-identity-field command and rejects injected modifier input.
- Replaced the manually assembled fixture generator with a real live-data `GameCore` run using actual
  starter creation/equipment, admin Event entry, Core projection and Core resolve. The generated file
  contains actual pre-check and committed selected-event payloads.
- Added game_server pass-through regression proving the new choice evaluation is preserved unchanged,
  produces no side message and requests the trailing root snapshot.

### Exact validation and results

1. First focused Core DTO run:

   ```bash
   cargo test -p game_core event_ --lib --quiet -- --test-threads=1
   ```

   Result: 54 passed, 1 failed, 0 ignored; the one stale assertion still expected scene-common
   `required_roll`. It was replaced with exact new-shape and strict legacy-rejection assertions.

2. Final focused Core DTO/Event run:

   Same command result: 55 passed, 0 failed, 0 ignored; 704 filtered out.

3. Phase 2 invalidation rerun after DTO/state cutover:

   ```bash
   cargo test -p game_core --lib --quiet -- --test-threads=1
   ```

   Result: 759 passed, 0 failed, 0 ignored.

4. game_server focused and full mapping/order validation:

   ```bash
   cargo test -p game_server event_state_mapping_preserves_choice_evaluation_and_requests_trailing_snapshot -- --test-threads=1
   cargo test -p game_server --quiet -- --test-threads=1
   ```

   Results: focused 1 passed, 0 failed, 0 ignored; full main target 31 passed, 0 failed, 0 ignored.
   Two auxiliary targets executed zero tests and are not counted as behavior evidence.

5. Actual producer fixture and deterministic equality:

   ```bash
   cargo run -p game_core --bin generate_event_contract_fixture --quiet > /tmp/event_employee_check_contract.json
   cargo run -p game_core --bin generate_event_contract_fixture --quiet > /tmp/event_employee_check_contract_second.json
   cmp -s /tmp/event_employee_check_contract.json /tmp/event_employee_check_contract_second.json
   diff -u /tmp/event_employee_check_contract_second.json '/mnt/f/unity projects/ark/Assets/Tests/Fixtures/event_employee_check_contract.json'
   ```

   Results: generator exits 0; repeated output is byte-identical; checked-in Unity fixture is exact;
   diff exit 0. Actual committed fixture pins raw 11 + equipment 2 = total 13 against DC 16,
   `failure`, with exact `archive_scanner` modifier and key evidence.

6. Formatting, scoped diff and legacy inventory:

   ```bash
   cargo fmt --all --check
   git diff --check -- <Phase 3 Core/server paths>
   rg -n 'required_roll|EventChoiceSnapshotDto|EventEmployeeCandidateDto' <Phase 3 production paths>
   ```

   Formatter and scoped diff exit 0. No legacy symbol remains in production DTO/generator/server
   mapping; the only server occurrence is a negative assertion that `required_roll` is absent.

### Gate decision

Phase 3 changed from `in_progress` to `gate_passed`. Core producer, generic server transport and the
actual fixture agree on one strict DTO. Phase 4 may migrate and validate the accessible Unity
consumer; the fixture file is the only Unity artifact changed before Phase 4.

## 2026-07-19 — Phase 4 Started

### Hypothesis and boundary

Unity can strictly consume and present the actual Core-authored choice evaluations and committed
result without calculating any gameplay value, while preserving local panel/text/selection cursor and
the identity-only command.

- Phase 4 changed from `not_started` to `in_progress`.
- Phase 5–6 remain blocked.
- This Phase may change Unity Event reader/store/composer/view/tests and consume the actual fixture.
  It must not add the production checked Event or alter Core gameplay policy.

## 2026-07-19 — Phase 4 Unity Consumer And Presentation Gate

### Changes

- Replaced the Event reader/model with a strict final-contract parser for choice-owned aptitude/DC/key,
  nested employee evaluations, typed modifier components, key evidence, exact chance and committed
  result provenance. Legacy scene-common roll/global-candidate fields are rejected.
- Migrated the Event composer to choice-first presentation. It displays only Core-authored eligibility,
  modifiers, chance, key evidence and committed roll/result; it does not derive gameplay values.
- Preserved the identity-only resolve command and local-only panel/text cursor. Changing a choice now
  clears only the local employee selection.
- Migrated the Event mock and focused EditMode/PlayMode tests to the actual Rust fixture and final DTO.

### Exact validation and results

1. Unity compilation/domain reload and console audit:

   ```text
   Unity refresh/domain reload via MCP; read_console(types=[error], count=10)
   ```

   Result: compilation completed; 0 error entries.

2. Focused EditMode contract suite:

   ```text
   Unity Test Runner EditMode: ArkClient.Tests.EditMode.EventFlowContractTests
   ```

   Initial fixture-staleness attempt: 6 total, 2 passed, 4 failed because Unity AssetDatabase had not
   imported the externally copied fixture. After a full asset refresh, the same six passed. A later
   expanded-suite initialization timed out with 0 executed tests and is not completion evidence.
   Final job `e78c3c61a9ba49499802ac05f0c96c8d`: 10 passed, 0 failed, 0 skipped.

3. Focused PlayMode Event render suite:

   ```text
   Unity Test Runner PlayMode: ArkClient.Tests.PlayMode.EventFlowRenderTests
   ```

   Job `0d2bf658cf40431394692b976b22c56d`: 2 completed of 2, status succeeded,
   no failures reported. The runner's final response did not provide a separate pass/fail/skip
   summary for this focused job; both named tests completed successfully.

4. Full Unity EditMode regression:

   ```text
   Unity Test Runner EditMode: all tests
   ```

   Job `a3aba9d70c35494da34c631be19a8b39`: 244 passed, 0 failed, 0 skipped.

5. Full Unity PlayMode regression:

   ```text
   Unity Test Runner PlayMode: all tests
   ```

   Job `c056d256659c4469ae5d56acbb092a43`: 62 total, 58 passed, 0 failed, 4 skipped.
   All four skips are explicit live-server tests requiring a separately running game_server; they are
   scheduled as actual transport evidence in Phase 5/6 and are not hidden failures.

6. Actual fixture, scoped diff and legacy checks:

   ```bash
   diff -u /tmp/event_employee_check_contract_final.json '/mnt/f/unity projects/ark/Assets/Tests/Fixtures/event_employee_check_contract.json'
   git diff --check -- <Phase 4 Unity paths>
   rg -n 'RequiredRoll|required_roll|interaction\?\["employee_candidates"\]|interaction\["employee_candidates"\]' <Phase 4 Unity paths>
   ```

   Fixture diff and scoped diff check exit 0. No production reader/composer/mock legacy field remains;
   the only `required_roll` matches are two strict negative tests that inject the removed field and
   assert fail-closed rejection.

### Gate decision

Phase 4 changed from `in_progress` to `gate_passed`. Actual Core fixture, server pass-through and Unity
strict consumer/presentation now agree on one final contract without client repair or local gameplay
calculation. Phase 5 may begin live-content authoring and actual end-to-end transport validation.

## 2026-07-19 — Phase 5 Started

### Hypothesis and boundary

A new production-loaded, generated-map-reachable checked Event can demonstrate the approved two-choice
vertical slice through Core, game_server WebSocket and Unity presentation without changing the Phase
1–4 gameplay/wire contract.

- Phase 5 changed from `not_started` to `in_progress` only after the Phase 4 gate passed.
- Phase 6 remains blocked.
- Phase 5 may author the first live checked Event, its explicit reachability policy, focused live-flow
  tests and the registered transport probe. Any change to Phase 1–4 owned artifacts invalidates and
  requires rerunning the corresponding gate evidence.

## 2026-07-19 — Phase 5 Reachable Live Vertical Slice And End-To-End Gate

### Changes

- Added production Event `sealed_archive_recovery_01` with exactly one checked entry scene and four
  terminal result scenes.
- Added two meaningfully different choices:
  - Analysis DC 16 `decode_archive_seal`, requiring `sealed_archive_access`, granting 80 Enkephalin on
    success and routing to decode-specific success/failure scenes.
  - Maintenance DC 12 `stabilize_archive_power`, no key, granting 30 Enkephalin on success and routing
    to power-specific success/failure scenes.
- Added weighted generated-map definition `event_sealed_archive` without changing WhiteNight's
  advance-only content.
- Added live contract/reachability, candidate/outcome/reward/invalid/duplicate/checkpoint regressions.
- Added standard-library-only registered actual WebSocket probe at
  `game_server/docs/probe/event_employee_check_ws_probe.py`.
- Migrated the Unity Event mock to the live Event and added live mock PlayMode rendering/interaction
  coverage.
- Fixed a cross-layer duplicate defect found by the actual probe: capability preflight had rejected an
  exact already-committed resolve before the stored-result reuse handler. The preflight now bypasses
  capability evaluation only when node/event/source scene/choice/employee all exactly match the
  committed result. Normal post-commit capability presentation remains disabled and nonmatching
  commands remain rejected.

### Exact validation and results

1. Live RON parse/contract during authoring:

   ```bash
   cargo test -p game_core --test ron_loading --quiet -- --test-threads=1
   cargo test -p game_core --test ron_loading live_sealed_archive_event_and_reachable_node_pin_the_vertical_slice_contract -- --exact --test-threads=1
   ```

   The first post-RON run passed the pre-existing 22/22. The new exact live contract test passed 1/1.
   Final full live RON result after adding it: 23 passed, 0 failed, 0 ignored.

2. Generated-map reachability:

   ```bash
   cargo test -p game_core generated_map_can_reach_the_live_sealed_archive_event_from_start --lib -- --test-threads=1
   ```

   Result: 1 passed, 0 failed, 0 ignored; 759 filtered out. An earlier command incorrectly included
   `--exact` without the module-qualified name and executed 0 tests; it was rejected as evidence and
   rerun correctly.

3. Live Core vertical slice:

   ```bash
   cargo test -p game_core live_sealed_archive --lib -- --test-threads=1
   ```

   Initial compile exposed test-only access/Hash assumptions; the test was rewritten to exercise the
   public behavior path and use explicit outcome slots. A later 4-test run had 3 pass/1 fail because
   the test expected a generic reason instead of the actual typed `missing_event_choice_key`; after
   correction the specific test passed 1/1. After the actual duplicate-preflight repair, final result:
   4 passed, 0 failed, 0 ignored; 759 filtered out.

   These tests pin live choice-specific candidates/chances, missing-key no mutation, both ordinary and
   both critical branches, 80/30 success grants, exact duplicate no double grant, nonmatching duplicate
   rejection and pre-check checkpoint raw-roll reuse across different choices.

4. Invalidated Phase 2/Core gates:

   ```bash
   cargo test -p game_core event_ --lib --quiet -- --test-threads=1
   cargo test -p game_core --lib --quiet -- --test-threads=1
   ```

   First Event rerun: 55 passed, 1 failed because an adjacent existing all-candidates-missing-key test
   expectation had been accidentally changed while correcting the new test. The original capability-
   level `no_eligible_payload` expectation was restored. Final Event result: 56 passed, 0 failed,
   0 ignored; 707 filtered out. Full Core result: 763 passed, 0 failed, 0 ignored.

5. game_server affected/broad validation:

   ```bash
   cargo check -p game_server --all-targets
   cargo test -p game_server --quiet -- --test-threads=1
   ```

   Results: check exit 0; server main target 31 passed, 0 failed, 0 ignored. Two auxiliary targets
   executed zero tests and are not behavior evidence.

6. Actual game_server WebSocket flow:

   ```bash
   cd game_server
   APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run
   python3 game_server/docs/probe/event_employee_check_ws_probe.py --port 18083 --admin-token dev --player-id 00000000-0000-0000-0000-00000000a120
   ```

   The first server attempt from the workspace root failed because `config/development.toml` is owned
   relative to `game_server`; the corrected working directory loaded data but sandbox port binding was
   denied, then the approved local-port run succeeded. A non-escalated probe connection was likewise
   sandbox-denied and rerun with local-loopback permission.

   The first actual flow reached reconnect/resolve and then timed out on duplicate. Server evidence
   showed the success grant had committed; diagnosis found the preflight/handler ordering defect above.
   After repair and rebuild, the final probe passed with 30 assertions: auth, command-result/snapshot
   ordering, live starter/equipment path, exact two-choice DTO, key reason, modifier/chance, active-
   session reconnect, committed provenance, exact duplicate reuse and no duplicate grant. A second
   final run with player `...a120` also passed. Server then shut down cleanly.

7. Unity live mock and invalidated Phase 4 gates:

   ```text
   Unity PlayMode: ArkClient.Tests.PlayMode.EventFlowRenderTests
   Unity EditMode: ArkClient.Tests.EditMode.EventFlowContractTests
   Unity EditMode: all tests
   Unity PlayMode: all tests
   ```

   First live-mock PlayMode job `90a4cd30e1b04562a51511045351c451`: 3 total, 2 passed, 1 failed
   because the test assumed a fixed reveal-cursor step. The test was changed to advance the real local
   cursor until choices are visible. Second job `b3ab5254edd5483e84232af1e95ec0fa`: 2 passed, 1 failed
   because the test expected disabled reason to render as raw button text; it now asserts the typed
   ViewModel reason while rendering the visible policy details. Final focused PlayMode job
   `7cf7af62f02246318a9c7e2cb5b221f9`: 3 completed of 3, status succeeded, no failures reported.

   Focused EditMode job `b6dcd283facc487d8f977aec9829bae2`: 10 passed, 0 failed, 0 skipped.
   Full EditMode job `0fc1a3a474984d3ea1501aee5670d3dc`: 244 passed, 0 failed, 0 skipped.
   Full PlayMode job `dc1e1a51558547d2bf0b754d79391f11`: 63 total, 59 passed, 0 failed,
   4 explicit live-server skips. The registered Python WebSocket probe supplies this Goal's actual
   running-server Event evidence. Final cleared console audit: 0 error entries.

8. Fixture, format, scoped diff and legacy checks:

   ```bash
   cargo run -p game_core --bin generate_event_contract_fixture --quiet > /tmp/event_employee_check_contract_phase5.json
   diff -u /tmp/event_employee_check_contract_phase5.json '/mnt/f/unity projects/ark/Assets/Tests/Fixtures/event_employee_check_contract.json'
   cargo fmt --all --check
   git diff --check -- <Phase 5 Core/data/server/Goal paths>
   git diff --check -- <Phase 5 Unity paths>
   rg -n 'required_roll|EmployeeCandidates|employee_check_fixture|event_fixture_choice' <Phase 5 production paths>
   ```

   Fixture diff, formatter and both scoped diff checks exit 0. Production live/Core/probe and Unity
   mock legacy searches returned no matches; `rg` exit 1 is the expected no-match result.

### Gate decision

Phase 5 changed from `in_progress` to `gate_passed`. The production-loaded Event is generated-map
reachable and demonstrates all approved vertical-slice conditions. Core, actual server transport and
accessible Unity presentation consume the same authoritative result. Phase 6 may begin canonical
cutover, final broad regression and independent completion review.

## 2026-07-19 — Phase 6 Started

### Hypothesis and boundary

The combined implementation can now be described by nearest durable canonical documents and pass one
final independent legacy/producer-consumer/broad-regression review without policy or compatibility
gaps.

- Phase 6 changed from `not_started` to `in_progress` only after the Phase 5 gate passed.
- No new gameplay/content policy is authorized. Any material correction invalidates the owning Phase
  gate and must be rerun before completion.

## 2026-07-19 — Phase 6 Canonical Cutover, Broad Regression And Completion Gate

### Canonical changes

- Updated `core_enhanced/docs/game_rulebook.md` with final aptitude/modifier/key/DC/d20/persistence and
  live vertical-slice gameplay semantics.
- Updated `core_enhanced/docs/core_runtime_contract.md` with the exact choice-specific DTO, Core
  authority, committed provenance and exact-duplicate execution boundary.
- Updated `core_enhanced/docs/data_loading_contract.md` with strict employee/Event/equipment schema,
  cross-catalog key and live Event/map/provider ownership.
- Updated `game_server/docs/unity_bridge_protocol.md` with exact Event command, result/snapshot order,
  reconnect and duplicate delivery semantics.
- Updated `core_enhanced/docs/probe_registry.md` with the game_server-owned Event probe route and
  exact server/probe command.
- Updated external `docs/unity_core_contract.md` and `docs/ui/screen_spec.md` with final wire example,
  client non-authority, choice-first presentation and committed display.

### Exact final validation and results

1. Full Core all-targets:

   ```bash
   cargo test -p game_core --all-targets -- --test-threads=1
   ```

   Result: exit 0. Behavior/data test targets executed 810 tests total, all passed:
   - Core lib: 763 passed, 0 failed, 0 ignored.
   - `live_item_skill_activation`: 3 passed.
   - `live_skill_catalog_audit`: 4 passed.
   - `ron_loading`: 23 passed.
   - `skill_runtime_contract`: 13 passed.
   - `skill_test_suite`: 4 passed.

   Main/generator binary unit targets each reported 0 tests and are not counted as behavior evidence.

2. Full game_server:

   ```bash
   cargo check -p game_server --all-targets
   cargo test -p game_server --quiet -- --test-threads=1
   ```

   Results: check exit 0; main target 31 passed, 0 failed, 0 ignored. Two auxiliary zero-test targets
   are not counted.

3. Final Unity broad suites and console:

   ```text
   Unity EditMode all — job d9017ef3ac984f6eab319226720df1c1
   Unity PlayMode all — job e9021e8278aa491faf493fc2deb6f69b
   Unity console clear + error read
   ```

   Results: EditMode 244 passed, 0 failed, 0 skipped. PlayMode 63 total, 59 passed, 0 failed,
   4 explicit live-server skips. The skipped tests are unrelated broad live-admin/maintenance/combat/
   endless smoke requiring port 8081; this Goal's mandatory actual Event flow ran separately through
   the registered probe. Final Unity error console: 0 entries.

4. Final actual server probe:

   ```bash
   cd game_server
   APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run
   python3 game_server/docs/probe/event_employee_check_ws_probe.py --port 18083 --admin-token dev --player-id 00000000-0000-0000-0000-00000000a121
   ```

   Result: PASS, 30 assertions, reconnect true, exact duplicate reused, no duplicate grant. Server
   shut down cleanly after the probe.

5. Final producer fixture/format/diff/legacy inventory:

   ```bash
   cargo run -p game_core --bin generate_event_contract_fixture --quiet > /tmp/event_employee_check_contract_phase6.json
   diff -u /tmp/event_employee_check_contract_phase6.json '/mnt/f/unity projects/ark/Assets/Tests/Fixtures/event_employee_check_contract.json'
   cargo fmt --all --check
   git diff --check -- <all Goal-owned Core/data/server/docs paths>
   git diff --check -- <all Goal-owned Unity/docs paths>
   rg -n <removed Event schema/client recalculation/compatibility patterns> <production and canonical paths>
   ```

   Results: fixture diff 0, formatter exit 0, both scoped diff checks exit 0. Removed-field searches
   found only explicit canonical statements that `required_roll` is absent and strict negative tests/
   server assertions; no production authority or live data occurrence remains. Unity calculation
   search found only assignment of the already-parsed Core chance. No Event-specific dual schema,
   fallback, alias or repair remains.

### Independent completion review

Every user-visible completion condition and invariant was mapped to direct evidence:

- Source of truth: stored Profile aptitude + choice/equipment live RON feed the single Core evaluator;
  projection and resolve call the same function.
- One-roll/state boundary: exhaustive outcome/chance tests, invalid no-mutation, live checkpoint and
  exact duplicate/reconnect probe preserve the same committed raw roll/provenance and no double grant.
- Producer/consumer: actual Core-generated fixture exactly matches Unity, server pass-through/order is
  tested, and Unity strict reader/composer/full suites are green without local calculation.
- Live content: `event_sealed_archive` can place `sealed_archive_recovery_01` on a connected generated
  path; the two choices demonstrate different aptitude/DC/key/reward/scenes and meaningful employees.
- Legacy/compatibility: live RON, DTO, server, Unity production/mock and current canonical use only the
  final shape; negative tests are the only old-field code occurrences.
- Baseline/dirty safety: no reset/restore/clean/stash was used; unrelated tracked/untracked work remains
  present. All Goal target files were patched in their current dirty state and final scoped whitespace
  checks pass.

The review found no material correction candidate, unresolved policy, inaccessible required consumer,
or missing completion evidence.

### Gate decision

Phase 6 changed from `in_progress` to `gate_passed`. Goal Status changed to `complete` and
Implementation complete to `true` only after all seven ordered Phase gates passed.
