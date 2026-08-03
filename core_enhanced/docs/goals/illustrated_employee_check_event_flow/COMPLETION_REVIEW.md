# Illustrated Employee-Check Event Flow Completion Review

Review date: 2026-07-14

Review procedure: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

Immediate correction: none

## Intended Outcome And Source Of Truth

G03 replaces the provisional flat Event/VN-like schema with one Core-owned persistent Event state
machine. Core and live data own identities, graph, panels, eligibility, one pure d20, committed
performer/result, success/failure mutation, battle composition, and completion. Unity owns only the
local panel/text cursor and uncommitted choice/employee selection. game_server transports the typed
envelope and exact command without Event-specific repair or gameplay policy.

## Item Audit Table

| Item | Current owner/result | Verdict | Fit | Correction |
| --- | --- | --- | --- | --- |
| Typed panels/interaction DTO | Core DTO and live schema | Complete | High | None |
| Employee eligibility and exact resolve | Shared Core evaluator/preflight | Complete | High | None |
| One persisted pure d20 | `EventSessionState.committed_check` | Complete | High | None |
| Atomic grant/failure/combat transition | Core Event planning plus Phase 2 battle entry | Complete | High | None |
| Reconnect/checkpoint/duplicate/retreat stability | persisted run Event session | Complete | High | None |
| Strict live bundle validation | live RON parser/domain/reference validation | Complete | High | None |
| Event/VN separation and old authority removal | typed Event route; no Event VN source | Complete | High | None |
| game_server transport | generic typed behavior/result/snapshot path | Complete | High | None |
| Unity route/reader/composer/commands | first-class Event feature slice | Complete | High | None |
| Canonical contracts | Core and Unity canonical documents | Complete | High | None |

## Runtime Evidence

- `src/game/world/event_node.rs` performs exact node/event/scene/choice/employee validation, calls
  the shared payload policy, rolls one deterministic `StdRng` d20 only before the first commit,
  classifies natural 1/20 and threshold outcomes, plans success mutations, persists
  `EventCommittedCheckState`, and returns the same result for an exact duplicate.
- `src/game/world/action_capability.rs` uses the same Event evaluator/payload validators for
  projection and execution preflight; Event commands no longer bypass the Phase 1 capability owner.
- `src/game/resources/selection.rs` stores only current identity, committed check, and optional
  started combat. The old completed-scenes, committed-choice, staged-reward, and Event reward-policy
  state is gone.
- `src/game/world/combat.rs` composes Event-started combat through the normal Phase 2 battle
  transaction, attempt, result, retry, and `RetreatBattle` path. No Event-specific reward timing or
  rollback pipeline remains.
- `src/game/data/visual_novel_trigger_data.rs` and
  `src/game/world/visual_novel_triggers.rs` have no `EventSceneEntered`; Event scene mutation does
  not start or route through global VN.
- Admin `AdminEnterEvent` is an explicitly gated debug fixture over the same live Event projection;
  it adds no player command alias or production gameplay authority.

## Data Evidence

- `src/game/data/event_data.rs` defines strict ordered panels, tagged advance versus
  employee-check choices, bounded required roll, one optional success action, success/failure scene
  identities, and graph validation.
- `src/game/data/live_ron.rs` and `src/game/data/validation.rs` validate Event grants through the
  normal reward contract and validate combat encounter/primary-abnormality references before
  publication. Invalid/unknown/removed fields fail closed.
- Production `game_resources/data/events/story/base.ron` is migrated to typed ordered panels and
  advance-only scenes. Its former choices had no gameplay effect, so no reward, combat, or balance
  meaning was invented.
- The full employee-check contract is exercised by strict authored fixtures and the Rust-generated
  Unity fixture, as authorized; live White Night content remains a legitimate advance-only Event.

## External Contract Evidence

- `src/game/behavior.rs` exposes one shared Event inner DTO and exact
  `ResolveEventChoiceCheck(node_id,event_id,current_scene_id,choice_id,employee_uuid)`. The behavior
  enum rejects unknown fields and the removed select command.
- `src/bin/generate_event_contract_fixture.rs` serializes the canonical employee-check snapshot
  consumed by `Assets/Tests/Fixtures/event_employee_check_contract.json`.
