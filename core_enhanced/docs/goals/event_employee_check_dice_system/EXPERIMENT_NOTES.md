# Event Employee Check Dice System Notes

Status: complete

Started: 2026-07-18

## Execution Input

- Goal name: `event_employee_check_dice_system`
- Approved description and policy:
  [event_employee_check_dice_policy_2026-07-17.md](../../decisions/event_employee_check_dice_policy_2026-07-17.md)
- User-visible problem: eligible employees currently have the same pure-d20 chance, so performer
  selection has little gameplay meaning.
- Target repositories: `/mnt/f/work/simulator`, `/mnt/f/unity projects/ark`.
- Finding ID: none.
- Approved non-scope and deferred policies are copied into `PLAN.md`.

## Structural Judgment

This remains one Goal despite crossing two repositories. Employee/Event/equipment authoring, Core
evaluation/persistence, server transport and Unity consumption are one contract boundary whose old and
new DTO cannot safely be completed independently. The Goal does not combine unrelated Event effects,
psychology systems or general UI refactoring.

Implementation is now divided into seven mandatory internal phases. Phase separation is an execution
and verification boundary, not a compatibility or deployment boundary. Phase 1–4 may stage one atomic
schema migration, but it is not a supported release candidate until the Unity consumer gate passes.

Each Phase must close with its own nonzero behavior/data/consumer validation as applicable. Final broad
validation supplements these gates; it cannot retroactively excuse skipping or carrying a failed Phase.

## Phase Status Registry

Last updated: 2026-07-19

This is the current execution-state registry. Update it only with matching chronological evidence in
`EXPERIMENTS.md`.

| Phase | Current state | Gate evidence |
| --- | --- | --- |
| Phase 0 — Owner Discovery And Dirty Baseline | gate_passed | 2026-07-19 owner map + Core 38 + RON 21 + server 30 + Unity 7/1 |
| Phase 1 — Schema, Strict Validation And Base Live Data | gate_passed | 2026-07-19: data 142 + profile/checkpoint 1/1/1 + RON 22 + fmt/check/diff |
| Phase 2 — Core Evaluation, RNG, Commit And Persistence | gate_passed | 2026-07-19: Event 55 + checkpoint 1 + Core lib 759 + server check |
| Phase 3 — DTO, Serialization, Server And Actual Fixture | gate_passed | 2026-07-19: Core 55/lib 759 + server 1/31 + deterministic actual fixture |
| Phase 4 — Unity Consumer And Player Presentation | gate_passed | 2026-07-19: Unity focused Edit 10 + Play 2; full Edit 244; full Play 58/0/4 |
| Phase 5 — Reachable Live Vertical Slice And End-To-End Flow | gate_passed | 2026-07-19: RON 23 + live Core 4/Event 56/lib 763 + server 31 + WS 30 assertions + Unity 10/3/244/59 |
| Phase 6 — Canonical Cutover, Broad Regression And Completion Review | gate_passed | 2026-07-19: Core all-targets 810 + server 31 + Unity Edit 244/Play 59/0/4 + WS 30 + fixture/canonical review |

Changing a Phase to `gate_passed` requires its exit-gate evidence and actual validation counts in
`EXPERIMENTS.md`. A later edit that invalidates prior evidence must change the affected Phase back to
`in_progress` or `gate_failed` until its gate is rerun.

## Source-Of-Truth State

- Current runtime/live data is authority until the migration is complete.
- The approved policy input and this Goal define the target and execution gate, not current gameplay.
- Target durable ownership must move into nearest Core, data/loading and external Unity canonical only
  after implementation matches it.
- Direct current-owner enumeration completed under Phase 0.

## Phase 0 Current Owner Map

