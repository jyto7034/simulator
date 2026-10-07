# U09 GEX-00 product design 001

Status: proposed implementation contract for the inherited frozen GOAL-CRSA-01 / GS-CRSA-001 / AG-CRSA-RUNTIME basis. NOT execution-ready. This file changes no product, policy, test, fixture, comparator or acceptance criterion. Existing artifacts retain their original authorship.

Post-freeze material change intake: Root accepted core_first/execution/porccubus_personal_shield_20261007_001/RECORD.ko.md (2974 bytes, sha256077df65c23e1d7b909e7c9e86fdb1600aeaabb3b1e381775adf5073ccdc44f38) at this safe pre-product-mutation boundary. The latest user direction is cast-time personal shields per originally eligible recipient, with separate remaining/duration and no late-join/range-exit removal; single conversion magnitude/penalties and unresolved lifecycle must be reconsidered. This differs materially from frozen shared2S. The Porccubus shared/2S and old shield membership/end/recast assumptions below are SUPERSEDED-PENDING, retained as historical proposal text, never an executable current requirement. Sections2.3,3.3,6 and11 contain such inherited shield references; the fixed-channel/aura contract, general R12 shared support and non-shield design remain proposals for reuse. Root owns successor policy/basis and unresolved personal-shield design, so this file does not substitute a new balance/lifecycle decision. Product/test writes, builds and runs remain0.

Owner: /root/u08_product_runner_resume, accepted spawn gpt-6.1-sol / xhigh. Root confirmed the same live allocation; no reassignment occurred. Baseline and actual source references are bound by ../baseline/BASELINE_001.json. Current source: 438 files, 4c7bbd317a9a12045267bd306057e771133096ccbccace6a51f484d23a46e2ac.

## 1. Basis, actual gaps and fixed boundaries

The inherited basis was HA-CRSA-001 → WR-CRSA-003 / FPF-CRSA-001 / PP-CRSA-001 / GS-CRSA-001 with then-empty change/obligation sets. It is now stale for execution. Root's POST_FREEZE_CHANGE_001.json supplies CHG-CRSA-001 / COD-CRSA-001; P2G_10_POLICY_002.json and P2G_10_CHECKPOINT_003.json locate the open WR-CRSA-004 checkpoint. Only PE04/08/09 are materially reopened,14 policy meanings preserved; O10→60 repairs are pending. The exact locator refs are in BASELINE_001.json. This file retains the old frozen-basis proposal and does not claim a repaired current executable basis.

P2G_60_HANDOFF_001.json, P2G_50_GOAL_SET.ko.md, P2G_40_DESIGN_001.ko.md and CODE_REVIEW_001.ko.md TD01–06 own that inherited proposal's semantics. The earlier runtime report is preserved, with Root's final TD06 clarifications applied here. RUNTIME_REVIEW_001.json's41 exact refs/ranges have been rechecked against unchanged product bytes. Its proposed APIs are not existing APIs.

Actual gaps:

- game/data/relic_data.rs:8–110,173–304 supports strict legacy damage selectors/effects only, requires nonempty numeric effects and negative authored corruption penalties, and contains no condition/grant/recipe graph.
- game/ability/authoring.rs:12–59,114–230 has ActionSite, EffectSite and source attribution; its editable descriptor is Instant literal Amount only. Target and Delivery are carrier properties deliberately excluded from effect writes.
- game/relic/plan.rs:15–123,172–282,305–414 produces a prospective damage explanation. It does not produce a personal executable SkillDef or structural closure.
- battle/core/mod.rs:536–546; sim.rs readiness/accepted/queued step; skill_runtime/flow.rs:479–511; skill_runtime/area.rs:747 onward re-read global SkillDef. PendingSkillCast has no selected plan owner (types.rs:242–252).
- sustained.rs:49–108,870–954 ties aura/shared shield to caster-centered area membership and currently uses shared_radius_units=None to identify personal protection priority. Fixed recipient and shared-stage classification must be separate.
- basic_attack.rs:438–477 chooses Auto or Triggered, but damage_runtime.rs:145–163 sees BasicAttack for both. New regular applicability cannot be inferred from DamageSource alone.
- World application.rs:403–475 already clones and commits a staged draft; maintenance.rs and relic_runtime.rs currently validate inventory/provider compatibility, not effective structural conflicts. node_flow.rs:1365/1604 and continuation.rs validate persisted ownership/generation before restore, providing the place for plan reconstruction.

Inherited frozen implementation invariants (Porccubus items3 and shared2S portions are superseded-pending by the change note above):

1. Same-effect tag/property AND plus separate exact provider/fragment/form identity. Historical plan is never a closure seed.
2. Recipient invalid terminates only its support connection. Original caster activation, channel, cost, expiry and original source end conditions remain.
3. Single2S remains at the shared protection stage. A one-member pool must not become a personal shield by clearing radius.
4. Only new explicitly typed regular coefficients use the regular predicate. Legacy untransformed ChannelAura keeps its existing applicability and arithmetic.
5. Monk changes only the damage footprint. The original Physical bite's settled HpLost>0 controls one Self heal, with its base amount/source/carrier/cost/timing preserved.
6. New provenance is not action/resource/reinput/boss authority. Ego-corrosion suppression stays upstream of effect compilation and execution.
7. No new gameplay proc cap, official S/duration/corruption penalty, employee active Auto support, U10 reactive subscriber or live-battle disk-resume requirement.

## 2. Schema and typed capability contract (API01)

### 2.1 Backward-compatible entry

Extend RelicDefinition and RelicCorruptionVariant with a default empty, strictly decoded rules: Vec<RelicRuleDef>. Existing effects: Vec<RelicDamageEffect> remains byte-compatible and retains its current validation and arithmetic. A new structural-only definition writes effects: [] explicitly and must have at least one rule. A legacy entry with both lists empty still fails. The old numeric validation must only be called for a nonempty effects list; this is a deliberate extension, not silently dropping a malformed legacy effect.

Normal and Corrupt select their own complete effect/rule lists, matching the existing variant replacement behavior. Corrupt penalties keep the existing mandatory authored negative penalty contract. Representative tests may use that existing authored penalty plus the original skill's HealthCost; do not add an official relic HP penalty or invent balance. Every nested struct uses deny_unknown_fields and every operation uses an explicit enum. No untagged fallback and no string field patch.

Proposed schema types:

