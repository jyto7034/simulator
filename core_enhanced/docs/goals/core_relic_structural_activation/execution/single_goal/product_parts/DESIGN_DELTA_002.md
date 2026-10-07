# U09 personal-shield connection delta — WR-CRSA-005 / RSP-CRSA-003

Status: targeted P2G-20 preparation proposal for CHG-CRSA-001 / O20, not an implementation or CodeReviewBinding. Owner: existing Parent-confirmed gpt-6.1-sol/xhigh product role. Reviewed 2026-10-07 UTC. Product/test/fixture edits, builds and functional/performance executions: 0.

DESIGN_001 remains unchanged. Its Porccubus shared-single pool, shared-stage classification, range-out/rejoin membership, inherited 2S/4S and old shield-recast proposal are superseded by WR005. The other proposed interfaces remain reusable, including the separate fixed channel connection, immutable selected plan and effect-local Monk result. This delta addresses only the new personal-shield boundary. Root owns the successor review, freeze, plan and goal-set artifacts; old FPF/PP/GS/HA001 do not authorize execution.

## 1. Actual reusable code and missing connection

Actual current source remains the accepted 438-member / 4c7bbd317a9a12045267bd306057e771133096ccbccace6a51f484d23a46e2ac basis. The selected source references below are read directly; no whole-source rescan or duplicate preimage archive is required.

- SustainedEffectDef::Shield with shared_radius_units=None already creates one target-owned capacity/remaining pool. Record has caster, target, checked expiry, activation ordinal and its own finite source; refresh_aura_members uses [record.target] for this personal case. Absorption orders personal before shared, higher priority before lower, then older activation. Depletion/expiry ends one record. Reconcile detects unavailable caster/target, and clear_sustained handles battle termination.
- These leaves can implement independent personal pools and ordinary expiry/priority. They do not implement the new release batch or whole prior target-set replacement. begin_sustained allocates one record and does not locate a same-provider/copy/effect group. Reusing it in a public per-target loop alone cannot establish the required atomic replacement.
- The existing ShieldExpr consumer evaluates an amount per target and emits BeginSustained separately; the new source amount must not use that fanout unchanged. The generic shared path still uses Some(radius), dynamically refreshed membership and one shared balance. Preserve it as R12 behavior.
- Current employee_members checks same-owner, active combatant, employee_trauma, Employee source identity and authored world-distance. It does not itself enforce Side::Player. Side has Player/Opponent. RuntimeUnit has no Guest identity discriminator; neither core_enhanced/src nor game_server/src defines a Guest type/field. Root accepted qualification through the currently supported normal owned Employee path only. Do not manufacture a Guest fixture/type or claim an existing Guest API. If a later Guest uses that Employee representation, this eligibility boundary needs review.
- Existing source captures original fragment copy/form and equipment persistent copy. Source observations retain attribution but strip execution rights. independent_origin additionally removes action/use-control/Flow/reinput and can support private finite lifetime. It is currently used by segmented pools, not automatically by every plain Shield.

## 2. Proposed authored family, recipe and typed property

Introduce an explicit personal-grant operation; keep existing Sustained(Shield), ShieldExpr and shared definitions compatible. Proposed minimum interface, names subject to the later whole-file implementation handover:

~~~rust
pub struct PersonalShieldGrantDef {
    pub amount: AmountExpr,
    pub duration_ms: u32,
    pub priority: i32,
    pub recipients: PersonalShieldRecipientsDef,
    pub source_creation: Option<ShieldCreationModifierDef>,
}
pub enum PersonalShieldRecipientsDef {
    PlayerEmployeesAtRelease { radius_units: u32 },
    SelectedPlayerEmployeeAtRelease { authored_reach_units: u32 },
}
// New SkillEffectDef case:
PersonalShieldGrant { definition: PersonalShieldGrantDef }

// New U09 capability/property, distinct from shared-shield support:
RelicProperty::PersonalShieldInitialAmount
StructuralRecipeDef::PersonalShieldSingle { initial_amount: AmountExpr }
~~~

ShieldCreationModifierDef is a bounded, typed, authored numeric program using the same checked Add/Scale/Replace composition contract proposed in DESIGN_001; absent means identity. Its final exact serde shape must be shared with Astra before tests are authored. No creation bonus is inferred from equipment stats, incoming-damage modifiers or an undocumented current shield bonus field: no such current consumer was found. The compiled descriptor separately exposes original base amount, relic initial-amount edits, authored source-creation program and their order.

