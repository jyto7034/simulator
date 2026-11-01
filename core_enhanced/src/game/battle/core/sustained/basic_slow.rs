use super::*;
use crate::game::ability::sustained::HitStatusKind;

impl BattleCore {
    /// Pure, live aggregation: strongest per definition, additive across
    /// definitions, capped below a complete basic-action lock.
    pub(in crate::game::battle::core) fn basic_speed_reduction(
        &self,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<u32, GameError> {
        let count = self.sustained.records.len() as u64;
        control::charge(
            count
                .checked_mul(count.saturating_add(1))
                .ok_or_else(|| overflow("basic_slow_query_work"))?,
            "basic_slow_query",
        )?;
        let mut sum = 0u32;
        for (id, record) in &self.sustained.records {
            if record.target != target || now >= record.expires_at {
                continue;
            }
            let Some(profile) = &record.hit_status else {
                continue;
            };
            let HitStatusKind::BasicAttackSlow { percent } = profile.kind else {
                continue;
            };
            let stronger = self.sustained.records.iter().any(|(other_id, other)| {
                other.target == target
                    && now < other.expires_at
                    && other.hit_status.as_ref().is_some_and(|p| {
                        p.definition == profile.definition
                            && p.kind
                                .basic_speed_reduction()
                                .is_some_and(|value| (value, other_id) > (percent, id))
                    })
            });
            if !stronger {
                sum = sum.saturating_add(percent).min(99);
            }
        }
        Ok(sum)
    }
}
