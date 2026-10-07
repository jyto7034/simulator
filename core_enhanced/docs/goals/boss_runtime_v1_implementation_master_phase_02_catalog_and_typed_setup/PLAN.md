# Boss Runtime v1 — Phase 02 Catalog And Typed Setup Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — only this child scope
Master Goal: `../boss_runtime_v1_implementation_master/PLAN.md`
Master Phase: P02
Activated: 2026-08-09
Completed: 2026-08-09
Simulator Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

Boss Runtime의 authoring/publication/setup foundation을 구현한다. 별도 Boss catalog의 Pattern, actor behavior,
encounter definition을 strict하게 parse·compile하고, optional PvE encounter/actor binding을 battle start 전에
immutable typed setup으로 resolve한다. Boss binding이 없는 현재 encounter는 기존 setup/scenario 의미를 그대로
유지한다.

## Direct dependency and handoff

- P01: `../boss_runtime_v1_implementation_master_phase_01_battle_lifecycle_correctness/COMPLETION_AUDIT.md`
  - Verdict: `Complete / High`
  - Handoff: `ready_for_p02`
- Coimplemented refactors:
  - CRA-P01-C01: embedded source inventory authority
  - CRA-P05-C04: one typed combat-setup plan
- Target policies: BP-01~BP-29의 definition/setup 부분, BP-32~BP-34의 static definition invariants

Current dirty files are the user-owned baseline. Investigation, failing characterization, implementation and
validation evidence must be written after each bounded unit rather than reconstructed at Phase end.

## In scope

- One separate logical Boss catalog with Pattern, actor behavior and encounter definition collections
- Stable typed IDs and strict `deny_unknown_fields` authoring
- Only the closed selector/control/state/task/condition/query/result/disposition/parameter vocabulary required by the
  approved v1 reference slice; known but unimplemented future variants must reject rather than fallback
- Immutable compilation of references, typed parameter frames, unique priorities, outcome/result exhaustiveness,
  capability compatibility and finite/zero-time graph properties
- Boss physical source in the one startup-only atomic live bundle and a mechanically derived exact source inventory
- Optional `PveEncounter` Boss encounter reference plus typed actor binding
- One private resolved combat setup plan that enters `BattleScenario`/spawn data before Core construction
- Exact no-binding parity and no tick-time raw Boss catalog/argument lookup
- Data/runtime canonical updates describing only the implemented foundation

## Out of scope

- Boss encounter/actor runtime state, scheduler, activation epochs or microstep execution
- Target selection execution, path-cost query, movement intent, route cursor or Task leaf adapters
- Official live Elite/NormalBoss/FinalBoss encounter opt-in or balance/content identity
- All-manifested-difficulty and full live setup feasibility gates owned by P05
- Public Boss DTO/event/checkpoint, game_server mapping or Unity work
- Dynamic filesystem loading, hot reload, generation migration or last-known-good publisher
- Generic script VM, deep override/inheritance, arbitrary expressions or placeholder future runtime variants
- Deferred/independent CRA candidates

## Milestone P02-M1 — current characterization

1. Inventory the actual embedded source set, loader candidate path, validation ownership and setup/scenario mutation path.
2. Add an exact-set inventory characterization and no-binding setup/scenario parity fixtures before production change.
3. Fix all source/test owners and pre-change hashes in `EXPERIMENTS.md`.
4. Refine the minimum supported schema vocabulary from BP-01~BP-29 and the P04 `TrackAndRejoin` fixture; do not invent
   official content or future mechanics.

## Milestone P02-M2 — strict Boss definitions and compiler

1. Add strict raw Boss catalog schemas with separate Pattern, actor behavior and encounter collections.
2. Add typed stable IDs, parameter signatures/bindings and immutable resolved definitions/frames.
3. Reject duplicate/dangling IDs, unsupported variants, missing/extra/wrong typed arguments, invalid targets,
   non-unique node priorities, incomplete Pattern outcome/Task result handling and incompatible capabilities.
4. Reject unbounded cycles and zero-time cycles; accept only statically finite/bounded Pattern graphs and valid resident
   Phase control graphs.
5. Prove valid definitions compile independently with no cross-call parameter leakage.

## Milestone P02-M3 — atomic bundle and inventory authority

1. Add the Boss source through `GameplayRonSourceSet`, `ParsedGameplayRon`, `GameDataBaseParts` and final
   `Arc<GameDataBase>` candidate construction.