~~~rust
pub struct RelicRuleDef {
    pub id: String,
    pub selector: StructuralSelectorDef,
    pub when: RelicConditionDef,       // default Always
    pub apply: RelicRuleActionDef,
}
pub struct StructuralSelectorDef {
    pub provider: ProviderPredicateDef, // exact equipment/fragment/form IDs; optional
    pub effect: EffectPredicateDef,     // AND on a single actual effect
}
pub struct ProviderPredicateDef {
    pub equipment_id: Option<String>,   // each optional/list field defaults empty
    pub fragment_id: Option<String>,
    pub form_id: Option<String>,
}
pub struct EffectPredicateDef {
    pub all_tags: Vec<String>,
    pub op: Option<EffectOperation>,
    pub has_property: Option<RelicProperty>,
}
pub enum RelicConditionDef {
    Always,
    SameEffect(EffectPredicateDef),
    Exists(StructuralSelectorDef),      // one witness satisfies the whole selector
    All(Vec<RelicConditionDef>),
    Any(Vec<RelicConditionDef>),
    Not(Box<RelicConditionDef>),
}
pub enum RelicRuleActionDef {
    Grant(GrantBodyDef),
    Edit { property: RelicProperty, operation: NumericEditDef },
    Recipe(StructuralRecipeDef),
}
pub enum NumericEditDef {
    Add(TypedRelicValue),
    Scale { increment_percent: i64 },  // same-kind increments summed once
    Replace(TypedRelicValue),
}
pub enum TypedRelicValue {
    Hp(i64),
    PercentPoints(i64),
    SpeedPoints(i64),
    DataUnits(i64),
}
~~~

ProviderPredicateDef reuses current exact equipment_id / fragment_id / form_id semantics. The proposed typed reference validator rejects a supplied form without its required fragment, rather than treating it as a broad tag. Employee/copy/runtime identities are bound by configuration, not writable authored selector fields. SameEffect and the outer effect selector share one witness. Exists selects a separate full witness; All(Exists(A), Exists(B)) deliberately permits different witnesses and does not mean same-effect AND. EffectOperation is normalized per actual operation family (Damage includes literal/expr damage), with its real amount/catalog capability still checked; it is not an inferred tag alias.

RelicProperty proposed stable enum:
DamageAmount, DamageFootprint, SupportAreaRadius, ShieldInitialAmount, RegularBasicDamagePercent, RegularBasicSpeedPoints. Structural support-target and footprint shape changes use recipes, not numeric string writes. DamageType and original Action/Target/Delivery/Channel/Cost/Time/Authority remain readable attributes without generic write rights.

GrantBodyDef is a strict union:
- InlineEffects { action_id, effects: Vec<GrantedEffectDef> }: finite primitive effects at an existing original provider action. Each granted effect has a local authored id, tags and SkillEffectDef, with no nested RelicRuleDef or subscriber.
- FiniteFlow { anchor_action_id, steps, flow }: a finite existing SkillFlowDef region with local addresses, grafted as an explicitly validated region of the carrier. It has no recursive grant template. Original flow region, references, delays, milestones and cost/authority are preserved; new local references resolve only inside the finite granted region or approved prior original results.

The minimal initial connector is InlineEffects Damage/DamageExpr. FiniteFlow must not be advertised as executable until its graft, source/catalog validation and runtime consumer are connected. This is implementation ordering within the atomic group, not a narrowed final policy. Existing nonreactive primitives are exposed only where their existing validator and real executor/resource capabilities permit execution. Unsupported reactive authoring returns a typed diagnostic. A granted HealthCost/resources effect never creates a missing capability.

### 2.2 Single descriptor table used by producer, query and executor

Add a U09-specific capability module. Do not broaden the old authoring EFFECT_SUPPORT table indiscriminately; it still serves existing literal authoring APIs. The new descriptor is a typed table with operation, consumer/carrier compatibility, property presence, value unit/range, allowed Add/Scale/Replace, recipe input/output support and preservation constraints.

Proposed public read-only interface:

~~~rust
pub struct RelicPropertyCapability {
    pub property: RelicProperty,
    pub value_type: RelicPropertyType,
    pub unit: RelicUnit,
    pub numeric_range: Option<(i64, i64)>,
    pub add: bool,
    pub scale: bool,
    pub replace: bool,
}
pub fn structural_capabilities() -> &'static [RelicCapabilityDescriptor];
pub(crate) fn capabilities_for(
    effect: &EffectiveEffectView<'_>,
) -> EffectiveCapabilities<'_>;
~~~

Supported initial numeric contracts:

| Property | Consumer and presence | Unit/range | Operations |
|---|---|---|---|
| DamageAmount | executable Damage/DamageExpr; original or granted | HP, nonnegative existing damage narrowing | Add/Scale/Replace |
| DamageFootprint | that damage's effective tile-area footprint only | TileRangePattern, existing validated mask geometry | recipe-only; no numeric edit |
| SupportAreaRadius | area support structure only, not fixed-single connection reach | data units, positive existing bounds | Add/Scale/Replace |
| ShieldInitialAmount | original source ShieldExpr/shared Shield source initialization | HP, existing shield narrowing | Add/Scale/Replace before single2S terminal doubling |
| RegularBasicDamagePercent | explicitly new RegularBasic support coefficient | percentage points, existing ChannelAura coefficient range | Add/Scale/Replace |
| RegularBasicSpeedPoints | explicitly new RegularBasic support coefficient | speed points, existing ChannelAura coefficient range | Add/Scale/Replace |

Values are checked in their declared unit and final supported range. Literal expression/result compatibility is compiled with existing value_expr units; no implicit resource-ID/unit cast. The actual DeliveryDef::TileArea footprint is TileRangePattern (ability.rs:194; battle/tile_range.rs:56), not a scalar radius. Its mask is validated and replaced through a typed recipe; this design does not advertise an invented numeric damage-radius property. Fixed-single support loses SupportAreaRadius, but retains a read-only original authored reach. An unconditional attempted Edit of a removed/unsupported property is rejected; a rule whose explicit property predicate is false is dormant with an explanation. Property presence and write permission are distinct.

### 2.3 Recipes and typed diagnostics

Proposed recipe enum:

~~~rust
pub enum StructuralRecipeDef {
    DamageFootprint { footprint: TileRangePattern },
    FixedRecipientSupport { shield_initial_multiplier: Option<u32> },
    RegularBasicSupport { damage_percent: u32, speed_points: u32 },
}
~~~