| Boundary | Current owner and evidence | Approved target delta |
| --- | --- | --- |
| Event authoring/graph | `data/event_data.rs`: strict panels, scene-common `required_roll: 1..=20`, choices, success/failure refs and global acyclic/reachable graph validation | choice-owned aptitude/DC/key, DC allowlist, one checked scene, no return cycle |
| Employee profile | `employee.rs`/`employee_data.rs`: serialized `EmployeeProfile` copied by `Employee::from_definition`; no Event aptitude | add required stable `event_aptitude` to the stored profile and all 12 definitions |
| Equipment authoring | `equipment_data.rs`: strict `EquipmentMetadata`; no Event metadata | typed modifier/key arrays and validation |
| Eligibility | `EmployeeCondition::eligibility(EmployeeActivity::Event)`: physical 0..4 and trauma 0..99 enabled; critical/breakdown/dead typed-disabled | preserve condition gate, add key gate after it, no numeric condition modifier |
| Resolve/mutation | `world/event_node.rs`: exact identity validation, node-seeded deterministic pure d20, natural 1/20, planned grant/combat, one committed check and exact duplicate reuse | shared choice/employee evaluator, modifiers/key/chance/provenance and same atomic path |
| Persistence | `RunCheckpointPayload` clones `event_sessions`, inventory and roster; checkpoint load restores them; Event combat retreat/retry already preserves the session | stored profile aptitude and expanded committed result ride the same clone boundary |
| Core projection | `behavior.rs` + `event_scene_snapshot`: shared inner Event DTO with scene-common roll, global candidate list and compact committed result | choice-specific evaluations and expanded committed result |
| Server | `PlayerGameClientMessage` directly deserializes typed `PlayerBehavior`; generic `BehaviorResult` payload and following state snapshot; no Event repair/mapping | remain generic pass-through, add exact serialization/order tests |
| Actual fixture | `src/bin/generate_event_contract_fixture.rs` output exactly equals Unity `event_employee_check_contract.json` after trimming terminal newline | generator becomes the only new-shape fixture source |
| Unity reader/store | `EventSceneModel` strictly parses current typed Event JSON; `GameSnapshotStore` supplies root state | parse final evaluations/provenance fail-closed |
| Unity command/presentation | `GameCommandBus` sends identity-only resolve; `EventPresentationState` owns local cursor/selection; `EventScreenComposer` shows current d20/candidates | preserve command/cursor authority; show choice-specific odds/reasons/result |
| Live Event | `events/story/base.ron` contains only advance scenes for `white_night_confession_01`; map node points to it | leave WhiteNight unchanged; new checked Event comes in Phase 5 |
| Durable canonical | Core rulebook/runtime and external Unity contract describe the current pure-d20 shape | update only in Phase 6 after implementation |

## Phase 0 Consumer And Persistence Classification

- Workspace-confirmed producers/consumers: Core runtime/live loader/DTO, game_server generic transport,
  Unity fixture/strict reader/store/command/composer/EditMode/PlayMode.
- Canonical-only consumer: none required beyond the accessible workspace consumer.
- Known but inaccessible consumer: none identified in current repository/canonical evidence.
- Unknown deployed consumers: not evidenced; no compatibility is inferred for hypothetical clients.
- External durable save/database: none found. Workspace dependencies/source contain no SQL/SQLite/
  sled/Redis/run-state file persistence. Current save semantics are the non-serialized in-memory
  `RunCheckpointPayload` clone owned by `GameCore`.

## Phase 0 Current/Target And Legacy Inventory

- Current Core `required_roll` authority is present in nine source/test files; current Unity roll
  authority is present in five Assets files.
- Production Event/map RON contains zero checked scenes and zero `required_roll` entries.
- game_server source contains zero Event roll/check repair fields.
- Target modifier/aptitude/key/chance symbols are absent from current Core/live/Unity production and
  test surfaces, as expected before migration.
- The actual Rust Event fixture matches the checked-in Unity fixture byte-for-byte after ignoring the
  terminal newline.

## Phase 0 Dirty Overlap Decision

- Every major Core target is already modified relative to repository HEAD; `live_ron.rs`, employee
  definitions and the Event fixture generator are untracked current files.
