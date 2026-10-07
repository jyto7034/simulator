# Core Enhanced Refactoring Audit Phase 08 — Candidate Catalog

Catalog Date: 2026-08-07
Status: complete; dispositions accepted by phase-exit audit `Complete / High`

Axes are ownership duplication, correctness/atomicity, lifecycle complexity, change amplification, test fragility,
failure blast radius and change frequency. Current dirty work makes frequency `unknown`; known subtotals do not
automatically decide disposition.

## Confirmed Candidates

### CRA-P08-C01 — Dispatch Every Trigger Occurrence Through One Typed Path

- Work Type: `correctness_correction`
- Problem Evidence: `run_on_battle_start_hooks` collects only ability activations. Live `parasite_seed` authors an
  OnBattleStart AllAllies Attack +6% effect that never runs. Basic instant/projectile and death paths each assemble
  effects and activations independently; the live test covers only the pendant activation.
- Current Authority And Consumers: item lookup is central, but four occurrence-specific paths decide which effect and
  activation classes run. Damage, stat/buff/heal commands, proc state and event logs consume their differing output.
- Target Authority: one occurrence dispatcher accepts typed trigger/counterpart/target/dead-caster context, collects
  sorted item effects and activations, and emits effects before activations as current damage/death paths do. Damage
  components remain explicit current-hit inputs; ordinary commands remain ordered output.
- Preserved Invariants: deterministic item/unit order and proc identity; Permanent stays battle-entry only; released
  effects survive control; OnAttack/OnHit timing, OnKill/OnDeath/OnAllyDeath targets and dead-caster ability support;
  no new target/delivery policy.
- Removal Or Consolidation Scope: remove battle-start activation-only assembly, duplicated basic-hit/death occurrence
  assembly, `TriggerSource`'s one-variant wrapper and unused dispatcher parameters where safe. Do not merge damage
  formula or ability delivery.
- Priority Axes And Total: `3 / 3 / 3 / 3 / 3 / 2 / unknown`; known subtotal `17/18`.
- Validation Safety Net: first add a live parasite-seed two-ally test with exact stat events/final stats and one
  activation+effect same-occurrence ordering case. Cover OnAttack, OnHit, OnKill, OnDeath and sorted OnAllyDeath
  effects plus activations, including inactive/dead caster rules.
- Policy/External Contract Decision: none for the live omitted modifier; effects-before-activations already matches
  existing damage/death behavior. The ambiguous OnHit bonus target signal remains outside this correction.
- Readiness: `test_first_then_implementation`.
- Disposition: `priority_candidate`.

### CRA-P08-C02 — Make Runtime Stat Mutation HP- and Death-Aware

- Work Type: `correctness_correction`
- Problem Evidence: negative MaxHealth mutation clamps `current_health` inside `UnitStats`, but `ApplyModifier` records
  only `StatChanged` and never calls HP logging/death. Active zero-HP units can result; source provenance is discarded.
- Current Authority And Consumers: `UnitStats` mutates both max/current values while damage runtime alone owns
  `HpChanged`, graveyard, buff cleanup, triggers and `UnitDied`. Movement also consumes a side update for speed.
- Target Authority: a single runtime stat-transition command computes before/after, updates dependent body state,
  emits `StatChanged`, routes any current-HP delta through the authoritative HP recorder and finalizes zero exactly
  once. Command provenance is retained for source/death credit.
- Preserved Invariants: modifier order and rounding, positive MaxHealth not automatically healing, current HP never
  exceeds max, movement speed updates immediately, one stat event, one HP event only when HP changes and one death.
- Removal Or Consolidation Scope: remove direct `target.stats.apply_modifier` mutation from command execution and
  source-less effect conversion. Persistent prebattle stat composition remains independent and event-free.
- Priority Axes And Total: `2 / 3 / 3 / 2 / 3 / 3 / unknown`; known subtotal `16/18`.
- Validation Safety Net: negative flat/percent MaxHealth cases that lower but do not kill and that reach zero; assert
  exact StatChanged/HpChanged order, source, graveyard, buff cleanup, trigger count and no duplicate death. Retain
  ordinary attack/move modifier skill tests.
- Policy/External Contract Decision: decide whether the originating trigger/caster receives kill credit when a
  max-health modifier kills. This affects event/result semantics and must not be guessed in implementation.
- Readiness: `test_first_then_kill_credit_decision`.
- Disposition: `priority_candidate`.

### CRA-P08-C03 — Use Saturating, Range-Preserving Stat Percentage Arithmetic

