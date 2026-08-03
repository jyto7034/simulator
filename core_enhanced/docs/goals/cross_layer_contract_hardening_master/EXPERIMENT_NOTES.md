# Cross-Layer Contract Hardening Master Experiment Notes

Status: active planning notes

Implementation complete: false

Created: 2026-07-11

This file records orchestration judgments, unresolved policy questions, cross-goal conflicts, and out-of-scope follow-up candidates. It is not a gameplay, data, transport, or Unity source of truth.

## Master-Level Decisions

### The master does not implement

The master owns the registry, dependency graph, policy queue, progress state, and completion review gates. It does not make production changes.

Reason:

- findings cross Core, live data, game_server, Unity, tests, and documents,
- their policy and validation gates differ,
- one implementation goal would create false completion and overlapping ownership.

### Child documents are created lazily

Do not create all child directories at master initialization.

Create a child PLAN.md, EXPERIMENTS.md, and EXPERIMENT_NOTES.md only when that child starts. This avoids dozens of stale empty goals and makes the active work boundary visible.

### Finding identifiers require source prefixes

DCA and CPR identifiers collide. All goal documents must use source-prefixed IDs.

Bare H-01, M-01, and similar identifiers are prohibited in progress tables.

### Progress state belongs in PLAN.md

PLAN.md is the only master progress source.

- EXPERIMENTS.md records chronological evidence.
- EXPERIMENT_NOTES.md records judgments and questions.
- Child status must not be inferred from prose in these notes.

### Policy delivery is not implementation completion

A child that reaches a required user decision uses:

~~~text
Status: policy_decision_required
Implementation complete: false
~~~

A policy-decision packet is a valid stopping deliverable, but it is not a completed implementation. After the user decides, resume the same child or explicitly retire it with evidence.

### Compatibility is not the default

Do not preserve stale DTOs, aliases, fallback ranges, duplicate capability lists, or deprecated examples merely to avoid coordinated changes.

If compatibility is unavoidable, the child must record:

- concrete external consumer,
- version or deployment window,
- removal condition,
- removal owner,
- user approval.

## Known Policy Questions

### Q01. Battle side-band ordering

Question:

Should battle state changes that do not produce simulation events use a new monotonic update revision?

Current behavior:

- event sequence orders battle log events,
- battle time orders simulation progress,
- pause, speed, and Visual Novel state can change without changing either value,
- Unity can discard the resulting update.

Recommended direction:

Add a separate monotonic update revision. Do not fabricate event sequence or battle time.

Decision recorded 2026-07-11:

Accepted. G01 will add an independent monotonic update revision and keep event sequence and battle time semantically separate.

2026-07-12 code-review refinement accepted by the user:

- Name the wire field `battle_state_revision` and scope it by `battle_uuid`; it orders authoritative battle-state commits, not packets or serialization attempts.
- Start the initial committed battle projection at revision 1. Increment exactly once after each successful transaction that changes any client-authoritative battle projection, including simulation/checkpoint state, playback, deployment, skill readiness, battle Visual Novel state, and exact battle capabilities.
- Finalize nested triggers, Visual Novel projection, and capability changes before stamping and building the update so one revision never combines state from two commit points.
- Failed commands, rejected/no-op mutations, `RequestBattleState`, ordinary serialization, duplicate resend, and resync do not increment revision. Checked overflow is an error; never saturate and reuse a revision.
- Every top-level `battle_update`, nested `battle_resync.update`, and battle command acknowledgement carries the same committed revision. A resync may authoritatively reapply the same current revision; an ordinary update may not.
- Unity orders ordinary state updates by `(battle_uuid, battle_state_revision)`. Event seq validates event-delta continuity only. Battle time drives simulation/presentation only and never decides staleness.
- A higher revision with the same event seq and battle time is valid and must apply. An equal/lower ordinary revision is duplicate/stale and ignored. A newer revision with an event-delta gap requests resync rather than guessing.
- The revision covers the complete battle projection. G06/G07 must add Core-authoritative exact battle capabilities to the revisioned update instead of retaining Unity's static battle-action whitelist.
- Revision does not replace ordered transport. Command-result side updates and actor tick pushes must use one ordered per-session battle outbox, or equivalent sequencing, so a later event delta cannot overtake the delta it depends on.
- Move `last_pushed_event_log_seq` out of `ActiveBattleSession` and the root gameplay snapshot. Core owns the immutable event log and current battle revision; the server connection owns its last-sent/applied event cursor. Core update/resync projection must be read-only with an explicit `after_seq` input.
- Battle reset creates a new `battle_uuid` and revision sequence. Reconnect/resync preserves the active battle's current revision; missing setup recovery must carry the immutable setup plus the current revisioned state rather than relying on time.

User confirmation on 2026-07-12:

- promote this refinement into the Master PLAN as the binding PD-01 contract,
- G01 must implement the detailed commit, transport, resync, capability, cursor-ownership, and cross-repository test requirements rather than only adding a numeric DTO field.

Affected children:

- G01
- G07
- G08

### Q02. Capability semantics

Question:

Does allowed_actions mean exact commands executable for the current payload and revision, or only coarse state affordances?

Current behavior:

- Core includes actions that handlers can still reject,
- Unity replaces active-battle actions with a static whitelist,
- battle transport has no dynamic replacement capability.

Recommended direction:

Core should produce one context-derived executable capability projection. If coarse affordances are desired, give them a different name.

2026-07-12 code-review refinement accepted by the user and promoted to binding PD-02:

- The current `allowed_actions: Vec<ActionKind>` cannot be made payload-exact because it has no item, employee, tile, skill, target, reward, choice, or offer identity. Replace it rather than redefining the same shape.
- Use one canonical two-level Core contract:
  1. context capability says which command families are relevant now and whether at least one executable payload exists,
  2. payload eligibility on domain DTOs/previews says whether a concrete offer, inventory item, employee, target, node, reward, or choice is executable and why.
- Replace `allowed_actions` with typed `action_capabilities`; do not retain both as dual authorities or a compatibility alias.
- A context capability contains typed action identity, scope (`gameplay`, `query`, or `recovery`), enabled state, and a stable typed disabled-reason code. Irrelevant actions are omitted; context-relevant but blocked actions remain disabled so Unity can explain them.
- Parameterized actions use domain projections rather than a generic untyped payload bag. Examples include Shop offer purchase eligibility, sale candidates/quotes, map-node entry eligibility, reward choices, inventory operations, battle deployment/withdrawal candidates, skill readiness/target eligibility, and VN scene/choice commands.
- For large or dynamic payload spaces such as employee+tile deployment, the existing preview/preflight command returns authoritative eligibility and reason for the selected payload. Final commit revalidates it.
- Projection and execution call the same pure preflight/validation functions. Do not duplicate eligibility rules in ActionScheduler, DTO construction, handlers, and Unity.
- Capability is a snapshot, not authorization. Every command revalidates atomically at execution. Battle mutation commands carry the expected `battle_state_revision`; stale revision returns a typed stale-revision result plus current authority/resync path.
- Battle capabilities are included in the revisioned battle update and committed under the same `battle_state_revision`. Non-battle capabilities are included in the authoritative root snapshot and relevant domain DTOs.
- Remove `ActionValidator`'s cached HashSet and the state-only `ActionScheduler` table as execution authorities. If a presentation cache remains, key it by authoritative revision and derive it from the canonical projector.
- Remove Unity's `battleRuntimeActions` whitelist and local action inference. Unity renders Core capabilities and domain eligibility, localizes typed reason codes, and sends a selected payload only when its authoritative eligibility is enabled.
- Read-only transport queries and recovery-only commands must not masquerade as ordinary player-facing gameplay actions. Scope controls presentation and dispatch, while Core remains the authority for all three.
- Blocking VN exposes only the command valid for the current scene (`AdvanceVisualNovel` or concrete choices), not both kinds. Playback exposes Pause or Resume according to current state, and selecting the already-active speed is disabled/no-op without consuming a battle revision.
- Keep G06 bounded: it owns the canonical capability DTO, typed reason vocabulary, shared projector/preflight seam, removal of `allowed_actions`/`ActionValidator`, and root consumer migration. Domain children own their payload eligibility: G07 battle, G03 Event, G21 Safe Zone research delivery, G24 inventory operations, and G28 Shop offers/sales/refresh. Do not turn G06 into one repository-wide behavior rewrite.

Affected children:

- G06
- G07
- G08
- G03
- G21
- G24
- G28

### Q03. Event node support

Question:

Is Event a supported current Unity feature or future-only content?

Current evidence:

- live map generation guarantees an Event node,
- live Event data exists,
- Core commands exist,
- ArkClient has no route or command flow.

Recommended direction:

Treat Event as supported live functionality and align Unity to the typed Core Event DTO.

Historical interim refinement accepted earlier on 2026-07-12, then superseded by the user's final Event interaction clarification recorded below:

- Treat Event as a first-class, persistent Core-owned state machine, not merely a new Unity screen. Core remains authoritative for `event_id`, node/session identity, current scene, committed choice, staged effects, Event-started combat, completion, and recovery.
- Unity must add a dedicated Event route, typed Event reader/model, screen composition, and typed `AdvanceEventScene`/`SelectEventChoice` command flow. Do not parse `SelectedEventModel.Raw`, route Event through Visual Novel UI, or fall back to `Unknown`.
- Event-node gameplay and the global Visual Novel overlay remain separate. If `EventSceneEntered` starts a blocking Visual Novel, completion returns to the unchanged Event scene and its current capability/eligibility.
- Apply PD-02 at the Event boundary: expose scene-level advance capability and choice-level typed eligibility/reason from the same Core preflight validators used by command execution. Replace the current bare `selectable` boolean and coarse InNode action pair.
- Add `current_scene_id` to choice commands, or an equivalent authoritative Event-session revision. A stale choice must return a typed stale-state rejection rather than being matched only by event/node/choice identity.
- Reconnect, checkpoint load, Event-to-Visual-Novel return, and Event-to-combat retry/result transitions restore the persisted Event session rather than restarting or locally reconstructing it.
- Remove `ApplyBossOmenStepResult` from Event authoring unless a concrete live policy requires it. It is absent from live RON and currently panics at execution; do not preserve a runtime-panic schema variant as nominally supported functionality.
- Live-bundle validation must reject missing Event references, invalid encounter references, unsupported effects, invalid scene graphs, and missing required presentation/localization resources before gameplay. Runtime authored-data panic is not an acceptable Event route outcome.
- Do not make “at least one Event every Floor” part of the UI-support decision. Generation frequency and repeat policy are separate content/progression policy. The current live bundle contains one Event, so the current guarantee repeats the same content across Floors.
- Event-combat reward timing is resolved as at-most-once per selected choice with a persisted commit marker. `OnVictory` qualifies the first committed victory; `OnAnyResolution` qualifies the first committed victory or defeat, including a retryable defeat; `Never` does not commit. `RetreatBattle` does not qualify and restores the pre-entry marker.
- Required end-to-end evidence must use actual Rust snapshots and cover scene advance, mutually exclusive choice/advance capability, choice rejection reasons, Visual Novel overlay return, reconnect/checkpoint restore, Event-started combat victory, retryable defeat/retreat, exhausted resolution, once-only reward commit, and node completion.

Affected children:

- G02
- G03
- G06
- G07
- G12
- G14
- G15

### PD-04. Event DTO shape

Question:

Should Unity align to the current Core Event DTO exactly, or should the typed Core DTO be cleaned up once before it becomes the durable cross-repository contract?

Current evidence:

- The old Unity document requires singular `script_id`, Event-local speaker/portrait fields, and `selectable`; those fields do not match runtime Core and conflate Event with global Visual Novel presentation.
- Core currently projects `EventSceneSnapshotDto` with flat `background_id`, ordered `script_ids`, flat `choices`, and authoring-domain `EventChoicePreview` directly embedded in the wire DTO.
- Snapshot `selected_event.event` and command-result `EventState.event` already reuse the same typed `EventSceneSnapshotDto`; game_server forwards it without Event-specific reinterpretation.
- Unity has no typed Event reader and retains a generic raw selected-event object. No live Event presentation resource IDs were found in Unity Assets.

Historical interim DTO refinement accepted earlier on 2026-07-12, then superseded by the user's final illustrated employee-check DTO clarification recorded below:

- Keep the Core-authoritative typed direction, but do not freeze the current exact shape merely because it exists.
- Remove the stale singular `script_id`, speaker, portrait, `selectable`, and other Event/VN compatibility fields without aliases.
- Rename Event-local ordered text resources from `script_ids` to `page_ids` in live RON and wire DTO together. These are ordered Event presentation pages, not global Visual Novel scripts. Do not accept both names.
- Nest local presentation references under `presentation: { background_id, page_ids }` so gameplay identity/interaction and Unity-local resources have separate typed boundaries.
- Replace flat `choices: []` inference with a tagged interaction:
  - `interaction: { type: "advance" }` for a scene progressed or completed by `AdvanceEventScene`,
  - `interaction: { type: "choices", choices: [...] }` for concrete choices.
- Do not expose the next scene ID or whether advance completes the Event. Core remains authoritative for the result. Root PD-02 `action_capabilities` owns advance enabled/reason state; do not duplicate it inside the interaction DTO.
- Each concrete choice owns PD-02 payload eligibility with typed `enabled` and optional stable disabled-reason code. Remove the bare `selectable` field.
- Define a dedicated public `EventChoicePreviewDto`; never embed the RON authoring struct directly in the wire contract.
- Remove authored preview `starts_combat`. Core derives the projected `starts_combat` value from the validated `StartCombat` effect, eliminating duplicate authoring authority.
- Replace arbitrary `risk: Option<String>` with optional `risk_hint_id`. It is presentation metadata, not the existing abnormality `RiskLevel` gameplay enum. `reward_hint_id` and `penalty_hint_id` remain optional authored disclosure IDs.
- `SelectEventChoice` includes `current_scene_id`; the command authority tuple is `node_id + event_id + current_scene_id + choice_id`. A new Event revision field is unnecessary unless later same-scene authoritative mutations cannot be ordered by this tuple and PD-02 state authority.
- Unity keeps a page cursor only as presentation-local state keyed by `node_id + event_id + current_scene_id`. Unrelated snapshots for the same scene preserve it; a new scene or a fresh client/reconnect starts that current scene from its first page. The page cursor is not saved or used to authorize gameplay.
- Unity uses a dedicated typed Event reader, route, composer, and command builder. Direct parsing of a generic `Raw` object and raw-ID display fallback are forbidden.
- Command-result `EventState.event` and snapshot `selected_event.event` use the same canonical inner DTO. Command results may improve immediate responsiveness, but the following authoritative state snapshot decides the final route/state.
- Build/release validation compares every live Event background/page/choice-label/risk/reward/penalty ID against the Unity presentation catalog manifest. Missing resources fail the contract before play.

Required evidence after acceptance:

- exact Rust serialization tests for snapshot and `EventState` command result,
- actual Rust JSON fixtures consumed by the Unity typed reader,
- tagged advance/choices exclusivity and ordered-page tests,
- projection/handler eligibility-reason parity and typed stale-scene rejection,
- proof that `starts_combat` is derived from actual effects,
- cross-repository resource-manifest success and missing-ID failure tests,
- searches proving singular/dual field aliases, Event speaker/portrait fields, `selectable`, and generic Raw Event parsing are absent.

Affected children:

- G02
- G03
- G06
- G12

### 2026-07-12 final PD-03/PD-04 Event intent clarification

The user supplied Limbus Company-style reference images and explicitly fixed the intended flow. This decision supersedes the interim Event-to-Visual-Novel recovery and generic page/preview designs above.

Accepted player flow:

1. Enter Event and show Event-specific images with text.
2. Click during text animation to reveal the current text; click after reveal to advance locally through the image/text presentation.
3. After all pre-choice text is shown, display choices.
4. Select a choice, then select an employee from the bottom roster portraits. These portraits select the performer and are not Event-scene characters or Visual Novel speakers.
5. Employee selection reveals/enables the progress button.
6. Progress sends one Core command. Core rolls one d20 exactly once.
7. Natural 1 is critical failure; natural 20 is critical success; for 2..19, a raw roll equal to or above the authored required roll succeeds.
8. Authored required roll is restricted to the d20 domain, 1 through 20.
9. The initial version has no employee, equipment, effect, trust, trauma, injury, advantage/disadvantage, bonus-die, or reroll modifier. Do not create dormant fields for them.
10. Success performs the choice's authored action, such as an item grant or existing battle start.
11. Failure shows authored failure image/text. After the final failure text is revealed, the next advance completes and consumes the Event node.
12. One check has one attempt. Employee swapping after resolution, alternate-choice retry, reroll, and reconnect reroll are forbidden.

Accepted authority details:

- Choice and employee selection remain Unity-local until progress is pressed.
- The resolve command commits `choice_id`, `employee_uuid`, raw d20, required roll, outcome, and resulting action/failure scene atomically.
- The chosen employee is initially a performer identity only. Selection does not automatically target that employee with reward, injury, trauma, or another personal effect.
- Critical success uses the normal success action and critical failure uses the normal failure sequence initially; separate critical rewards/penalties are deferred until playtest evidence justifies them.
- Event is not Visual Novel. Remove `EventSceneEntered` VN triggering and all Event speaker/portrait/camera/dialogue-stage contracts.
- Event-started combat uses the shared battle system. The Event check remains committed, and battle abandonment restores the battle pre-entry state after that check rather than reopening the roll.

Accepted DTO correction:

- Presentation is ordered `panels[]`, each with explicit `image_id` and ordered nonempty `text_ids[]`.
- Interaction is tagged `advance` or `employee_check_choices`.
- A checked choice exposes `choice_id`, `label_id`, `required_roll`, typed choice eligibility, and typed employee candidates. It has no risk/reward/penalty/combat preview or modifier fields initially.
- `ResolveEventChoiceCheck` carries `node_id`, `event_id`, `current_scene_id`, `choice_id`, and `employee_uuid`; it replaces the resolving `SelectEventChoice` command without a compatibility path.
- A persisted `committed_check` exposes performer, raw roll, required roll, and `failure | success | critical_failure | critical_success` for reconnect-safe result presentation.
- Live image/text/choice-label IDs must resolve through the Unity presentation manifest before release.