FixedRecipientSupport's authored multiplier is validated against the registered recipe: 2 for the approved shared-shield single recipe, absent for aura. It is not arbitrary private-field mutation. RegularBasicSupport final values are authored state replacements, so the normal typed aura base can declare20/15, normal single rules40/20 and Corrupt single80/30. Do not infer a specific relic identity from tags or hardcode official content IDs. Their authored selector/recipe structure proves applicability.

Recipe descriptors declare writes and preserve lists. The compiler checks the whole atomic bundle: target qualification/reach, original provider/source, carrier, channel/use lifetime, cost/resource owner, time, action/delivery authority, damage type and sibling effects. Multiple compatible identical recipe writes merge provenance. Incompatible overlap is a conflict even if applying one order would appear to work.

Proposed diagnostic code enum:
UnknownProperty, UnsupportedOperation, UnitMismatch, MissingProperty, InvalidReference, UnboundedGrantIdentity, NegativeDependencyCycle, UnprovenStructuralDependency, ReplaceConflict, StructuralWriteConflict, InvalidPreserveContract, ArithmeticOverflow, OutOfRange, ProviderUnavailable, StalePlanBinding. Budget/cancellation uses existing authoring/native fault types, not a success/dormancy fallback.

RelicCompileDiagnostic owns code, exact relic source file/definition/rule/field path, affected original/generated sites, expected/actual typed values and dependency-cycle witness where relevant. Convert it to LiveRonError preserving producer phase Compile or FinalValidation as appropriate, not an invented parse error. Relic source paths are bound while winning RON definitions are assembled, parallel to skill_data::bind_definition_sources (593–622). Generated source is not manufactured from a fake SkillId.

## 3. Finite identities, proof and composition

### 3.1 Identity universe before closure

Original effects retain EffectSite. Generated effects have an explicit distinct key:

~~~rust
pub enum EffectiveEffectKey {
    Original(EffectSite),
    Generated {
        relic_copy_uuid: Uuid,
        grant_rule_id: String,
        anchor: OriginalProviderSite,
        local_effect_id: String,
    },
}
~~~

OriginalProviderSite is a provider's original ActionSite / EffectSite anchor, including exact provider definition/form in the configuration key. A grant matched by an already generated effect still emits the same finite local candidates at the original provider anchor. It does not create another grant definition or extend a recursive identity path. Empty/missing original anchor, duplicate local IDs, or authoring that demands one fresh identity per generated witness is rejected before publication.

Bound all candidates as a checked sum over actual equipped copy × authored grant site × eligible original provider anchor × local body sites. Bound dependency edges/frontier/result/source-string storage before allocation. No generated counter/UUID/RNG allocation is used by compilation. A deterministic encoded executor EffectSite can index existing catalogs, but EffectiveEffectKey remains the unambiguous identity/provenance authority; no truncated digest collision is accepted.

Publication computes a symbolic finite proof for eligible provider anchors and co-equip combinations up to the existing three-slot limit, including duplicate copies only where authored duplicate policy allows them. Incompatible definitions are excluded only by their actual inventory contract. Charge the checked combination/edge bound before traversing it. Runtime/prospective compilation instantiates that proof with exact current copies and fails closed if the instantiated graph disagrees. This is neither a new gameplay total cap nor an unmetered enumeration.

### 3.2 Closure and stable structure

Represent effect existence, property presence and rule activation as finite indexed facts, not an append-only stream of newly cloned definitions. Start with current original effects and unconditional grants; never with old plan outputs.

Build a signed dependency graph. Positive SCCs containing only monotone existence/grant facts are solved by a worklist; add each previously inactive candidate at most once. No seed means no activation. Seed removal means a fresh evaluation with those facts absent. Slot/UUID order only sorts provenance and cannot choose semantic precedence.

Nonmonotone writers (property removal, operation/structure replacement) and Not reads introduce stratification constraints. A reader of a final property depends on every writer that can change its presence/type, so it cannot observe a transient pre-recipe property and keep an invalid activation. Apply proved lower-stratum atomic recipe writers, freeze that stratum's effective descriptor facts, then evaluate dependent conditions. Stable positive cycles must not be rejected merely for being cycles. A negative SCC, self-removing selector or structural dependency without a stable interpretation is publication-rejected with the exact cycle/write witness. Idempotent equal writes may be proved stable; do not silently treat all replacements as monotone.

For this initial implementation, condition predicates inspect tag/operation/property presence/exact identity, not arbitrary evolving numeric values. This avoids making general arithmetic fixed-point solving a claimed public capability. Unsupported numeric-value conditions are rejected, rather than ignored.

Keep every rule/copy in the explanation with Active / Dormant(reason) or Rejected(diagnostic). Missing weapon is ProviderUnavailable and does not synthesize a provider. A valid unmet condition is dormant. An actually selected incompatible write is rejected and cannot be hidden as dormancy.

### 3.3 One deterministic numeric path

For each exact effect/property gather active contributions without applying them in UUID order:

1. Select original base or one agreed Replace value. Equal same-unit/same-meaning Replace merges all provenance; different values or structural meanings reject.
2. Sum Add in the property's unit with checked wide arithmetic.
3. Sum same-kind Scale increments; compute one checked rational scale and truncate once to the property's integer unit.
4. Validate signedness/range and existing consumer narrowing.

Legacy damage flat/percent entries are normalized as Add(HP) and Scale increments at the same existing U08 composition stage; do not call both normalized composition and RelicNumericBinding composition on the same final payload. Runtime-dependent DamageExpr retains its original compiled evaluator: its base result is narrowed at the original stage, then the single compiled modifier program executes. Prospectively unresolved values remain RuntimeContextRequired; query does not pretend a fabricated actor evaluation was actual runtime damage.

Shield source expressions/modifiers are evaluated once for the caster at the original initialization point. Then the single2S recipe doubles the source result once with checked arithmetic. It neither re-evaluates on each member nor applies recipient equipment. A generic ShieldInitialAmount edit is upstream of that terminal recipe and cannot masquerade as a second runtime multiplier.

## 4. Personal compiler, public query and binding (API02)

### 4.1 Layers and signatures

No global mutable cache or shared catalog mutation. Start without a persistent World cache: query and prospective mutation invoke the same pure compiler. Battle may retain immutable compiled plans in the existing cold relic-owned branch, indexed by a complete configuration key. Add caching only when its bounds/currentness are explicit and measurements justify it.

