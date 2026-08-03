# Battle Capability And Reconnect Authority Notes

## Approved Boundary

- G01 owns battle UUID/revision commit semantics, server delivery cursor/order, Unity freshness,
  reconnect ordering, and same-revision resync.
- G06 owns shared action identity/scope/reason/capability types, common/root projection, strict Unity
  parsing/localization, and removal of the old global table/cache.
- G07 owns battle context capability, battle payload eligibility/preflight, expected-revision command
  behavior, blocking-VN recovery reachability, reconnect/resync capability integration, and removal
  of Unity's battle-only static action set.
- Later domain goals retain targeting/range, Event, result/retreat, employee condition, BossOmen, and
  recovery UX policy.

## Initial Structural Questions For Source Review

- Which exact battle mutation commands already share pure validators, and which validate only inside
  mutation handlers?
- Which eligibility spaces are bounded DTO lists (employees, deployed units, skills) and which need a
  selected-payload preview (employee/tile/target combinations)?
- Where can expected battle UUID/revision be placed without duplicating authority across command
  envelope, individual payloads, server actor, and Core behavior?
- How are setup loss, attach, blocking battle VN, and explicit resync currently distinguished?
- Which battle presentation rules are still inferred in Unity beyond the static action set?
- Which server helper document contradicts the actual ordered side-message implementation?

## Long-Term Constraints

- Do not add a second revision counter or use event seq/time as stale-command authority.
- Do not cache capability separately from its committed revision unless it is a derived, revision-
  keyed presentation cache with no execution authority.
- Do not enumerate an unbounded employee/tile/skill/target Cartesian product.
- Do not let recovery/query disappear merely because ordinary gameplay is blocked.
- Do not preserve current static/local Unity behavior behind a transition flag or fallback parser.

## Follow-Up Candidates

None at initialization. Record only after current-source discovery.

## Exploration Tool Failure

- The first semantic search across Core/server/Unity returned HTTP 429 for every request.
- No source conclusion is based on it. The fallback is bounded local source inspection plus the
  already assigned read-only Warp Grep subagent; Unity state/test verification remains MCP-owned.

## First Revalidated Authority Split

- Core update projection is revisioned but capability-free. G07 should extend the complete update,
  not create a second standalone capability message.
- Expected battle authority is absent at every command boundary. The long-term seam should be one
  typed execution context/envelope value passed to Core once, not copied into every battle behavior
  variant or checked only in game_server.
- Server actor already owns event delivery cursor/order. Expected UUID/revision must not be added to
  that cursor or used as a replacement for ordered side-message delivery.
- Query and recovery commands are semantically distinct: `RequestBattleState` and deployment-range
  preview are query-scoped; setup-loss recovery is recovery-scoped. They should remain executable
  without a mutation expected-revision precondition while still carrying/returning current
  authority where needed for resync.
- Current Core deployment and skill handlers contain real atomic validators. Extract/compose them
  rather than reproducing their rules in a projector.
- Current Unity local deployment validation duplicates Core tile/affinity/occupancy policy and cost
  readiness. It may remain useful for rendering geometry only after Core provides eligibility, but
  it cannot remain command authority.
- Current `LiveBattleSkillReadinessDto` already has `manual_activation_allowed`, target presence, and
  reasons, but its free-form strings are not the stable typed reason contract accepted by PD-02.
- Battle playback projection already contains paused/speed facts; G07 must project mutually exclusive
  Pause/Resume and selected-speed eligibility from the same state rather than asking Unity to infer.

## Dirty Worktree Boundary

- Core `behavior.rs`, `world.rs`, `combat.rs`, `state.rs`, tests, server actor files, and Unity Core/
  test files already include user or G01/G06 work. G07 patches must preserve all unrelated diffs.
- Deleted battle records, documentation reorganizations, log files, unrelated UI goals/assets, and
  generated cache files are not G07 scope.

## Baseline Classification

- Core focused 9/9 and server focused 5/5 protect G01 revision, query, recovery, tick, and final
  ordering behavior and must remain green.
- Unity focused 6/6 proves the open MCP runner and current transport behavior. Static battle-action
  substitution and local presenter readiness tests are legacy evidence, not completion tests for
  G07.
- The deployment-range query test remains useful but its current ungated submission will need to
  become query-capability/payload-preflight aware without turning it into a gameplay action.

## Battle Action And Payload Ownership Matrix

