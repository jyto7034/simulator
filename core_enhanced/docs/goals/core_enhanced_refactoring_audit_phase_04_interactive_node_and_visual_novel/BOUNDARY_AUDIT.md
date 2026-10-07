# Core Enhanced Refactoring Audit Phase 04 — Boundary Audit

Audit Date: 2026-08-07
Status: investigation complete; completion audit pending

## Exact Boundary

Phase 4 owns current Event, facility-family and top-level Visual Novel session lifecycle. It reads run/checkpoint,
economy, combat and projection seams only far enough to classify carrier/consumer behavior.

### Primary runtime

- `src/game/world/event_node.rs`
- `src/game/world/facility.rs`
- `src/game/world/map_content.rs`
- `src/game/world/headquarters.rs`
- `src/game/world/reward.rs`
- `src/game/world/shop.rs` at session commit/lifecycle seams only
- `src/game/world/visual_novel_flow.rs`
- `src/game/world/visual_novel_triggers.rs`
- `src/game/visual_novel.rs`
- `src/game/resources/selection.rs`

### Direct carriers and consumers

- `src/game/map/session.rs`
- `src/game/world/{state,node_flow,active_run,snapshot,action_capability}.rs`
- `src/game/world.rs` command routing and pre-run start flow
- `src/game/employee_psychology.rs` node-completion consumer
- `src/game/world/combat.rs` only at Event retreat/resume and VN trigger transport handoffs
- `src/game/behavior.rs` only for exact command/result identity

### Data and canonical

- `src/game/data/{event_data,visual_novel_data,visual_novel_trigger_data,validation,live_ron}.rs`
- published Event, VN, VN-trigger, map and run-policy RON directly consumed by this boundary
- `docs/{core_runtime_contract,data_loading_contract,game_rulebook}.md`

Combat preview/setup, active battle mechanics, DTO architecture and server/Unity refactor discovery remain later
Phases. Shop/reward/maintenance asset arithmetic remains Phase 3 authority.

## Authority Matrix

| Domain | Admission and current authority | Persistent/retry authority | Completion/consumer | Finding |
| --- | --- | --- | --- | --- |
| Node routing | `GameCoreState.node_session` with node id/kind/category/payload | run map/progression and checkpoint | common node completion | carrier, not content authority |
| Event | active `ActiveNodeContent::Event` | `RunState.event_sessions[node_id]` | result/grant/combat then common completion | intentional active mirror plus persistent re-entry state |
| Shop | active `ActiveNodeContent::Shop` | `RunState.shop_sessions[node_id]` | economy operation then common exit completion | intentional active mirror plus persisted page/history |
| Reward | active `ActiveNodeContent::Reward` | no run cache | clone-staged grant, claimed state, common exit | one node-local authority |
| SavePoint | active marker with node id | checkpoint payload created on completion | recovery, research delivery, checkpoint | typed marker plus common completion |
| Maintenance | active marker with node id | inventory/fragment state is Phase 3 | repeated operations until common completion | typed marker, not generic facility ledger |
| Headquarters | active contact plus run recruitment active session | recruitment queue/session in `RunState` | recruit/emergency/shop branch | split is intentional; transaction implementation is duplicated |
| VN | `GameCoreState.visual_novel.active/queue` | top-level state survives run checkpoint | exact scene/choice progression and source hook | sole session authority |
| VN trigger history | `GameCoreState.visual_novel_triggers` | persistent set plus checkpoint-rewind set in payload | trigger evaluator | Core-owned, not Unity-visible |

## Event Lifecycle Trace

1. Node confirm is wrapped by `with_state_transaction`; `enter_event_node` resolves exact live `event_id`, restores
   or creates the run-persisted session, mirrors it into active content and projects the authored entry scene.
2. `AdvanceEventScene` requires exact node/event/current-scene identity. A scene edge writes both mirrors; an End
   edge stages common node completion and removes the persisted session when the node commits.
3. `ResolveEventChoiceCheck` revalidates exact scene/choice/living employee and equipped-owned key/modifier state.
   One node-seeded d20 is independent of chosen payload. The committed state stores source scene, choice, employee,
   aptitude/DC, raw/component/total evidence and outcome.
4. Grant and battle-start effects are planned against clones before authoritative mutation. The exact committed
   duplicate returns the stored result without reroll/regrant; any differing replay rejects.
5. A started Event combat keeps the Event session in the run. Retreat restores the entry snapshot and re-entry
   resumes the already committed combat; successful node completion removes the session.

Runtime, canonical and focused tests agree on these semantics. No current Event correctness defect was confirmed.
The paired active+persistent write is repeated at entry, which is a structural candidate rather than evidence that
one of the two roles should be removed.

## Facility-Family Lifecycle Trace

- Node confirm itself is a full non-battle transaction. `try_enter_map_content_node` creates the typed active
  content after resolving live payload references.
- SavePoint recovery and planning-return research delivery are staged by common completion; invalid pending
  research rolls back node/progression/recovery.