- Unity's current Event reader/composer/tests/fixture are untracked; adjacent command/selection/mock/
  canonical files are modified.
- The large diffs are the already completed typed Event/employee/data architecture that this Goal
  extends. Current file contents and green baselines are authoritative; restoring or copying a clean
  historical version would destroy user work.
- Safe approach: patch only exact current structs/functions/data entries with `apply_patch`, inspect
  every scoped diff after mutation, and avoid unrelated formatting. No target file showed a conflicting
  concurrent Dice implementation, so Phase 1 can proceed cautiously.

## Dirty Worktree Warning

Both repositories are heavily dirty and contain extensive user-owned tracked and untracked work.

Likely overlapping areas visible from initialization status alone include:

- Core Event, employee, equipment, world/state/behavior/data-validation and tests
- live employee/equipment/Event RON and bundle loader
- game_server player actor messages/handlers/state
- Unity Event reader/composer/view-model/tests and current contract docs

These are observations from path names in `git status`, not evidence that the changes implement or
conflict with this Goal. Before touching any production file, inspect its exact current diff and record
whether the Goal can merge safely. Do not restore, stash, clean, overwrite or reformat unrelated work.

The approved policy source under `core_enhanced/docs/tmp/` is itself currently inside an untracked
directory. Preserve it as user-owned input.

## External Dependencies

- Accessible external Unity repository and live Unity Editor/test runner are expected completion
  dependencies because DTO and player-facing presentation change.
- A running game_server and registered WebSocket probe path are expected for the actual vertical-slice
  transport check.
- Existence of any external durable save/database or inaccessible deployed consumer is discovery
  pending.

## Policy Questions

None currently. The approved policy fixes aptitude assignments, modifier/DC values, one-roll identity,
key provenance, persistence meaning, UI authority and compatibility stance.

If direct investigation contradicts any of those assumptions, do not reinterpret the policy; record
the evidence and change Status according to the stop conditions.

## Alternatives Not Applied

- Did not split Core, server and Unity into separate Goals because that would create an unauthorized
  intermediate dual contract.
- Did not create a Master Goal or any follow-up Goal.
- Did not turn personality, trust, memory, level, condition magnitude or combat stats into roll
  modifiers.
- Did not add multiple aptitudes, ranks, advantage/reroll, multiple keys or critical effects.
- Did not select concrete implementation types/files beyond the approved target shape before direct
  investigation.
- Did not run repository-wide searches, tests or formatters before required Goal initialization.

## Out-Of-Scope Follow-Up Candidates

The approved policy lists possible future mechanics—multiple checks, advantage/reroll, assistance,
psychology/trust/memory effects, key consumption, hidden choice discovery and critical-specific
effects. They are not current tasks. Do not create Goals or implement them automatically.

## Omitted Or Unverified Evidence

- No consumer outside the two accessible repositories has been verified because none is identified as
  a required current consumer.
- Phase 0 did not run full Core/Unity broad suites; its required focused baselines ran, and later Phase
  gates require full affected/broad suites.

## Remaining Risk

- Extensive overlapping dirty changes may require careful merging or may trigger a stop condition.
- Current code/data may have advanced beyond the approved policy's baseline description.
- External persistence or a deployed legacy consumer could require an unapproved migration decision.
- The first live vertical slice must demonstrate meaningful employee choice rather than only a
  deterministic aptitude lookup.

## Resume Point

All seven Phase gates passed. Goal Status is complete; the Goal documents remain execution history and
the updated Core/server/Unity canonical now owns the durable contract.

## Completion Evidence Matrix

