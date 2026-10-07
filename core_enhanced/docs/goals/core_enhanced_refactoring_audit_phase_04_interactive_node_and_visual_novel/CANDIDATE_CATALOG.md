# Core Enhanced Refactoring Audit Phase 04 — Candidate Catalog

Catalog Date: 2026-08-07
Status: investigation complete; completion audit pending

Priority axes are ordered as: ownership duplication, correctness/atomicity, lifecycle complexity, change
amplification, test fragility, failure blast radius, change frequency. Dirty accumulated history makes change
frequency `unknown`; known subtotals are not automatic priority bands.

## Confirmed Candidates

### CRA-P04-C01 — Reject Runtime-Invalid VN Repeat/Source Pairs At Publication

- Work Type: `correctness_correction`
- Problem Evidence:
  - Typed trigger validation checks IDs and source-local values but not repeat/source compatibility.
  - Official live bundle validation adds exact RunStart policy, but does not reject
    `MapNodeEntered + OncePerBattle`.
  - Runtime `consumed_key_for_trigger()` rejects that pair as invalid static data. The node-enter transaction rolls
    back, but an invalid bundle was already published and matching nodes cannot be entered.
  - Existing trigger-data/live-RON tests cover empty IDs, graph/reference errors and RunStart policy, not this
    publication/runtime disagreement.
- Current Authority And Consumers: live/typed validator publishes the trigger; runtime repeat-key derivation owns
  the de facto accepted-combination table; node admission is the first failing consumer.
- Target Authority: typed trigger validation owns the same accepted/rejected table already enforced by runtime.
  Runtime keeps defensive checks. Do not forbid currently accepted pairs such as battle source plus `OncePerNode`
  without a separate policy decision.
- Preserved Invariants: current live catalog, trigger match/order, repeat key identity, checkpoint policy, consumed
  timing, error provenance, no fallback and no session activation for invalid data.
- Removal Or Consolidation Scope: remove the validation gap only; do not redesign repeat semantics or add sources.
- Priority Axes And Total: `1 / 2 / 1 / 1 / 2 / 2 / unknown`; known subtotal `9/18`.
- Validation Safety Net: first add typed and official-bundle negative tests for every currently runtime-rejected
  pair, including `MapNodeEntered + OncePerBattle`; assert source/object/field error context and no publication.
- Policy/External Contract Decision: none for mirroring existing runtime rejection. Any change to currently accepted
  pair meaning is excluded.
- Readiness: `test_first_then_implementation`.
- Disposition: `priority_candidate` because it corrects pre-publication validation ownership.

### CRA-P04-C02 — Replace Headquarters Partial Rollback With The Common World Transaction

- Work Type: `behavior_preserving_refactor`
- Problem Evidence:
  - `with_headquarters_transaction` manually snapshots/restores ten state fields while
    `with_state_transaction` already owns full non-battle rollback and monotonic compendium handling.
  - HQ completion temporarily `take()`s active content to bypass `plan_complete_current_node()`'s HQ rejection.
  - Recruit, emergency and open-shop branches must remember the private rollback list whenever a new mutable field
    is added, creating independent atomicity ownership and change amplification.
  - Happy-path queue/roster/shop/emergency tests exist, but no late completion failure asserts complete HQ rollback.
- Current Authority And Consumers: HQ domain owns branch staging; two transaction helpers independently own
  rollback coverage; common node completion owns map/progression/cleanup.
- Target Authority: the common non-battle transaction owns rollback for all three HQ branches. Node completion
  planning receives an explicit resolved-HQ context instead of hidden active-content removal.
- Preserved Invariants: candidate order, exact roster/queue ownership, UUID/currency changes, shop handoff, common
  node outcome, checkpoint/research behavior, error type and no partial mutation.
- Removal Or Consolidation Scope: remove `with_headquarters_transaction` and the `take`/restore bypass. Do not merge
  recruitment state into `ActiveNodeContent` or change HQ options.
- Priority Axes And Total: `2 / 2 / 2 / 2 / 2 / 2 / unknown`; known subtotal `12/18`.
- Validation Safety Net: add a late-failure characterization (for example invalid pending planning-return research)
  for recruit and emergency completion, compare complete pre/post Core state, then run current 18 node-session and
  map completion suites.
- Policy/External Contract Decision: none; only private transaction ownership changes.
- Readiness: `ready_after_characterization_test`.
- Disposition: `candidate`.

### CRA-P04-C03 — Centralize Event And Shop Persistent/Active Session Commits

- Work Type: `behavior_preserving_refactor`
- Problem Evidence:
  - `commit_event_session` and `commit_shop_session` update the run-persisted session and active mirror together.
  - Event map entry, Shop map entry and HQ-to-Shop transition repeat direct insert-plus-active assignment instead of
    using the same domain commit boundary.
  - Event helper fails without a run, while Shop helper silently updates only active state; this implicit divergence
    is not named at call sites.
  - Current checkpoint/re-entry tests prove the two roles are both required, but structure does not guarantee every
    production activation keeps them synchronized.
- Current Authority And Consumers: `RunState` owns persisted Event/Shop progress; active content owns current
  admission/projection; domain helpers and entry code share paired-write ownership.
- Target Authority: one private Event commit path and one private Shop commit path own paired persistence and active
  mirroring with explicit required-run versus active-only modes. All production entry/update transitions use them.
- Preserved Invariants: exact session clones, page/purchase/check history, checkpoint and retreat behavior, active
  result shape, no new serialization and current error ordering.
- Removal Or Consolidation Scope: consolidate paired writes only. Do not remove persistent caches, genericize every
  facility variant, or change admin/test fixture setup.