2. Replace duplicated/manual source count reconciliation with one mechanically derived inventory authority.
3. Prove exact normalized source set and missing/extra source detection.
4. Prove any Boss parse/compile/reference error rejects the whole startup candidate; do not add hot reload/LKG semantics.
5. Keep the official live Boss catalog empty of encounter opt-in content in this Phase.

## Milestone P02-M4 — typed combat setup plan

1. Add optional strict `PveEncounter` Boss runtime reference and typed actor bindings.
2. Introduce one private resolved setup aggregate after encounter/preview/domain authorities resolve their values.
3. Resolve Boss encounter/actor definitions and immutable argument frames before scenario/Core construction.
4. Carry typed bindings into scenario/spawns with deterministic actor identity and no raw catalog lookup after start.
5. Characterize and preserve normal, Boss-class-without-binding, survival, authored tactical plan and Event battle setup.
6. Inject invalid binding/late setup failures and prove no world/session partial mutation.

## Preserved invariants

- No-binding encounter preview, reward, group/event order, route cells, objective and spawn identity are unchanged.
- Ordinary route enemy behavior remains the active runtime authority until an actor is explicitly bound.
- Existing startup loader remains embedded, versioned, fail-closed and all-or-nothing.
- Abnormality/enemy data continues to own identity/stats/attack/skill; PvE owns battlefield/objective/wave/reward.
- Boss graph/runtime details remain internal and are not added to public Wire/checkpoint/save.
- Same seed and authored input preserve deterministic UUID/group/unit ordering.
- No missing definition, unsupported variant or bad binding becomes a no-op/default/fallback.

## DTO, data, save and deployment

- DTO/Wire migration: none. P02 types are Core-internal setup/runtime inputs.
- Data migration: one new strict Boss source joins the existing embedded bundle schema generation. There is no dual
  reader, alias or partial catalog publication.
- Live content: the production Boss collections may be empty but structurally valid; no official encounter opts in.
- Save/checkpoint migration: none; active battle remains session-memory state and P02 adds no runtime activation state.
- Deployment: rebuild/restart publishes one new atomic embedded data generation. Existing in-memory battles are not
  converted and no hot reload path is introduced.

## Focused validation

- Exact embedded source inventory and path reconciliation tests
- Boss raw parse and compiler success/failure matrix
- Duplicate/ref/variant/priority/outcome/result/capability/loop/parameter rejection tests
- Atomic live bundle failure tests and strict live loading
- No-binding preview→resolved-plan→scenario parity tests
- Bound fixture resolution, deterministic actor mapping and no-runtime-lookup tests
- Late setup failure no-world-mutation tests

Every exact command and actual pass/fail/ignored/filtered count is recorded. A zero-match filter is failure.

## Broad validation

After focused closure:

```text
cargo fmt --all -- --check
cargo test -p game_core --lib
cargo test -p game_core --test ron_loading
cargo test -p game_server
cargo test --workspace --all-targets
```

If current test topology shows a smaller authoritative setup module target, record and add it; do not replace the
required full Core/live-loading/workspace gates above.

## Completion conditions

1. Boss Pattern/actor/encounter collections are separate strict schemas and compile to immutable definitions.
2. Required IDs/references/variants/priorities/outcomes/results/capabilities/loops/parameters are fully validated.
3. Boss source participates in the one atomic embedded candidate and exact inventory comes from one authority.
4. Invalid Boss data rejects the entire candidate without fallback or partial publication.
5. Optional encounter/actor bindings resolve before battle start into one typed setup/scenario plan.
6. Runtime after Core construction needs no raw Boss catalog lookup or argument binding.
7. No-binding production setup is semantically identical across representative current battle kinds.
8. No official live Boss opt-in, runtime scheduler, public Wire or unrelated refactor enters the Phase.
9. Data/runtime canonicals and focused/broad tests pass with actual counts.
10. User-owned dirty changes are preserved and completion audit returns `Complete / High`.

## Stop conditions

- Minimum P04 vocabulary requires a user-visible Boss policy not covered by BP-01~BP-34.
- A serialized/public consumer boundary is required to carry P02 internal definitions.
- Atomic setup requires a compatibility layer, partial catalog publication or active-battle migration.
- Current user edits change inside an overlapping owner and cannot be reconciled at symbol/hunk scope.
- Correct exhaustive/finite validation cannot be defined without expanding into generic scripting or future content.

No stop condition became active. Independent completion audit: `Complete / High`, handoff `ready_for_p03`.