| Action | Scope | Context projection source | Payload eligibility/preflight source | Expected authority |
| --- | --- | --- | --- | --- |
| `RequestBattleState` | query | active battle exists | `since_seq <= latest_seq` in read-only projection | not required; returns current authority |
| `RecoverBattleSetupLoss` | recovery | active battle exists, including blocking VN | immutable setup plus current update projection | not required; returns current authority |
| `RequestDeploymentRangePreview` | query | at least one currently selectable deployment payload and no blocking VN | employee/position deployment validator plus range builder | not required; read-only |
| `DeployUnit` | gameplay | any employee has an executable deployment payload and no blocking VN | employee state, affinity/cell, deployed/redeploy lock, current cost, profile/static validation, selected facing | required |
| `WithdrawUnit` | gameplay | any currently deployed player employee and no blocking VN | exact deployed employee lookup plus BattleCore withdrawal validation | required |
| `ActivateSkill` | gameplay | any deployed employee has executable current readiness and no blocking VN | existing `resolve_manual_skill_cast_request` / selected target validation | required |
| `RetreatBattle` | gameplay | active battle and no blocking VN | current G15-owned handler policy remains unchanged | required |
| `PauseBattle` | gameplay | active, unpaused battle and no blocking VN | same playback predicate | required |
| `ResumeBattle` | gameplay | active, paused battle and no blocking VN | same playback predicate | required |
| `SetBattleSpeed` | gameplay | at least one non-current speed and no blocking VN | per-speed entry; current speed disabled with typed `already_active` | required |
| battle `AdvanceVisualNovel` | gameplay | active battle VN whose scene advances/ends | existing VN scene/session validator | required |
| battle `SelectVisualNovelChoice` | gameplay | active battle VN whose scene has concrete choices | existing VN choice/session validator | required |

Context capabilities belong in every `LiveBattleUpdateDto`. Bounded payload eligibility belongs in
typed battle-domain projection, not a generic bag. Playback speeds and finite employee identities
can be projected. Exact deploy employee/tile/facing and skill target combinations must still use
selected Core preview/preflight rather than an unbounded Cartesian product.

## Expected Authority Placement Decision

- Add one typed `BattleStateAuthorityDto { battle_uuid, battle_state_revision }` to the top-level
  player command envelope, not to every `PlayerBehavior` variant.
- Pass it once through a typed Core execution context alongside source command ID and server event
  cursor. Core, not game_server, decides whether the current behavior is a battle mutation and
  validates exact UUID/revision before any domain mutation.
- Query/recovery omit the expected authority and return current authority through existing revisioned
  update/resync transport. Battle mutations, including battle VN progression and retreat, require it.
- A missing authority for an active battle mutation is a typed rejection. A mismatched UUID/revision
  changes no state/revision. game_server emits the request-scoped typed error followed by the current
  `battle_resync` through the existing ordered actor boundary; do not add a generic JSON details bag.
- Unity derives the envelope authority only from `BattleRuntimeState.BattleId` and
  `LastAppliedRevision` at send time. It never fabricates revision 1 or falls back to seq/time.

This is a technical realization of accepted PD-01/PD-02, not a new gameplay/UX policy decision.

## Independent Warp Grep Reconciliation

- The read-only Warp Grep independently selected the same command-envelope authority and revisioned
  update capability placement.
- It confirmed blocking VN currently stops tick only, not all gameplay commands, and that
  deployment preview omits cost/cooldown/deployed checks.
- It identified `BattleCore::resolve_manual_skill_cast_request` as the strongest shared pure skill
  seam; `live_skill_readiness` currently duplicates it and can drift.
- It recommends ordered `error -> battle_resync` for stale authority, preserving G01 actor cursor/
  outbox ownership.
- It confirmed the current test that leaves manual skill enabled with no valid target conflicts with
  accepted PD-02 and must be replaced with typed `no_valid_target` eligibility.
- It confirmed the stale helper contract at `game_server/docs/unity_bridge_protocol.md` and the exact
  Unity local inference surfaces already found locally.

## First Reversible Authority Trial

- The optional supplied-authority check is intentionally an intermediate experiment. Completion
  requires active battle mutations to reject missing authority with no legacy fallback.
- Comparing authority before common/domain validation proves stale rejection can precede every
  mutation without altering handler policy.
- The first compile failure is useful exhaustive-match evidence: every new public error variant must
  update snapshot/server mappings explicitly rather than disappear into catch-all handling.
- A subsequent 0-test exact filter is not counted. The goal retains it as failed command-selection
  evidence and requires a nonzero rerun.
- The nonzero 1/1 rerun confirms pre-mutation authority comparison is a viable shared seam. It does
  not authorize retaining `None` as a mutation fallback at completion.
