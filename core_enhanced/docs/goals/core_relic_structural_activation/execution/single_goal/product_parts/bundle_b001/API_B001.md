# U09 bundle B — actual implementation handoff, pending independent cfg verification

The implementation input is 456 selected files / `da23e705d36addd97ba208db7a94c726919cc6e509bba34df2cf454a527aeb46`, captured by `u09_product_bundle_b_check012.json`. The library check passed with the inherited locked/offline, jobs 1, debug 0, incremental 0 environment. This is a compile observation. No U09 runtime test, public CLI, Host test, regression matrix or performance run was executed by Sol for this bundle. B is not CLOSED and the Goal is not complete.

Policies remain GS-CRSA-003 / HA002 / WR006, including CODE_REVIEW_002 TD07–12 and DESIGN_DELTA_002. Authored example amounts below are syntax examples, not official balance or test expectations.

## Actual interfaces and publication

`game/relic/structural.rs` exposes the read-only `explain_effective_plan(&ValidatedGameData, employee_uuid: Uuid, &RelicInventory, &[ProspectiveProvider]) -> Result<Option<RelicEffectivePlanDto>, RelicPlanError>`. The crate-private `compile_effective_plan(EffectivePlanInput) -> Result<Option<Arc<EmployeeEffectivePlan>>, RelicPlanError>` pins the validated generation; `validate_effective_configuration` uses the same compiler. A legacy-only loadout returns `None`. The pure unpublished component compiler is separate and never mints a validated generation.

`EmployeeEffectivePlan` holds `generation: ValidatedGameData` plus plan components. Components own finite Original/Generated effect keys, contributions, recipes, grants, provider entries, usage/fingerprint and `EffectiveProviderPlan` entries. Each provider entry owns the selected `Arc<SkillDef>`, `Arc<CompiledSkillFlow>`, private `Arc<ValueExprCatalog>`, exact source-site-to-effective-key mapping and selected-recipient data-unit reach. These private owners have no deserializer. Generated keys retain relic copy UUID, rule ID, original anchor, local action and local effect; runtime aliases do not replace that identity.

The former A publication rejection is replaced by `structural/compiler.rs::validate_publication` plus `structural/publication.rs`. Every normal/corrupt legal relic multiset of at most three distinct copies is checked against equipment bundles of at most three copies and each compatible active fragment form individually. Relic duplicate/incompatibility admission uses the existing inventory validator. Equipment admission uses the real Any3 slot predicate; actual duplicate bindings on one equipment copy/definition/skill are coalesced only in the provider table. Trigger bindings/schedules remain unchanged. Different copies, definitions, forms and action occurrences remain distinct.

Equipment-only proof sets are checked separately. Fragment proof requires exactly one real selected Weapon profile and calls `UnitCombatProfile::employee_default`, `apply_weapon_profile` and the existing `form_compatibility_report`. The current empty capability-tags context is preserved. No Employee, World, action or generation authority is created. The proof-only identity is discarded; actual World/runtime plans always pin the selected validated identity.

The checked catalog bound is `(1 + normal/corrupt variant choices)^3 × (1 + equipment definitions)^3 × (1 + active forms)`, precharged through existing authoring work before combination traversal. Actual compiler/storage/text work remains charged as well. Each complete candidate retains the original `max_compile_work`; each effective amount compilation is limited to that plan's remaining work before scanning. There is no cap/workload exception. A large catalog may legitimately exhaust its trusted work/storage budget; no partial proof is treated as successful publication.

`structural_capabilities()` returns typed operation/property/unit/range descriptors with `runtime_connected: true`; the prospective DTO uses `execution_status: "runtime_supported_prospective_plan"`. This states the implemented consumer contract and is not an accepted cast or a test PASS. Existing literal-authoring predicates and the old numeric query retain their separate meanings.

## Public actor query, readiness and staged World ownership

The public read-only command is `PlayerBehavior::RequestRelicEffectivePlan { employee_uuid }` / `ActionKind::RequestRelicEffectivePlan`, with `BehaviorResult::RelicEffectivePlan { result: Option<RelicEffectivePlanDto> }` and the corresponding application command/result kinds. It is routed through CoreCommon and the same `run.is_some()` read context as RequestRelicNumericPlan. COMMON_ACTION_PROJECTION_ORDER has 49 entries; the effective query follows the numeric query. Save authority is not expanded. Actor transport remains the existing generic application-command/command-result route; no Server source was changed or built here.