- Work Type: `correctness_correction`
- Problem Evidence: AttackIntervalMs percent conversion casts `u64` to `i64` and multiplies unchecked;
  `percent_amount_ceil` narrows an unbounded quotient with `as u32`; draft scaling can wrap Attack/MaxHealth and
  bypass the UnitStats max-health cap. Current tests cover large ordinary percent modifiers but not these branches.
- Current Authority And Consumers: `UnitStats`, employee percentage helper and stat pipeline each implement a portion
  of percent arithmetic; battle build consumes their output for scheduling, damage and HP.
- Target Authority: typed saturating helpers calculate signed/unsigned percent deltas in `i128/u128`, apply the
  destination stat's declared range once and preserve explicit ceil/truncation rules.
- Preserved Invariants: live numbers and modifier order are byte-for-byte equivalent; AttackIntervalMs stays at
  least 1; MaxHealth respects its cap and current HP; defense sign and scale ceil semantics remain unchanged.
- Removal Or Consolidation Scope: replace unchecked casts/multiplication and duplicated clamp snippets only. Do not
  change balance percentages, stat order or certified-power policy.
- Priority Axes And Total: `1 / 2 / 1 / 2 / 2 / 2 / unknown`; known subtotal `10/18`.
- Validation Safety Net: boundary table at 0, 1, i64/i32/u32 maxima and extreme signed percent; direct draft scale
  tests at the published 207% stage and exact live-value golden comparisons.
- Policy/External Contract Decision: none; saturation restores declared numeric domains.
- Readiness: `test_first_then_implementation`.
- Disposition: `candidate`.

### CRA-P08-C04 — Close Equipment Effect And Activation Publication Validation

- Work Type: `correctness_correction`
- Problem Evidence: equipment validation does not resolve ApplyBuff IDs or activation skill IDs and does not reject
  effect kinds that their trigger occurrence drops. Runtime missing references and unsupported combinations silently
  no-op. A two-ID integration assertion is not bundle validation.
- Current Authority And Consumers: equipment, buff and skill databases index independently; runtime trigger dispatch
  attempts late lookup. `GameDataBase` cross-catalog validation does not join these edges.
- Target Authority: publication validation resolves every effect/activation reference and applies one explicit
  trigger/effect/target compatibility matrix shared with C01's dispatcher. Error paths identify equipment and field.
- Preserved Invariants: current live bundle loads unchanged; valid Permanent, battle-start, attack/hit and death-family
  behavior stays exact; strict unknown-field policy and no runtime fallback remain.
- Removal Or Consolidation Scope: remove late silent missing-reference behavior and ad hoc integration ID lists. Keep
  runtime defensive lookup for integrity, but publication is the authoring authority.
- Priority Axes And Total: `2 / 2 / 1 / 3 / 3 / 2 / unknown`; known subtotal `13/18`.
- Validation Safety Net: missing buff/skill, invalid Permanent effect, unsupported damage effect occurrence and valid
  live bundle tests with exact object/field/cause. Retain weapon/recipe/event metadata tests.
- Policy/External Contract Decision: settle whether OnHit Counterpart BonusDamage is retaliation or a targetless
  current-hit modifier before freezing that part of the matrix. Reference validation can proceed independently.
- Readiness: `references_ready_matrix_after_C01_and_policy`.
- Disposition: `candidate`.

### CRA-P08-C05 — Consolidate Committed Damage Resolution And Remove Ghost Result Authority

- Work Type: `behavior_preserving_refactor`
- Problem Evidence: instant basic resolution calls a helper while projectile impact repeats OnAttack/OnHit effects,
  activations, formula, resonance and command ordering; general ApplyDamage repeats context/application assembly.
  `BattleCommand::UnitDied` is calculated then ignored, and multiple request/context fields have no semantic reader.
- Current Authority And Consumers: `calculate_damage` owns arithmetic, but three surrounding executors own overlapping
  lifecycle. Core logs/results consume the applied outcome; public Rust tests construct the broad carriers.
- Target Authority: one typed committed-hit plan materializes source/target snapshot and explicit trigger policy,
  runs the formula, applies/logs HP/death/resonance once and returns an execution summary. Preview remains pure.
- Preserved Invariants: launch source and live target defenses, per-target/hit roll identity, basic-only OnAttack/
  OnHit semantics, effect-before-activation order, actual-delta resonance, same-time death and exact event causes.
