//! Positive direct-strike progression. Counts belong to the captured copy,
//! authored rule, cast and target, and survive only while that cast has carriers.
use super::{relic_causal::CausalSourceData, BattleCore};
use crate::game::{
    battle::{
        damage::{DamageSource, DamageSourceSnapshot},
        ids::UnitInstanceId,
        runtime_control as control,
        runtime_facts::{RuntimeFactSource, StrikeOrdinal},
    },
    behavior::GameError,
};
use std::sync::Arc;
fn overflow(site: &'static str) -> GameError {
    control::reject(control::BattleRuntimeFault {
        code: control::RuntimeFaultCode::CounterOverflow,
        site,
        limit: None,
        requested: None,
    })
}
use uuid::Uuid;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PreNumericAmount {
    pub base: u32,
    pub program: crate::game::relic::structural::compose::NumericProgram,
}
crate::authoring::clone_shape::copy_shape!(PreNumericAmount);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct WedgeKey {
    pub copy: Uuid,
    pub rule: String,
    pub cast: u64,
    pub target: UnitInstanceId,
}
crate::authoring::clone_shape::struct_shape!(WedgeKey {
    copy,
    rule,
    cast,
    target
});
#[derive(Debug, Clone)]
pub(super) struct WedgeProgress {
    count: u64,
    last_strike: StrikeOrdinal,
    last_n: u64,
}
crate::authoring::clone_shape::struct_shape!(WedgeProgress {
    count,
    last_strike,
    last_n
});
impl BattleCore {
    pub(super) fn source_has_wedge(&self, s: &RuntimeFactSource) -> bool {
        s.causal().and_then(|c| c.program.as_ref()).is_none()
            && !s.reaction
            && s.independent_object().is_none()
            && s.relic_effective()
                .is_some_and(|b| !b.plan.wedge_rules.is_empty())
    }
    pub(super) fn bind_pre_numeric(
        &self,
        s: &RuntimeFactSource,
        amount: PreNumericAmount,
    ) -> Result<RuntimeFactSource, GameError> {
        if !self.source_has_wedge(s) {
            return Ok(s.clone());
        }
        control::admit_retained_growth(
            0,
            Some(
                std::mem::size_of::<PreNumericAmount>()
                    + std::mem::size_of::<CausalSourceData>()
                    + 128,
            ),
            "wedge_amount_owner",
        )?;
        s.with_causal(Some(Arc::new(CausalSourceData {
            context: s.causal().and_then(|c| c.context.clone()),
            program: s.causal().and_then(|c| c.program.clone()),
            amount: Some(Arc::new(amount)),
            cause: s.causal().and_then(|c| c.cause.clone()),
        })))
    }
    fn wedge_keys(
        &self,
        s: &DamageSourceSnapshot,
        target: UnitInstanceId,
    ) -> Result<Vec<(WedgeKey, u32, Option<u32>)>, GameError> {
        if s.source != DamageSource::Ability || !self.source_has_wedge(&s.fact_source) {
            return Ok(Vec::new());
        }
        let Some(binding) = s.fact_source.relic_effective() else {
            return Ok(Vec::new());
        };
        let Some(cast) = &s.fact_source.cast else {
            return Ok(Vec::new());
        };
        let Some(authored) = &s.fact_source.authored else {
            return Ok(Vec::new());
        };
        if binding.effect(&authored.site).is_some_and(|e| {
            matches!(&*e.delivery,
            crate::game::ability::DeliveryDef::TileArea {area,..} if area.duration_ms>0)
        }) {
            return Ok(Vec::new());
        }

        let mut keys = Vec::new();
        control::charge(binding.plan.wedge_rules.len() as u64, "wedge_rule_scan")?;
        for rule in &binding.plan.wedge_rules {
            control::charge(rule.effect_sites.len() as u64, "wedge_original_site_scan")?;
            if rule.effect_sites.contains(&authored.site) {
                control::admit_retained_growth(
                    0,
                    Some(
                        (keys.len() + 1) * std::mem::size_of::<(WedgeKey, u32, Option<u32>)>()
                            + 128
                            + rule.contribution.rule_id.len(),
                    ),
                    "wedge_key_selection",
                )?;
                keys.reserve_exact(1);
                keys.push((
                    WedgeKey {
                        copy: rule.contribution.copy_uuid,
                        rule: rule.contribution.rule_id.clone(),
                        cast: cast.cast_seq,
                        target,
                    },
                    rule.increment_percent,
                    rule.cap_percent,
                ));
            }
        }
        Ok(keys)
    }
    pub(super) fn apply_wedge(
        &mut self,
        s: &mut DamageSourceSnapshot,
        target: UnitInstanceId,
        strike: StrikeOrdinal,
    ) -> Result<(), GameError> {
        let keys = self.wedge_keys(s, target)?;
        if keys.is_empty() {
            return Ok(());
        }
        let amount = s
            .fact_source
            .causal()
            .and_then(|c| c.amount.as_ref())
            .ok_or_else(|| control::invariant("wedge_pre_numeric_capture"))?;
        let mut percent = 0_i128;
        for (key, rate, cap) in keys {
            control::charge(1, "wedge_progress_read")?;
            let progress = self
                .runtime_facts
                .causal
                .as_deref()
                .and_then(|c| c.wedges.get(&key));
            let n = match progress {
                Some(p) if p.last_strike == strike => Some(p.last_n),
                Some(p) => p.count.checked_add(1),
                None => Some(1),
            }
            .ok_or_else(|| control::invariant("wedge_strike_count"))?;
            percent = percent
                .checked_add(
                    i128::from(n)
                        .checked_mul(i128::from(rate))
                        .ok_or_else(|| overflow("wedge_damage_arithmetic"))?
                        .min(cap.map_or(i128::MAX, i128::from)),
                )
                .ok_or_else(|| overflow("wedge_damage_arithmetic"))?;
        }
        use crate::game::relic::structural::{
            authoring::{EffectOperation, RelicProperty, TypedRelicValue},
            capabilities,
        };
        let descriptor =
            capabilities::descriptor(EffectOperation::Damage, RelicProperty::DamageAmount)
                .ok_or_else(|| control::invariant("wedge_damage_descriptor"))?;
        let value = amount
            .program
            .apply_additional_percent(
                TypedRelicValue::Hp(i64::from(amount.base)),
                descriptor,
                percent,
            )
            .map_err(|_| overflow("wedge_damage_arithmetic"))?;
        s.base_damage =
            u32::try_from(value.value()).map_err(|_| overflow("wedge_damage_arithmetic"))?;
        Ok(())
    }
    pub(super) fn commit_wedge(
        &mut self,
        s: &DamageSourceSnapshot,
        target: UnitInstanceId,
        strike: StrikeOrdinal,
    ) -> Result<(), GameError> {
        for (key, _, _) in self.wedge_keys(s, target)? {
            control::charge(1, "wedge_positive_strike_commit")?;
            let state = self.causal_state()?;
            if state
                .wedges
                .get(&key)
                .is_some_and(|p| p.last_strike == strike)
            {
                continue;
            }
            // A committed positive strike consumes this exact captured rule;
            // expression previews and later segments of the same strike do not.
            if let Some(binding)=s.fact_source.relic_effective(){
                if let Some(rule)=binding.plan.wedge_rules.iter().find(|r|r.contribution.copy_uuid==key.copy&&r.contribution.rule_id==key.rule){
                    if let Some(coverage)=self.authoring_observation.as_mut().and_then(|o|o.coverage.as_mut()){
                        coverage.price_record()?;coverage.record_contribution(&rule.contribution,crate::mod_validation::CoverageStage::ConsumedOrEffect,"wedge_positive");
                    }
                }
            }
            let state=self.causal_state()?;
            let n = state
                .wedges
                .get(&key)
                .map_or(Some(1), |p| p.count.checked_add(1))
                .ok_or_else(|| overflow("wedge_damage_arithmetic"))?;
            if !state.wedges.contains_key(&key) {
                super::relic_causal::admit_tree_row::<WedgeKey, WedgeProgress>(
                    state.wedges.len(),
                    key.rule.len() + 64,
                    "wedge_progress_owner",
                )?;
            }
            state.wedges.insert(
                key,
                WedgeProgress {
                    count: n,
                    last_strike: strike,
                    last_n: n,
                },
            );
        }
        Ok(())
    }
}
