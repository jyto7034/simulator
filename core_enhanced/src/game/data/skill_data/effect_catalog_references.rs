use super::SkillDatabase;
use crate::game::{
    ability::{
        mobile_effect::MobileProfile,
        sustained::{BasicAttackOverlayProfile, SustainedEffectDef},
        tile_burst::TileBurstSource,
        SkillEffectDef,
    },
    data::{equipment_data::EquipmentDatabase, relic_data::RelicDatabase, DataValidationError},
};

impl SkillDatabase {
    /// Standalone skills validate structure. Explicit catalog references are
    /// resolved here, once the complete generation is available, even when no
    /// fragment currently equips the skill. Action/support variants retain their
    /// additional validation in the structural relic validator.
    pub(crate) fn validate_effect_catalog_references(
        &self,
        equipment: &EquipmentDatabase,
        relics: &RelicDatabase,
    ) -> Result<(), DataValidationError> {
        for (si, skill) in self.skills.iter().enumerate() {
            let refs = EffectReferences {
                object: skill.id.as_str(),
                equipment,
                relics,
            };
            refs.charge(1, "tile_burst_reference_skill")?;
            for (ti, step) in skill.steps.iter().enumerate() {
                refs.charge(1, "tile_burst_reference_step")?;
                for (ei, effect) in step.effects.iter().enumerate() {
                    refs.charge(1, "tile_burst_reference_effect")?;
                    refs.effect(
                        &effect.op,
                        &format!("skills[{si}].steps[{ti}].effects[{ei}].op"),
                    )?;
                }
            }
        }
        Ok(())
    }
}