Deferred incremental-development candidates, not initial schema:

- employee check attributes or base modifiers,
- equipment/effect modifiers,
- advantage/disadvantage,
- bonus dice and rerolls,
- distinct critical branches,
- employee-targeted outcome effects,
- probability/forecast presentation beyond the authored required roll.

### Q04. Abnormality basic-attack ranges

Question:

What authored tile pattern should each of the 30 live abnormalities use?

Current behavior:

Every live abnormality uses no authored pattern and the Melee fallback, including projectile attacks with large legacy range_units values.

Recommended eventual direction:

Remove the special-unit fallback after every finalized live entry has an approved explicit range policy. Do not synthesize values from old range_units without balance approval.

User deferral recorded 2026-07-11:

- Detailed abnormality combat settings are not yet finalized, so assigning ranges now would create accidental balance policy.
- G05 is deferred, not complete and not not-applicable.
- The current Melee fallback is temporarily tolerated only for the 30 incomplete placeholder entries. It is not a canonical content contract and must not be expanded as a convenience path for finalized content.
- Do not edit live abnormality RON, add fake explicit Melee values, or harden tests around the placeholder output.
- Removal trigger: once per-abnormality basic-attack roles and detailed combat settings are approved, create G05, review a proposed assignment table, author explicit reusable/custom patterns, replace placeholder tests, and remove the fallback.

Affected child:

- G05

### Q05. Global target reach, target-centered TileArea, and future field wave

Status: accepted by user on 2026-07-12 as PD-06.

Binding decision:

- The stationary final boss can acquire a valid unit anywhere on the battlefield through Global cast reach.
- Global reach is target acquisition only. It never means whole-field damage.
- Selection captures the unit's tile. A required TileArea footprint is projected from that tile, and the attack still resolves there if the selected unit dies or withdraws before impact.
- Cast targeting owns target kind/rule/reach/air eligibility; TileArea owns its footprint; a reused-target step does not duplicate either contract.
- Remove `WholeFieldValidTiles + TileArea` and the overloaded `defense_tile_range` interpretation without compatibility schema or fallback.
- Current deployed units do not move, so do not invent tracking/homing variants. Symmetric areas do not require caster facing.
- A true whole-field expanding wave is a separate future delivery: it begins at the caster tile, expands circularly by battlefield-space distance, crosses void without blockage, activates valid tiles on contact, and hits a unit at most once per wave.
- Wave runtime is not part of G09 now. Do not add a loadable but unexecutable RON/enum placeholder; preserve only an architectural extension seam and follow-up note.

Affected children:

- G09: reach/footprint ownership, validation, geometry, runtime, DTO migration
- G10: explicit target selection authority and area-aware rule only when authored
- G18: canonical examples and stale mixed-range documentation

### Q06. Skill target authority

Question:

Should skills use explicit UnitTargetRule or inherit weapon targeting_profile when not specified?

Current behavior:

Live skills explicitly author UnitTargetRule. No inheritance schema exists. Documents promise inheritance.

User decision on 2026-07-12:

- basic attacks use the equipped weapon `targeting_profile`,
- every skill explicitly authors its own cast-target selection rule,
- reusable targeting presets are allowed only through an explicit skill reference,
- omission never inherits from the weapon and fails strict live loading,
- changing weapons does not silently alter skill targeting,
- two skills may intentionally choose the same preset; independent authority does not require different values.

Affected child:

- G10

### Q07. Enemy range previews

Question:

Should every enemy range preview travel in the normal checkpoint, and should Unity display it outside explicit telegraphs?

Current behavior:

Core sends previews for every active unit. Unity parses them but normally renders selected employee previews.

Affected children:

- G11
- G13

User decision on 2026-07-12:

- there is no enemy range-preview feature,
- remove enemy range-preview data from normal and checkpoint projections, Unity readers/rendering, canonical documents, mocks, and tests,
- do not retain an empty/reserved compatibility field,
- actual enemy actions and separately authored attack telegraphs are distinct from a persistent/selectable range preview and are not removed by this decision.
- this is the current product policy, not a permanent ban; a later preview feature changes the DTO intentionally instead of preserving an empty field now.

### Q08. RON strictness

Question:

Should gameplay RON reject unknown fields and historical aliases consistently across every domain?

Current behavior:

Strictness differs by schema. Several aliases remain accepted. Some pool lookups use first-match semantics.

User decision on 2026-07-12:

- a dedicated authoring program, not manual editing, produces live gameplay RON,
- enforce an exact schema version and maximally strict canonical parsing,
- reject unknown, removed, aliased, missing required, duplicate, unsupported, out-of-range, internally inconsistent, and cross-reference-invalid data,
- never keep first-match duplicate behavior or partially load a broken bundle,
- validate before export and validate independently again in Core, preferably from shared schema/validator authority,
- report file/domain/object/field-path context,
- save-data migration is separate and does not justify lenient gameplay RON.

Affected children:

- G12
- G19
- G22

### Q09. BossOmen Gate and rollback

Question:

Does a forced BossOmen replace the floor terminal Gate, and can the player load a checkpoint after forced-boss failure?

Current behavior:

Forced boss victory advances the floor immediately. Forced boss defeat enters RunFailed with no recovery action.

Victory policy accepted 2026-07-11:

- Completing the forced BossOmen boss commits the boss CombatResult, authored rewards/research, accumulated employee penalties, consumable duration, and chain completion.
- It does not advance the Floor directly.
- It opens a newly created dedicated exit Gate connected after the completed boss. That Gate is the only map progression route to the next Floor.
- The player returns to the map, enters and confirms the Gate, receives the standard Gate trauma recovery, and only then advances the Floor.
- The old direct `FloorAdvanced` branch and its tests are policy-regression legacy to replace, not preserve as a fallback.

Defeat policy accepted 2026-07-11:

- The forced boss is the final boss of its BossOmen chain, not a final boss for the Endless run as a whole.
- A non-abandonment defeat commits the CombatResult, accumulated employee penalties, and consumable usage, then enters RunFailed.
- If a SavePoint checkpoint exists and its load budget remains, RunFailed exposes an explicit checkpoint-load action as a convenience recovery choice.
- Accepting recovery consumes one load and restores the checkpoint's authoritative state. Declining recovery, or having no remaining load, leaves the run ended.
- Core must expose the recovery capability and command independently of ordinary player action gating; G08 must implement the Unity-facing action and UX.

Affected children:

- G14
- G08

### Q10. Failure, retry, battle-stage abandonment, and casualty timing

Questions and resolution:

- Resolved: after a non-abandonment result commits, fatal roster failure (`NoLivingEmployees`) preempts retry and cancel.
- Resolved: numeric employee-condition changes incurred inside the attempt do not survive battle-stage abandonment (`RetreatBattle`); they return to the battle-entry snapshot.
- Resolved: there is no exhausted-abandonment result. `RetreatBattle` restores the pre-entry attempt counter and node state.
- Resolved: every committed victory or defeat consumes one combat-duration use; battle-stage abandonment restores duration and direct effect consumption.
- Resolved 2026-07-12: boss defeat enters RunFailed immediately regardless of recovery nodes. The accepted remaining-checkpoint-load action may still be offered from RunFailed.
- Resolved 2026-07-12: after a non-boss combat failure leaves zero deployable employees, defer RunFailed only if the current map exposes an immediately enterable node with explicit Treatment or Recruitment capability.
- During that deferral, only qualifying recovery nodes are enterable. After the node resolves, re-evaluate deployability and enter RunFailed if no deployable employee exists.
- Future-reachable nodes, generic Safe labels, Shops, or nodes without an explicit treatment/recruitment capability do not defer failure.

Affected children:

- G15
- G26

Recommended direction:

Define one state-transition table before implementation. Apply employee outcomes, item duration, attempt consumption, node completion, and failure atomically according to that table.

User statement recorded 2026-07-11:

The user initially stated that death has priority over battle-stage abandonment. This is not yet a complete decision because runtime review exposed two different meanings of “death.”

Runtime clarification recorded 2026-07-11:

- Battle HP reaching 0 immediately marks that runtime unit instance `Dead`, emits `UnitDied`, and reports `became_incapacitated = true`.
- This tactical death is not persistent employee death. The same employee can currently redeploy in the same battle after the 90-second defeat cooldown with 60% Battle HP.
- During `CompleteCombatResult`, each incapacitated participant result applies the live post-battle policy: trauma +40, Run HP loss equal to 25% of maximum, and an injury.
- Only then does `Employee::apply_incapacitation` set persistent `EmployeeLifeState::Dead`, and only if cumulative trauma reaches 100 or Run HP reaches 0, subject to the one-use death-prevention effect.
- The focused test `post_battle_resolution_applies_employee_incapacitation_trauma` explicitly keeps a normal employee Alive after one Battle HP 0 result and passed on 2026-07-11.

User clarification recorded 2026-07-11:

