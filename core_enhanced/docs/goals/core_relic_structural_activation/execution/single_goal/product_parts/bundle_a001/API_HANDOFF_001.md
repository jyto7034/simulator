# U09 bundle A actual product API handoff

This is an internal connection of GS-CRSA-003 / GOAL-CRSA-01. Library compilation passed, not normal structural publication/runtime or Goal completion. Source/test/fixture execution ownership remains Root-coordinated; Sol wrote no test, comparator, expected value or cfg(test) registration.

## Actual boundaries

- `game::relic::structural::authoring`: strict serde declarations, bounded recursive condition decoder, finite primitive union. `RelicDefinition.rules` and `RelicCorruptionVariant.rules` default to an empty vector and omit the empty field on serialization, preserving old canonical serialization. A structural-only definition explicitly supplies `effects: []` and nonempty `rules`; both empty is invalid. Corrupt variant penalties remain mandatory and negative under the existing contract.
- Public Rust read-only function `structural::explain_effective_plan(generation: &ValidatedGameData, employee_uuid: Uuid, inventory: &RelicInventory, providers: &[ProspectiveProvider]) -> Result<Option<dto::RelicEffectivePlanDto>, RelicPlanError>` actually calls the private finite compiler and owned DTO exporter. A legacy-only loadout returns `None`; this is not a substitute for an absent structural plan or a publication success. The query does not discover active actors/providers from World or admit an action.
- Private `EffectivePlanInput` / `compile_effective_plan` / `EmployeeEffectivePlan::explain` support an actual prospective loadout. The owner retains the strong immutable `ValidatedGameData` generation, selected copies/providers, finite candidate set, closure state, typed composition and fingerprint. No owner constructor/Deserialize or mutable execution capability is exported in the DTO.
- DTO `schema_version=1`, `scope="prospective_loadout"`, `execution_status="runtime_binding_pending"`. It exposes generation/fingerprint/providers/rule states and exact provider-index + Original/Generated effect identities, final literal properties or runtime-expression programs, recipes, amount explanation, authored `source_creation` and compile usage. Original and Generated identities are distinct; Generated includes relic copy, rule, original anchor, local action and local effect. Rule targets and effects sort by provider index and exact key.
- Existing World actor/query and CLI descriptors still use the U08 numeric / R11 literal surfaces. The new Rust capability table is reachable through `structural::capabilities::structural_capabilities()`, but each `runtime_connected` is false. No actor-effective CLI command or normal structural public query result is claimed.
- Final raw `GameDataBase::new` calls `validate_publication` after ordinary value/resource/reference compilation. During A it refuses nonempty structural rules, new `PersonalShieldGrant`, and `ChannelAura.basic_scope=RegularOnly` with typed compile diagnostics. Legacy empty rules and default `LegacyBasic` aura pass this new gate. Ordered runtime dispatch also refuses the unfinished PersonalShieldGrant branch. This temporary refusal must be replaced only when the full producer/co-equip proof and query/readiness/release/runtime owners/consumers are connected. Do not remove it merely to make raw tests green.

## Exact declarations for the independent test author

`RelicRuleDef { id: String, selector: StructuralSelectorDef (default), when: RelicConditionDef (default Always), apply: RelicRuleActionDef }`.

`StructuralSelectorDef { provider: ProviderPredicateDef (default), effect: EffectPredicateDef (default) }`; provider has optional `equipment_id`, `fragment_id`, `form_id` (form requires fragment). Effect predicate has AND `all_tags`, optional `op`, optional `has_property`. `SameEffect` checks one current candidate; `Exists` checks one complete selector witness, so fields are not assembled from unrelated siblings.

`RelicConditionDef`: `Always`, `SameEffect(EffectPredicateDef)`, `Exists(StructuralSelectorDef)`, `All(Vec<_>)`, `Any(Vec<_>)`, `Not(Box<_>)`. Decoder applies the existing expression parse depth/node limits before another recursive condition child. `RelicRuleActionDef`: `Grant(GrantBodyDef)`, `Edit { property, operation }`, `Recipe(StructuralRecipeDef)`.

