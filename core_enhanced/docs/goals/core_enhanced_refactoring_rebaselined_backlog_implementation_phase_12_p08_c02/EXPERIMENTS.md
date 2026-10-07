# Phase 12 CRA-P08-C02 Experiments

Status: complete
Implementation Complete: true

## 2026-08-10 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user's Master execution request remains active and authorizes the single active Child.
- Phase 11 CRA-P08-C01 has a current `Complete / High` audit with improvement class `none`; its 207-line audit was
  fully reread before initialization.
- Phase 10 CRA-P08-C03 froze arithmetic/cap behavior required by this transition.
- Phase 12 is next in the exact registry; Phases 13~44 remain pending and unauthorized.
- CRA-POL-001 is approved and bounds lethal attribution/order; CRA-POL-006 freezes unrelated public/data/save/wire
  shape.

### Evidence Carried Into Initialization

- CRA-P08-C02 remains Tier A correctness correction with `2/3/3/2/3/3/unknown = 16/18`.
- Historical defect: runtime negative MaxHealth mutation can clamp current HP without HpChanged/death, leave an
  Active zero-HP unit and discard causal source.
- Approved behavior: StatChanged→conditional HpChanged→normal death; causal existing unit is killer, unsourced cause
  has no killer; complete transition and cleanup exactly once.
- Required proof: flat/percent, nonlethal/lethal, sourced/unsourced decision table; exact source/order/state/cleanup;
  nested death no-duplicate; ordinary modifier/body/prebattle preservation.
- This Phase supplies the HP/death/provenance seam required by P08-C05 and must not absorb committed-damage
  consolidation, publication validation or disconnected DeathHandler cleanup.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Master Phase 12 coordination changes from `ready` to `active`; no other Child is active.
- Phase 12 production status/source/data investigation: not started.
- Test executions: none.
- Production/test/data/canonical mutation before initialization: none.

### Next Experiment

Read all three Child documents completely, capture expanded dirty status and prospective target hashes, then trace
all runtime modifier producers, public/private carriers, HP/death recorders and causal kill-credit consumers before
running nonzero baselines or adding tests.

## 2026-08-10 — E01 Dirty Baseline, Runtime Graph And Pre-Mutation Tests

### Dirty Boundary

- HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded post-initialization status: 430 entries; normalized SHA-256
  `6c8177ac19209522647271b30cf254171778145395ec115c10d39a500c293416`.
- Prospective fingerprints before Phase 12 content mutation include:
  - `commands.rs` `b21cc1419e51fa57daac0974039bab0fe6275b97cfb4fb769487fd1ac2779329`
  - `sim.rs` `0877dc56e431de88f1bc78d51479a06e7209b1f127d566928ae4c988b6f42153`
  - `death_runtime.rs` `b4a96fee4b7b5ec52612a8cc8a43ffefa9686d9e3b3fba415a90c84d3864471b`
  - `damage_runtime.rs` `59d91880320c8e3163767e4b9a8184bad5296e505918d12dcec1f6397d058fac`
  - `triggers.rs` `ae9d377dfbc988ad36b96ac44aba452b70a53175c28ff9b36d6948357a1e4ea8`
  - `stats.rs` `9c4d2c5645a5a701c041eae8ba14edbdfca232182eff0eca1da152c9b535fc1f`
  - `event_log.rs` `397e2380a21c79dc3addf10177c8ab0a7d85c25f80eba808da9894b09ef36e02`
  - `behavior.rs` `65349de8daf5230cd00041b9f3da4b66e76239245e608b98a2623312b2276139`
  - `core/mod.rs` `c6255583aa6af6bc1c4fec782f31b0199ec336f30831a5b87e91adeff00e2002`
  - live item test `0d85e737e5938722321bfe55a178c479be910239a7dd87ecc8f151d2710ec5a2`
  - live RON `322d7455a256082f03804c0dcc7a77a4caa3a06919d5e34b667042b411bcd1a6`
  - canonical `e1657fb76bd76d6ab831c7e511330399329972a1d7c8daafc7f634a4e3168971`
- All prospective tracked files except the clean skill projectile runtime are already modified by user/campaign work.
  Existing Phase 11 trigger, field-work, tactical/control, event/DTO, test and data hunks remain user-owned.

### Current Producer And Authority Graph

- Workspace-wide `BattleCommand::ApplyModifier` inventory has two production Core producers: item triggered effects
  in `trigger_commands_from_effects` and skill `ModifyStats` in `build_skill_step_commands`. The former knows the
  trigger unit and the latter knows the caster, but both flatten to the source-less public command variant.
- Direct `process_commands` calls and the disconnected top-level `battle::death` model construct the same public
  variant without a causal source. The latter remains P08-C06 scope and has no production BattleCore consumer.
