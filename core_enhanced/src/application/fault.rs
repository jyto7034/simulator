//! First runtime failure of a published battle, outside the rollback candidate.
use super::*;
use crate::game::{
    battle::runtime_control::{BattleRuntimeFault, ResourceFaultCode, RuntimeFaultCode},
    value_expr::{ValueExprFaultCode, ValueExprPhase},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BattleFaultOperation {
    Command(ApplicationCommandKind),
    Elapsed,
    HostTick,
    NextEvent,
    DisconnectPause,
}

/// No owned strings, content IDs, trace buffers, or history are retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BattleFaultCause {
    Native {
        fault: BattleRuntimeFault,
    },
    Expression {
        code: ValueExprFaultCode,
        phase: ValueExprPhase,
        node_index: usize,
    },
    Internal {
        code: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ApplicationBattleFault {
    pub authority: ApplicationBattleAuthority,
    pub operation: BattleFaultOperation,
    pub cause: BattleFaultCause,
}

fn cause(error: &GameError) -> Option<BattleFaultCause> {
    match error {
        GameError::BattleRuntimeFault(fault) => {
            use RuntimeFaultCode::*;
            let terminal = match fault.code {
                HealthCostUnavailable | ShieldUnavailable | InvalidControlTarget => false,
                Resource(code) => match code {
                    ResourceFaultCode::Insufficient | ResourceFaultCode::ReservationConflict => {
                        false
                    }
                    ResourceFaultCode::UnknownDefinition
                    | ResourceFaultCode::InvalidOwner
                    | ResourceFaultCode::OwnerKindMismatch
                    | ResourceFaultCode::StaleScope
                    | ResourceFaultCode::WrongLifetime
                    | ResourceFaultCode::StaleLease
                    | ResourceFaultCode::StaleRevision
                    | ResourceFaultCode::StaleReservation
                    | ResourceFaultCode::Limit
                    | ResourceFaultCode::CounterOverflow
                    | ResourceFaultCode::Invariant => true,
                },
                InvalidLimits | WorkLimit | CloneLimit | ActiveActionLimit
                | RetainedResultLimit | DeliveryLimit | ActorLimit | ProviderLimit
                | ControlStepLimit | StrikeSegmentLimit | PredeathBuffLimit | RetainedByteLimit
                | CounterOverflow | Invariant => true,
            };
            terminal.then_some(BattleFaultCause::Native { fault: *fault })
        }
        GameError::ValueExpressionFault(fault)
            if !matches!(fault.phase, ValueExprPhase::Compile) =>
        {
            Some(BattleFaultCause::Expression {
                code: fault.code,
                phase: fault.phase,
                node_index: fault.node_index,
            })
        }
        GameError::BattleStateRevisionOverflow
        | GameError::MissingResource(_)
        | GameError::InvalidUnitStats(_)
        | GameError::InvalidStaticData(_) => {
            Some(BattleFaultCause::Internal { code: error.code() })
        }
        _ => None,
    }
}

impl CoreApplication {
    /// Allocation-free first-fault query; available even when larger projections fail.
    pub fn battle_fault(&self) -> Option<&ApplicationBattleFault> {
        self.battle_fault.as_ref()
    }

    /// Shared guard for all mutating and durable-export ports and host writers.
    pub fn ensure_battle_not_faulted(&self) -> Result<(), ApplicationError> {
        if let Some(fault) = self.battle_fault {
            // The fixed-size capsule is independent of interrupted diagnostic scopes.
            Err(ApplicationError {
                kind: ApplicationErrorKind::BattleRuntimeFault,
                code: "battle_faulted".into(),
                details: ApplicationDocument(json!({"battle_fault": fault})),
            })
        } else {
            Ok(())
        }
    }

    pub(super) fn runtime_error(
        &mut self,
        error: GameError,
        operation: BattleFaultOperation,
        runtime_mutation: bool,
    ) -> ApplicationError {
        if runtime_mutation && self.battle_fault.is_none() {
            if let (Some(authority), Some(cause)) =
                (self.core.active_battle_authority(), cause(&error))
            {
                self.battle_fault = Some(ApplicationBattleFault {
                    authority: ApplicationBattleAuthority {
                        battle_uuid: authority.battle_uuid,
                        battle_state_revision: authority.battle_state_revision,
                    },
                    operation,
                    cause,
                });
                #[cfg(feature = "runtime-observation")]
                if let Some(observer) = self.observer.as_ref() {
                    observer.end_battle();
                }
            }
        }
        ApplicationError::from_domain(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_normal_resource_and_control_refusals_are_not_terminal() {
        use RuntimeFaultCode::*;
        for code in [
            HealthCostUnavailable,
            ShieldUnavailable,
            InvalidControlTarget,
            Resource(ResourceFaultCode::Insufficient),
            Resource(ResourceFaultCode::ReservationConflict),
        ] {
            assert!(cause(&GameError::BattleRuntimeFault(BattleRuntimeFault {
                code,
                site: "test",
                limit: None,
                requested: None
            }))
            .is_none());
        }
        for code in [
            Invariant,
            WorkLimit,
            CounterOverflow,
            Resource(ResourceFaultCode::Invariant),
            Resource(ResourceFaultCode::StaleLease),
        ] {
            assert!(cause(&GameError::BattleRuntimeFault(BattleRuntimeFault {
                code,
                site: "test",
                limit: None,
                requested: None
            }))
            .is_some());
        }
        for error in [
            GameError::InvalidAction,
            GameError::InsufficientResources,
            GameError::UnitNotFound,
            GameError::PositionOccupied,
            GameError::OutOfBounds,
        ] {
            assert!(cause(&error).is_none());
        }
    }
}
