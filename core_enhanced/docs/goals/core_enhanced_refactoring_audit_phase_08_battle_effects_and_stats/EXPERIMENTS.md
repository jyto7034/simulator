# Core Enhanced Refactoring Audit Phase 08 — Experiments

Status: complete

## 2026-08-07 — Activation gate

### Evidence and result

- Parent Master, Phase 0 boundary and Phase 7 completion audit were rechecked.
- Direct predecessor Phase 7 is `Complete / High`; the Phase 8 reserved path did not exist.
- Phase 3 persistent input, Phase 6 lifecycle, Phase 7 spatial/tactical and Phase 9~10 carrier boundaries were
  identified.
- No other active child exists. Phase 8 is investigation-only and production mutation remains prohibited.
- Exact start commit remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.

### Tests

No runner was executed during activation.

### Decision

Phase 8 may perform read-only production investigation and write only audit artifacts. Phase 9 remains pending.

## 2026-08-07 — Scoped production baseline

### Exact categories and initial hashes

| Category | Initial SHA-256 |
| --- | --- |
| top-level effect/stat runtime (5 files) | `525e1dd41a459fdc82a2c6798d10a1c8f120cd5ed9f17a01652061d70726cb14` |
| Core effect runtimes (4 files) | `374e2df02f2a0aef4f0edf05447e799bfee9f15ac934ee1c479811438b1fe64e` |
| shared battle carriers (8 files) | `8e8f972c981a27c7bef910a6e562627babca5151396ed7bccd9d329730f54bc4` |
| relevant validation carriers (4 files) | `c3e4fb48771d26f2fa3c7fa83a29a5918e1df1e0e4f0bb75cf1ea8c9ba95f596` |
| persistent/typed input carriers (10 files) | `082d3f024469d97343575bed0749eb49ad5e7b84bca55e3d2f09378ab37d3574` |
| world/result carriers (3 files) | `4e8e5492a2aa8a66721388ea961aef9c10a2f82a7e38a4d33eea8c8186f1ef91` |
| relevant live RON (7 files) | `226d90e2b207842468461cc4d4fb2aecff859c8406b9d115b52449b351876780` |
| skill-test module tree (5 files) | `cd51e3cb2a5b621f6d4bf6512ceb4dbf23e9cb4516f930b8d6cfb1eca9b0d713` |
| focused integration sources (3 files) | `06d0288b1fd559f92a049f2aabe0e2f807ddd9cf44411c22b1faec0f73e255e9` |
| focused world test source (1 file) | `87861813beff818f37e914cb3e25515e4b324508225711feb6c43fd03eae53ef` |
| nearest canonical (3 files) | `9fddfc637fe690798b159a41138625588ca4f003c7405bc787cd9eaea7569f38` |
| scoped Git status text | `c19804b54fe6c354a8fd5bacb21864e75d74612aa22ef9e01b05197287c68298` |

Top-level runtime is `battle/{buffs,cooldown,damage,death,stat_pipeline}.rs`; Core runtime is
`core/{damage_runtime,death_runtime,resonance_runtime,triggers}.rs`. Shared battle carriers are
`core/{build,commands,mod,sim,types}.rs` plus `battle/{event_log,result_stats,types}.rs`. Persistent/typed carriers
are `stats.rs`, `employee.rs`, `growth.rs`, `data/{mod,validation,abnormality_data,corroded_employee_data,
equipment_data,skill_fragment_data,skill_data}.rs`.

### Dirty-state result

Twenty-five scoped production/data/test/canonical paths were already modified, including buff/death runtime,
equipment/skill publication, live skill/policy data and world combat tests. These are user-owned baseline changes.
An initial read-only hash attempt named a nonexistent `data/buff_data.rs`; that category result was discarded and
recomputed with the actual `battle/buffs.rs` plus typed data module/validation carriers before this baseline was
recorded. At final replay, four later aggregate outputs were found associated with the wrong category labels/path
vectors. The originally written values `28c6...`, `4e8e...`, `226d...`, and `cf95...` were discarded and the table
above was corrected from the exact path lists described here. All constituent mtimes predate this audit, scoped Git
status stayed byte-identical, and no production write tool was used; this is an audit-record correction, not a file
content change.

### Tests

No runner was executed for the baseline.

## 2026-08-07 — Static effect/stat gaps before focused validation

### Evidence and result

- Battle-entry stats have one intended order: employee base → fragments → active consumable, then draft weapon
  profile → growth → equipment `Permanent` effects → enhancement → encounter scale → condition HP.
  Certified power deliberately excludes consumable and condition HP. No second production stat builder was found;
  the producer-less `GrowthStack` carrier is already a Phase 3 rejected deletion signal and is not duplicated here.
- `run_on_battle_start_hooks` collects only `ability_activations`. It never calls `collect_all_triggers` or
  `trigger_commands_from_effects` for `TriggerType::OnBattleStart`. Published equipment `parasite_seed` authors an
  `OnBattleStart` `Modifier(Attack +6%)` targeted to `AllAllies`, so that current live effect is accepted and indexed
  but never executes. Existing live integration covers `resonance_pendant`'s OnBattleStart ability activation only.