struct EffectReferences<'a> {
    object: &'a str,
    equipment: &'a EquipmentDatabase,
    relics: &'a RelicDatabase,
}
impl EffectReferences<'_> {
    fn error(&self, path: &str, cause: &str) -> DataValidationError {
        DataValidationError::at(self.object, path, cause)
    }
    fn charge(&self, work: u64, site: &'static str) -> Result<(), DataValidationError> {
        crate::authoring::control::charge(work, site).map_err(|e| self.error("steps", e.code()))
    }
    fn relic(&self, id: &str, path: &str) -> Result<(), DataValidationError> {
        self.charge(
            self.relics.relics.len() as u64 + 1,
            "effect_relic_reference",
        )?;
        if self.relics.get_by_id(id).is_none() {
            return Err(self.error(path, "Skill effect references unknown relic"));
        }
        Ok(())
    }
    fn modifiers<'a>(
        &self,
        ids: impl Iterator<Item = &'a str>,
        path: &str,
    ) -> Result<(), DataValidationError> {
        for (i, id) in ids.enumerate() {
            self.relic(id, &format!("{path}[{i}].relic_definition"))?;
        }
        Ok(())
    }
    fn overlay(
        &self,
        profile: &BasicAttackOverlayProfile,
        path: &str,
    ) -> Result<(), DataValidationError> {
        if let Some(rule) = profile
            .line
            .as_ref()
            .and_then(|line| line.piercing.as_ref())
        {
            self.relic(
                &rule.relic_definition,
                &format!("{path}.line.piercing.relic_definition"),
            )?;
        }
        Ok(())
    }
    fn mobile(&self, profile: &MobileProfile, path: &str) -> Result<(), DataValidationError> {
        self.modifiers(
            profile
                .modifiers
                .iter()
                .map(|m| m.relic_definition.as_str()),
            &format!("{path}.modifiers"),
        )
    }
    fn sustained(&self, def: &SustainedEffectDef, path: &str) -> Result<(), DataValidationError> {
        match def {
            SustainedEffectDef::BasicAttackOverlay { profile, .. } => {
                self.overlay(profile, &format!("{path}.BasicAttackOverlay.profile"))
            }
            SustainedEffectDef::State { program, .. }
            | SustainedEffectDef::PreparedState { program, .. } => {
                let kind = if matches!(def, SustainedEffectDef::State { .. }) {
                    "State"
                } else {
                    "PreparedState"
                };
                if let Some(profile) = &program.basic_attack {
                    self.overlay(profile, &format!("{path}.{kind}.program.basic_attack"))?;
                }
                Ok(())
            }
            SustainedEffectDef::Parry { guard_charge, .. } => {
                if let Some(charge) = guard_charge {
                    self.relic(
                        &charge.relic_definition,
                        &format!("{path}.Parry.guard_charge.relic_definition"),
                    )?;
                }
                Ok(())
            }
            SustainedEffectDef::HitStatus { .. }
            | SustainedEffectDef::PreparedBasic { .. }
            | SustainedEffectDef::ReactiveHeal { .. }
            | SustainedEffectDef::TargetState { .. }
            | SustainedEffectDef::Control { .. }
            | SustainedEffectDef::Survival { .. }
            | SustainedEffectDef::SegmentedShield { .. }
            | SustainedEffectDef::IndependentShield { .. }
            | SustainedEffectDef::Shield { .. }
            | SustainedEffectDef::ChannelAura { .. }
            | SustainedEffectDef::Modifiers { .. }
            | SustainedEffectDef::CombatModifiers { .. } => Ok(()),
        }
    }
    fn effect(&self, op: &SkillEffectDef, path: &str) -> Result<(), DataValidationError> {
        match op {
            SkillEffectDef::Sustained { definition } => {
                self.sustained(definition, &format!("{path}.Sustained.definition"))
            }
            SkillEffectDef::RelationAction { program } => self.modifiers(
                program
                    .modifiers
                    .iter()
                    .map(|m| m.relic_definition.as_str()),
                &format!("{path}.RelationAction.program.modifiers"),
            ),
            SkillEffectDef::CounterChannel { program, .. } => self.modifiers(
                program
                    .modifiers
                    .iter()
                    .map(|m| m.relic_definition.as_str()),
                &format!("{path}.CounterChannel.program.modifiers"),
            ),
            SkillEffectDef::PreparedVolley { program } => self.modifiers(
                program
                    .modifiers
                    .iter()
                    .map(|m| m.relic_definition.as_str()),
                &format!("{path}.PreparedVolley.program.modifiers"),
            ),
            SkillEffectDef::PairedAreas { program } => self.modifiers(
                program
                    .modifiers
                    .iter()
                    .map(|m| m.relic_definition.as_str()),
                &format!("{path}.PairedAreas.program.modifiers"),
            ),
            SkillEffectDef::PresenceBurst { program } => self.modifiers(
                program
                    .modifiers
                    .iter()
                    .map(|m| m.relic_definition.as_str()),
                &format!("{path}.PresenceBurst.program.modifiers"),
            ),
            SkillEffectDef::InputPlan { program } => {
                let path = format!("{path}.InputPlan.program.modifiers");
                for (i, modifier) in program.modifiers.iter().enumerate() {
                    self.relic(
                        &modifier.relic_definition,
                        &format!("{path}[{i}].relic_definition"),
                    )?;
                    for (ri, id) in modifier.requires.iter().enumerate() {
                        self.relic(id, &format!("{path}[{i}].requires[{ri}]"))?;
                    }
                }
                Ok(())
            }
            SkillEffectDef::GuardCounter { program } => self.relic(
                &program.charge.relic_definition,
                &format!("{path}.GuardCounter.program.charge.relic_definition"),
            ),
            SkillEffectDef::FusionPlacement { program } => {
                let path = format!("{path}.FusionPlacement.program");
                for (i, tier) in program.tiers.iter().enumerate() {
                    self.charge(1, "fusion_tier_reference")?;
                    self.mobile(&tier.profile, &format!("{path}.tiers[{i}].profile"))?;
                }
                if let Some(seed) = &program.seed {
                    self.relic(
                        &seed.relic_definition,
                        &format!("{path}.seed.relic_definition"),
                    )?;
                    self.modifiers(
                        seed.modifiers.iter().map(|m| m.relic_definition.as_str()),
                        &format!("{path}.seed.modifiers"),
                    )?;
                }
                Ok(())
            }
            SkillEffectDef::StatusStrike { program } => {
                let path = format!("{path}.StatusStrike.program");
                self.modifiers(
                    program
                        .range_modifiers
                        .iter()
                        .map(|m| m.relic_definition.as_str()),
                    &format!("{path}.range_modifiers"),
                )?;
                self.modifiers(
                    program
                        .from
                        .modifiers
                        .iter()
                        .map(|m| m.relic_definition.as_str()),
                    &format!("{path}.from.modifiers"),
                )?;
                self.modifiers(
                    program
                        .to
                        .modifiers
                        .iter()
                        .map(|m| m.relic_definition.as_str()),
                    &format!("{path}.to.modifiers"),
                )?;
                self.mobile(&program.mobile, &format!("{path}.mobile"))?;
                if let Some(death) = &program.death {
                    self.relic(
                        &death.relic_definition,
                        &format!("{path}.death.relic_definition"),
                    )?;
                }
                Ok(())
            }
            SkillEffectDef::TileBurst { program } => {
                let path = format!("{path}.TileBurst.program");
                if let TileBurstSource::WeaponStacks {
                    weapon,
                    key,
                    conversion,
                    ..
                } = &program.source
                {
                    self.charge(
                        self.equipment.items.len() as u64 + 1,
                        "tile_burst_weapon_reference",
                    )?;
                    let marks = self
                        .equipment
                        .get_by_id(weapon)
                        .and_then(|m| m.weapon_profile.as_ref())
                        .and_then(|p| p.mark_weapon.as_ref())
                        .ok_or_else(|| {
                            self.error(
                                &format!("{path}.source.WeaponStacks.weapon"),
                                "Tile burst requires an existing mark weapon",
                            )
                        })?;
                    if &marks.key != key {
                        return Err(self.error(
                            &format!("{path}.source.WeaponStacks.key"),
                            "Tile burst and weapon must share one explicit store",
                        ));
                    }
                    if let Some(orb) = conversion {
                        self.relic(
                            &orb.relic_definition,
                            &format!("{path}.source.WeaponStacks.conversion.relic_definition"),
                        )?;
                    }
                }
                self.modifiers(
                    program
                        .modifiers
                        .iter()
                        .map(|m| m.relic_definition.as_str()),
                    &format!("{path}.modifiers"),
                )
            }
            SkillEffectDef::PersonalShieldGrant { .. }
            | SkillEffectDef::CreateObject { .. }
            | SkillEffectDef::Pull { .. }
            | SkillEffectDef::HealthCost { .. }
            | SkillEffectDef::RechargeShield { .. }
            | SkillEffectDef::ShieldExpr { .. }
            | SkillEffectDef::Damage { .. }
            | SkillEffectDef::DamageExpr { .. }
            | SkillEffectDef::ModifyDamage { .. }
            | SkillEffectDef::Heal { .. }
            | SkillEffectDef::HealExpr { .. }
            | SkillEffectDef::ModifyResonance { .. }
            | SkillEffectDef::ModifyTacticalResource { .. }
            | SkillEffectDef::Resources { .. }
            | SkillEffectDef::ModifyStats { .. }
            | SkillEffectDef::ApplyBuff { .. }
            | SkillEffectDef::InterruptCast
            | SkillEffectDef::ExtraAttack { .. } => Ok(()),
        }
    }
}
