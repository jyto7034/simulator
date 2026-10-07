use super::GameCore;
use crate::game::behavior::{ActionKind, GameError};

/// Exact application owner for a routed Core operation. The owner is a routing
/// fact, not permission to mutate the authoritative root directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ApplicationOwner {
    MapRun,
    Headquarters,
    Maintenance,
    Shop,
    Reward,
    Event,
    VisualNovel,
    Battle,
    Admin,
    ActiveRunRecovery,
    PlayerProfile,
    #[cfg(test)]
    TestFixture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ApplicationRoute {
    Behavior(ActionKind),
    BattleElapsedAdvance,
    BattleServerTick,
    BattleEventBucket,
    BattleDisconnectPause,
    AdminCommand,
    ActiveRunRollbackRestore,
    ActiveRunContinue,
    MergeEncounterCompendiumProfile,
    MergePlayerProfile,
    ResetPlayerProfile,
    #[cfg(test)]
    SetEnkephalinFixture,
}

impl ApplicationRoute {
    pub(super) const fn owner(self) -> ApplicationOwner {
        match self {
            Self::Behavior(action) => behavior_owner(action),
            Self::BattleElapsedAdvance
            | Self::BattleServerTick
            | Self::BattleEventBucket
            | Self::BattleDisconnectPause => ApplicationOwner::Battle,
            Self::AdminCommand => ApplicationOwner::Admin,
            Self::ActiveRunRollbackRestore => ApplicationOwner::ActiveRunRecovery,
            Self::ActiveRunContinue => ApplicationOwner::ActiveRunRecovery,
            Self::MergeEncounterCompendiumProfile | Self::MergePlayerProfile => {
                ApplicationOwner::PlayerProfile
            }
            Self::ResetPlayerProfile => ApplicationOwner::PlayerProfile,
            #[cfg(test)]
            Self::SetEnkephalinFixture => ApplicationOwner::TestFixture,
        }
    }
}

pub(super) const fn behavior_owner(action: ActionKind) -> ApplicationOwner {
    match action {
        ActionKind::StartNewGame
        | ActionKind::ContinueRun
        | ActionKind::ResetActiveRun
        | ActionKind::RetryPlayerProfile
        | ActionKind::ResetPlayerProfile
        | ActionKind::SelectStarterEmployees
        | ActionKind::RequestMapData
        | ActionKind::SelectMapNode
        | ActionKind::ConvertMapNodeWithMonolith
        | ActionKind::ConfirmEnterNode
        | ActionKind::CancelSelectedNode
        | ActionKind::SweepMasteredElite
        | ActionKind::CompleteMasteredEliteSweep
        | ActionKind::CompleteNode
        | ActionKind::LoadRunCheckpoint
        | ActionKind::ConfirmRunFailure
        | ActionKind::AbandonRun => ApplicationOwner::MapRun,
        ActionKind::RecruitEmployee
        | ActionKind::RequestEmergencySupplies
        | ActionKind::OpenHeadquartersShop => ApplicationOwner::Headquarters,
        ActionKind::EquipItem
        | ActionKind::UnEquipItem
        | ActionKind::UseConsumableItem
        | ActionKind::EquipSkillFragment
        | ActionKind::UnequipSkillFragment
        | ActionKind::SelectSkillFragmentForm
        | ActionKind::TransferSkillFragment
        | ActionKind::AwakenSkillFragment
        | ActionKind::DismantleSkillFragment
        | ActionKind::DismantleEquipment
        | ActionKind::EnhanceEquipment
        | ActionKind::EquipRelic
        | ActionKind::UnequipRelic
        | ActionKind::RequestRelicState
        | ActionKind::RequestRelicNumericPlan
        | ActionKind::PreviewRelicCorruption
        | ActionKind::CorruptRelic
        | ActionKind::PreviewRelicDismantle
        | ActionKind::DismantleRelic => ApplicationOwner::Maintenance,
        ActionKind::PurchaseItem
        | ActionKind::SellItem
        | ActionKind::RefreshShop
        | ActionKind::ExitShop => ApplicationOwner::Shop,
        ActionKind::SelectReward | ActionKind::ClaimReward | ActionKind::ExitReward => {
            ApplicationOwner::Reward
        }
        ActionKind::AdvanceEventScene
        | ActionKind::ResolveEventChoiceCheck
        | ActionKind::ChooseRelicCorruption => ApplicationOwner::Event,
        ActionKind::AdvanceVisualNovel | ActionKind::SelectVisualNovelChoice => {
            ApplicationOwner::VisualNovel
        }
        ActionKind::CompleteCombatResult
        | ActionKind::RequestBattleState
        | ActionKind::RecoverBattleSetupLoss
        | ActionKind::RequestDeploymentRangePreview
        | ActionKind::DeployUnit
        | ActionKind::WithdrawUnit
        | ActionKind::SetEngagementStance
        | ActionKind::TacticalRelocateUnit
        | ActionKind::TacticalSwapUnits
        | ActionKind::FocusBattleUnit
        | ActionKind::ClearBattleFocus
        | ActionKind::BeginDeploymentSelection
        | ActionKind::CancelDeploymentSelection
        | ActionKind::OpenBattleSettings
        | ActionKind::CloseBattleSettings
        | ActionKind::BeginSkillTargeting
        | ActionKind::CancelSkillTargeting
        | ActionKind::PreviewSkillTarget
        | ActionKind::PreviewTacticalCommand
        | ActionKind::ActivateSkill
        | ActionKind::RetreatBattle
        | ActionKind::PauseBattle
        | ActionKind::ResumeBattle
        | ActionKind::SetBattleSpeed => ApplicationOwner::Battle,
    }
}

