use std::collections::{BTreeMap, BTreeSet};

use crate::game::{
    battle::ids::UnitInstanceId, behavior::GameError, enums::Side, resources::Position,
};

use super::BattleCore;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpawnRequestId(pub String);

impl From<&str> for SpawnRequestId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpawnTriggerId(pub String);

impl From<&str> for SpawnTriggerId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct RemovalEventId(pub String);

impl From<&str> for RemovalEventId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// Provenance is descriptive only. The kernel applies the same transaction and
/// exact-once rules to wave and summon requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnRequestSource {
    Wave {
        wave_id: String,
    },
    Summon {
        summoner: UnitInstanceId,
        definition_id: String,
        count: u32,
    },
    Independent(std::sync::Arc<IndependentSpawnIdentity>),
}

/// Stable caller input. Current occupancy is deliberately absent: replaying a
/// settled request must still work after its objects occupy the original cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndependentSpawnIdentity {
    pub creator: UnitInstanceId,
    pub definition_id: String,
    pub positions: Vec<Position>,
    pub allow_same_source_evolution: bool,
    pub wall_cost: Option<crate::game::ability::objects::ObjectWallCostDef>,
    pub source: crate::game::battle::runtime_facts::RuntimeFactSource,
    pub creation_action: Option<crate::game::battle::runtime_facts::ActionOrdinal>,
    pub time_ms: u64,
}
crate::authoring::clone_shape::struct_shape!(IndependentSpawnIdentity {
    creator,
    definition_id,
    positions,
    allow_same_source_evolution,
    wall_cost,
    source,
    creation_action,
    time_ms
});

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum SpawnOccupancyClass {
    Summoner,
    SameFaction,
    OpposingCombatant,
    ProtectedObjective,
    StaticDefenseObject,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SpawnOccupantKind {
    Summoner,
    SameFaction,
    OpposingCombatant,
    ProtectedObjective,
    StaticDefenseObject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnOccupancyPolicy {
    RequireUnoccupied,
    AllowOccupied(Vec<SpawnOccupancyClass>),
}

impl SpawnOccupancyPolicy {
    pub fn validate(&self) -> Result<(), SpawnPolicyError> {
        let Self::AllowOccupied(classes) = self else {
            return Ok(());
        };

        if classes.is_empty() {
            return Err(SpawnPolicyError::EmptyAllowOccupied);
        }

        let mut unique = BTreeSet::new();
        for class in classes {
            if !unique.insert(*class) {
                return Err(SpawnPolicyError::DuplicateOccupancyClass(*class));
            }
        }

        if unique.contains(&SpawnOccupancyClass::Any) && unique.len() != 1 {
            return Err(SpawnPolicyError::AnyMustBeExclusive);
        }
        if unique.contains(&SpawnOccupancyClass::Summoner)
            && unique.contains(&SpawnOccupancyClass::SameFaction)
        {
            return Err(SpawnPolicyError::SummonerOverlapsSameFaction);
        }

        Ok(())
    }

    pub(crate) fn allows(&self, occupants: &[SpawnOccupantKind]) -> bool {
        match self {
            Self::RequireUnoccupied => occupants.is_empty(),
            Self::AllowOccupied(classes) => {
                classes.contains(&SpawnOccupancyClass::Any)
                    || occupants.iter().all(|occupant| match occupant {
                        SpawnOccupantKind::Summoner => {
                            classes.contains(&SpawnOccupancyClass::Summoner)
                                || classes.contains(&SpawnOccupancyClass::SameFaction)
                        }
                        SpawnOccupantKind::SameFaction => {
                            classes.contains(&SpawnOccupancyClass::SameFaction)
                        }
                        SpawnOccupantKind::OpposingCombatant => {
                            classes.contains(&SpawnOccupancyClass::OpposingCombatant)
                        }
                        SpawnOccupantKind::ProtectedObjective => {
                            classes.contains(&SpawnOccupancyClass::ProtectedObjective)
                        }
                        SpawnOccupantKind::StaticDefenseObject => {
                            classes.contains(&SpawnOccupancyClass::StaticDefenseObject)
                        }
                    })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpawnPolicyError {
    EmptyAllowOccupied,
    DuplicateOccupancyClass(SpawnOccupancyClass),
    AnyMustBeExclusive,
    SummonerOverlapsSameFaction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnPlacement {
    pub order: u32,
    pub position: Position,
    /// `false` represents invalid terrain or an intrinsically blocked cell.
    /// Occupancy is represented separately so an allowed static defense object
    /// does not accidentally become a terrain failure.
    pub terrain_walkable: bool,
    pub occupants: Vec<SpawnOccupantKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnRequest {
    pub id: SpawnRequestId,
    pub source: SpawnRequestSource,
    pub owner: Side,
    pub occupancy: SpawnOccupancyPolicy,
    pub placements: Vec<SpawnPlacement>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpawnRejection {
    InvalidRequestId,
    InvalidPolicy(SpawnPolicyError),
    EmptyPlacementSet,
    DuplicatePlacementOrder(u32),
    DuplicatePosition(Position),
    UnwalkablePosition(Position),
    OccupancyRejected(Position),
    DuplicateSpawnedUnitId(UnitInstanceId),
    MutationFailed { game_error_code: String },
    LimitReached { max_alive: u32, max_total: u32 },
    InsufficientSpace { required: u32, available: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpawnCancellationReason {
    SummonerRemoved { removal_event_id: RemovalEventId },
    OwnerStateExited { state_activation_id: u64 },
    WaveDependencyUnavailable { dependency_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpawnRequestSettlement {
    Spawned { unit_ids: Vec<UnitInstanceId> },
    Evolved { unit_ids: Vec<UnitInstanceId> },
    Rejected { reason: SpawnRejection },
    Canceled { reason: SpawnCancellationReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnKernelError {
    /// The opt-in attempt owns the exact latched cancellation/limit diagnostic.
    /// Never a spawn rejection, cached settlement, or ordinary gameplay result.
    ExecutionInterrupted,
    RequestIdentityConflict {
        request_id: SpawnRequestId,
    },
    TriggerIdentityConflict {
        trigger_id: SpawnTriggerId,
    },
    TriggerNotRegistered {
        trigger_id: SpawnTriggerId,
    },
    TriggerAlreadyBound {
        trigger_id: SpawnTriggerId,
    },
    TriggerNotBound {
        trigger_id: SpawnTriggerId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SpawnRequestStatus {
    Pending,
    Settled(SpawnRequestSettlement),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpawnRequestRecord {
    request: SpawnRequest,
    status: SpawnRequestStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TriggerStatus {
    Registered,
    Bound(SpawnRequestId),
    Settled(SpawnRequestId),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SpawnKernelState {
    requests: BTreeMap<SpawnRequestId, SpawnRequestRecord>,
    triggers: BTreeMap<SpawnTriggerId, TriggerStatus>,
    compendium_candidates: BTreeMap<u64, CompendiumCandidate>,
    compendium_candidate_ids: BTreeSet<String>,
}

crate::authoring::clone_shape::newtype_shape!(SpawnRequestId, SpawnTriggerId, RemovalEventId);
crate::authoring::clone_shape::copy_shape!(SpawnOccupancyClass, SpawnOccupantKind);
crate::authoring::clone_shape::struct_shape!(SpawnKernelState {
    requests,
    triggers,
    compendium_candidates,
    compendium_candidate_ids,
});
crate::authoring::clone_shape::struct_shape!(SpawnRequestRecord { request, status });
crate::authoring::clone_shape::struct_shape!(SpawnRequest {
    id,
    source,
    owner,
    occupancy,
    placements
});
crate::authoring::clone_shape::struct_shape!(SpawnPlacement {
    order,
    position,
    terrain_walkable,
    occupants
});
crate::authoring::clone_shape::struct_shape!(CompendiumCandidate {
    id,
    kind,
    base_uuid
});
crate::authoring::clone_shape::enum_shape!(SpawnRequestSource {
    units: [],
    tuples: [Independent(identity)],
    records: [
        Wave { wave_id },
        Summon {
            summoner,
            definition_id,
            count
        }
    ]
});
crate::authoring::clone_shape::enum_shape!(SpawnOccupancyPolicy {
    units: [RequireUnoccupied],
    tuples: [AllowOccupied(classes)],
    records: []
});
crate::authoring::clone_shape::enum_shape!(SpawnRequestStatus {
    units: [Pending],
    tuples: [Settled(settlement)],
    records: []
});
crate::authoring::clone_shape::enum_shape!(TriggerStatus {
    units: [Registered],
    tuples: [Bound(id), Settled(id)],
    records: []
});
crate::authoring::clone_shape::enum_shape!(SpawnRequestSettlement {
    units: [],
    tuples: [],
    records: [
        Spawned { unit_ids },
        Evolved { unit_ids },
        Rejected { reason },
        Canceled { reason }
    ]
});
crate::authoring::clone_shape::enum_shape!(SpawnCancellationReason {
    units: [],
    tuples: [],
    records: [
        SummonerRemoved { removal_event_id },
        OwnerStateExited {
            state_activation_id
        },
        WaveDependencyUnavailable { dependency_id }
    ]
});
crate::authoring::clone_shape::enum_shape!(SpawnRejection {
    units: [InvalidRequestId, EmptyPlacementSet],
    tuples: [
        InvalidPolicy(reason),
        DuplicatePlacementOrder(order),
        DuplicatePosition(position),
        UnwalkablePosition(position),
        OccupancyRejected(position),
        DuplicateSpawnedUnitId(id)
    ],
    records: [
        MutationFailed { game_error_code },
        LimitReached {
            max_alive,
            max_total
        },
        InsufficientSpace {
            required,
            available
        }
    ]
});
crate::authoring::clone_shape::enum_shape!(SpawnPolicyError {
    units: [
        EmptyAllowOccupied,
        AnyMustBeExclusive,
        SummonerOverlapsSameFaction
    ],
    tuples: [DuplicateOccupancyClass(class)],
    records: []
});
crate::authoring::clone_shape::enum_shape!(CompendiumCandidateKind {
    units: [Abnormality, CorrodedEmployee],
    tuples: [],
    records: []
});

fn clone_for_spawn<T: Clone + crate::authoring::clone_shape::CloneShape>(
    value: &T,
) -> Result<T, SpawnKernelError> {
    crate::authoring::clone_shape::controlled_clone(value)
        .map_err(|_| SpawnKernelError::ExecutionInterrupted)
}

impl SpawnKernelState {
    pub(super) fn independent_request_snapshots(
        &self,
    ) -> Result<Vec<super::object_query::ObjectCreationReceipt>, GameError> {
        use super::object_query::{ObjectCreationReceipt, ObjectSourceView};
        crate::authoring::control::charge(self.requests.len() as u64, "object_request_query")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.independent_retained_extent().and_then(|n| {
                n.checked_add(
                    self.requests
                        .len()
                        .checked_mul(std::mem::size_of::<ObjectCreationReceipt>())?,
                )
            }),
            "object_request_query_storage",
        )?;
        let mut result = Vec::with_capacity(self.requests.len());
        for (id, record) in &self.requests {
            let SpawnRequestSource::Independent(input) = &record.request.source else {
                continue;
            };
            result.push(ObjectCreationReceipt {
                request_id: crate::authoring::clone_shape::controlled_clone(&id.0)?,
                definition_id: crate::authoring::clone_shape::controlled_clone(
                    &input.definition_id,
                )?,
                side: record.request.owner,
                time_ms: input.time_ms,
                positions: crate::authoring::clone_shape::controlled_clone(&input.positions)?,
                source: ObjectSourceView::from_runtime(
                    input.creator,
                    input.creation_action,
                    &input.source,
                )?,
                settlement: match &record.status {
                    SpawnRequestStatus::Pending => None,
                    SpawnRequestStatus::Settled(value) => {
                        Some(crate::authoring::clone_shape::controlled_clone(value)?)
                    }
                },
            });
        }
        Ok(result)
    }

    pub(super) fn independent_entity_count(&self) -> Result<usize, GameError> {
        crate::authoring::control::charge(
            self.requests.len() as u64,
            "independent_request_entities",
        )?;
        let mut count = 0usize;
        for record in self.requests.values() {
            if let SpawnRequestSource::Independent(input) = &record.request.source {
                count = count
                    .saturating_add(1)
                    .saturating_add(input.positions.len())
                    .saturating_add(record.request.placements.len());
                for placement in &record.request.placements {
                    count = count.saturating_add(placement.occupants.len());
                }
                if let SpawnRequestStatus::Settled(
                    SpawnRequestSettlement::Spawned { unit_ids }
                    | SpawnRequestSettlement::Evolved { unit_ids },
                ) = &record.status
                {
                    count = count.saturating_add(unit_ids.len());
                }
            }
        }
        Ok(count)
    }

    pub(super) fn independent_retained_extent(&self) -> Option<usize> {
        let mut bytes = 0usize;
        for (id, record) in &self.requests {
            let SpawnRequestSource::Independent(input) = &record.request.source else {
                continue;
            };
            if let Some(cost) = &input.wall_cost {
                bytes = bytes.checked_add(cost.pool_id.capacity())?;
            }

            let node = std::mem::size_of::<(SpawnRequestId, SpawnRequestRecord)>()
                .checked_add(16)?
                .checked_mul(16)?;
            bytes = bytes
                .checked_add(node)?
                .checked_add(id.0.capacity())?
                .checked_add(record.request.id.0.capacity())?
                .checked_add(std::mem::size_of::<IndependentSpawnIdentity>() + 64)?
                .checked_add(input.definition_id.capacity())?
                .checked_add(
                    input
                        .positions
                        .capacity()
                        .checked_mul(std::mem::size_of::<Position>())?,
                )?
                .checked_add(input.source.retained_extent()?)?
                .checked_add(
                    record
                        .request
                        .placements
                        .capacity()
                        .checked_mul(std::mem::size_of::<SpawnPlacement>())?,
                )?;
            for placement in &record.request.placements {
                bytes = bytes.checked_add(
                    placement
                        .occupants
                        .capacity()
                        .checked_mul(std::mem::size_of::<SpawnOccupantKind>())?,
                )?;
            }
            if let SpawnRequestStatus::Settled(
                SpawnRequestSettlement::Spawned { unit_ids }
                | SpawnRequestSettlement::Evolved { unit_ids },
            ) = &record.status
            {
                bytes = bytes.checked_add(
                    unit_ids
                        .capacity()
                        .checked_mul(std::mem::size_of::<UnitInstanceId>())?,
                )?;
            }
            if let SpawnRequestStatus::Settled(SpawnRequestSettlement::Rejected {
                reason: SpawnRejection::MutationFailed { game_error_code },
            }) = &record.status
            {
                bytes = bytes.checked_add(game_error_code.capacity())?;
            }
        }
        Some(bytes)
    }

    pub(super) fn request_count(&self) -> usize {
        self.requests.len()
    }

    pub(super) fn reset_independent_requests(&mut self) {
        self.requests.retain(|_, record| {
            !matches!(record.request.source, SpawnRequestSource::Independent(_))
        });
    }

    fn identity_request(request: SpawnRequest) -> Result<SpawnRequest, SpawnKernelError> {
        // Only a domain rejection preserves the original (unnormalized) identity.
        // The outer error must never fall back to a successful request.
        // Already ordered placements have the same identity on BOTH paths:
        // successful normalization and every domain rejection return this exact
        // value. Move that owner instead of cloning/sorting/validating a value
        // whose validation result this identity-only operation discards.
        crate::authoring::control::charge(
            request.placements.len() as u64,
            "spawn_identity_order_scan",
        ).map_err(|_| SpawnKernelError::ExecutionInterrupted)?;
        if request.placements.is_sorted_by_key(|placement| placement.order) {
            return Ok(request);
        }
        Ok(Self::normalize_request(&request)?.unwrap_or(request))
    }

    fn normalize_request(
        request: &SpawnRequest,
    ) -> Result<Result<SpawnRequest, SpawnRequestSettlement>, SpawnKernelError> {
        let reject = |reason| SpawnRequestSettlement::Rejected { reason };

        // Charge before the first byte scan, including invalid identifiers.
        crate::authoring::control::charge(
            u64::try_from(request.id.0.len()).unwrap_or(u64::MAX),
            "spawn_request_id_scan",
        )
        .map_err(|_| SpawnKernelError::ExecutionInterrupted)?;
        if request.id.0.trim().is_empty() {
            return Ok(Err(reject(SpawnRejection::InvalidRequestId)));
        }
        if let Err(error) = request.occupancy.validate() {
            return Ok(Err(reject(SpawnRejection::InvalidPolicy(error))));
        }
        if request.placements.is_empty() {
            return Ok(Err(reject(SpawnRejection::EmptyPlacementSet)));
        }

        let mut normalized = clone_for_spawn(request)?;
        normalized
            .placements
            .sort_by_key(|placement| placement.order);
        let mut orders = BTreeSet::new();
        let mut positions = BTreeSet::new();
        for placement in &normalized.placements {
            if !orders.insert(placement.order) {
                return Ok(Err(reject(SpawnRejection::DuplicatePlacementOrder(
                    placement.order,
                ))));
            }
            if !positions.insert((placement.position.x, placement.position.y)) {
                return Ok(Err(reject(SpawnRejection::DuplicatePosition(
                    placement.position,
                ))));
            }
            if !placement.terrain_walkable {
                return Ok(Err(reject(SpawnRejection::UnwalkablePosition(
                    placement.position,
                ))));
            }
            if !request.occupancy.allows(&placement.occupants) {
                return Ok(Err(reject(SpawnRejection::OccupancyRejected(
                    placement.position,
                ))));
            }
        }
        Ok(Ok(normalized))
    }

    fn reserve_request(&mut self, request: SpawnRequest) -> Result<(), SpawnKernelError> {
        if let Some(existing) = self.requests.get(&request.id) {
            if existing.request == request {
                return Ok(());
            }
            return Err(SpawnKernelError::RequestIdentityConflict {
                request_id: request.id,
            });
        }
        if let SpawnRequestSource::Independent(input) = &request.source {
            let bytes = std::mem::size_of::<(SpawnRequestId, SpawnRequestRecord)>()
                .checked_add(16)
                .and_then(|n| n.checked_mul(16))
                .and_then(|n| n.checked_add(request.id.0.len().checked_mul(2)?))
                .and_then(|n| n.checked_add(std::mem::size_of::<IndependentSpawnIdentity>() + 64))
                .and_then(|n| n.checked_add(input.definition_id.capacity()))
                .and_then(|n| {
                    n.checked_add(
                        input
                            .positions
                            .capacity()
                            .checked_mul(std::mem::size_of::<Position>())?,
                    )
                })
                .and_then(|n| n.checked_add(input.source.retained_extent()?))
                .and_then(|n| {
                    n.checked_add(request.placements.capacity().checked_mul(
                        std::mem::size_of::<SpawnPlacement>()
                            + 2 * std::mem::size_of::<UnitInstanceId>(),
                    )?)
                });
            let bytes = bytes.and_then(|bytes| {
                request.placements.iter().try_fold(bytes, |n, p| {
                    n.checked_add(
                        p.occupants
                            .capacity()
                            .checked_mul(std::mem::size_of::<SpawnOccupantKind>())?,
                    )
                })
            });
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                bytes,
                "independent_request_ledger",
            )
            .map_err(|_| SpawnKernelError::ExecutionInterrupted)?;
        }
        self.requests.insert(
            clone_for_spawn(&request.id)?,
            SpawnRequestRecord {
                request,
                status: SpawnRequestStatus::Pending,
            },
        );
        Ok(())
    }

    fn settlement(&self, request_id: &SpawnRequestId) -> Option<&SpawnRequestSettlement> {
        let record = self.requests.get(request_id)?;
        match &record.status {
            SpawnRequestStatus::Pending => None,
            SpawnRequestStatus::Settled(settlement) => Some(settlement),
        }
    }

    fn settle(&mut self, request_id: &SpawnRequestId, settlement: SpawnRequestSettlement) {
        let record = self
            .requests
            .get_mut(request_id)
            .expect("spawn requests are reserved before settlement");
        record.status = SpawnRequestStatus::Settled(settlement);
    }

    pub(crate) fn register_trigger(
        &mut self,
        trigger_id: SpawnTriggerId,
    ) -> Result<bool, SpawnKernelError> {
        match self.triggers.get(&trigger_id) {
            None => {
                self.triggers.insert(trigger_id, TriggerStatus::Registered);
                Ok(true)
            }
            Some(TriggerStatus::Registered) => Ok(false),
            Some(_) => Err(SpawnKernelError::TriggerIdentityConflict { trigger_id }),
        }
    }

    pub(crate) fn bind_trigger(
        &mut self,
        trigger_id: &SpawnTriggerId,
        request: SpawnRequest,
    ) -> Result<(), SpawnKernelError> {
        let request = Self::identity_request(request)?;
        match self.triggers.get(trigger_id) {
            None => {
                return Err(SpawnKernelError::TriggerNotRegistered {
                    trigger_id: clone_for_spawn(trigger_id)?,
                });
            }
            Some(TriggerStatus::Registered) => {}
            Some(TriggerStatus::Bound(existing)) if existing == &request.id => {
                self.reserve_request(request)?;
                return Ok(());
            }
            Some(_) => {
                return Err(SpawnKernelError::TriggerAlreadyBound {
                    trigger_id: clone_for_spawn(trigger_id)?,
                });
            }
        }

        let request_id = clone_for_spawn(&request.id)?;
        let trigger_id = clone_for_spawn(trigger_id)?;
        self.reserve_request(request)?;
        self.triggers
            .insert(trigger_id, TriggerStatus::Bound(request_id));
        Ok(())
    }

    pub(crate) fn settle_trigger(
        &mut self,
        trigger_id: &SpawnTriggerId,
    ) -> Result<SpawnRequestSettlement, SpawnKernelError> {
        let request_id = match self.triggers.get(trigger_id) {
            None => {
                return Err(SpawnKernelError::TriggerNotRegistered {
                    trigger_id: clone_for_spawn(trigger_id)?,
                });
            }
            Some(TriggerStatus::Registered) => {
                return Err(SpawnKernelError::TriggerNotBound {
                    trigger_id: clone_for_spawn(trigger_id)?,
                });
            }
            Some(TriggerStatus::Bound(request_id) | TriggerStatus::Settled(request_id)) => {
                clone_for_spawn(request_id)?
            }
        };
        let Some(settlement) = self.settlement(&request_id) else {
            return Err(SpawnKernelError::TriggerNotBound {
                trigger_id: clone_for_spawn(trigger_id)?,
            });
        };
        let settlement = clone_for_spawn(settlement)?;
        let trigger_id = clone_for_spawn(trigger_id)?;
        self.triggers
            .insert(trigger_id, TriggerStatus::Settled(request_id));
        Ok(settlement)
    }

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub(crate) fn cancel_for_summoner_removal(
        &mut self,
        request: SpawnRequest,
        removal_event_id: RemovalEventId,
    ) -> Result<SpawnRequestSettlement, SpawnKernelError> {
        let request = Self::identity_request(request)?;
        let request_id = clone_for_spawn(&request.id)?;
        self.reserve_request(request)?;
        if let Some(existing) = self.settlement(&request_id) {
            return clone_for_spawn(existing);
        }
        let settlement = SpawnRequestSettlement::Canceled {
            reason: SpawnCancellationReason::SummonerRemoved { removal_event_id },
        };
        self.settle(&request_id, clone_for_spawn(&settlement)?);
        Ok(settlement)
    }

    pub(crate) fn cancel_request(
        &mut self,
        request: SpawnRequest,
        reason: SpawnCancellationReason,
    ) -> Result<SpawnRequestSettlement, SpawnKernelError> {
        let request = Self::identity_request(request)?;
        let request_id = clone_for_spawn(&request.id)?;
        self.reserve_request(request)?;
        if let Some(existing) = self.settlement(&request_id) {
            return clone_for_spawn(existing);
        }
        let settlement = SpawnRequestSettlement::Canceled { reason };
        self.settle(&request_id, clone_for_spawn(&settlement)?);
        Ok(settlement)
    }

    pub(crate) fn reject_request(
        &mut self,
        request: SpawnRequest,
        reason: SpawnRejection,
    ) -> Result<SpawnRequestSettlement, SpawnKernelError> {
        let request = Self::identity_request(request)?;
        let request_id = clone_for_spawn(&request.id)?;
        self.reserve_request(request)?;
        if let Some(existing) = self.settlement(&request_id) {
            return clone_for_spawn(existing);
        }
        let settlement = SpawnRequestSettlement::Rejected { reason };
        self.settle(&request_id, clone_for_spawn(&settlement)?);
        Ok(settlement)
    }

    pub(crate) fn record_compendium_candidate(
        &mut self,
        order: u64,
        candidate: CompendiumCandidate,
    ) -> Result<bool, CompendiumCandidateError> {
        if self.compendium_candidate_ids.contains(&candidate.id) {
            let existing = self
                .compendium_candidates
                .values()
                .find(|existing| existing.id == candidate.id)
                .expect("candidate ID index and ordered ledger must agree");
            if existing == &candidate {
                return Ok(false);
            }
            return Err(CompendiumCandidateError::IdentityConflict(candidate.id));
        }
        if self.compendium_candidates.contains_key(&order) {
            return Err(CompendiumCandidateError::OrderConflict(order));
        }
        self.compendium_candidate_ids.insert(candidate.id.clone());
        self.compendium_candidates.insert(order, candidate);
        Ok(true)
    }

    pub(crate) fn compendium_candidates(&self) -> Vec<CompendiumCandidate> {
        self.compendium_candidates.values().cloned().collect()
    }
}

impl BattleCore {
    pub(super) fn replay_independent_request(
        &self,
        id: &SpawnRequestId,
        owner: Side,
        input: &IndependentSpawnIdentity,
    ) -> Result<Option<SpawnRequestSettlement>, SpawnKernelError> {
        crate::game::battle::runtime_control::charge(1, "independent_request_replay")
            .map_err(|_| SpawnKernelError::ExecutionInterrupted)?;
        let Some(record) = self.spawn_kernel.requests.get(id) else {
            return Ok(None);
        };
        let matches = record.request.owner == owner
            && matches!(&record.request.source, SpawnRequestSource::Independent(old) if old.as_ref() == input);
        if !matches {
            return Err(SpawnKernelError::RequestIdentityConflict {
                request_id: clone_for_spawn(id)?,
            });
        }
        match &record.status {
            SpawnRequestStatus::Pending => Ok(None),
            SpawnRequestStatus::Settled(settlement) => clone_for_spawn(settlement).map(Some),
        }
    }

    /// Executes a spawn batch as one clone-backed transaction. Preflight checks
    /// every cell before mutation, then the Kernel invokes exactly one commit
    /// operation per placement in canonical order. A mutation error or duplicate
    /// returned identity restores the entire core. Replaying the same request
    /// returns its cached terminal settlement.
    pub fn execute_spawn_request<F>(
        &mut self,
        request: SpawnRequest,
        mutate: F,
    ) -> Result<SpawnRequestSettlement, SpawnKernelError>
    where
        F: FnMut(&mut BattleCore, &SpawnPlacement) -> Result<UnitInstanceId, GameError>,
    {
        self.execute_spawn_request_with_evolution(request, false, mutate)
    }

    pub(super) fn execute_spawn_request_with_evolution<F>(
        &mut self,
        request: SpawnRequest,
        evolved: bool,
        mut mutate: F,
    ) -> Result<SpawnRequestSettlement, SpawnKernelError>
    where
        F: FnMut(&mut BattleCore, &SpawnPlacement) -> Result<UnitInstanceId, GameError>,
    {
        crate::authoring::control::poll("spawn_request_entry")
            .map_err(|_| SpawnKernelError::ExecutionInterrupted)?;
        let normalized_result = SpawnKernelState::normalize_request(&request)?;
        let identity_request = normalized_result.as_ref().unwrap_or(&request);
        if let Some(existing) = self.spawn_kernel.requests.get(&request.id) {
            if &existing.request != identity_request {
                return Err(SpawnKernelError::RequestIdentityConflict {
                    request_id: request.id,
                });
            }
            if let SpawnRequestStatus::Settled(settlement) = &existing.status {
                return clone_for_spawn(settlement);
            }
        }

        let normalized = match normalized_result {
            Ok(normalized) => normalized,
            Err(settlement) => {
                let request_id = clone_for_spawn(&request.id)?;
                let cached = clone_for_spawn(&settlement)?;
                self.spawn_kernel.reserve_request(request)?;
                self.spawn_kernel.settle(&request_id, cached);
                return Ok(settlement);
            }
        };
        self.spawn_kernel
            .reserve_request(clone_for_spawn(&normalized)?)?;

        self.preflight_authoring_clone()
            .map_err(|_| SpawnKernelError::ExecutionInterrupted)?;
        let before = self.clone();
        crate::authoring::control::poll("battle_core_clone_complete")
            .map_err(|_| SpawnKernelError::ExecutionInterrupted)?;
        if self.event_log.prepare_authoring_write().is_err() {
            *self = before;
            return Err(SpawnKernelError::ExecutionInterrupted);
        }
        let mut unit_ids = Vec::with_capacity(normalized.placements.len());
        let mut unique_unit_ids = BTreeSet::new();
        let mut rejected = None;
        for placement in &normalized.placements {
            let outcome = mutate(self, placement);
            if crate::authoring::control::poll("spawn_placement_complete").is_err() {
                *self = before;
                return Err(SpawnKernelError::ExecutionInterrupted);
            }
            match outcome {
                Ok(unit_id) if unique_unit_ids.insert(unit_id) => unit_ids.push(unit_id),
                Ok(unit_id) => {
                    rejected = Some(SpawnRejection::DuplicateSpawnedUnitId(unit_id));
                    break;
                }
                Err(error) => {
                    rejected = Some(SpawnRejection::MutationFailed {
                        game_error_code: error.code().to_owned(),
                    });
                    break;
                }
            }
        }
        let rollback = rejected.is_some();
        let settlement = if let Some(reason) = rejected {
            SpawnRequestSettlement::Rejected { reason }
        } else if evolved {
            SpawnRequestSettlement::Evolved { unit_ids }
        } else {
            SpawnRequestSettlement::Spawned { unit_ids }
        };
        // Keep the rollback owner alive through this last fallible copy, also
        // when all callbacks succeeded. Interruption is never a cached rejection.
        let cached = match clone_for_spawn(&settlement) {
            Ok(cached) => cached,
            Err(error) => {
                *self = before;
                return Err(error);
            }
        };
        if rollback {
            *self = before;
        }
        self.spawn_kernel.settle(&normalized.id, cached);
        Ok(settlement)
    }

    pub fn register_spawn_trigger(
        &mut self,
        trigger_id: SpawnTriggerId,
    ) -> Result<bool, SpawnKernelError> {
        self.spawn_kernel.register_trigger(trigger_id)
    }

    pub fn bind_spawn_trigger(
        &mut self,
        trigger_id: &SpawnTriggerId,
        request: SpawnRequest,
    ) -> Result<(), SpawnKernelError> {
        self.spawn_kernel.bind_trigger(trigger_id, request)
    }

    pub fn settle_spawn_trigger(
        &mut self,
        trigger_id: &SpawnTriggerId,
    ) -> Result<SpawnRequestSettlement, SpawnKernelError> {
        self.spawn_kernel.settle_trigger(trigger_id)
    }

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn cancel_spawn_for_summoner_removal(
        &mut self,
        request: SpawnRequest,
        removal_event_id: RemovalEventId,
    ) -> Result<SpawnRequestSettlement, SpawnKernelError> {
        self.spawn_kernel
            .cancel_for_summoner_removal(request, removal_event_id)
    }

    pub fn cancel_spawn_request(
        &mut self,
        request: SpawnRequest,
        reason: SpawnCancellationReason,
    ) -> Result<SpawnRequestSettlement, SpawnKernelError> {
        self.spawn_kernel.cancel_request(request, reason)
    }

    pub fn reject_spawn_request(
        &mut self,
        request: SpawnRequest,
        reason: SpawnRejection,
    ) -> Result<SpawnRequestSettlement, SpawnKernelError> {
        self.spawn_kernel.reject_request(request, reason)
    }

    pub fn record_compendium_candidate(
        &mut self,
        order: u64,
        candidate: CompendiumCandidate,
    ) -> Result<bool, CompendiumCandidateError> {
        self.spawn_kernel
            .record_compendium_candidate(order, candidate)
    }

    pub fn compendium_candidates(&self) -> Vec<CompendiumCandidate> {
        self.spawn_kernel.compendium_candidates()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompendiumCandidateKind {
    Abnormality,
    CorrodedEmployee,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompendiumCandidate {
    pub id: String,
    pub kind: CompendiumCandidateKind,
    pub base_uuid: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompendiumCandidateError {
    OrderConflict(u64),
    IdentityConflict(String),
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, sync::Arc};

    use uuid::Uuid;

    use crate::game::{battle::scenario::BattleScenario, data::GameDataBuilder};

    use super::*;

    fn core() -> BattleCore {
        BattleCore::new_from_scenario(
            BattleScenario::empty((8, 8)),
            Arc::new(GameDataBuilder::empty().build()),
            7,
        )
    }

    fn request(policy: SpawnOccupancyPolicy) -> SpawnRequest {
        SpawnRequest {
            id: SpawnRequestId::from("wave:alpha:1"),
            source: SpawnRequestSource::Wave {
                wave_id: "alpha".to_owned(),
            },
            owner: Side::Opponent,
            occupancy: policy,
            placements: vec![
                SpawnPlacement {
                    order: 1,
                    position: Position::new(2, 1),
                    terrain_walkable: true,
                    occupants: Vec::new(),
                },
                SpawnPlacement {
                    order: 0,
                    position: Position::new(1, 1),
                    terrain_walkable: true,
                    occupants: Vec::new(),
                },
            ],
        }
    }

    #[test]
    fn ordered_spawn_identity_moves_owner_and_matches_full_normalization() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        for sorted in [false, true] {
            for case in 0..8 {
                let mut input = request(SpawnOccupancyPolicy::RequireUnoccupied);
                match case {
                    1 => input.id.0.clear(),
                    2 => input.occupancy = SpawnOccupancyPolicy::AllowOccupied(Vec::new()),
                    3 => input.placements.clear(),
                    4 => input.placements[1].order = input.placements[0].order,
                    5 => input.placements[1].position = input.placements[0].position,
                    6 => input.placements[0].terrain_walkable = false,
                    7 => input.placements[0].occupants.push(SpawnOccupantKind::OpposingCombatant),
                    _ => (),
                }
                if sorted { input.placements.sort_by_key(|p| p.order); }
                let expected = SpawnKernelState::normalize_request(&input).unwrap().unwrap_or_else(|_| input.clone());
                let was_sorted = input.placements.is_sorted_by_key(|p| p.order);
                let owner = input.placements.as_ptr();
                let actual = SpawnKernelState::identity_request(input).unwrap();
                assert_eq!(actual, expected, "case {case}, sorted {sorted}");
                if was_sorted { assert_eq!(owner, actual.placements.as_ptr(), "retain the existing allocation"); }
            }
        }
        let mut input = request(SpawnOccupancyPolicy::RequireUnoccupied);
        input.placements.sort_by_key(|p| p.order);
        let limits = ExecutionLimits::default();
        let (_, moved) = control::run(&limits, &CancellationToken::default(), ExecutionPhase::TrialPreparation,
            || SpawnKernelState::identity_request(input.clone())).unwrap();
        let (_, copied) = control::run(&limits, &CancellationToken::default(), ExecutionPhase::TrialPreparation,
            || SpawnKernelState::normalize_request(&input)).unwrap();
        assert!(moved.work < copied.work);
        let mut state = SpawnKernelState::default();
        let trigger = SpawnTriggerId::from("ordered-fixture");
        state.register_trigger(trigger.clone()).unwrap();
        let before = format!("{state:?}");
        let failure = control::with_test_cancellation("spawn_identity_order_scan", 1, || control::run(
            &limits, &CancellationToken::default(), ExecutionPhase::TrialPreparation,
            || state.bind_trigger(&trigger, input.clone()),
        ));
        assert_eq!(failure.err().unwrap().code, "authoring_cancelled");
        assert_eq!(format!("{state:?}"), before);
        state.bind_trigger(&trigger, input).unwrap();
    }

    #[test]
    fn occupancy_policy_rejects_ambiguous_allow_lists() {
        assert_eq!(
            SpawnOccupancyPolicy::AllowOccupied(Vec::new()).validate(),
            Err(SpawnPolicyError::EmptyAllowOccupied)
        );
        assert_eq!(
            SpawnOccupancyPolicy::AllowOccupied(vec![
                SpawnOccupancyClass::SameFaction,
                SpawnOccupancyClass::SameFaction,
            ])
            .validate(),
            Err(SpawnPolicyError::DuplicateOccupancyClass(
                SpawnOccupancyClass::SameFaction
            ))
        );
        assert_eq!(
            SpawnOccupancyPolicy::AllowOccupied(vec![
                SpawnOccupancyClass::Any,
                SpawnOccupancyClass::OpposingCombatant,
            ])
            .validate(),
            Err(SpawnPolicyError::AnyMustBeExclusive)
        );
        assert_eq!(
            SpawnOccupancyPolicy::AllowOccupied(vec![
                SpawnOccupancyClass::Summoner,
                SpawnOccupancyClass::SameFaction,
            ])
            .validate(),
            Err(SpawnPolicyError::SummonerOverlapsSameFaction)
        );
    }

    #[test]
    fn owned_clone_shape_spawn_state_keeps_cached_settlement_and_request_payload() {
        use crate::authoring::{
            clone_shape::CloneShape,
            control::{self, CancellationToken, CloneMeter, ExecutionLimits, ExecutionPhase},
        };
        let mut core = core();
        let request = request(SpawnOccupancyPolicy::RequireUnoccupied);
        let settlement = core
            .execute_spawn_request(request.clone(), |_, placement| {
                Ok(Uuid::from_u128(100 + placement.order as u128).into())
            })
            .unwrap();
        let original = format!("{:?}", core.spawn_kernel);
        let (candidate, usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                let mut meter = CloneMeter::default();
                core.spawn_kernel.visit_clone_shape(&mut meter).unwrap();
                meter.finish("spawn_storage", "spawn_copy").unwrap();
                core.spawn_kernel.clone()
            },
        )
        .unwrap();
        assert_eq!(format!("{candidate:?}"), original);
        assert_eq!(candidate.settlement(&request.id), Some(&settlement));
        let mut copied = false;
        let failure = control::run(
            &ExecutionLimits {
                max_work: usage.work - 1,
                ..Default::default()
            },
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                let mut meter = CloneMeter::default();
                core.spawn_kernel.visit_clone_shape(&mut meter)?;
                meter.finish("spawn_storage", "spawn_copy")?;
                let _candidate = core.spawn_kernel.clone();
                copied = true;
                Ok::<_, GameError>(())
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_work_limit");
        assert!(!copied);
        assert_eq!(format!("{:?}", core.spawn_kernel), original);
        let retry = core
            .execute_spawn_request(request, |_, _| panic!("cached request mutated"))
            .unwrap();
        assert_eq!(retry, settlement);
    }

    #[test]
    fn preflight_checks_every_occupant_and_rejects_whole_batch() {
        let mut core = core();
        let mut request = request(SpawnOccupancyPolicy::AllowOccupied(vec![
            SpawnOccupancyClass::SameFaction,
        ]));
        request.placements[0].occupants = vec![
            SpawnOccupantKind::SameFaction,
            SpawnOccupantKind::OpposingCombatant,
        ];
        let called = Cell::new(false);

        let settlement = core
            .execute_spawn_request(request, |_, _| {
                called.set(true);
                Ok(UnitInstanceId(Uuid::from_u128(99)))
            })
            .unwrap();

        assert!(!called.get());
        assert_eq!(
            settlement,
            SpawnRequestSettlement::Rejected {
                reason: SpawnRejection::OccupancyRejected(Position::new(2, 1))
            }
        );
    }

    #[test]
    fn same_faction_allowance_includes_summoner_but_summoner_allowance_is_narrow() {
        let mut core = core();
        let mut broad = request(SpawnOccupancyPolicy::AllowOccupied(vec![
            SpawnOccupancyClass::SameFaction,
        ]));
        broad.placements.truncate(1);
        broad.placements[0].occupants = vec![SpawnOccupantKind::Summoner];
        assert!(matches!(
            core.execute_spawn_request(broad, |_, _| { Ok(UnitInstanceId(Uuid::from_u128(90))) })
                .unwrap(),
            SpawnRequestSettlement::Spawned { .. }
        ));

        let mut narrow = request(SpawnOccupancyPolicy::AllowOccupied(vec![
            SpawnOccupancyClass::Summoner,
        ]));
        narrow.id = SpawnRequestId::from("wave:alpha:2");
        narrow.placements.truncate(1);
        narrow.placements[0].occupants = vec![SpawnOccupantKind::SameFaction];
        assert_eq!(
            core.execute_spawn_request(narrow, |_, _| { Ok(UnitInstanceId(Uuid::from_u128(91))) })
                .unwrap(),
            SpawnRequestSettlement::Rejected {
                reason: SpawnRejection::OccupancyRejected(Position::new(2, 1))
            }
        );
    }

    #[test]
    fn mutation_failure_after_an_earlier_placement_rolls_back_and_is_cached_exactly_once() {
        let mut core = core();
        let before_event_queue = core.event_queue_debug_snapshot();
        let before_event_log = core.event_log.to_json_string().unwrap();
        let attempts = Cell::new(0);
        let request = request(SpawnOccupancyPolicy::RequireUnoccupied);

        let first = core
            .execute_spawn_request(request.clone(), |core, placement| {
                attempts.set(attempts.get() + 1);
                if placement.order == 0 {
                    core.skill_cast_seq = 10;
                    return Ok(UnitInstanceId(Uuid::from_u128(10)));
                }
                core.skill_cast_seq = 99;
                Err(GameError::InvalidAction)
            })
            .unwrap();
        let replay = core
            .execute_spawn_request(request, |_, _| {
                attempts.set(attempts.get() + 1);
                Ok(UnitInstanceId(Uuid::from_u128(11)))
            })
            .unwrap();

        assert_eq!(first, replay);
        assert_eq!(attempts.get(), 2);
        assert_eq!(core.skill_cast_seq, 0);
        assert_eq!(core.event_queue_debug_snapshot(), before_event_queue);
        assert_eq!(core.event_log.to_json_string().unwrap(), before_event_log);
    }

    #[test]
    fn authoring_candidate_preparation_failure_is_not_a_cached_spawn_rejection() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        use crate::game::battle::event_log::BattleLogEvent;
        let mut core = core();
        core.record_event_log(
            0,
            BattleLogEvent::BattleStart {
                width: 8,
                height: 8,
            },
        );
        let original_log = core.event_log.to_json_string().unwrap();
        let request = request(SpawnOccupancyPolicy::RequireUnoccupied);
        let calls = Cell::new(0);
        // Admit the actual normalization/reservation prefix, then reject the
        // larger whole-core snapshot. Earlier prefix failures have their own test.
        let (_, prefix_usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                let normalized = SpawnKernelState::normalize_request(&request)?.unwrap();
                SpawnKernelState::default().reserve_request(clone_for_spawn(&normalized)?)
            },
        )
        .unwrap();
        let limits = ExecutionLimits {
            max_clone_slots: prefix_usage.clone_slots_peak,
            ..ExecutionLimits::default()
        };
        let failure = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                let result = core.execute_spawn_request(request.clone(), |_, _| {
                    calls.set(calls.get() + 1);
                    Ok(UnitInstanceId(Uuid::from_u128(1)))
                });
                assert_eq!(result, Err(SpawnKernelError::ExecutionInterrupted));
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_clone_limit");
        assert_eq!(calls.get(), 0);
        assert_eq!(core.event_log.to_json_string().unwrap(), original_log);
        // Reservation predates the rollback snapshot; it remains retryable.
        assert!(matches!(
            core.spawn_kernel.requests[&request.id].status,
            SpawnRequestStatus::Pending
        ));
        let (result, _) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                core.execute_spawn_request(request.clone(), |_, placement| {
                    calls.set(calls.get() + 1);
                    Ok(UnitInstanceId(Uuid::from_u128(placement.order as u128 + 1)))
                })
            },
        )
        .unwrap();
        assert!(matches!(
            result.unwrap(),
            SpawnRequestSettlement::Spawned { .. }
        ));
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn authoring_spawn_interrupt_restores_mutations_and_stops_before_next_placement() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        use crate::game::battle::event_log::BattleLogEvent;
        let mut core = core();
        let request = request(SpawnOccupancyPolicy::RequireUnoccupied);
        let before_log = core.event_log.to_json_string().unwrap();
        let before_seq = core.event_log_seq;
        let cancellation = CancellationToken::default();
        let calls = Cell::new(0);
        let failure = control::run(
            &ExecutionLimits::default(),
            &cancellation,
            ExecutionPhase::TrialExecution,
            || {
                let result = core.execute_spawn_request(request.clone(), |core, _| {
                    calls.set(calls.get() + 1);
                    core.skill_cast_seq = 91;
                    core.record_event_log(
                        1,
                        BattleLogEvent::BattleStart {
                            width: 8,
                            height: 8,
                        },
                    );
                    cancellation.cancel();
                    Err(GameError::InvalidAction)
                });
                assert_eq!(result, Err(SpawnKernelError::ExecutionInterrupted));
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_cancelled");
        assert_eq!(calls.get(), 1);
        assert_eq!(core.skill_cast_seq, 0);
        assert_eq!(core.event_log_seq, before_seq);
        assert_eq!(core.event_log.to_json_string().unwrap(), before_log);
        assert!(matches!(
            core.spawn_kernel.requests[&request.id].status,
            SpawnRequestStatus::Pending
        ));
    }

    #[test]
    fn authoring_spawn_normalization_interrupt_never_falls_back_or_reserves() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let limits = ExecutionLimits {
            max_clone_slots: 1,
            ..Default::default()
        };
        let original = request(SpawnOccupancyPolicy::RequireUnoccupied);
        let mut candidate = core();
        let calls = Cell::new(0);
        let failure = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                let result = candidate.execute_spawn_request(original.clone(), |_, _| {
                    calls.set(calls.get() + 1);
                    Ok(Uuid::from_u128(1).into())
                });
                assert_eq!(result, Err(SpawnKernelError::ExecutionInterrupted));
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_clone_limit");
        assert_eq!(calls.get(), 0);
        assert!(candidate.spawn_kernel.requests.is_empty());

        // The identity fallback is used by cancellation/binding too. A control
        // error there must not become an owned request or terminal cancellation.
        let failure = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                let result = candidate.spawn_kernel.cancel_request(
                    original.clone(),
                    SpawnCancellationReason::OwnerStateExited {
                        state_activation_id: 7,
                    },
                );
                assert_eq!(result, Err(SpawnKernelError::ExecutionInterrupted));
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_clone_limit");
        assert!(candidate.spawn_kernel.requests.is_empty());

        let mut invalid = original;
        invalid.placements[0].terrain_walkable = false;
        let identity = SpawnKernelState::identity_request(invalid.clone()).unwrap();
        assert_eq!(identity, invalid); // Original order survives domain rejection.
        let settlement = candidate
            .execute_spawn_request(invalid, |_, _| panic!("invalid request mutated"))
            .unwrap();
        assert!(matches!(
            settlement,
            SpawnRequestSettlement::Rejected { .. }
        ));
    }

    #[test]
    fn authoring_spawn_final_copy_failure_restores_and_cached_retry_does_not_mutate() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        // Cover both successful callbacks and an ordinary domain mutation error.
        for domain_error in [false, true] {
            let original = request(SpawnOccupancyPolicy::RequireUnoccupied);
            let initial = core();
            let mutate = |core: &mut BattleCore, placement: &SpawnPlacement| {
                core.skill_cast_seq = u64::from(placement.order) + 100;
                if domain_error && placement.order == 1 {
                    Err(GameError::InvalidAction)
                } else {
                    Ok(Uuid::from_u128(u128::from(placement.order) + 1).into())
                }
            };
            let mut measured = initial.clone();
            let (expected, usage) = control::run(
                &ExecutionLimits::default(),
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || measured.execute_spawn_request(original.clone(), mutate),
            )
            .unwrap();
            let expected = expected.unwrap();
            let mut exact = initial.clone();
            let (result, exact_usage) = control::run(
                &ExecutionLimits {
                    max_work: usage.work,
                    ..Default::default()
                },
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || exact.execute_spawn_request(original.clone(), mutate),
            )
            .unwrap();
            assert_eq!(result.unwrap(), expected);
            assert_eq!(usage, exact_usage);

            let mut candidate = initial.clone();
            let calls = Cell::new(0);
            let failure = control::run(
                &ExecutionLimits {
                    max_work: usage.work - 1,
                    ..Default::default()
                },
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || {
                    let result =
                        candidate.execute_spawn_request(original.clone(), |core, placement| {
                            calls.set(calls.get() + 1);
                            mutate(core, placement)
                        });
                    assert_eq!(result, Err(SpawnKernelError::ExecutionInterrupted));
                },
            )
            .unwrap_err();
            assert_eq!(failure.code, "authoring_work_limit");
            assert_eq!(failure.site, "owned_clone_copy");
            assert_eq!(calls.get(), 2);
            assert_eq!(candidate.skill_cast_seq, initial.skill_cast_seq);
            assert_eq!(
                candidate.event_queue_debug_snapshot(),
                initial.event_queue_debug_snapshot()
            );
            assert_eq!(
                candidate.event_log.to_json_string().unwrap(),
                initial.event_log.to_json_string().unwrap()
            );
            assert!(matches!(
                candidate.spawn_kernel.requests[&original.id].status,
                SpawnRequestStatus::Pending
            ));
            assert_eq!(
                candidate
                    .execute_spawn_request(original.clone(), mutate)
                    .unwrap(),
                expected
            );

            let before_cache = format!("{:?}", candidate.spawn_kernel);
            let (cached, cache_usage) = control::run(
                &ExecutionLimits::default(),
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || {
                    candidate.execute_spawn_request(original.clone(), |_, _| {
                        panic!("cached request mutated")
                    })
                },
            )
            .unwrap();
            assert_eq!(cached.unwrap(), expected);
            let failure = control::run(
                &ExecutionLimits {
                    max_work: cache_usage.work - 1,
                    ..Default::default()
                },
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || {
                    assert_eq!(
                        candidate.execute_spawn_request(original.clone(), |_, _| panic!(
                            "cached request mutated"
                        )),
                        Err(SpawnKernelError::ExecutionInterrupted)
                    );
                },
            )
            .unwrap_err();
            assert_eq!(failure.code, "authoring_work_limit");
            assert_eq!(failure.site, "owned_clone_copy");
            assert_eq!(format!("{:?}", candidate.spawn_kernel), before_cache);
            assert_eq!(
                candidate
                    .execute_spawn_request(original, |_, _| panic!("cached request mutated"))
                    .unwrap(),
                expected
            );
        }
    }

    #[test]
    fn removal_first_cancels_bound_summon_and_restore_does_not_refire() {
        let mut core = core();
        let trigger_id = SpawnTriggerId::from("trigger:boss:50-percent");
        let summoner = UnitInstanceId(Uuid::from_u128(1));
        let request = SpawnRequest {
            id: SpawnRequestId::from("summon:boss:50-percent"),
            source: SpawnRequestSource::Summon {
                summoner,
                definition_id: "reinforcement".to_owned(),
                count: 2,
            },
            ..request(SpawnOccupancyPolicy::RequireUnoccupied)
        };

        assert!(core.register_spawn_trigger(trigger_id.clone()).unwrap());
        core.bind_spawn_trigger(&trigger_id, request.clone())
            .unwrap();
        let canceled = core
            .cancel_spawn_for_summoner_removal(request.clone(), RemovalEventId::from("remove:boss"))
            .unwrap();
        assert!(matches!(canceled, SpawnRequestSettlement::Canceled { .. }));
        assert_eq!(core.settle_spawn_trigger(&trigger_id).unwrap(), canceled);

        let mut restored = core.clone();
        let attempts = Cell::new(0);
        let replay = restored
            .execute_spawn_request(request, |_, _| {
                attempts.set(attempts.get() + 1);
                Ok(summoner)
            })
            .unwrap();
        assert_eq!(replay, canceled);
        assert_eq!(attempts.get(), 0);
        assert!(!restored.register_spawn_trigger(trigger_id).unwrap_or(false));
    }

    #[test]
    fn bound_trigger_rejects_same_request_id_with_changed_payload() {
        let mut core = core();
        let trigger_id = SpawnTriggerId::from("trigger:wave:alpha");
        let original = request(SpawnOccupancyPolicy::RequireUnoccupied);
        assert!(core.register_spawn_trigger(trigger_id.clone()).unwrap());
        core.bind_spawn_trigger(&trigger_id, original.clone())
            .unwrap();
        let mut changed = original;
        changed.owner = Side::Player;

        assert!(matches!(
            core.bind_spawn_trigger(&trigger_id, changed),
            Err(SpawnKernelError::RequestIdentityConflict { .. })
        ));
    }

    #[test]
    fn compendium_candidates_are_ordered_and_idempotent() {
        let mut core = core();
        let corrosion = CompendiumCandidate {
            id: "corrosion:2".to_owned(),
            kind: CompendiumCandidateKind::CorrodedEmployee,
            base_uuid: Uuid::from_u128(2),
        };
        let abnormality = CompendiumCandidate {
            id: "abnormality:1".to_owned(),
            kind: CompendiumCandidateKind::Abnormality,
            base_uuid: Uuid::from_u128(1),
        };

        assert!(core
            .record_compendium_candidate(20, corrosion.clone())
            .unwrap());
        assert!(core
            .record_compendium_candidate(10, abnormality.clone())
            .unwrap());
        assert!(!core
            .record_compendium_candidate(30, corrosion.clone())
            .unwrap());
        assert_eq!(core.compendium_candidates(), vec![abnormality, corrosion]);

        let conflicting = CompendiumCandidate {
            id: "corrosion:2".to_owned(),
            kind: CompendiumCandidateKind::CorrodedEmployee,
            base_uuid: Uuid::from_u128(999),
        };
        assert_eq!(
            core.record_compendium_candidate(30, conflicting),
            Err(CompendiumCandidateError::IdentityConflict(
                "corrosion:2".to_owned()
            ))
        );
    }

    #[test]
    fn successful_batch_preserves_authored_order_without_a_numeric_cap() {
        let mut core = core();
        let mut request = request(SpawnOccupancyPolicy::AllowOccupied(vec![
            SpawnOccupancyClass::Any,
        ]));
        request.placements = (0..128)
            .rev()
            .map(|order| SpawnPlacement {
                order,
                position: Position::new(order as i32, 0),
                terrain_walkable: true,
                occupants: vec![SpawnOccupantKind::StaticDefenseObject],
            })
            .collect();

        let mut observed_orders = Vec::new();
        let settlement = core
            .execute_spawn_request(request, |_, placement| {
                observed_orders.push(placement.order);
                Ok(UnitInstanceId(Uuid::from_u128(
                    u128::from(placement.order) + 1,
                )))
            })
            .unwrap();
        assert_eq!(observed_orders.len(), 128);
        assert!(observed_orders.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(matches!(
            settlement,
            SpawnRequestSettlement::Spawned { ref unit_ids } if unit_ids.len() == 128
        ));
    }

    #[test]
    fn duplicate_unit_identity_rejects_and_rolls_back_the_batch() {
        let mut core = core();
        let duplicate = UnitInstanceId(Uuid::from_u128(3));
        let settlement = core
            .execute_spawn_request(
                request(SpawnOccupancyPolicy::RequireUnoccupied),
                |core, _| {
                    core.skill_cast_seq += 1;
                    Ok(duplicate)
                },
            )
            .unwrap();

        assert_eq!(
            settlement,
            SpawnRequestSettlement::Rejected {
                reason: SpawnRejection::DuplicateSpawnedUnitId(duplicate)
            }
        );
        assert_eq!(core.skill_cast_seq, 0);
    }

    #[test]
    fn request_identity_cannot_be_reused_with_different_payload() {
        let mut core = core();
        let original = request(SpawnOccupancyPolicy::RequireUnoccupied);
        core.execute_spawn_request(original.clone(), |_, _| {
            Ok(UnitInstanceId(Uuid::from_u128(1)))
        })
        .unwrap();
        let mut changed = original;
        changed.owner = Side::Player;

        assert!(matches!(
            core.execute_spawn_request(changed, |_, _| { Ok(UnitInstanceId(Uuid::from_u128(1))) }),
            Err(SpawnKernelError::RequestIdentityConflict { .. })
        ));
    }
}