- The game intentionally uses Arknights-like same-battle redeployment.
- A tactical Battle HP 0 / `UnitDied` is not automatically permanent employee death.
- Each tactical death applies the complete authored death-penalty bundle once, regardless of whether it is the employee's first deployment or a later redeployment.
- The attempt-local employee state immediately receives the applicable trauma increase, Run HP loss, injury, trust/consumable mitigation, death-prevention consumption, and resulting life-state transition.
- The next deployment must calculate Battle HP from that updated employee state; it must not replace the result with a fixed 60% current-HP override.
- Trauma changes are immediate inside the active battle attempt but are not immediately committed as irreversible roster state.
- If working trauma reaches the persistent death threshold after the active attempt's death-prevention rules, the employee is dead and cannot redeploy for the remainder of that attempt.
- Battle-stage abandonment (`RetreatBattle`) is equivalent to a quick-load of the exact authoritative state from immediately before battle entry. It restores numeric employee condition, persistent life state, death-prevention availability, attempt count, consumables/resources, RNG state, node progress, rewards, and other gameplay state changed after entry.
- Therefore even a persistent-death transition reached during the abandoned attempt is discarded by battle-stage abandonment. The earlier “death takes priority” statement applies within an attempt and to committed non-abandonment results, not across a whole-stage quick-load rollback.

Confirmed policy regression:

- `docs/goals/core_redeploy_lifecycle_policy/POLICY_DECISION_REPORT.md` records that the earlier death-redeploy policy used updated trauma, but then replaces it with fixed 60% HP because immediate trauma timing was more complex.
- `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`, `docs/game_rulebook.md`, runtime code, and current redeploy tests repeat that fixed-60% decision.
- This historical simplification is superseded by the user's direct policy clarification. It must not be preserved as compatibility behavior.
- Current runtime first builds a roster-derived draft whose HP already reflects Run HP and trauma, then the death redeploy lock overwrites current HP with 60% of final max HP. Trauma is currently applied only during `CompleteCombatResult`, so the next deployment does not see it.

Long-term ownership:

- G26 owns an idempotent per-runtime-instance incapacitation ledger and an attempt-local employee-condition working state captured from the battle-entry roster snapshot.
- Tactical death updates the complete working death-penalty state immediately and exactly once per dead runtime instance. A fresh deployment reads that working state, so its Battle HP naturally reflects accumulated Run HP and trauma penalties.
- An accepted non-abandonment resolution commits the working condition atomically; battle-stage abandonment restores the complete pre-entry snapshot rather than selectively undoing fields.
- The pre-entry snapshot must include every authoritative mutation source touched by entry or battle execution, including attempt counters and deterministic RNG state. A partial employee-only rollback is insufficient.
- BattleCore keeps instance lifecycle and replay facts; it must not own or duplicate persistent roster policy. The world layer owns the transaction boundary.
- The fixed `DeathRedeployPercent(60)` path and tests become legacy to remove, not a fallback to retain.
- G15 consumes G26 commit/discard semantics when defining battle-stage abandonment, retry, and result transitions.

Persistent death, full-state battle-stage-abandonment rollback, and repeated-death penalty semantics are resolved. Every tactical death adds another complete death penalty; victory/defeat commits the accumulated attempt state, while battle-stage abandonment discards it. Employee-level XP, result-summary, and MVP aggregation are not death penalties and must remain one coherent employee outcome. Fatal committed results now have a defined priority over retry/cancel. Every committed victory or defeat decrements combat-consumable duration once and commits direct effect consumption; battle-stage abandonment restores both. All committed results use one coherent CombatResult before their next transition.

### Q11. Battle records, debug traces, and encounter compendium

Questions:

- Is the representative record keyed by battle, node, encounter, or abnormality?
- Should records from a timeline later rewound by checkpoint load remain?
- Is debug export gameplay-fatal?

Recommended direction:

Remove the gameplay battle-record subsystem. Keep optional developer diagnostics non-authoritative and non-fatal.

Initial user decision on 2026-07-12:

- the game does not create or retain player-facing gameplay battle records,
- remove battle-record persistence, checkpoint-history preservation, DTO/UI surfaces, record-key policy, and tests rather than selecting battle/node/encounter/abnormality identity,
- story documents and enemy-compendium entries may be collected, but they are separate collection systems and must not reuse battle-record state or schema,
- optional developer diagnostic traces may exist only outside authoritative gameplay/save/checkpoint state,
- diagnostic export failure never fails, rolls back, or changes an authoritative transition.

Corrected compendium decision on 2026-07-12:

- the game has an abnormality compendium and corrosion compendium, but no other player-facing encounter/battle history,
- collect canonical abnormality IDs and corrosion profile IDs that actually spawn in a combat-node attempt; preview-only and never-spawned wave entries do not count,
- commit those discoveries only when victory or final defeat consumes the combat node,
- retryable/non-consuming outcomes and battle-stage abandonment do not register discoveries,
- the compendium is game-global and monotonic rather than run-local,
- once committed, an entry survives run checkpoint load, older run-save load, new runs, RunFailed, and all ordinary rollback,
- only an explicit whole-game/profile or compendium reset removes discoveries,
- persist discovered ID sets only and resolve current display information from the live compendium catalog,
- node consumption plus discovery addition is one reliable idempotent commit,
- debug battle traces never produce or mutate compendium entries.

Affected children:

- G16
- G29, consuming the G15 node/result commit seam

### Q12. Employee result aggregation

Question:

How do multiple RuntimeUnit instances for one employee resolve to one persistent employee outcome?

Questions include:

- survival XP once or per deployment,
- trauma once or per defeat,
- final HP from latest deployment or worst state,
- MVP and survived flags across instances.

Recommended direction:

Keep instance history for replay, then perform one explicit employee-level persistent aggregation.

Affected child:

- G26

### Q13. Research delivery at Safe Zone return

Question:

Should pending research deliver on entry to selected node categories, or when node activity ends and the player returns to Safe Zone?

Runtime evidence:

- `ConfirmEnterNode` currently delivers on a Rust hard-coded allowlist of Support, HeadquartersContact, Shop, and Reward,
- Maintenance, Event, Combat, Boss, Gate, and Start do not deliver,
- Safe Zone itself is a Unity-local preparation/presentation space rather than a Core node category.

Corrected user decision on 2026-07-12:

- replace the earlier Maintenance-capability proposal,
- node activity queues completed research but does not deliver on any category's entry,
- automatically deliver all pending results when Core authoritatively returns from resolved node content to map/planning state presented as Safe Zone,
- never use a Unity-local screen-enter signal as grant authority,
- delivery and pending removal are atomic/idempotent and are presented in Safe Zone,
- nested Event combat waits for its owning node to resolve; a Gate delivers on the new Floor planning boundary; RunFailed does not synthesize Safe Zone.

Affected child:

- G21

### Q14. Inventory-full rewards

Question:

What happens when a result grants an item but the inventory is full and no inventory action is available?

Runtime revalidation on 2026-07-11:

- equipment copies are separate owned instances and consume one of 20 equipment slots each,
- consumable copies are also separate owned instances and consume one of 30 consumable slots each,
- artifacts use 10 unique slots and reject duplicate UUIDs,
- equipment materials and skill fragments are the actual id-keyed stack inventories,
- the consumable DTO has no quantity, and each reward grant creates a fresh owned UUID,
- the rulebook's phrase that consumable use decreases a duration or stack is stale or underspecified relative to runtime and is not a general inventory stacking contract.

User decision on 2026-07-11:

- ordinary inventory is logically infinite,
- its persisted logical `reserved_slot_count` starts at 50 and expands by enough 10-slot increments whenever a successful addition would create a new occupied slot beyond the current boundary,
- stack quantity increases do not consume another slot or expand the reserve, and the reserve never shrinks after items are removed,
- the reserve is not a separate overflow inventory and never delays a reward,
- identical consumables use one definition-id quantity stack,
- equipment remains instance-owned,
- equipment materials, skill fragments, fragment dust, and currencies remain separate non-slot resources,
- slot occupancy counts equipment instances and distinct consumable stacks,
- fixed equipment/consumable admission caps and capacity-based `InventoryFull` results are removed.

Artifact inclusion in this decision was withdrawn later on 2026-07-11 when the user identified the subsystem as legacy. The final Phase 6 choreography lets G24 establish ordinary inventory while leaving Artifact isolated and untouched, then G28 removes Artifact merchandise dependencies, and G27 deletes the isolated subsystem. Artifact is never adapted into the new ordinary inventory.

Recommended structural interpretation adopted for G24: reserve is explicit persisted Unity-facing high-water presentation state, not Rust `HashMap`/`Vec` memory capacity. A grant transaction updates it only when the corresponding new-slot addition commits successfully. Storage remains unbounded regardless of the displayed empty-slot reserve.

This resolves capacity-based ordinary-item reward deadlock without deferred claim, overflow storage, replacement selection, explicit decline, or reserved-capacity gameplay state. Artifact is no longer treated as an ordinary inventory-capacity case.

Affected child:

- G24 and G27

### Q15. Artifact system identity

Question:

Should the legacy Artifact subsystem be removed, or should the concept be rebuilt with a distinct canonical role?

