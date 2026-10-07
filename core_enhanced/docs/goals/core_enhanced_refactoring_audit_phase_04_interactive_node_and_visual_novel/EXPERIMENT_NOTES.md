# Core Enhanced Refactoring Audit Phase 04 — Experiment Notes

Status: complete

## Boundary decision

- Authority family: Event and Facility-family node session admission/selection/action/outcome/completion plus
  VisualNovel trigger/session start/advance/choice/finish/cleanup. Facility-family means SavePoint, Maintenance,
  Shop, Reward and HeadquartersContact lifecycle; Phase 3 retains their economy mutation internals.
- Direct predecessor: Phase 3 `Complete / High`.
- Phase 2 run/map/checkpoint is a carrier, not reopened authority.
- Phase 3 persistent asset transaction staging is a handoff, not proof of session atomicity.
- Combat preview/setup, active battle and command/DTO projection retain Phases 5~10 ownership.

## Source-of-truth distinctions

- Authored event/facility/VN data authorizes definitions and references; live session state owns current progress.
- Run/node state can own admission/progression while a typed session owns in-session lifecycle; exact split is
  discovery pending and will be traced from mutation code.
- Behavior/snapshot/result values are derived consumers unless actual mutation proves otherwise.
- Checkpoint carriage does not prove session self-consistency, replay behavior or cleanup correctness.
- Test builders and historical Goal decisions do not override current runtime/live data/canonical.

## Dirty worktree precautions

- Repository baseline is start commit `f318414fb0156a67e04c5479000c2a2683fc0c09` plus the current dirty tree.
- `world/visual_novel_flow.rs`, facility tests, canonical, live data and shared world/behavior paths already include
  user changes or may overlap this Phase's read-only evidence.
- Only Parent/Phase audit artifacts are audit-owned.
- Production content hashes and scoped status will be captured before investigation and compared at Phase exit.

## External systems and consumers

- `game_resources` may be read only for live data actually published into this boundary.
- `game_server` may be read only for a direct Core result/session/snapshot mapping needed for consumer
  classification; server refactor discovery is excluded.
- Unity runtime/UI/tests and external Unity canonical are explicitly out-of-scope/unverified under the Master.
- Other public Rust consumers remain unknown unless the current workspace/canonical proves them.

## Initial rejected assumptions

- Do not merge Event, Facility and VN sessions merely because all are interactive nodes.
- Do not call a DTO or selected-event snapshot the session authority.
- Do not infer idempotence or stale-action safety from a happy-path completion test.
- Do not classify cloning/staging as waste before failure/no-mutation and final commit ownership are traced.
- Do not treat a test-only invalid session as production-reachable without admission/data evidence.
- Do not delete or merge tests because names/setups look similar; CRA-14 requires unique-invariant and replacement
  safety-net evidence.
- Do not choose retry, cancellation, trigger priority, reward timing or narrative policy during this audit.

## Test-code audit policy for this Phase

- Each assigned family records the production path it exercises, unique invariant, action/assertion meaning,
  fixture boundary, overlaps and one of `retain`, `strengthen`, `merge`, `remove`, `unverified`.
- Simple success/happy-path tests are retained when they uniquely protect session assembly, authoritative state
  transition, result shape or cleanup.
- `remove` requires proof of no unique contract plus an equal or stronger surviving safety net.
- Runner green counts are not quality evidence; no runner is scheduled until a specific static evidence gap is
  recorded.

## Activation checklist

1. Required Master and direct dependency documents read completely — complete.
2. Phase 4 path/dependency/active-child/dirty gate — complete.
3. Initialize required Phase documents and Parent active registry — complete.
4. Freeze scoped production-content/status baseline — complete; six aggregate hashes recorded in `EXPERIMENTS.md`.
5. Inventory authority/data/canonical/test surfaces and complete candidate/disposition artifacts — complete.
6. Perform independent P4-01~P4-11 completion audit — complete (`Complete / High`).

## Open questions to resolve from evidence

- Which type owns active Event and Facility session identity, and whether run/node state duplicates it.
- Whether all action commands bind to the current session/node/generation and reject stale/replayed inputs.
- Where reward/cost/psychology and node progression become atomic with session completion.
- Which component owns VN trigger priority/deduplication and how completion clears or advances state.
- Whether checkpoint restore validates interactive-session references/lifecycle or merely carries them.
- Which test families uniquely protect rejection/no-mutation/cleanup rather than fixture construction.

## Resolved authority findings

- `RunState.event_sessions` and `RunState.shop_sessions` are checkpoint/re-entry authorities. The corresponding
  `ActiveNodeContent` value is the currently admitted command/projection mirror; removing either role would change
  retry/retreat/checkpoint behavior.
- `NodeSession` owns selected node identity/category/payload and common completion routing. It does not own
  Event/Shop progress or VN state.
- Event exact duplicate identity is deliberately idempotent; other stale/replayed identities fail before commit.
  One node-seeded d20 and committed grant/combat state survive checkpoint/retreat as canonical requires.
- Facility variants remain heterogeneous typed lifecycles. Their common ownership is node admission/completion,
  not a generic mutable session schema.
- `GameCoreState.visual_novel` is the sole active/queue authority; run checkpoint owns only the rewind-scoped
  trigger history. Active VN preservation across checkpoint is intentional canonical behavior.
- VN trigger priority, repeat history and consumed timing are Core-owned. Unity/external projection is a consumer
  and remains Phase 10/out-of-scope here.

## Confirmed static gaps before runners

- HQ duplicates the common transaction contract with a manually enumerated partial rollback frame and a temporary
  active-content planner bypass.
- Event/Shop persistent+active paired writes and VN session construction/enqueue are repeated across entry paths.
- Typed/live validation does not reject `MapNodeEntered + OncePerBattle`, although runtime rejects the same authored
  combination.
- No focused test currently injects a late HQ completion failure or drives Monolith exchange VN queue-front plus
  accept/decline through world commands.
- `committed_choice_id` has no `Some` writer, but its public serialized Rust compatibility surface is unverified.

## Phase handoff condition

Phase 5 may consume only a proven run/node lifecycle seam. It remains inactive until this Phase classifies every
assigned surface/test family, produces a complete candidate/disposition catalog, proves no production mutation
and receives `Complete / High`.

## Phase 4 completion handoff

- Verdict: `Complete / High`; production mutation and initial dirty-state drift were not observed.
- Four confirmed candidates: C01 trigger publication correction, C02 HQ transaction consolidation, C03
  Event/Shop paired-session commit consolidation, C04 VN session factory/queue-policy consolidation.
- Event exact duplicate/no-regrant, checkpoint one-roll, retreat resume, heterogeneous facility lifecycle and
  top-level VN/checkpoint ownership are retained contracts rather than refactor targets.
- Phase 5 may consume common node completion and Event combat handoff, but must independently audit preview/setup,
  manifested difficulty, deployment and battle-entry authority.