`world/relic_runtime.rs::structural_providers` captures the current active fragment copy/form and all actual equipment-copy skill providers. `validate_structural_loadouts` is called before staged publication of relic equip/variant changes, equipment and fragment changes, and Continue/rollback restoration routes. Failure preserves that transaction's prestate. It does not refund an earlier committed cast cost. Stored inventory/copies remain authoritative; an executable plan is not saved. `capture_structural_battle_owner` rebuilds derived plans when forming a battle.

`BattleCore::set_effective_owner` accepts an optional private `Arc<EffectiveBattleOwner>` and verifies its generation against the battle. This is the native test seam; crate tests can compile an actual plan, put it in the employee UUID map with `EffectiveBattleOwner::new`, and install it on a battle built from the same validated generation. Actual Employee source-owned UUIDs and exact provider copies must match. No synthetic Guest type or employee Auto support was added.

`current_skill_view` / `cloned_current_skill` serve current query/readiness. `pending_skill_view` reads the accepted action's source, and `skill_for_source` / `amount_entry_for_source` serve the retained accepted/cast/release owner. An effective source uses its private catalog without falling back to an unrelated base entry. `EffectiveSkillBinding::validate`, runtime action-source pointer binding and amount-entry pointer checks retain actor/provider/generation/source authority. Strong plan retention does not authorize execution by itself.

World ActivateSkill obtains `current_skill_cast_selection`; focus, preview and pending target checks therefore read the selected skill. `requires_explicit_structural_selection` requires the accepted explicit target for new selected recipes and selected personal declarations. AutoCastStart cannot manufacture that selection. Generic old PlayerUnit selection keeps its old behavior when no new typed selected program is present. Fixed support additionally preserves original authored recipient qualification/reach and caster exclusion; a single personal grant allows a qualified caster.

## Finite branch lifetime and result scope

`ability/flow.rs::attach_branches` keeps the original root, waits, costs, delays and continuation timing. A finite side branch is reserved once per actual original anchor occurrence and starts at that occurrence's EffectsSettled. A Released parent continues independently. Generated anchors do not recursively spawn branches; original Repeat occurrences each have their own finite child.

`skill_runtime/flow/structural.rs` owns the rare `SideOccurrence` collection. A child is preleased from the original authorized action/provider/generation and has no parent use control. The reserved branch source is a chain hold, not the original anchor Delivery and not a child Action's Delivery. Every new child Action uses the unchanged ordinary finite resource path, giving separate Delivery scopes. `resource_runtime.rs` was not changed. Only original same-payload impacts/ticks use existing finite-completion semantics.

Branch cleanup waits for its own released payloads. Cancellation closes unreleased branch authority; legally released payloads retain their own existing delivery tickets. Parent busy/input/channel duration is not extended, and an ended source is not revived. `cast.rs` keeps the actual cast owner while side/delivery work is pending and removes it after settlement. Candidate clone/native retained accounting includes cold side/occurrence owners and private plan extent.

The new `FlowCondition::EffectCompare { node_id, effect_id, metric, comparison, value }` resolves an exact effect site plus producer occurrence. `Compare` retains its complete-Action aggregation meaning. `skill_runtime/flow/results.rs` keeps private `ScopedResultWitness` owners: exact per-occurrence own results and a child prefix seeded only from compiled guaranteed producer scopes. It never copies the latest cumulative parent witness. Own results are merged once, including the exact settled anchor, while child results remain local to that child occurrence. Legacy ActionHandle/global result meaning is preserved. Released Repeat occurrences keep their result owner until their own delivery settles; a late child result cannot settle into the legacy parent frontier after branch cancellation/removal.

Illustrative finite self-heal branch, anchored at an existing original step `bite` with effect `main`:

```ron
(
    id: "after_bite",
    selector: (provider: (fragment_id: Some("example_fragment"), form_id: Some("base"))),
    apply: Grant(FiniteFlow(
        anchor_action_id: "bite",
        steps: [(
            id: "heal_once", target: SelfUnit, delivery: Instant,
            effects: [(id: "heal", op: Heal(amount: 5))],
        )],
        flow: Branch(
            id: "hp_check",
            condition: EffectCompare(node_id: "bite", effect_id: "main",
                metric: HpLost, comparison: Greater, value: Hp(0)),
            then_node: Action(id: "heal_action", step_id: "heal_once", wait: EffectsSettled),
            else_node: Wait(id: "no_heal", duration_ms: 0),
        ),
    )),
)
```

This example was not executed. The original anchor's actual ID/scope must be supplied by the independent author's chosen fixture.

## Effect footprint and regular support

