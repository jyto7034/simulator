//! Pre-hit status conversion and copy-owned chase effects. No content-name dispatch.
mod chase;
use super::{types::SkillStepResult, BattleCore};
use crate::{
    authoring::{clone_shape::controlled_clone, control},
    game::{
        ability::{
            mobile_effect::*,
            status_strike::*,
            sustained::{HitModifierSource, HitStatusKind, HitStatusProfile},
            SkillDef, SkillEffectDef, SkillHitTargetFilter,
        },
        battle::{
            damage::{DamageModifiers, DamageSource, DamageSourceSnapshot, DamageType},
            event_log::SkillCastTarget,
            ids::UnitInstanceId,
            runtime_facts::{RuntimeFactSource, StatusPredecessor, StatusTransitionOrigin},
        },
        behavior::GameError,
        enums::Side,
        skill_fragment::FragmentSourceSnapshot,
    },
};
use std::{collections::BTreeMap, sync::Arc};
#[derive(Debug, Clone, PartialEq, Eq)]
struct StatusContext {
    owner: UnitInstanceId,
    side: Side,
    fragment: Arc<FragmentSourceSnapshot>,
    program: Arc<StatusStrikeDef>,
    source: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(StatusContext {
    owner,
    side,
    fragment,
    program,
    source
});
impl StatusContext {
    fn retained_extent(&self) -> Option<usize> {
        std::mem::size_of::<Self>()
            .checked_add(self.program.retained_extent()?)?
            .checked_add(self.source.retained_extent()?)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReactiveHitCapture {
    context: Arc<StatusContext>,
    before: Vec<StatusPredecessor>,
}
crate::authoring::clone_shape::struct_shape!(ReactiveHitCapture { context, before });
impl ReactiveHitCapture {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = std::mem::size_of::<Self>()
            .checked_add(self.context.retained_extent()?)?
            .checked_add(
                self.before
                    .capacity()
                    .checked_mul(std::mem::size_of::<StatusPredecessor>())?,
            )?;
        for p in &self.before {
            n = n.checked_add(p.retained_extent()?)?;
        }
        Some(n)
    }
}
impl BattleCore {
    pub(super) fn status_strike_declaration(skill: &SkillDef) -> Option<&Arc<StatusStrikeDef>> {
        skill.steps.iter().flat_map(|s| &s.effects).find_map(|e| {
            if let SkillEffectDef::StatusStrike { program } = &e.op {
                Some(program)
            } else {
                None
            }
        })
    }
    fn status_context(
        &self,
        owner: UnitInstanceId,
        require_active: bool,
        source: &RuntimeFactSource,
    ) -> Result<Option<Arc<StatusContext>>, GameError> {
        let Some(u) = self
            .units
            .get(&owner)
            .filter(|u| !u.corroded && (!require_active || u.is_active()))
        else {
            return Ok(None);
        };
        let Some(fragment) = &u.fragment_source else {
            return Ok(None);
        };
        let Some(program) = self
            .game_data
            .skill_data
            .get_by_id(&fragment.skill_id)
            .and_then(Self::status_strike_declaration)
        else {
            return Ok(None);
        };
        let mut source = source.independent_origin()?;
        source.actor = Some(owner);
        source.fragment = Some(fragment.clone());
        source.equipment = None;
        source.admit_owner("status_subscription_source")?;
        let context = StatusContext {
            owner,
            side: u.owner,
            fragment: fragment.clone(),
            program: program.clone(),
            source,
        };
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            context.retained_extent(),
            "status_context",
        )?;
        Ok(Some(Arc::new(context)))
    }
    fn apply_reactive_status(
        &mut self,
        c: &StatusContext,
        target: UnitInstanceId,
        status: &ReactiveStatusDef,
        before: Vec<StatusPredecessor>,
        now: u64,
    ) -> Result<bool, GameError> {
        let mut duration_changes = Vec::new();
        let mut vulnerabilities = Vec::new();
        for m in &status.modifiers {
            if let Some(copy) = self.equipped_normal_relic_copy(c.owner, &m.relic_definition) {
                match m.change {
                    ReactiveChange::Duration { negative, percent } => {
                        duration_changes.push((negative, percent))
                    }
                    ReactiveChange::Vulnerability {
                        percent,
                        duration_ms,
                    } => vulnerabilities.push((m, copy, percent, duration_ms)),
                }
            }
        }
        let duration = super::marks::modified_period(status.duration_ms, &duration_changes)?;
        let kind = match status.kind {
            ReactiveKind::Slow { percent } => HitStatusKind::Slow { percent },
            ReactiveKind::Freeze => HitStatusKind::Freeze,
        };
        let mut source = controlled_clone(&c.source)?;
        if !before.is_empty() {
            source = source.with_status_transition(StatusTransitionOrigin { previous: before })?;
        }
        let prior = std::mem::replace(&mut self.runtime_fact_source, source);
        let result = self.begin_committed_hit_status(
            c.owner,
            target,
            c.side,
            duration,
            HitStatusProfile {
                definition: status.definition.clone(),
                kind,
                modifier_source: None,
                vulnerability: None,
            },
            now,
        );
        self.runtime_fact_source = prior;
        let applied = result?;
        if applied {
            for (m, copy, percent, duration_ms) in vulnerabilities {
                let definition = format!("reactive-vulnerability:{copy}");
                self.replace_hit_status_definition(target, &definition, now)?;
                let points = u32::try_from(
                    u128::from(percent.numerator).div_ceil(u128::from(percent.denominator)),
                )
                .map_err(|_| GameError::InvalidAction)?;
                let prior =
                    std::mem::replace(&mut self.runtime_fact_source, controlled_clone(&c.source)?);
                let result = self.begin_committed_hit_status(
                    c.owner,
                    target,
                    c.side,
                    duration_ms,
                    HitStatusProfile {
                        definition,
                        kind: HitStatusKind::DamageTaken { percent: points },
                        modifier_source: Some(HitModifierSource {
                            copy_uuid: copy,
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
        Ok(applied)
    }
    pub(super) fn owned_damage_status(
        &mut self,
        snapshot: &DamageSourceSnapshot,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        if !self
            .units
            .get(&target)
            .is_some_and(|u| u.is_active() && u.owner != snapshot.source_side)
        {
            return Ok(());
        }
        if let Some(capture) = &snapshot.reactive_capture {
            let c = &capture.context;
            if self.has_status_definition(target, &c.program.to.definition, now)? {
                return Ok(());
            }
            if !capture.before.is_empty() {
                if self.apply_reactive_status(
                    c,
                    target,
                    &c.program.to,
                    controlled_clone(&capture.before)?,
                    now,
                )? {
                    self.remove_status_predecessors(&capture.before, now)?;
                }
                return Ok(());
            }
        }
        let Some(c) = self.status_context(snapshot.source_id, true, &snapshot.fact_source)? else {
            return Ok(());
        };
        if !c.program.owned_damage
            || self.has_status_definition(target, &c.program.to.definition, now)?
        {
            return Ok(());
        }
        let old = self.status_predecessors(target, c.owner, &c.program.from.definition, now)?;
        if !old.is_empty() && !c.program.from.refresh {
            return Ok(());
        }
        // Apply before removing the old contribution: immunity/failure leaves it intact.
        if self.apply_reactive_status(&c, target, &c.program.from, Vec::new(), now)? {
            self.remove_status_predecessors(&old, now)?;
        }
        Ok(())
    }
    pub(super) fn execute_status_strike(
        &mut self,
        owner: UnitInstanceId,
        p: &StatusStrikeDef,
        now: u64,
    ) -> Result<SkillStepResult, GameError> {
        let cast = self
            .runtime_fact_source
            .cast
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let target = self
            .active_skill_casts
            .get(&cast.cast_seq)
            .and_then(|c| c.flow.as_ref())
            .and_then(|f| f.accepted_target());
        let Some(SkillCastTarget::Direction { facing }) = target else {
            return Err(GameError::InvalidAction);
        };
        let source = controlled_clone(&self.runtime_fact_source)?;
        let c = self
            .status_context(owner, true, &source)?
            .ok_or(GameError::InvalidAction)?;
        if c.program.definition != p.definition {
            return Err(GameError::InvalidAction);
        }
        let tile = self
            .sample_unit_body_at(owner, now)
            .ok_or(GameError::InvalidAction)?
            .position
            .project_to_tile();
        let mut range = &p.range;
        for m in &p.range_modifiers {
            if self
                .equipped_normal_relic_copy(owner, &m.relic_definition)
                .is_some()
            {
                if let MobileChange::Range { pattern } = &m.change {
                    range = pattern;
                }
            }
        }
        let tiles = self
            .valid_tile_area_tiles(range, tile, facing)
            .ok_or(GameError::InvalidAction)?;
        let mut targets = self.collect_tile_area_targets_at(
            now,
            c.side,
            owner,
            &tiles,
            SkillHitTargetFilter::Enemies,
            false,
        );
        targets.sort_by(|a, b| {
            self.enemy_route_progress(*b)
                .total_cmp(&self.enemy_route_progress(*a))
                .then(a.cmp(b))
        });
        let attack = self
            .units
            .get(&owner)
            .ok_or(GameError::InvalidAction)?
            .stats
            .attack;
        let power = p
            .power
            .resolve(attack, 1)
            .map_err(GameError::InvalidStaticData)?;
        let mut primary = None;
        for target in targets {
            let position = self
                .sample_unit_body_at(target, now)
                .ok_or(GameError::InvalidAction)?
                .position;
            let mut snapshot = self
                .damage_source_snapshot_template_for_unit(
                    owner,
                    DamageSource::Ability,
                    p.damage_type,
                    power,
                    DamageModifiers::default(),
                    0,
                    now,
                    false,
                )
                .ok_or(GameError::InvalidAction)?;
            let capture = ReactiveHitCapture {
                context: c.clone(),
                before: self.status_predecessors(target, owner, &p.from.definition, now)?,
            };
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                capture.retained_extent(),
                "status_pre_hit_capture",
            )?;
            snapshot.reactive_capture = Some(Arc::new(capture));
            let result =
                self.execute_committed_damage(super::damage_runtime::CommittedDamageInput {
                    target_id: target,
                    source_snapshot: snapshot,
                    committed_at_ms: now,
                    policy: super::damage_runtime::CommittedDamagePolicy::GeneralCommand,
                });
            control::poll("status_strike_damage")?;
            if result.is_some_and(|r| r.accepted_hit_count > 0) && primary.is_none() {
                primary = Some(position);
            }
        }
        if let Some(position) = primary {
            self.spawn_status_mobile(&c, position, None, now)?;
        }
        Ok(SkillStepResult {
            applied_effect_count: 1,
            ..Default::default()
        })
    }
    pub(super) fn status_target_died(
        &mut self,
        target: UnitInstanceId,
        lethal: Option<&crate::game::battle::runtime_facts::DamageFact>,
        ordinary: bool,
        now: u64,
    ) -> Result<(), GameError> {
        if !ordinary
            || lethal.is_some_and(|f| {
                f.source.reaction
                    || f.source
                        .mobile_effect()
                        .is_some_and(|o| o.motion == MobileMotion::ChaseDelay)
            })
        {
            return Ok(());
        }
        let Some(position) = self
            .sample_unit_body_at(target, now)
            .map(|b| b.position)
            .or_else(|| self.units.get(&target).map(|u| u.body.position))
        else {
            return Ok(());
        };
        control::charge(self.units.len() as u64, "status_death_owners")?;
        let owners: Vec<_> = self.units.keys().copied().collect();
        for owner in owners {
            let Some(c) = self.status_context(owner, false, &RuntimeFactSource::default())? else {
                continue;
            };
            let Some(death) = &c.program.death else {
                continue;
            };
            let Some(copy) = self.equipped_normal_relic_copy(owner, &death.relic_definition) else {
                continue;
            };
            let mut states = Vec::new();
            for def in &death.states {
                states.extend(self.status_predecessors(target, owner, def, now)?);
            }
            if states.is_empty() {
                continue;
            }
            self.spawn_status_mobile(
                &c,
                position,
                Some(Arc::new(MobileDeathOrigin {
                    target,
                    relic: HitModifierSource {
                        copy_uuid: copy,
                        definition: death.relic_definition.clone(),
                        rule: death.definition.clone(),
                    },
                    states,
                })),
                now,
            )?;
        }
        Ok(())
    }
}
pub(super) use chase::ChaseState;
#[cfg(test)] mod tests;
