use super::*;
use crate::game::battle::{damage::DamageType, tile_range::TileRangePattern};

impl<'a> Lower<'a> {
    pub(super) fn tile_pattern(
        &mut self,
        id: &str,
        port: &str,
    ) -> Result<TileRangePattern, Diagnostic> {
        let range = self.input(id, port, "skill-range")?;
        let pattern = TileRangePattern {
            include_anchor_tile: self.doc.nodes[range].values.get("includeAnchor")
                == Some(&FieldValue::Boolean(true)),
            rows: self
                .text(range, "pattern")
                .split('\n')
                .map(str::to_owned)
                .collect(),
        };
        pattern
            .validate()
            .map_err(|e| self.error(range, None, &e))?;
        Ok(pattern)
    }
    pub(super) fn duration_allow_zero(&mut self, id: &str, port: &str) -> Result<u32, Diagnostic> {
        let time = self.input(id, port, "skill-seconds")?;
        let Some(FieldValue::Number(number)) = self.doc.nodes[time].values.get("value") else {
            return Err(self.error(time, None, "Time is unresolved."));
        };
        structure::milliseconds(&serde_json::json!(number))
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| self.error(time, None, "Time requires exact milliseconds in u32 range."))
    }
    pub(super) fn coefficient(&self, id: &str) -> Result<(u64, u64), Diagnostic> {
        self.exact_ratio(id, "coefficient")
    }
    pub(super) fn exact_ratio(&self, id: &str, field: &str) -> Result<(u64, u64), Diagnostic> {
        let Some(FieldValue::Number(n)) = self.doc.nodes[id].values.get(field) else {
            return Err(self.error(
                id,
                Some(field),
                "Power requires an exact positive bounded decimal ratio.",
            ));
        };
        self.exact_numeric_ratio(*n, id, field)
    }
    pub(super) fn exact_numeric_ratio(
        &self,
        n: f64,
        id: &str,
        field: &str,
    ) -> Result<(u64, u64), Diagnostic> {
        let error = || {
            self.error(
                id,
                Some(field),
                "Power requires an exact positive bounded decimal ratio.",
            )
        };
        if !n.is_finite() || n <= 0.0 {
            return Err(error());
        }
        let text = n.to_string();
        let (digits, exponent) = match text.split_once('e') {
            Some((digits, exponent)) => (digits, exponent.parse::<i32>().map_err(|_| error())?),
            None => (text.as_str(), 0),
        };
        let fraction = digits.split_once('.').map_or(0, |(_, f)| f.len() as i32);
        let mut numerator = digits
            .replace('.', "")
            .parse::<u64>()
            .map_err(|_| error())?;
        let scale = fraction.checked_sub(exponent).ok_or_else(error)?;
        let denominator = if scale < 0 {
            numerator = numerator
                .checked_mul(10u64.checked_pow(scale.unsigned_abs()).ok_or_else(error)?)
                .ok_or_else(error)?;
            1
        } else {
            10u64.checked_pow(scale as u32).ok_or_else(error)?
        };
        let mut a = numerator;
        let mut b = denominator;
        while b != 0 {
            let next = a % b;
            a = b;
            b = next;
        }
        Ok((numerator / a, denominator / a))
    }
    pub(super) fn delayed_area_action(
        &mut self,
        request: &str,
        entry: &str,
        plan: &'a str,
        admission: ResonanceAdmission,
    ) -> Result<GraphOperation, Diagnostic> {
        self.self_selection(request)?;
        let (prepare_ms, body) = if self.doc.nodes[plan].kind == "skill-sequence-plan" {
            let wait = self.input(plan, "first", "skill-wait-plan")?;
            let prepare = self.duration_allow_zero(wait, "duration")?;
            (prepare, self.input_any(plan, "next")?)
        } else {
            (0, plan)
        };
        let (body, on_hit_heal_percent) = if self.doc.nodes[body].kind == "skill-on-hit-plan" {
            let heal = self.input(body, "after", "skill-self-heal-plan")?;
            let percent = self.number(heal, "percent")?;
            (
                self.input(body, "body", "skill-delayed-area-plan")?,
                Some(percent),
            )
        } else {
            (body, None)
        };
        if self.doc.nodes[body].kind != "skill-delayed-area-plan"
            || self.text(body, "selectionTime") != "accepted"
        {
            return Err(self.error(
                body,
                None,
                "This plan requires a delayed area with admission-time target selection.",
            ));
        }
        let selector = self.input(body, "selector", "skill-enemy-selector")?;
        self.link(entry, "selector", selector, "selector")?;
        if self.text(selector, "order") != "route" {
            return Err(self.error(
                selector,
                None,
                "This selector requires route progress ordering.",
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
        let program = DelayedAreaProgram {
            prepare_ms,
            delivery_delay_ms: self.duration_allow_zero(body, "delay")?,
            acquisition,
            footprint,
            include_air,
            damage_type,
            coefficient_numerator,
            coefficient_denominator,
            on_hit_heal_percent,
        };
        program.validate().map_err(|e| self.error(body, None, &e))?;
        Ok(GraphOperation::ResonanceDelayedArea { admission, program })
    }
}
