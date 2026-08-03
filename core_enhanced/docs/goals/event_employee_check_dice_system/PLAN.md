# Event Employee Check Dice System

Status: complete

Implementation complete: true

Created: 2026-07-18

Last updated: 2026-07-19

Goal name: `event_employee_check_dice_system`

Goal type: single cross-layer contract implementation

Approved policy input:
[event_employee_check_dice_policy_2026-07-17.md](../../decisions/event_employee_check_dice_policy_2026-07-17.md)

Audit finding ID: none. This Goal comes from an explicitly approved gameplay policy, not an audit
finding.

Target repositories:

- Core, live RON and game_server: `/mnt/f/work/simulator`
- Accessible Unity consumer and canonical: `/mnt/f/unity projects/ark`

## Objective And User-Visible Problem

The baseline Event employee check used one authoritative pure d20 roll but gave every eligible
employee the same chance. Selecting a performer therefore had identity but little gameplay meaning.

Implement one complete Event employee-check contract in which each checked choice owns an aptitude
and DC, each employee owns one stable Event aptitude, equipped items can provide a typed modifier or
non-consumed choice key, Core projects exact per-choice/per-employee evaluation, and Core commits one
deterministic roll with its full typed provenance. Unity presents the authored choices, candidates,
odds, reasons and committed result without recalculating them.

## Single-Goal Boundary

This Goal covers one tightly coupled source-of-truth boundary:

```text
Employee/Event/Equipment live authoring
  -> Core validation and authoritative evaluator
  -> one-shot Event state mutation and persistence
  -> typed behavior/snapshot DTO
  -> game_server pass-through and ordering
  -> Unity reader/store/command/presentation
  -> first live employee-check Event
```

The layers cannot safely become independently complete because old and new choice/check DTOs must
not coexist as competing authority. This is one contract migration rather than multiple independent
gameplay Goals.

## Current Runtime And Live-Data Evidence

Direct implementation and final verification established the current contract:

- Event remains an independent node category with `advance` and choice-specific
  `employee_check_choices` interactions.
- Resolve uses exact
  `ResolveEventChoiceCheck(node_id,event_id,current_scene_id,choice_id,employee_uuid)` identity.
- Stored Profile aptitude, Event choice DC/key and equipped-owned equipment metadata feed one shared
  Core evaluator used by projection and resolve-time revalidation.
- Core commits one node-seeded raw d20 and typed modifier/key/outcome provenance in
  `EventSessionState.committed_check`; exact duplicate, reconnect, checkpoint and retreat/retry reuse it.
- `EmployeeActivity::Event` owns condition eligibility; condition is not a numeric modifier.
- Live `sealed_archive_recovery_01` is a generated-map-reachable two-choice checked Event using
  `archive_scanner` and the authored starter roster path.
- Live `white_night_confession_01` remains advance-only.
- Baseline investigation found no checked live Event, deployed compatibility consumer or external
  durable save/database requiring legacy migration.

## Source Of Truth And Ownership

### Current authority before migration

- Actual runtime code and production-loaded live RON/data remain current authority.
- Existing durable Core/Unity canonical describes the current pure-d20 contract.
- This PLAN and its approved policy input describe the approved target, not current gameplay.

### Target authority after migration

- `EmployeeDefinition.event_aptitude` is required authoring input copied once when an employee
  profile is created.
- Stored `EmployeeProfile.event_aptitude` is the run/checkpoint runtime authority; current definitions
  do not overwrite an existing profile.
- Event choice RON owns exactly one aptitude, one allowed DC and at most one choice-key requirement.
- Equipment RON owns typed Event modifier/key metadata.
- Core owns eligibility, key-provider selection, modifier composition, exact odds, RNG draw, outcome,
  atomic commit and persisted provenance.
- game_server transports typed Core output without recalculation.
- Unity owns local text/image reveal and presentation only; it does not calculate eligibility, DC,
  modifier, key authority, probability, roll or outcome.