- Trigger dispatch is split by occurrence. Basic attack instant and projectile paths separately collect OnAttack/
  OnHit effects and activations; death runtime separately handles OnDeath/OnKill/OnAllyDeath; battle start handles
  activations only; `Permanent` uses the stat pipeline. `trigger_commands_from_effects` also carries two unused
  parameters. This split caused the live omission and creates competing orchestration even though effect lookup itself
  is centralized.
- Equipment publication validates weapon, recipe and event-check shape but does not validate triggered-effect buff
  references, activation skill references or trigger/effect/target compatibility. Unknown buff IDs therefore reach an
  `ApplyBuff` event and silently no-op; unknown activation skills reach `invoke_ability` and no-op. `BonusDamage` and
  `ModifyDamage` are consumed only in current-hit OnAttack/OnHit calculation and silently ignored by ordinary command
  conversion at other triggers. Current live references resolve, but the publication boundary does not enforce it.
- Runtime `BattleCommand::ApplyModifier` applies `UnitStats::apply_modifier` and records only `StatChanged`. A negative
  max-health modifier can clamp `current_health` to zero without `HpChanged`, lifecycle death, graveyard insertion or
  `UnitDied`, contradicting the canonical HP/death source. The command also discards effect-source provenance, so kill
  attribution for such a transition requires an explicit policy/contract choice.
- `UnitStats::apply_modifier` uses signed `i64` multiplication for `AttackIntervalMs` percent changes after casting
  `u64` to `i64`; large valid typed values can wrap or panic. `percent_amount_ceil` narrows an unbounded `u64`
  percentage result with `as u32`, and `apply_draft_stat_scale` can therefore wrap high attack/max-health values or
  exceed `UnitStats`' own max-health cap. Current published values are small, but typed live fields and public
  constructors do not establish the required arithmetic bound.
- `calculate_damage` is the one pure formula authority, but committed basic hits have two near-parallel orchestration
  paths: instant uses `calculate_basic_attack_damage_snapshot`, while projectile impact repeats OnAttack/OnHit effect,
  activation, damage, resonance and command sequencing. General `ApplyDamage` repeats context construction for skill,
  area and buff damage. `DamageRequest.time_ms`, multiple `DamageContext` fields and the calculated
  `BattleCommand::UnitDied` are not semantic inputs; the latter is processed as an explicit no-op because HP mutation
  already owns death.
- Public `battle/death.rs` is disconnected from `BattleCore`. Its `DeathHandler` and only test have no workspace
  consumer, ignore most `Effect` variants and model unit removal, while production death retains the runtime unit,
  writes the graveyard/log, clears buffs and dispatches effects plus ability activations in `core/death_runtime.rs`.
- Active buff identity, stale-expiry rejection, periodic cadence, control derivation, proc cooldown/count state,
  deterministic roll identity and resonance clamping use battle-time state coherently in the inspected paths. Death
  changes lifecycle before nested triggers, clears caster/target buffs deterministically and world reconciliation uses
  an instance idempotence ledger before the final staged roster commit.
- `TriggeredEffect.target` is ignored when `BonusDamage`/`ModifyDamage` is folded into the current hit. Published
  `thorned_lantern_guard` authors OnHit `BonusDamage` to `CounterpartUnit`, while the current formula adds it to damage
  received by the trigger owner. The generic target contract and the effect's "current attack" description do not
  settle whether this is retaliation or an intentionally targetless current-hit modifier, so Phase 8 records a policy
  signal rather than inventing a gameplay correction.

### Focused validation reason

Run the smallest current stat, damage, buff/control, trigger/cooldown/resonance, death, live-item, skill-composition
and world-incapacitation families to establish actual assertion meaning and counts. Green results cannot close the
missing live OnBattleStart effect, max-health/death gap, arithmetic boundary, publication validation gap, duplicated
hit orchestration or disconnected death authority recorded above.

## 2026-08-07 — Focused validation

| Command/filter group | Passed | Failed | Ignored | Meaning |
| --- | ---: | ---: | ---: | --- |
| damage formula | 22 | 0 | 0 | type/resistance/modifier/crit/breakdown/feedback arithmetic |
| buff metadata and live RON loading | 5 | 0 | 0 | ID, capability, stack bounds and current published buff shape |
| unit stats and battle-entry pipeline | 8 | 0 | 0 | ordinary modifier safety and current composition order |
| equipment publication validation | 6 | 0 | 0 | current weapon/recipe/event-check validation, not trigger references |
| trigger collection, target and proc limits | 5 | 0 | 0 | item order, OnAttack counterpart, activation/cooldown/count |
| control, resonance and runtime command families | 16 | 0 | 0 | overlap, cancellation, released effects, gain locks and ordinary mutation |
| death application and disconnected handler | 3 | 0 | 0 | production HP death/buff cleanup plus the obsolete model's sole test |
| damage snapshot, deterministic roll and log families | 9 | 0 | 0 | launch/impact state, identity and authoritative `HpChanged` metadata |
| live item and skill composition integrations | 20 | 0 | 0 | live activations and multi-step damage/heal/buff/stat event behavior |
| world attempt/incapacitation commits | 3 | 0 | 0 | instance idempotence and final condition commit |