`GrantBodyDef`: `InlineEffects { action_id, effects }` or `FiniteFlow { anchor_action_id, steps, flow: Arc<SkillFlowDef> }`. Local step has id/delay_ms/SkillTarget/delivery Arc<DeliveryDef>/effects. Granted effect has id/tags/op. Closed primitive union: Damage, DamageExpr, Heal, HealExpr, HealthCost, PersonalShieldGrant. Expression/Flow/TileRange owned ASTs use Arc in new authored leaves for safe shallow source cloning. No nested rule, subscriber, template, arbitrary string path or reactive grant is decoded. Existing real Flow compiler proves the authored finite body; its existing Compare remains action-scoped.

`NumericEditDef`: `Add(TypedRelicValue)`, `Scale { increment_percent: i64 }`, `Replace(TypedRelicValue)`. Typed values: `Hp(i64)`, `PercentPoints(i64)`, `SpeedPoints(i64)`, `DataUnits(i64)`. `Scale` is an additive percentage increment, not an absolute factor. The program canonicalizes equal Replace, rejects different Replace, sums Add/increments with checked i128 arithmetic, and applies `(replace_or_base + add) * (100 + increments) / 100` once with signed truncation. Negative factors/out-of-range/overflow are errors. Unknown runtime bases retain a program rather than an invented evaluated number. `NumericProgram::include` is crate-private for product/child module use; public `apply` is pure arithmetic without authority.

Actual property descriptors:

| Operation/property | Unit and consumer range | Numeric |
| --- | --- | --- |
| Damage / DamageAmount | HP 0..=u32::MAX, existing DamageExpr narrowing | yes |
| Damage / DamageFootprint | actual TileRangePattern, recipe only | no |
| PersonalShieldGrant / PersonalShieldInitialAmount | HP 1..=u32::MAX, positive existing Shield capacity | yes |
| PersonalShieldGrant or ChannelAura / SupportAreaRadius | data units 1..=u32::MAX | yes |
| ChannelAura / RegularBasicDamagePercent | percentage points 0..=10000, existing aura bound | yes |
| ChannelAura / RegularBasicSpeedPoints | speed points 0..=10000, existing aura bound | yes |

Recipe union: `DamageFootprint { footprint: Arc<TileRangePattern> }`, `FixedRecipientSupport`, `RegularBasicSupport { damage_percent, speed_points }`, `PersonalShieldSingle { initial_amount: Arc<AmountExpr> }`. Footprint is effect-local on Instant/TileArea damage, not a scalar radius and not a mutation of the shared step. Original TileArea damage already exposes this property. Fixed-channel recipe removes area property without terminating the source channel. Single-personal recipe removes area property, preserves original amount EvalMoment and uses the explicit authored amount. Unconditional edits of a removed property fail; explicit false property predicates become dormant. Regular recipe introduces new typed regular coefficient properties, leaving the old untransformed aura declaration/canonical serialization unchanged.

`SkillEffectDef::PersonalShieldGrant { definition: PersonalShieldGrantDef }` has `amount: Arc<AmountExpr>`, positive `duration_ms: u32`, `priority: i32`, `recipients`, optional `source_creation: ShieldCreationModifierDef { operations: Vec<NumericEditDef> }`. Recipient enum is `PlayerEmployeesAtRelease { radius_units: u32 }` or `SelectedPlayerEmployeeAtRelease { authored_reach_units: u32 }`. Source creation is a new authored typed program, not an existing bonus field. It is explained separately after relic initial-amount composition. Creation amount rejects resolved recipient sets, member stats/HP, recipient stats/HP and TargetEmployee resources; bounded authored aggregates and existing legitimate source/result references retain their permitted expressions. Scalar units/resources and actual owner selectors are validated. Source/result runtime binding and new effect occurrence are still pending. Snapshot/Live and delivery capabilities use the existing catalog helper and actual Goodbye deadline owner.

Area declaration requires the actual Instant SelfUnit carrier; explicit single requires Instant AllySingle and PlayerUnit cast selection. Grant HP cost preserves its existing Instant SelfUnit carrier. The new family is not declared as a RechargeShield source: the existing legacy source-skill validation recognizes only prior ShieldExpr/Sustained Shield. Actual runtime group, finite pools, release-time eligible Employee set, once-only source amount evaluation, whole old-set atomic recast and identity termination are next-bundle work. No new Guest identity or balance value is inferred.

### Syntax examples, not executable fixtures or parser/run evidence

The exact derived serde constructors support the following declaration syntax. Astra must independently parse/observe it; Sol did not run it as a trial or write an oracle.

