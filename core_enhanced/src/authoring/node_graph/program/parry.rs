use crate::game::{
    ability::{sustained::SustainedEffectDef, SkillId},
    battle::damage::DamageType,
    data::skill_fragment_data::ParryGuardDef,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParryProgram {
    pub window_ms: u32,
    pub invulnerable_ms: u32,
    pub miss_recovery_ms: u32,
    pub guard: ParryGuardDef,
    pub coefficient_numerator: u64,
    pub coefficient_denominator: u64,
    pub damage_type: DamageType,
    #[serde(default)]
    pub charge: Option<crate::game::ability::guard_charge::GuardChargeDef>,
}
impl ParryProgram {
    pub(crate) fn definition(&self, counter_skill: SkillId) -> Result<SustainedEffectDef, String> {
        let duration_ms = self
            .window_ms
            .checked_add(self.invulnerable_ms.max(self.miss_recovery_ms))
            .ok_or("Parry duration overflow")?;
        Ok(SustainedEffectDef::Parry {
            duration_ms,
            window_ms: self.window_ms,
            invulnerable_ms: self.invulnerable_ms,
            miss_recovery_ms: self.miss_recovery_ms,
            reduction_percent: 0,
            counter_skill,
            end_on_resolution: true,
            fragment_guard: Some(self.guard),
            guard_charge: self.charge.clone().map(std::sync::Arc::new),
        })
    }
    pub(crate) fn validate(&self) -> Result<(), String> {
        let limit = (1u64 << 53) - 1;
        if self.coefficient_numerator == 0
            || self.coefficient_denominator == 0
            || self.coefficient_numerator > limit
            || self.coefficient_denominator > limit
            || !matches!(self.damage_type, DamageType::Physical | DamageType::Magic)
        {
            return Err(
                "Parry counter requires a positive bounded weapon ratio and physical/magic damage"
                    .into(),
            );
        }
        self.definition("counter".into())?.validate()
    }
}