- Server production compiles with the typed envelope field. Ten compile failures are test literal
  migration inventory, not a reason to add serde aliases, inferred current revision, or a defaulted
  internal command constructor.
- Server test fixtures now state authority presence/absence explicitly, and envelope deserialization
  preserves the typed UUID/revision. `None` remains temporary for mutation fixtures until the atomic
  mandatory-authority cutover.

## First Battle Projection Slice

- The central final projection hook is preferable to putting capability in a separate side message:
  it sees the current global VN state and stamps the same already-committed update object.
- The low-level empty vector is an intermediate builder detail, not an externally valid final
  projection. Completion must prove every emitted update passes the finalizer.
- Initial deployment context enabling currently uses finite cost/readiness facts only and is a trial;
  final projection must compose exact deploy/cooldown/cell/profile preflight and typed payload DTOs.
- The first playback test failure happened in pre-run setup, before this projection. It must be
  reproduced before drawing a dependency conclusion.
- Reproduction identified a control-flow placement bug, not a policy conflict: the finalizer is
  global, so battle-only projection must be lazy inside battle-result variants. Keep the finalizer;
  narrow evaluation rather than adding a fake empty-battle fallback.
- Focused behavior now proves the important recovery invariant: blocking VN removes ordinary battle
  gameplay capability but retains revisioned query/recovery and the one valid VN progression family.
- game_server's manual top-level BattleUpdate field enumeration is a required explicit transport
  seam. Compile failures in exact patterns/fixtures prevent silent field loss and should be converted
  into preservation assertions.
- Top-level BattleUpdate and nested BattleResync now preserve one typed update-owned capability
  vector. No root snapshot capability is used as a battle fallback.

## Unity Context Capability Cutover

- Battle capability is now state owned by `BattleRuntimeState` and replaced only when an update/
  authoritative resync projection applies. Root capabilities remain separate for non-battle routes.
- Missing or malformed battle capabilities reject the whole projection and request resync; Unity
  never substitutes the old static set.
- The first four failing tests are expected stale-fixture evidence. Shared test builders must emit
  current typed objects, and old all-actions expectations must be replaced with exclusivity/recovery
  behavior.
- Shared fixtures now emit explicit capability objects and the static compatibility behavior test is
  gone. Unity obtains mutation authority only from the applied runtime revision; query transport
  remains authority-free.
- One MCP polling connection dropped transiently, but the same persistent job completed 2/2 and was
  retrieved by job ID. No test rerun or result fabrication was needed.
- Stale recovery does not place current authority in an untyped error-details bag. The ordered
  BattleResync is the typed current authority/projection and reuses G01 same-revision repair.
- Unity requests resync from its last applied event seq only after the typed stale code; event seq is
  still continuity input, not mutation freshness authority.
- Missing authority is now a distinct stable code, not treated as stale and not repaired by silently
  using current state. Query/recovery do not require it.
- Test helpers may read current authority only in test code and must remain explicit at each mutation;
  production has no auto-authority convenience path.
- Broad 611/629 is a clean migration inventory: every failure reports
  `MissingBattleStateAuthority`; none reports a changed domain result. Keep the precondition and
  migrate tests, including the separate map-flow module.
- The next broad rerun reached 626/629. Its three remaining failures are the same bounded
  missing-authority test migration class (all direct deploy calls), not new runtime behavior
  failures; production auto-authority remains forbidden.
- The correction reached 628/629 and exposed one additional conditional deploy in the same live RON
  defense flow. This is not a new policy or runtime defect; all other Core library tests pass under
  mandatory authority.
- Mandatory authority is now fully adopted by the Core test suite: 629/629. The explicit test helper
  is only transport setup; production still rejects absent and stale authority before mutation.
- Payload eligibility is domain-owned rather than a Cartesian action enumeration: finite employee,
  deployed-unit, skill, and four-speed options are projected; employee×tile×facing and skill×target
  remain selected Core preview/preflight requests.
- `manual_activation_allowed` now means an executable skill payload exists at this revision. A
  target-required skill with no valid target is disabled with stable `no_valid_target`; Unity must
  not keep the button enabled to begin an impossible local targeting flow.
- Current Pause, Resume, or speed is a disabled payload (`already_active`) and execution returns the
  same typed reason without revision/event/time mutation.
- Server test compilation deliberately exposed all explicit checkpoint constructors missing the new
  playback eligibility. They were migrated directly; no defaulting/alias path was introduced.
