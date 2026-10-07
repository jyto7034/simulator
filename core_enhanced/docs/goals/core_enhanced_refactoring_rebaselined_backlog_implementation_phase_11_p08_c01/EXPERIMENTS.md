# Phase 11 CRA-P08-C01 Experiments

Status: complete
Implementation Complete: true

## 2026-08-10 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user's Master execution request remains active and authorizes the single active Child.
- Phase 10 CRA-P08-C03 has a current `Complete / High` audit with improvement class `none`.
- Phase 11 is next in the exact registry; Phases 12~44 remain pending and unauthorized.
- CRA-POL-002 is approved and bounds this Phase; CRA-POL-006 freezes unrelated public/data/save/wire shape.

### Evidence Carried Into Initialization

- CRA-P08-C01 remains a Tier-A correctness correction with `3/3/3/3/3/2/unknown = 17/18`.
- Historical live defect: battle start collects activations only, so `parasite_seed` OnBattleStart AllAllies Attack
  +6% is published but never runs.
- Occurrence assembly remains split across battle start, instant/projectile basic hits and death paths even though
  sorted item lookup, some effects-before-activations behavior and proc state already exist.
- Required proof: live parasite two-ally exact events/final stats; same-occurrence ordering; complete attack/hit/death
  class matrix; dead-caster rules; CRA-POL-002 one-shot non-recursive retaliation.
- This Phase supplies the typed occurrence seam required by P08-C04/C05/C06 and must not absorb their publication,
  committed-damage consolidation or cleanup scope.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Master Phase 11 coordination changes from `ready` to `active`; no other Child is active.
- Phase 11 production status/source/data investigation: not started.
- Test executions: none.
- Production/test/data/canonical mutation before initialization: none.

### Next Experiment

Read all three Child documents completely, capture expanded dirty status and prospective target hashes, then trace
all six live occurrence paths, proc state, targeting, command order and current retaliation behavior before baselines.

## 2026-08-10 — E01 Dirty Baseline, Live Call Graph And Pre-Mutation Tests

### Dirty Boundary

- HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded post-initialization status: 422 entries; normalized SHA-256
  `3110fee7867552d896ca59d5e4533142dc77506f466080b1b0f02ea6c1c62b9f`.
- Prospective production fingerprints captured before content inspection:
  - clean `triggers.rs`: `f3dff22a15072ed8c0eb0a895c23e8644eb5359a0195b71a96d24e5b58d53081`
  - modified `commands.rs`: `5a05eadbadcbcdb70b7d941c94aabe8eaca9727bb9ac4699a12650f5e56317a7`
  - modified `death_runtime.rs`: `b26cafa91d9a5c25713cfc75c0c5b1f8362e1db2b9d795004f4f2830c0bc46fa`
  - modified `sim.rs`: `780361c78d42170dac289f841e58da7afc5fe2c419e1c3a6bdfcc499a90abbbb`
  - clean `basic_attack_projectile.rs`:
    `5d3e8dbe9ab2be9c7de5283af21552d63fe7518198357a387daf8c070cc26332`
  - clean `damage_runtime.rs`: `f4405c734ccec49f99c4291b922d5b9115b2a474451582768c9c633eed61a231`
  - modified `types.rs`: `172a64c42c856e271d4567b136a4ab0501469827bfdfeb288bd08c47b2fa8247`
  - modified live test: `3df4ac516204fcfabd593bf7efe7647063c6b869070a2928c564cdbcf3540ba3`
  - modified live equipment RON:
    `322d7455a256082f03804c0dcc7a77a4caa3a06919d5e34b667042b411bcd1a6`
  - modified canonical: `6e20d6a6c0609e6217b3a00d98371b7e13af12ef7d649f86855238c74c1ec00d`
- Scoped diffs show the existing `death_runtime.rs` changes are field-work settlement/reconciliation, while
  `commands.rs`, `sim.rs` and `types.rs` contain prior tactical/control/attack-epoch work. Those hunks are user-owned
  and must remain untouched except for separable trigger call sites.

### Current Six-Occurrence Call Graph

- `triggers.rs` sorts owned runtime items by instance UUID but traverses them twice through parallel
  `collect_all_triggers` and `collect_all_trigger_activations` APIs. `TriggerSource` has only the Item variant.
- `run_on_battle_start_hooks` creates one deterministic occurrence ID per sorted unit and dispatches activations
  only. It never calls `trigger_commands_from_effects`.
- Instant basic attack obtains current-hit OnAttack/OnHit effects in `damage_runtime.rs`, converts ordinary effects
  there, then separately collects activations in `commands.rs` after damage application.
- Projectile impact repeats the same effect/activation assembly in `basic_attack_projectile.rs`.
- `death_trigger_commands` independently emits OnDeath, OnKill and sorted OnAllyDeath effects followed by
  activations; only OnDeath activation commands opt into dead-caster delivery.
- `target_usefulness.rs` is a read-only preview consumer of current-hit effects, not a runtime occurrence dispatcher.
- `BattleCommand::ApplyDamage` intentionally does not collect item OnAttack/OnHit occurrences for ordinary
  skill/buff/area damage; this Phase preserves that boundary.

