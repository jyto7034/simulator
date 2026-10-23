use super::*;
use crate::game::battle::runtime_facts::MobileEffectOrigin;
pub(in crate::game::battle::core) struct MobileDamageInput<'a> {
    pub profile: &'a MobileProfile,
    pub units: u32,
    pub template: &'a DamageSourceSnapshot,
    pub source: &'a RuntimeFactSource,
    pub owner: UnitInstanceId,
    pub side: Side,
    pub relic_copies: &'a BTreeMap<String, Uuid>,
    pub origin: MobileEffectOrigin,
    pub capture: Option<Arc<MobileCapture>>,
    pub target: UnitInstanceId,
    pub now: u64,
}
impl BattleCore {
    pub(in crate::game::battle::core) fn execute_mobile_hit(
        &mut self,
        input: MobileDamageInput<'_>,
    ) -> Result<(), GameError> {
        let MobileDamageInput {
            profile: p,
            units,
            template,
            source,
            owner,
            side,
            relic_copies,
            origin,
            capture,
            target,
            now,
        } = input;
        let mut power = p
            .power
            .resolve(template.source_attack, units)
            .map_err(GameError::InvalidStaticData)?;
        let percentages = p
            .modifiers
            .iter()
            .filter_map(|m| {
                if let MobileChange::Power { percent } = m.change {
                    Some((false, percent))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if power > 0 {
            power = super::super::marks::modified_period(power, &percentages)?;
        }
        let mut snapshot = controlled_clone(template)?;
        snapshot.base_damage = power;
        snapshot.damage_type = p.damage_type;
        snapshot.mobile_capture = capture;
        snapshot.fact_source = snapshot.fact_source.with_mobile_effect(origin)?;
        let outcome =
            self.execute_committed_damage(super::super::damage_runtime::CommittedDamageInput {
                target_id: target,
                source_snapshot: snapshot,
                committed_at_ms: now,
                policy: super::super::damage_runtime::CommittedDamagePolicy::GeneralCommand,
            });
        control::poll("mobile_explosion_damage")?;
        if outcome.is_some_and(|r| r.actual_hp_loss > 0)
            && self.units.get(&target).is_some_and(|u| u.is_active())
        {
            for m in &p.modifiers {
                if let MobileChange::Vulnerability {
                    percent,
                    duration_ms,
                } = m.change
                {
                    let points = u32::try_from(
                        u128::from(percent.numerator).div_ceil(u128::from(percent.denominator)),
                    )
                    .map_err(|_| GameError::InvalidAction)?;
                    let copy = relic_copies.get(&m.id).ok_or(GameError::InvalidAction)?;
                    let definition = format!("mobile-vulnerability:{copy}");
                    self.replace_hit_status_definition(target, &definition, now)?;
                    let prior =
                        std::mem::replace(&mut self.runtime_fact_source, controlled_clone(source)?);
                    let result = self.begin_committed_hit_status(
                        owner,
                        target,
                        side,
                        duration_ms,
                        HitStatusProfile {
                            definition,
                            kind: HitStatusKind::DamageTaken { percent: points },
                            modifier_source: Some(HitModifierSource {
                                copy_uuid: *copy,
                                definition: m.relic_definition.clone(),
                                rule: m.id.clone(),
                            }),
                            vulnerability: None,
                        },
                        now,
                    );
                    self.runtime_fact_source = prior;
                    result?;
                }
            }
        }
        Ok(())
    }
}