PersonalShieldInitialAmount is HP-capacity interpreted through actual ValueUnit::Magnitude and existing shield narrowing. It supports the approved typed Add/Scale/Replace operations. Validate final range against the existing positive Shield capacity and narrowing contract; invalid dynamic capacity/overflow/unit/budget is an error, never a silent saturating success. Existing ShieldExpr zero-skip behavior remains its own compatibility path. duration/priority and original cast cost/time/source are preserved by this recipe; it does not write arbitrary private fields.

The recipe atomically changes this family's area grant into one manually selected eligible employee and supplies its authored strengthened amount. It removes the editable area-grant property; original reach stays a read-only targeting constraint. It does not infer an amount from area target count, inherit 2S/4S, or change the fixed channel-aura recipe. An area modifier against a removed property follows the existing explicit predicate/dormancy versus unconditional invalid-write distinction.

Proposed public RON connection, illustrative representative numbers only, not official Porccubus balance:

~~~ron
(id: "shield", op: PersonalShieldGrant(definition: (
    amount: (moment: Snapshot, root: Const((value: 120, unit: Magnitude))),
    duration_ms: 5000, priority: 10,
    recipients: PlayerEmployeesAtRelease(radius_units: 2000000),
    source_creation: None,
)))
// A selected relic rule may use:
Recipe(PersonalShieldSingle(initial_amount:
    (moment: Snapshot, root: Const((value: 240, unit: Magnitude)))))
~~~

This is proposed serde syntax, not accepted current syntax or a parsed fixture. Normal/Corrupt amount values are explicit typed authored values. Official formula, coefficients, final range/duration/balance and corruption penalty remain unadopted; an independently authored original HealthCost is not an official penalty.

## 3. Release set and one source evaluation

Resolve readiness/preview/manual activation through the already proposed selected effective-plan binding. Both caster and recipients must meet the current real player/provider path: caster active Player combatant with validated actual provider/copy; recipient active Player owned Employee combatant, supported employee condition, original authored reach and current UnitInstanceId. Include eligible caster and both Ground/Platform. Do not substitute MobilityKind for tile kind, or broaden to every friendly combatant/summon/object. Reuse existing world-position distance units. Guest is unsupported by the current Employee qualification contract as above.

Freeze the actual sorted unique recipient set when this effect is released, not at preview, pause request acceptance or later aura refresh. Area grant scans eligible units once; selected grant revalidates the one admitted Unit target under original qualification/reach. Changes before release follow the admitted action's original validity/cancellation rules. After release no scans for new entrants, range reentry, moving caster or equipment-driven amplification create/refill another pool.

Create one source-scoped amount capture/evaluation, then relic initial-amount composition, then the authored source-creation program once; distribute that same final initialized capacity into separate value-state pools. Each target gets one new activation and remaining value, not a shared mutable Arc balance. Snapshot uses its captured source frame; Live builds the original permitted source frame at consumption/creation. Never reevaluate this new nonchannel grant on absorption or per recipient. Reject recipient-dependent requirements for this source-only amount family rather than silently rebinding them to caster; retain the existing bounded authored aggregates and valid source/result references where their real producer exists. Other ShieldExpr/Damage/Heal moment semantics remain unchanged.

Amount catalog, capture frontier and executor need explicit new operation dispatch. Current AmountPlanKind::Shield and narrowing are reusable; current U08 relic_numeric binding only applies Damage and is not a second shield modifier stage. Preserve original source actor/fragment/equipment/effect and selected generation. Action-result authority is validated before conversion to private finite lifetime.

## 4. Group identity and atomic whole-set recast

Within one BattleCore, proposed replacement identity is:

~~~rust
pub enum PersonalShieldProviderCopy {
    Fragment(Uuid),
    Equipment(Uuid), // actual persistent original copy, not definition UUID
}
pub struct PersonalShieldGrantGroupKey {
    pub provider: UnitInstanceId,
    pub copy: PersonalShieldProviderCopy,
    pub original_effect: EffectSite,
}
~~~