Current evidence:

- the 779-line rulebook never defines Artifact as a regular build or reward system,
- ten live Artifacts are reachable through live shops,
- every owned Artifact is automatically active for every allied unit through the battle stat and side-global trigger pipelines,
- live effects are equipment-like flat/percent stats, healing, and combat triggers,
- the normal Unity inventory ignores the Core Artifact snapshot,
- existing ownership is unique, non-removable, and capped at ten, while duplicate offers fail only at purchase/grant time,
- removal is cross-layer: live shops, data loader/registry, inventory/DTO, reward/admin, scenario/runtime, stat/trigger, event log, server/Unity contracts, and tests.

Recommended direction:

- remove the current Artifact schema and effects instead of fixing duplicate acquisition around a legacy identity,
- do not adopt `ContainmentRelic`/격리 유물: it was an assistant-invented working label, not an existing setting or user-approved concept,
- remove Artifact from ordinary inventory, shops, rewards, battle scenario/stat/trigger machinery, snapshots, admin, and live data without a compatibility layer,
- do not invent a substitute system merely to preserve the old code,
- revisit a future run-wide rule-modifier system only after its setting, acquisition loop, and at least three concrete content designs prove that it changes real player decisions without duplicating equipment, skill fragments, or consumables.

The current names, effects, prices, RON schema, and identifiers are not migration requirements.

User decision on 2026-07-11:

- remove Artifact completely without replacement,
- do not preserve or migrate any current Artifact content or behavior,
- do not introduce the assistant-proposed `ContainmentRelic` concept,
- any future run-wide system is a separate goal requiring new setting and content approval.

Affected children:

- G27, with G24 consuming the resulting ordinary-inventory boundary

### Q16. Shop identity and economy contract

Question:

What canonical job should Shop perform, and which current mechanics should survive the replacement of its prototype economy?

Current evidence:

- the rulebook recognizes Shop as a one-use Safe Node but defines no merchandise, stock, pricing, sale, refresh, discount, progression, or Headquarters-Shop policy,
- map entry picks one entire merchant deterministically from a pool, then clones that merchant's fixed visible and hidden UUID pages into a session,
- `DiscountShop` has no price effect; all purchases charge the item's base price,
- non-reroll shops can contain permanently unreachable hidden stock,
- reroll is one free whole-page swap, so visible counts change instead of maintaining a stable offer count,
- live content includes an empty initial page, zero-price placeholder equipment, Artifact-only merchants, Artifact-mixed merchants, and no floor/risk gate for the merchants named as high-risk or final,
- `rachel_shop` and Headquarters basic supply are Artifact-only; Gebura and Binah mix Artifact stock,
- all seven `live_pool: true` consumables have authored prices and effects but no live acquisition route,
- Core sends visible item metadata, but Unity reduces merchandise to UUID labels and does not show price, effect, affordability, sale quote, or disabled reason,
- Core and Unity always advertise reroll in `InShop`, even when runtime will reject it,
- Unity omits consumable sales, can expose bound equipment as sellable, and retains stale local selections across stock and route changes,
- current Shop-focused tests pass, which proves the prototype is internally consistent rather than proving it is the intended game economy.

Recommended direction:

- define one shared Shop session, purchase/sale command family, offer DTO, and economy authority rather than separate Shop implementations,
- support two encounter contexts with distinct authoritative merchandise profiles: Headquarters support and a random exploration Unknown Shop,
- use the Headquarters-support profile for reliable support-oriented preparation and the Unknown-Shop profile for rarer, higher-value opportunities,
- make consumables a normal merchandise candidate so their existing live system gains an acquisition route,
- author an intentional initial catalog for each context without turning its current categories into a permanent global allowlist,
- keep the offer representation and validation extensible enough that skill fragments, abnormality-specific equipment, or other future merchandise can be added without deleting a contract that forbids those categories,
- replace static merchant UUID pages with deterministic per-session offers derived from run seed, floor/risk, node identity, and a canonical composed item pool,
- define reusable global item-pool presets and let each Shop compose preset references with weight multipliers plus optional local entries for bespoke merchandise,
- use the same typed pool-entry shape for preset and local entries; do not introduce parallel legacy and preset purchase schemas,
- flatten the composed sources before sampling, reject duplicate merchandise references across them, and sample without duplicate merchandise inside one page,
- give every pool entry a `repeatable` policy with default `false`; purchased non-repeatable merchandise is excluded from later pages in the same Shop session,
- permit `repeatable: true` merchandise to reappear after a refresh while still forbidding duplicate copies inside one page,
- treat `max_uses` as a ceiling; if purchase-history filtering leaves fewer than `offer_count` eligible unique candidates, disable refresh with an explicit pool-exhaustion reason,
- never fill a partial refresh page or override `repeatable`; a rejected refresh leaves currency, use count, and the current page unchanged,
- make selling optional per Shop through `sell_policy: None | Some(price_percent)` rather than globally enabling fixed 50% resale,
- allow an optional `accepted_tags` filter; no filter accepts every definition/state-sellable merchandise, while a filter only narrows that set,
- let definition and owned-state restrictions override Shop acceptance, including equipped, bound, quest-locked, and otherwise non-removable merchandise,
- use validated typed tag IDs rather than a Rust category allowlist,
- compute sale quotes with checked integer arithmetic and floor division; a zero quote is disabled instead of committing a zero-value sale,
- compute buy quotes with checked ceiling division from base price and the authored Shop buy percentage; allow an optional pool-entry percentage override using the same calculation,
- reject zero-price Shop purchase authoring and require every possible live sale quote to be strictly lower than every possible live buy quote for the same merchandise,
- validate the no-arbitrage invariant across all composed live Shop pools rather than only within one Shop; do not silently cap an invalid quote at runtime,
- let Core project authoritative sell availability, quote, and disabled reason and commit inventory removal plus Enkephalin grant atomically,
- make Core project offer identity, item/effect detail, final price, quantity/limit, affordability, disabled reason, sale quote, and refresh state; Unity displays this projection and does not recalculate economy rules,
- keep hidden generation state server-side,
- retain selling as a meaningful alternative to Maintenance dismantling, but move the sale ratio and eligibility into explicit policy/data,
- replace `can_reroll` and the free whole-page swap with an optional per-Shop RON refresh policy,
- treat low-value-refresh/high-value-no-refresh as a default content convention, not a runtime invariant; allow either profile to author refresh when the situation calls for it,
- require an enabled refresh policy to author `base_cost` and `max_uses`; `base_cost: 0` explicitly represents free refresh,
- author a stable per-Shop `offer_count`; both initial generation and every successful refresh produce exactly that many offers,
- refresh discards the complete current page, including slots emptied by purchases, then replaces it atomically with a newly generated deterministic page,
- project refresh availability and disabled reason from Core instead of letting Unity infer it from Shop value or encounter context,
- delete Artifact merchants and references rather than retaining IDs as compatibility shells.

The previous recommendation to simply replace Rachel/HQ stock with consumables and retain Gebura/Binah was too narrow and is withdrawn. It would preserve the prototype schema without first deciding what Shop is.

User decision on 2026-07-12:

- Shop is one shared system, not two different mechanics,
- the Headquarters-support Shop is encountered through Headquarters support,
- an Unknown Shop may be encountered randomly during exploration,
- the two contexts use different merchandise profiles,
- the Unknown Shop sells higher-value items than the Headquarters-support Shop,
- therefore the recommendation to remove Headquarters Shop access is withdrawn,
- the initial catalog may omit skill fragments or other categories, but their absence is not a permanent Shop invariant,
- do not add a contract test or validator asserting that skill fragments, abnormality-specific equipment, or similar future goods can never appear in Shop offers,
- global item-pool presets are reusable authored data; each Shop composes preset references with weight multipliers and may add optional local entries for Shop-specific goods,
- preset and local entries share one typed merchandise-entry contract,
- composition rejects duplicate merchandise references, while generated pages contain no duplicate merchandise,
- pool-entry `repeatable` defaults to `false`; the session records purchased merchandise identities and filters non-repeatable purchases from later refreshes,
- `repeatable: true` allows cross-refresh reappearance but does not permit same-page duplication,
- `max_uses` is an upper bound, not a guarantee that every remaining use stays executable after purchases,
- fewer than `offer_count` eligible unique candidates disables refresh and projects an explicit pool-exhaustion reason,
- pool exhaustion never produces a partial page, consumes cost/use, or relaxes repeatability,
- selling is authored per Shop: `None` disables it and `Some(price_percent)` enables it at that base-price percentage,
- `accepted_tags: None` accepts all merchandise allowed by its own definition/current state; `Some(tags)` narrows acceptance to validated typed tag IDs,
- definition/state denial always wins over Shop tags,
- quote rounding uses checked floor division and zero-value quotes are non-executable,
- buy quote rounding uses checked ceiling division and zero-price Shop offers are invalid,
- profitable resale is forbidden both within one Shop and across Shops; for every Shop-purchasable merchandise, `max_live_sell_quote < min_live_buy_quote`,
- same-session resale may remain available, but it must be strictly loss-making under the same invariant,
- remove the runtime-global fixed 50% resale rule; Core owns quote calculation and atomic sale commit,
- refresh availability is authored per Shop in RON rather than hard-coded by high/low value,
- default low-value Shops may enable refresh and default high-value Shops may disable it, but authored exceptions are valid in both directions,
- enabled refresh authors `base_cost` and `max_uses` in RON; zero cost is valid and explicitly free,
- the active session tracks remaining uses, and Core projects current cost, remaining uses, executable state, and disabled reason,
- each Shop authors `offer_count`; initial entry and every refresh expose that stable number of offers,
- refresh replaces the entire page and refills purchase-created empty slots rather than swapping unequal visible/hidden lists,
- the current `can_reroll` whole-page swap is not the accepted refresh contract,
- exact initial catalog, preset composition, and concrete balance percentages remain authored content work rather than unresolved global policy.