~~~rust
pub(crate) struct EffectivePlanInput<'a> {
    pub data: &'a GameDataBase,              // already validated generation only
    pub generation: &'a ContentIdentity,
    pub employee_uuid: Uuid,
    pub inventory: &'a RelicInventory,
    pub providers: &'a [ProspectiveProvider],
}
pub(crate) fn compile_effective_plan(
    input: EffectivePlanInput<'_>,
) -> Result<Arc<EmployeeEffectivePlan>, RelicPlanError>;
pub(crate) fn validate_effective_configuration(
    input: EffectivePlanInput<'_>,
) -> Result<(), RelicPlanError>;
pub(crate) fn explain_effective_plan(
    plan: &EmployeeEffectivePlan,
) -> Result<RelicEffectivePlanDto, GameError>;
~~~

These names are proposed, not present in source. External integration tests use normal loader/commands/query; crate-local compiler tests may call these crate-private entrypoints after registration handover. The public explanation never supplies a trusted runtime token.

Configuration key includes content identity, employee identity, original skill/provider definition, equipment persistent-copy UUID, fragment copy UUID and selected form, canonical relic copy/definition/variant set. Runtime binding adds UnitInstanceId, actual runtime item instance, accepted ActionHandle and selected plan fingerprint. Query's prospective key has no fabricated runtime UUID/action. Skills with identical SkillId but distinct employee/copy/form/generation are not interchangeable.

EmployeeEffectivePlan owns:
- immutable original generation reference and complete configuration key;
- final original/generated effect graph, canonical closure evidence and diagnostics;
- original SkillDef plus immutable effective executable skill/Flow views per provider;
- effect-local execution policies indexed by EffectiveEffectKey;
- validated local amount catalog and amount-modifier programs;
- recipe/preserve contracts and sorted contribution provenance;
- accounted retained extent and finite-proof fingerprint.

No typed rule / no transformed provider takes the exact existing original/numeric path. Do not eagerly clone every SkillDef/amount catalog into an empty owner. Any type-size or cold owner increase remains measurable, not presumed zero.

### 4.2 Amount/Flow catalog adapter

ValueExprCatalog::compile_skills (catalog.rs:92 onward) and SkillDatabase::compiled_flow (skill_data.rs:451–470) are reusable validators. Factor their borrowed one-skill/one-effective-effect producer helpers so the effective candidate gets the same result/resource/Flow/delivery capability validation without reconstructing the full GameDataBase.

Original unmodified effects may share their existing compiled entries. Edited/granted effects get plan-local compiled entries keyed by their validated effective source site and exact plan fingerprint. The validator consumes the effect-local target/delivery capability overlay, rather than blindly using the step's carrier for every effect. Recheck attached_to, EarlierEffect/previous settled results, use-lifetime references, Flow node sites and dependency scope after rewriting.

Runtime must select original or plan-local catalog through a private selected-plan resolver. Preserve authored==compiled source, generation, provider/action/delivery and Snapshot/Live checks in value_expr_runtime.rs:2466 onward. Never register a new public catalog generation to conceal a local transform and never skip a source mismatch merely because a relic exists. Generated source keeps original provider definition/site plus a separate RelicContribution; existing EffectOrigin::Added is not abused to pretend the relic is a skill definition.

### 4.3 New public read-only envelope

Keep RequestRelicNumericPlan and RelicNumericPlanDto unchanged. Add:

~~~rust
PlayerBehavior::RequestRelicEffectivePlan { employee_uuid: Uuid }
BehaviorResult::RelicEffectivePlan { result: RelicEffectivePlanDto }
pub struct RelicEffectivePlanDto {
    pub schema_version: u32,               // 1
    pub generation: ContentIdentity,
    pub employee_uuid: Uuid,
    pub scope: EffectivePlanScope,         // prospective_loadout
    pub fingerprint: String,
    pub providers: Vec<EffectiveProviderDto>,
    pub rules: Vec<RelicRuleStateDto>,
    pub effects: Vec<EffectiveEffectDto>,
    pub capabilities: Vec<RelicCapabilityDto>,
}
~~~

Effect DTO has original/generated key, original site/provider, final operation, per-effect execution structure, before/after typed property values or unresolved runtime value, final property support, all contributing copy/variant/rule refs, active/dormant explanation. It contains no private ActionToken, RuntimeFactSource, resource capability or executable Arc.

Update exhaustive ActionKind/route/intent/result conversion/query classification and owned DTO admission. Existing CoreCommon query context restrictions remain; new query grants no mutation. Host typed result serialization uses the normal envelope, and documentation must state version1. Global SkillCatalogDto version3 remains original catalog, not an actor-specific transformed catalog. Readiness/preview can return the selected effective plan fingerprint and support reach/qualification through their typed extension; do not reinterpret the old single step tile_area field as multiple effect footprints.

## 5. Runtime owner selection, release and source (API02/API04/API05)

Resolve by actor+actual provider before readiness/preview/manual/basic selection. For transformed skills obtain a strong immutable RuntimeEffectivePlanBinding privately constructed from a validated current plan. Bind it at admission, before accepting/costing an action. Store/retrieve the pending owner through the accepted action's cold source/plan binding, so queued activation cannot simply recompile the new current loadout. Active cast, Flow wake, deferred step, released projectile/area and delayed child all retain the same owner.

Prefer a private cold binding beside the existing relic source slots rather than inflating every ordinary event/pending structure. If implementation requires a field in PendingSkillCast or ActiveSkillCast, include its real layout/cap impact and use the same release/lifetime rules. Ownership layout is an internal choice, not grounds to relax PC/R05.

Proposed internal resolver:

~~~rust
pub(super) fn selected_skill(
    &self,
    source: &RuntimeFactSource,
    actor: UnitInstanceId,
    skill: &SkillId,
) -> Result<SelectedSkillView, GameError>;
pub(super) fn effective_skill_for_pending(
    &self,
    pending: &PendingSkillCast,
) -> Result<SelectedSkillView, GameError>;
~~~

SelectedSkillView is Original or Effective(strong validated owner), and exposes immutable skill/Flow/effect execution/amount catalog. Use it at World combat.rs focus/preview/activate, BattleCore sim.rs readiness/start/SkillStep, flow.rs wake and area.rs tick. Inventory/latest catalog is not an alternate delayed lookup after acceptance.