| Completion condition | Direct evidence |
| --- | --- |
| Meaningful choice/employee selection | live DTO/Core test pins Analysis DC16 scanner +2/key/35%, matching analyst +3 but missing key/40%, Maintenance DC12 technician +3/60% and nonmatching 45% |
| Core-only roll and exact result | 480-case matrix, chance table, shared evaluator call inventory, committed typed provenance and Unity no-recalculation tests |
| Invalid commands do not mutate | condition/key/stale identity tests and live missing-key pre-commit state/resource assertion |
| Duplicate/reconnect/checkpoint stability | live Core duplicate/checkpoint test, generic retreat/retry tests, actual WebSocket reconnect + duplicate/no-double-grant 30-assertion probe |
| Reachable production content | live RON 23/23, explicit weighted map definition and connected generated-map reachability test |
| Producer/server/Unity agreement | deterministic actual fixture diff 0, server 31/31, Unity focused 10/10 + 3/3 and broad 244/244 + 59/0/4 |
| WhiteNight preserved | live data keeps its three advance-only scenes; strict live RON and existing live Event flow test pass |
| Final schema/canonical cutover | current Core/data/server/Unity canonical updated; legacy production searches and scoped diff checks pass |

## Final Migration Classification

- DTO migration: yes, breaking atomic cutover from scene-common roll/global candidates to
  choice-specific evaluations and expanded committed provenance; no compatibility alias.
- Data/live RON migration: yes, 12 required aptitudes, equipment Event metadata, new checked Event and
  explicit map definition; no old checked live content required conversion.
- Save migration: no external serialized migration. Existing in-memory run/checkpoint clone shape now
  includes stored aptitude and expanded committed state and was verified across restore.
- Removed legacy: scene-level `required_roll`, global Event employee candidate DTO/Unity assumptions and
  any client/server roll repair path.

## Final Remaining Risk

- Four ordinary Unity PlayMode tests remain explicitly skipped because they require a separate port
  8081 live server and cover broader admin/maintenance/combat/endless flows. They are not Event Dice
  completion tests; the registered Event probe executed the required actual server flow.
- No inaccessible required current consumer or external durable save was identified. Hypothetical
  deployed consumers were not claimed as verified and no compatibility was inferred for them.
- The repositories remain broadly dirty with user-owned unrelated work. This Goal does not claim those
  unrelated changes are validated or complete.

## Phase 5 Structural Decisions

- The live checked Event is a new sibling of WhiteNight. `event_sealed_archive` is an explicit weighted
  Event definition, so the generator can place it on a connected path while WhiteNight remains
  advance-only and unchanged.
- Analysis DC16/key/80 and Maintenance DC12/no-key/30 provide different risk/reward, result scenes and
  employee incentives without introducing penalties or new effect types.
- The live Maintenance specialist owns the scanner and is therefore the only keyed Analysis candidate
  despite aptitude mismatch (+2, 35%). The Analysis specialist exposes the better theoretical match
  (+3, 40%) but typed missing-key ineligibility. The Maintenance choice reverses the incentive (+3,
  60%), proving selection is meaningful.
- Exact duplicate reuse is an execution exception, not a presented capability. Only the five committed
  identities bypass common preflight; the existing stored-result handler remains the single result
  authority. This preserves fail-closed behavior for every nonmatching replay.
- The registered Python probe implements only RFC 6455 transport framing with the standard library;
  it has no gameplay logic and validates server-authored fields/order exactly.
- Unity's live mock carries the production Event IDs/choices/policy and typed evaluations. It remains a
  presentation fixture, not an alternate gameplay authority.

## Phase 4 Structural Decisions

- Unity treats every gameplay-facing Event field as required and typed. It validates shape/range but
  never recomputes modifier totals, chance or outcome from components.
- Employee evaluations are owned by their selected choice. Changing choice clears the local performer
  cursor so an employee evaluation can never silently cross choice/DC/key context.
- The committed display is rendered only from persisted Core provenance: raw roll, typed components,
  modifier/total, DC, key evidence and outcome.
- Local illustration/text reveal remains presentation state and sends no resolve command. The command
  continues to carry only event/node/scene/choice/employee identities.
