use super::*;
use crate::game::battle::runtime_facts::{ActionCancelReason, ActionTerminalKind};
impl BattleCore {
    pub(in crate::game::battle::core) fn validate_prepared_volley_start(
        &self,
        caster: UnitInstanceId,
        skill: &SkillDef,
    ) -> Result<(), GameError> {
        if !Self::prepared_volley_skill(skill) {
            return Ok(());
        }
        let o = self
            .sustained
            .volleys
            .owners
            .get(&self.volley_key(caster).ok_or(GameError::InvalidAction)?)
            .ok_or(GameError::InvalidAction)?;
        if o.caster != Some(caster)
            || o.active.is_some()
            || (continuous(&o.program).is_none() && !Self::volley_payable(o))
        {
            return Err(GameError::InvalidAction);
        }
        Ok(())
    }
    fn volley_payable(o: &Owner) -> bool {
        o.balance >= 1 || (o.can_borrow && o.balance > -i64::from(o.program.ledger.credit_limit))
    }
    pub(in crate::game::battle::core) fn release_prepared_volley(
        &mut self,
        caster: UnitInstanceId,
        now: u64,
    ) -> Result<super::super::types::SkillStepResult, GameError> {
        let key = self.volley_key(caster).ok_or(GameError::InvalidAction)?;
        let o = self
            .sustained
            .volleys
            .owners
            .get(&key)
            .ok_or(GameError::InvalidAction)?;
        if o.active.is_some() || (continuous(&o.program).is_none() && !Self::volley_payable(o)) {
            return Err(GameError::InvalidAction);
        }
        let c = continuous(&o.program);
        let first = o.program.policy.first_ms;
        let deadline = now
            .checked_add(o.program.policy.deadline_ms.into())
            .ok_or(GameError::InvalidAction)?;
        let source = self.reserve_finite_source(controlled_clone(&self.runtime_fact_source)?)?;
        let id = self.next_volley_id()?;
        self.admit_authoring_runtime_growth(1, "prepared_volley_activation")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<Active>() + 128),
            "prepared_volley_activation_storage",
        )?;
        let a = Active {
            channel_started_at: self.channel_started_at(caster, now)?.unwrap_or(now),
            id,
            caster,
            source,
            phase: if c.is_some() {
                VolleyPhase::Continuous
            } else {
                VolleyPhase::Preparing
            },
            prepared: 0,
            created: 0,
            deadline,
            next_at: Some(if c.is_some() {
                now
            } else {
                now.checked_add(first.into())
                    .ok_or(GameError::InvalidAction)?
                    .min(deadline)
            }),
            next_cost: c
                .map(|(_, ms, _)| now.checked_add(ms.into()).ok_or(GameError::InvalidAction))
                .transpose()?,
            flight: None,
            hits: BTreeMap::new(),
        };
        self.sustained.volleys.owners.get_mut(&key).unwrap().active = Some(a);
        self.pause_channel_basic_attack(caster)?;
        self.schedule_prepared_volley(key)?;
        control::poll("prepared_volley_started")?;
        Ok(super::super::types::SkillStepResult {
            applied_effect_count: 1,
            ..Default::default()
        })
    }
    pub(super) fn schedule_prepared_volley(&mut self, key: Uuid) -> Result<(), GameError> {
        let Some(a) = self
            .sustained
            .volleys
            .owners
            .get(&key)
            .and_then(|o| o.active.as_ref())
        else {
            return Ok(());
        };
        if let Some(time_ms) = a.next_at.into_iter().chain(a.next_cost).min() {
            self.event_queue.push(BattleEvent::PreparedVolleyTick {
                source: key,
                generation: a.id,
                time_ms,
            });
        }
        Ok(())
    }
    pub(in crate::game::battle::core) fn drive_prepared_volley(
        &mut self,
        key: Uuid,
        id: u64,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(o) = self.sustained.volleys.owners.get(&key) else {
            return Ok(());
        };
        let Some(a) = o.active.as_ref().filter(|a| a.id == id) else {
            return Ok(());
        };
        if a.next_at != Some(now) && a.next_cost != Some(now) {
            return Ok(());
        }
        let p = o.program.clone();
        let caster = a.caster;
        if a.next_cost == Some(now) {
            let (percent, ms, _) = continuous(&p).ok_or(GameError::InvalidAction)?;
            let u = self.units.get(&caster).ok_or(GameError::InvalidAction)?;
            let cost = u32::try_from(
                (u128::from(u.stats.max_health) * u128::from(percent.numerator))
                    .div_ceil(u128::from(percent.denominator) * 100),
            )
            .map_err(|_| GameError::InvalidAction)?
            .min(u.stats.current_health);
            let source = controlled_clone(&a.source)?;
            let prior = std::mem::replace(&mut self.runtime_fact_source, source);
            let result = self.pay_health_cost(caster, cost, true, now);
            self.runtime_fact_source = prior;
            result?;
            let Some(a) = self
                .sustained
                .volleys
                .owners
                .get_mut(&key)
                .and_then(|o| o.active.as_mut())
                .filter(|a| a.id == id)
            else {
                return Ok(());
            };
            a.next_cost = Some(now.checked_add(ms.into()).ok_or(GameError::InvalidAction)?);
        }
        let Some(a) = self
            .sustained
            .volleys
            .owners
            .get(&key)
            .and_then(|o| o.active.as_ref())
            .filter(|a| a.id == id)
        else {
            return Ok(());
        };
        if a.next_at != Some(now) {
            return self.schedule_prepared_volley(key);
        }
        match a.phase {
            VolleyPhase::Preparing | VolleyPhase::Decaying => {
                let o = self.sustained.volleys.owners.get_mut(&key).unwrap();
                let payable = Self::volley_payable(o);
                let a = o.active.as_mut().unwrap();
                if a.phase == VolleyPhase::Decaying || now >= a.deadline {
                    a.phase = VolleyPhase::Decaying;
                    a.prepared = a.prepared.saturating_sub(1);
                    a.next_at = Some(
                        now.checked_add(p.policy.decay_ms.into())
                            .ok_or(GameError::InvalidAction)?,
                    );
                    if a.prepared == 0 {
                        return self.finish_prepared_volley(key, now, true, false);
                    }
                } else if payable && a.created < p.policy.maximum {
                    if o.balance < 1 {
                        o.borrowed = true;
                    }
                    o.balance = o.balance.checked_sub(1).ok_or(GameError::InvalidAction)?;
                    a.prepared = a.prepared.checked_add(1).ok_or(GameError::InvalidAction)?;
                    a.created += 1;
                    if a.created == p.policy.maximum {
                        a.deadline = a.deadline.min(
                            now.checked_add(p.policy.hold_ms.into())
                                .ok_or(GameError::InvalidAction)?,
                        );
                        a.next_at = Some(a.deadline);
                    } else {
                        // Next interval is the (created+1)th point of the authored linear sequence.
                        let den = u64::from(p.policy.maximum.saturating_sub(1).max(1));
                        let numerator = u64::from(p.policy.first_ms) * den
                            - u64::from(p.policy.first_ms - p.policy.last_ms)
                                * u64::from(a.created);
                        let interval = numerator.div_ceil(den);
                        a.next_at = Some(
                            now.checked_add(interval)
                                .ok_or(GameError::InvalidAction)?
                                .min(a.deadline),
                        );
                    }
                    control::poll("prepared_volley_portal_paid")?;
                } else {
                    a.next_at = Some(
                        now.checked_add(u64::from(p.policy.last_ms))
                            .ok_or(GameError::InvalidAction)?
                            .min(a.deadline),
                    );
                }
            }
            VolleyPhase::Firing | VolleyPhase::Continuous => {
                self.sustained
                    .volleys
                    .owners
                    .get_mut(&key)
                    .unwrap()
                    .active
                    .as_mut()
                    .unwrap()
                    .next_at = None;
                self.release_next_volley_projectile(key, id, now)?;
            }
        }
        self.schedule_prepared_volley(key)
    }
    pub(in crate::game::battle::core) fn reinput_prepared_volley(
        &mut self,
        caster: UnitInstanceId,
        skill: &SkillId,
        id: u64,
        now: u64,
    ) -> Result<(), GameError> {
        let key = self.volley_key(caster).ok_or(GameError::InvalidAction)?;
        // Expiry/collapse is evaluated before the same-time input even when no clock event was requested.
        self.drive_prepared_volley(key, id, now)?;
        let o = self
            .sustained
            .volleys
            .owners
            .get(&key)
            .ok_or(GameError::InvalidAction)?;
        let a = o
            .active
            .as_ref()
            .filter(|a| a.id == id)
            .ok_or(GameError::InvalidAction)?;
        if &o.fragment.skill_id != skill || o.caster != Some(caster) {
            return Err(GameError::InvalidAction);
        }
        if a.phase == VolleyPhase::Continuous {
            return self.finish_prepared_volley(key, now, false, false);
        }
        if a.phase == VolleyPhase::Firing
            || a.prepared == 0
            || self.unit_is_in_defense_stance(caster)
            || self.unit_is_field_working(caster)
            || self
                .unit_control_state(caster, now)
                .is_some_and(|s| s.hard_cc_active() || s.silenced)
        {
            return Err(GameError::InvalidAction);
        }
        let a = self
            .sustained
            .volleys
            .owners
            .get_mut(&key)
            .unwrap()
            .active
            .as_mut()
            .unwrap();
        a.phase = VolleyPhase::Firing;
        a.next_at = Some(now);
        control::poll("prepared_volley_frozen")?;
        self.drive_prepared_volley(key, id, now)
    }
    pub(in crate::game::battle::core) fn stop_prepared_volley(
        &mut self,
        caster: UnitInstanceId,
        now: u64,
        direct: bool,
    ) -> Result<(), GameError> {
        let Some(key) = self.volley_key(caster) else {
            return Ok(());
        };
        let Some(o) = self.sustained.volleys.owners.get(&key) else {
            return Ok(());
        };
        if !direct
            && o.program
                .modifiers
                .iter()
                .any(|m| matches!(m.change, VolleyChange::Protection))
            && self.units.get(&caster).is_some_and(|u| u.is_active())
        {
            return Ok(());
        }
        self.finish_prepared_volley(key, now, false, false)
    }
    pub(in crate::game::battle::core) fn finish_prepared_volley(
        &mut self,
        key: Uuid,
        now: u64,
        natural: bool,
        terminal: bool,
    ) -> Result<(), GameError> {
        let Some(a) = self
            .sustained
            .volleys
            .owners
            .get_mut(&key)
            .and_then(|o| o.active.take())
        else {
            return Ok(());
        };
        self.event_queue
            .discard_prepared_volley_events(Some(key), false)?;
        if !natural {
            if let Some(action) = &a.source.action {
                self.runtime_facts.terminal(
                    action,
                    ActionTerminalKind::Cancelled(ActionCancelReason::Interrupted),
                    now,
                )?;
            }
        }
        self.close_source_resource_delivery(a.source.resource_delivery().map(Arc::as_ref))?;
        if let Some(ticket) = &a.source.delivery {
            self.runtime_facts.close_delivery(ticket, now)?;
        }
        self.reconcile_resonance_use(a.caster);
        if !terminal && self.units.get(&a.caster).is_some_and(|u| u.is_active()) {
            self.event_queue.push(BattleEvent::AttackStart {
                time_ms: now,
                attacker_instance_id: a.caster,
                target_instance_id: None,
                schedule_next: true,
                cause: Default::default(),
            });
        }
        control::poll("prepared_volley_finished")
    }
    pub(in crate::game::battle::core) fn clear_prepared_volleys(
        &mut self,
        now: u64,
    ) -> Result<(), GameError> {
        let keys: Vec<_> = self.sustained.volleys.owners.keys().copied().collect();
        for key in keys {
            self.finish_prepared_volley(key, now, false, true)?;
            let o = &self.sustained.volleys.owners[&key];
            let after = CreditCarry {
                definition: controlled_clone(&o.program.ledger.definition)?,
                unpaid: u32::try_from(o.balance.saturating_neg().max(0))
                    .map_err(|_| GameError::InvalidAction)?,
                borrowed: o.borrowed,
            };
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(std::mem::size_of::<CreditSettlement>() + 256 + after.definition.len()),
                "credit_terminal_storage",
            )?;
            self.sustained.volleys.settlements.push(CreditSettlement {
                copy: key,
                before: o.before.as_ref().map(controlled_clone).transpose()?,
                after,
            });
        }
        let flights: Vec<_> = self.sustained.volleys.flights.keys().copied().collect();
        for id in flights {
            let f = self.sustained.volleys.flights.remove(&id).unwrap();
            self.finish_volley_projectile(f, now, VolleyFlightEnd::BattleEnded, false)?;
        }
        self.sustained.volleys.owners.clear();
        self.sustained.volleys.marks.clear();
        self.sustained.volleys.next_flight = None;
        self.event_queue
            .discard_prepared_volley_events(None, true)?;
        Ok(())
    }
}
