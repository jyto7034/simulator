# Phase 08 Candidate Reaudit

## Current Dispositions

| Candidate | Current evidence and Boss impact | Disposition |
| --- | --- | --- |
| CRA-P08-C01 | `run_on_battle_start_hooks` still dispatches activations only; instant, projectile and death paths still assemble effect/activation occurrences separately | `still_valid` |
| CRA-P08-C02 | `ApplyModifier` mutates `UnitStats` directly, records source-less `StatChanged`, and never routes MaxHealth-induced HP loss through HP/death authority | `still_valid` |
| CRA-P08-C03 | AttackInterval percent arithmetic still narrows `u64` to `i64` and multiplies unchecked; `percent_amount_ceil` still narrows an out-of-range quotient; draft scale assigns uncapped values | `still_valid` |
| CRA-P08-C04 | equipment validation still checks local weapon/event/recipe metadata but does not resolve triggered `ApplyBuff` or activation skill IDs or enforce an executable occurrence matrix | `still_valid` |
| CRA-P08-C05 | instant basic damage uses a helper while projectile impact and general `ApplyDamage` retain overlapping committed-hit assembly; ignored `BattleCommand::UnitDied` and broad carriers remain | `still_valid` |
| CRA-P08-C06 | public `battle::death::{DeadUnit, DeathProcessResult, DeathHandler}` and its lone partial-model test remain disconnected from production `core::death_runtime` | `still_valid` |

## Candidate Evidence And Readiness

### C01 — one typed occurrence dispatcher is still absent

Current live `parasite_seed` authors `OnBattleStart` +6% Attack for `AllAllies`, but battle start collects only
`ability_activations`. The live battle-start integration exercises `orchestra_baton`'s activation and does not equip
or assert the parasite effect. OnAttack/OnHit assembly is divided between instant and projectile execution, while
death separately assembles OnDeath/OnKill/OnAllyDeath. Effects-before-activations exists on some paths but is not one
typed invariant. Work type remains `correctness_correction`, Tier A, `test_first_then_implementation`.

### C02 — stat mutation still bypasses HP/death lifecycle

`UnitStats::add_max_health` clamps current HP when max HP decreases. `process_commands` observes only before/after
stats, updates movement speed and logs `StatChanged { source_instance_id: None }`. It does not emit `HpChanged`, call
death finalization, preserve modifier provenance or assign kill credit. An active zero-HP unit is therefore still
possible. Work type remains `correctness_correction`; tests can be added now, while lethal modifier attribution
remains an explicit policy gate before implementation.

### C03 — declared numeric ranges are not preserved at extremes

The AttackInterval percent branch still uses `self.attack_interval_ms as i64` and ordinary multiplication.
`percent_amount_ceil` uses a sufficiently wide product for two `u32`s but casts a quotient greater than `u32::MAX`
back with `as u32`. Draft MaxHealth/Attack scaling assigns that value directly and does not reapply the UnitStats
MaxHealth cap. Current ordinary-value tests do not exercise these branches. Work type remains policy-free
`correctness_correction`, ready after boundary tests.

### C04 — publication validation still omits equipment effect edges

`EquipmentDatabase::validate_contract` checks weapon profiles, event metadata and recipe references. Global reference
validation joins skills to buffs and other catalogs but never walks equipment triggered effects or activation
bindings. Runtime missing references can therefore remain late no-ops. Reference validation is independently ready;
the OnHit Counterpart BonusDamage compatibility row stays gated by target-meaning policy and C01's dispatcher matrix.

### C05/C06 — duplicate and false authorities remain visible

`calculate_basic_attack_damage_snapshot` centralizes part of instant calculation, yet projectile impact reconstructs
live target context, trigger effects/activations, formula, resonance and command ordering itself. General
`ApplyDamage` builds another context. `BattleCommand::UnitDied` is still produced by formula result and ignored by the
Core executor. Consolidation remains a `behavior_preserving_refactor` after C01/C02 characterization and public Rust
carrier inventory.

The top-level DeathHandler still removes units and handles only Modifier subsets, unlike production death, which
owns retained runtime units, graveyard, buff/movement/field-work cleanup, event lineage and complete occurrences.
Its cleanup remains gated by transferring sorted/dedupe assertions and external public Rust consumer inventory.

## Boss Leaf-System Reconciliation

Boss runtime owns when authored work is canceled or ordinary route combat resumes. It invalidates unreleased attack
epochs/casts and deliberately leaves released projectile, damage, effect and death work to existing leaf authorities.
`boss_runtime_allows_combat_action` only arbitrates entry into ordinary combat; it does not implement an alternate
trigger, stat, formula or death path. Thus C01/C05 remain independent as previously classified, the other four remain
unaffected current candidates, and no distinct `CRR-P08-*` root cause was found.