RuntimeFactSource primary actor/equipment/fragment/action/cast/delivery remain original. A new private cold RelicPlanBinding carries selected plan and read-only generated contributions. Observation projections strip authority as before; retaining a plan explanation does not make a projected source executable. Internal executor still requires actual accepted action/provider/generation/delivery guards. Child/proc operations resolve their own original provider and preserve finite-child rules instead of inheriting a broad arbitrary plan capability.

Released payload retains strong owner and existing amount capture. Snapshot evaluates at its existing capture point; Live updates allowed values/context only, never plan structure/generation selection. Configuration change cannot rewrite an already selected/released payload. Withdraw/source-death ends only owners already ended by original use/channel/survival policy. Allowed remaining released payload keeps its original validated owner until done. Last strong runtime/cast/delivery/query reference drops the plan naturally; Weak-only observation cannot guarantee continued availability.

## 6. Fixed support recipient and inherited single2S (API03)

SUPERSEDED-PENDING for Porccubus: the old shared pool, range-out/rejoin membership, shared protection stage,2S multiplier and proposed shield recast lifecycle below are preserved only as this draft's prior frozen-basis design. New personal-shield quantity/multipliers/eligibility/lifecycle must follow Root's successor basis. The fixed recipient/channel-aura design still applies to the unchanged fetus contract; existing general R12 shared shield support is not redefined by the Porccubus policy change.

Add a compiled support-target policy, distinct from the caster lease and original skill target:

~~~rust
pub enum SupportRecipientPolicy {
    Area { radius_units: u32, include_caster: bool },
    FixedAtActivation { authored_reach_units: u32, include_caster: bool },
}
pub enum ProtectionStage { Personal, Shared }
pub struct FixedSupportConnection {
    pub recipient: UnitInstanceId,
    pub state: SupportConnectionState,
}
pub enum SupportConnectionState {
    InRange,
    OutOfRange,
    TerminalInvalid { reason: RecipientInvalidReason },
}
~~~

Fixed connection is mutable value state inside the existing sustained Record/World draft, not shared RefCell/Mutex state. Record.caster and record.target for source lease remain caster-owned for aura/shared shield. Connection recipient is a separate field; do not overwrite the caster lease target. Original area source qualifies employee identity, owner side, active combatant and employee trauma rules. Apply the exact same eligibility predicate and authored world-distance radius at selection, readiness, preview, activation and refresh. Tile reach/AllySingle alone is insufficient; existing manual Unit target is reused with this adapter. Fetus excludes caster; approved shield support includes eligible caster.

BeginSkillTargeting / PreviewSkillTarget / ActivateSkill retain existing Unit target payloads and battle revision/authority. Effective cast selection becomes PlayerUnit for the fixed support, but this does not mutate original action/carrier source fields or allow a missing provider. Pause acceptance validates and stores the selected plan/recipient; resume revalidates qualification/reach under original admitted-target rules. Existing no-target ReinputSustainedSkill stops the exact activation; target reinput cannot switch recipient.

At activation:
1. Validate support structure and exact selected UnitInstanceId.
2. Admit owner/connection/member storage.
3. Compute shield initialization S once from caster source; apply approved2 multiplier once.
4. Create one sustained activation, one shared capacity/remaining pool and at most one fixed member.
5. For a matching recast replacement, identify old activation by caster/provider copy/form/generation/original producer site. Atomically end that old source activation and publish the new owner only after the new draft is valid. Generic begin_sustained currently has no such shield guarantee, so add the exact recipe path, not a blanket all-shields replacement rule.

Refresh:
- Valid inside original reach: membership is [same recipient].
- Alive/eligible same identity outside reach: membership empty, state OutOfRange, same pool/caster/channel/cost/expiry.
- Re-enter reach with same valid identity: membership restored, state InRange, same remaining; no refill.
- Withdrawn/dead/combat-invalid identity: membership empty and that connection terminal. Same employee redeployed with a new UnitInstanceId cannot revive it. Terminal does not end source activation or waive its ongoing cost.
- Original caster source end/expiry/depletion/recast rules end the source independently and release connection/owner.

ShieldPoolView retains original radius/reach information, while absorption uses explicit ProtectionStage for transformed pools. Legacy record classification is unchanged. Personal shields absorb before this shared single pool; priority/activation ordering within stage remains. Damage mitigation precedes shields, excess reaches HP, PayHealthCost bypasses shield, Heal/Rest do not refill and explicit original recharge retains its original contract.

Public sustained witness extension is schema-versioned and read-only: activation/caster, original provider/site and plan fingerprint, recipient policy/selected recipient/link state, source expires_at/channel state, protection stage, capacity/remaining and current members. Do not equate the latest witness with a full lifetime history.

## 7. Regular-basic coefficient and contributor transport (API04)

Add explicit runtime occurrence classification at AttackStart:

~~~rust
pub enum BasicAttackOccurrenceKind { Regular, Triggered }
pub enum SupportCoefficientScope { LegacyBasic, RegularBasic }
pub struct SupportContributionSnapshot {
    pub activation: u64,
    pub provider_actor: UnitInstanceId,
    pub generation: ContentIdentity,
    pub original_provider: OriginalProviderIdentity,
    pub original_site: EffectSite,
    pub contributions: Vec<RelicContributionDto>,
    pub damage_percent: u32,
    pub speed_points: u32,
}
~~~

Use existing AttackKind::Auto / schedule_next to classify current regular cycle; Triggered and ExtraAttack stay Triggered. DamageSource::BasicAttack alone is not enough. Preserve the existing snapshot creation time in basic_attack.rs:463; do not move it from AttackStart to impact while solving this problem.

Legacy ChannelAuraDef inputs default to LegacyBasic and their original consumers/arithmetic remain. Add an explicit default-compatible basic_scope field or an equivalent typed compiled overlay for authored new RegularBasic support. Representative base20/15 must itself opt into that new typed scope, and recipes replace its coefficients with40/20 or80/30. A legacy input does not become regular-only because the implementation changed a shared helper.

Resolve scope applicability before same-family priority/nonstack selection. Otherwise an inapplicable typed regular winner could suppress a legacy contributor during Triggered execution. Keep original family dominance/priority/tie contract for applicable contributors; two sources of the same family do not stack. Damage aura modifies base before recipient attacker's U08 numeric modifier once and original resistance/shield pipeline. Speed points modify only regular basic scheduling/interval at its original boundary; no global Attack stat, active proc, resonance or work speed mutation.

