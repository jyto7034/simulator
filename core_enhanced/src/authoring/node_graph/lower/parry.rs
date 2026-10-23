use super::*;
use crate::game::{battle::damage::DamageType, data::skill_fragment_data::ParryGuardDef};
impl<'a> Lower<'a> {
    pub(super) fn parry_action(
        &mut self,
        request: &str,
        plan: &'a str,
        admission: ResonanceAdmission,
    ) -> Result<GraphOperation, Diagnostic> {
        self.self_selection(request)?;
        let guard = self.input(plan, "guard", "skill-passive-guard")?;
        let reflection = self.only("skill-direct-reflection")?;
        self.link(reflection, "guard", guard, "guard")?;
        if self.text(reflection, "damageType") != "original" {
            return Err(self.error(
                reflection,
                None,
                "Native direct reflection requires the original damage type.",
            ));
        }
        let power = self.input(plan, "power", "skill-weapon-power")?;
        let (coefficient_numerator, coefficient_denominator) = self.coefficient(power)?;
        let damage_type = match self.text(plan, "damageType") {
            "physical" => DamageType::Physical,
            "magic" => DamageType::Magic,
            _ => return Err(self.error(plan, None, "Parry damage type is unresolved.")),
        };
        let charge = if self
            .doc
            .nodes
            .values()
            .any(|n| n.kind == "skill-guard-charge")
        {
            let id = self.only("skill-guard-charge")?;
            self.link(id, "guard", guard, "guard")?;
            self.link(id, "parry", plan, "plan")?;
            let condition = self.taunt_condition(id)?;
            let (n, d) = self.exact_ratio(id, "factor")?;
            let (sn, sd) = self.exact_ratio(id, "splash")?;
            Some(crate::game::ability::guard_charge::GuardChargeDef {
                definition: id.into(),
                relic_definition: self.text(condition, "definition").into(),
                capacity: self.number(id, "capacity")?,
                primary: crate::game::ability::marks::MarkRatio {
                    numerator: n,
                    denominator: d,
                },
                splash: crate::game::ability::marks::MarkRatio {
                    numerator: sn,
                    denominator: sd,
                },
                range: self.tile_pattern(id, "range")?,
            })
        } else {
            None
        };
        let program = ParryProgram {
            window_ms: self.positive_duration(plan, "window")?,
            invulnerable_ms: self.positive_duration(plan, "immunity")?,
            miss_recovery_ms: self.positive_duration(plan, "failure")?,
            guard: ParryGuardDef {
                reduction_percent: self.number(guard, "percent")?,
                reflection_chance_percent: self.number(reflection, "chance")?,
                reflection_percent: self.number(reflection, "percent")?,
            },
            coefficient_numerator,
            coefficient_denominator,
            damage_type,
            charge,
        };
        program.validate().map_err(|e| self.error(plan, None, &e))?;
        Ok(GraphOperation::ResonanceParry { admission, program })
    }
}
