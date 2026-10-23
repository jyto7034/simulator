use super::*;
use crate::game::ability::sustained::HitStatusKind;
impl BattleCore {
    pub(in crate::game::battle::core) fn hit_stun_until(
        &self,
        target: UnitInstanceId,
        now: Option<u64>,
    ) -> Option<u64> {
        self.sustained
            .records
            .values()
            .filter(|r| {
                r.target == target
                    && now.is_none_or(|n| n < r.expires_at)
                    && r.hit_status
                        .as_ref()
                        .is_some_and(|p| p.kind.is_hard_control())
            })
            .map(|r| r.expires_at.saturating_sub(1))
            .max()
    }
    pub(in crate::game::battle::core) fn hit_damage_taken_percent(
        &self,
        target: UnitInstanceId,
    ) -> Result<i32, GameError> {
        // Reapplication/overlap of one definition keeps the strongest live
        // contribution; different authored definitions add percentage points.
        control::charge(
            self.sustained.records.len() as u64,
            "hit_status_composition",
        )?;
        let mut total = 0i32;
        for (id, r) in &self.sustained.records {
            if r.target != target {
                continue;
            }
            let Some(p) = &r.hit_status else {
                continue;
            };
            if let HitStatusKind::StackDamageTaken { per_stack, stacks } = p.kind {
                let value = (u128::from(per_stack.numerator) * u128::from(stacks))
                    .div_ceil(u128::from(per_stack.denominator));
                total = total.saturating_add(value.min(i32::MAX as u128) as i32);
                continue;
            }
            let HitStatusKind::DamageTaken { percent } = p.kind else {
                continue;
            };
            control::charge(self.sustained.records.len() as u64, "hit_status_overlap")?;
            let stronger=self.sustained.records.iter().any(|(other,o)|o.target==target&&o.hit_status.as_ref().is_some_and(|q|q.definition==p.definition&&matches!(q.kind,HitStatusKind::DamageTaken{percent:n} if (n,other)>(percent,id))));
            if !stronger {
                total = total.saturating_add(percent as i32);
            }
        }
        Ok(total)
    }
}
impl BattleCore {
    pub(in crate::game::battle::core) fn hit_basic_locked(&self, target: UnitInstanceId) -> bool {
        self.sustained.records.values().any(|r| {
            r.target == target
                && r.hit_status
                    .as_ref()
                    .is_some_and(|p| p.kind == HitStatusKind::BasicLock)
        })
    }
    pub(super) fn merge_stacked_hit_status(
        &mut self,
        target: UnitInstanceId,
        profile: &mut crate::game::ability::sustained::HitStatusProfile,
        now: u64,
    ) -> Result<(), GameError> {
        let HitStatusKind::StackDamageTaken {
            per_stack,
            mut stacks,
        } = profile.kind
        else {
            return Ok(());
        };
        let owner = self
            .runtime_fact_source
            .fragment
            .as_ref()
            .map(|f| f.copy_uuid);
        if owner.is_none() {
            return Err(GameError::InvalidAction);
        }
        control::charge(self.sustained.records.len() as u64, "hit_stack_merge")?;
        let mut prior = Vec::new();
        for (id, r) in &self.sustained.records {
            if r.target == target
                && r.expires_at > now
                && r.source.fragment.as_ref().map(|f| f.copy_uuid) == owner
            {
                if let Some(p) = &r.hit_status {
                    if p.definition == profile.definition {
                        if let HitStatusKind::StackDamageTaken {
                            per_stack: old,
                            stacks: n,
                        } = p.kind
                        {
                            if old != per_stack {
                                return Err(GameError::InvalidAction);
                            }
                            stacks = stacks.checked_add(n).ok_or(GameError::InvalidAction)?;
                            prior.push(*id);
                        }
                    }
                }
            }
        }
        for id in prior {
            self.end_sustained(id, now, SustainedEndReason::Replaced)?;
        }
        profile.kind = HitStatusKind::StackDamageTaken { per_stack, stacks };
        Ok(())
    }
}

impl BattleCore {
    /// One declaration's reapplication replaces its own finite contribution.
    pub(in crate::game::battle::core) fn replace_hit_status_definition(
        &mut self,
        target: UnitInstanceId,
        definition: &str,
        now: u64,
    ) -> Result<(), GameError> {
        control::charge(self.sustained.records.len() as u64, "hit_status_replace")?;
        let ids: Vec<_> = self
            .sustained
            .records
            .iter()
            .filter(|(_, r)| {
                r.target == target
                    && r.hit_status
                        .as_ref()
                        .is_some_and(|p| p.definition == definition)
            })
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            self.end_sustained(id, now, SustainedEndReason::Replaced)?;
        }
        Ok(())
    }
}