The immutable attack snapshot stores applied typed coefficient/contributor info in a cold source slot. Later channel end/range exit/recipe change does not recompute an already released attack. Provider actor is the support caster; primary damage actor/kill credit remains the recipient attacker. No action/resource/reinput capability is copied from the caster into the recipient's attack. Checked new coefficient arithmetic rejects overflow/out-of-range; do not add a new saturating success fallback.

## 8. Effect-local footprint and Monk (API05)

Use a plan-owned execution overlay keyed by EffectiveEffectKey. Keep SkillStepDef target/delivery and sibling operations untouched:

~~~rust
pub enum EffectiveEffectExecution {
    InheritCarrier,
    DamageFootprint { footprint: TileRangePattern },
    Support { recipient: SupportRecipientPolicy, protection: Option<ProtectionStage> },
}
~~~

DamageFootprint is a spatial resolution override for that damage at the original carrier's execution point. It is not a new independent action/projectile or a blanket TileArea delivery replacement. Initially register only real consumer/geometry combinations implemented by the existing spatial executor (Instant and tile-area damage, including their authored amount capabilities). Projectile impact support is advertised only after its local-at-impact consumer is implemented; released projectile ownership remains required for all supported transformed/generated effects.

Factor target collection/execution helpers to accept the selected effect's view. For an inherited effect use the exact old target path. For footprint damage collect qualified enemies using the new footprint at the original anchor/facing/release point; retain one carrier action/delivery and the original effect address. Existing result prefix, attached effect gates and compiled delivery capabilities must refer to that damage's actual target/result set. Other effects use their own inherited target route and cannot accidentally heal every new damage target.

Monk representative authoring reuses existing finite Flow and FlowMetric. A bite-only Action can use existing Compare(HpLost). A mixed Action with another original/granted damage needs an exact effect scope, because the current Compare is whole-node aggregate and could otherwise heal when only that sibling lost HP. Add an explicit new FlowCondition variant without changing old Compare constructors or semantics:

~~~rust
FlowCondition::EffectCompare {
    node_id: String,
    effect_id: String,
    metric: FlowMetric,
    comparison: FlowComparison,
    value: FlowLiteral,
}
~~~

Compile node/effect membership and EffectsSettled dependency with the existing finite Flow validator. The new result scope is private (ActionHandle, Flow action occurrence, EffectiveEffectKey), with a bounded cold effect-result table only where required by a typed scoped condition. Damage settlement contributes actual_hp_loss to that exact effect occurrence; parent aggregate results remain unchanged. Sibling damage within the same activation, or a separate repeated occurrence/hit from another producer, cannot contribute to the bite's scoped result. Multiple targets/hits belonging to the same bite occurrence aggregate only that bite's actual HP loss, and its one settled branch cannot emit one heal per hit. Clear/reset scoped results with the corresponding Flow occurrence, not a new global hit subscriber. Price scope/table/lookup/storage before allocation or scan, and retain the scope through permitted released deliveries. Exact generated references are compiled from local keys; client strings cannot manufacture a result from another plan/copy/action.

Representative sequence:
- Action bite node points to the Physical damage producer, waits EffectsSettled.
- Branch EffectCompare { node_id: bite-node, effect_id: bite, metric: HpLost, comparison: Greater, value: Hp(0) }; existing Compare is equivalent only when that Action has exactly the qualifying bite damage.
- True action executes original Self heal step exactly once; false finite path has no heal.
- Flow result is the bite producer's aggregate actual_hp_loss (damage_runtime.rs:541/592), not contact/resolved target count or another damage action's result.
- Footprint recipe targets only the bite effect; Self heal step/value/type/source/carrier/cost/delay/use lifetime are untouched.

Reuse flow.rs:174–209 phase2 gate and apply_hp_delta's active-caster/max-HP rules. Fully absorbed/invalid/invulnerable/empty bite has0 HP loss and no heal. Partial absorption and any true HP loss permit one fixed heal. Kills are not required and HP-proportional lifesteal is not introduced. A dead/non-healable original caster does not revive or generate excess shield.

No per-target heal trigger is introduced: one settled producer occurrence, one branch and one Self-heal Action enforce the once-per-producer result. The representative contains one bite occurrence; a separately authored finite Repeat keeps its normal occurrence lifecycle instead of leaking a previous result. Existing official fetus/monk BasicAttackModifier placeholders remain unchanged; normal external authored representatives prove the new structure without inventing official content.

## 9. Staged mutations, restore and ownership (API02/API06)

### 9.1 Prospective mutation

Add a post-operation, pre-Continue/publication/commit effective-configuration validator inside World::prepare_application_scoped after inventory/provider reconciliation. Use an explicit affected employee set for relic/equipment/fragment/form/variant operations; transfers include previous and new owner. For correctness before optimization, scanning all actual equipped typed-relic employees is allowed only with charged bounded work and no no-relic allocation. Do not rely solely on validation inside handle_equip_relic because equipment/form/provider changes can activate conflicts too.

Current reusable entrypoints: relic_runtime.rs handle_equip_relic/unequip/corrupt; maintenance.rs handle_equip_item/unequip_item/equip_skill_fragment/transfer/select_form; helpers.rs plan_equip_item_payload and form compatibility; World application.rs's staged outer draft. Catalog references/graph conflicts are validated before committing the staged draft. Corrupt catalyst consumption and variant writes occur only on that draft. Early or late plan/admission failure drops draft and preserves slots, inventory, existing plan, resources, UUID/RNG and save state. No shared mutable compiled cache escapes the draft.

Use the same compiler for preview; direct conflict is a typed rejected prospective result, not a silent dormant row. Missing weapon keeps the existing provider-unavailable classification. Unconditional applicable rules must not create an unauthorized phantom provider to conceal that condition.

### 9.2 Save and restore

Keep OwnedRelic/RelicInventory and checkpoint/current/entry graph serialization as the persistent authority. Do not serialize compiled closures, Arc plans, activation pool/link or cache as trusted checkpoint fields.

Extend the existing validate_run_checkpoint_payload_with_seed / validate_persistent_owner_payload and continuation validator with pure effective-configuration validation for all relevant employees in current, entry and SavePoint payloads before live mutation. Rebuild selected plans from the pinned generation and actual restored copy/form/variant set. Same generation pinning and all-or-nothing restore remain. Fresh Core/Continue must not trust saved derived indices/fingerprints supplied by the client.