- After implementation, nearest Core and external Unity canonical documents own the durable contract;
  this Goal remains execution history.

## Required Invariants

- Core remains the sole gameplay and RNG authority.
- Only a valid first resolve consumes exactly one d20 draw and commits state.
- Choice, employee, DC, aptitude and equipment do not alter the raw-roll RNG input; the same Event and
  RNG continuation yields the same raw roll even when a pre-check checkpoint is restored and another
  valid choice/employee is submitted.
- Invalid/stale/ineligible/missing-key payloads consume no RNG and mutate no Event state.
- Natural 1 is always `critical_failure`; natural 20 is always `critical_success`.
- Duplicate, reconnect, checkpoint, retry and retreat return the exact committed result without
  recalculation, another success action or another RNG draw.
- One Event definition has at most one reachable checked scene, and no checked-scene return cycle.
- Condition affects Event eligibility only, not numeric roll modifiers.
- Personality, trust, memory, level, experience, battle tier, `CombatAptitude`, consumables and skill
  fragments do not modify the roll.
- Only the selected performer's currently equipped owned item instances can contribute.
- Equipment modifiers are highest-only, never summed; UUID order deterministically resolves ties.
- Choice keys are non-consumed, provide no implicit numeric bonus and must belong to the performer.
- Preview is not commit authority. Core re-evaluates current Profile, Condition and equipped items at
  resolve time.
- Result effects remain staged/atomic with Event session and committed-check mutation.
- Existing advance-only WhiteNight content is not converted into a checked Event.
- No compatibility alias, default repair, dual-read, dual-write or Unity inference is added.

## Approved Gameplay And Data Policy

### Employee aptitude

Each employee has exactly one stable aptitude:

```text
Medical
Security
Analysis
Reconnaissance
Containment
Maintenance
```

The 12 approved employee-definition assignments are:

| Aptitude | Starter | Recruitment |
| --- | --- | --- |
| Medical | `field_medic` | `hq_field_medic` |
| Security | `breach_guard` | `hq_security_guard` |
| Analysis | `records_clerk` | `hq_records_observer` |
| Reconnaissance | `scout_runner` | `hq_recovery_runner` |
| Containment | `containment_trainee` | `hq_containment_aide` |
| Maintenance | `maintenance_tech` | `hq_maintenance_aide` |

An aptitude match contributes `+3`; mismatch contributes `+0` and is not ineligibility.

### Choice check

Every choice in `employee_check_choices` owns:

```text
check:
  aptitude: EventAptitude
  dc: 8 | 12 | 16 | 20
  required_choice_key: Option<EventChoiceKeyId>
```

Scene-level `required_roll` is removed. Unchecked progression continues to use `advance` scenes.

### Formula and chance

```text
employee_modifier = aptitude match ? 3 : 0
equipment_modifier = highest applicable equipped modifier, 0..2
modifier_total = employee_modifier + equipment_modifier
total_roll = raw_roll + modifier_total

raw_roll == 1  -> critical_failure
raw_roll == 20 -> critical_success
total_roll >= dc -> success
otherwise -> failure
```

Allowed equipment modifier amounts are `+1` and `+2`. `success_chance_percent` is the number of raw
d20 faces producing `success` or `critical_success`, multiplied by five. Critical success/failure
remain separate 5% presentation outcomes.

Approved probability table:

| DC | No aptitude/equipment | Aptitude +3 | Aptitude + equipment +2 |
| ---: | ---: | ---: | ---: |
| 8 | 65% | 80% | 90% |
| 12 | 45% | 60% | 70% |
| 16 | 25% | 40% | 50% |
| 20 | 5% | 20% | 30% |

### Equipment modifier and key

Equipment definitions may author:

```text
event_check_modifiers[] { aptitude, amount: 1 | 2 }
event_choice_keys[] EventChoiceKeyId
```

- Duplicate aptitude modifiers/keys inside one definition and invalid amounts/IDs fail strict
  validation.