Use exact original EffectSite, including action and effect, plus the actual selected provider kind/copy; do not select a key merely because an equipment or fragment snapshot happens to be present. Different provider battle identities, copies and effects never replace each other. Cast sequence, new target list, pool activation, relic variant, selected plan fingerprint and current range are occurrence/provenance, not partitions that let a same-copy/effect recast leave old targets behind. Form/content generation remain validated pinned-owner/source facts; an original effect changed to a genuinely different site is distinguished by that exact site. Generated effects keep DESIGN_001's finite explicit generated-effect key/anchor, without collapsing sibling generated sites into an invented shared original effect.

FragmentSourceSnapshot.copy_uuid is available. EquipmentSourceSnapshot.persistent_copy_uuid is optional for scenario-created equipment. A normal copy-based grant requires the actual persistent provider copy; runtime_item_instance_id/definition_uuid cannot impersonate it. Any separate scenario-only adapter must expose its limited provider identity honestly and cannot count as the normal copy acceptance witness.

Add an optional immutable cold group/selected-plan provenance binding to new-family records; their pools/expiry remain independently cloned value state. A charged scan of sustained.records can identify all prior group activations without a second mutable group index. This scan must include prior targets outside the new range. Storage retained visitors, clone_shape and DTO projections account the binding and temporary replacement-ID list explicitly.

Proposed private executor boundary:

~~~rust
pub(super) fn prepare_personal_shield_grant(
    &self, selected: &ValidatedEffectiveEffect, source: &RuntimeFactSource,
    caster: UnitInstanceId, selected_target: Option<UnitInstanceId>, now_ms: u64,
) -> Result<PreparedPersonalShieldGrant, GameError>;
pub(super) fn apply_personal_shield_grant_in_candidate(
    &mut self, prepared: PreparedPersonalShieldGrant,
) -> Result<(), GameError>;
~~~

PreparedPersonalShieldGrant is privately constructible, tied to the current candidate/release source and pinned validated owner, with exact prior IDs, admitted new recipient records and common creation amount. It is not an authored/public capability and cannot be prepared against one World then applied to another.

Before mutation, charge the actual unit eligibility scan, prior-group scan/string comparisons, amount work, target sorting and every checked bound. Admit simultaneous live-old plus prepared-new peak storage, expiry entries, activation ordinal growth, command/observation vector capacity, source owners and each private resource/delivery. Overflow/cancellation/admission failure leaves previous state intact. Do not deduct old retained storage first to understate the preparation peak.

Apply the new batch and removal of every prior matching grant inside the existing World/native staged candidate; only its final successful commit publishes cost/resonance/RNG/ID, prior terminations and new pools. Helpers may perform removals then insertions in that discarded candidate, but observers cannot see a partial replacement. Existing commands.rs BeginSustained catches errors by breaking a loop; a new batch cannot rely on that silent path for failure atomicity. Propagate the typed failure through the existing runtime control/candidate boundary, including native execution, instead of treating an interrupted loop as completed replacement.

Successful release replaces the whole prior set, even where a particular old recipient is absent from the new set. A rejected/cancelled release preserves it. A valid normal area release includes its eligible caster; a valid selected release contains its revalidated one recipient. Therefore this contract needs no new normal empty-success rule. Missing provider/invalid caster/invalid selected target is a failed or cancelled admitted action, not a successful partial replacement.

## 5. Independent pool lifetime and unchanged shared protection

At creation first validate accepted action/provider/generation and current execution/resource authority. Then derive a read-only origin and reserve a private finite source for each pool, preserving actor/copy/original authored site and strong immutable selected plan. RuntimeFactSource::independent_origin is a reusable detachment pattern, not a public authority grant. Remove the producer use-control/Flow/action lease from the lasting shield; keep historical cast/Flow occurrence separately in cold observation if required. Closing one pool's private source must not close a sibling pool or the completed creating action.

Recipient depletion/expiry/withdraw/death/combat-invalid identity ends only that recipient's record. Range exit/reentry retains the same remaining/expiry; redeployment's new UnitInstanceId cannot inherit it. Provider withdraw/death/combat-invalid identity ends all its grant records; battle termination clears them. Existing reconcile covers active-state loss; the new-family predicate must also cover losing Employee/Player combat validity while still lifecycle-active, without changing generic R12 reconciliation. The scan/removal is bounded and charged. Preserve normal battle source-end timing.