- Maintenance keeps only node identity locally. Equipment/fragment validation and mutation stay in the Phase 3
  economy boundary; exiting uses common completion.
- Shop session page, offers, refresh uses and purchase history are persisted in the run and mirrored active. Shop
  operations clone/commit through the domain helper; exit uses common completion.
- Reward grant state is clone-staged. A failed multi-effect reward does not commit prior effects; a successful
  claim changes state before a separate exit completes the node.
- Headquarters recruitment queue/active session is run authority. Recruit, emergency supplies and shop branches
  stage domain state, but use `with_headquarters_transaction`, a manually enumerated rollback frame separate from
  the common transaction. Planning temporarily removes active HQ content to bypass the planner's HQ rejection.

The heterogeneous variants have different retry, persistence and action semantics. A generic facility session
would erase real ownership distinctions and is rejected.

## Visual Novel And Trigger Lifecycle Trace

1. The official loader strictly validates VN graphs, trigger local shape/references and exactly one policy-correct
   RunStart trigger for each supported mode.
2. Trigger evaluation matches the Core event, sorts priority descending then id ascending, derives a repeat key,
   constructs one session, activates or queues it, then marks the trigger consumed. `Always` has no consumed key.
3. An active session is never interrupted. Trigger sessions append to the queue; the Monolith exchange deliberately
   inserts ahead of an existing queue so the blocking exchange precedes the merchant Shop session.
4. Advance/choice commands require exact `session_id + current_scene_id`; choice commands also require an authored
   choice id. Scene changes mutate only the active session. End removes active and promotes queue index zero.
5. HQ briefing completion prepares starter candidates and moves to starter selection. Monolith accept/decline is
   clone-planned before VN progression and commits run state after the session transition.
6. Active/queued VN state is top-level and is not rewound by run checkpoint. Only the checkpoint-policy history set
   is saved/restored. This can legitimately queue a new session behind an active preserved session with a new UUID.

### Validation gap

`VisualNovelTriggerDatabase::validate_contract()` does not encode the runtime rejection table. In particular,
`MapNodeEntered + OncePerBattle` passes official bundle validation and is rejected only by
`consumed_key_for_trigger()` at node entry. The outer node transaction prevents partial node mutation, but the
invalid authored bundle has already been published and the node is unusable. This is a correctness correction,
not a new repeat policy: the target validator should reject only combinations the current runtime already rejects.

### Internal parallel state

`VisualNovelSession.committed_choice_id` has no `Some` writer in workspace production code. Constructors and scene
changes only set `None`, while snapshot selectability reads it. Removal or giving it replay meaning is not currently
safe: the struct is public/serialized and external Rust consumers plus retry policy are unknown.

## Atomicity And Replay Disposition

| Signal | Current evidence | Disposition |
| --- | --- | --- |
| Event exact duplicate | committed identity match returns same result, no grant/roll repeat | retained contract |
| Event stale/different replay | capability and handler reject before mutation | retained contract |
| Facility action replay | current game state and active typed session revalidate each action | retained; no universal idempotence promised |
| VN stale session/scene | exact identifiers reject after scene/session changes | retained contract |
| VN exact retry response replay | canonical does not promise cached duplicate response | policy migration if desired |
| Event/Shop active plus run copy | persistence and active projection roles differ | retain roles; consolidate paired writes |
| HQ rollback helper | duplicates common transaction field ownership | confirmed refactor candidate |
| VN completion late failure | possible only after invalid/unpublished state under current gates; no current live defect proven | retain and characterize candidate paths, no correction claim |

## Canonical And Live-Data Alignment

- Current live Event graph and Event validation match exact scene/choice/key/check/action contracts.
- Current live trigger catalog contains Standard and Endless RunStart triggers with required blocking,
  `OncePerRun` and persistent policy; referenced VNs exist.
- Top-level VN/checkpoint ownership, ordering, consumed timing and blocking transport match
  `core_runtime_contract.md`.
- SavePoint/Gate recovery does not manufacture psychology events; only node advancement consumes psychology state.
- Monolith exchange ordering and cloned accept/decline state match canonical, but no focused world integration test
  currently drives this lifecycle end to end.

## Consumer Classification

- Confirmed authoritative consumers: Core command admission, run checkpoint, battle entry rollback, common node
  completion and Core trigger evaluation.
- Confirmed derived consumers: Core behavior results, root/Event/VN snapshots and battle-update VN projection.
- Supporting server transport: consumes Core results/snapshots; no server-owned session policy was found or claimed.
- Unity and external Rust consumers: unverified/out-of-scope. Public serialized-type cleanup therefore remains
  deferred.

## Boundary Verdict

The current boundary has coherent ownership and strong replay/checkpoint tests. Four executable candidates remain:
one publication correctness correction and three behavior-preserving consolidation targets. Generic session
unification, cache deletion and retry-policy invention are rejected or deferred.
