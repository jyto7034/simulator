use super::*;
use crate::game::{ability::SkillHitTargetFilter, battle::damage::DamageType};

impl<'a> Lower<'a> {
    pub(super) fn ground_action(
        &mut self,
        request: &str,
        admit: &str,
        admission: ResonanceAdmission,
    ) -> Result<GraphOperation, Diagnostic> {
        let selection = self.input(request, "selection", "skill-target-selection")?;
        if self.text(selection, "mode") != "player-tile" {
            return Err(self.error(
                selection,
                None,
                "Independent ground requires an explicitly selected tile.",
            ));
        }
        let selection_range = self.tile_pattern(selection, "range")?;
        let area = self.only("skill-ground-area")?;
        self.link(area, "use", admit, "use")?;
        let receipt = self.input(area, "in", "skill-resonance-result")?;
        self.link(receipt, "in", admit, "out")?;
        self.link(receipt, "receipt", admit, "receipt")?;
        let end = self.only("skill-resonance-end")?;
        self.link(end, "in", area, "out")?;
        self.link(end, "use", admit, "use")?;
        let placement = self.input(area, "area", "skill-range-placement")?;
        if self.text(placement, "origin") != "selected-tile" {
            return Err(self.error(
                placement,
                Some("origin"),
                "This ground target requires a fixed, world-aligned selected tile.",
            ));
        }
        // Position/facing timing fields are inactive for selected-tile placement.
        let footprint = self.tile_pattern(placement, "range")?;
        let duration_ms = self.positive_duration(area, "duration")?;
        if self.doc.nodes.values().any(|n| n.kind == "skill-ground-status") {
            return self.ground_status_action(area, admission, selection_range, footprint, duration_ms);
        }
        let periodic = self.only("skill-ground-periodic")?;
        self.link(periodic, "area", area, "area")?;
        let interval_ms = self.positive_duration(periodic, "interval")?;
        let first = match self.text(periodic, "first") {
            "immediate" => GroundFirstTick::Immediate,
            "after-interval" => GroundFirstTick::AfterInterval,
            _ => {
                return Err(self.error(periodic, Some("first"), "Choose the first periodic event."))
            }
        };
        let targets = self.only("skill-ground-targets")?;
        self.link(targets, "in", periodic, "out")?;
        self.link(targets, "area", area, "area")?;
        let filter = self.input(targets, "filter", "skill-ground-filter")?;
        let hit_targets = match self.text(filter, "side") {
            "enemies" => SkillHitTargetFilter::Enemies,
            "allies" => SkillHitTargetFilter::Allies,
            "all" => SkillHitTargetFilter::Any,
            _ => return Err(self.error(filter, Some("side"), "Choose ground recipients.")),
        };
        let include_caster = match self.text(filter, "caster") {
            "include" => true,
            "exclude" => false,
            _ => return Err(self.error(filter, Some("caster"), "Choose caster membership.")),
        };
        let damage = self
            .doc
            .nodes
            .values()
            .any(|node| node.kind == "skill-ground-damage");
        let effect_id = self.only(if damage {
            "skill-ground-damage"
        } else {
            "skill-ground-heal"
        })?;
        self.link(effect_id, "in", targets, "out")?;
        self.link(effect_id, "member", targets, "member")?;
        let power = self.input(
            effect_id,
            "power",
            if damage {
                "skill-damage-amount"
            } else {
                "skill-healing-amount"
            },
        )?;
        let amount = self.number(power, "value")?;
        let effect = if damage {
            let damage_type = match self.text(effect_id, "damageType") {
                "physical" => DamageType::Physical,
                "magic" => DamageType::Magic,
                "true" => DamageType::True,
                _ => {
                    return Err(self.error(
                        effect_id,
                        Some("damageType"),
                        "Choose the ground damage type.",
                    ))
                }
            };
            GroundEffect::Damage {
                amount,
                damage_type,
            }
        } else {
            GroundEffect::Heal { amount }
        };
        let ended = self.only("skill-ground-ended")?;
        self.link(ended, "area", area, "area")?;
        let cue = self.only("skill-cue")?;
        self.link(cue, "event", ended, "cue")?;
        let program = GroundProgram {
            selection_range,
            footprint,
            duration_ms,
            interval_ms,
            first,
            hit_targets,
            include_caster,
            effect,
        };
        program.validate().map_err(|e| self.error(area, None, &e))?;
        Ok(GraphOperation::ResonanceGround { admission, program })
    }
}