- A required key has a published equipment provider.
- Multiple equipped providers use the lowest instance UUID deterministically.
- Evaluation and committed result preserve typed `EventChoiceKeyEvidence` with key ID, equipment
  instance UUID and definition ID.
- Equipment modifier provenance stores instance UUID, definition ID, aptitude and amount.
- Event choice and equipment do not directly reference each other's Event/choice/equipment IDs.

### DTO and committed result

Core projects choice-specific evaluations containing authored choice/check data, choice eligibility,
employee eligibility/reasons, employee/equipment/total modifiers, typed components, optional key
evidence and exact success chance.

The resolve request retains identity fields only. It does not accept client-authored DC, modifier,
key or probability.

Committed result preserves at least:

```text
source_scene_id
choice_id
employee_uuid
aptitude
dc
raw_roll
modifier_components[]
choice_key_evidence?
modifier_total
total_roll
outcome
```

## Unresolved Policy

None in the approved scope.

If investigation shows a required external durable save/database, deployed legacy checked payload,
multiple-check production content, or a consumer requiring compatibility, stop with
`policy_decision_required` or `externally_blocked` as appropriate. Do not invent migration or
compatibility policy.

## In Scope

- Current command/state/RNG/rollback and producer-consumer discovery for this Event boundary
- Required employee aptitude schema, profile creation, import/checkpoint persistence and all 12 live
  employee definitions
- Choice-owned Event check schema and strict graph/data validation
- Equipment Event modifier/key schema and cross-catalog validation
- Core evaluator, exact chance projection, resolve revalidation and atomic committed result
- Typed behavior/snapshot DTO migration and exact serialization shape
- game_server pass-through/message ordering and actual producer fixture/probe
- Unity reader/store/selection/command/presentation for choice evaluations and committed result
- One new live employee-check Event and explicit map/pool reachability
- Focused, integration and broad validation
- Nearest Core/server/external Unity canonical updates after implementation stabilizes
- Final independent completion review of this single Goal

## Explicitly Out Of Scope

- Converting `white_night_confession_01`
- More than one employee check or roll history per Event
- Advantage/disadvantage, reroll, reroll resource or bonus dice
- Assistance, passive/opposed/group checks
- Multiple choice keys, AND/OR, key quantity, consumption, hidden discovery or reveal persistence
- Personality/trust/memory/continuous condition roll modifiers
- Psychology warning/refusal behavior
- Performer-targeted failure condition or critical-only gameplay rewards/effects
- Generic progression, rank, proficiency or aptitude growth
- Active consumable or skill-fragment roll effects
- Equipment type/name/combat stat inference
- Karmic dice or hidden streak correction
- General Event UX redesign unrelated to the approved semantic screen
- Independent refactors, unrelated content/balance work or new Goal/Master creation

Out-of-scope discoveries may be recorded as follow-up candidates only.

## Implementation Scope

The concrete file/module list is discovery pending. Implementation must cover the complete current
owner chain rather than a test-only or one-layer patch. Work is divided into seven mandatory internal
phases below. They are not separate Goals and do not authorize intermediate legacy/new schemas as
supported release states.

## Mandatory Phase Protocol

The following rules are execution requirements, not recommendations.

- Execute Phase 0 through Phase 6 in order. Do not begin production mutation belonging to a later
  Phase until the immediately preceding Phase exit gate is recorded as passed.
- Only one Phase may be `in_progress` at a time.
- Phase-local states are `not_started`, `in_progress`, `gate_failed`, and `gate_passed`. They are
  execution markers and do not replace the Goal-level Status values.
- At the start of each Phase, record its hypothesis, exact input state, affected dirty diffs and
  selected focused/broad validation in `EXPERIMENTS.md`/`EXPERIMENT_NOTES.md`.
- At the end of each Phase, run that Phase's mandatory validation. Record exact commands or Unity
  job IDs, actual executed test counts, pass/fail/skip counts, baseline comparison, failures,
  corrections and final rerun.