- Removal Or Consolidation Scope: remove the duplicate projectile block, context construction and no-op UnitDied
  command; prune unread request/context/result fields after workspace/external inventory. Do not conflate targeting,
  projectile collision or skill-area lifecycle.
- Priority Axes And Total: `3 / 1 / 3 / 3 / 2 / 2 / unknown`; known subtotal `14/18`.
- Validation Safety Net: run identical instant/projectile hits with stat drift, source death, target defense drift,
  bonus/modifier effects, crit identity, kill triggers and logs; preserve pure preview tests separately.
- Policy/External Contract Decision: public Rust carrier removals require consumer inventory. Phase 9 must confirm
  delivery integration ownership without redefining the damage plan.
- Readiness: `after_C01_C02_characterization_and_external_inventory`.
- Disposition: `priority_refactor_candidate`.

### CRA-P08-C06 — Delete The Disconnected Public Death Handler

- Work Type: `cleanup`
- Problem Evidence: `battle/death.rs::{DeadUnit,DeathProcessResult,DeathHandler}` has no workspace consumer beyond its
  one test. It queues removal and supports only Modifier subsets, unlike production Core death which retains units,
  owns lifecycle/graveyard/buffs/logs and dispatches all supported effect/activation paths.
- Current Authority And Consumers: the file is publicly exported but disconnected; `core/death_runtime.rs` is the
  sole workspace production authority.
- Target Authority: `finalize_unit_death` and C01's occurrence dispatcher remain the only death implementation.
- Preserved Invariants: one runtime death, deterministic trigger order, dead-caster activation, graveyard, movement/
  buff/field-work cleanup, world incapacitation and public event/result behavior.
- Removal Or Consolidation Scope: delete the module/export and its false-authority unit test. Do not delete Phase 6
  event-log validation or production death tests.
- Priority Axes And Total: `3 / 1 / 2 / 1 / 2 / 1 / unknown`; known subtotal `10/18`.
- Validation Safety Net: transfer the sole useful dedupe/sorted-ally assertion to production death occurrence tests,
  add effect/activation chaining, then compile-search and run death/world families.
- Policy/External Contract Decision: the public Rust module requires external-consumer inventory or explicit breaking
  cleanup approval. It is not a Unity/server DTO.
- Readiness: `after_C01_and_external_consumer_inventory`.
- Disposition: `cleanup_candidate`.

## Required Ordering

1. C01 starts with the failing live battle-start effect test.
2. C02 starts with max-HP lifecycle tests and needs a kill-credit decision before implementation.
3. C03 is independent after boundary tests.
4. C04 reference validation can start independently; compatibility-matrix enforcement follows C01 and the OnHit
   target decision.
5. C05 follows C01/C02 characterization so consolidation preserves corrected occurrence/HP semantics.
6. C06 follows C01 assertion transfer and public-consumer review.

## Rejected, Deferred And Unverified Signals

| Signal | Disposition | Evidence/reason |
| --- | --- | --- |
| Entry stat order | `retain` | explicit two-stage boundary and composition tests; no second builder |
| Delete producer-less GrowthStack | `rejected_duplicate` | Phase 3 already rejected; serialized/external policy unknown |
| Damage formula | `retain_authority` | one pure implementation with typed breakdown and strong unit tests |
| Buff identity/cadence/stale expiry | `retain_and_strengthen` | coherent battle-time map; add reapply/cause characterization |
| Control derivation | `retain` | metadata capability and one central current-state query match canonical |
| Proc roll/cooldown/count | `retain` | stable identity separate from log sequence; limits are explicit state |
| Resonance | `retain` | one clamped runtime state and tested Hard CC/gain-lock behavior |
| Death/world incapacitation ledger | `retain` | production lifecycle and per-instance staged handoff are distinct and coherent |
| OnHit Counterpart BonusDamage meaning | `policy_or_feature_migration` | live authoring, target docs and current-hit description conflict |
| Mixed-type fixed/mitigated feedback | `unverified_policy_signal` | breakdown is typed but event tag is primary-type aggregate; no live True bonus |
| Skill OnHit applicability | `deferred_policy_signal` | current OnHit dispatch is basic-attack-only; canonical does not settle expansion |
| Event-log death validator | `deferred_phase_06` | already dispositioned as disconnected validation harness; do not duplicate |
| Target selection/projectile/area geometry | `deferred_phase_09` | only effect carriers were inspected |
| DTO/server/Unity consumers | `deferred_phase_10_or_unverified` | public Rust cleanup is gated; Unity not read |
