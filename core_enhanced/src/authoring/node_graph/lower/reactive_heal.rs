use super::*;
use crate::game::{ability::sustained::ReactiveHealProfile, battle::tile_range::TileRangePattern};

impl<'a> Lower<'a> {
    pub(super) fn input_any(&mut self, id: &str, port: &str) -> Result<&'a str, Diagnostic> {
        let Some(origins) = self.doc.nodes[id].inputs.get(port).filter(|v| v.len() == 1) else {
            return Err(self.error(id, Some(port), "Exactly one producer is required."));
        };
        let producer = origins[0].source.node.as_str();
        self.used.insert(producer);
        Ok(producer)
    }
    pub(super) fn self_selection(&mut self, request: &str) -> Result<(), Diagnostic> {
        if self.doc.nodes[request].inputs.contains_key("selection") {
            let selection = self.input(request, "selection", "skill-target-selection")?;
            if self.mode(selection) != "self"
                || self.doc.nodes[selection].inputs.contains_key("range")
            {
                return Err(self.error(
                    selection,
                    None,
                    "This native plan requires self selection without a range.",
                ));
            }
        }
        Ok(())
    }
    pub(super) fn positive_duration(&mut self, id: &str, port: &str) -> Result<u32, Diagnostic> {
        let seconds = self.input(id, port, "skill-seconds")?;
        match self.doc.nodes[seconds].values.get("value") {
            Some(FieldValue::Number(n)) => {
                structure::milliseconds(&serde_json::json!(n)).and_then(|n| u32::try_from(n).ok())
            }
            _ => None,
        }
        .filter(|n| *n > 0)
        .ok_or_else(|| {
            self.error(
                seconds,
                None,
                "Duration requires exact positive milliseconds in u32 range.",
            )
        })
    }
    pub(super) fn reactive_heal_action(
        &mut self,
        request: &str,
        plan: &str,
        admission: ResonanceAdmission,
    ) -> Result<GraphOperation, Diagnostic> {
        let selection = self.input(request, "selection", "skill-target-selection")?;
        if self.mode(selection) != "player-ally" {
            return Err(self.error(
                selection,
                None,
                "Reactive heal requires selection of an allied employee.",
            ));
        }
        let range_node = self.input(selection, "range", "skill-range")?;
        let range = TileRangePattern {
            include_anchor_tile: self.doc.nodes[range_node].values.get("includeAnchor")
                == Some(&FieldValue::Boolean(true)),
            rows: self
                .text(range_node, "pattern")
                .split('\n')
                .map(str::to_owned)
                .collect(),
        };
        range
            .validate()
            .map_err(|e| self.error(range_node, None, &e))?;
        let duration_ms = self.positive_duration(plan, "duration")?;
        let profile = ReactiveHealProfile {
            definition: format!("{}:{plan}", self.doc.document),
            charges: self.number(plan, "charges")?,
            max_health_percent: self.number(plan, "percent")?,
        };
        profile.validate().map_err(|e| self.error(plan, None, &e))?;
        Ok(GraphOperation::ResonanceReactiveHeal {
            admission,
            duration_ms,
            profile,
            range,
        })
    }
}
