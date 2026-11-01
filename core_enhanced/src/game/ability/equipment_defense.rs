//! Passive defense bound to an equipped copy; cooldowns belong to battle state.
use crate::game::data::{equipment_data::EquipmentDatabase, live_ron::LiveRonError};
mod flat;
mod conditional;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentDefenseBinding {
    pub equipment: String,
    pub program: EquipmentDefenseProgram,
    #[serde(skip)]
    pub(crate) source: Option<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EquipmentDefenseProgram {
    ConditionalReduction(super::conditional_damage::EquipmentConditionalDamage),
    WearerReduction { percent: crate::game::value_expr::AmountExpr },
    PhysicalBasicFlatReduction { amount: crate::game::value_expr::AmountExpr },
    ReadyWaitDamage { per_second_percent: u32, max_percent: u32, duration_ms: u32 },
    NearbyHostileDefense {
        range: crate::game::battle::tile_range::TileRangePattern,
        per_enemy: u32,
        max_bonus: u32,
    },
    EmployeeRevival {
        cumulative: crate::game::resource::ResourceId,
        battle_used: crate::game::resource::ResourceId,
        max_revivals: u32,
        restore_percent: u8,
        invulnerable_ms: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transform_into: Option<String>,
    },
    ChannelOpeningGuard { duration_ms: u32, reduction_percent: u8 },
    LastWill {
        range: crate::game::battle::tile_range::TileRangePattern,
        attack_percent: u32,
        damage_type: crate::game::battle::damage::DamageType,
    },
    IdleHpGuard { recharge_ms: u32, reduction_percent: u8 },
    RechargingSkillBarrier {
        recharge_ms: u32,
    },
    BasicAttackEvasion {
        chance_percent: u8,
    },
    DeferredDamage {
        defer_percent: u8,
        duration_ms: u32,
        tick_ms: u32,
        kill_clear_percent: u8,
    },
    OverhealReserve {
        max_hp_cap_percent: u8,
        trigger_hp_percent: u8,
    },
    AdaptiveBasicShield {
        max_hp_percent: u16,
        duration_ms: u32,
        cooldown_ms: u32,
        priority: i32,
    },
}
impl EquipmentDefenseProgram {
    pub(crate) fn validate(&self) -> Result<(), (&'static str, &'static str)> {
        match self {
            Self::ConditionalReduction(_) | Self::PhysicalBasicFlatReduction { .. } | Self::WearerReduction { .. } => Ok(()),
            Self::ReadyWaitDamage { per_second_percent, .. } if !(1..=10000).contains(per_second_percent) => Err(("ReadyWaitDamage.per_second_percent", "waiting rate must be from 1 to 10000 percent per second")),
            Self::ReadyWaitDamage { max_percent, .. } if !(1..=10000).contains(max_percent) => Err(("ReadyWaitDamage.max_percent", "damage cap must be from 1 to 10000 percent")),
            Self::ReadyWaitDamage { duration_ms: 0, .. } => Err(("ReadyWaitDamage.duration_ms", "damage window must be positive milliseconds")),
            Self::ReadyWaitDamage { .. } => Ok(()),
            Self::NearbyHostileDefense { per_enemy, .. } if *per_enemy == 0 || *per_enemy > i32::MAX as u32 => Err(("NearbyHostileDefense.per_enemy", "per-enemy defense must be from 1 to i32::MAX")),
            Self::NearbyHostileDefense { max_bonus, .. } if *max_bonus == 0 || *max_bonus > i32::MAX as u32 => Err(("NearbyHostileDefense.max_bonus", "defense cap must be from 1 to i32::MAX")),
            Self::NearbyHostileDefense { range, .. } => {
                if range.rows.len() > 33 || range.rows.iter().any(|row| row.len() > 33) {
                    return Err(("NearbyHostileDefense.range", "nearby range must fit 33 by 33 tiles"));
                }
                range.validate().map_err(|_| ("NearbyHostileDefense.range", "nearby defense requires a valid anchored tile pattern"))
            },
            Self::EmployeeRevival { max_revivals: 0, .. } => Err(("EmployeeRevival.max_revivals", "revival limit must be positive")),
            Self::EmployeeRevival { restore_percent, .. } if !(1..=100).contains(restore_percent) => Err(("EmployeeRevival.restore_percent", "restored HP percentage must be from 1 to 100")),
            Self::EmployeeRevival { invulnerable_ms: 0, .. } => Err(("EmployeeRevival.invulnerable_ms", "damage immunity duration must be positive")),
            Self::EmployeeRevival { cumulative, battle_used, .. } if cumulative == battle_used => Err(("EmployeeRevival.battle_used", "run and attempt resources must be distinct")),
            Self::EmployeeRevival { transform_into: Some(id), max_revivals, .. }
                if id.trim().is_empty() || *max_revivals == u32::MAX => Err(("EmployeeRevival.transform_into", "transformation requires a definition and room for one final trigger")),
            Self::EmployeeRevival { .. } => Ok(()),
            Self::ChannelOpeningGuard { duration_ms: 0, .. } => Err((
                "ChannelOpeningGuard.duration_ms", "opening duration must be positive milliseconds")),
            Self::ChannelOpeningGuard { reduction_percent, .. } if !(1..=100).contains(reduction_percent) => Err((
                "ChannelOpeningGuard.reduction_percent", "opening reduction must be from 1 to 100")),
            Self::LastWill { attack_percent, .. } if !(1..=10000).contains(attack_percent) => {
                Err((
                    "LastWill.attack_percent",
                    "last will percent requires 1..10000",
                ))
            }
            Self::LastWill { range, .. } => {
                if range.rows.len() > 33 || range.rows.iter().any(|row| row.len() > 33) {
                    return Err(("LastWill.range", "last will range must fit 33 by 33 tiles"));
                }
                range.validate().map_err(|_| {
                    (
                        "LastWill.range",
                        "last will requires a valid nonempty anchored tile pattern",
                    )
                })
            }

            Self::DeferredDamage { defer_percent, .. }
                if *defer_percent == 0 || *defer_percent > 100 =>
            {
                Err((
                    "DeferredDamage.defer_percent",
                    "defer percentage must be from 1 to 100",
                ))
            }
            Self::DeferredDamage {
                kill_clear_percent, ..
            } if *kill_clear_percent > 100 => Err((
                "DeferredDamage.kill_clear_percent",
                "clear percentage must be from 0 to 100",
            )),
            Self::DeferredDamage {
                duration_ms,
                tick_ms,
                ..
            } if *duration_ms == 0
                || *tick_ms == 0
                || duration_ms % tick_ms != 0
                || duration_ms / tick_ms > 1000 =>
            {
                Err((
                    "DeferredDamage.tick_ms",
                    "positive duration must be an exact multiple of tick, with 1 to 1000 ticks",
                ))
            }
            Self::OverhealReserve {
                max_hp_cap_percent, ..
            } if *max_hp_cap_percent == 0 || *max_hp_cap_percent > 100 => Err((
                "OverhealReserve.max_hp_cap_percent",
                "reserve cap must be from 1 to 100",
            )),
            Self::OverhealReserve {
                trigger_hp_percent, ..
            } if *trigger_hp_percent == 0 || *trigger_hp_percent > 100 => Err((
                "OverhealReserve.trigger_hp_percent",
                "trigger percentage must be from 1 to 100",
            )),
            Self::IdleHpGuard { recharge_ms: 0, .. } => Err(("IdleHpGuard.recharge_ms", "idle recharge must be positive milliseconds")),
            Self::IdleHpGuard { reduction_percent, .. } if !(1..=100).contains(reduction_percent) => Err(("IdleHpGuard.reduction_percent", "idle guard reduction must be from 1 to 100")),
            Self::IdleHpGuard { .. } => Ok(()),
            Self::RechargingSkillBarrier { recharge_ms: 0 } => Err((
                "RechargingSkillBarrier.recharge_ms",
                "recharge must be positive milliseconds",
            )),
            Self::BasicAttackEvasion { chance_percent } if *chance_percent > 100 => Err((
                "BasicAttackEvasion.chance_percent",
                "evasion chance must be an integer percentage from 0 to 100",
            )),
            Self::AdaptiveBasicShield { max_hp_percent, .. }
                if *max_hp_percent == 0 || *max_hp_percent > 10000 =>
            {
                Err((
                    "AdaptiveBasicShield.max_hp_percent",
                    "shield percentage must be from 1 to 10000",
                ))
            }
            Self::AdaptiveBasicShield { duration_ms: 0, .. } => Err((
                "AdaptiveBasicShield.duration_ms",
                "shield duration must be positive milliseconds",
            )),
            Self::AdaptiveBasicShield { cooldown_ms: 0, .. } => Err((
                "AdaptiveBasicShield.cooldown_ms",
                "shield cooldown must be positive milliseconds",
            )),
            Self::ChannelOpeningGuard { .. }
            | Self::RechargingSkillBarrier { .. }
            | Self::BasicAttackEvasion { .. }
            | Self::AdaptiveBasicShield { .. }
            | Self::OverhealReserve { .. }
            | Self::DeferredDamage { .. } => Ok(()),
        }
    }
}

#[derive(Debug)]
pub(crate) struct CompiledEquipmentDefense {
    pub conditional_reduction: Option<super::conditional_damage::CompiledConditionalDamage>,
    pub incoming_flat: Option<Arc<crate::game::value_expr::CompiledAmountPlan>>,
    pub wearer_reduction: Option<Arc<crate::game::value_expr::CompiledAmountPlan>>,
    pub revival_source: Option<Arc<super::authoring::EffectSource>>,
    pub transformation_source: Option<Arc<super::authoring::EffectSource>>,
    pub transformation_uuid: Option<uuid::Uuid>,
    pub last_will_source: Option<Arc<super::authoring::EffectSource>>,
    pub deferred_source: Option<Arc<super::authoring::EffectSource>>,
    pub reserve_source: Option<Arc<super::authoring::EffectSource>>,
    pub idle_guard_source: Option<Arc<super::authoring::EffectSource>>,
    pub adaptive_source: Option<Arc<super::authoring::EffectSource>>,
    pub program: EquipmentDefenseProgram,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
#[derive(Debug, Default)]
pub(crate) struct EquipmentDefenseCatalog {
    pub has_conditional_reduction: bool,
    pub has_incoming_flat: bool,
    pub has_wearer_reduction: bool,
    pub has_ready_wait: bool,
    pub has_nearby_defense: bool,
    pub has_revival: bool,
    pub has_last_will: bool,
    pub has_idle_guard: bool,
    pub has_adaptive: bool,
    pub has_reserve: bool,
    pub has_deferred: bool,
    pub has_channel_opening: bool,
    pub by_equipment: BTreeMap<uuid::Uuid, Arc<CompiledEquipmentDefense>>,
}
impl EquipmentDefenseCatalog {
    pub(crate) fn compile(equipment: &EquipmentDatabase, resources: &crate::game::resource::ResourceCatalog, abnormalities: &crate::game::data::abnormality_data::AbnormalityDatabase, limits: &crate::game::value_expr::ExprLimits) -> Result<Self, LiveRonError> {
        let mut by_equipment = BTreeMap::new();
        let mut retained = 0usize;
        let mut has_incoming_flat = false;
        let mut has_wearer_reduction = false;
        let mut has_conditional_reduction = false;
        let mut has_ready_wait = false;
        let mut has_nearby_defense = false;
        let mut has_revival = false;
        let mut has_last_will = false;
        let mut has_idle_guard = false;
        let mut has_adaptive = false;
        let mut has_reserve = false;
        let mut has_deferred = false;
        let mut has_channel_opening = false;
        for (index, binding) in equipment.equipment_defenses.iter().enumerate() {
            let (file, path) = binding.source.clone().unwrap_or_else(|| {
                (
                    "data/equipments/base.ron".into(),
                    format!("equipment_defenses[{index}]"),
                )
            });
            let error = |field: &str, cause: String| {
                LiveRonError::validation(
                    &file,
                    "equipment_defenses",
                    Some(binding.equipment.clone()),
                    Some(format!("{path}.{field}")),
                    cause,
                )
            };
            crate::authoring::control::charge(
                equipment.items.len() as u64 + 1,
                "equipment_defense_reference",
            )
            .map_err(|e| error("equipment", format!("{e:?}")))?;
            let item = equipment
                .get_by_id(&binding.equipment)
                .ok_or_else(|| error("equipment", "defense requires existing equipment".into()))?;
            if by_equipment.contains_key(&item.uuid) {
                return Err(error("equipment", "duplicate defense binding".into()));
            }
            binding
                .program
                .validate()
                .map_err(|(p, reason)| error(&format!("program.{p}"), reason.into()))?;
            if matches!(binding.program, EquipmentDefenseProgram::LastWill { .. })
                && item.equipment_type != crate::game::data::equipment_data::EquipmentType::Armor
            {
                return Err(error("equipment", "last will requires armor".into()));
            }
            let (incoming_flat, wearer_reduction) = binding.compile_wearer_amounts(limits)
                .map_err(|(p, reason)| error(&p, reason))?;
            has_incoming_flat |= incoming_flat.is_some();
            has_wearer_reduction |= wearer_reduction.is_some();
            let conditional = binding.compile_conditional_reduction(limits)
                .map_err(|(p, reason)| error(&p, reason))?;
            let (conditional_reduction, conditional_bytes) = match conditional {
                Some((plan, bytes, work)) => {
                    crate::authoring::control::charge(work, "conditional_defense_compile")
                        .map_err(|e| error("program", format!("{e:?}")))?;
                    // Include the retained authored AST, in addition to compiled plans.
                    crate::authoring::control::charge_serialized(&binding.program, "conditional_defense_authored_copy")
                        .map_err(|e| error("program", format!("{e:?}")))?;
                    let authored = serde_json::to_vec(&binding.program)
                        .map_err(|e| error("program", e.to_string()))?.len();
                    let bytes = bytes.checked_add(authored.checked_mul(2)
                        .ok_or_else(|| error("program", "conditional storage overflow".into()))?)
                        .ok_or_else(|| error("program", "conditional storage overflow".into()))?;
                    if bytes > limits.max_retained_bytes {
                        return Err(error("program.ConditionalReduction", "combined conditional defense storage exceeds limit".into()));
                    }
                    (Some(plan), bytes)
                }
                None => (None, 0),
            };
            has_conditional_reduction |= conditional_reduction.is_some();
            let mut transformation_uuid = None;
            if let EquipmentDefenseProgram::EmployeeRevival { cumulative, battle_used, max_revivals, transform_into, .. } = &binding.program {
                if let Some(id) = transform_into {
                    crate::authoring::control::charge(abnormalities.items.len() as u64, "metamorphosis_definition_reference")
                        .map_err(|e| error("program.EmployeeRevival.transform_into", format!("{e:?}")))?;
                    transformation_uuid = Some(abnormalities.get_by_id(id).ok_or_else(||
                        error("program.EmployeeRevival.transform_into", "transformation requires an existing abnormality definition".into()))?.uuid);
                }
                let required_cap = max_revivals.checked_add(u32::from(transform_into.is_some()))
                    .ok_or_else(|| error("program.EmployeeRevival.max_revivals", "trigger count overflow".into()))?;
                use crate::game::resource::{ResourceOwnerKind, ResourceLifetime};
                if item.equipment_type != crate::game::data::equipment_data::EquipmentType::Accessory {
                    return Err(error("equipment", "employee revival requires accessory".into()));
                }
                for (id, lifetime, field) in [(cumulative, ResourceLifetime::Run, "cumulative"), (battle_used, ResourceLifetime::Attempt, "battle_used")] {
                    crate::authoring::control::charge(resources.resources.len() as u64, "revival_resource_reference")
                        .map_err(|e| error("program.EmployeeRevival", format!("{e:?}")))?;
                    let valid = resources.resources.iter().find(|d| &d.id == id).is_some_and(|d|
                        d.owner == ResourceOwnerKind::Employee && d.lifetime == lifetime && d.initial == 0
                        && if lifetime == ResourceLifetime::Run { d.cap >= required_cap } else { d.cap == 1 });
                    if !valid { return Err(error(&format!("program.EmployeeRevival.{field}"),
                        format!("requires Employee {lifetime:?} resource with initial 0 and compatible cap"))); }
                }
            }
            has_ready_wait |= matches!(binding.program, EquipmentDefenseProgram::ReadyWaitDamage { .. });
            has_nearby_defense |= matches!(binding.program, EquipmentDefenseProgram::NearbyHostileDefense { .. });
            has_channel_opening |= matches!(binding.program, EquipmentDefenseProgram::ChannelOpeningGuard { .. });
            let field = match binding.program {
                EquipmentDefenseProgram::ConditionalReduction(_) => "ConditionalReduction",
                EquipmentDefenseProgram::WearerReduction { .. } => "WearerReduction",
                EquipmentDefenseProgram::PhysicalBasicFlatReduction { .. } => "PhysicalBasicFlatReduction",
                EquipmentDefenseProgram::ReadyWaitDamage { .. } => "ReadyWaitDamage",
                EquipmentDefenseProgram::NearbyHostileDefense { .. } => "NearbyHostileDefense",
                EquipmentDefenseProgram::EmployeeRevival { .. } => "EmployeeRevival",
                EquipmentDefenseProgram::ChannelOpeningGuard { .. } => "ChannelOpeningGuard",
                EquipmentDefenseProgram::LastWill { .. } => "LastWill",
                EquipmentDefenseProgram::IdleHpGuard { .. } => "IdleHpGuard",
                EquipmentDefenseProgram::DeferredDamage { .. } => "DeferredDamage",
                EquipmentDefenseProgram::OverhealReserve { .. } => "OverhealReserve",
                EquipmentDefenseProgram::RechargingSkillBarrier { .. } => {
                    "RechargingSkillBarrier.recharge_ms"
                }
                EquipmentDefenseProgram::BasicAttackEvasion { .. } => {
                    "BasicAttackEvasion.chance_percent"
                }
                EquipmentDefenseProgram::AdaptiveBasicShield { .. } => "AdaptiveBasicShield",
            };
            let field_path = format!("{path}.program.{field}");
            let adaptive_bytes = if matches!(
                binding.program,
                EquipmentDefenseProgram::AdaptiveBasicShield { .. }
                    | EquipmentDefenseProgram::EmployeeRevival { .. }
                    | EquipmentDefenseProgram::LastWill { .. }
                    | EquipmentDefenseProgram::OverhealReserve { .. }
                    | EquipmentDefenseProgram::DeferredDamage { .. }
                    | EquipmentDefenseProgram::IdleHpGuard { .. }
            ) {
                std::mem::size_of::<super::authoring::EffectSource>()
                    + 64
                    + file.len()
                    + field_path.len()
                    + binding.equipment.len() * 2
                    + 32
            } else {
                0
            };
            let range_bytes =
                if let EquipmentDefenseProgram::LastWill { range, .. } | EquipmentDefenseProgram::NearbyHostileDefense { range, .. } = &binding.program {
                    crate::game::relic::structural::support::pattern_extent(range)
                        .ok_or_else(|| error("program", "range storage overflow".into()))?
                } else {
                    0
                };
            let revival_bytes = if let EquipmentDefenseProgram::EmployeeRevival { cumulative, battle_used, transform_into, .. } = &binding.program {
                cumulative.as_str().len() + battle_used.as_str().len() + transform_into.as_ref().map_or(0, |id| id.len()
                    + std::mem::size_of::<super::authoring::EffectSource>() + file.len() + field_path.len() + binding.equipment.len() * 2 + 128)
            } else { 0 };
            let flat_bytes = if let Some(plan) = incoming_flat.as_ref().or(wearer_reduction.as_ref()) {
                crate::authoring::control::charge_serialized(&binding.program, "incoming_flat_authored_copy")
                    .map_err(|e| error("program", format!("{e:?}")))?;
                let authored = serde_json::to_vec(&binding.program)
                    .map_err(|e| error("program", e.to_string()))?.len();
                plan.cost().plan_retained_bytes.checked_add(std::mem::size_of::<crate::game::value_expr::CompiledAmountPlan>() + 64)
                    .and_then(|n| n.checked_add(plan.cost().nodes.checked_mul(std::mem::size_of::<crate::game::value_expr::ValueExpr>() + 64)?))
                    .and_then(|n| n.checked_add(plan.cost().authored_members.checked_mul(std::mem::size_of::<crate::game::value_expr::AuthoredMember>() + 128)?))
                    .and_then(|n| n.checked_add(authored.checked_mul(2)?))
                    .ok_or_else(|| error("program", "fixed reduction storage overflow".into()))?
            } else { 0 };
            let additional = conditional_bytes + flat_bytes + revival_bytes + range_bytes
                + adaptive_bytes
                + std::mem::size_of::<CompiledEquipmentDefense>()
                + 128
                + binding.equipment.len()
                + file.len()
                + field_path.len();
            crate::authoring::control::admit(
                crate::authoring::control::Resource::MetadataBytes,
                retained,
                additional,
                "equipment_defense_catalog_storage",
            )
            .map_err(|e| error("program", format!("{e:?}")))?;
            retained = retained
                .checked_add(additional)
                .ok_or_else(|| error("program", "catalog storage overflow".into()))?;
            let revival_source =
                matches!(binding.program, EquipmentDefenseProgram::EmployeeRevival { .. }).then(|| {
                    use super::authoring::{ActionSite, EffectSite, EffectSource};
                    Arc::new(EffectSource {
                        source_file: file.clone(),
                        field_path: field_path.clone(),
                        site: EffectSite {
                            action: ActionSite {
                                definition_id: binding.equipment.clone().into(),
                                action_id: "employee-revival".into(),
                            },
                            effect_id: "restore".into(),
                        },
                        origin_definition_id: binding.equipment.clone().into(),
                        origin: Default::default(),
                    })
                });
            let transformation_source = transformation_uuid.map(|_| {
                let mut source = revival_source.as_ref().unwrap().as_ref().clone();
                source.site.effect_id = "transform".into();
                Arc::new(source)
            });
            has_revival |= revival_source.is_some();
            let idle_guard_source =
                matches!(binding.program, EquipmentDefenseProgram::IdleHpGuard { .. }).then(|| {
                    use super::authoring::{ActionSite, EffectSite, EffectSource};
                    Arc::new(EffectSource {
                        source_file: file.clone(),
                        field_path: field_path.clone(),
                        site: EffectSite {
                            action: ActionSite {
                                definition_id: binding.equipment.clone().into(),
                                action_id: "idle-hp-guard".into(),
                            },
                            effect_id: "prevent".into(),
                        },
                        origin_definition_id: binding.equipment.clone().into(),
                        origin: Default::default(),
                    })
                });
            has_idle_guard |= idle_guard_source.is_some();
            let last_will_source =
                matches!(binding.program, EquipmentDefenseProgram::LastWill { .. }).then(|| {
                    use super::authoring::{ActionSite, EffectSite, EffectSource};
                    Arc::new(EffectSource {
                        source_file: file.clone(),
                        field_path: field_path.clone(),
                        site: EffectSite {
                            action: ActionSite {
                                definition_id: binding.equipment.clone().into(),
                                action_id: "last-will".into(),
                            },
                            effect_id: "blast".into(),
                        },
                        origin_definition_id: binding.equipment.clone().into(),
                        origin: Default::default(),
                    })
                });
            has_last_will |= last_will_source.is_some();
            let adaptive_source = matches!(
                binding.program,
                EquipmentDefenseProgram::AdaptiveBasicShield { .. }
            )
            .then(|| {
                use super::authoring::{ActionSite, EffectSite, EffectSource};
                Arc::new(EffectSource {
                    source_file: file.clone(),
                    field_path: field_path.clone(),
                    site: EffectSite {
                        action: ActionSite {
                            definition_id: binding.equipment.clone().into(),
                            action_id: "adaptive-basic-shield".into(),
                        },
                        effect_id: "shield".into(),
                    },
                    origin_definition_id: binding.equipment.clone().into(),
                    origin: Default::default(),
                })
            });
            let reserve_source = matches!(
                binding.program,
                EquipmentDefenseProgram::OverhealReserve { .. }
            )
            .then(|| {
                use super::authoring::{ActionSite, EffectSite, EffectSource};
                Arc::new(EffectSource {
                    source_file: file.clone(),
                    field_path: field_path.clone(),
                    site: EffectSite {
                        action: ActionSite {
                            definition_id: binding.equipment.clone().into(),
                            action_id: "overheal-reserve".into(),
                        },
                        effect_id: "release".into(),
                    },
                    origin_definition_id: binding.equipment.clone().into(),
                    origin: Default::default(),
                })
            });
            let deferred_source = matches!(
                binding.program,
                EquipmentDefenseProgram::DeferredDamage { .. }
            )
            .then(|| {
                use super::authoring::{ActionSite, EffectSite, EffectSource};
                Arc::new(EffectSource {
                    source_file: file.clone(),
                    field_path: field_path.clone(),
                    site: EffectSite {
                        action: ActionSite {
                            definition_id: binding.equipment.clone().into(),
                            action_id: "deferred-damage".into(),
                        },
                        effect_id: "repay".into(),
                    },
                    origin_definition_id: binding.equipment.clone().into(),
                    origin: Default::default(),
                })
            });
            has_deferred |= deferred_source.is_some();
            has_reserve |= reserve_source.is_some();
            has_adaptive |= adaptive_source.is_some();
            by_equipment.insert(
                item.uuid,
                Arc::new(CompiledEquipmentDefense {
                    conditional_reduction,
                    incoming_flat,
                    wearer_reduction,
                    revival_source,
                    transformation_source,
                    transformation_uuid,
                    last_will_source,
                    adaptive_source,
                    idle_guard_source,
                    reserve_source,
                    deferred_source,
                    program: binding.program.clone(),
                    definition_id: binding.equipment.clone(),
                    source_file: file,
                    field_path,
                }),
            );
        }
        Ok(Self {
            has_conditional_reduction,
            has_incoming_flat,
            has_wearer_reduction,
            by_equipment,
            has_ready_wait,
            has_nearby_defense,
            has_revival,
            has_last_will,
            has_adaptive,
            has_idle_guard,
            has_reserve,
            has_deferred,
            has_channel_opening,
        })
    }
}
