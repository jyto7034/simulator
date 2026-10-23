use super::*;
use crate::game::battle::damage::DamageType;

impl<'a> Lower<'a> {
    pub(super) fn scheduled_area_action(
        &mut self,
        request: &str,
        plan: &'a str,
        admission: ResonanceAdmission,
    ) -> Result<GraphOperation, Diagnostic> {
        self.self_selection(request)?;
        if self.text(plan, "first") != "immediate" {
            return Err(self.error(
                plan,
                None,
                "This schedule starts with an immediate opportunity.",
            ));
        }
        let count = self.input(plan, "count", "skill-count-positive")?;
        let opportunities = self.number(count, "value")?;
        let interval_ms = self.positive_duration(plan, "interval")?;
        let body = self.input(plan, "body", "skill-delayed-area-plan")?;
        if self.text(body, "selectionTime") != "start" {
            return Err(self.error(
                body,
                None,
                "Scheduled impacts select a target at each opportunity.",
            ));
        }
        let selector = self.input(body, "selector", "skill-enemy-selector")?;
        if self.text(selector, "order") != "route" {
            return Err(self.error(
                selector,
                None,
                "This schedule requires route progress ordering.",
            ));
        }
        let include_air = match self.text(selector, "air") {
            "include" => true,
            "exclude" => false,
            _ => return Err(self.error(selector, None, "Unresolved airborne policy.")),
        };
        let acquisition = self.tile_pattern(selector, "range")?;
        let footprint = self.tile_pattern(body, "range")?;
        let power = self.input(body, "power", "skill-weapon-power")?;
        let (coefficient_numerator, coefficient_denominator) = self.coefficient(power)?;
        let damage_type = match self.text(body, "damageType") {
            "physical" => DamageType::Physical,
            "magic" => DamageType::Magic,
            "true" => DamageType::True,
            _ => return Err(self.error(body, None, "Unresolved damage type.")),
        };
        let impact = DelayedAreaProgram {
            prepare_ms: 0,
            delivery_delay_ms: self.duration_allow_zero(body, "delay")?,
            acquisition,
            footprint,
            include_air,
            damage_type,
            coefficient_numerator,
            coefficient_denominator,
            on_hit_heal_percent: None,
        };
        let program = ScheduledAreaProgram {
            opportunities,
            interval_ms,
            impact,
        };
        program.validate().map_err(|e| self.error(plan, None, &e))?;
        Ok(GraphOperation::ResonanceScheduledArea { admission, program })
    }
}