`core/effective_delivery.rs` resolves DamageFootprint per authored effect. Instant uses the original caster release position/facing with qualified enemy targets. TileArea preserves the original anchor/tracking/filter/carrier and uses separate per-effect OncePerArea/OnEnter hit history. Sibling effects keep their original targets. Amount release capture and actual consume resolution use that effect's target set. `effective_footprint_usefulness` connects the same typed footprint to readiness/usefulness and persistent-area admission. Actual effect-result recording supplies scoped HpLost; shield prevention/zero HP loss cannot count as bite HP loss.

`SupportCoefficientScope::{LegacyBasic, RegularOnly}` is the actual aura contract. Applicability is filtered before family priority/tie selection, so an ineligible RegularOnly winner cannot hide an eligible legacy source for Triggered/ExtraAttack. The no-new-scope path keeps the old helper. Regular attacks freeze coefficients and read-only source contributors at attack release in `RuntimeFactSource::regular_support()` / `RegularBasicSupportSnapshot`; later aura end does not rewrite the released attack. `DamageFact.modifier_sources` observes those contributors while original attacker/credit/resource authority remains unchanged. Different family sums/narrowing use checked new typed consumers; old ordinary content has no new regular-scope owner.

## Personal finite sources and fixed connection

`PersonalShieldGrantDef` owns `amount: Arc<AmountExpr>`, positive duration, priority, typed recipients and optional `source_creation: ShieldCreationModifierDef`. The latter is a new authored typed program, not an existing equipment bonus field. Recipe/property namespaces keep TileRangePattern distinct from SupportAreaRadius data units.

Illustrative original typed effect:

```ron
PersonalShieldGrant(definition: (
    amount: (moment: Snapshot, root: Const((value: 120, unit: Magnitude))),
    duration_ms: 5000, priority: 0,
    recipients: PlayerEmployeesAtRelease(radius_units: 2000),
    source_creation: Some((operations: [Add(Hp(10))])),
))
```

The area carrier is Instant SelfUnit. SelectedPlayerEmployeeAtRelease requires the strict selected PlayerUnit/AllySingle carrier, except a compiler-proven PersonalShieldSingle lowering preserves the original Self carrier and stores accepted selection separately.

`begin_personal_shield_grant` evaluates the source amount once, applies the source-creation program once, captures the actual release-time normal Player Employee set, and admits the complete old/new record set before mutation. Every recipient owns an independent finite personal pool with existing priority/protection ordering. Group identity is caster instance + actual equipment or fragment provider snapshot + lossless Original/Generated effective key. A recast replaces the whole old set for that group, including members now outside range; other sources remain independent. Empty/zero creation is refused.

Recipient invalidation ends only that recipient's personal pool. Source identity/provider end ends that source's remaining pools. Released caster CC alone does not undo a grant. Redeploy/new UnitInstanceId cannot inherit old recipient/source identity. `PersonalShieldGrantView` identifies the effective source and reports `legacy_recharge_supported: false`; generic legacy RechargeShield excludes this family without changing its old latest-record behavior.

FixedRecipientSupport keeps the channel owned by the original caster and its original cost/expiry. `FixedSupportConnectionView` exposes selected UnitInstanceId, authored data-unit reach and InRange/OutOfRange/TerminalInvalid. Range-out removes membership; return reconnects the same live identity/pool. Death/withdraw/redeploy makes that connection terminal, with no identity transfer and no automatic caster/channel termination solely due to recipient invalidation.

## Independent author seams and current limits

Astra owns every test/include/registration. Existing A expectations for the publication gate and `runtime_binding_pending` now need successor authoring against these real interfaces. No inherited test was rerun to claim those old expectations still pass.

The next observations should exercise public legal co-equip publication/conflicts and World query/rollback/Continue, current-vs-retained generation, pending/cast/release admission, original Released timing with a long anchor, same-time later parent/sibling result isolation, original/child Repeat and cancellation, distinct child Delivery resource scopes, effect-local sibling footprint/HP-loss/self-heal, fixed recipient lifetime, RegularOnly family eligibility and immutable contributor sources, and personal whole-set replacement/provider/recipient/legacy-recharge boundaries. Numeric/work/storage oracles remain independent author decisions; no cap increase or expected-value exemption was introduced.

The historical format001 ran before the final explicit-selection guard. Its cfg span classifier was found incorrect for enum variants/match arms. CFG_TEST_PRESERVATION.json and CFG_TEST_RESTORATION_001.json are invalid as final test-byte proof; the failed records and check011 remain preserved. Production behavior_owner is restored, check012 passes, and the new independent successor cfg verifier is Astra's assignment. No successor comparator/parser was written or run by Sol after Root's stop instruction. Final cfg preservation and final cfg-safe format currentness are pending Root/Astra acceptance.
