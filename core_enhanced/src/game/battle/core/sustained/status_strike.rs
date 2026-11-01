use super::*;
use crate::game::{ability::sustained::HitStatusKind, battle::runtime_facts::StatusPredecessor};
impl SustainedState {
    pub(super) fn hit_slow_modifier(&self, target: UnitInstanceId) -> Option<StatModifier> {
        // Different source records preserve their own deadlines. One strongest slow controls movement.
        self.records
            .values()
            .filter(|r| r.target == target)
            .filter_map(|r| match r.hit_status.as_ref()?.kind {
                HitStatusKind::Slow { percent } => Some(percent),
                _ => None,
            })
            .max()
            .map(|p| StatModifier {
                stat: crate::game::stats::StatId::MoveSpeedUnitsPerMs,
                kind: crate::game::stats::StatModifierKind::Percent,
                value: -(p as i32),
            })
    }
}
impl BattleCore {
    pub(in crate::game::battle::core) fn status_predecessors(
        &self,
        target: UnitInstanceId,
        owner: UnitInstanceId,
        definition: &str,
        now: u64,
    ) -> Result<Vec<StatusPredecessor>, GameError> {
        control::charge(self.sustained.records.len() as u64, "status_source_query")?;
        let mut out = Vec::new();
        for (id, r) in &self.sustained.records {
            if r.target != target
                || r.caster != owner
                || r.expires_at <= now
                || !r
                    .hit_status
                    .as_ref()
                    .is_some_and(|s| s.definition == definition)
            {
                continue;
            }
            if let Some(fragment) = &r.source.fragment {
                out.push(StatusPredecessor {
                    activation: *id,
                    owner,
                    fragment: fragment.clone(),
                    definition: definition.into(),
                });
            }
        }
        Ok(out)
    }
    pub(in crate::game::battle::core) fn has_status_definition(
        &self,
        target: UnitInstanceId,
        definition: &str,
        now: u64,
    ) -> Result<bool, GameError> {
        control::charge(self.sustained.records.len() as u64, "status_blocker_query")?;
        Ok(self.sustained.records.values().any(|r| {
            r.target == target
                && r.expires_at > now
                && r.hit_status
                    .as_ref()
                    .is_some_and(|s| s.definition == definition)
        }))
    }
    pub(in crate::game::battle::core) fn remove_status_predecessors(
        &mut self,
        states: &[StatusPredecessor],
        now: u64,
    ) -> Result<(), GameError> {
        for s in states {
            if self.sustained.records.contains_key(&s.activation) {
                self.end_sustained(s.activation, now, SustainedEndReason::Replaced)?;
            }
        }
        Ok(())
    }
}