- Priority Axes And Total: `2 / 1 / 2 / 2 / 2 / 2 / unknown`; known subtotal `11/18`.
- Validation Safety Net: characterize each map/HQ entry and mutation path with equality between persisted and active
  session, plus checkpoint/re-entry and no-run helper semantics before changing signatures.
- Policy/External Contract Decision: none for private production paths. Public/admin fixture behavior remains
  unchanged.
- Readiness: `ready_after_characterization_test`.
- Disposition: `candidate`.

### CRA-P04-C04 — Centralize VN Session Assembly With Explicit Queue Placement

- Work Type: `behavior_preserving_refactor`
- Problem Evidence:
  - Trigger evaluation and Monolith exchange independently look up VN definition, translate presentation mode,
    allocate session UUID, copy entry scene, set blocking/source and initialize choice state.
  - Trigger activation appends behind the active queue, while Monolith deliberately inserts at queue front. The
    ordering difference is canonical but is encoded by separate manual mutations rather than an explicit policy.
  - Every direct test session constructor repeats the same fields, including the always-`None`
    `committed_choice_id`, increasing fixture coupling.
- Current Authority And Consumers: trigger and Monolith paths duplicate session assembly; top-level VN runtime owns
  activation/queue; canonical owns the distinct queue order.
- Target Authority: one private session factory resolves definition/entry/presentation/UUID and one enqueue method
  accepts an explicit `Tail` or `FrontOfQueue` placement. Trigger and Monolith supply source, blocking and placement.
- Preserved Invariants: UUID sequence, entry scene, presentation mode, trigger priority, active non-interruption,
  trigger tail ordering, Monolith queue-front ordering, blocking and serialized projection.
- Removal Or Consolidation Scope: remove duplicated construction/conversion and raw queue mutations. Do not merge
  VN definitions with triggers, alter public DTOs or change completion-hook identity in this candidate.
- Priority Axes And Total: `2 / 1 / 2 / 2 / 2 / 2 / unknown`; known subtotal `11/18`.
- Validation Safety Net: retain current trigger order/checkpoint tests and add a world Monolith exchange test that
  proves queue-front ordering, exact UUID progression, accept/decline state and no Shop action before blocking VN.
- Policy/External Contract Decision: none while factory/enqueue stay private and public session shape is preserved.
- Readiness: `ready_after_characterization_test`.
- Disposition: `candidate`.

## Rejected, Deferred And Unverified Signals

| Signal ID | Signal | Evidence and reasoning | Disposition |
| --- | --- | --- | --- |
| CRA-P04-S01 | Merge Event, Shop, Reward, SavePoint, Maintenance, HQ and VN into one generic session | They differ in persistence, identity, command payload, retry, completion and checkpoint semantics. Shared interactivity is not shared authority. | `rejected` |
| CRA-P04-S02 | Remove `RunState.event_sessions`/`shop_sessions` because active content duplicates them | Run caches survive checkpoint/re-entry/retreat; active content is cleared on checkpoint load and current-node cleanup. Both roles are tested and canonical. | `rejected` |
| CRA-P04-S03 | Delete `VisualNovelSession.committed_choice_id` | No `Some` writer exists and current projection always treats choices selectable, but the public serialized Rust type may have external consumers. | `unverified`; cleanup deferred pending API/consumer decision |
| CRA-P04-S04 | Use `committed_choice_id` to make VN choice retries idempotent | Canonical specifies exact identity rejection but no cached duplicate-response policy. This changes retry/choice projection semantics. | `policy_or_feature_migration`; deferred |
| CRA-P04-S05 | Remove unused `VisualNovelSource` variants | Workspace producers use only HQ/Map/Battle, while EventNode appears in tests and every variant is public DTO projection metadata. External/future consumers are unknown. | `unverified`; do not delete |
| CRA-P04-S06 | Forbid battle trigger `OncePerNode` because it stores a battle UUID in `Node` scope | Runtime intentionally accepts this pair; canonical does not define whether the naming is accidental. C01 must not invent a policy. | `deferred` policy decision |
| CRA-P04-S07 | Treat every Event/VN handler as defective because it is not wrapped by `with_state_transaction` | Current production data gates references; Event grants/battles and Monolith choice are clone-staged; focused tests prove reachable failure no-mutation. No live post-commit failure was established. | `rejected` as unsupported correction; retain focused characterization |
| CRA-P04-S08 | Identify Monolith completion hook by VN asset id forever | Current code does so and live data is consistent, but asset identity doubling as domain-purpose identity is fragile. A typed purpose may affect public serialized session/API or checkpoint design. | `deferred`; compatibility design required before candidate |
| CRA-P04-S09 | Remove or merge similarly named Event integration tests wholesale | Synthetic, live-RON, checkpoint, retreat, branch, DTO and evaluator tests protect different layers. Name/setup similarity is insufficient under CRA-14. | `rejected`; one exact redundant unit case is separately classified in `TEST_AUDIT.md` |

## Cross-Boundary Handoff

- Phase 5 may trust node admission/common completion and Event-started-combat persistence, but owns combat setup
  internals independently.
- Phase 10 owns DTO architecture; Phase 4 only proves current projections consume active session authority.
- Final should keep C01 separate from the Phase 1 loader candidates: C01 is a domain repeat/source contract gap,
  while Phase 1 C01/C03 address source inventory/publication mapping.
- C02 and C03 may share a future private world-session transaction implementation, but should not be merged until
  characterization proves one helper preserves both HQ completion and persisted-session semantics.