- `process_commands` applies `UnitStats::apply_modifier`, updates body speed and records
  `StatChanged { source_instance_id: None }`; it neither emits HpChanged nor finalizes zero HP.
- `UnitStats::add_max_health` owns saturating arithmetic and clamps current HP to the new maximum, including zero.
  Persistent `stat_pipeline.rs` calls UnitStats directly and remains event-free.
- `damage_runtime.rs` owns metadata-rich HpChanged and hands zero HP to
  `death_runtime.rs::finalize_unit_death`, which owns lifecycle/action/movement, buff cleanup, graveyard, UnitDied and
  Phase 11 OnDeath/OnKill/OnAllyDeath dispatch.
- Battle log and behavior DTO already contain optional `source_instance_id` on both StatChanged and HpChanged, so no
  serialized carrier change is required.
- `BattleCommand` is a public Rust enum. Preserve its existing source-less variant for compatibility; carry the known
  item/skill modifier source through a private Core execution batch/context and use direct public commands as the
  unsourced CRA-POL-001 case. No hidden side table or public field/variant change is needed.

### Pre-Mutation Nonzero Baselines

- Ordinary stat/resonance command: 1/1 passed.
- Runtime movement-speed modifier/body update: 1/1 passed.
- UnitStats arithmetic family: 4/4 passed.
- Production death buff cleanup: 1/1 passed.
- Phase 11 death occurrence path: 1/1 passed.
- Live item integration: 5/5 passed.
- Full live RON: 25/25 passed.
- Immediate predecessor current full Core inventory: 966/966 green; Phase 12 will rerun it after mutation.

### Result And Next Experiment

The candidate remains `still_valid`. Add compile-current decision tables through direct unsourced commands and actual
skill/item sourced paths, prove missing HpChanged/death/provenance red behavior, then implement the private sourced
execution context plus one HP/death-aware runtime stat transition.

## 2026-08-10 — E02 Test-First HP/Death/Provenance Reds

### Added Proofs

- Added one eight-case decision table crossing flat/percent, sourced/unsourced and nonlethal/lethal MaxHealth
  reductions. Sourced cases execute an actual skill cast; unsourced cases execute the unchanged public
  `BattleCommand::ApplyModifier`. Assertions cover exact stats, source, StatChanged→HpChanged→UnitDied order,
  metadata, killer, lifecycle, graveyard, summary and no active zero-HP state.
- Added a sourced lethal integration with target/caster buffs, OnKill stat effect, OnDeath dead-caster heal and OnKill
  living-caster heal. It requires cleanup, one target HpChanged/death, causal killer/credit, no resurrection or
  duplicate target mutation, and the living source's nested heal.
- Strengthened the live production `parasite_seed` test to require its equipment owner as StatChanged causal source.

### Exact Red Results

- Decision table: 1 selected, failed on case 0 because expected one HpChanged but found zero.
- Sourced lethal cleanup: 1 selected, failed because the clamped target lifecycle remained Active.
- Live parasite provenance: 1 selected, failed with no matching events because runtime still emitted
  `source_instance_id: None`.
- All three compiled against the pre-production-change tree and selected nonzero tests.

### Next Experiment

Keep the public command source-less, add one Core-private execution provenance context, route item and skill modifier
producers through it, and hand actual current-HP deltas to the existing HP/death authority after StatChanged.

## 2026-08-10 — E03 Private Provenance Context, HP-Aware Transition And Green Validation

### Implementation

- Added Core-private `RuntimeCommandBatch`, carrying only optional modifier source plus existing
  `Vec<BattleCommand>`. Existing direct `process_commands` remains the unsourced adapter, so the public command enum,
  constructors and matches are unchanged.
- `process_commands_with_modifier_source` now performs one runtime stat transition: active check, exact UnitStats
  arithmetic, body-speed update, source-aware StatChanged, then conditional HP/death handoff. Lethal transitions also
  contribute one killed-target execution summary.
- Added `record_runtime_stat_hp_change_and_finalize` under `damage_runtime.rs`, the existing HP owner. It emits one
  metadata-empty `HpChanged(reason: Command)` for an actual clamp and calls existing death finalization at zero.
- Phase 11 item occurrence plans now keep effect batches sourced to the trigger owner while activation commands stay
  unsourced until their invoked skill supplies its caster. Battle start, instant/projectile basic hits and death
  occurrence ordering remain effect-before-activation and preserve per-occurrence source.
- Skill instant, projectile and area execution pass the existing caster as modifier source without changing skill
  command/data/event carriers.
- Persistent stat pipeline and UnitStats arithmetic were not changed. `BattleCommand`, `BattleLogEvent`, behavior DTO,
  save/wire and RON shapes were not changed.