- A zero-test run, compile-only result where behavior is required, stale child evidence, or an
  unexecuted planned command cannot pass a Phase gate.
- Do not postpone a failed Phase validation to Phase 6. Set the Phase to `gate_failed`, keep the Goal
  `active` when safe correction remains, and fix/rerun within that Phase.
- If failure exposes an unapproved policy, scope split, inaccessible mandatory consumer or unsafe
  dirty overlap, change the Goal Status under the stated stop conditions instead of advancing.
- Phase exit evidence must cover every artifact changed during that Phase. Unrelated broad tests may
  wait for Phase 6 only when the Phase-specific gate still directly proves its own contract.
- Schema/DTO work may be internally staged across Phase 1–4, but no compatibility alias, dual schema,
  silent fallback or client repair may be introduced. The worktree is not a supported release
  candidate until the Phase 4 consumer gate passes.
- After a Phase gate passes, later work that modifies one of that Phase's owned artifacts invalidates
  the affected evidence. Record why it changed and rerun that Phase's relevant gate before final
  completion.

Initial Phase state:

| Phase | Name | Initial state | Dependency |
| --- | --- | --- | --- |
| 0 | Owner Discovery And Dirty Baseline | not_started | Goal initialization complete |
| 1 | Schema, Strict Validation And Base Live Data | not_started | Phase 0 gate |
| 2 | Core Evaluation, RNG, Commit And Persistence | not_started | Phase 1 gate |
| 3 | DTO, Serialization, Server And Actual Fixture | not_started | Phase 2 gate |
| 4 | Unity Consumer And Player Presentation | not_started | Phase 3 gate |
| 5 | Reachable Live Vertical Slice And End-To-End Flow | not_started | Phase 4 gate |
| 6 | Canonical Cutover, Broad Regression And Completion Review | not_started | Phase 5 gate |

### Phase 0 — Owner Discovery And Dirty Baseline

Purpose: replace every discovery-pending assumption with direct current evidence and establish a
reproducible pre-mutation baseline.

Required work:

- Trace the exact command entry through Event evaluation, RNG, state mutation, effect commit,
  checkpoint/retry/retreat and snapshot/result projection.
- Enumerate current employee/Event/equipment RON schema, production live entries, loader and strict/
  cross-catalog validation.
- Enumerate Core DTO serialization, game_server mapping/message ordering and accessible Unity reader,
  store, command, composer/view and fixture/mock paths.
- Classify all consumers as workspace-confirmed, canonical-only, inaccessible-known or unknown.
- Inspect exact current diffs for every likely target file in both dirty repositories.
- Run focused dirty-state baselines for current Event/data/Core/server/Unity behavior before any
  production mutation.
- Confirm current live data has no checked Event requiring compatibility and confirm whether any
  external durable save/database exists.

Mandatory validation:

- Current Event-focused Core tests with nonzero executed count
- Current live RON loading tests with nonzero executed count
- Current game_server Event/transport tests with nonzero executed count
- Current Unity Event-focused EditMode tests with nonzero executed count
- Applicable current Unity Event PlayMode baseline, or a direct recorded reason it is not yet
  applicable to the current flow
- Exact current serialized payload/fixture inspection and current legacy-symbol inventory

Exit gate:

- Current/target delta, owner map, dirty overlap decision, consumer classification, external-save
  result and every baseline count are recorded.
- No unresolved scope/policy/compatibility question or unsafe target-file overlap remains.
- No production artifact has been changed.

### Phase 1 — Schema, Strict Validation And Base Live Data

Purpose: establish fail-closed employee/Event/equipment authoring and stable base profile data without
yet claiming the complete player flow.

Required work:

- Add `EventAptitude` and required employee definition/profile creation/import/checkpoint shape.
- Author all 12 approved employee aptitude assignments.
- Add choice-owned check schema with only DC 8/12/16/20 and optional typed key.
- Remove scene-level checked `required_roll` schema without compatibility.
- Add equipment Event modifier/key schema and strict local/cross-catalog validation.
- Enforce one reachable checked scene and no checked-scene return cycle.
- Migrate existing fixtures/tests to the new current schema while preserving advance-only WhiteNight.
- Author only the base equipment metadata needed later; do not add the live checked Event yet.

Mandatory validation:

- Focused positive/negative schema tests for every new required/removed/unknown/duplicate/range rule
- Profile-copy/restore/import authority tests and all 12 live assignment audit
- Checked-scene count/cycle and broken reference tests
- Equipment modifier/key local and cross-catalog validation tests
- Production-shaped embedded bundle/`ron_loading` execution
- Core data-focused regression suite and Rust formatting/diff checks for Phase 1 artifacts

Exit gate:

- All current live data loads under only the new strict schema.
- Missing/legacy/invalid authoring fails before publication and no compatibility path remains.
- WhiteNight remains advance-only and unchanged in gameplay meaning.
- Every Phase 1 mandatory validation passes with recorded nonzero counts.

### Phase 2 — Core Evaluation, RNG, Commit And Persistence

Purpose: implement the authoritative gameplay behavior and prove its deterministic mutation and
rollback boundaries before client presentation work.

Required work:

- Implement one shared Core evaluator used by projection and resolve-time revalidation.
- Implement aptitude +3, highest-only equipped +1/+2, UUID tie-break, key gating/evidence and exact
  chance enumeration.
- Implement natural 1/20 outcome priority and total-roll comparison.
- Ensure a valid first resolve draws once; choice/employee does not influence raw-roll generation.
- Persist typed modifier/key provenance and full committed result.
- Preserve duplicate, reconnect, checkpoint, retry, retreat and staged success effect atomicity.
- Ensure invalid/stale/ineligible/missing-key commands consume no RNG/state.

Mandatory validation:

- Formula/chance exhaustive raw-face matrix for all allowed DC/modifier boundaries
- Highest-only/tie/key-provider and eligibility-priority tests
- Projection/resolve evaluator equivalence and stale preview revalidation tests
- RNG draw-count and invalid-command no-consumption tests
- Pre-check checkpoint restore with different valid choice/employee producing the same raw roll
- Exact duplicate/reconnect/checkpoint/retry/retreat reuse tests
- Success/failure/critical and reward/combat atomicity/no-duplicate-effect tests
- Core Event/world focused suite, Core library regression and game_server compile check if public Core
  types changed

Exit gate:

- Every approved gameplay invariant is directly pinned in Core with nonzero passing tests.
- No later layer performs or must perform a gameplay recalculation.
- Core state/persistence behavior is green; any affected server compilation is green.

### Phase 3 — DTO, Serialization, Server And Actual Fixture

Purpose: finalize the authoritative wire shape and server pass-through before Unity migration.

Required work:

- Finalize choice evaluations, candidate reasons/modifiers/key evidence/chance and committed-result DTO.
- Keep resolve request identity-only.
- Update exact serialization/deserialization and game_server typed mapping/message order.
- Produce an actual Rust-generated fixture/payload from the real Core producer.
- Remove legacy scene-common check fields and stale server/fixture assumptions.
- Record the exact new wire inventory and current Unity consumer delta for Phase 4.

Mandatory validation:

- Exact Core JSON shape tests using the actual producer
- Exact game_server top-level/nested mapping, correlation and message-order tests
- Actual fixture generation plus deterministic compare/drift test
- Core DTO-focused tests, full game_server tests and server check
- Legacy DTO/field search across Core, server and generated fixtures
- Scoped formatting/diff checks

Exit gate:

- Core and game_server expose one final typed wire contract and all server validation is green.
- The checked-in/consumed fixture is provably generated by the actual Core producer.
- No legacy server/fixture schema or compatibility mapping remains.
- Phase 4 has an exact consumer migration input; the worktree is still not a supported release until
  the Unity gate passes.

### Phase 4 — Unity Consumer And Player Presentation