Final session/quantity addendum accepted on 2026-07-12:

- every offer explicitly authors grant quantity under the strict generated-data contract,
- stack sales carry an explicit quantity; equipment sales use exact instance UUID and quantity one,
- Core validates quantity and computes checked total quotes,
- a Shop session persists its current page and purchased holes, purchased non-repeatable identities, refresh use count, and deterministic RNG continuation across reopen/reconnect,
- checkpoint load restores the Shop session rather than regenerating it,
- an exploration Shop discards its session only on node completion; a Headquarters sub-Shop shares the parent session lifecycle,
- initial refresh cost remains the same authored `base_cost` for every successful use; escalation requires a later explicit cost schedule,
- these additions resolve the remaining structural policy gap; concrete merchandise and balance remain incremental content work.

Affected children:

- G28, G27, and G24

## 2026-07-12 — Remaining policy triage after Shop decisions

The policy registry still mixes user-facing gameplay choices with technical contract defaults. The user delegated basic policies to the assistant. Do not continue asking about serialization strictness, arithmetic, validation, DTO authority, or removal of unused duplicate fields unless implementation uncovers a material gameplay consequence.

User-facing decisions resolved on 2026-07-12:

1. G15: boss defeat is immediate RunFailed; non-boss NoDeployableEmployees failure is deferred only by immediately enterable Treatment/Recruitment-capable nodes.
2. G11/G13: remove enemy range preview entirely; actual attack telegraphs remain a separate system.
3. G16/G29: remove gameplay battle records, isolate optional developer diagnostics, and implement the separate monotonic global abnormality/corrosion compendium committed on combat-node consumption.
4. G21: pending research delivers automatically on the authoritative return to Safe Zone/planning, not on entry to Maintenance or another category.
5. G22: defer Equipment/Maintenance content scope and balance until those designs are ready; do not invent or preserve placeholder policy.

Assistant-owned technical defaults, not further user questions:

- G06: remove `allowed_actions` and establish the accepted two-level Core contract: context `action_capabilities` plus typed domain payload eligibility. Capability projection and execution share validators; execution revalidates atomically.
- G03/G02: Event is supported live functionality because live generation guarantees it; Unity aligns to the typed Core DTO without aliases.
- G09/G10/G18: implement accepted PD-06 by separating Global cast reach from required target-centered TileArea footprint, capturing the selected tile, and removing the mixed legacy schema. The future caster-centered circular field wave remains documentation-only until its own implementation goal.
- G10: accepted PD-07 makes explicit per-skill targeting selection authoritative; basic attacks alone use weapon profiles, and missing skill selection fails live loading.
- G12: accepted PD-09 makes generated gameplay RON exact-schema and maximally strict; historical aliases, first-match duplicates, invalid references, unsupported variants, and partial bundles are rejected.
- G19: remove unused duplicate `default_blocking` authoring instead of defining a second fallback authority.
- G28: accepted PD-18 includes explicit quantities, persistent deterministic sessions, constant initial refresh cost, strict quotes, and prototype removal; initial pools/percentages remain authored balance.

Status correction:

- G28 is ready for child creation; remaining initial Shop values are authored content work, not a global policy gate.
- G24 policy is accepted and is blocked only by the G27 Artifact-removal boundary.
- G11, G15, G16, and G21 now have accepted user-facing policy and are ready subject to their recorded dependencies.
- G22 is explicitly deferred by the user.

## Phase-Oriented Execution and Review Decision

User decision on 2026-07-12:

- implement the accepted policies as dependency-aware Phases rather than policy-number order,
- continue using bounded child goals for one source of truth or tightly coupled boundary; a Phase is orchestration, not one giant implementation goal,
- after every Phase, create a dedicated review goal and apply `docs/goal_completion_review_guide.md`,
- do not begin/merge the next Phase until the review proves the current Phase rather than trusting child checkboxes,
- review goals are read-only by default; broad fixes become bounded correction child goals and the Phase review is rerun,
- preserve failed/partial review evidence instead of overwriting it after corrections.

Accepted Phase grouping:

1. Phase 0 restores trustworthy Core/server/Unity fixture baselines through G04.
2. Phase 1 establishes revision, capability, battle reconnect/resync, and strict generated-data authority through G01/G06/G07/G12.
3. Phase 2 establishes employee attempt state, exact result/abandonment transactions, BossOmen Gate/failure recovery, and Unity recovery through G26/G15/G14/G08.
4. Phase 3 implements PD-03/PD-04 as one Event vertical slice after shared combat transitions are stable, then removes duplicate VN authority through G02/G03/G19.
5. Phase 4 handles reproduction/reward seams, battle-record removal, permanent global compendium, and authoritative Safe Zone research delivery through G17/G23/G16/G29/G21.
6. Phase 5 removes enemy preview before final per-skill/global-area targeting migration, then updates canonical examples/evidence through G11/G10/G09/G13/G18/G20.
7. Phase 6 is one coordinated but separately owned economy campaign: G24 final ordinary inventory, G28 final Shop and Artifact merchandise removal, then G27 complete Artifact deletion.
8. Phase 7 performs workflow/canonical synchronization and the final master completion review through G25 and the accumulated Phase evidence.

Review gate judgment:

- Phase pass requires every required item to be `Complete` or evidence-backed `Not applicable`, no immediate correction, required focused/broad validation, and no cross-component authority conflict.
- `Partially complete`, `Document-only complete`, `Policy decision needed`, `Low` long-term fit, missing mandatory runners, or unowned cross-component findings block the next Phase.
- `Medium` fit may pass only when it has no current contract risk and a bounded follow-up owner/trigger.
- Each review goal creates the mandatory three goal documents plus `COMPLETION_REVIEW.md` using the guide evidence template.

Deferred lane:

- G05/PD-05 and G22/PD-14 do not receive placeholder implementation merely to close the active sequence.
- If activated later, each becomes a normal reviewed phase with user-approved content policy and values.

## 2026-07-14 — Phase 2 closure and Phase 3 entry boundary

- R02 is Complete / High with no immediate correction; Phase 2 is closed.
- Phase 3 consumes the fixed Phase 1/2 authority and transaction seams. Event work must not add an
  Event-specific battle transport, result settlement, abandonment rollback, BossOmen recovery, or
  Unity execution authority.
- G02 starts first and owns shared selected-event/existing wire identity alignment only. G03 owns
  the illustrated employee-check state machine and flow; G19 owns duplicate VN authority removal.

## Phase 0 Execution Note — 2026-07-12

G04 is now active at `docs/goals/server_runstart_fixture_contract/`.

The first current-state observation is that the reported server fixture owner, `../game_server/src/game/player_game_actor/handlers.rs`, is already modified in the user's dirty worktree. G04 must inspect the exact current diff before patching and preserve every unrelated change. A green server suite obtained by relaxing the Standard RunStart trigger invariant, adding a test-only bypass, or deleting the affected assertions would fail the child contract.

No later Phase implementation is authorized until G04's child review and the separate R00 Phase review pass.

G04 completion update:

- G04 passed its child review with `Complete` / High fit.
- The historical server result changed from 19 passed / 6 failed to 25 passed / 0 failed while Core remained 681 passed / 0 failed.
- The paired server fixture closes DCA-H-08 and the regression-gate portion of CPR-M-01.
- CPR-M-01 also describes late supported-mode RunStart presence/policy validation. That concern is not closed by a fixture edit and is now explicitly assigned to G12's strict bundle-validation scope.
- Phase 0 is `review_required`; R00 must independently verify the evidence and ownership handoff before Phase 1.

R00 completion update:

- R00 independently reproduced Core 681/681 and server 25/25 and passed with Complete/High.
- Phase 0 is complete; no correction child is required.
- Phase 1 is active. G01 is the first executable child; G06 and G07 remain ordered behind it.
- G12 is also Phase 1-ready as the strict-data boundary, but it must not be implemented in parallel with overlapping public/runtime seams unless its child PLAN proves independence.

G01 startup update:

- G01 documents exist and the child is active.
- Current evidence confirms `battle_state_revision` must be implemented together with Core cursor removal, a proven single server sender/outbox, and Unity revision-first application. Treating it as a DTO-only field would fail PD-01 and the long-term review.
- G06/G07 capability semantics remain out of G01 except for the revisioned extension seam.

## Cross-Goal Collision Notes