Post-release Hard CC/Silence alone leaves these nonchannel pools intact while provider remains otherwise valid. New records use aura=None and shared_radius_units=None; stop_channel_with_reason already selects only aura records. They do not stop ordinary attacks or blocking. Pre-release cancellation still cancels the unreleased action and preserves old grants under the staged mutation contract.

Use existing personal absorption before shared, descending priority/older activation, independent source receipts and damage mitigation before shields. PayHealthCost bypasses absorption. Healing/Rest/time do not recharge. Actual recharge_sustained_shield currently selects the latest caster/source-SkillId nonsegmented shield, so merely retaining a historical cast snapshot would accidentally make a new grant eligible. Explicit family/group capability must exclude personal grants from that legacy selector without changing the selector/order/recharge of existing eligible generic records. Do not advertise RechargeShield authority for the new family. Keep existing generic and segmented-pool contracts unchanged. No conversion of Some(radius) shared records into personal records; no global aura filter, shared priority or old capacity semantics change.

## 6. Public witness and independent test connection

Extend the planned API01 capability/explain surface with the new operation, PersonalShieldInitialAmount and area/selected recipient kind. Extend API02 selected-plan targeting/query with current Employee eligibility/reach and pinned source identity. Extend API03 sustained witness through the existing sustained_effect_snapshot path with optional new-family provider-copy/effect group, release time, selected plan generation/fingerprint and pool activation/capacity/remaining/expiry/target. Existing ShieldPoolView already exposes the latter pool fields; do not describe multiple targets as one balance. Projections carry no action/resource authority.

Before Astra test implementation, hand over the final exact serde constructors/defaults and typed diagnostics, normal acquire/equip/target/activate public routing, optional witness fields and private preparation admission charge sites. Astra owns every test/helper/oracle and cfg(test) registration under explicit whole-file handover. This document writes no tests and fixes no expected values.

Independent observations needed from that new interface:

- Release set includes eligible caster and Player Ground/Platform employees; excludes currently unsupported/non-Employee identities and Opponent. Preview/paused acceptance does not freeze an area recipient list early.
- One source amount/modifier evaluation; recipient equipment cannot change it; independent damage drains only the damaged target's pool. Selected recipe produces its authored amount once, unrelated to former area member count.
- Late join/range exit/reentry/moving caster cannot refill; recipient terminal/deploy identity and provider-wide terminal differ; released CC/Silence leaves the nonchannel pool valid.
- Recast removes all same-group previous recipients, including outside new range, while different provider/copy/effect protection survives. Late admission/cancellation failure preserves old pools, resonance and RNG/ID exactly.
- Existing generic shared shield ordering/membership, explicit generic recharge, personal priority ties, cost bypass, normal basic/blocking and untransformed channel coefficients remain compatible.

No new test counts, PASS claims, cap exceptions or official content/balance are established here.

## 7. Exact reviewed references

Successor inputs (actual file SHA256, distinct from their formal semantic artifact digests):

| Path relative to core_relic_structural_activation | Bytes | SHA256 |
| --- | ---: | --- |
| P2G_20_SOURCE_PLAN_003.json | 31583 | bf856772700ccd36471c3f855548e811cf974ffb738262b92a8a54976872d1d9 |
| P2G_10_POLICY_003.json | 49285 | f3390c1d4c621cb1840a52d4148bd9d0df9c4f219c98dff728582270a16a1dc3 |
| P2G_10_RESULT_003.json | 2674 | 497cbb88a2a913da22236bbecba77893c201a568c2e62fe70af3f0bbb8f4b335 |
| POST_FREEZE_CHANGE_001.json | 55154 | 2e44269c1dae474202894a12dda36cffca3245085be4b962b9f2bb201f780c4d |

Formal selected refs: RSP-CRSA-003 / 748cbefa6b4b7124378c7a3d9545e0d47e4e8c7072558e92d76090b444880a02; WR-CRSA-005 / 2b202bfd70840d4fbc3c60915d8021e6f8b68e06390ba8ecbb5bf9eb9754a6ec; CHG-CRSA-001 / 2c8c13e91b13f7df16413edede903baa590aa09792413963a2aa3c0793cc696e. O20's prerequisite O10 resolution is RES-CRSA-CHG001-O10-001 / 7c7be8d42bd39c1dc3c2d0663aa6b5f8f6968e6fbdea09e2015355862d18b001. Their JSON includes full selected refs; this proposal does not create a replacement formal artifact.