- game_server uses its existing generic `PlayerBehavior` deserialization, `BehaviorResult` command
  result, and state-snapshot ordering. No Event JSON mapping or repair layer was added.
- Unity `EventSceneModel` requires exact string identities, complete string panel arrays, boolean
  eligibility objects, bounded integer rolls, supported interaction/outcome values, and no Raw
  Event fallback.
- Unity `EventScreenComposer`, `GameUiSelectionState`, router, command bus, button/body handlers,
  and UGUI adapter implement local reveal/selection plus Core-gated typed progress. The image id is
  preserved as an illustration resource hook; producing or inventing missing art assets is content
  work outside this contract-hardening goal.

## Test And Validation Evidence

- Core focused Event/capability/serialization/admin checks passed, including 7 map-flow Event tests,
  pure d20 boundaries, exact command serialization, and the live admin projection.
- Core all-targets: 728 passed, 0 failed, 0 ignored. Zero-test binary targets were accounted for.
- Core `cargo check --all-targets` and final `cargo fmt --all -- --check`: passed.
- game_server all-targets: 28 passed, 0 failed, 0 ignored. Zero-test binary/example targets were
  accounted for.
- Unity focused Event EditMode: 7/7; Event UGUI PlayMode: 1/1.
- Unity full EditMode: 201/201; ordinary PlayMode: 52/52.
- Actual latest Rust server to Unity Event route/advance round trip: 1/1. The server loaded the live
  bundle, entered `white_night_confession_01`, accepted the exact advance command, and shut down
  gracefully.
- Checkpoint tests preserve committed choice/performer/roll/outcome and started combat; map/combat
  tests prove exact duplicate reuse, stale rejection without mutation, one grant, failure completion,
  battle start, retreat, retry, victory, and defeat behavior.

## Legacy And Fallback Audit

- Production Core/live/Unity runtime contains no `SelectEventChoice`, `EventSceneEntered`, flat
  Event presentation, immediate/staged Event effects, Event combat reward commit policy, Event Raw
  reader, VN fallback, modifier/advantage/reroll field, or compatibility alias.
- Lowercase `select_event_choice` and flat `background_id/script_ids` occur only in explicit
  rejection tests.
- Relevant Event tests have no ignore marker.
- Canonical Core runtime/rulebook/loading and Unity core/screen contracts describe the current
  panels/interaction/committed-check shape and Event/VN separation.

## Completion-Review Correction

The review found one Unity reader defect: non-string scalars could be converted with `ToString()` and
malformed `text_ids` entries could be filtered. This was an exact-contract violation, not a new
policy question. The reader now fails closed on non-string identities, mixed text arrays, missing or
non-boolean eligibility, and non-integer roll fields. A dedicated negative test was added, focused
7/7 and broad 201/201 EditMode passed, and the live Rust-server flow passed again on the corrected
reader.

## Cross-Component Findings

- Core root snapshot, `EventState` result, Rust fixture, server transport, Unity reader, composer,
  and canonical JSON now agree on one inner DTO and exact identities.
- Event success combat composes the fixed Phase 2 transaction; it does not redefine attempt,
  reward, retreat, checkpoint, or recovery semantics.
- Event commands compose the fixed Phase 1 capability authority; projection and execution do not
  define competing eligibility.
- Event and global VN are separate runtime/UI sources. G19 can now remove remaining global VN
  authoring duplication without preserving an Event dependency.

## Long-Term Direction Review

- Fit: High.
- Improvement class: none.
- Reason: the change removes multiple provisional schema/effect/reward/VN owners and replaces them
  with one persistent typed state machine, one strict authoring graph, one capability evaluator, one
  exact command, one normal battle composition point, and one typed Unity route. Future Event
  content can add panels, checked choices, grants, or normal combat without another schema or
  authority migration.

## Remaining Risk And Follow-Up Classification

- No immediate correction remains inside G03.
- Final art/localization resources for semantic `image_id`/`text_ids` are content production, not a
  hidden gameplay source or compatibility gap. G03 deliberately does not invent those assets.
- G19 remains the required next Child for global VN authoring authority cleanup; it must not reopen
  the Event contract.

## Review Conclusion

G03 is Complete / High with no immediate correction. G19 may begin against the fixed invariant that
Event has no VN dependency, Raw fallback, duplicate schema, or Event-specific battle authority.