- The actual Core-generated fixture is the consumer contract input. AssetDatabase staleness was fixed
  by importing the asset; no parser fallback or fixture-specific production branch was added.
- Four full-PlayMode explicit live-server tests remain intentionally skipped in the ordinary suite;
  Phase 5 supplies the required running-server transport evidence instead of reclassifying skips as
  passes.

## Phase 3 Structural Decisions

- `employee_candidates` is nested under each choice; there is no scene-common candidate authority.
- Modifier components are the persisted/serialized typed sources. Aggregate employee/equipment values
  remain evaluation conveniences, while committed result carries only typed components plus totals.
- The actual fixture generator starts a real live-backed Core run, selects three authored starters,
  enters a fixture Event through the production admin path and resolves through the exact player
  command. It no longer constructs an Event DTO manually.
- The fixture is untracked pre-existing user-owned Unity work and was updated in place. Its pre-check
  root remains compatible with the fixture envelope, and `committed_selected_event` adds the actual
  committed payload for strict consumer verification.
- Server ordering remains generic: command result is emitted first, then a trailing state snapshot
  when `send_state_snapshot` is true. The new Event mapping produces no side message and requests that
  trailing snapshot; no Event-specific repair exists.

## Phase 2 Structural Decisions

- The evaluator fails closed on any disagreement between employee loadout refs, owned instance
  identity, base definition UUID and `equipped_to`; it does not infer or repair ownership.
- Applicable modifiers are sorted by amount descending then owned instance UUID ascending. Key
  providers are UUID ascending. Inventory-only items are never considered.
- Condition ineligibility is evaluated before missing key. A key-only item changes eligibility but
  contributes zero numeric modifier.
- Chance uses the same outcome function as commit by enumerating all raw faces 1..20; no parallel
  closed-form probability authority exists.
- Existing node-seeded d20 generation remains independent of choice and employee. Session commit is
  the one-shot boundary; the invalid-then-valid and pre-check restore test proves invalid identity does
  not perturb the resulting raw roll.
- The temporary outward `required_roll` DTO projection remains a Phase 1–3 staging artifact only.
  Phase 3 must remove it and expose per-choice `dc` and evaluations directly from this evaluator.

## Phase 1 Structural Decisions And Remaining Stage Boundary

- `EventAptitude` is owned by the employee domain and stored directly in `EmployeeProfile`; the
  definition is creation input only. Existing checkpoint cloning of the roster therefore persists it
  without a second authority or migration layer.
- `EventChoiceKeyId` is owned by Event authoring and reused by equipment metadata. Its validator is
  shared so Event requirements and equipment providers cannot drift on identifier syntax.
- Empty equipment modifier/key arrays use serde defaults because absence is the canonical neutral
  value for optional metadata, not an alias for a removed field or a compatibility schema.
- Cross-catalog validation rejects any required Event key absent from the published equipment
  catalog, while resolve-time ownership/equipped-provider checks remain Phase 2 behavior.
- The existing global graph validator already rejects every scene cycle, which is stronger than and
  directly protects the required checked-scene return-cycle rule. A dedicated regression pins that
  case.
- `archive_scanner` is the minimum Phase 5 seed: Analysis +2 and `sealed_archive_access`, authored on
  Maintenance-specialist `maintenance_tech`. This deliberately permits a later key-provider versus
  aptitude-match choice instead of making key possession an automatic best performer.
- Phase 1–4 are an explicitly non-release staging interval. Core's old outward Event DTO still
  projects a scene-common value from the first choice solely until the choice-specific DTO cutover;
  there is no live checked Event and no old RON schema. Phase 3 must remove that old DTO field rather
  than preserve it as compatibility.

## Phase 1 Failed Attempt

The first 142-test data gate exposed three stale employee unit fixtures missing required
`event_aptitude` (139 pass, 3 fail). They were migrated; the exact gate then passed 142/142. This was a
new-contract fixture failure, not a baseline defect and not hidden as success.
