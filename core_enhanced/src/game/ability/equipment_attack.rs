//! Authored basic-attack contributions attached to a real equipment copy.
//! Contributions are captured per attack. Landed statuses use the ordinary
//! sustained-effect owner; durable balances use the typed resource authority.
use super::{authoring::*, state_program::*};
use crate::game::{
    battle::types::BattleUnitThreatClass,
    data::{
        equipment_data::{EquipmentDatabase, EquipmentType},
        live_ron::LiveRonError,
        relic_data::RelicDatabase,
    },
    resource::{ResourceCatalog, ResourceLifetime, ResourceOwnerSelector},
    value_expr::{
        self as expr, AmountExpr, CompiledAmountPlan, DeliveryCapabilities, EvalMoment, ExprLimits,
        ValueRef, ValueUnit,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    mem::size_of,
    sync::Arc,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentAttackBinding {
    pub equipment: String,
    pub program: EquipmentAttackProgram,
    #[serde(skip)]
    pub(crate) source: Option<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentAttackProgram {
    pub reactions: Vec<EquipmentAttackReaction>,
    /// Percentage penetration for the wearer's direct Physical basic/skill damage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_physical_penetration_percent: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_physical_penetration_flat: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_magic_penetration_percent: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_magic_penetration_flat: Option<u32>,
    /// Elapsed uninterrupted regular basic-attack activity, independent of hits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuity: Option<EquipmentAttackContinuity>,
    /// Elapsed battle time since deployment or last regular basic release.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_gap: Option<EquipmentReleaseGap>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon_stages: Option<super::weapon_stages::EquipmentWeaponStages>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub magazine: Option<EquipmentMagazine>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cohort: Option<EquipmentCohort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub victory_forms: Option<super::equipment_forms::EquipmentVictoryForms>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death_burst: Option<super::death_burst::EquipmentDeathBurst>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bonus: Option<EquipmentAttackBonus>,
    /// Evaluated from the wearer's current HP when a regular attack starts.
    /// Unlike weapon damage/kill reactions this contribution accepts any slot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basic_speed_points: Option<AmountExpr>,
    /// Live percentage increase of the direct primary basic-attack component.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_damage_percent: Option<AmountExpr>,
    /// One live percentage for a finalized spatial damage judgment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_damage_percent: Option<AmountExpr>,
    /// Current other allied employees in a wearer-centered range supply the
    /// finite collection for this outgoing damage percentage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nearby_ally_damage: Option<EquipmentNearbyAllyDamage>,
    /// Pure condition and percentage captured once at each target strike.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conditional_damage: Option<super::conditional_damage::EquipmentConditionalDamage>,
    /// A landed basic strike applies this finite status once to each target.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_hit_status: Option<EquipmentHitStatus>,
    /// One settlement over actual hostile HP loss from a regular basic action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basic_lifesteal: Option<EquipmentBasicLifesteal>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentReleaseGap {
    pub per_second_percent: u32,
    pub max_percent: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentAttackContinuity {
    pub per_second_percent: u32,
    pub max_percent: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentNearbyAllyDamage {
    pub range: crate::game::battle::tile_range::TileRangePattern,
    pub percent: AmountExpr,
}
#[derive(Debug)]
pub(crate) struct CompiledNearbyAllyDamage {
    pub range: crate::game::battle::tile_range::TileRangePattern,
    pub percent: Arc<CompiledAmountPlan>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentBasicLifesteal {
    pub percent: u32,
    pub max_heal: u32,
}

#[derive(Debug)]
pub(crate) struct CompiledEquipmentBasicLifesteal {
    pub definition: EquipmentBasicLifesteal,
    pub source: Arc<EffectSource>,
}

/// A real hostile cohort activated independently of fragment shot resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentCohort {
    pub spawn: super::hostile_cohort::HostileCohortDef,
    pub prioritize: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounty: Option<CohortBounty>,
}

/// Pending and confirmed balances are owned by the same persistent equipment
/// copy. Each tier specifies the total flat Attack contribution at its threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CohortBounty {
    pub pending: crate::game::resource::ResourceId,
    pub confirmed: crate::game::resource::ResourceId,
    pub tiers: Vec<CohortBountyTier>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CohortBountyTier {
    pub threshold: u32,
    pub attack: u32,
}
impl CohortBounty {
    pub(crate) fn attack(&self, balance: u32) -> u32 {
        self.tiers.iter().rev().find(|tier| balance >= tier.threshold)
            .map_or(0, |tier| tier.attack)
    }
    fn validate(&self, resources: &ResourceCatalog) -> Result<(), (String, String)> {
        let fail = |field: &str, message: &str| (format!("program.cohort.bounty.{field}"), message.into());
        if self.tiers.is_empty() || self.tiers.len() > 32 {
            return Err(fail("tiers", "bounty requires 1..32 tiers"));
        }
        let mut previous = (0, 0);
        for (i, tier) in self.tiers.iter().enumerate() {
            if tier.threshold <= previous.0 || tier.attack <= previous.1 || tier.attack > i32::MAX as u32 {
                return Err(fail(&format!("tiers[{i}]"), "bounty thresholds and total Attack must increase from zero; Attack must fit i32"));
            }
            previous = (tier.threshold, tier.attack);
        }
        for (field, id, lifetime) in [
            ("pending", &self.pending, ResourceLifetime::Attempt),
            ("confirmed", &self.confirmed, ResourceLifetime::Run),
        ] {
            let declaration = resources.resources.iter().find(|d| &d.id == id)
                .ok_or_else(|| fail(field, "unknown bounty resource"))?;
            if declaration.owner != crate::game::resource::ResourceOwnerKind::EquipmentCopy
                || declaration.lifetime != lifetime || declaration.initial != 0 || declaration.cap == 0 {
                return Err(fail(field, "bounty requires EquipmentCopy, zero initial, positive cap and matching Attempt/Run lifetime"));
            }
            if lifetime == ResourceLifetime::Run && declaration.cap != previous.0 {
                return Err(fail(field, "confirmed cap must equal the last tier threshold"));
            }
        }
        Ok(())
    }
    fn retained_extent(&self) -> Option<usize> {
        self.pending.retained_text_capacity().checked_add(self.confirmed.retained_text_capacity())?
            .checked_add(self.tiers.capacity().checked_mul(size_of::<CohortBountyTier>())?)
    }
}

/// Copy-owned finite ammunition. Geometry belongs to the weapon's existing
/// projectile profile. Enhanced reward rounds are an optional directional-only extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentMagazine {
    pub capacity: u32,
    pub reload_ms: u32,
    pub normal_percent: u32,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "magazine_optional_field")]
    pub enhanced_primary_percent: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "magazine_optional_field")]
    pub enhanced_secondary_percent: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "magazine_optional_field")]
    pub enhanced_max_hits: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "magazine_optional_field")]
    pub reward_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentHitStatus {
    /// Omitted legacy inputs remain guaranteed. Every accepted target strike rolls once.
    #[serde(default = "guaranteed_hit_chance", skip_serializing_if = "is_guaranteed_hit_chance")]
    pub chance_percent: u8,
    /// Slow contributions are independent by default. A weapon can explicitly
    /// replace the same copy's movement slow while preserving other sources.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub refresh_movement_slow: bool,
    pub duration_ms: u32,
    pub profile: super::sustained::HitStatusProfile,
}