In-memory active battle clone, host retained Core detach/attach and permitted released payload retain their private owners normally. This does not add a new live-battle disk-resume schema or change active_run/continuation versions merely to save derived plans. Existing strict version mismatch rejection remains.

### 9.3 Lifetime matrix

| Owner | Creation and replacement | Mutable state / release |
|---|---|---|
| Generation descriptor/proof | winning catalog validation | immutable, held by validated generation |
| Prospective compiler candidate | query/mutation/restore validation | temporary, dropped on failure or explanation completion |
| Employee effective plan | actual current configuration | immutable, current binding replaced; old plan not a seed |
| Accepted runtime binding | admitted actor/provider/action | immutable strong owner through pending/active/Flow/delivery |
| Fixed support connection/pool | admitted source activation | value state inside cloned candidate; end by original source rules |
| Support attack projection | original AttackStart snapshot | immutable coefficient/source; no authority, drop after delivery/fact retention |
| Query DTO / observer | explicit read-only projection | owned bounded data/Weak where suitable; not a gameplay keepalive requirement |

Retain no reverse strong plan→BattleCore/World/activation edges. Existing fact/trace retention may prolong read-only provenance within its actual bounds; enumerate it in lifetime tests instead of claiming Arc count alone proves no leak.

## 10. Admission, clone and cost witness (API06)

Every new ownership shape implements CloneShape or a precise custom visitor. The existing Arc CloneShape meters only the handle; native retained visitors must explicitly price the immutable pointee and source strings. Shared pointees are counted under a documented owner/reference policy once per retained root, not silently omitted or multiplied inconsistently.

Work/admission order:
1. Checked borrowed definition/configuration preflight: source text bytes, candidate/node/edge bounds and possible combination work.
2. Charge scan/comparison before lookup/comparison, admit metadata/frontier/temporary graph before collection allocation.
3. Charge sorted canonical composition comparisons and wide arithmetic; admit clone/rewrite and source/catalog/Flow payload before copy.
4. Admit final immutable plan and runtime binding/recipient/member/contributor/DTO storage before creating it.
5. On plan replacement retain old owner plus candidate simultaneously for actual temporary peak; successful commit then releases replaced owners.
6. Recheck final retained/entity/work at existing World and native timestamp candidate commit points.

No finished helper cost is claimed yet. Implement named charge sites (proposed): relic_structure_preflight, relic_candidate_bound, relic_dependency_scan, relic_closure_visit, relic_rule_compare, relic_property_compose, relic_plan_copy, relic_plan_catalog, relic_plan_owner, relic_support_connection, relic_support_projection, relic_effective_dto. Test author derives exact/exact-minus-one from final actual visitor/helper and original resource scope, not these names or guessed constants. Existing ExprLimits, authoring work/storage and native runtime work/storage/cap values remain.

Native runtime_facts.rs candidate/timestamp paths already clone and atomically commit. Extend retained_runtime_fact_extent, runtime entity counts, source/damage snapshot retained_extent and controlled clone shape for new real owners. Saturated or disabled observation cannot skip gameplay admission or alter applicability. Existing fault injection/cancellation and before/after authoritative state witnesses are reused under Astra ownership; add narrow new seams only where a final helper boundary actually requires them.

No-relic fast path must avoid candidate graph/plan/source allocations and preserve original gameplay. Layout and added legitimate compile/query cost are still compared in PC/R05; do not promise cost neutrality before measurement.

## 11. Minimal illustrative strict RON and test-facing contract

These examples are the proposed syntax to be implemented and then bound by actual producer tests. They are not executable fixtures today. New IDs, S/duration/cost/penalty values are representative, not official balance. No expected oracle number is copied from a draft test plan.

~~~ron
(relics: [
  (id: "u09_explosion_seed", uuid: "a5090000-0000-0000-0000-000000000001",
   name: "Finite grant example", description: "One finite primitive at original cast",
   rarity: "probe", allow_duplicates: false, effects: [],
   rules: [
     (id: "grant_explosion",
      selector: (provider: (fragment_id: Some("u09_cast_fragment"),
                            form_id: Some("base")), effect: ()),
      when: Always,
      apply: Grant(InlineEffects(action_id: "bite", effects: [
        (id: "explosion", tags: ["u09_explosion"],
         op: Damage(amount: 40, damage_type: Physical))
      ])))
   ], dismantle_equipment_dust: 1),
  (id: "u09_explosion_boost", uuid: "a5090000-0000-0000-0000-000000000002",
   name: "Generated selector example", description: "Select the actual granted effect",
   rarity: "probe", allow_duplicates: false, effects: [],
   rules: [
     (id: "boost",
      selector: (provider: (), effect: (all_tags: ["u09_explosion"],
                                        op: Some(Damage),
                                        has_property: Some(DamageAmount))),
      when: Always,
      apply: Edit(property: DamageAmount, operation: Add(Hp(7))))
   ], dismantle_equipment_dust: 1)
])
~~~

A separate normal external skill defines original action bite and a real fragment/form with that provider; loader validates those exact references. The example's selector identifies an actual generated effect, never a tag union from its neighbors.

Recipe fragments to be placed in rules on exactly compatible authored original effects:

~~~ron
// SUPERSEDED-PENDING Porccubus shared2S example; not current authoring authority
apply: Recipe(FixedRecipientSupport(shield_initial_multiplier: Some(2)))

// new typed RegularBasic aura source; final state replaces coefficients
apply: Recipe(RegularBasicSupport(damage_percent: 40, speed_points: 20))
// same source's Corrupt rules use 80 and30; original base is20 and15.
// FixedRecipientSupport(None) is in the same atomic bundle for single membership.

// Physical bite only: heal/cost/channel are not selected by this predicate
selector: (provider: (), effect: (all_tags: ["u09_bite"],
                                  op: Some(Damage),
                                  has_property: Some(DamageFootprint)))
apply: Recipe(DamageFootprint(footprint: (include_anchor_tile: true,
                                         rows: ["XXX", "X@X", "XXX"])))
~~~

The footprint uses the actual TileRangePattern shape syntax and original projection/filter/carrier policy; this symmetric mask is an illustrative structural input, not an oracle. Both selector structs have serde(default) on optional/list fields. The first complete relic example is the minimum parser/plan connector. Source publication tests must prove strict unknown fields, unit/range/reference errors and generated-source paths before relying on it.

Independent test author API01–07 handoff:

| Pending API | Proposed concrete answer | Remaining actual binding |
|---|---|---|
| API01 | RelicRuleDef/condition/action/property/value/recipe enums above; capability table and RelicCompileDiagnostic | exact serde constructor defaults and producer source codes after implementation |
| API02 | compile_effective_plan, validate_effective_configuration, new RequestRelicEffectivePlan/DTO version1 | actual public routing and result conversions |
| API03 | existing Unit manual payload + SupportRecipientPolicy/FixedSupportConnection/typed sustained witness | actual eligibility/revision/terminal connection consumer |
| API04 | BasicAttackOccurrenceKind + SupportCoefficientScope + immutable SupportContributionSnapshot | actual source transport, legacy fallback and family resolution |
| API05 | per-effect EffectiveEffectExecution; Flow EffectCompare using existing HpLost and bounded effect-occurrence result | actual footprint consumer/catalog overlay and result scope |
| API06 | charged borrowed preflight + explicit retained visitors + existing fault scopes | exact final helper arithmetic/cost boundaries and narrow test access |
| API07 | new U09-only receipt/source/archive/label registry and exported typed public witness | execution registry is Root/test-harness preparation, not a product API or GEX-00 execution claim |

For initial independently written component tests use real validated builder/pinned generation and two distinct employees/copies/forms; no public executable plan constructor accepting forged fingerprints. For public tests use normal external RON→validation/registration→Event/reward grant→maintenance equip→targeted command→read-only explanation. Private boss/runtime helpers remain component evidence. FactSummary latest is not an all-hit log; use pre/post HP/pool, producer aggregate and retained typed trace under actual bounded export.

## 12. Proposed file ownership and implementation bundles

Sol retains all product files until an explicit whole-file handover. Astra writes every cfg(test) module/include/registration, including inside product files; Sol does not add those test registrations. No concurrent edit of the same file. Existing tests/helpers/assertions remain Astra-owned and unmodified by Sol.

Proposed new product files (currently absent, preimage is absence):
- core_enhanced/src/game/relic/structural.rs: strict compiled graph/input/key/plan, compiler interfaces.
- core_enhanced/src/game/relic/structural/capabilities.rs: single descriptor/preserve table.
- core_enhanced/src/game/relic/structural/closure.rs: finite proof/dependency/worklist.
- core_enhanced/src/game/relic/structural/compose.rs: checked composition/recipe bundles.
- core_enhanced/src/game/relic/structural/dto.rs: owned read-only version1 explanation.
- core_enhanced/src/game/battle/core/effective_skill.rs: private selected-plan/source/eligibility consumer adapter.

Proposed existing product edits, all within the archived438 preimage:
- Schema/publication: game/data/relic_data.rs, game/data/mod.rs, game/data/content_pack/mod.rs, game/data/skill_data.rs, game/value_expr/catalog.rs, game/ability/authoring.rs, game/ability/flow.rs, game/ability/sustained.rs, game/relic.rs, game/relic/plan.rs.
- Runtime binding/consumers: battle/core/{mod.rs,relic_numeric.rs,sim.rs,types.rs,basic_attack.rs,damage_runtime.rs,sustained.rs,runtime_facts.rs,value_expr_runtime.rs,commands.rs,build.rs}; battle/core/skill_runtime/{flow.rs,projectile.rs,area.rs}; battle/{runtime_facts.rs,damage.rs,skill_source.rs}.
- World/query/restore: game/world/{application.rs,relic_runtime.rs,maintenance.rs,helpers.rs,combat.rs,snapshot.rs,node_flow.rs,active_run/continuation.rs}; game/behavior.rs; application.rs. State.rs only if a derived runtime field truly needs an owner outside the existing cold branch; no trusted save-plan field.
- Source/admission visitors are changed beside their actual owner, not in an unmetered generic utility. Official game_resources balance/catalog placeholders are not product changes in this proposed atomic runtime bundle. Normal external examples/fixtures are test/public work, separately owned.

Minimize edits after implementation confirms the actual call graph; an additional product file requires Root's bundle scope, not a private expansion. These are proposed scopes, not permission to edit now.

Astra proposed new test paths and registrations are owned by its TEST_PLAN_001.json (independent expected values are not adopted here). Expected sequential registration handovers:
1. game/relic.rs for new structural test module after Sol's product module edits are closed.
2. battle/core/boss_runtime.rs for its test include only after Sol is idle for that whole file.
3. game/world/tests/mod.rs to Astra for new World test registration.
4. authoring/trial.rs after any required product API adaptation is closed.
Any other fixture/test registry follows the same explicit whole-file ownership transfer. Return the file with before/diff/after and writer0 before the next product edit.

Suggested connected product order within AG-CRSA-RUNTIME:
A. strict schema/capability/finite compiler and pure plan+DTO producer;
B. original/generated source/catalog/Flow binding and real effective consumers;
C. fixed support/regular/Monk, then World staged mutations/restore/admission;
D. Root reviews shared actual API/diff and transfers registration/test ownership.
No intermediate A/B/C is a completed runtime node. Build/check/focused execution begins only in a separate authorized bundle with all source writers0 and a sole runner.

## 13. Completion of this preparation and unresolved implementation evidence

Preparation closes only the current-source/preimage intake and concrete proposed interfaces. It does not close V01–17, AG-CRSA-RUNTIME, WP-CRSA-PUBLIC or WP-CRSA-VERIFY. Required new actual validation remains: strict producer/closure/numeric/recipe, connected consumer/target/source/Flow, normal public RON acquisition/query, save/lifetime/late-fault, impacted223+separate31, original PC/R05 and final nonparticipant whole audit.

A material post-freeze Porccubus policy conflict was found during baseline currentness collection and accepted by Root. No current executable claim is made for the superseded shared2S sections; their successor basis is pending. Compiler stratification, plan-local catalog factoring, exact runtime owner storage, finite Flow graft and final measured costs are proposed implementation obligations, not verified features. If actual implementation requires weaker preservation or a different supported semantic boundary, return that finding to Root without changing the policy/oracle.

Baseline reuse binds the independently audited U08 source and historical publication delta, and separately records the later material documentation-policy change; it does not imply old binaries/evidence prove new U09 execution or the changed shield policy. Existing R05 peak-only warning and shared-host/native historical binary limits remain explicit. Product/test/source writer0 and build/test/performance runner0 at handoff; successor policy/basis and the concrete implementation bundle are pending.