### CRA-POL-002 Reproduction

- `calculate_damage` currently consumes every OnHit BonusDamage effect without inspecting its authored target.
  Therefore live `thorned_lantern_guard` CounterpartUnit bonus increases the defender's incoming hit instead of
  damaging the attacker.
- The existing `ApplyDamage` leaf already supplies non-recursive damage delivery because it calculates with an empty
  OnHit slice and only the supplied snapshot's explicit OnAttack effects. The occurrence plan can hand retaliation
  to this typed leaf without consolidating committed-hit authority.

### Pre-Mutation Nonzero Baselines

- Core trigger collector: 1/1 passed.
- Core item activation/proc: 3/3 passed.
- Core OnAttack counterpart target: 1/1 passed.
- Core production death cleanup: 1/1 passed.
- `live_item_skill_activation`: 3/3 passed.
- `ron_loading`: 25/25 passed.
- Full Core test inventory at baseline: 958 tests, zero benchmarks.

### Result And Next Experiment

The candidate remains `still_valid`. Add the live parasite two-ally red proof and focused typed dispatcher tests,
then implement one sorted item occurrence plan and route all six runtime occurrence classes through it.

## 2026-08-10 — E02 Test-First Live Omission And Retaliation Reds

### Added Proofs

- Extended the existing live-item integration fixture with the unchanged published `parasite_seed` definition and a
  second active ally. The assertion requires exactly two time-zero Attack +6% `StatChanged` events in sorted runtime
  unit order, exact 10→10 and 50→53 arithmetic, and no enemy application.
- Added a Core basic-hit case where attacker and defender both own the same OnHit CounterpartUnit flat-8
  BonusDamage item. The assertion requires defender HP 90, attacker HP 92, owner-attributed event order and exactly
  two damage events, proving no return reflection.

### Exact Red Results

- Live parasite filter: 1 selected, failed with `left: []`; no battle-start StatChanged event exists.
- Retaliation filter: 1 selected, failed with defender HP `82` versus expected `90`; current calculation folded the
  flat 8 into the defender's incoming basic hit and never delivered a separate retaliation.
- One discarded invocation used an exact short name and selected zero tests; it is explicitly not evidence. The
  immediately repeated non-exact unique filter selected the intended one test and produced the red above.

### Next Experiment

Introduce the typed occurrence context/plan and one-pass sorted payload collection, then route battle start and both
basic-hit deliveries through it before extending the death/ordering matrix.

## 2026-08-10 — E03 Typed Dispatcher, Retaliation And Green Validation

### Implementation

- Replaced parallel item effect/activation traversals with `collect_item_trigger_payload`, which sorts runtime item
  UUIDs once and collects both payload classes in that order.
- Added private typed `TriggerOccurrenceContext` and `TriggerOccurrencePlan`. The plan preserves explicit current-hit
  effects, ordinary effect commands and activation commands as distinct typed phases; `into_commands` guarantees
  ordinary effects before activations for a single occurrence.
- Routed sorted-unit OnBattleStart, instant OnAttack/OnHit, projectile OnAttack/OnHit, OnDeath, OnKill and sorted
  OnAllyDeath through the dispatcher. Removed the activation-only battle-start path, all runtime calls to the old
  activation collector and the single-variant `TriggerSource` wrapper.
- Kept source OnAttack damage inputs in their released snapshot, while the plan owns live ordinary effects and
  activations. This preserves projectile/source-death behavior without moving damage or delivery authority.
- Classified OnHit CounterpartUnit BonusDamage out of the incoming current-hit slice. The existing BonusDamage
  component arithmetic now exposes a crate-private snapshot helper; each authored retaliation becomes one
  owner-attributed `ApplyDamage` command with its authored damage type.
- Retaliation uses `include_on_attack_damage_effects = false`, and ordinary `ApplyDamage` has no OnHit slice, so it
  cannot recursively collect item OnAttack/OnHit. Dead/missing attackers produce no command and no amplification.
- Updated read-only basic-attack preview to use the same current-hit classification. Ordinary skill/buff/area damage
  remains outside item OnAttack/OnHit collection.
- Updated `game_rulebook.md` with the six-trigger order, dead-caster rule, live parasite behavior and approved
  non-recursive retaliation meaning. No RON, public Rust, save/checkpoint or wire carrier changed.

### Focused Proof

- Original red live parasite case is green: 1/1, exact two allies in sorted unit order and no enemy.
- Original red instant retaliation case is green and strengthened to two authored retaliations: defender receives
  only base 10; attacker receives ordered owner-attributed 8 then 5; no return reflection.
- Projectile parity retaliation: 1/1 green with exact 10 then 8 damage events.
- Dead/missing attacker decision table: 1/1 green, no current-hit input, no retaliation command.
- Typed OnAttack/OnHit/OnKill/OnDeath/OnAllyDeath context table: 5 cases in one test, exact target/proc identity and
  OnDeath-only dead-caster bit.