```ron
(
  id: "syntax_only",
  uuid: "10000000-0000-4000-8000-000000000001",
  name: "Typed syntax",
  description: "Illustrative authored value",
  rarity: "example",
  allow_duplicates: false,
  effects: [],
  rules: [
    (id: "damage_edit",
     selector: (effect: (op: Some(Damage))),
     apply: Edit(property: DamageAmount, operation: Add(Hp(7))))
  ],
  dismantle_equipment_dust: 1,
)
```

```ron
PersonalShieldGrant(definition: (
  amount: (moment: Snapshot, root: Const((value: 120, unit: Magnitude))),
  duration_ms: 8000,
  priority: 10,
  recipients: PlayerEmployeesAtRelease(radius_units: 2000000),
  source_creation: Some((operations: [Add(Hp(3))])),
))
```

```ron
Recipe(PersonalShieldSingle(initial_amount:
  (moment: Snapshot, root: Const((value: 240, unit: Magnitude)))))
```

These are representative authored values, not adopted official formula/range/duration/coefficient/Corrupt penalty. Standalone serde is distinct from the fail-closed normal publication boundary.

## Finite compiler and current evidence seam

Up-front candidates come from canonical three-slot copies x explicit grant rules x actual original action anchors x finite local sites. Grants are never replayed against their own generated actions. Other bindings use this fixed universe, including dormant generated sites. Signed/structural read dependencies are stratified; negative/nonmonotone cycles fail. Same-stratum positive closure uses a finite active worklist; an unseeded cycle stays dormant. Structural property producers settle before dependent consumers. Legacy U08 numeric skill/granted damage modifiers enter the same composition pass; legacy BasicAttack remains its existing separate consumer for now.

The private compile work counter uses the existing data expression max_compile_work, checked work charges and native retained admission before copied owned structures. Predicate, dependency, frontier, canonical-order, fingerprint and DTO copies/serialization have named charges; existing expression/Flow helpers retain their own accounting and limits. DTO compile_usage reports this compiler counter, not a claim to merge every nested helper's global authoring cost. No cap/workload/expected value has been raised. Global co-equip symbolic proof, native runtime plan clone/storage visitors, occurrence/result prefix bindings, dynamic actual narrowing and selected plan admission/currentness are unfinished; focused pure compiler results cannot substitute for them.

For A-only pure compilation tests, the inherited cfg(test/debug/test-fixtures) `ValidatedGameData::from(Arc<GameDataBase>)` fixture-generation path exists. A deliberately staged immutable database can use it, under an explicitly labeled fixture boundary. Normal raw loading/build publication remains refused; do not present the fixture conversion as public acquisition evidence. The final runtime must use actual normal published generation and rebuild after staged mutation/Continue rather than saving executable plan owners.

`RelicPlanError` has code, definition_id/rule_id/field_path/effect/cause. Fault codes include InvalidReference, UnsupportedOperation, UnitMismatch, MissingProperty, UnboundedGrantIdentity, NegativeDependencyCycle, UnprovenStructuralDependency, ReplaceConflict, StructuralWriteConflict, InvalidPreserveContract, ArithmeticOverflow, OutOfRange, ProviderUnavailable, StalePlanBinding, BudgetExceeded, Cancelled; `producer_code()` supplies typed relic_structural_* diagnostics. Future generation/provider active binding is not implied by successful prospective explanation.

## Check and handover

check001 failed with ordinary enum/CloneShape integration errors and is immutable. Root added authoring/amount.rs and target_usefulness.rs to the owned declaration dispatch scope; no semantic exception. scoped_format002/check002 passed; readonly adapter addition then scoped_format003/check003 passed. Check command is cargo check -p game_core --lib --offline --locked -j 1, inherited target/cache, jobs1/debug0/incremental0. Each receipt records actual UTC/env/source/raw/exit. No tests, full build, perf or finalizers ran. check003 has twelve warnings, including the as-yet-unused private validate_effective_configuration seam; warning text remains original.

All original cfg(test) item bytes/registrations and external tests/fixtures remain unchanged from BASELINE001. Astra alone writes registration under explicit whole-file handover; Sol does not own the test code. Source writer/runner are returned to zero. The API and actual source diff are handed to Root for the small independent test connection and the next runtime bundle.