### Battle transport files

G01, G06, G07, and G08 may all touch battle DTOs, capability projection, server side messages, and Unity stores.

Rule:

- G01 establishes revision semantics first.
- G06 establishes capability meaning.
- G07 combines them for battle/reconnect.
- G08 consumes the resulting public contract.

Do not implement them in parallel if they edit the same public DTO.

### Unity command and screen files

G02, G03, and G08 can overlap GameCommandBus, action names, snapshot readers, presenters, and tests.

Rule:

- G02 aligns existing field identities first.
- G03 adds Event flow against the corrected typed contract.
- G08 adds recovery and roster command flows after capability policy is stable.

### Range and live schema files

G05, G09, G10, G11, G12, G18, and G22 can overlap skill targeting contracts, range schemas, live RON, and previews.

Rule:

- G12 establishes validation ownership.
- G05 handles current live abnormality range content.
- G09 and G10 settle skill delivery and target authority.
- G11 settles external preview projection.
- G18 updates examples last.

### Combat result and checkpoint files

G14, G15, G16, G17, and G26 can overlap world/combat state transitions, checkpoint payloads, result statistics, and records.

Rule:

- resolve BossOmen and general failure policy first,
- then employee aggregation and record identity,
- keep debug export isolation independent where possible,
- do not let telemetry failure participate in authoritative commit.

### Artifact legacy boundary

G27 overlaps G02, G12, G23, G24, and G28 through inventory/root snapshots, strict live-data validation, reward kinds, shop stock, and public DTOs. It also removes or replaces battle scenario, stat, trigger, event-log, and admin surfaces.

Rule:

- PD-17 is settled: there is no replacement run-relic state,
- G24 first establishes the final ordinary equipment/consumable identity, quantity, stack, reserve, and transaction boundary while leaving Artifact isolated,
- G28 then consumes that final ordinary-inventory boundary and removes Artifact-dependent Shop content/contracts,
- G27 finally deletes the isolated Artifact registry/schema and every remaining cross-layer surface,
- G12 and G23 consume the resulting schema and reward vocabulary rather than independently preserving `GrantArtifact` or Artifact validation,
- G24, G28, and G27 may share a coordinated integration branch but do not implement or review the same source-of-truth seam in parallel.

### Shop economy and Unity offer boundary

G28 overlaps G02, G06, G12, G24, and G27 through selected-event DTOs, contextual actions, Shop RON validation, consumable stacks, Artifact removal, and the actual Unity Shop route.

Rule:

- begin G28 only after G24 establishes the final ordinary inventory quantity/identity authority,
- G28 owns Shop identity, offer generation, authoritative price/eligibility projection, refresh semantics, sale quotes, Headquarters-Shop separation, and the matching Unity interaction contract,
- G06 may generalize contextual capability infrastructure, but it must not independently invent Shop-specific eligibility or refresh behavior,
- G28 consumes G24 transactions for definition-stacked consumables and equipment instances; it does not create a parallel Shop inventory adapter,
- hidden generation state remains Core-private unless an explicit gameplay requirement proves otherwise,
- do not preserve the current `ShopType`, visible/hidden dual-page schema, or UUID-only Unity mock as a compatibility contract.

## Out-Of-Scope For This Master Initialization

- No Core code changes.
- No live RON changes.
- No game_server changes.
- No Unity changes.
- No child goal directories.
- No policy choice is adopted merely because this file recommends it.
- No compatibility or migration layer is authorized.
- No temporary audit file is deleted or promoted yet.

## Excluded Finding

DCA-M-26 is excluded as not actionable.

Reason:

The durable rule says there is no separate live Maintenance CraftEquipment action. EquipItem combination is an existing distinct action path. These statements can coexist.

Re-open only if a real UI, command, or data contradiction is demonstrated.

## Supplemental CPR Findings

The following confirmed issues are not independently represented by DCA and must not be lost:

- redeployed instances must retain intended per-death penalties while eliminating duplicate employee-level XP, summary rows, and unstable MVP/status aggregation,
- fatal defeat exposing retry and cancel before RunFailed,
- debug filesystem export breaking authoritative transition,
- late Event RewardEffect validation and panic variant,
- whole-stage `RetreatBattle` rollback is intended; the remaining defect is the absence of transactional live-trauma state for redeployment,
- missing extra-wave and map-event bundle validation.

Their primary owners are G26, G15, G16, and G12 as listed in PLAN.md.

## Terminology Decision

The user clarified on 2026-07-11 that the rollback discussion concerns whole-stage abandonment, not a deployed unit leaving the field.

- `WithdrawUnit` / 유닛 철수: individual unit leaves; battle continues; withdrawal redeployment rules apply; no snapshot rollback.
- `UnitDied` / 전투불능: individual runtime unit reaches Battle HP 0; attempt-local trauma and death redeployment rules apply.
- `RetreatBattle` / 전투 포기: abandon the entire combat stage; restore the exact pre-entry authoritative snapshot.
- `BattleEnd` defeat / 전투 패배: battle finishes without abandonment; non-abandonment result commit rules apply.

Do not use bare `퇴각` in policy prose going forward.

## Follow-Up Candidates

Record here until the owning child begins:

- Exact cross-repository revision fixture and actual Unity consumption were completed by G01; later DTO goals should reuse that checked-in typed-generator pattern.
- Unity MCP against the already-open Editor is the proven test path; do not create a second project copy while that session is available.
- Whether manifest behavior evidence should identify concrete test functions or scenario IDs.
- Whether runtime identity risk experiments should compare event-order traces before and after observation-only log insertion.
- Whether the master registry should move to a machine-readable manifest after the first child reviews prove the schema useful.

Do not implement these candidates from the master directly.

## G01 Completion Boundary

- G01 review: Complete / High / improvement class none on 2026-07-13.
- Core owns checked battle projection revision commits; server connection owns event delivery cursor/order; Unity owns last-applied UUID/revision and presentation continuity.
- Same-battle reconnect/setup recovery is canonical. Destructive setup-loss recovery, Core cursor ownership, time/seq freshness, and revisionless mocks are closed legacy paths.
- G06 may add common context capability and payload eligibility projection, but it must preserve G01 commit/reconnect ownership and leave battle-specific expected-revision integration to G07.

## G06 Startup Boundary

- G06 documents were created on 2026-07-13 before common capability code search.
- G06 owns the shared typed capability/scope/reason/projector/preflight seam and root/server/common Unity migration.
- Battle capability/expected revision/static whitelist remains G07; concrete Event/Safe Zone/inventory/Shop payload policy remains with its registered domain child.

## G06 Completion Boundary

- G06 review: Complete / High / improvement class none on 2026-07-13.
- Core common/root projection plus current-state preflight is the capability authority; server is
  transparent and Unity does not reconstruct absent common actions.
- `allowed_actions`, cached `ActionValidator`, `ActionScheduler`, old admin terminology, and
  checkpoint DTO `can_load` are removed with no alias or fallback.
- G07 inherits `ActionCapabilityDto`, scope/reason vocabulary, the strict Unity reader/store, and
  G01 revision semantics. It owns battle context/payload projection, expected revision command
  integration, reconnect/resync capability behavior, and removal of the remaining battle-only
  static Unity action set.
- Domain children retain concrete payload eligibility ownership; they must extend the typed seam
  rather than introduce another global table.

## G07 Startup Boundary

- G07 documents were created on 2026-07-13 after rereading R00, G01, G06, PD-01, PD-02, and Phase 1
  authority.
- G07 must consume rather than redefine `battle_state_revision`, server cursor/order ownership, and
  the G06 typed capability model.
- Its implementation boundary ends at current battle capability/eligibility, stale command
  preconditions, reconnect/resync integration, static Unity battle substitution removal, and the
  contradictory server helper contract.

## 2026-07-13 — G07 completion boundary

- G07 is Complete / High / no immediate correction.
- Core battle mutation authority is exactly `battle_uuid + battle_state_revision`; event seq and
  battle time remain delivery continuity and simulation/presentation facts only.
- Revisioned battle context capability and bounded domain payload eligibility are one complete Core
  projection. Unity has no static/root/local execution-authority fallback.
- Query/setup-loss recovery remains available through its typed scope during blocking battle VN;
  final recovery/roster UX remains G08.
- Selected deployment preview is the Core authority for employee/tile/facing eligibility. Final
  skill targeting/range policy remains G09/G10 and was not pulled into G07.
- Live RON and saves were not changed. G12 remains the next Phase 1 child for strict generated-data
  and bundle-validation authority, then R01 must review the combined Phase 1 boundary.

## 2026-07-13 — G12 startup boundary

- G12 documents were created only after rereading PD-09, its four finding handoffs, Phase 1 gates,
  G04/R00, and G01/G06/G07 completion boundaries.
- G12 must establish one exact-version canonical production bundle and all-or-nothing validation
  publication seam; later domain goals extend that seam rather than adding lenient loaders.
- The first implementation decision must follow actual production loader/live-data discovery. The
  master does not preselect a wrapper, version-field location, or diagnostic type.

## 2026-07-13 — G12 completion boundary