- The actual Rust fixture generator had also missed G06 `action_capabilities`; strict construction
  caught that stale fixture and it is now part of the same explicit current wire shape.
- Unity selected deployment now requires the Core preview response and its typed deploy eligibility
  before Confirm can send. Local affinity/tile geometry remains only for pre-highlight UX and is no
  longer the selected payload execution gate.
- Unity rejects a revisioned update when playback options, deployment candidates, withdraw entries,
  or skill readiness lack well-formed eligibility. Missing data does not become locally enabled.
- MCP structured validation is not sufficient evidence for overlapping anchor edits. A same-region
  edit briefly corrupted a class boundary and a whole-method replacement dropped `[Test]`; direct
  file reread, editor compilation, and nonzero test counts remain mandatory.
- Full EditMode 167/187 confirms strict parsing is exposing stale fixtures rather than silently
  coercing them. `missing_core_eligibility` remains an internal diagnostic and must not be localized
  or treated as a disabled gameplay reason; current mocks/actual Rust fixture must be migrated.
- Shared current builders plus regenerated typed Rust JSON reduced the inventory to 177/187. The
  remaining local affinity/occupied-drop tests protect the superseded Unity-authority behavior and
  must be replaced with Core selected-preview rejection tests rather than adapted around local gates.
- EditMode is now 187/187. Deploy candidate absence for an already deployed employee is valid set
  membership semantics, not a missing eligibility error; Unity shows it as deployed and does not
  attempt to localize the internal missing diagnostic.
- Actual Rust-generated JSON is validated by value equality before entering the Unity fixture, and
  Unity's revision/resync test consumes it successfully.
- PlayMode's first broad failure prevented cleanup and produced nine downstream scene/UI failures.
  Focused reproduction and rerun proved they were contamination, not nine G07 regressions. Final
  PlayMode is 51 passed / 2 explicit live-server skips / 0 failed.
- A movement presentation test legitimately requires a present empty live-deployment DTO; using
  `deployment: null` passed eligibility validation but violated that test's runtime reader. The fixture
  now mirrors production with complete empty arrays instead of weakening either consumer.
- All-targets are green: Core 670 nonzero tests and server 27 nonzero tests. Zero-test groups are
  explicitly excluded from evidence totals.
- Final legacy search finds no active static battle whitelist, local affordability/deployment
  execution validator, or free-form skill readiness reason. Historical audit/tmp occurrences remain
  evidence of the original defect, not competing canonical policy.
- `game_server/docs/unity_bridge_protocol.md` was the one material canonical mismatch. It now
  distinguishes root non-battle capabilities from revisioned active-battle authority and records
  exact stale-command ordering. This is a required G07 contract correction, not scope expansion.
- Final MCP consumer reread found one residual permissive boundary: Core's deployed-unit projection
  always produces `skill_readiness`, including a typed disabled payload when no manual skill exists,
  but the public Rust field remained optional and Unity accepted its absence. That permits a malformed
  update to hide the action locally instead of triggering resync. Make the field required in Rust and
  Unity strict validation; this narrows the wire contract to actual production behavior and removes a
  fallback rather than introducing new gameplay policy.
- The required-field cutover exposed exactly three stale Unity fixtures and no current production
  path. Migrating those fixtures to an explicit disabled skill readiness object restored 188/188
  EditMode. This confirms absence was test drift, not a legitimate no-skill wire variant.
- Live battle ticks make a client-rendered authority stale naturally. The accepted design is not to
  suppress that race or let the server execute against a newer revision: reject without mutation,
  deliver current resync, then let a new user/probe intent use a new request ID. The corrected smoke
  probe demonstrates this exact path against live Core/server state.
- External Unity canonical and workflow documents now distinguish non-battle root capability from
  revisioned battle capability/payload authority. Old local affordability and target-missing button
  semantics were removed rather than retained as migration advice.
- Post-format validation through the pinned Unity MCP instance remains strict-green: the six-case
  malformed-payload test passes 1/1, the Editor is idle with no compile/reload work, and Console has
  zero errors. Scoped Core/server/Unity diff checks pass and no relevant ignored battle-authority
  test remains. The next allowed action is the mandated completion review.
- Completion review closed DCA-H-11, DCA-H-12, DCA-M-12 and the G07 battle portions of PD-01/PD-02
  as Complete / High / no immediate correction. No unresolved policy question or G07-local refactor
  remains. Preserve this authority boundary for G12 and later domain goals; do not absorb G08 UX or
  G09/G10 targeting policy into the completed goal.