fn guaranteed_hit_chance() -> u8 { 100 }
fn is_guaranteed_hit_chance(value: &u8) -> bool { *value == 100 }

#[derive(Debug)]
pub(crate) struct CompiledEquipmentHitStatus {
    pub definition: EquipmentHitStatus,
    pub source: Arc<EffectSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentAttackReaction {
    pub id: String,
    pub event: EquipmentAttackEvent,
    /// Tested against the actual normal relic copy at attack admission. A
    /// conditional extra grant can compose with the unconditional base grant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_relic: Option<String>,
    pub body: Vec<StateOp>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EquipmentAttackEvent {
    PrimaryBasicKill {
        eligibility: StateKillEligibility,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        threat: Option<BattleUnitThreatClass>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentAttackBonus {
    pub amount: AmountExpr,
    pub secondary_percent: u32,
}

#[derive(Debug)]
pub(crate) struct CompiledEquipmentReaction {
    pub event: EquipmentAttackEvent,
    pub requires_relic: Option<String>,
    pub program: Arc<CompiledStateProgram>,
}

#[derive(Debug)]
pub(crate) struct CompiledDirectPenetration {
    pub damage_type: crate::game::battle::damage::DamageType,
    pub percent: u32,
    pub flat: u32,
    pub source: Arc<EffectSource>,
}

#[derive(Debug)]
pub(crate) struct CompiledEquipmentAttack {
    pub fingerprint: [u8; 32],
    pub direct_penetration: Vec<CompiledDirectPenetration>,
    pub continuity: Option<EquipmentAttackContinuity>,
    pub continuity_source: Arc<EffectSource>,
    pub release_gap: Option<EquipmentReleaseGap>,
    pub release_gap_source: Arc<EffectSource>,
    pub weapon_stages: Option<Arc<super::weapon_stages::CompiledWeaponStages>>,
    pub magazine: Option<EquipmentMagazine>,
    pub cohort: Option<EquipmentCohort>,
    pub victory_forms: Option<super::equipment_forms::EquipmentVictoryForms>,
    pub death_burst: Option<super::death_burst::EquipmentDeathBurst>,
    pub death_burst_source: Arc<EffectSource>,
    pub forms_source: Arc<EffectSource>,
    pub cohort_source: Arc<EffectSource>,
    pub magazine_source: Arc<EffectSource>,
    pub reactions: Vec<CompiledEquipmentReaction>,
    pub bonus: Option<Arc<CompiledAmountPlan>>,
    pub basic_speed_points: Option<Arc<CompiledAmountPlan>>,
    pub primary_damage_percent: Option<Arc<CompiledAmountPlan>>,
    pub area_damage_percent: Option<Arc<CompiledAmountPlan>>,
    pub nearby_ally_damage: Option<CompiledNearbyAllyDamage>,
    pub conditional_damage: Option<super::conditional_damage::CompiledConditionalDamage>,
    pub on_hit_status: Option<CompiledEquipmentHitStatus>,
    pub basic_lifesteal: Option<CompiledEquipmentBasicLifesteal>,
    pub secondary_percent: u32,
    pub retained_bytes: usize,
}

#[derive(Debug, Default)]
pub(crate) struct EquipmentAttackCatalog {
    pub has_direct_penetration: bool,
    pub has_continuity: bool,
    pub has_release_gap: bool,
    pub has_primary_damage: bool,
    pub has_area_damage: bool,
    pub has_nearby_ally_damage: bool,
    pub has_conditional_damage: bool,
    pub by_equipment: BTreeMap<uuid::Uuid, Arc<CompiledEquipmentAttack>>,
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 128 && id.trim() == id && !id.chars().any(char::is_control)
}

impl EquipmentAttackBinding {
    fn effect_source(&self, path: &str) -> EffectSource {
        let (file, base) = self.source.clone().unwrap_or_else(|| {
            (
                "typed:equipment-attacks".into(),
                "equipment_attacks[0]".into(),
            )
        });
        EffectSource {
            source_file: file,
            field_path: format!("{base}.program.{path}"),
            site: EffectSite {
                action: ActionSite {
                    definition_id: format!("equipment:{}", self.equipment).into(),
                    action_id: "basic_attack".into(),
                },
                effect_id: path.into(),
            },
            origin_definition_id: format!("equipment:{}", self.equipment).into(),
            origin: EffectOrigin::Original,
        }
    }

    pub(crate) fn compile(
        &self,
        resources: &ResourceCatalog,
        limits: &ExprLimits,
    ) -> Result<Arc<CompiledEquipmentAttack>, (String, String)> {
        let fail = |path: &str, reason: &str| (path.to_owned(), reason.to_owned());
        if !valid_id(&self.equipment) {
            return Err(fail("equipment", "invalid equipment identity"));
        }
        let definition = &self.program;
        for (field, value, max) in [
            ("direct_physical_penetration_percent", definition.direct_physical_penetration_percent, 100),
            ("direct_magic_penetration_percent", definition.direct_magic_penetration_percent, 100),
            ("direct_physical_penetration_flat", definition.direct_physical_penetration_flat, i32::MAX as u32),
            ("direct_magic_penetration_flat", definition.direct_magic_penetration_flat, i32::MAX as u32),
        ] {
            if value.is_some_and(|value| value > max) {
                return Err(fail(&format!("program.{field}"), &format!("direct penetration requires 0..{max}")));
            }
        }
        if let Some(clock) = definition.release_gap {
            for (field, value) in [("per_second_percent", clock.per_second_percent), ("max_percent", clock.max_percent)] {
                if value == 0 || value > 10000 {
                    return Err((format!("program.release_gap.{field}"), "release gap requires 1..10000 percent".into()));
                }
            }
        }
        if let Some(clock) = definition.continuity {
            for (field, value) in [("per_second_percent", clock.per_second_percent), ("max_percent", clock.max_percent)] {
                if !(1..=10_000).contains(&value) {
                    return Err((format!("program.continuity.{field}"), "attack continuity requires 1..10000 percent".into()));
                }
            }
        }
        if definition.reactions.len() > MAX_STATE_HANDLERS
            || (definition.reactions.is_empty()
                && definition.direct_physical_penetration_percent.is_none()
                && definition.direct_physical_penetration_flat.is_none()
                && definition.direct_magic_penetration_percent.is_none()
                && definition.direct_magic_penetration_flat.is_none()
                && definition.continuity.is_none()
                && definition.release_gap.is_none()
                && definition.bonus.is_none()
                && definition.basic_speed_points.is_none()
                && definition.primary_damage_percent.is_none()
                && definition.area_damage_percent.is_none()
                && definition.nearby_ally_damage.is_none()
                && definition.conditional_damage.is_none()
                && definition.on_hit_status.is_none()
                && definition.basic_lifesteal.is_none()
                && definition.magazine.is_none()
                && definition.cohort.is_none()
                && definition.victory_forms.is_none()
                && definition.weapon_stages.is_none()
                && definition.death_burst.is_none())
        {
            return Err(fail(
                "program",
                "attack program requires weapon stages, death burst, forms, cohort, magazine, speed, primary/area/nearby ally/conditional damage, direct physical penetration, bonus, hit status, lifesteal or 1..32 reactions",
            ));
        }
        crate::authoring::control::charge_serialized(self, "equipment_attack_compile")
            .map_err(|e| fail("program", e.code()))?;
        let check_resource = |resource: &crate::game::resource::ResourceId, owner| {
            crate::authoring::control::charge(
                resources.resources.len() as u64 + 1,
                "equipment_attack_resource",
            )
            .map_err(|e| fail("resource", e.code()))?;
            let declaration = resources
                .resources
                .iter()
                .find(|r| &r.id == resource)
                .ok_or_else(|| fail("resource", "unknown attack resource"))?;
            if owner != ResourceOwnerSelector::SourceEquipment {
                return Err(fail("owner", "attack callbacks require SourceEquipment"));
            }
            declaration
                .validate_selector(owner)
                .map_err(|e| ("owner".into(), e.to_string()))?;
            if !matches!(
                declaration.lifetime,
                ResourceLifetime::Run | ResourceLifetime::Attempt
            ) {
                return Err(fail(
                    "resource",
                    "attack callbacks require Run or Attempt resources",
                ));
            }
            Ok(())
        };
        // Forbid lifetime and local-field operations before sharing the state
        // compiler. No field can masquerade as a weapon's persistent balance.
        fn check_ops(ops: &[StateOp], depth: usize, count: &mut usize) -> Result<(), String> {
            *count = count
                .checked_add(ops.len())
                .ok_or("reaction operation overflow")?;
            if depth > MAX_STATE_DEPTH || *count > MAX_STATE_OPERATIONS {
                return Err("reaction operation budget exceeded".into());
            }
            for (i, op) in ops.iter().enumerate() {
                let result = match op {
                    StateOp::HealOwner { .. }
                    | StateOp::ApplyOwnerState { .. }
                    | StateOp::Resources { .. } => Ok(()),
                    StateOp::If {
                        then_ops, else_ops, ..
                    } => check_ops(then_ops, depth + 1, count)
                        .and_then(|()| check_ops(else_ops, depth + 1, count)),
                    _ => Err(
                        "equipment reaction has no local fields, lifetime or child invocation"
                            .into(),
                    ),
                };
                result.map_err(|e| format!("[{i}]: {e}"))?;
            }
            Ok(())
        }
        let mut reactions = Vec::with_capacity(definition.reactions.len());
        let mut names = BTreeSet::new();
        let mut retained_bytes = size_of::<CompiledEquipmentAttack>()
            + reactions.capacity() * size_of::<CompiledEquipmentReaction>();
        use crate::game::battle::damage::DamageType;
        let penetration_fields = [
            ("direct_physical_penetration_percent", definition.direct_physical_penetration_percent, DamageType::Physical, true),
            ("direct_physical_penetration_flat", definition.direct_physical_penetration_flat, DamageType::Physical, false),
            ("direct_magic_penetration_percent", definition.direct_magic_penetration_percent, DamageType::Magic, true),
            ("direct_magic_penetration_flat", definition.direct_magic_penetration_flat, DamageType::Magic, false),
        ];
        let mut direct_penetration = Vec::with_capacity(penetration_fields.iter().filter(|(_, value, _, _)| value.is_some()).count());
        retained_bytes += direct_penetration.capacity() * size_of::<CompiledDirectPenetration>();
        for (field, value, damage_type, is_percent) in penetration_fields {
            let Some(value) = value else { continue };
            let source = Arc::new(self.effect_source(field));
            retained_bytes += size_of::<EffectSource>() + 64
                + serde_json::to_vec(source.as_ref()).map_err(|e| fail(&format!("program.{field}"), &e.to_string()))?.len();
            direct_penetration.push(CompiledDirectPenetration {
                damage_type, percent: if is_percent { value } else { 0 },
                flat: if is_percent { 0 } else { value }, source,
            });
        }
        let basic_lifesteal = definition.basic_lifesteal.map(|definition| {
            if !(1..=100).contains(&definition.percent) {
                return Err(fail("program.basic_lifesteal.percent", "lifesteal percent requires 1..100"));
            }
            if !(1..=i32::MAX as u32).contains(&definition.max_heal) {
                return Err(fail("program.basic_lifesteal.max_heal", "lifesteal per-attack cap requires 1..i32::MAX"));
            }
            let source = Arc::new(self.effect_source("basic_lifesteal"));
            retained_bytes += size_of::<EffectSource>() + 64
                + serde_json::to_vec(source.as_ref())
                    .map_err(|e| fail("program.basic_lifesteal", &e.to_string()))?.len();
            Ok(CompiledEquipmentBasicLifesteal { definition, source })
        }).transpose()?;
        let weapon_stages = definition.weapon_stages.as_ref().map(|stages| {
            stages.validate()?;
            if definition.magazine.is_some() {
                return Err(fail("program.weapon_stages", "weapon stages and magazine cannot both own projectile delivery"));
            }
            let base_source = Arc::new(self.effect_source("weapon_stages.base"));
            let upgraded_source = Arc::new(self.effect_source("weapon_stages.upgraded"));
            retained_bytes += size_of::<super::weapon_stages::CompiledWeaponStages>() + 64;
            for source in [&base_source, &upgraded_source] {
                retained_bytes += size_of::<EffectSource>() + 64
                    + serde_json::to_vec(source.as_ref())
                        .map_err(|e| fail("program.weapon_stages", &e.to_string()))?.len();
            }
            Ok(Arc::new(super::weapon_stages::CompiledWeaponStages {
                definition: stages.clone(), base_source, upgraded_source,
            }))
        }).transpose()?;
        if let Some(burst) = &definition.death_burst {
            burst.validate()?;
            retained_bytes = retained_bytes.checked_add(burst.retained_extent()
                .ok_or_else(|| fail("program.death_burst", "burst storage overflow"))?)
                .ok_or_else(|| fail("program.death_burst", "burst storage overflow"))?;
        }
        if let Some(forms) = &definition.victory_forms {
            forms.validate(resources)?;
            retained_bytes = retained_bytes.checked_add(forms.retained_extent()
                .ok_or_else(|| fail("program.victory_forms", "form storage overflow"))?)
                .ok_or_else(|| fail("program.victory_forms", "form storage overflow"))?;
        }
        if let Some(cohort) = &definition.cohort {
            cohort.spawn.validate_fields().map_err(|(field, reason)| fail(&format!("program.cohort.spawn.{field}"), &reason))?;
            if let Some(bounty) = &cohort.bounty {
                bounty.validate(resources)?;
                retained_bytes = retained_bytes.checked_add(bounty.retained_extent()
                    .ok_or_else(|| fail("program.cohort.bounty", "bounty storage overflow"))?)
                    .ok_or_else(|| fail("program.cohort.bounty", "bounty storage overflow"))?;
            }
            retained_bytes = retained_bytes.checked_add(cohort.spawn.retained_extent()
                .ok_or_else(|| fail("program.cohort", "cohort storage overflow"))?)
                .ok_or_else(|| fail("program.cohort", "cohort storage overflow"))?;
        }
        let mut work = 0u64;
        let mut operations = 0;
        if let Some(magazine) = &definition.magazine {
            if !(1..=1000).contains(&magazine.capacity) {
                return Err(fail(
                    "program.magazine.capacity",
                    "magazine capacity requires 1..1000",
                ));
            }
            if magazine.reload_ms == 0 {
                return Err(fail(
                    "program.magazine.reload_ms",
                    "magazine reload requires positive milliseconds",
                ));
            }
            let enhanced_fields = [magazine.enhanced_primary_percent.is_some(),
                magazine.enhanced_secondary_percent.is_some(), magazine.enhanced_max_hits.is_some(),
                magazine.reward_family.is_some()];
            if enhanced_fields.iter().any(|v| *v) && !enhanced_fields.iter().all(|v| *v) {
                return Err(fail("program.magazine", "enhanced rounds require primary/secondary percent, hit limit and reward family together"));
            }
            for (field, value) in [
                ("normal_percent", Some(magazine.normal_percent)),
                ("enhanced_primary_percent", magazine.enhanced_primary_percent),
                ("enhanced_secondary_percent", magazine.enhanced_secondary_percent),
            ] {
                if value.is_some_and(|v| !(1..=10000).contains(&v)) {
                    return Err(fail(&format!("program.magazine.{field}"), "round percent requires 1..10000"));
                }
            }
            if magazine.enhanced_max_hits == Some(0) {
                return Err(fail("program.magazine.enhanced_max_hits", "round hit limit requires 1..255"));
            }
            if let Some(family) = &magazine.reward_family {
                if !valid_id(family) {
                    return Err(fail("program.magazine.reward_family", "invalid cohort reward family"));
                }
                retained_bytes += family.len();
            }
        }
        let mut death_burst_source = self.effect_source("death_burst");
        death_burst_source.site.action.action_id = "death_burst".into();
        let death_burst_source = Arc::new(death_burst_source);
        retained_bytes += size_of::<EffectSource>() + 64
            + serde_json::to_vec(death_burst_source.as_ref()).map_err(|e| fail("program.death_burst", &e.to_string()))?.len();
        let forms_source = Arc::new(self.effect_source("victory_forms"));
        retained_bytes += size_of::<EffectSource>() + 2 * size_of::<usize>()
            + serde_json::to_vec(forms_source.as_ref()).map_err(|e| fail("program.victory_forms", &e.to_string()))?.len();
        let cohort_source = Arc::new(self.effect_source("cohort"));
        retained_bytes += size_of::<EffectSource>() + 64
            + serde_json::to_vec(cohort_source.as_ref()).map_err(|e| fail("program.cohort", &e.to_string()))?.len();
        let release_gap_source = Arc::new(self.effect_source("release_gap"));
        retained_bytes += size_of::<EffectSource>() + 64
            + serde_json::to_vec(release_gap_source.as_ref()).map_err(|e| fail("program.release_gap", &e.to_string()))?.len();
        let continuity_source = Arc::new(self.effect_source("continuity"));
        retained_bytes += size_of::<EffectSource>() + 64
            + serde_json::to_vec(continuity_source.as_ref()).map_err(|e| fail("program.continuity", &e.to_string()))?.len();
        let magazine_source = Arc::new(self.effect_source("magazine"));
        retained_bytes += size_of::<EffectSource>()
            + 64
            + serde_json::to_vec(magazine_source.as_ref())
                .map_err(|e| fail("program.magazine", &e.to_string()))?
                .len();
        let on_hit_status = definition
            .on_hit_status
            .as_ref()
            .map(|status| -> Result<_, (String, String)> {
                status.profile.validate_authored_provenance()
                    .map_err(|reason| fail("program.on_hit_status.profile", &reason))?;
                if status.chance_percent > 100 {
                    return Err(fail("program.on_hit_status.chance_percent", "hit status chance must be from 0 to 100 percent"));
                }
                if status.refresh_movement_slow && !matches!(status.profile.kind, super::sustained::HitStatusKind::Slow { .. }) {
                    return Err(fail("program.on_hit_status.refresh_movement_slow", "movement slow refresh requires a Slow status"));
                }
                if status.duration_ms == 0 {
                    return Err(fail(
                        "program.on_hit_status.duration_ms",
                        "hit status requires positive duration",
                    ));
                }
                if matches!(
                    status.profile.kind,
                    super::sustained::HitStatusKind::StackDamageTaken { .. }
                ) {
                    return Err(fail(
                        "program.on_hit_status.profile.kind",
                        "fragment-owned stacks require a fragment source, not an equipment hit",
                    ));
                }
                status
                    .profile
                    .validate()
                    .map_err(|e| ("program.on_hit_status.profile".into(), e))?;
                let source = self.effect_source("on_hit_status");
                retained_bytes += size_of::<CompiledEquipmentHitStatus>()
                    + status
                        .profile
                        .retained_extent()
                        .ok_or_else(|| fail("program.on_hit_status", "status storage overflow"))?
                    + serde_json::to_vec(&source)
                        .map_err(|e| ("program.on_hit_status".into(), e.to_string()))?
                        .len();
                Ok(CompiledEquipmentHitStatus {
                    definition: status.clone(),
                    source: Arc::new(source),
                })
            })
            .transpose()?;
        for (i, reaction) in definition.reactions.iter().enumerate() {
            let path = format!("program.reactions[{i}]");
            if !valid_id(&reaction.id)
                || !names.insert(&reaction.id)
                || reaction
                    .requires_relic
                    .as_deref()
                    .is_some_and(|id| !valid_id(id))
            {
                return Err(fail(&path, "invalid or duplicate reaction identity/relic"));
            }
            check_ops(&reaction.body, 0, &mut operations)
                .map_err(|e| (format!("{path}.body"), e))?;
            let EquipmentAttackEvent::PrimaryBasicKill {
                eligibility,
                threat,
            } = reaction.event;
            let program = StateProgram {
                id: reaction.id.clone(),
                owns_use: false,
                end_on_source_exit: true,
                replacement: StateReplacement::ReplaceSameSource,
                basic_attack: None,
                fields: vec![],
                handlers: vec![StateHandler {
                    id: reaction.id.clone(),
                    event: StateEvent::BasicAttackKill {
                        primary_only: true,
                        once_per_action: true,
                        eligibility,
                        threat,
                    },
                    body: reaction.body.clone(),
                }],
            };
            program
                .check_resources(|op| check_resource(&op.resource, op.owner))
                .map_err(|(p, reason)| {
                    (
                        format!(
                            "{path}.body{}",
                            p.strip_prefix("handlers[0].body").unwrap_or(&p)
                        ),
                        reason,
                    )
                })?;
            let source = self.effect_source(&format!("reactions[{i}].body"));
            let program = program
                .compile_reaction(&source, limits)
                .map_err(|reason| (format!("{path}.body"), reason))?;
            retained_bytes += program.retained_bytes
                + 2 * size_of::<usize>()
                + reaction.requires_relic.as_ref().map_or(0, String::len);
            work += program.compile_work;
            reactions.push(CompiledEquipmentReaction {
                event: reaction.event,
                requires_relic: reaction.requires_relic.clone(),
                program,
            });
        }
        let bonus = definition
            .bonus
            .as_ref()
            .map(|bonus| {
                if bonus.amount.moment != EvalMoment::Snapshot || bonus.secondary_percent > 100 {
                    return Err(fail(
                        "program.bonus",
                        "bonus requires Snapshot and secondary percent 0..100",
                    ));
                }
                let mut capabilities = DeliveryCapabilities::constant_only();
                capabilities.resources = true;
                let plan = expr::compile(
                    &bonus.amount,
                    &self.effect_source("bonus.amount"),
                    capabilities,
                    limits,
                )
                .map_err(|e| ("program.bonus.amount".into(), e.to_string()))?;
                if plan.result_unit() != &ValueUnit::Magnitude
                    || plan.requirements().resolved_targets
                {
                    return Err(fail(
                        "program.bonus.amount",
                        "bonus requires Magnitude without target collections",
                    ));
                }
                for reference in &plan.requirements().references {
                    let ValueRef::Resource {
                        resource, owner, ..
                    } = reference
                    else {
                        return Err(fail(
                            "program.bonus.amount",
                            "bonus reads only captured equipment resources",
                        ));
                    };
                    check_resource(resource, *owner)
                        .map_err(|(_, reason)| ("program.bonus.amount".into(), reason))?;
                }
                retained_bytes += plan.cost().plan_retained_bytes
                    + size_of::<CompiledAmountPlan>()
                    + 2 * size_of::<usize>();
                work += plan.cost().compile_work;
                Ok(Arc::new(plan))
            })
            .transpose()?;
        let basic_speed_points = definition
            .basic_speed_points
            .as_ref()
            .map(|amount| {
                let path = "program.basic_speed_points";
                if amount.moment != EvalMoment::Live {
                    return Err(fail(
                        path,
                        "basic speed requires Live at regular attack start",
                    ));
                }
                let plan = expr::compile(
                    amount,
                    &self.effect_source("basic_speed_points"),
                    DeliveryCapabilities::constant_only(),
                    limits,
                )
                .map_err(|e| (path.into(), e.to_string()))?;
                if plan.result_unit() != &ValueUnit::AttackSpeedPoints
                    || plan.requirements().resolved_targets
                    || plan.requirements().references.iter().any(|r| {
                        !matches!(
                            r,
                            ValueRef::HitPoints {
                                subject: expr::ValueSubject::Source,
                                ..
                            }
                        )
                    })
                {
                    return Err(fail(
                        path,
                        "basic speed requires AttackSpeedPoints and only wearer HP references",
                    ));
                }
                retained_bytes += plan.cost().plan_retained_bytes
                    + size_of::<CompiledAmountPlan>()
                    + 2 * size_of::<usize>();
                work += plan.cost().compile_work;
                Ok(Arc::new(plan))
            })
            .transpose()?;
        let primary_damage_percent = definition.primary_damage_percent.as_ref().map(|amount| {
            let path = "program.primary_damage_percent";
            if amount.moment != EvalMoment::Live {
                return Err(fail(path, "primary damage requires Live at each direct primary strike"));
            }
            let plan = expr::compile(amount, &self.effect_source("primary_damage_percent"),
                DeliveryCapabilities { target_at_consume: true, ..DeliveryCapabilities::constant_only() }, limits)
                .map_err(|e| (path.into(), e.to_string()))?;
            if plan.result_unit() != &ValueUnit::Scalar || plan.requirements().resolved_targets
                || plan.requirements().references.iter().any(|r| !matches!(r,
                    ValueRef::HitPoints { subject: expr::ValueSubject::Target, .. })) {
                return Err(fail(path, "primary damage requires Scalar percent and only target HP references"));
            }
            retained_bytes += plan.cost().plan_retained_bytes + size_of::<CompiledAmountPlan>() + 2 * size_of::<usize>();
            work += plan.cost().compile_work;
            Ok(Arc::new(plan))
        }).transpose()?;
        let area_damage_percent = definition.area_damage_percent.as_ref().map(|amount| {
            let path = "program.area_damage_percent";
            if amount.moment != EvalMoment::Live {
                return Err(fail(path, "area damage requires Live at each finalized spatial damage batch"));
            }
            let plan = expr::compile(amount, &self.effect_source("area_damage_percent"),
                DeliveryCapabilities { resolved_targets_at_consume: true,
                    ..DeliveryCapabilities::constant_only() }, limits)
                .map_err(|e| (path.into(), e.to_string()))?;
            if plan.result_unit() != &ValueUnit::Scalar || !plan.requirements().references.is_empty()
                || plan.requirements().state_owner_dot_targets
                || plan.segments.iter().flat_map(|ops| ops.iter()).any(|op| matches!(op,
                    expr::CompiledOp::Aggregate { .. }) && !matches!(op,
                    expr::CompiledOp::Aggregate {
                        source: expr::FiniteSource::ResolvedTargets,
                        grouping: expr::Grouping::WholeSet,
                        reduction: expr::Reduction::Count,
                        outer_reduction: None, .. })) {
                return Err(fail(path, "area damage requires Scalar percent with only whole-set resolved target counts"));
            }
            retained_bytes += plan.cost().plan_retained_bytes + size_of::<CompiledAmountPlan>() + 2 * size_of::<usize>();
            work += plan.cost().compile_work;
            Ok(Arc::new(plan))
        }).transpose()?;
        let nearby_ally_damage = definition.nearby_ally_damage.as_ref().map(|policy| {
            let range_path = "program.nearby_ally_damage.range";
            if policy.range.rows.len() > 33 || policy.range.rows.iter().any(|r| r.len() > 33) {
                return Err(fail(range_path, "nearby ally range must fit 33 by 33 tiles"));
            }
            policy.range.validate().map_err(|reason| (range_path.into(), reason))?;
            let amount = &policy.percent;
            let path = "program.nearby_ally_damage.percent";
            if amount.moment != EvalMoment::Live {
                return Err(fail(path, "nearby ally damage requires Live at each outgoing damage application"));
            }
            let plan = expr::compile(amount, &self.effect_source("nearby_ally_damage.percent"),
                DeliveryCapabilities { resolved_targets_at_consume: true,
                    ..DeliveryCapabilities::constant_only() }, limits)
                .map_err(|e| (path.into(), e.to_string()))?;
            if plan.result_unit() != &ValueUnit::Scalar || !plan.requirements().references.is_empty()
                || plan.requirements().state_owner_dot_targets
                || plan.segments.iter().flat_map(|ops| ops.iter()).any(|op| matches!(op,
                    expr::CompiledOp::Aggregate { .. }) && !matches!(op,
                    expr::CompiledOp::Aggregate {
                        source: expr::FiniteSource::ResolvedTargets,
                        grouping: expr::Grouping::WholeSet,
                        reduction: expr::Reduction::Count,
                        outer_reduction: None, .. })) {
                return Err(fail(path, "nearby ally damage requires Scalar percent with only whole-set resolved target counts"));
            }
            retained_bytes += plan.cost().plan_retained_bytes + size_of::<CompiledAmountPlan>() + 2 * size_of::<usize>();
            work += plan.cost().compile_work;
            retained_bytes += policy.range.rows.capacity() * size_of::<String>()
                + policy.range.rows.iter().map(String::capacity).sum::<usize>();
            Ok(CompiledNearbyAllyDamage { range: policy.range.clone(), percent: Arc::new(plan) })
        }).transpose()?;
        let conditional_damage = definition.conditional_damage.as_ref().map(|definition| {
            let (compiled, bytes, cost) = definition.compile(&self.effect_source("conditional_damage"), limits)?;
            retained_bytes = retained_bytes.checked_add(bytes).ok_or_else(|| fail("program.conditional_damage", "conditional storage overflow"))?;
            work = work.checked_add(cost).ok_or_else(|| fail("program.conditional_damage", "conditional work overflow"))?;
            Ok::<_, (String, String)>(compiled)
        }).transpose()?;
        if retained_bytes > limits.max_retained_bytes || work > limits.max_compile_work {
            return Err(fail("program", "combined attack program budget exceeded"));
        }
        let bytes =
            serde_json::to_vec(definition).map_err(|e| ("program".into(), e.to_string()))?;
        Ok(Arc::new(CompiledEquipmentAttack {
            fingerprint: Sha256::digest(bytes).into(),
            direct_penetration,
            continuity: definition.continuity,
            continuity_source,
            release_gap: definition.release_gap,
            release_gap_source,
            weapon_stages,
            magazine: definition.magazine.clone(),
            cohort: definition.cohort.clone(),
            victory_forms: definition.victory_forms.clone(),
            death_burst: definition.death_burst.clone(),
            death_burst_source,
            forms_source,
            cohort_source,
            magazine_source,
            reactions,
            bonus,
            basic_speed_points,
            primary_damage_percent,
            area_damage_percent,
            nearby_ally_damage,
            conditional_damage,
            on_hit_status,
            basic_lifesteal,
            secondary_percent: definition.bonus.as_ref().map_or(0, |b| b.secondary_percent),
            retained_bytes,
        }))
    }
}

impl EquipmentAttackCatalog {
    pub(crate) fn compile(
        equipment: &EquipmentDatabase,
        resources: &ResourceCatalog,
        relics: &RelicDatabase,
        limits: &ExprLimits,
    ) -> Result<Self, LiveRonError> {
        let mut by_equipment = BTreeMap::new();
        let mut retained = 0usize;
        for (index, binding) in equipment.equipment_attacks.iter().enumerate() {
            let (file, path) = binding.source.clone().unwrap_or_else(|| {
                (
                    "data/equipments/base.ron".into(),
                    format!("equipment_attacks[{index}]"),
                )
            });
            let error = |field: &str, cause: String| {
                LiveRonError::validation(
                    &file,
                    "equipment_attacks",
                    Some(binding.equipment.clone()),
                    Some(format!("{path}.{field}")),
                    cause,
                )
            };
            crate::authoring::control::charge(
                equipment.items.len() as u64 + 1,
                "equipment_attack_weapon_reference",
            )
            .map_err(|e| error("equipment", format!("{e:?}")))?;
            let item = equipment.get_by_id(&binding.equipment).ok_or_else(|| {
                error(
                    "equipment",
                    "attack binding requires existing equipment".into(),
                )
            })?;
            let weapon_contribution = !binding.program.reactions.is_empty()
                || binding.program.bonus.is_some()
                || binding.program.basic_lifesteal.is_some()
                || binding.program.weapon_stages.is_some()
                || binding.program.magazine.is_some();
            if weapon_contribution
                && (item.equipment_type != EquipmentType::Weapon || item.weapon_profile.is_none())
            {
                return Err(error(
                    "equipment",
                    "damage and kill reactions require a weapon with a combat profile".into(),
                ));
            }
            if (weapon_contribution
                && item
                    .weapon_profile
                    .as_ref()
                    .is_some_and(|p| p.kill_growth.is_some()))
                || by_equipment.contains_key(&item.uuid)
            {
                return Err(error(
                    "equipment",
                    "duplicate attack binding or conflicting legacy weapon growth".into(),
                ));
            }
            if binding.program.basic_lifesteal.is_some()
                && item.triggered_effects.values().flatten().any(|effect|
                    matches!(effect.effect(), crate::game::stats::Effect::BasicLifesteal { .. }))
            {
                return Err(error("program.basic_lifesteal", "duplicate authored and native weapon lifesteal".into()));
            }
            if let Some(stages) = &binding.program.weapon_stages {
                let profile = item.weapon_profile.as_ref().expect("weapon contribution checked above");
                if profile.mark_weapon.is_none()
                    || !matches!(profile.action, crate::game::data::abnormality_data::BasicAttackAction::Damage)
                {
                    return Err(error("program.weapon_stages", "weapon stages require an explicit mark-weapon evolution and a single-damage weapon".into()));
                }
                for (name, shot) in [("base", &stages.base), ("upgraded", &stages.upgraded)] {
                    let mut candidate = crate::authoring::clone_shape::controlled_clone(profile)
                        .map_err(|e| error("program.weapon_stages", format!("{e:?}")))?;
                    shot.apply(&mut candidate);
                    candidate.try_validate_runtime_contract(&item.id)
                        .map_err(|e| error(&format!("program.weapon_stages.{name}.{}", e.field_suffix), e.cause))?;
                }
            }
            if let Some(magazine) = &binding.program.magazine {
                use crate::game::ability::{DeliveryDef, ProjectileHitPolicy};
                use crate::game::data::abnormality_data::BasicAttackAction;
                let valid = item.weapon_profile.as_ref().is_some_and(|p| {
                    match (&p.action, &p.delivery) {
                        (BasicAttackAction::Damage, DeliveryDef::Projectile { hit_policy: ProjectileHitPolicy::DirectionalCollision, .. }) => true,
                        (BasicAttackAction::Damage | BasicAttackAction::DamageArea { .. },
                            DeliveryDef::Projectile { hit_policy: ProjectileHitPolicy::TargetLocked, .. }) => magazine.reward_family.is_none(),
                        _ => false,
                    }
                });
                if !valid {
                    return Err(error("program.magazine", "magazine requires a single directional projectile or plain target-locked damage/area projectile; enhanced rounds require directional delivery".into()));
                }
            }
            for (i, reaction) in binding.program.reactions.iter().enumerate() {
                crate::authoring::control::charge(
                    reaction
                        .requires_relic
                        .as_ref()
                        .map_or(1, |_| relics.relics.len() as u64 + 1),
                    "equipment_attack_relic_reference",
                )
                .map_err(|e| error("program.reactions", format!("{e:?}")))?;
                if reaction
                    .requires_relic
                    .as_ref()
                    .is_some_and(|id| relics.get_by_id(id).is_none())
                {
                    return Err(error(
                        &format!("program.reactions[{i}].requires_relic"),
                        "unknown relic".into(),
                    ));
                }
            }
            let program = binding
                .compile(resources, limits)
                .map_err(|(p, cause)| error(&p, cause))?;
            let additional = program
                .retained_bytes
                .checked_add(128)
                .ok_or_else(|| error("program", "catalog storage overflow".into()))?;
            crate::authoring::control::admit(
                crate::authoring::control::Resource::MetadataBytes,
                retained,
                additional,
                "equipment_attack_catalog_storage",
            )
            .map_err(|e| error("program", format!("{e:?}")))?;
            retained = retained
                .checked_add(additional)
                .ok_or_else(|| error("program", "catalog storage overflow".into()))?;
            by_equipment.insert(item.uuid, program);
        }
        let has_primary_damage = by_equipment.values().any(|p| p.primary_damage_percent.is_some());
        let has_area_damage = by_equipment.values().any(|p| p.area_damage_percent.is_some());
        let has_nearby_ally_damage = by_equipment.values().any(|p| p.nearby_ally_damage.is_some());
        let has_conditional_damage = by_equipment.values().any(|p| p.conditional_damage.is_some());
        let has_release_gap = by_equipment.values().any(|p| p.release_gap.is_some());
        let has_continuity = by_equipment.values().any(|p| p.continuity.is_some());
        let has_direct_penetration = by_equipment.values().any(|p| !p.direct_penetration.is_empty());
        Ok(Self { by_equipment, has_primary_damage, has_area_damage, has_nearby_ally_damage, has_conditional_damage, has_continuity, has_release_gap, has_direct_penetration })
    }
}

// Preserve the pre-existing raw RON/JSON field representation while allowing
// the four enhanced-round fields to be absent together for a plain magazine.
mod magazine_optional_field {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde::de::{self, IntoDeserializer, Visitor};
    use std::{fmt, marker::PhantomData};
    pub fn serialize<T: Serialize, S: Serializer>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error> {
        match value { Some(value) => value.serialize(serializer), None => serializer.serialize_none() }
    }
    pub fn deserialize<'de, T: Deserialize<'de>, D: Deserializer<'de>>(deserializer: D) -> Result<Option<T>, D::Error> {
        struct Field<T>(PhantomData<T>);
        impl<'de, T: Deserialize<'de>> Visitor<'de> for Field<T> {
            type Value = Option<T>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str("a magazine scalar or optional scalar") }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> { T::deserialize(value.into_deserializer()).map(Some) }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> { T::deserialize(value.into_deserializer()).map(Some) }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> { T::deserialize(value.into_deserializer()).map(Some) }
            fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> { T::deserialize(value.into_deserializer()).map(Some) }
            fn visit_some<D: Deserializer<'de>>(self, value: D) -> Result<Self::Value, D::Error> { T::deserialize(value).map(Some) }
            fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> { Ok(None) }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> { Ok(None) }
        }
        deserializer.deserialize_any(Field(PhantomData))
    }
}
