//! Private evidence from actual world producers, outside saved gameplay state.
//! No log, source graph, outcome String or RNG is owned by this fixed witness.

use serde::Serialize;
use uuid::Uuid;

use super::GameCore;
use crate::game::{battle::types::BattleVerdict, behavior::GameError, resources::GameState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub(crate) enum TrialBattleStatus {
    EntryPending,
    Running,
    Finished,
    Exited,
}

crate::authoring::clone_shape::enum_shape!(TrialBattleStatus {
    units: [EntryPending, Running, Finished, Exited],
    tuples: [],
    records: []
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct TrialBattleWitness {
    pub entry_ordinal: u64,
    pub battle_uuid: Uuid,
    pub time_zero_completed: bool,
    pub last_battle_time_ms: u64,
    pub status: TrialBattleStatus,
    pub terminal_verdict: Option<BattleVerdict>,
}

crate::authoring::clone_shape::struct_shape!(TrialBattleWitness {
    entry_ordinal,
    battle_uuid,
    time_zero_completed,
    last_battle_time_ms,
    status,
    terminal_verdict
});

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub(crate) struct TrialLifecycle {
    pub entries: u64,
    pub total_elapsed_ms: u64,
    pub current: Option<TrialBattleWitness>,
}

crate::authoring::clone_shape::struct_shape!(TrialLifecycle {
    entries,
    total_elapsed_ms,
    current
});

impl TrialLifecycle {
    pub(crate) fn is_ready(&self) -> bool {
        self.current.is_some_and(|battle| {
            battle.time_zero_completed
                && matches!(
                    battle.status,
                    TrialBattleStatus::Running | TrialBattleStatus::Finished
                )
        })
    }

    fn battle_mut(&mut self, uuid: Uuid) -> Result<&mut TrialBattleWitness, GameError> {
        self.current
            .as_mut()
            .filter(|battle| battle.battle_uuid == uuid)
            .ok_or_else(invalid_witness)
    }
}

fn invalid_witness() -> GameError {
    GameError::InvalidStaticData("invalid authoring battle lifecycle witness".into())
}

impl GameCore {
    pub(crate) fn authoring_lifecycle(&self) -> Option<TrialLifecycle> {
        self.authoring_lifecycle
    }

    pub(super) fn authoring_battle_entered(&mut self, uuid: Uuid) -> Result<(), GameError> {
        let Some(lifecycle) = self.authoring_lifecycle.as_mut() else {
            return Ok(());
        };
        let active = self
            .state
            .active_battle
            .as_ref()
            .ok_or_else(invalid_witness)?;
        if active.battle_uuid != uuid || active.execution.last_event_time_ms() != 0 {
            return Err(invalid_witness());
        }
        let ordinal = lifecycle
            .entries
            .checked_add(1)
            .ok_or_else(invalid_witness)?;
        lifecycle.entries = ordinal;
        lifecycle.current = Some(TrialBattleWitness {
            entry_ordinal: ordinal,
            battle_uuid: uuid,
            time_zero_completed: false,
            last_battle_time_ms: 0,
            status: TrialBattleStatus::EntryPending,
            terminal_verdict: None,
        });
        self.state
            .active_battle
            .as_mut()
            .ok_or_else(invalid_witness)?
            .battle
            .bind_authoring_observation(
                crate::game::battle::core::authoring_observation::ObservationKey {
                    entry_ordinal: ordinal,
                    battle_uuid: uuid,
                },
            )?;
        Ok(())
    }

    pub(super) fn authoring_time_zero_completed(&mut self, uuid: Uuid) -> Result<(), GameError> {
        let Some(lifecycle) = self.authoring_lifecycle.as_mut() else {
            return Ok(());
        };
        let battle = lifecycle.battle_mut(uuid)?;
        if battle.time_zero_completed || matches!(battle.status, TrialBattleStatus::Exited) {
            return Err(invalid_witness());
        }
        battle.time_zero_completed = true;
        if matches!(battle.status, TrialBattleStatus::EntryPending) {
            battle.status = TrialBattleStatus::Running;
        }
        Ok(())
    }

    pub(super) fn authoring_battle_advanced(
        &mut self,
        uuid: Uuid,
        previous_ms: u64,
        current_ms: u64,
    ) -> Result<(), GameError> {
        let Some(lifecycle) = self.authoring_lifecycle.as_mut() else {
            return Ok(());
        };
        let elapsed = current_ms
            .checked_sub(previous_ms)
            .ok_or_else(invalid_witness)?;
        let total = lifecycle
            .total_elapsed_ms
            .checked_add(elapsed)
            .ok_or_else(invalid_witness)?;
        let battle = lifecycle.battle_mut(uuid)?;
        if battle.last_battle_time_ms != previous_ms
            || !matches!(
                battle.status,
                TrialBattleStatus::EntryPending | TrialBattleStatus::Running
            )
        {
            return Err(invalid_witness());
        }
        battle.last_battle_time_ms = current_ms;
        lifecycle.total_elapsed_ms = total;
        Ok(())
    }

    pub(super) fn authoring_battle_finished(
        &mut self,
        uuid: Uuid,
        verdict: BattleVerdict,
    ) -> Result<(), GameError> {
        let Some(lifecycle) = self.authoring_lifecycle.as_mut() else {
            return Ok(());
        };
        let active = self
            .state
            .active_battle
            .as_ref()
            .ok_or_else(invalid_witness)?;
        if active.battle_uuid != uuid || !active.execution.is_finished() {
            return Err(invalid_witness());
        }
        let battle = lifecycle.battle_mut(uuid)?;
        if battle.last_battle_time_ms != active.execution.last_event_time_ms()
            || !matches!(
                battle.status,
                TrialBattleStatus::EntryPending | TrialBattleStatus::Running
            )
        {
            return Err(invalid_witness());
        }
        battle.status = TrialBattleStatus::Finished;
        battle.terminal_verdict = Some(verdict);
        Ok(())
    }

    /// Invalidates an existing witness after actual application root changes.
    /// This cannot create an entry or mark time zero complete from a snapshot.
    pub(super) fn reconcile_authoring_lifecycle(&mut self) -> Result<(), GameError> {
        if self.authoring_observation.is_some() {
            self.authoring_observation()?;
        }
        let Some(lifecycle) = self.authoring_lifecycle.as_mut() else {
            return Ok(());
        };
        let active_uuid = self
            .state
            .active_battle
            .as_ref()
            .map(|battle| battle.battle_uuid);
        let Some(battle) = lifecycle.current.as_mut() else {
            return if active_uuid.is_none() {
                Ok(())
            } else {
                Err(invalid_witness())
            };
        };
        match battle.status {
            TrialBattleStatus::EntryPending => return Err(invalid_witness()),
            TrialBattleStatus::Running => match active_uuid {
                Some(uuid) if uuid == battle.battle_uuid && battle.time_zero_completed => {}
                None => battle.status = TrialBattleStatus::Exited,
                Some(_) => return Err(invalid_witness()),
            },
            TrialBattleStatus::Finished => {
                if active_uuid.is_some() || !battle.time_zero_completed {
                    return Err(invalid_witness());
                }
                if !matches!(self.state.lifecycle.phase(), GameState::CombatResult { battle_uuid } if *battle_uuid == battle.battle_uuid)
                {
                    battle.status = TrialBattleStatus::Exited;
                }
            }
            TrialBattleStatus::Exited if active_uuid.is_some() => return Err(invalid_witness()),
            TrialBattleStatus::Exited => {}
        }
        Ok(())
    }
}