- G12 is Complete / High / no immediate correction.
- Core owns exact gameplay-RON schema acceptance and the final all-or-nothing return boundary;
  game_server only consumes success and Unity does not reproduce data validation.
- Candidate allocation before generated preview validation is private and non-publishing; no
  candidate escapes if preview/briefing generation fails.
- Fixture builders may retain panic wrappers around the same validators, but production startup does
  not enter them and they are not compatibility/fallback paths.
- Later domain goals must extend the shared parsed-bundle validator and canonical schema rather than
  adding independent readers or aliases.
- R01 is now the only Phase 1 exit gate; Phase 2 implementation remains blocked until it passes.

## G09 Completion Boundary

- G09 is Complete / High / no immediate correction on 2026-07-14.
- Skill cast targeting owns explicit target selection, acquisition reach/range, and air eligibility;
  TileArea delivery independently owns one concrete affected-tile footprint.
- Selected cast tile capture survives target death/withdrawal and remains the delivery center;
  Unity consumes Core final cells/events through strict catalog v2 without geometry reconstruction.
- Step-local range/air authority, mixed WholeField/TileArea meaning, implicit live reach/air, and
  loadable expanding-wave placeholders are closed legacy paths.
- G13 may now clean battlefield projection wording; G18 follows with final skill examples, then G20
  owns live catalog evidence governance before R05.

## G13 Completion Boundary

- G13 is Complete / High / no immediate correction on 2026-07-14.
- Actor HUD world position is projection input, not render space; the dedicated screen-space overlay
  implementation and canonical wording now agree.
- FacilityEntity remains a future/wire-reserved enum/parser shape that strict live validation and
  combat setup do not produce. Enum existence is not production support.
- G18 may update final skill examples without revisiting G11 preview removal, G09 range/delivery
  authority, or G13 battlefield presentation policy.

## G18 Completion Boundary

- G18 is Complete / High / no immediate correction on 2026-07-14.
- Current canonical examples are downstream of strict live RON, runtime targeting, typed catalog v2,
  and actual live skill ids; historical ids/fields/versions are not compatibility documentation.
- G20 inherits the final live schema and may strengthen evidence governance, but must not reinterpret
  catalog membership as behavior proof without an explicit scenario link.

## G20 Completion Boundary

- G20 is Complete / High / no immediate correction on 2026-07-14.
- Manifest v2 is derived membership/delivery/reference ownership evidence, not a behavior
  certificate. Named executable assertions remain the only behavior proof.
- The fragment index is mechanically complete for 30 abnormality-origin fragments; the concept
  starter and 22 full E.G.O set subset remain distinct counts.
- R05 must independently review the combined Phase 5 boundary and rerun its own focused/broad gates
  rather than treating child completion documents as proof.

## R05 Initial Review Boundary

- Phase 5 runtime/data/DTO/canonical/evidence composition is Complete / High.
- R05 cannot pass because full Unity PlayMode is sequence-dependent after focused hover tests; exact
  starter-selection rerun reproduces the retained pointer state.
- G31 owns only deterministic Input System pointer/test cleanup isolation. It must not change starter
  selection UX or Phase 5 gameplay contracts to make assertions pass.
- After G31 review, R05 appends correction evidence and reruns the same focused→full order plus a
  repeated full suite. Phase 6 remains blocked meanwhile.

## G31 Completion Boundary

- G31 is Complete / High / no immediate correction on 2026-07-14.
- Test input/object lifecycle is now class-level fixture authority; production starter hover UX and
  every Phase 5 DTO/runtime contract are unchanged.
- R05 correction rerun must not reuse G31's verdict as proof: it independently repeats focused Core /
  Unity composition, broad Core/server/EditMode, focused→full PlayMode, and console/hygiene checks.

## R05 Completion Boundary

- R05 is Complete / High / no immediate correction after its appended rerun on 2026-07-14.
- Phase 5 owns final Player-only previews, explicit skill selection/reach/air, independent TileArea
  footprint/captured anchor, catalog v2, current canonical examples, and non-overclaiming catalog
  evidence governance.
- G31's class-level Unity test isolation remains validation infrastructure and does not alter UX.
- Phase 6 begins with G24 ordinary inventory quantity/capacity authority; it must not preserve or
  normalize Artifact into that ordinary inventory contract because G27 removes Artifact later.

## G24 Completion Boundary

- G24 is Complete / High / no immediate correction on 2026-07-14.
- Core `Inventory` is the sole ordinary equipment/consumable identity, quantity, occupancy, and
  reserve authority. Unity consumes the persisted reserve and required quantity without a fallback.
- Ordinary grants cannot fail because slots are occupied; invalid identity/quantity mutations remain
  checked and atomic. Checkpoint and battle-entry clones preserve exact high-water state.
- Artifact remains an unchanged isolated legacy collection excluded from ordinary occupancy/reserve.
- G28 is now active and must reuse G24 transactions for purchase and quantity-aware sale. It owns
  Shop session/economy/quote/refresh behavior and Artifact merchandise removal.
- G27 remains blocked on G28; R06 remains blocked on G27.

## G28 Completion Boundary

- G28 is Complete / High / no immediate correction on 2026-07-14.
- Core owns deterministic Shop composition/session/page identity, checked total buy and quantity-aware
  sale quotes, typed eligibility, atomic commits, and checkpoint restoration.
- Headquarters and Unknown contexts share the same contract through distinct authored profiles;
  Unity consumes full typed offers/reasons/quotes and sends offer or owned-item selectors only.
- Prototype fixed visible/hidden pages, reroll vocabulary/behavior, raw UUID-only offers, global 50%
  sale, Artifact merchandise, and zero-price placeholder offers are closed production paths.
- G27 is now active. It may delete Artifact from live data, loaders, registries, inventory/reward/
  battle/runtime/server/Unity contracts and tests without touching or adapting the final Shop.
- R06 remains blocked until G27 passes its own completion review.

## G27 Completion Boundary

- G27 is Complete / High / no immediate correction on 2026-07-14.
- The legacy subsystem has no current live-data, schema, registry, inventory, reward/admin, battle,
  DTO/server, generated-fixture, Unity or canonical surface and no replacement/compatibility state.
- G24 ordinary inventory and G28 Shop are the only current acquisition/storage/sale boundaries and
  were not reinterpreted during deletion.
- Historical goal text remains historical evidence; it does not authorize current decoding,
  runtime behavior, content or public fields.
- R06 is now active and must independently rerun the combined live acquisition-to-storage-to-sale
  evidence plus repository-wide legacy Shop/fixed-capacity/deleted-subsystem audits.

## R06 Completion Boundary

- R06 is Complete / High / no correction finding on 2026-07-14.
- Phase 6 has one composed Core economy authority from live acquisition through inventory,
  checkpoint/rollback, Shop offer/quote/refresh/sale and Unity projection.
- Ordinary fixed-capacity admission, prototype Shop behavior and the deleted legacy subsystem have
  no current production/compatibility path.
- Phase 7 begins with G25 governance/canonical synchronization. R07 must independently review the
  entire active master scope after G25; it cannot reuse R06 as final proof.

## G25 Completion Boundary

- G25 is Complete / High / no immediate correction on 2026-07-14.
- Current work discovery is owned by master/goal status; durable policy is owned by domain/wire
  canonicals; completed goals remain historical evidence and cannot be the sole live authority.
- The probe registry owns routing only. External Unity contracts own payload semantics and probes
  remain executable consumers.
- All active-scope master findings and children now have a consistent disposition. G05 and G22
  remain user-deferred with unchanged activation triggers.
- R07 is active and must independently validate final cross-layer composition, ownership,
  legacy absence, broad suites and deferred/excluded disposition.

## R07 And Master Completion Boundary

- R07 is Complete / High / no correction child on 2026-07-14.
- Phase 0..7, all non-deferred children and both correction reruns are terminal with current combined
  executable evidence and durable policy ownership.
- G05/G22 retain their explicit user/content triggers; they were not implemented, invented or
  silently converted into active defects. EX-01 remains not applicable.
- No active child, review, correction or unowned active-scope finding remains. Future work begins
  only through a new request or the documented deferred activation path.

## 2026-07-13 — R01 startup boundary

- R01 is review-only and must treat child completion documents as inputs rather than proof.
- It owns the combined Phase 1 authority audit, not new revision/capability/data policy.
- Any material correction keeps Phase 1 active and becomes an explicit correction goal; R01 must not
  hide it behind documentation-only completion.

## 2026-07-13 — R01 correction boundary

- G30 is the proposed single correction goal for MSTR-05/MSTR-06 because both are the same G07
  battle execution/recovery authority seam across Core/server/Unity.
- Long-term correction must compose one pure battle context evaluator for projection and execution;
  a handler-by-handler blocking-VN special case is not acceptable.
- Missing and stale authority keep distinct typed errors but share the same non-mutating current
  resync builder and ordering.
- Deep `BattleCore::deploy_player_unit` staging/rollback remains a non-blocking follow-up candidate;
  do not expand G30 unless source work proves it is required for the two review findings.
- No new gameplay/UX/schema policy is needed. The only external block is authorization to start the
  unplanned correction child under the user's working protocol.