- Same-occurrence effect-before-proc-before-cast: Core OnAttack and battle-start live cases green.
- Production death path: OnDeath dead-caster cast, OnKill and two sorted OnAllyDeath effect/activation plans green.
- Complete commands module: 34/34 green at its recorded run; subsequent additions are included in full Core green.
- Sorted item payload collector: 1/1 green for both effects and activations.

### Broad And Release Proof

- Full Core lib: 964/964 green.
- Live item integration: 5/5 green.
- Live skill catalog: 4/4 green.
- Skill runtime contract: 13/13 green.
- Skill suite: 4/4 green.
- Live RON: 25/25 green.
- `game_server --all-targets`: 59/59 green; remaining four target binaries contain zero tests and pass.
- Release retaliation filters: 2/2 plus the separately named instant case 1/1 green.
- Release battle-start filter: 3/3 green.
- `cargo check -p game_core --all-targets` passed before final test expansion; every expanded target subsequently
  compiled under debug/release test builds.

### Static Result

- Old item `TriggerSource`, `collect_trigger_activations` and `collect_all_trigger_activations` references: zero.
- Direct effect/activation materializers are now called only by the typed dispatcher; the activation helper remains
  independently unit-tested as the proc identity leaf.
- `git diff --check`: clean.
- `cargo fmt --all -- --check` first reported only new-hunk formatting; formatter applied those mechanical changes.
- Final `cargo check -p game_core --all-targets`, `cargo fmt --all -- --check` and `git diff --check`: all clean.
- Final expanded status: 426 entries; normalized SHA-256
  `d0cbdad7950fa556dc1c0e647089bd61740852956b34bf5f4a392252ae3821b3`.
- Final scoped fingerprints:
  - `triggers.rs` `ae9d377dfbc988ad36b96ac44aba452b70a53175c28ff9b36d6948357a1e4ea8`
  - `damage.rs` `e92d03779bb460e370326559244cd95fd842da78050ec92493005169e5497076`
  - `damage_runtime.rs` `59d91880320c8e3163767e4b9a8184bad5296e505918d12dcec1f6397d058fac`
  - `basic_attack_projectile.rs`
    `cc7d0fac82902f873a96994e5a964261fd449e69f3fe1df13ade39197f885a9b`
  - `commands.rs` `33daf065dfd2956798d06912df4026e0227598768d778ff4e2ddb7990fa973d8`
  - `death_runtime.rs` `b4a96fee4b7b5ec52612a8cc8a43ffefa9686d9e3b3fba415a90c84d3864471b`
  - `sim.rs` `0877dc56e431de88f1bc78d51479a06e7209b1f127d566928ae4c988b6f42153`
  - `target_usefulness.rs`
    `88effbf9691c7a0ec40efa96335ec6d0f3a7bb9e5f9b7a6603493ea232562fdc`
  - live item test `0d85e737e5938722321bfe55a178c479be910239a7dd87ecc8f151d2710ec5a2`
  - canonical `e1657fb76bd76d6ab831c7e511330399329972a1d7c8daafc7f634a4e3168971`

### Next Experiment

Run final all-target check/format/diff/status fingerprints, reconcile every completion condition, then perform the
independent Phase 11 completion audit under the full review guide.

## 2026-08-10 — E04 Independent Exit Audit And Closure

### Audit Corrections

- The initial audit found that the live AllAllies proof did not directly execute AllEnemies. Added a deterministic
  two-enemy/excluded-ally target-order test. Its first compile attempt named a nonexistent command variant; the test
  was corrected to `ApplyModifier` and passed debug and release. Production code was unchanged.
- The restarted audit found that dead/missing attackers were covered but a retaliation released before its owner
  dies from the triggering hit lacked direct lifecycle proof. Added exact fatal-hit death/source/event-order proof;
  it passed debug and in the three-case release retaliation filter. Production code was unchanged.
- Each finding was corrected under the active implementation authorization, then the audit was reset. After the last
  correction all 567 lines of `goal_completion_review_guide.md` were reread before the final independent pass.

### Current Validation

- Commands 36/36; trigger payload 1/1; live item 5/5; live RON 25/25; live skill catalog 4/4; skill runtime 13/13;
  skill suite 4/4; complete Core lib 966/966; all have zero failures and zero ignored tests.
- `game_server --all-targets`: main target 59/59; four additional binaries ran zero tests and are not counted as
  behavior proof.
- Release: retaliation 3/3, exact AllEnemies 1/1, live battle-start 3/3; zero failures/ignored.
- Final `cargo check -p game_core --all-targets`, `cargo fmt --all -- --check` and `git diff --check` passed.
- Audit-artifact-inclusive status: 427 entries, normalized SHA-256
  `62d865f48b214a863964a090a515f0161404dbad90803362330d7e6b7d267ca7`.
- Final commands fingerprint after the two test-only audit corrections:
  `b21cc1419e51fa57daac0974039bab0fe6275b97cfb4fb769487fd1ac2779329`.

### Result

- `COMPLETION_AUDIT.md`: `Complete / High`, improvement class `none`, unresolved material finding none.
- Phase 11 is complete. Phase 12 may become ready only through the Master registry gate and after the completed audit
  is fully reread.
