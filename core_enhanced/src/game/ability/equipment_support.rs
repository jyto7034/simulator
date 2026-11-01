//! Equipped-copy support rules. Bounded gain fractions belong to the battle wearer.
use crate::game::data::{equipment_data::EquipmentDatabase, live_ron::LiveRonError};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
mod recipient;
pub use recipient::{ControlDurationKind, ControlDurationProgram};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentSupportBinding {
    pub equipment: String,
    pub program: EquipmentSupportProgram,
    #[serde(skip)]
    pub(crate) source: Option<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentSupportProgram {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shield_beneficiary_damage_percent: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub received_heal_percent: Option<crate::game::value_expr::AmountExpr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_duration: Option<ControlDurationProgram>,
    pub heal_percent: u32,
    pub shield_percent: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_heal_barrier: Option<DirectHealBarrierProgram>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basic_heal_echo: Option<BasicHealEchoProgram>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resonance_gain: Option<ResonanceGainProgram>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_heal_reaction: Option<ActualHealReactionProgram>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActualHealReactionProgram {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub self_shield: Option<ActualHealShieldProgram>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_haste: Option<ActualHealHasteProgram>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActualHealShieldProgram {
    pub actual_heal_percent: u32,
    pub max_hp_cap_percent: u8,
    pub duration_ms: u32,
    pub priority: i32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActualHealHasteProgram {
    pub attack_speed_points: u32,
    pub duration_ms: u32,
}
crate::authoring::clone_shape::struct_shape!(ActualHealReactionProgram {
    self_shield,
    target_haste
});
crate::authoring::clone_shape::struct_shape!(ActualHealShieldProgram {
    actual_heal_percent,
    max_hp_cap_percent,
    duration_ms,
    priority
});
crate::authoring::clone_shape::struct_shape!(ActualHealHasteProgram {
    attack_speed_points,
    duration_ms
});
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectHealBarrierProgram {
    pub hp_threshold_percent: u8,
    pub actual_heal_percent: u32,
    pub duration_ms: u32,
    pub cooldown_ms: u32,
    pub priority: i32,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasicHealEchoProgram {
    pub actual_heal_percent: u32,
    pub max_hp_cap_percent: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResonanceGainProgram {
    pub percent: u32,
    pub channels: ResonanceGainChannels,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResonanceGainChannels {
    Automatic,
    Authored,
    AutomaticAndAuthored,
}
crate::authoring::clone_shape::copy_shape!(ResonanceGainChannels);
crate::authoring::clone_shape::struct_shape!(ResonanceGainProgram { percent, channels });
impl EquipmentSupportProgram {
    pub(crate) fn validate(&self) -> Result<(), (&'static str, &'static str)> {
        if let Some(program) = &self.control_duration { program.validate()?; }
        if self.shield_beneficiary_damage_percent.is_some_and(|p| p > 10_000) {
            return Err(("shield_beneficiary_damage_percent", "shield beneficiary increase must be from 0 to 10000 percent"));
        }
        for (field, value) in [
            ("heal_percent", self.heal_percent),
            ("shield_percent", self.shield_percent),
        ] {
            if value > 10_000 {
                return Err((field, "support increase must be from 0 to 10000 percent"));
            }
        }
        if let Some(barrier) = self.direct_heal_barrier {
            if barrier.hp_threshold_percent > 100 {
                return Err((
                    "direct_heal_barrier.hp_threshold_percent",
                    "heal barrier HP threshold must be from 0 to 100 percent",
                ));
            }
            if barrier.actual_heal_percent == 0 || barrier.actual_heal_percent > 10_000 {
                return Err((
                    "direct_heal_barrier.actual_heal_percent",
                    "heal barrier ratio must be from 1 to 10000 percent",
                ));
            }
            if barrier.duration_ms == 0 {
                return Err((
                    "direct_heal_barrier.duration_ms",
                    "heal barrier requires positive duration",
                ));
            }
        }
        if let Some(echo) = self.basic_heal_echo {
            if echo.actual_heal_percent == 0 || echo.actual_heal_percent > 10_000 {
                return Err((
                    "basic_heal_echo.actual_heal_percent",
                    "basic heal echo ratio must be from 1 to 10000 percent",
                ));
            }
            if echo.max_hp_cap_percent == 0 || echo.max_hp_cap_percent > 100 {
                return Err((
                    "basic_heal_echo.max_hp_cap_percent",
                    "basic heal echo cap must be from 1 to 100 percent",
                ));
            }
        }
        if let Some(gain) = self.resonance_gain {
            if gain.percent > 10_000 {
                return Err((
                    "resonance_gain.percent",
                    "resonance gain increase must be from 0 to 10000 percent",
                ));
            }
        }
        if let Some(reaction) = self.actual_heal_reaction {
            if reaction.self_shield.is_none() && reaction.target_haste.is_none() {
                return Err((
                    "actual_heal_reaction",
                    "actual heal reaction requires shield or haste",
                ));
            }
            if let Some(shield) = reaction.self_shield {
                if shield.actual_heal_percent == 0 || shield.actual_heal_percent > 10_000 {
                    return Err((
                        "actual_heal_reaction.self_shield.actual_heal_percent",
                        "heal shield ratio must be from 1 to 10000 percent",
                    ));
                }
                if shield.max_hp_cap_percent == 0 || shield.max_hp_cap_percent > 100 {
                    return Err((
                        "actual_heal_reaction.self_shield.max_hp_cap_percent",
                        "heal shield cap must be from 1 to 100 percent",
                    ));
                }
                if shield.duration_ms == 0 {
                    return Err((
                        "actual_heal_reaction.self_shield.duration_ms",
                        "heal shield requires positive duration",
                    ));
                }
            }
            if let Some(haste) = reaction.target_haste {
                if haste.attack_speed_points == 0 || haste.attack_speed_points > 10_000 {
                    return Err((
                        "actual_heal_reaction.target_haste.attack_speed_points",
                        "heal haste must be from 1 to 10000 points",
                    ));
                }
                if haste.duration_ms == 0 {
                    return Err((
                        "actual_heal_reaction.target_haste.duration_ms",
                        "heal haste requires positive duration",
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct CompiledEquipmentSupport {
    pub received_heal: Option<Arc<crate::game::value_expr::CompiledAmountPlan>>,
    pub program: EquipmentSupportProgram,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
    pub reaction_source: Option<Arc<super::authoring::EffectSource>>,
    pub echo_source: Option<Arc<super::authoring::EffectSource>>,
    pub barrier_source: Option<Arc<super::authoring::EffectSource>>,
}
#[derive(Debug, Default)]
pub(crate) struct EquipmentSupportCatalog {
    pub has_shield_beneficiary_damage: bool,
    pub has_received_heal: bool,
    pub has_control_duration: bool,
    pub has_basic_heal_echo: bool,
    pub has_resonance_gain: bool,
    pub has_actual_heal_reaction: bool,
    pub by_equipment: BTreeMap<uuid::Uuid, Arc<CompiledEquipmentSupport>>,
}
impl EquipmentSupportCatalog {
    pub(crate) fn compile(equipment: &EquipmentDatabase, limits: &crate::game::value_expr::ExprLimits) -> Result<Self, LiveRonError> {
        let mut has_received_heal = false;
        let mut has_shield_beneficiary_damage = false;
        let mut has_control_duration = false;
        let mut by_equipment = BTreeMap::new();
        let mut retained = 0usize;
        let mut has_basic_heal_echo = false;
        let mut has_resonance_gain = false;
        let mut has_actual_heal_reaction = false;
        for (index, binding) in equipment.equipment_supports.iter().enumerate() {
            let (file, path) = binding.source.clone().unwrap_or_else(|| {
                (
                    "data/equipments/base.ron".into(),
                    format!("equipment_supports[{index}]"),
                )
            });
            let error = |field: &str, cause: String| {
                LiveRonError::validation(
                    &file,
                    "equipment_supports",
                    Some(binding.equipment.clone()),
                    Some(format!("{path}.{field}")),
                    cause,
                )
            };
            crate::authoring::control::charge(
                equipment.items.len() as u64 + 1,
                "equipment_support_reference",
            )
            .map_err(|e| error("equipment", format!("{e:?}")))?;
            let item = equipment
                .get_by_id(&binding.equipment)
                .ok_or_else(|| error("equipment", "support requires existing equipment".into()))?;
            if by_equipment.contains_key(&item.uuid) {
                return Err(error("equipment", "duplicate support binding".into()));
            }
            binding
                .program
                .validate()
                .map_err(|(p, reason)| error(&format!("program.{p}"), reason.into()))?;
            let received_heal = binding.compile_received_heal(limits).map_err(|(p,e)| error(&p,e))?;
            has_received_heal |= received_heal.is_some();
            has_shield_beneficiary_damage |= binding.program.shield_beneficiary_damage_percent.is_some_and(|p| p > 0);
            has_control_duration |= binding.program.control_duration.is_some();
            let recipient_bytes = recipient::compiled_storage_bound(&binding.program, received_heal.as_deref()).map_err(|e| error("program",e))?;
            let field_path = format!("{path}.program");
            let additional = recipient_bytes + std::mem::size_of::<CompiledEquipmentSupport>()
                + 128
                + binding.equipment.len()
                + file.len()
                + field_path.len()
                + if binding.program.direct_heal_barrier.is_some() {
                    std::mem::size_of::<super::authoring::EffectSource>()
                        + 64
                        + binding.equipment.len() * 2
                        + file.len()
                        + field_path.len()
                } else {
                    0
                }
                + if binding.program.actual_heal_reaction.is_some() {
                    std::mem::size_of::<super::authoring::EffectSource>()
                        + 192
                        + binding.equipment.len() * 2
                        + file.len()
                        + field_path.len()
                } else {
                    0
                }
                + if binding.program.basic_heal_echo.is_some() {
                    std::mem::size_of::<super::authoring::EffectSource>()
                        + 64
                        + binding.equipment.len() * 2
                        + file.len()
                        + field_path.len()
                } else {
                    0
                };
            crate::authoring::control::admit(
                crate::authoring::control::Resource::MetadataBytes,
                retained,
                additional,
                "equipment_support_catalog_storage",
            )
            .map_err(|e| error("program", format!("{e:?}")))?;
            retained = retained
                .checked_add(additional)
                .ok_or_else(|| error("program", "catalog storage overflow".into()))?;
            let barrier_source = binding.program.direct_heal_barrier.map(|_| {
                use super::authoring::{ActionSite, EffectSite, EffectSource};
                Arc::new(EffectSource {
                    source_file: file.clone(),
                    field_path: format!("{field_path}.direct_heal_barrier"),
                    site: EffectSite {
                        action: ActionSite {
                            definition_id: binding.equipment.clone().into(),
                            action_id: "direct-heal-barrier".into(),
                        },
                        effect_id: "barrier".into(),
                    },
                    origin_definition_id: binding.equipment.clone().into(),
                    origin: Default::default(),
                })
            });
            has_basic_heal_echo |= binding.program.basic_heal_echo.is_some();
            has_resonance_gain |= binding
                .program
                .resonance_gain
                .is_some_and(|g| g.percent > 0);
            has_actual_heal_reaction |= binding.program.actual_heal_reaction.is_some();
            let reaction_source = binding.program.actual_heal_reaction.map(|_| {
                use super::authoring::{ActionSite, EffectSite, EffectSource};
                Arc::new(EffectSource {
                    source_file: file.clone(),
                    field_path: format!("{field_path}.actual_heal_reaction"),
                    site: EffectSite {
                        action: ActionSite {
                            definition_id: binding.equipment.clone().into(),
                            action_id: "actual-heal-reaction".into(),
                        },
                        effect_id: "reaction".into(),
                    },
                    origin_definition_id: binding.equipment.clone().into(),
                    origin: Default::default(),
                })
            });
            let echo_source = binding.program.basic_heal_echo.map(|_| {
                use super::authoring::{ActionSite, EffectSite, EffectSource};
                Arc::new(EffectSource {
                    source_file: file.clone(),
                    field_path: format!("{field_path}.basic_heal_echo"),
                    site: EffectSite {
                        action: ActionSite {
                            definition_id: binding.equipment.clone().into(),
                            action_id: "basic-heal-echo".into(),
                        },
                        effect_id: "self-heal".into(),
                    },
                    origin_definition_id: binding.equipment.clone().into(),
                    origin: Default::default(),
                })
            });
            by_equipment.insert(
                item.uuid,
                Arc::new(CompiledEquipmentSupport {
                    program: binding.program.clone(),
                    received_heal,
                    definition_id: binding.equipment.clone(),
                    source_file: file,
                    field_path,
                    reaction_source,
                    barrier_source,
                    echo_source,
                }),
            );
        }
        Ok(Self {
            has_shield_beneficiary_damage,
            has_received_heal,
            has_control_duration,
            by_equipment,
            has_basic_heal_echo,
            has_resonance_gain,
            has_actual_heal_reaction,
        })
    }
}