Thirty-one valid invocations executed 97 passing tests with no failure or ignore. No scoped `#[ignore]` marker was
found. Green execution does not cover the live `parasite_seed` effect because the live battle-start test equips only
the activation-bearing pendant. It also does not cover trigger reference rejection, runtime max-health death,
extreme attack-interval/draft-scale arithmetic or a shared instant/projectile hit executor.

## 2026-08-07 — Candidate and CRA-14 disposition

- Confirmed C01 battle-start/occurrence dispatch, C02 HP-aware runtime stat mutation and C03 total stat arithmetic as
  correctness candidates.
- Confirmed C04 effect/activation publication validation, with target/effect compatibility policy explicitly gated.
- Confirmed C05 committed-damage lifecycle consolidation as a behavior-preserving refactor and C06 disconnected
  death-handler deletion as cleanup with a public Rust consumer gate.
- Retained formula, buff/control/proc/resonance and employee-attempt authorities; deferred OnHit bonus target meaning,
  mixed feedback, skill OnHit scope and Phase 9/10 carriers without inventing policy.
- The sole test removal candidate is the disconnected handler test after its useful ordering/dedupe assertion moves to
  production death coverage.

## 2026-08-07 — Final no-mutation proof

The corrected exact initial categories and final recomputation are identical:

| Category | Initial SHA-256 | Final SHA-256 | Result |
| --- | --- | --- | --- |
| top-level effect/stat runtime | `525e1dd41a459fdc82a2c6798d10a1c8f120cd5ed9f17a01652061d70726cb14` | `525e1dd41a459fdc82a2c6798d10a1c8f120cd5ed9f17a01652061d70726cb14` | identical |
| Core effect runtimes | `374e2df02f2a0aef4f0edf05447e799bfee9f15ac934ee1c479811438b1fe64e` | `374e2df02f2a0aef4f0edf05447e799bfee9f15ac934ee1c479811438b1fe64e` | identical |
| shared battle carriers | `8e8f972c981a27c7bef910a6e562627babca5151396ed7bccd9d329730f54bc4` | `8e8f972c981a27c7bef910a6e562627babca5151396ed7bccd9d329730f54bc4` | identical |
| relevant validation carriers | `c3e4fb48771d26f2fa3c7fa83a29a5918e1df1e0e4f0bb75cf1ea8c9ba95f596` | `c3e4fb48771d26f2fa3c7fa83a29a5918e1df1e0e4f0bb75cf1ea8c9ba95f596` | identical |
| persistent/typed input carriers | `082d3f024469d97343575bed0749eb49ad5e7b84bca55e3d2f09378ab37d3574` | `082d3f024469d97343575bed0749eb49ad5e7b84bca55e3d2f09378ab37d3574` | identical |
| world/result carriers | `4e8e5492a2aa8a66721388ea961aef9c10a2f82a7e38a4d33eea8c8186f1ef91` | `4e8e5492a2aa8a66721388ea961aef9c10a2f82a7e38a4d33eea8c8186f1ef91` | identical |
| relevant live RON | `226d90e2b207842468461cc4d4fb2aecff859c8406b9d115b52449b351876780` | `226d90e2b207842468461cc4d4fb2aecff859c8406b9d115b52449b351876780` | identical |
| skill-test module tree | `cd51e3cb2a5b621f6d4bf6512ceb4dbf23e9cb4516f930b8d6cfb1eca9b0d713` | `cd51e3cb2a5b621f6d4bf6512ceb4dbf23e9cb4516f930b8d6cfb1eca9b0d713` | identical |
| focused integration sources | `06d0288b1fd559f92a049f2aabe0e2f807ddd9cf44411c22b1faec0f73e255e9` | `06d0288b1fd559f92a049f2aabe0e2f807ddd9cf44411c22b1faec0f73e255e9` | identical |
| focused world test source | `87861813beff818f37e914cb3e25515e4b324508225711feb6c43fd03eae53ef` | `87861813beff818f37e914cb3e25515e4b324508225711feb6c43fd03eae53ef` | identical |
| nearest canonical | `9fddfc637fe690798b159a41138625588ca4f003c7405bc787cd9eaea7569f38` | `9fddfc637fe690798b159a41138625588ca4f003c7405bc787cd9eaea7569f38` | identical |
| scoped Git status text | `c19804b54fe6c354a8fd5bacb21864e75d74612aa22ef9e01b05197287c68298` | `c19804b54fe6c354a8fd5bacb21864e75d74612aa22ef9e01b05197287c68298` | identical |

Only Master/Phase audit artifacts changed. Cargo wrote ignored target/debug artifacts; no production/data/test/
canonical/server/Unity file was edited, formatted, moved or deleted.