pub(crate) struct PreparedApplicationCommit<T> {
    draft: GameCore,
    output: T,
    route: ApplicationRoute,
    publication_profile: Option<crate::game::encounter_compendium::PlayerProfileState>,
}

impl<T> PreparedApplicationCommit<T> {
    pub(crate) fn draft(&self) -> &GameCore {
        &self.draft
    }

    pub(crate) fn output(&self) -> &T {
        &self.output
    }

    pub(crate) fn take_publication_profile(
        &mut self,
    ) -> Option<crate::game::encounter_compendium::PlayerProfileState> {
        self.publication_profile.take()
    }

    pub(crate) fn map_output<U>(self, map: impl FnOnce(T) -> U) -> PreparedApplicationCommit<U> {
        PreparedApplicationCommit {
            draft: self.draft,
            output: map(self.output),
            route: self.route,
            publication_profile: self.publication_profile,
        }
    }

    pub(crate) fn with_output<U>(self, output: U) -> PreparedApplicationCommit<U> {
        PreparedApplicationCommit {
            draft: self.draft,
            output,
            route: self.route,
            publication_profile: self.publication_profile,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PublicationDisposition {
    Accepted,
    // Reserved by the P03 outcome contract for a delivery/diagnostic fault
    // observed after the infallible authority commit. Current Core prepares all
    // player-facing projection before commit, so production does not construct
    // this disposition yet.
    #[allow(dead_code)]
    AcceptedRecoveryPending,
}

struct CommittedApplication<T> {
    output: T,
    disposition: PublicationDisposition,
}

impl<T> CommittedApplication<T> {
    /// Publishing an already committed result is deliberately infallible. A
    /// diagnostic/projection delivery problem can only change the accepted
    /// disposition; it can never turn the authoritative commit into `Err`.
    fn publish(self) -> T {
        match self.disposition {
            PublicationDisposition::Accepted | PublicationDisposition::AcceptedRecoveryPending => {
                self.output
            }
        }
    }

    #[cfg(test)]
    fn with_recovery_pending(mut self) -> Self {
        self.disposition = PublicationDisposition::AcceptedRecoveryPending;
        self
    }
}

impl GameCore {
    #[cfg(feature = "runtime-observation")]
    pub(crate) fn observation_identity(&self) -> crate::runtime_observation::MeasurementIdentity {
        let settings = self.runtime_safety();
        crate::runtime_observation::MeasurementIdentity {
            content_sha256: crate::runtime_observation::digest_bytes(
                &self.content_identity().semantic_sha256,
            ),
            raw_profile_sha256: crate::runtime_observation::digest_bytes(settings.raw_sha256()),
            effective_profile_sha256: crate::runtime_observation::digest_bytes(
                settings.effective_sha256(),
            ),
            run_seed: self.run_seed,
            profile_revision: settings
                .development_measurement()
                .map(|profile| profile.revision()),
            profile_warning_bytes: settings.nominal_native_runtime().max_retained_bytes as u64,
            selected_profile_hard_bytes: settings.native_runtime().max_retained_bytes as u64,
        }
    }

    #[cfg(feature = "runtime-observation")]
    pub(crate) fn observation_snapshot(
        &self,
        battle_extent: Option<usize>,
        pending_extent: Option<usize>,
        relic_extent: Option<usize>,
    ) -> crate::runtime_observation::CommittedSnapshot {
        let (battle, usage) = self.state.active_battle.as_ref().map_or_else(
            || (None, crate::runtime_observation::Usage::default()),
            |active| {
                let mut usage = active.battle.observation_usage();
                usage.0[8] = active.execution.last_event_time_ms();
                usage.0[crate::runtime_observation::EXTENT] = battle_extent.unwrap_or(0) as u64;
                (Some(active.battle_uuid), usage)
            },
        );
        crate::runtime_observation::CommittedSnapshot {
            battle,
            usage,
            owner_extent: crate::runtime_observation::NativeOwnerExtent::from_scans(
                battle_extent,
                pending_extent,
            )
            .with_relic_owner(relic_extent),
            identity: Some(self.observation_identity()),
        }
    }

    pub(super) fn execute_application<T>(
        &mut self,
        route: ApplicationRoute,
        operation: impl FnOnce(&mut Self) -> Result<T, GameError>,
    ) -> Result<T, GameError> {
        let prepared = self.prepare_application(route, operation)?;
        Ok(self.commit_prepared_application(prepared))
    }

    pub(super) fn prepare_application<T>(
        &mut self,
        route: ApplicationRoute,
        operation: impl FnOnce(&mut Self) -> Result<T, GameError>,
    ) -> Result<PreparedApplicationCommit<T>, GameError> {
        self.prepare_application_internal(route, false, operation)
    }

    pub(super) fn prepare_application_for_publication<T>(
        &mut self,
        route: ApplicationRoute,
        operation: impl FnOnce(&mut Self) -> Result<T, GameError>,
    ) -> Result<PreparedApplicationCommit<T>, GameError> {
        self.prepare_application_internal(route, true, operation)
    }

    fn prepare_application_internal<T>(
        &mut self,
        route: ApplicationRoute,
        publication: bool,
        operation: impl FnOnce(&mut Self) -> Result<T, GameError>,
    ) -> Result<PreparedApplicationCommit<T>, GameError> {
        if !crate::game::battle::runtime_control::has_operation_scope() {
            let limits = self
                .state
                .active_battle
                .as_ref()
                .map(|active| active.battle.runtime_fact_limits())
                .or_else(|| self.pending_resource_limits());
            return crate::game::battle::runtime_control::run_deferred(|| match limits {
                Some(limits) => {
                    if self
                        .game_data
                        .runtime_safety()
                        .development_measurement()
                        .is_none()
                    {
                        #[cfg(feature = "runtime-observation")]
                        let warning = self
                            .game_data
                            .runtime_safety()
                            .nominal_native_runtime()
                            .max_retained_bytes;
                        crate::game::battle::runtime_control::run(limits, || {
                            #[cfg(feature = "runtime-observation")]
                            crate::game::battle::runtime_control::observe_profile_warning(warning);
                            self.prepare_application_scoped(route, publication, operation)
                        })
                    } else {
                        let settings = std::sync::Arc::clone(self.game_data.runtime_safety());
                        crate::game::battle::runtime_control::run_with_profile(
                            limits,
                            &settings,
                            || self.prepare_application_scoped(route, publication, operation),
                        )
                    }
                }
                None => self.prepare_application_scoped(route, publication, operation),
            });
        }
        self.prepare_application_scoped(route, publication, operation)
    }

    // Keep scope selection separate from the candidate body: a deferred entry
    // must not reserve the large candidate frame twice through self-recursion.
    fn prepare_application_scoped<T>(
        &mut self,
        route: ApplicationRoute,
        publication: bool,
        operation: impl FnOnce(&mut Self) -> Result<T, GameError>,
    ) -> Result<PreparedApplicationCommit<T>, GameError> {
        if self
            .state
            .active_battle
            .as_ref()
            .is_some_and(|active| active.battle.forced_pause_remaining_ms().is_some())
        {
            let allowed = match route {
                ApplicationRoute::BattleServerTick | ApplicationRoute::BattleDisconnectPause => {
                    true
                }
                ApplicationRoute::Behavior(
                    ActionKind::RequestBattleState
                    | ActionKind::PreviewSkillTarget
                    | ActionKind::PreviewTacticalCommand
                    | ActionKind::RequestDeploymentRangePreview,
                ) => true,
                ApplicationRoute::Behavior(_)
                | ApplicationRoute::BattleElapsedAdvance
                | ApplicationRoute::BattleEventBucket
                | ApplicationRoute::AdminCommand
                | ApplicationRoute::ActiveRunContinue
                | ApplicationRoute::MergeEncounterCompendiumProfile
                | ApplicationRoute::MergePlayerProfile
                | ApplicationRoute::ResetPlayerProfile
                | ApplicationRoute::ActiveRunRollbackRestore => false,
                #[cfg(test)]
                ApplicationRoute::SetEnkephalinFixture => false,
            };
            if !allowed {
                return Err(GameError::ActionUnavailable {
                    action: match route {
                        ApplicationRoute::Behavior(action) => action,
                        _ => ActionKind::ActivateSkill,
                    },
                    reason: crate::game::behavior::ActionDisabledReasonCode::CorrosionForcedPause,
                });
            }
        }
        if self
            .state
            .active_battle
            .as_ref()
            .is_some_and(|active| active.settings_open())
        {
            if let ApplicationRoute::Behavior(action) = route {
                if !super::battle_interaction::settings_allows_action(action) {
                    return Err(GameError::ActionUnavailable {
                        action,
                        reason: crate::game::behavior::ActionDisabledReasonCode::SettingsOpen,
                    });
                }
            }
        }
        // Resolve the exhaustive owner before staging. There is no unregistered
        // catch-all route and no owner-specific live write capability.
        let _owner = route.owner();
        crate::authoring::control::charge(1, "application_entry")?;
        self.state.validate_application_context()?;
        self.validate_run_resources()?;

        if self.state.has_relic_owner() && crate::game::battle::runtime_control::is_active() {
            let base = if let Some(active) = &self.state.active_battle {
                active.battle.retained_runtime_fact_extent()?
            } else {
                self.pending_resource_extent()?.unwrap_or(0)
            };
            let total = base
                .checked_add(self.state.relic_owned_extent()?)
                .ok_or_else(|| {
                    GameError::InvalidStaticData("relic operation seed overflow".into())
                })?;
            crate::game::battle::runtime_control::seed_retained_extent(total)?;
        }
        if let Some(active) = &self.state.active_battle {
            active.battle.preflight_runtime_fact_clone()?;
        } else if let Some(extent) = self.pending_resource_extent()? {
            crate::game::battle::runtime_control::seed_retained_extent(extent)?;
        }
        self.state.preflight_resource_clone(true)?;
        let mut draft = crate::authoring::clone_shape::controlled_clone(self)?;
        if let Some(active) = draft.state.active_battle.as_mut() {
            active.battle.event_log.prepare_authoring_write()?;
        }
        // Existing World capture above stays outside the newly opt-in relic
        // native scope. First-empty grants activate lazily in their typed domain.
        if draft.state.has_relic_owner() {
            draft.ensure_relic_operation()?;
        }
        draft.prepare_continue_before_operation(route)?;
        let output = operation(&mut draft)?;
        draft.reconcile_employee_breakdown();
        draft.reconcile_relic_loadouts()?;
        if matches!(
            route,
            ApplicationRoute::ActiveRunContinue
                | ApplicationRoute::ActiveRunRollbackRestore
                | ApplicationRoute::Behavior(
                    ActionKind::EquipRelic
                        | ActionKind::UnequipRelic
                        | ActionKind::CorruptRelic
                        | ActionKind::ChooseRelicCorruption
                        | ActionKind::EquipItem
                        | ActionKind::UnEquipItem
                        | ActionKind::EquipSkillFragment
                        | ActionKind::UnequipSkillFragment
                        | ActionKind::SelectSkillFragmentForm
                        | ActionKind::TransferSkillFragment
                        | ActionKind::AwakenSkillFragment
                        | ActionKind::DismantleSkillFragment
                        | ActionKind::EnhanceEquipment
                        | ActionKind::DismantleEquipment
                        | ActionKind::DismantleRelic
                        | ActionKind::SellItem
                        | ActionKind::ContinueRun
                )
        ) {
            draft.validate_structural_loadouts()?;
        }
        draft.reconcile_run_resources()?;
        draft.reconcile_authoring_lifecycle()?;
        draft.refresh_continue_after_operation(route)?;
        #[cfg(feature = "runtime-observation")]
        let mut battle_extent = None;
        if let Some(active) = &draft.state.active_battle {
            let _extent = active.battle.retained_runtime_fact_extent()?;
            #[cfg(feature = "runtime-observation")]
            {
                battle_extent = Some(_extent);
            }
        }
        let _pending_extent = draft.pending_resource_extent()?;
        // The new owner scan is a gameplay validation boundary in both feature
        // configurations. Observation only consumes its already computed value.
        let _relic_extent = if draft.state.has_relic_owner() {
            Some(draft.state.relic_owned_extent()?)
        } else {
            None
        };
        crate::authoring::control::poll("application_operation_complete")?;
        draft.state.validate_application_context()?;
        let publication_profile = publication
            .then(|| draft.capture_player_profile())
            .transpose()?;
        crate::authoring::control::poll("application_commit")?;

        #[cfg(feature = "runtime-observation")]
        crate::runtime_observation::staged(draft.observation_snapshot(
            battle_extent,
            _pending_extent,
            _relic_extent,
        ));
        Ok(PreparedApplicationCommit {
            draft,
            output,
            route,
            publication_profile,
        })
    }

    pub(crate) fn commit_prepared_application<T>(
        &mut self,
        prepared: PreparedApplicationCommit<T>,
    ) -> T {
        self.commit_application(prepared).publish()
    }

    /// The sole live-root replacement port for application operations.
    fn commit_application<T>(
        &mut self,
        prepared: PreparedApplicationCommit<T>,
    ) -> CommittedApplication<T> {
        let PreparedApplicationCommit {
            draft,
            output,
            route,
            publication_profile: _,
        } = prepared;
        let _committed_route = route;
        *self = draft;
        #[cfg(feature = "runtime-observation")]
        crate::runtime_observation::committed(std::ptr::from_ref(self) as usize);
        CommittedApplication {
            output,
            disposition: PublicationDisposition::Accepted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::behavior::PlayerBehavior;
    use crate::game::data::GameDataBuilder;
    use crate::game::map::GameMode;
    use uuid::Uuid;

    #[test]
    fn every_behavior_kind_has_exactly_one_application_owner() {
        assert_eq!(ActionKind::ALL.len(), 75);
        for action in ActionKind::ALL {
            let route = ApplicationRoute::Behavior(*action);
            assert_eq!(route.owner(), behavior_owner(*action));
        }
    }

    #[test]
    fn committed_publication_fault_is_typed_as_accepted_recovery_pending() {
        let committed = CommittedApplication {
            output: 7_u8,
            disposition: PublicationDisposition::Accepted,
        }
        .with_recovery_pending();
        assert_eq!(
            committed.disposition,
            PublicationDisposition::AcceptedRecoveryPending
        );
        assert_eq!(committed.publish(), 7);
    }

    #[test]
    fn incoherent_parallel_context_is_rejected_before_application_staging() {
        let mut core = GameCore::new(GameDataBuilder::empty().build_arc(), 0xCA03_0002);
        core.state.pending_game_mode = Some(GameMode::Standard);
        let before_phase = core.state.lifecycle.snapshot();
        let before_pending = core.state.pending_game_mode;

        let error = core
            .execute(
                Uuid::from_u128(1),
                PlayerBehavior::StartNewGame {
                    game_mode: GameMode::Standard,
                    replace_active_run: false,
                },
            )
            .expect_err("incoherent NotStarted context must not be published or staged");

        assert!(matches!(error, GameError::BattleRuntimeFault(fault)
            if fault.code == crate::game::battle::runtime_control::RuntimeFaultCode::Invariant
                && fault.site == "application_context"));
        assert_eq!(core.state.lifecycle.snapshot(), before_phase);
        assert_eq!(core.state.pending_game_mode, before_pending);
        assert!(core.state.player.is_none());
    }
}
