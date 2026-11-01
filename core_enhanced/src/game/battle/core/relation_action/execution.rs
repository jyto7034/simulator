use super::*;
use crate::game::battle::{
    damage::{DamageModifiers, DamageSource, DamageType},
    runtime_facts::{ActionCancelReason, ActionTerminalKind},
};
impl BattleCore {
    pub(in crate::game::battle::core) fn release_relation_action(
        &mut self,
        caster: UnitInstanceId,
        now: u64,
    ) -> Result<super::super::types::SkillStepResult, GameError> {
        let key = self.relation_key(caster).ok_or(GameError::InvalidAction)?;
        let o = self
            .sustained
            .relations
            .owners
            .get_mut(&key)
            .ok_or(GameError::InvalidAction)?;
        let admitted = o.pending.take().ok_or(GameError::InvalidAction)?;
        if self.runtime_fact_source.action.as_ref() != Some(&admitted.action) || o.active.is_some()
        {
            return Err(GameError::InvalidAction);
        }
        let waiting_basic = Self::relation_entry(&o.program, &admitted).next_basic;
        let source = self.reserve_finite_source(controlled_clone(&self.runtime_fact_source)?)?;
        let id = self.next_relation_id()?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<Active>()),
            "relation_activation_storage",
        )?;
        let active = Active {
            channel_started_at: None,
            id,
            admitted,
            source,
            snapshot: None,
            waiting_basic,
            bound_basic: None,
            bound_epoch: 0,
            start_stock: 0,
            primary_position: None,
            last_strike_primary: None,
            continuations: Vec::new(),
            next_at: None,
            next_self: None,
            stance: ZERO,
            hits: BTreeMap::new(),
            ramps: BTreeMap::new(),
            actual_hp_loss: 0,
            hit_index: 0,
        };
        self.admit_authoring_runtime_growth(1, "relation_activation_entity")?;
        self.sustained
            .relations
            .owners
            .get_mut(&key)
            .unwrap()
            .active = Some(active);
        if !waiting_basic {
            self.start_relation_execution(key, None, now)?;
        }
        Ok(super::super::types::SkillStepResult {
            applied_effect_count: 1,
            ..Default::default()
        })
    }
    pub(in crate::game::battle::core) fn bind_relation_basic(
        &mut self,
        snapshot: &DamageSourceSnapshot,
        epoch: u64,
    ) -> Result<(), GameError> {
        let Some(key) = self.relation_key(snapshot.source_id) else {
            return Ok(());
        };
        let Some(o) = self.sustained.relations.owners.get_mut(&key) else {
            return Ok(());
        };
        let Some(a) = o.active.as_mut().filter(|a| a.waiting_basic) else {
            return Ok(());
        };
        if a.bound_basic.is_some() && a.bound_epoch != epoch {
            a.bound_basic = None;
        }
        if a.bound_basic.is_none() {
            a.bound_basic = snapshot.fact_source.action.clone();
            a.bound_epoch = epoch;
        }
        control::poll("relation_basic_bound")
    }
    pub(in crate::game::battle::core) fn release_relation_basic(
        &mut self,
        source: &DamageSourceSnapshot,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<bool, GameError> {
        let Some(key) = self.relation_key(source.source_id) else {
            return Ok(false);
        };
        let Some(a) = self
            .sustained
            .relations
            .owners
            .get(&key)
            .and_then(|o| o.active.as_ref())
        else {
            return Ok(false);
        };
        if !a.waiting_basic
            || a.bound_basic.is_none()
            || a.bound_basic.as_ref() != source.fact_source.action.as_ref()
        {
            return Ok(false);
        }
        let mut source = controlled_clone(source)?;
        self.capture_release_gap(&mut source, now)?;
        let facing = self.units[&source.source_id]
            .facing_direction
            .unwrap_or(FacingDirection::Up);
        let admitted = &mut self
            .sustained
            .relations
            .owners
            .get_mut(&key)
            .unwrap()
            .active
            .as_mut()
            .unwrap()
            .admitted;
        admitted.primary = Some(target);
        admitted.facing = facing;
        self.grant_basic_attack_release_resonance(source.source_id, now);
        self.release_prepared_basic(&source)?;
        self.consume_basic_overlay(source.source_id, now)?;
        self.start_relation_execution(key, Some(source), now)?;
        Ok(true)
    }
    fn start_relation_execution(
        &mut self,
        key: Uuid,
        basic: Option<DamageSourceSnapshot>,
        now: u64,
    ) -> Result<(), GameError> {
        let o = &self.sustained.relations.owners[&key];
        let caster = o.caster;
        let stock = o.stock;
        let mut a = controlled_clone(o.active.as_ref().ok_or(GameError::InvalidAction)?)?;
        let e = Self::relation_entry(&o.program, &a.admitted);
        let plan = e.plan;
        let self_interval = e.self_damage.map(|v| v.0);
        let mut snapshot = if let Some(s) = basic {
            s
        } else {
            self.damage_source_snapshot_template_for_unit(
                caster,
                DamageSource::Ability,
                DamageType::Physical,
                0,
                DamageModifiers::default(),
                0,
                now,
                false,
            )
            .ok_or(GameError::InvalidAction)?
        };
        if snapshot.source == DamageSource::BasicAttack {
            let action = snapshot
                .fact_source
                .action
                .as_ref()
                .ok_or(GameError::InvalidAction)?;
            let ticket = self
                .runtime_facts
                .delivery(action)?
                .ok_or(GameError::InvalidAction)?;
            snapshot.fact_source.resource_capabilities = RuntimeFactSource::resource_capabilities(
                snapshot.fact_source.resource_context().cloned(),
                ticket.resource.clone(),
            )?;
            snapshot.fact_source.delivery = Some(ticket);
        } else {
            snapshot.fact_source = controlled_clone(&a.source)?;
        }
        a.start_stock = stock;
        a.channel_started_at = Some(self.channel_started_at(caster, now)?.unwrap_or(now));
        a.waiting_basic = false;
        a.primary_position = a
            .admitted
            .primary
            .and_then(|t| self.units.get(&t).map(|u| u.body.projected_tile()));
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            snapshot
                .fact_retained_extent()
                .and_then(|n| n.checked_add(std::mem::size_of::<Continuation>())),
            "relation_start_storage",
        )?;
        a.snapshot = Some(snapshot);
        a.continuations.push(Continuation::Plan {
            index: plan,
            paid: 0,
        });
        a.next_at = Some(now);
        a.next_self = self_interval
            .map(|n| now.checked_add(n.into()).ok_or(GameError::InvalidAction))
            .transpose()?;
        self.sustained
            .relations
            .owners
            .get_mut(&key)
            .unwrap()
            .active = Some(a);
        self.pause_channel_basic_attack(caster)?;
        let id = self.sustained.relations.owners[&key]
            .active
            .as_ref()
            .unwrap()
            .id;
        self.drive_relation_action(key, id, now)
    }
    pub(super) fn relation_activation_exists(&self, key: Uuid, id: u64) -> bool {
        self.sustained
            .relations
            .owners
            .get(&key)
            .and_then(|o| o.active.as_ref())
            .is_some_and(|a| a.id == id)
    }
    fn publish_relation_action(&mut self, key: Uuid, a: &Active) -> Result<(), GameError> {
        let copy = controlled_clone(a)?;
        self.sustained
            .relations
            .owners
            .get_mut(&key)
            .ok_or(GameError::InvalidAction)?
            .active = Some(copy);
        Ok(())
    }
    pub(in crate::game::battle::core) fn drive_relation_action(
        &mut self,
        key: Uuid,
        id: u64,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(o) = self.sustained.relations.owners.get(&key) else {
            return Ok(());
        };
        let Some(active) = o
            .active
            .as_ref()
            .filter(|a| a.id == id && a.next_at == Some(now) && !a.waiting_basic)
        else {
            return Ok(());
        };
        let p = o.program.clone();
        let caster = o.caster;
        let mut a = controlled_clone(active)?;
        let mut deadline = None;
        while let Some(frame) = a.continuations.pop() {
            control::charge(1, "relation_plan_step")?;
            if !self.relation_activation_exists(key, id) {
                return Ok(());
            }
            if !self.relation_refresh_primary(caster, &p, &mut a, now)? {
                return self.finish_relation_action(key, now, true, false);
            }
            self.admit_authoring_runtime_growth(2, "relation_continuation_entities")?;
            if a.continuations.len() + 2 > a.continuations.capacity() {
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    2usize.checked_mul(std::mem::size_of::<Continuation>()),
                    "relation_continuation_storage",
                )?;
                a.continuations.reserve_exact(2);
            }
            match frame {
                Continuation::ResumeAt { time } => {
                    if now < time {
                        a.continuations.push(Continuation::ResumeAt { time });
                        deadline = Some(time);
                        break;
                    }
                }
                Continuation::ClearStance => a.stance = ZERO,
                Continuation::Plan { index, paid } => match &p.plans[index].op {
                    ActionPlan::Sequence { first, next } => {
                        a.continuations
                            .push(Continuation::Plan { index: *next, paid });
                        a.continuations.push(Continuation::Plan {
                            index: *first,
                            paid,
                        });
                    }
                    ActionPlan::Wait { duration_ms } => {
                        a.continuations.push(Continuation::ResumeAt {
                            time: now
                                .checked_add((*duration_ms).into())
                                .ok_or(GameError::InvalidAction)?,
                        })
                    }
                    ActionPlan::Stance {
                        duration_ms,
                        reduction,
                    } => {
                        a.stance = *reduction;
                        a.continuations.push(Continuation::ClearStance);
                        a.continuations.push(Continuation::ResumeAt {
                            time: now
                                .checked_add((*duration_ms).into())
                                .ok_or(GameError::InvalidAction)?,
                        });
                    }
                    ActionPlan::Strike { .. } => {
                        self.publish_relation_action(key, &a)?;
                        self.relation_strike(key, &p, index, paid, &mut a, now)?;
                    }
                    ActionPlan::Spend { amount, body, .. } => {
                        let stock = self.sustained.relations.owners[&key].stock;
                        let cost = match amount {
                            Some(c) => c
                                .evaluate(a.start_stock, paid)
                                .ok_or(GameError::InvalidAction)?,
                            None => stock,
                        };
                        if stock < cost {
                            return self.finish_relation_action(key, now, true, false);
                        }
                        self.sustained.relations.owners.get_mut(&key).unwrap().stock = stock - cost;
                        a.last_strike_primary = None;
                        a.continuations.push(Continuation::AfterSpend {
                            plan: index,
                            paid: cost,
                            target: a.admitted.primary,
                        });
                        a.continuations.push(Continuation::Plan {
                            index: *body,
                            paid: cost,
                        });
                        control::poll("relation_stage_spent")?;
                    }
                    ActionPlan::Repeat { count, .. } => {
                        let count = count
                            .evaluate(a.start_stock, paid)
                            .ok_or(GameError::InvalidAction)?;
                        if count > 0 {
                            a.continuations.push(Continuation::Repeat {
                                plan: index,
                                paid,
                                start: now,
                                index: 0,
                                count,
                            });
                        }
                    }
                    ActionPlan::Timed {
                        base_ms,
                        per_count_ms,
                        count,
                        interval_ms,
                        delayed_first,
                        ..
                    } => {
                        let count = count
                            .evaluate(a.start_stock, paid)
                            .ok_or(GameError::InvalidAction)?;
                        let duration = u64::from(*base_ms)
                            .checked_add(u64::from(*per_count_ms) * u64::from(count))
                            .ok_or(GameError::InvalidAction)?;
                        a.continuations.push(Continuation::Timed {
                            plan: index,
                            paid,
                            end: now.checked_add(duration).ok_or(GameError::InvalidAction)?,
                            next: now
                                .checked_add(if *delayed_first {
                                    u64::from(*interval_ms)
                                } else {
                                    0
                                })
                                .ok_or(GameError::InvalidAction)?,
                        });
                    }
                },
                Continuation::AfterSpend { plan, paid, target } => {
                    if let ActionPlan::Spend {
                        after: Some(status),
                        ..
                    } = &p.plans[plan].op
                    {
                        if let Some(t) = a.last_strike_primary.or(target) {
                            self.relation_apply_status(
                                caster,
                                t,
                                &p,
                                status,
                                a.start_stock,
                                paid,
                                &a.source,
                                now,
                            )?;
                        }
                    }
                }
                Continuation::Repeat {
                    plan,
                    paid,
                    start,
                    index,
                    count,
                } => {
                    let ActionPlan::Repeat {
                        body,
                        time_ms,
                        timing,
                        ..
                    } = &p.plans[plan].op
                    else {
                        return Err(GameError::InvalidAction);
                    };
                    let offset = match timing {
                        PlanRepeatTiming::FixedTotal => {
                            (u128::from(index) * u128::from(*time_ms) / u128::from(count)) as u64
                        }
                        PlanRepeatTiming::Interval { delayed_first } => {
                            u64::from(index + u32::from(*delayed_first)) * u64::from(*time_ms)
                        }
                    };
                    let due = start.checked_add(offset).ok_or(GameError::InvalidAction)?;
                    if now < due {
                        a.continuations.push(Continuation::Repeat {
                            plan,
                            paid,
                            start,
                            index,
                            count,
                        });
                        deadline = Some(due);
                        break;
                    }
                    if index < count {
                        if index + 1 < count || matches!(timing, PlanRepeatTiming::FixedTotal) {
                            a.continuations.push(Continuation::Repeat {
                                plan,
                                paid,
                                start,
                                index: index + 1,
                                count,
                            });
                        }
                        a.continuations
                            .push(Continuation::Plan { index: *body, paid });
                    }
                }
                Continuation::Timed {
                    plan,
                    paid,
                    end,
                    next,
                } => {
                    if now >= end {
                        continue;
                    }
                    let due = next.min(end);
                    if now < due {
                        a.continuations.push(Continuation::Timed {
                            plan,
                            paid,
                            end,
                            next,
                        });
                        deadline = Some(due);
                        break;
                    }
                    let ActionPlan::Timed {
                        body, interval_ms, ..
                    } = &p.plans[plan].op
                    else {
                        return Err(GameError::InvalidAction);
                    };
                    a.continuations.push(Continuation::Timed {
                        plan,
                        paid,
                        end,
                        next: next
                            .checked_add((*interval_ms).into())
                            .ok_or(GameError::InvalidAction)?,
                    });
                    a.continuations
                        .push(Continuation::Plan { index: *body, paid });
                }
            }
            if !self.relation_activation_exists(key, id) {
                return Ok(());
            }
            self.publish_relation_action(key, &a)?;
        }
        if !self.relation_activation_exists(key, id) {
            return Ok(());
        }
        if a.continuations.is_empty() {
            self.publish_relation_action(key, &a)?;
            return self.finish_relation_action(key, now, true, false);
        }
        if a.next_self.is_some_and(|t| t <= now) {
            self.publish_relation_action(key, &a)?;
            let (interval, percent, damage_type) = Self::relation_entry(&p, &a.admitted)
                .self_damage
                .ok_or(GameError::InvalidAction)?;
            let source = self.sustained.relations.owners[&key].intrinsic_source();
            self.relation_self_damage(caster, source, percent, 1, damage_type, false, now)?;
            if !self.relation_activation_exists(key, id) {
                return Ok(());
            }
            a.next_self = Some(
                now.checked_add(interval.into())
                    .ok_or(GameError::InvalidAction)?,
            );
        }
        a.next_at = deadline.into_iter().chain(a.next_self).min();
        let time_ms = a.next_at.ok_or(GameError::InvalidAction)?;
        if time_ms <= now {
            return Err(GameError::InvalidAction);
        }
        let source = controlled_clone(&a.source)?;
        self.sustained
            .relations
            .owners
            .get_mut(&key)
            .unwrap()
            .active = Some(a);
        self.event_queue.push_with_context(
            BattleEvent::RelationActionTick {
                source: key,
                generation: id,
                time_ms,
            },
            None,
            source,
        );
        control::poll("relation_action_yielded")
    }
    pub(in crate::game::battle::core) fn stop_relation_action(
        &mut self,
        caster: UnitInstanceId,
        now: u64,
        manual: bool,
    ) -> Result<(), GameError> {
        let Some(key) = self.relation_key(caster) else {
            return Ok(());
        };
        let Some(o) = self.sustained.relations.owners.get(&key) else {
            return Ok(());
        };
        let Some(a) = &o.active else {
            return Ok(());
        };
        if manual && !Self::relation_entry(&o.program, &a.admitted).manual_stop {
            return Err(GameError::InvalidAction);
        }
        if a.waiting_basic && !manual {
            return Ok(());
        }
        self.finish_relation_action(key, now, false, false)
    }
    pub(super) fn finish_relation_action(
        &mut self,
        key: Uuid,
        now: u64,
        natural: bool,
        terminal: bool,
    ) -> Result<(), GameError> {
        let Some(o) = self.sustained.relations.owners.get_mut(&key) else {
            return Ok(());
        };
        let Some(a) = o.active.take() else {
            return Ok(());
        };
        let caster = o.caster;
        self.event_queue
            .discard_relation_events(Some(key), false, true)?;
        if !natural {
            self.runtime_facts.terminal(
                &a.admitted.action,
                ActionTerminalKind::Cancelled(ActionCancelReason::Interrupted),
                now,
            )?;
        }
        if let Some(snapshot) = &a.snapshot {
            if snapshot.source == DamageSource::BasicAttack {
                self.finish_basic_lifesteal(snapshot, a.actual_hp_loss, now)?;
                if let Some(ticket) = &snapshot.fact_source.delivery {
                    self.runtime_facts.close_delivery(ticket, now)?;
                }
            }
        }
        self.close_source_resource_delivery(a.source.resource_delivery().map(|d| d.as_ref()))?;
        if let Some(ticket) = &a.source.delivery {
            self.runtime_facts.close_delivery(ticket, now)?;
        }
        self.reconcile_resonance_use(caster);
        if !terminal && !a.waiting_basic && self.units.get(&caster).is_some_and(|u| u.is_active()) {
            self.event_queue.push(BattleEvent::AttackStart {
                time_ms: now,
                attacker_instance_id: caster,
                target_instance_id: None,
                schedule_next: true,
                cause: Default::default(),
            });
        }
        control::poll("relation_action_finished")
    }
}