Purpose: make the accessible client consume and present the final Core contract without local
authority or inference.

Required work:

- Migrate Unity reader/store/model to the actual Rust fixture and strict final fields.
- Implement choice-first selection and per-choice employee evaluation display.
- Display aptitude, DC, eligibility/reason, modifier components/total, exact chance and key provenance.
- Keep the resolve command identity-only and re-render the committed result from Core output.
- Preserve local illustration/text reveal and ensure animation skip/cursor never sends extra RNG/
  resolve commands.
- Remove legacy reader/mock/presenter assumptions; do not locally repair missing fields.

Mandatory validation:

- Unity compilation/domain reload and console error audit
- Focused EditMode reader/store/model/composer/command tests against the actual Rust fixture
- Disabled reason, key evidence, exact odds, critical/ordinary result and no-local-recalculation tests
- Focused PlayMode render/interaction tests for choice selection, performer selection, roll action,
  animation skip and committed display
- Full Unity EditMode suite after focused success
- Applicable ordinary PlayMode regression for touched presentation/command paths
- Scoped Unity diff/legacy-field checks

Exit gate:

- Unity compiles with zero relevant errors and every mandatory focused/full job has recorded passing
  counts.
- Actual Core fixture, server contract and Unity consumer agree without dual schema or repair.
- Phase 1–4 collectively form the first cross-layer release-candidate contract.

### Phase 5 — Reachable Live Vertical Slice And End-To-End Flow

Purpose: prove the approved contract in production-loaded content and actual transport/presentation,
not only isolated fixtures.

Required work:

- Author the first checked Event meeting the vertical-slice requirements below.
- Add an explicit reachable Event node or Event-pool policy and required equipment provider path.
- Prove success/failure/critical branches and supported reward or existing combat start.
- Exercise real candidate evaluation, modifier, key gating, committed result and rollback/reconnect.
- Run the registered live WebSocket flow through actual game_server and accessible Unity path where
  required by the probe contract.

Mandatory validation:

- Production embedded live RON loading and generated map/content reachability tests
- Core live Event success/failure/critical and invalid-command flows
- Live duplicate/reconnect/checkpoint and applicable Event-started combat retreat flow
- Actual game_server WebSocket probe with exact message/payload assertions
- Focused Unity live/mock PlayMode rendering/interaction flow as applicable
- Focused Core/server/Unity regressions for all artifacts changed after their prior Phase gates
- Scoped legacy/current-data and diff checks

Exit gate:

- A production-loaded, actually reachable checked Event demonstrates every vertical-slice condition.
- Actual transport and accessible presentation consume the same Core-authored result.
- All Phase 5 mandatory validations and invalidated prior gates pass with recorded counts.

### Phase 6 — Canonical Cutover, Broad Regression And Completion Review

Purpose: independently verify the combined repository state, move durable documentation to the actual
contract and close only when no material gap remains.

Required work:

- Update nearest gameplay, Core runtime, data/loading, server transport, Unity wire and relevant UX
  canonical from direct implementation evidence.
- Search every current runtime/data/DTO/server/Unity fixture/mock/test/canonical surface for removed
  `required_roll`, legacy candidate shape, client recalculation, compatibility and fallback.
- Rerun every gate invalidated by later changes.
- Run all required broad validation and an independent completion review mapping each visible
  condition/invariant to code, live data, producer-consumer evidence and tests.
- Record exclusions, pre-existing failures, remaining risks and dirty-worktree preservation.

Mandatory validation:

- Rust formatting and full `game_core --all-targets` including live RON tests
- Full game_server check and tests
- Full Unity EditMode and ordinary PlayMode suites with explicit skip classification
- Unity console error audit
- Actual fixture drift comparison and final registered live WebSocket probe
- Scoped Core/external diff checks and exact legacy/current/fallback searches
- Independent completion review with correction candidates explicitly classified

Exit gate:

- Every mandatory command/job reports passing counts; no required validation is missing.
- Canonical, Core, live data, server, fixture and Unity describe one final contract.
- No material legacy, compatibility, fallback, source-of-truth split or immediate correction remains.
- Only then set Phase 6 to `gate_passed`, Goal Status to `complete`, and Implementation complete to
  `true`.

## First Live Vertical Slice

The first checked Event must demonstrate:

- at least two choices with different aptitudes and DCs;
- meaningfully different success/failure consequences or risk/reward;
- matching and non-matching employee probabilities;
- one equipped-item modifier;
- one optional key-gated choice and typed missing-key reason;
- one realistic non-matching employee alternative made relevant by modifier or key;
- success and failure panels;
- one supported success reward or existing combat start;
- committed display of raw roll, modifier provenance, total, DC and outcome;
- exact duplicate, reconnect/checkpoint and applicable Event-started combat retreat stability.

It is not required to author every aptitude, key or future outcome effect as content.

## User-Visible Completion Conditions

- In the new live Event, the player first selects a checked choice and sees every candidate's Core-
  authored aptitude, eligibility/reason, exact modifier breakdown and exact success chance.
- A matching employee has +3; an applicable equipped item contributes only the deterministic highest
  +1/+2; a key-gated choice is disabled for candidates without the equipped key.
- The resolve action sends identity only, rolls once in Core and shows raw roll, modifier components,
  total, DC, critical/ordinary outcome and key provenance where applicable.
- Changing local selection/animation or reconnecting cannot change/consume the roll. Duplicate,
  checkpoint and retreat/retry behavior returns the exact committed result and does not repeat effects.
- Invalid/stale/ineligible/key-missing commands do not consume RNG or mutate state.
- Existing advance-only Event behavior, including WhiteNight confession, remains unchanged.
- Current Core, live data, server, Unity and durable canonical all describe and consume one final
  schema with no legacy `required_roll` authority or compatibility fallback.

## DTO Migration

Required: yes.

- Replace scene-common check data with choice-specific typed checks/evaluations.
- Extend candidate projection with typed eligibility, modifiers, components, optional key evidence and
  exact success chance.
- Extend committed-check/result serialization with aptitude, DC, typed provenance, total and outcome.
- Preserve the existing resolve command's identity fields and server correlation/order unless direct
  investigation proves an approved contract conflict.
- Remove the old DTO shape in the same contract cutover; no dual-read/write or client repair.

The exact final field/message inventory and producer-consumer evidence are recorded chronologically in
`EXPERIMENTS.md` and structurally in `EXPERIMENT_NOTES.md`.

## Data And Live RON Migration

Required: yes, fail-closed.

- Author explicit Event aptitude for all 12 approved employee definitions.
- Add typed Event modifier/key metadata to selected equipment needed by the vertical slice.
- Replace/remove scene-level checked `required_roll` schema and require a check on every choice in an
  `employee_check_choices` scene.
- Add one new checked Event and an explicit reachable map node or Event-pool policy.
- Do not migrate or reinterpret the existing advance-only WhiteNight Event.
- Reject unknown aptitudes, unsupported DCs, duplicate/invalid equipment metadata, missing providers,
  multiple reachable checked scenes, checked-scene cycles and broken action/scene references before
  publication.

Baseline verification confirmed there was no checked live Event, so no legacy gameplay content was
migrated. WhiteNight was not reinterpreted.

## Save Migration

Required for current in-memory run/checkpoint state: yes.

- Persist `EmployeeProfile.event_aptitude` and the expanded committed result/key evidence.
- Preserve exact checkpoint, reconnect, retry and retreat semantics.
- Do not overwrite stored profile aptitude from current live definitions on restore.

External durable save/database migration: none. No such persistence consumer exists in the accessible
contract. Current in-memory run/checkpoint cloning now carries stored aptitude and expanded committed
provenance directly; no compatibility reader or serialized save migration was added.

## Compatibility Policy

No compatibility, fallback, alias, dual-read or dual-write is approved.