- Updated `game_rulebook.md` with sourced/unsourced runtime modifier semantics, no-clamp behavior,
  StatChanged→HpChanged→death order, normal killer credit and event-free prebattle separation.

### Focused And Broad Proof

- Exact red tests are green: decision table 1/1 (eight cases), nested cleanup 1/1 and live parasite source 1/1.
- Added no-clamp preservation: positive then nonlethal-negative MaxHealth changes retain current HP, emit two
  StatChanged and zero HpChanged; 1/1 green.
- Complete commands module: 39/39 green.
- Ordinary stat/resonance 1/1; movement/body 1/1; death buff cleanup 1/1; UnitStats 4/4; Phase 11 trigger payload 1/1;
  live item 5/5.
- Skill runtime 13/13, skill suite 4/4, live skill catalog 4/4, stat pipeline 6/6, result stats 7/7 and live RON 25/25.
- Complete Core lib: 969/969 green.
- `game_server --all-targets`: main target 59/59; four additional binaries ran zero tests and passed.
- Release MaxHealth modifier filter: 3/3 green. Release live parasite provenance: 1/1 green.
- `cargo check -p game_core --all-targets`, `cargo fmt --all -- --check` and `git diff --check`: passed.

### Static And Dirty Result

- The only production ApplyModifier producers remain trigger effects and skill ModifyStats; all trigger runtime paths
  use sourced batches and all four skill execution callers (instant, spatial area path, projectile, persistent area)
  pass the caster source. Direct public command and disconnected P08-C06 model remain unsourced by design.
- New structs are `pub(in crate::game::battle::core)` and non-serde. `damage.rs`, `event_log.rs`, `behavior.rs`,
  `stats.rs` and live RON retain their E01 hashes; no public/serialized field or variant changed.
- Expanded status: 431 entries; normalized SHA-256
  `529e4ecea370161c5543f23017ee66625945eec81e4b6e753cef74774189eb3b`.
- Final relevant fingerprints:
  - `commands.rs` `11f9b17c992faa7b445764f474c214538b3041e3e66c74e4660551f360551c42`
  - `damage_runtime.rs` `4253a1a2ca532567315d8897e878f59f64bcb5906524031c0d59222d08954072`
  - `triggers.rs` `84928567ad0e9fb438896cb40b49592c77a7d07b7d23e84b5c163048e77ac01a`
  - `death_runtime.rs` `096a68265a37d6af46cf7c3eaaae10bdb4a632c4323ee17f4b0062f2384fc6d9`
  - `sim.rs` `050dc74b736d97796909ab1468b003018d5b07a269d651be880cbe53b239bde5`
  - `basic_attack_projectile.rs`
    `32abc30493cde913a6899df35050b3fa4bb3cb3c18dae819a2fc3477ae5e3ef8`
  - skill projectile `6de51ab750f571e8c9fecb921d770bb9a39cd31b98e9fe2e57dd149795f463a3`
  - skill area `2a22998063d482cc37eeeb620f61c6fac20a63ff1e0c5085fad094858b6978ee`
  - live item test `fae73da9c9fe30b3b4f919828100d6b1342a20677a4fe68396970c877d03d5a0`
  - canonical `6c53ecd5d0588aace1a2b043b31434d004327d60938ecf61066ac19acfd31951`

### Next Experiment

Reconcile every completion condition, reread the full completion-review guide, independently audit current runtime,
tests, public/data compatibility, dirty preservation and canonical, then close only on `Complete / High`.

## 2026-08-10 — E04 Phase-Exit Completion Audit

### Independent Review

- The full 567-line completion-review guide was read in bounded sections; an earlier truncated oversized read was
  discarded. Current Child/policy documents, producer/consumer paths, HP/death authority, tests, public/data
  compatibility, dirty evidence and canonical were independently rechecked.
- Audit-current reruns passed: commands 39/39, production live item 5/5 and release MaxHealth 3/3. Current
  `cargo check -p game_core --all-targets`, formatter check and `git diff --check` also passed.
- E03 broad evidence remains current under unchanged production/data fingerprints: complete Core 969/969, server
  main 59/59, live RON 25/25 and affected skill/stat/result suites all green.
- Audit artifact: `COMPLETION_AUDIT.md`, 226 lines, fully reread after correcting two record-only formatting/command
  namespace typos.

### Verdict And Boundary

- Primary verdict: `Complete`.
- Long-term fit: `High`.
- Improvement class: `none`.
- Unresolved material finding, immediate correction and policy question: none.
- Child changes to `complete` with `Implementation Complete: true`. Phase 13 P08-C04 may become ready; no Phase 13
  production work is authorized until its own initialization gates complete.