Source paths below are relative to core_enhanced/src; ranges are actual reviewed lines, not claims of reading whole files:

| Source path | Reviewed lines | SHA256 |
| --- | --- | --- |
| game/ability.rs | 807–873 | 14fc93f673d70e0287f43d9006b2ad41be252259ad701cdc49d1f63464386312 |
| game/ability/sustained.rs | 25–47,310–337,449–512 | 5eb4bbb235041bf9d3b15d5a19811ff6db45f7a4a44864c09a14288c98cc001d |
| game/ability/authoring.rs | 1–72,140–197 | 3289f0b4dfe6b63b034cf6f33080bb1ee386a5d9a4036bfcec6d2d5781bbc577 |
| game/enums.rs | 24–50 | 85255ba6c40ba34e3d8aa3c546d5d54e187d84146347f0de7db04c0bc0971b58 |
| game/battle/types.rs | 268–366,409–466 | 82d0e4a9b3846fe92173b77ff7213e794053879e3c57f0ff8d5f1efa05e0bbe5 |
| game/battle/core/types.rs | 730–770 | ecd72efcbaa8d4af90a1f4c15200757cd930b72c99084cd27ebca6d4713f67b3 |
| game/battle/core/build.rs | 43–114,209–276 | ce874103aff24f26fa2c4e6bb21b1d0eeecfd6e94268781b3ab0ecdbdfa7dde7 |
| game/battle/core/sustained.rs | 1–108,276–596,650–797,832–962,1078–1134,1524–1615,1664–1735 | 0115ad2a39eb555eb48152d5d36338d78176981e96ef155a2c2691fb23aae80c |
| game/battle/core/sim.rs | 2840–2940,3200–3333 | 5083446fa80b665423ae70dcd162587278bab217ba428723df740b5c4668722e |
| game/battle/core/commands.rs | 599–695,884–941,1010–1047,1225–1276 | 9ad35869b2c348188566a156c56b74b95c1dd092e6813548c6189ee1d39d423c |
| game/battle/runtime_facts.rs | 110–241,520–560 | 4233c22361524a35b0bbe0e0d4b1dbb01a902d6d8a324296460314bc07b4982f |
| game/battle/core/runtime_facts.rs | 1609–1690,1910–1964,1978–2079 | fcbedf47e30f416945db46819b13a485e82b9093277f5862895ac7ef0cfdf310 |
| game/battle/core/damage_runtime.rs | 439–470,1053–1089 | 10d31895331bffe37ffe34c9809829671551b67a426803c8b925e1002519b1e9 |
| game/value_expr/mod.rs | 31–132,193–260 | 14a5b69f7be921001b686a548c7faae07e24e42e15095ae6881d773d264cd1be |
| game/value_expr/catalog.rs | 128–170,388–439 | 1a300e1174d258c7deac0b5dc6334715e51de546d5229251ce79f22686898b42 |
| game/battle/core/value_expr_runtime.rs | 844–908,1740–1822,2540–2582,2690–2748 | 8b53fa85fe8e00ab49e8df5de44c2b885c90fd26f56a42b14c42aa66d1765434 |
| game/skill_fragment.rs | 20–79 | 9d88d378471268ddab9abaee1ad1e0f68438dc44947184fc964116daca05ccfc |

Existing baseline/designed source preimage and reusable report refs remain BASELINE_001.json and DESIGN_001.md; this delta does not recopy them. Actual targeted readback at 2026-10-07T09:10:24.368869Z matched all 17 selected source files to their existing 438-member archive preimage and matched the four successor file refs above. Existing BASELINE_001 (141239B / 993f151eb72992e0df525e5ebfd316f69117a473e4c203189db71ad385862acd) and DESIGN_001 (57779B / fc1c83e122c87ae056b73918386f8811b8a892a69bacf29f62111925398b02d9) remained byte-identical. No relevant cargo/rustc/host/author-content/run.py process was visible in this PID namespace; this is not proof of whole shared-host quiescence. Only this preparation document was written, and its writer is returned to 0 when closed. Final return supplies its actual hash. Remaining policy-repair artifacts and implementation authorization belong to Root; this is not execution-ready or observed personal-shield behavior.