The current live bundle reportedly has no checked Event to preserve. If deployed payload/save evidence
contradicts this, implementation stops before migration until the user approves a bounded compatibility
policy with consumers, duration, removal conditions and dedicated tests.

## Required Focused Validation

The executed command/job inventory and actual counts are in `EXPERIMENTS.md`. Required behavior
evidence includes:

- Strict employee/Event/equipment schema and cross-catalog rejection matrix
- All 12 live employee aptitude assignments and profile-copy/restore authority
- Formula and exact probability table over all raw faces 1..20
- Natural 1/20 priority at every allowed DC/modifier boundary
- Highest-only equipment modifier and UUID tie-break
- Key provider selection, candidate/choice eligibility and typed evidence
- One shared projection/resolve evaluator and resolve-time stale revalidation
- Invalid request consumes no RNG/state; valid request consumes one draw
- Same raw roll after pre-check checkpoint restore with different valid choice/employee
- Exact duplicate/reconnect/checkpoint/retry/retreat committed-result reuse
- Success reward/combat action atomicity and no duplicate effect
- Checked-scene count/cycle validation
- Exact Core JSON and game_server transport/message ordering
- Actual Rust producer fixture consumed by Unity
- Unity reader/store/composer/command tests for selection, disabled reasons, odds and result display
- Focused Unity PlayMode render/interaction flow for the live vertical slice
- Registered live WebSocket probe through the actual server when the final flow is reachable

Zero-test runs, mocks agreeing only with mocks, compile-only results and hand-authored payloads without
actual producer equality are not completion evidence.

## Required Broad Validation

Final commands/jobs must be selected after discovery and recorded with actual counts. Minimum expected
surfaces:

- Rust formatting check
- Full `game_core` all-targets suite including live RON loading
- Full `game_server` check and tests
- Full Unity EditMode suite
- Ordinary Unity PlayMode suite, with explicit environment skips separated
- Unity console compiler/runtime error audit
- Actual producer fixture drift check and live WebSocket vertical-slice probe
- Scoped Core and external Unity diff/whitespace checks
- Legacy/current schema searches across runtime, live data, server, Unity fixtures/mocks/tests and
  current canonical
- Independent completion review connecting every visible condition/invariant to direct evidence

## Omitted Validation And Rationale

None is pre-approved for omission.

After discovery, an unrelated platform/deployment test may be omitted only if its lack of relevance,
baseline state and remaining risk are recorded. Core/server/Unity contract and the live vertical-slice
flow are mandatory for completion.

## Stop Conditions

Set `scope_decision_required`, `policy_decision_required` or `externally_blocked` and stop affected
mutation when any of the following occurs:

- The requested result requires multiple independently deployable source-of-truth boundaries rather
  than this one atomic Event contract.
- Production contains multiple employee checks per Event or checked-scene semantics incompatible with
  the approved single-check model.
- A durable external save/database or deployed legacy payload requires compatibility/migration not
  approved here.
- Choice/employee-dependent raw-roll seeding is required by current persistence guarantees.
- Performer-targeted condition, critical effects, `SelectedEmployee` reward context or another
  explicitly deferred gameplay policy is required to make the vertical slice valid.
- Safe work cannot be merged with overlapping dirty user changes in Core, live data, server or Unity.
- Actual Core producer, server mapping or required Unity consumer/test runner is inaccessible.
- A required baseline or completion test cannot execute and no evidence-equivalent safe path exists.
- First content requires unapproved aptitude, DC, modifier, multiple key or balance exception.
- Completion would require weakening current Event/rollback/atomicity behavior or adding silent
  fallback/compatibility.

## Completion Gate

Status becomes `complete` only when all user-visible conditions and invariants have direct evidence,
the entire producer-to-consumer schema is final, the live vertical slice is reachable, all required
focused/broad validations pass with actual counts, legacy schema/current wording is gone, durable
canonical matches implementation, and an independent completion review finds no material correction.
