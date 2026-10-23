//! Battle event log contract.
//!
//! `BattleEventLog` is the append-only log of battle events produced by the
//! current DefenseRoute live battle model. It is not an offline replay driver;
//! it powers the current battle result, developer traces, and behavior tests. Live
//! `battle_update.events_delta` payloads are converted to explicit wire DTOs at
//! the transport boundary.

use std::{
    io::{self, Write},
    ops::{Deref, DerefMut},
    sync::Arc,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

use crate::{
    game::resources::Position,
    game::{
        ability::{
            SkillAreaTickPolicy, SkillAreaTracking, SkillHitTargetFilter, SkillId,
            SkillPresentationDef,
        },
        battle::cooldown::CooldownSource,
        battle::core::movement::{types::EventLogVec2, MovementSegmentEndKind},
        battle::damage::{DamageBreakdown, DamageFeedbackTag, DamageSource, DamageType},
        battle::ids::UnitInstanceId,
        battle::{
            buffs::BuffId,
            types::{
                BattleUnitRole, BattleUnitSourceIdentity, BattleUnitThreatClass, BattleWinner,
                MobilityKind,
            },
        },
        enums::Side,
        stats::{StatModifier, UnitStats},
    },
};

pub const BATTLE_EVENT_LOG_VERSION: u32 = 32;
pub const BATTLE_HISTORY_MAX_EVENTS: usize = 8_192;
pub const BATTLE_HISTORY_MAX_ENCODED_BYTES: usize = 4 * 1024 * 1024;
pub const BATTLE_HISTORY_MAX_SPAN_MS: u64 = 180_000;

#[derive(Default)]
struct EncodedLengthWriter {
    len: usize,
}

impl Write for EncodedLengthWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.len = self
            .len
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("encoded event length overflow"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn encoded_json_len(value: &impl Serialize) -> Option<usize> {
    let mut writer = EncodedLengthWriter::default();
    serde_json::to_writer(&mut writer, value)
        .ok()
        .map(|()| writer.len)
}

const DEFERRED_EVENT_STATIC_UPPER_BYTES: usize = 2_048;
const DEFERRED_EVENT_MAX_UPPER_BYTES: usize = 16 * 1_024;
const ORDINARY_HISTORY_RESERVE_LIMIT: usize = 186;

#[derive(Debug, Clone, Default)]
pub struct BattleEventEntries(Arc<Vec<BattleEventLogEntry>>, bool);

impl BattleEventEntries {
    fn with_capacity(capacity: usize) -> Self {
        Self(Arc::new(Vec::with_capacity(capacity)), false)
    }
}

impl From<Vec<BattleEventLogEntry>> for BattleEventEntries {
    fn from(entries: Vec<BattleEventLogEntry>) -> Self {
        Self(Arc::new(entries), true)
    }
}

impl Deref for BattleEventEntries {
    type Target = Vec<BattleEventLogEntry>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl DerefMut for BattleEventEntries {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // A public same-length edit can change both encoded size and time.
        // Internal append/evict paths clear this only after updating metadata.
        self.1 = true;
        if Arc::strong_count(&self.0) > 1 && !crate::authoring::control::is_active() {
            // Keep the ordinary writer's small reserve across lazy COW. Copying
            // only len otherwise forces the next append to double its storage.
            // Public callers can reserve arbitrary space: never inherit that
            // spare beyond the ordinary initial reserve or one bounded growth
            // step. Larger logs keep their existing geometric reserve as well.
            let reserve_ceiling = self
                .0
                .len()
                .saturating_mul(2)
                .min(BATTLE_HISTORY_MAX_EVENTS)
                .max(ORDINARY_HISTORY_RESERVE_LIMIT);
            let capacity = self.0.len().max(self.0.capacity().min(reserve_ceiling));
            let mut entries = Vec::with_capacity(capacity);
            entries.extend(self.0.iter().cloned());
            self.0 = Arc::new(entries);
        }
        Arc::make_mut(&mut self.0)
    }
}

impl<'a> IntoIterator for &'a BattleEventEntries {
    type Item = &'a BattleEventLogEntry;
    type IntoIter = std::slice::Iter<'a, BattleEventLogEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut BattleEventEntries {
    type Item = &'a mut BattleEventLogEntry;
    type IntoIter = std::slice::IterMut<'a, BattleEventLogEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl Serialize for BattleEventEntries {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.as_ref().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BattleEventEntries {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<BattleEventLogEntry>::deserialize(deserializer).map(Self::from)
    }
}

#[inline]
fn owned_payload_shape(entry: &BattleEventLogEntry) -> (usize, usize) {
    let (dynamic_string_bytes, collection_elements) = match &entry.event {
        BattleLogEvent::UnitSpawned { unit_source, .. } => {
            let bytes = match unit_source {
                BattleUnitSourceIdentity::Employee { .. }
                | BattleUnitSourceIdentity::DefenseObject { .. }
                | BattleUnitSourceIdentity::TestFixture { .. } => 0,
                BattleUnitSourceIdentity::Abnormality { abnormality_id, .. } => {
                    abnormality_id.len()
                }
                BattleUnitSourceIdentity::CorrodedEmployee { profile_id, .. }
                | BattleUnitSourceIdentity::FacilityEntity { profile_id, .. } => profile_id.len(),
                BattleUnitSourceIdentity::IndependentObject { definition_id, .. }
                | BattleUnitSourceIdentity::SummonedUnit { definition_id, .. } => {
                    definition_id.len()
                }
            };
            (bytes, 0)
        }
        BattleLogEvent::FieldWorkAssignmentStarted {
            object_id,
            work_cell_id,
            ..
        }
        | BattleLogEvent::FieldWorkAssignmentStopped {
            object_id,
            work_cell_id,
            ..
        } => (object_id.len().saturating_add(work_cell_id.len()), 0),
        BattleLogEvent::FieldWorkCompleted { object_id, .. } => (object_id.len(), 0),
        BattleLogEvent::AutoCastStart { skill_id, .. } => {
            (skill_id.as_ref().map_or(0, |id| id.as_str().len()), 0)
        }
        BattleLogEvent::ManualCastStart { skill_id, .. }
        | BattleLogEvent::AbilityCast { skill_id, .. } => (skill_id.as_str().len(), 0),
        BattleLogEvent::SkillCastInterrupted {
            interrupted_skill_id,
            ..
        }
        | BattleLogEvent::SkillCastCancelled {
            interrupted_skill_id,
            ..
        } => (interrupted_skill_id.as_str().len(), 0),
        BattleLogEvent::TriggeredAbilityProc { skill_id, .. } => (skill_id.as_str().len(), 0),
        BattleLogEvent::AbilityStepTriggered {
            skill_id,
            step_id,
            presentation,
            ..
        } => {
            let presentation_bytes = presentation.as_ref().map_or(0, |presentation| {
                presentation
                    .cast_state
                    .as_ref()
                    .map_or(0, String::len)
                    .saturating_add(
                        presentation
                            .projectile_vfx_id
                            .as_ref()
                            .map_or(0, String::len),
                    )
                    .saturating_add(presentation.impact_vfx_id.as_ref().map_or(0, String::len))
                    .saturating_add(presentation.target_anchor.as_ref().map_or(0, String::len))
            });
            (
                skill_id
                    .as_str()
                    .len()
                    .saturating_add(step_id.len())
                    .saturating_add(presentation_bytes),
                0,
            )
        }
        BattleLogEvent::SkillAreaDeclared {
            skill_id,
            step_id,
            shape,
            ..
        } => {
            let BattleSkillAreaShape::TilePattern { affected_tiles } = shape;
            (
                skill_id.as_str().len().saturating_add(step_id.len()),
                affected_tiles.len(),
            )
        }
        BattleLogEvent::SkillProjectileLaunched {
            skill_id,
            step_id,
            projectile_vfx_id,
            ..
        } => (
            skill_id
                .as_str()
                .len()
                .saturating_add(step_id.len())
                .saturating_add(projectile_vfx_id.as_ref().map_or(0, String::len)),
            0,
        ),
        BattleLogEvent::SkillProjectileImpacted {
            skill_id,
            step_id,
            impact_vfx_id,
            ..
        } => (
            skill_id
                .as_str()
                .len()
                .saturating_add(step_id.len())
                .saturating_add(impact_vfx_id.as_ref().map_or(0, String::len)),
            0,
        ),
        BattleLogEvent::IndependentObjectRemoved { receipt } => (receipt.definition.0.len(), 0),
        BattleLogEvent::IndependentObjectEvolved { receipt } => (
            receipt
                .previous_definition
                .0
                .len()
                .saturating_add(receipt.definition.0.len()),
            0,
        ),
        BattleLogEvent::HpChanged {
            damage_breakdown,
            feedback_tags,
            ..
        } => (
            0,
            damage_breakdown
                .as_ref()
                .map_or(0, Vec::len)
                .saturating_add(feedback_tags.len()),
        ),
        BattleLogEvent::CorrosionStarted { .. }
        | BattleLogEvent::CorrosionEnded { .. }
        | BattleLogEvent::BattleStart { .. }
        | BattleLogEvent::ItemSpawned { .. }
        | BattleLogEvent::UnitDeployed { .. }
        | BattleLogEvent::StanceChanged { .. }
        | BattleLogEvent::UnitTacticallyRelocated { .. }
        | BattleLogEvent::UnitDisplaced { .. }
        | BattleLogEvent::UnitsTacticallySwapped { .. }
        | BattleLogEvent::UnitWithdrawn { .. }
        | BattleLogEvent::MovementSegmentStarted { .. }
        | BattleLogEvent::MovementStopped { .. }
        | BattleLogEvent::AttackStart { .. }
        | BattleLogEvent::AttackResolve { .. }
        | BattleLogEvent::AttackMiss { .. }
        | BattleLogEvent::BasicAttackProjectileLaunched { .. }
        | BattleLogEvent::BasicAttackProjectileImpacted { .. }
        | BattleLogEvent::AutoCastEnd { .. }
        | BattleLogEvent::ManualCastEnd { .. }
        | BattleLogEvent::BuffApplied { .. }
        | BattleLogEvent::BuffTick { .. }
        | BattleLogEvent::BuffExpired { .. }
        | BattleLogEvent::StatChanged { .. }
        | BattleLogEvent::ResonanceChanged { .. }
        | BattleLogEvent::UnitDied { .. }
        | BattleLogEvent::BattleEnd { .. } => (0, 0),
    };
    (dynamic_string_bytes, collection_elements)
}

#[inline]
fn deferred_encoded_len_upper(entry: &BattleEventLogEntry) -> Option<usize> {
    let source_command_bytes = entry.source_command_id.as_ref().map_or(0, String::len);
    let (dynamic_string_bytes, collection_elements) = owned_payload_shape(entry);
    let upper = DEFERRED_EVENT_STATIC_UPPER_BYTES
        .checked_add(source_command_bytes.checked_mul(6)?)?
        .checked_add(dynamic_string_bytes.checked_mul(6)?)?
        .checked_add(collection_elements.checked_mul(256)?)?;
    (upper <= DEFERRED_EVENT_MAX_UPPER_BYTES).then_some(upper)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleHistoryCursorError {
    SnapshotResyncRequired {
        requested: u64,
        oldest_available_after_seq: u64,
        latest: u64,
    },
    InvalidFuture {
        requested: u64,
        latest: u64,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MovementStopReason {
    Rooted,
    Displaced,
    TargetAcquired,
    AttackStarted,
    CastStarted,
    HardCC,
    WaitRepath,
    /// Logical arrival: the destination tile is now occupied.
    /// The accompanying continuous coordinates remain the actual simulation-space
    /// stop position and are not snapped for presentation.
    Arrived,
    Died,
    InvalidState,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BattleEventRootCause {
    /// Root entries created during battle initialization (spawns, start markers, etc).
    Init,
    /// Root entries emitted by scheduled periodic systems (e.g., auto-attacks).
    Period,
    /// Root entries emitted by the engine/system when there is no meaningful parent seq.
    System,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "cause_type", rename_all = "snake_case")]
pub enum BattleEventCause {
    Parent { seq: u64 },
    Root { kind: BattleEventRootCause },
}

impl BattleEventCause {
    pub fn parent_seq(&self) -> Option<u64> {
        match self {
            Self::Parent { seq } => Some(*seq),
            Self::Root { .. } => None,
        }
    }

    pub fn root_kind(&self) -> Option<BattleEventRootCause> {
        match self {
            Self::Parent { .. } => None,
            Self::Root { kind } => Some(*kind),
        }
    }
}

impl Default for BattleEventCause {
    fn default() -> Self {
        Self::Root {
            kind: BattleEventRootCause::System,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttackKind {
    /// Scheduled auto attack (attack interval based).
    Auto,
    /// Triggered/one-off attack (e.g., ability extra attack).
    Triggered,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttackDelivery {
    Instant,
    Projectile,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BattleProjectileGuidance {
    Homing,
    Fixed,
}

/// Bounded, diagnostic-only active battle event window.
///
/// Retention is limited independently by event count, compact-JSON encoded
/// entry-array bytes, and battle-time span. Sequence identity survives eviction;
/// gameplay result and continuation authority live outside this diagnostic type.
#[derive(Debug, Clone, Serialize)]
pub struct BattleEventLog {
    #[serde(skip)]
    limits: crate::runtime_safety::BattleHistoryLimits,
    pub version: u32,
    pub entries: BattleEventEntries,
    #[serde(skip)]
    floor_seq: u64,
    #[serde(skip)]
    latest_seq: u64,
    #[serde(skip)]
    earliest_time_ms: u64,
    #[serde(skip)]
    latest_time_ms: u64,
    #[serde(skip)]
    encoded_entry_array_bytes: usize,
    #[serde(skip)]
    accounted_entry_count: usize,
    #[serde(skip)]
    observed_entry_count: usize,
    #[serde(skip)]
    unaccounted_encoded_upper_bytes: usize,
    #[serde(skip)]
    sequence_exhausted: bool,
}

// Whole log Clone shares the entries Arc; write-side COW is separately admitted
// by prepare_authoring_write after the actual candidate has been cloned.
impl crate::authoring::clone_shape::CloneShape for BattleEventEntries {
    fn visit_clone_shape(
        &self,
        meter: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        let Self(entries, _dirty) = self;
        // The dirty bit is inline bookkeeping, with no additional owned storage.
        meter.scan_storage(Some(1))?;
        crate::authoring::clone_shape::CloneShape::visit_clone_shape(entries, meter)
    }
}
crate::authoring::clone_shape::struct_shape!(BattleEventLog {
    limits,
    version,
    entries,
    floor_seq,
    latest_seq,
    earliest_time_ms,
    latest_time_ms,
    encoded_entry_array_bytes,
    accounted_entry_count,
    observed_entry_count,
    unaccounted_encoded_upper_bytes,
    sequence_exhausted,
});

impl BattleEventLog {
    /// Prepare a controlled candidate for its first write. Clone keeps the
    /// existing Arc semantics; callers invoke this fallible seam after staging,
    /// before entering the infallible event recorder. Ordinary calls keep lazy
    /// COW. Entry records, nested elements and shared source handles are storage
    /// units; string lengths are copy work, not exact allocator/RSS bytes.
    pub(crate) fn prepare_authoring_write(
        &mut self,
    ) -> Result<(), crate::game::behavior::GameError> {
        use crate::authoring::control::{self, CloneMeter};
        if !control::is_active() {
            return Ok(());
        }
        control::poll("battle_log_write_preparation")?;
        if Arc::strong_count(&self.entries.0) == 1 {
            return Ok(());
        }

        // Vec::clone copies live entries, not the source's spare capacity.
        // Admit the scan before visiting entries; each payload shape is O(1).
        let entries = self.entries.len().saturating_add(1);
        let mut meter = CloneMeter::default();
        meter.add_storage(Some(entries));
        meter.admit_storage("battle_log_cow_entries")?;
        control::charge(entries as u64, "battle_log_cow_shape")?;
        for entry in &self.entries {
            control::poll("battle_log_cow_entry")?;
            let (strings, elements) = owned_payload_shape(entry);
            meter.add_storage(
                elements.checked_add(usize::from(entry.skill_effect_source.is_some())),
            );
            meter.add_string_bytes(strings);
            meter.add_string_bytes(entry.source_command_id.as_ref().map_or(0, String::len));
        }
        meter.finish("battle_log_cow_storage", "battle_log_cow_copy")?;
        Arc::make_mut(&mut self.entries.0);
        control::poll("battle_log_cow_complete")
    }

    pub fn new() -> Self {
        Self::with_runtime_safety(crate::runtime_safety::RuntimeSafetySettings::embedded())
    }

    pub fn with_runtime_safety(settings: &crate::runtime_safety::RuntimeSafetySettings) -> Self {
        Self::with_history_limits(settings.battle_history())
    }

    fn with_history_limits(limits: crate::runtime_safety::BattleHistoryLimits) -> Self {
        Self {
            limits,
            version: BATTLE_EVENT_LOG_VERSION,
            entries: BattleEventEntries::with_capacity(
                ORDINARY_HISTORY_RESERVE_LIMIT.min(limits.max_events),
            ),
            floor_seq: 0,
            latest_seq: 0,
            earliest_time_ms: 0,
            latest_time_ms: 0,
            encoded_entry_array_bytes: 2,
            accounted_entry_count: 0,
            observed_entry_count: 0,
            unaccounted_encoded_upper_bytes: 0,
            sequence_exhausted: false,
        }
    }

    pub fn from_entries(entries: Vec<BattleEventLogEntry>) -> Self {
        Self::from_entries_with_runtime_safety(
            entries,
            crate::runtime_safety::RuntimeSafetySettings::embedded(),
        )
    }

    pub fn from_entries_with_runtime_safety(
        entries: Vec<BattleEventLogEntry>,
        settings: &crate::runtime_safety::RuntimeSafetySettings,
    ) -> Self {
        let mut event_log = Self::with_runtime_safety(settings);
        for entry in entries {
            event_log.append(entry);
        }
        event_log
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn entries_after_seq(&self, last_seen_seq: u64) -> Vec<BattleEventLogEntry> {
        let start = self
            .entries
            .partition_point(|entry| entry.seq <= last_seen_seq);
        self.entries[start..].iter().cloned().collect()
    }

    pub fn floor_seq(&self) -> u64 {
        self.floor_seq
    }

    pub fn latest_seq(&self) -> u64 {
        self.latest_seq
            .max(self.entries.last().map(|entry| entry.seq).unwrap_or(0))
    }

    #[cfg(feature = "runtime-observation")]
    pub(crate) fn observation_usage(&self) -> [u64; 3] {
        // The opaque application path reconciles accounting before publication.
        // Never traverse potentially dirty public fixture entries for metrics.
        [
            self.observed_entry_count as u64,
            self.encoded_entry_array_bytes
                .saturating_add(self.unaccounted_encoded_upper_bytes) as u64,
            self.latest_time_ms.saturating_sub(self.earliest_time_ms),
        ]
    }

    pub fn encoded_entry_array_bytes(&self) -> usize {
        if self.entries.is_empty() {
            return 2;
        }
        if self.accounted_entry_count == self.entries.len()
            && self.observed_entry_count == self.entries.len()
            && !self.entries.1
        {
            return self.encoded_entry_array_bytes;
        }
        2_usize.saturating_add(
            self.entries
                .iter()
                .filter_map(encoded_json_len)
                .sum::<usize>()
                .saturating_add(self.entries.len().saturating_sub(1)),
        )
    }

    pub fn retained_time_span_ms(&self) -> u64 {
        if self.observed_entry_count == self.entries.len() && !self.entries.1 {
            return self.latest_time_ms().saturating_sub(self.earliest_time_ms);
        }
        self.entries
            .iter()
            .map(|entry| entry.time_ms)
            .min()
            .map_or(0, |earliest| self.latest_time_ms().saturating_sub(earliest))
    }

    pub fn sequence_exhausted(&self) -> bool {
        self.sequence_exhausted
    }

    pub fn validate_cursor(&self, after_seq: u64) -> Result<(), BattleHistoryCursorError> {
        let latest = self.latest_seq();
        if after_seq > latest {
            return Err(BattleHistoryCursorError::InvalidFuture {
                requested: after_seq,
                latest,
            });
        }
        if after_seq < self.floor_seq {
            return Err(BattleHistoryCursorError::SnapshotResyncRequired {
                requested: after_seq,
                oldest_available_after_seq: self.floor_seq,
                latest,
            });
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn append(&mut self, entry: BattleEventLogEntry) {
        if let Some(encoded_upper) = deferred_encoded_len_upper(&entry) {
            self.append_with_deferred_encoded_len(entry, encoded_upper);
        } else {
            let encoded_len = encoded_json_len(&entry);
            self.append_with_encoded_len(entry, encoded_len);
        }
        #[cfg(feature = "runtime-observation")]
        {
            let usage = self.observation_usage();
            crate::runtime_observation::peak(crate::runtime_observation::HISTORY_EVENTS, usage[0]);
            crate::runtime_observation::peak(crate::runtime_observation::HISTORY_BYTES, usage[1]);
            crate::runtime_observation::peak(crate::runtime_observation::HISTORY_SPAN, usage[2]);
        }
    }

    fn reserve_for_next_entry(&mut self) {
        if self.entries.len() < self.entries.capacity() {
            return;
        }
        let additional = self
            .limits
            .max_events
            .saturating_sub(self.entries.capacity())
            .min(self.entries.capacity())
            .max(1);
        self.entries.reserve_exact(additional);
    }

    pub(crate) fn close_for_sequence_exhaustion(&mut self) {
        self.sequence_exhausted = true;
    }

    #[cfg(test)]
    pub(crate) fn append_with_encoded_len_for_test(
        &mut self,
        entry: BattleEventLogEntry,
        encoded_len: Option<usize>,
    ) {
        self.append_with_encoded_len(entry, encoded_len);
    }

    fn append_with_encoded_len(&mut self, entry: BattleEventLogEntry, encoded_len: Option<usize>) {
        if self.sequence_exhausted {
            return;
        }
        self.ensure_accounting();

        let previous_latest = self.latest_seq();
        if previous_latest != 0 && entry.seq != previous_latest.saturating_add(1) {
            self.close_window_through(entry.seq);
            return;
        }
        if previous_latest == 0 && self.floor_seq == 0 {
            self.floor_seq = entry.seq.saturating_sub(1);
        }
        self.latest_seq = entry.seq;
        if self.entries.is_empty() {
            self.earliest_time_ms = entry.time_ms;
        } else {
            self.earliest_time_ms = self.earliest_time_ms.min(entry.time_ms);
        }
        self.latest_time_ms = self.latest_time_ms.max(entry.time_ms);
        let Some(encoded_len) = encoded_len else {
            self.close_window_through(entry.seq);
            return;
        };
        if encoded_len.saturating_add(2) > self.limits.max_encoded_entry_array_bytes {
            self.close_window_through(entry.seq);
            return;
        }

        if !self.entries.is_empty() {
            self.encoded_entry_array_bytes = self.encoded_entry_array_bytes.saturating_add(1);
        }
        self.encoded_entry_array_bytes = self.encoded_entry_array_bytes.saturating_add(encoded_len);
        self.reserve_for_next_entry();
        self.entries.push(entry);
        self.accounted_entry_count = self.accounted_entry_count.saturating_add(1);
        self.observed_entry_count = self.observed_entry_count.saturating_add(1);
        self.entries.1 = false;
        self.enforce_caps();
    }

    #[inline]
    fn append_with_deferred_encoded_len(
        &mut self,
        entry: BattleEventLogEntry,
        encoded_upper: usize,
    ) {
        if self.sequence_exhausted {
            return;
        }
        if self.observed_entry_count != self.entries.len() || self.entries.1 {
            self.ensure_accounting();
        }
        let previous_latest = self.latest_seq();
        if previous_latest != 0 && entry.seq != previous_latest.saturating_add(1) {
            self.close_window_through(entry.seq);
            return;
        }
        if previous_latest == 0 && self.floor_seq == 0 {
            self.floor_seq = entry.seq.saturating_sub(1);
        }
        self.latest_seq = entry.seq;
        if self.entries.is_empty() {
            self.earliest_time_ms = entry.time_ms;
        } else {
            self.earliest_time_ms = self.earliest_time_ms.min(entry.time_ms);
        }
        self.latest_time_ms = self.latest_time_ms.max(entry.time_ms);
        let comma_bytes = usize::from(!self.entries.is_empty());
        self.unaccounted_encoded_upper_bytes = self
            .unaccounted_encoded_upper_bytes
            .saturating_add(encoded_upper)
            .saturating_add(comma_bytes);
        self.reserve_for_next_entry();
        self.entries.push(entry);
        self.observed_entry_count = self.observed_entry_count.saturating_add(1);
        self.entries.1 = false;
        self.enforce_caps();
    }

    #[inline]
    fn enforce_caps(&mut self) {
        if self.entries.len() > self.limits.max_events
            || self.retained_time_span_ms() > self.limits.max_span_ms
            || self
                .encoded_entry_array_bytes
                .saturating_add(self.unaccounted_encoded_upper_bytes)
                > self.limits.max_encoded_entry_array_bytes
        {
            self.ensure_accounting();
        }
        while self.entries.len() > self.limits.max_events
            || self.encoded_entry_array_bytes > self.limits.max_encoded_entry_array_bytes
            || self.retained_time_span_ms() > self.limits.max_span_ms
        {
            self.evict_oldest();
        }
    }

    fn evict_oldest(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let entry = self.entries.remove(0);
        self.entries.1 = false;
        let encoded_len = encoded_json_len(&entry).unwrap_or(self.encoded_entry_array_bytes);
        self.accounted_entry_count = self.accounted_entry_count.saturating_sub(1);
        self.observed_entry_count = self.observed_entry_count.saturating_sub(1);
        self.floor_seq = entry.seq;
        self.encoded_entry_array_bytes = self.encoded_entry_array_bytes.saturating_sub(encoded_len);
        if self.entries.is_empty() {
            self.encoded_entry_array_bytes = 2;
        } else {
            self.encoded_entry_array_bytes = self.encoded_entry_array_bytes.saturating_sub(1);
        }
        if entry.time_ms == self.latest_time_ms {
            self.latest_time_ms = self
                .entries
                .iter()
                .map(|retained| retained.time_ms)
                .max()
                .unwrap_or(0);
        }
        if entry.time_ms == self.earliest_time_ms {
            self.earliest_time_ms = self
                .entries
                .iter()
                .map(|retained| retained.time_ms)
                .min()
                .unwrap_or(0);
        }
    }

    fn close_window_through(&mut self, seq: u64) {
        let through = seq.max(self.latest_seq());
        self.clear_retained_entries();
        self.floor_seq = self.floor_seq.max(through);
        self.latest_seq = self.latest_seq.max(through);
    }

    fn clear_retained_entries(&mut self) {
        self.entries.clear();
        self.entries.1 = false;
        self.accounted_entry_count = 0;
        self.observed_entry_count = 0;
        self.unaccounted_encoded_upper_bytes = 0;
        self.encoded_entry_array_bytes = 2;
        self.earliest_time_ms = 0;
        self.latest_time_ms = 0;
    }

    fn ensure_accounting(&mut self) {
        if self.entries.is_empty() {
            self.encoded_entry_array_bytes = 2;
            self.accounted_entry_count = 0;
            self.observed_entry_count = 0;
            self.unaccounted_encoded_upper_bytes = 0;
            self.earliest_time_ms = 0;
            self.entries.1 = false;
            return;
        }
        if self.accounted_entry_count == self.entries.len()
            && self.observed_entry_count == self.entries.len()
            && !self.entries.1
        {
            return;
        }
        let encoded_lengths = self
            .entries
            .iter()
            .map(encoded_json_len)
            .collect::<Option<Vec<_>>>();
        let Some(encoded_lengths) = encoded_lengths else {
            let latest = self.entries.last().map(|entry| entry.seq).unwrap_or(0);
            self.close_window_through(latest);
            return;
        };
        if self.floor_seq == 0 {
            self.floor_seq = self
                .entries
                .first()
                .map(|entry| entry.seq.saturating_sub(1))
                .unwrap_or(0);
        }
        self.latest_seq = self
            .latest_seq
            .max(self.entries.last().map(|entry| entry.seq).unwrap_or(0));
        self.latest_time_ms = self
            .entries
            .iter()
            .map(|entry| entry.time_ms)
            .max()
            .unwrap_or(0);
        self.earliest_time_ms = self
            .entries
            .iter()
            .map(|entry| entry.time_ms)
            .min()
            .unwrap_or(0);
        self.encoded_entry_array_bytes = 2_usize
            .saturating_add(encoded_lengths.iter().copied().sum::<usize>())
            .saturating_add(encoded_lengths.len().saturating_sub(1));
        self.accounted_entry_count = encoded_lengths.len();
        self.observed_entry_count = encoded_lengths.len();
        self.unaccounted_encoded_upper_bytes = 0;
        self.entries.1 = false;
    }

    fn latest_time_ms(&self) -> u64 {
        if self.entries.1 {
            return self
                .entries
                .iter()
                .map(|entry| entry.time_ms)
                .max()
                .unwrap_or(0);
        }
        self.latest_time_ms
            .max(self.entries.last().map(|entry| entry.time_ms).unwrap_or(0))
    }
}

impl<'de> Deserialize<'de> for BattleEventLog {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::deserialize_with_runtime_safety(
            deserializer,
            crate::runtime_safety::RuntimeSafetySettings::embedded(),
        )
    }
}

impl BattleEventLog {
    pub fn deserialize_with_runtime_safety<'de, D: Deserializer<'de>>(
        deserializer: D,
        settings: &crate::runtime_safety::RuntimeSafetySettings,
    ) -> Result<Self, D::Error> {
        use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
        struct Entries(crate::runtime_safety::BattleHistoryLimits);
        impl<'de> DeserializeSeed<'de> for Entries {
            type Value = BattleEventLog;
            fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
                struct Window(crate::runtime_safety::BattleHistoryLimits);
                impl<'de> Visitor<'de> for Window {
                    type Value = BattleEventLog;
                    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.write_str("battle history entries")
                    }
                    fn visit_seq<A: SeqAccess<'de>>(
                        self,
                        mut seq: A,
                    ) -> Result<Self::Value, A::Error> {
                        let mut log = BattleEventLog::with_history_limits(self.0);
                        while let Some(entry) = seq.next_element::<BattleEventLogEntry>()? {
                            log.append(entry);
                        }
                        Ok(log)
                    }
                }
                d.deserialize_seq(Window(self.0))
            }
        }
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Version,
            Entries,
        }
        struct LogVisitor(crate::runtime_safety::BattleHistoryLimits);
        impl<'de> Visitor<'de> for LogVisitor {
            type Value = BattleEventLog;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a battle history with version and entries")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let (mut version, mut log) = (None, None);
                while let Some(field) = map.next_key::<Field>()? {
                    match field {
                        Field::Version => {
                            if version.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version = Some(map.next_value::<u32>()?);
                        }
                        Field::Entries => {
                            if log.is_some() {
                                return Err(serde::de::Error::duplicate_field("entries"));
                            }
                            log = Some(map.next_value_seed(Entries(self.0))?);
                        }
                    }
                }
                let mut log = log.ok_or_else(|| serde::de::Error::missing_field("entries"))?;
                log.version = version.ok_or_else(|| serde::de::Error::missing_field("version"))?;
                Ok(log)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let version = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::missing_field("version"))?;
                let mut log = seq
                    .next_element_seed(Entries(self.0))?
                    .ok_or_else(|| serde::de::Error::missing_field("entries"))?;
                log.version = version;
                Ok(log)
            }
        }
        deserializer.deserialize_struct(
            "BattleEventLog",
            &["version", "entries"],
            LogVisitor(settings.battle_history()),
        )
    }
}

impl Default for BattleEventLog {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleEventLogEntry {
    /// Bounded together with this entry; shared definition metadata is retained
    /// by Arc. The serialized byte cap does not account for this core-only field.
    #[serde(skip)]
    pub skill_effect_source: Option<Arc<super::skill_source::SkillEffectExecutionSource>>,
    pub time_ms: u64,
    pub seq: u64,
    #[serde(default)]
    pub cause: BattleEventCause,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_command_id: Option<String>,
    pub event: BattleLogEvent,
}

impl BattleEventLogEntry {
    pub fn cause_seq(&self) -> Option<u64> {
        self.cause.parent_seq()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum SkillCastTarget {
    Unit { unit_instance_id: UnitInstanceId },
    Tile { position: Position },
    TilePair { first: Position, second: Position },
    Direction { facing: crate::game::battle::tile_range::FacingDirection },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BattleSkillAreaShape {
    TilePattern { affected_tiles: Vec<Position> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BattleLogEvent {
    CorrosionStarted {
        actor: UnitInstanceId,
        duration_ms: u64,
    },
    CorrosionEnded {
        actor: UnitInstanceId,
    },
    BattleStart {
        width: u8,
        height: u8,
    },
    ItemSpawned {
        item_instance_id: Uuid,
        owner: Side,
        owner_unit_instance_id: UnitInstanceId,
        base_uuid: Uuid,
    },
    UnitSpawned {
        unit_instance_id: UnitInstanceId,
        owner: Side,
        #[serde(default)]
        role: BattleUnitRole,
        #[serde(default)]
        threat_class: BattleUnitThreatClass,
        #[serde(default)]
        mobility_kind: MobilityKind,
        base_uuid: Uuid,
        unit_source: BattleUnitSourceIdentity,
        world_position: EventLogVec2,
        stats: UnitStats,
    },
    UnitDeployed {
        employee_uuid: Uuid,
        unit_instance_id: UnitInstanceId,
        position: Position,
        facing: crate::game::battle::tile_range::FacingDirection,
    },
    StanceChanged {
        unit_instance_id: UnitInstanceId,
        before: crate::game::battle::types::EngagementStance,
        after: crate::game::battle::types::EngagementStance,
        cooldown_until_ms: u64,
    },
    FieldWorkAssignmentStarted {
        object_id: String,
        work_cell_id: String,
        worker_unit_instance_id: UnitInstanceId,
    },
    FieldWorkAssignmentStopped {
        object_id: String,
        work_cell_id: String,
        worker_unit_instance_id: UnitInstanceId,
    },
    FieldWorkCompleted {
        object_id: String,
        repaired_unit_instance_id: Option<UnitInstanceId>,
        requested_flat_hp: u32,
        actual_repaired_hp: u32,
    },
    UnitDisplaced {
        source_instance_id: UnitInstanceId,
        unit_instance_id: UnitInstanceId,
        from_position: Position,
        to_position: Position,
        from_world_position: EventLogVec2,
        to_world_position: EventLogVec2,
    },
    UnitTacticallyRelocated {
        unit_instance_id: UnitInstanceId,
        from_position: Position,
        to_position: Position,
        from_world_position: EventLogVec2,
        to_world_position: EventLogVec2,
        before_facing: crate::game::battle::tile_range::FacingDirection,
        after_facing: crate::game::battle::tile_range::FacingDirection,
        cooldown_until_ms: u64,
    },
    UnitsTacticallySwapped {
        first_unit_instance_id: UnitInstanceId,
        second_unit_instance_id: UnitInstanceId,
        first_from_position: Position,
        first_to_position: Position,
        second_from_position: Position,
        second_to_position: Position,
        first_from_world_position: EventLogVec2,
        first_to_world_position: EventLogVec2,
        second_from_world_position: EventLogVec2,
        second_to_world_position: EventLogVec2,
        first_before_facing: crate::game::battle::tile_range::FacingDirection,
        first_after_facing: crate::game::battle::tile_range::FacingDirection,
        second_before_facing: crate::game::battle::tile_range::FacingDirection,
        second_after_facing: crate::game::battle::tile_range::FacingDirection,
        cooldown_until_ms: u64,
    },
    IndependentObjectRemoved {
        receipt: crate::game::battle::core::ObjectRemovalReceipt,
    },
    IndependentObjectEvolved {
        receipt: crate::game::battle::core::ObjectEvolutionReceipt,
    },
    UnitWithdrawn {
        unit_instance_id: UnitInstanceId,
        world_position: EventLogVec2,
        position: Position,
    },
    MovementSegmentStarted {
        unit_instance_id: UnitInstanceId,
        start: EventLogVec2,
        target: EventLogVec2,
        started_at_ms: u64,
        ends_at_ms: u64,
        end_kind: MovementSegmentEndKind,
    },
    MovementStopped {
        unit_instance_id: UnitInstanceId,
        reason: MovementStopReason,
        world_position: EventLogVec2,
        stopped_at_ms: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        until_ms: Option<u64>,
    },
    AttackStart {
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<AttackKind>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delivery: Option<AttackDelivery>,
    },
    AttackResolve {
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<AttackKind>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delivery: Option<AttackDelivery>,
    },
    AttackMiss {
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<AttackKind>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delivery: Option<AttackDelivery>,
    },
    BasicAttackProjectileLaunched {
        projectile_id: Uuid,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        start: EventLogVec2,
        aim: EventLogVec2,
        fired_at_ms: u64,
        expected_impact_time_ms: u64,
    },
    BasicAttackProjectileImpacted {
        projectile_id: Uuid,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        impact_position: EventLogVec2,
        hit: bool,
    },
    AutoCastStart {
        caster_instance_id: UnitInstanceId,
        skill_id: Option<SkillId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<SkillCastTarget>,
    },
    AutoCastEnd {
        caster_instance_id: UnitInstanceId,
    },
    ManualCastStart {
        caster_instance_id: UnitInstanceId,
        skill_id: SkillId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<SkillCastTarget>,
    },
    ManualCastEnd {
        caster_instance_id: UnitInstanceId,
    },
    SkillCastInterrupted {
        interrupter_instance_id: UnitInstanceId,
        caster_instance_id: UnitInstanceId,
        interrupted_skill_id: SkillId,
        interrupted_cast_seq: u64,
    },
    SkillCastCancelled {
        caster_instance_id: UnitInstanceId,
        interrupted_skill_id: SkillId,
        interrupted_cast_seq: u64,
        reason: SkillCastCancelReason,
    },
    TriggeredAbilityProc {
        skill_id: SkillId,
        caster_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target_instance_id: Option<UnitInstanceId>,
        activation_source: CooldownSource,
        binding_index: usize,
    },
    AbilityCast {
        skill_id: SkillId,
        caster_instance_id: UnitInstanceId,
        target_instance_id: Option<UnitInstanceId>,
    },
    AbilityStepTriggered {
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        target_instance_id: Option<UnitInstanceId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        presentation: Option<SkillPresentationDef>,
    },
    SkillAreaDeclared {
        area_id: Uuid,
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<SkillCastTarget>,
        shape: BattleSkillAreaShape,
        /// Tile area events use `shape.affected_tiles` as their gameplay and
        /// presentation source of truth. `origin`, `center`, and
        /// `direction_hint` are retained for VFX anchoring.
        origin: EventLogVec2,
        center: EventLogVec2,
        direction_hint: EventLogVec2,
        start_time_ms: u64,
        duration_ms: u32,
        /// Presentation lifetime. Instant gameplay areas use a short non-zero
        /// value so debug visuals are visible without changing simulation rules.
        display_duration_ms: u32,
        #[serde(default)]
        warning_ms: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tick_interval_ms: Option<u32>,
        tick_policy: SkillAreaTickPolicy,
        #[serde(default)]
        tracking: SkillAreaTracking,
        hit_targets: SkillHitTargetFilter,
        include_caster: bool,
    },
    SkillProjectileLaunched {
        delivery_id: Uuid,
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<SkillCastTarget>,
        guidance: BattleProjectileGuidance,
        start: EventLogVec2,
        aim: EventLogVec2,
        fired_at_ms: u64,
        expected_end_time_ms: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        projectile_vfx_id: Option<String>,
    },
    SkillProjectileImpacted {
        delivery_id: Uuid,
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        first_hit_unit_id: Option<UnitInstanceId>,
        impact_position: EventLogVec2,
        terminal: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        impact_vfx_id: Option<String>,
    },
    BuffApplied {
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        buff_id: BuffId,
        duration_ms: u64,
    },
    BuffTick {
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        buff_id: BuffId,
    },
    BuffExpired {
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        buff_id: BuffId,
        reason: BuffExpireReason,
    },
    HpChanged {
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        delta: i32,
        hp_before: u32,
        hp_after: u32,
        reason: HpChangeReason,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        damage_source: Option<DamageSource>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        damage_type: Option<DamageType>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        raw_damage: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        final_damage: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        damage_breakdown: Option<Vec<DamageBreakdown>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        critical: Option<bool>,
        #[serde(default)]
        feedback_tags: Vec<DamageFeedbackTag>,
    },
    StatChanged {
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        modifier: StatModifier,
        stats_before: UnitStats,
        stats_after: UnitStats,
    },
    ResonanceChanged {
        unit_instance_id: UnitInstanceId,
        before: u32,
        after: u32,
        max: u32,
    },
    UnitDied {
        unit_instance_id: UnitInstanceId,
        owner: Side,
        killer_instance_id: Option<UnitInstanceId>,
        world_position: EventLogVec2,
        position: Position,
    },
    BattleEnd {
        winner: BattleWinner,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuffExpireReason {
    Natural,
    Replaced,
    TargetDied,
    CasterDied,
    TargetWithdrawn,
    CasterWithdrawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillCastCancelReason {
    Withdrawn,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HpChangeReason {
    Corrosion,
    HealthCost,
    BasicAttack,
    Command,
    Rest,
    FieldWork,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn end_entry(seq: u64, time_ms: u64) -> BattleEventLogEntry {
        BattleEventLogEntry {
            skill_effect_source: None,
            time_ms,
            seq,
            cause: BattleEventCause::default(),
            source_command_id: None,
            event: BattleLogEvent::BattleEnd {
                winner: BattleWinner::Draw,
            },
        }
    }

    #[test]
    fn native_log_cow_preserves_only_bounded_spare_and_original_entries() {
        let mut original = BattleEventLog::new();
        original.append(end_entry(1, 0));
        let mut candidate = original.clone();
        assert!(Arc::ptr_eq(&original.entries.0, &candidate.entries.0));
        candidate.append(end_entry(2, 1));
        assert_eq!(candidate.entries.capacity(), ORDINARY_HISTORY_RESERVE_LIMIT);
        assert_eq!(original.entries.len(), 1);
        assert_eq!(candidate.entries.len(), 2);
        let mut oversized = BattleEventEntries::with_capacity(ORDINARY_HISTORY_RESERVE_LIMIT * 8);
        oversized.push(end_entry(1, 0));
        let mut copied = oversized.clone();
        copied.push(end_entry(2, 1));
        assert_eq!(copied.capacity(), ORDINARY_HISTORY_RESERVE_LIMIT);
        assert_eq!(oversized.len(), 1);
        let mut growing = BattleEventEntries::with_capacity(ORDINARY_HISTORY_RESERVE_LIMIT * 2);
        growing.extend((1..=200).map(|seq| end_entry(seq, seq)));
        let mut grown_copy = growing.clone();
        grown_copy.push(end_entry(201, 201));
        assert_eq!(grown_copy.capacity(), ORDINARY_HISTORY_RESERVE_LIMIT * 2);
        assert_eq!(growing.len(), 200);
    }

    #[test]
    fn allocation_free_encoded_length_matches_compact_json_bytes() {
        let entry = end_entry(17, 23_456);
        assert_eq!(
            encoded_json_len(&entry),
            Some(serde_json::to_vec(&entry).unwrap().len())
        );
    }

    #[test]
    fn same_length_public_edit_invalidates_size_and_time_before_next_append() {
        let mut log =
            BattleEventLog::with_history_limits(crate::runtime_safety::BattleHistoryLimits {
                max_events: 4,
                max_encoded_entry_array_bytes: 1024,
                max_span_ms: 10,
            });
        log.append(end_entry(1, 20));
        log.append(end_entry(2, 21));
        let original = log.clone();
        log.entries[0].source_command_id = Some("x".repeat(2048));
        log.entries[0].time_ms = 0;
        assert_eq!(
            log.encoded_entry_array_bytes(),
            serde_json::to_vec(&log.entries).unwrap().len()
        );
        assert_eq!(log.retained_time_span_ms(), 21);
        log.append(end_entry(3, 22));
        assert_eq!(log.entries.len(), 2);
        assert_eq!(log.floor_seq(), 1);
        assert_eq!(log.retained_time_span_ms(), 1);
        assert!(log.encoded_entry_array_bytes() <= 1024);
        assert_eq!(original.entries.len(), 2);
        assert!(original.entries[0].source_command_id.is_none());
    }

    #[test]
    fn standard_serde_round_trip_matches_compact_event_log_json() {
        let event_log = BattleEventLog::from_entries(vec![BattleEventLogEntry {
            skill_effect_source: None,
            time_ms: 42,
            seq: 7,
            cause: BattleEventCause::Root {
                kind: BattleEventRootCause::System,
            },
            source_command_id: Some("command-42".to_string()),
            event: BattleLogEvent::BattleEnd {
                winner: BattleWinner::Draw,
            },
        }]);

        let compact = event_log.to_json_string().expect("compact event log json");
        assert_eq!(
            compact,
            serde_json::to_string(&event_log).expect("standard compact serialization")
        );

        let round_tripped: BattleEventLog =
            serde_json::from_str(&compact).expect("standard event log deserialization");
        assert_eq!(
            serde_json::to_value(round_tripped).expect("round-tripped event log value"),
            serde_json::to_value(event_log).expect("original event log value")
        );
    }

    #[test]
    fn clone_shares_entries_until_mutation_then_preserves_isolation() {
        let original = BattleEventLog::from_entries(vec![end_entry(1, 10), end_entry(2, 20)]);
        let original_json = original.to_json_string().expect("original event log json");
        let mut staged = original.clone();

        assert!(Arc::ptr_eq(&original.entries.0, &staged.entries.0));

        staged.append(end_entry(3, 30));

        assert!(!Arc::ptr_eq(&original.entries.0, &staged.entries.0));
        assert_eq!(
            original.to_json_string().expect("unchanged original json"),
            original_json
        );
        assert_eq!(
            original
                .entries
                .iter()
                .map(|entry| entry.seq)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(
            staged
                .entries
                .iter()
                .map(|entry| entry.seq)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(staged.floor_seq(), 0);
        assert_eq!(staged.latest_seq(), 3);
    }

    #[test]
    fn authoring_cow_admission_precedes_copy_and_preserves_retry_and_continuation() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let original = BattleEventLog::from_entries(vec![end_entry(1, 10), end_entry(2, 20)]);
        let original_json = original.to_json_string().unwrap();
        let mut ordinary = original.clone();
        ordinary.prepare_authoring_write().unwrap();
        assert!(Arc::ptr_eq(&original.entries.0, &ordinary.entries.0));

        let mut candidate = original.clone();
        let limits = ExecutionLimits {
            max_clone_slots: 2,
            ..ExecutionLimits::default()
        };
        let failure = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                assert!(candidate.prepare_authoring_write().is_err());
                assert!(Arc::ptr_eq(&original.entries.0, &candidate.entries.0));
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_clone_limit");
        assert_eq!(failure.requested, Some(3));
        let limits = ExecutionLimits {
            max_clone_slots: 3,
            ..ExecutionLimits::default()
        };
        let ((), usage) = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                candidate.prepare_authoring_write().unwrap();
                assert!(!Arc::ptr_eq(&original.entries.0, &candidate.entries.0));
                // A second preparation of the same private candidate is not a copy.
                candidate.prepare_authoring_write().unwrap();
            },
        )
        .unwrap();
        assert_eq!(usage.clone_slots_peak, 3);
        assert_eq!(usage.work, 6);
        ordinary.append(end_entry(3, 30));
        candidate.append(end_entry(3, 30));
        assert_eq!(
            candidate.to_json_string().unwrap(),
            ordinary.to_json_string().unwrap()
        );
        assert_eq!(candidate.floor_seq(), ordinary.floor_seq());
        assert_eq!(
            candidate.encoded_entry_array_bytes(),
            ordinary.encoded_entry_array_bytes()
        );
        assert_eq!(original.to_json_string().unwrap(), original_json);
    }

    #[test]
    fn authoring_cow_prices_nested_storage_strings_and_shared_source_handles() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        use crate::game::ability::authoring::{ActionSite, EffectOrigin, EffectSite, EffectSource};
        use crate::game::battle::skill_source::{
            SkillCastExecutionSource, SkillEffectExecutionSource,
        };
        let source = Arc::new(SkillEffectExecutionSource {
            authored: Arc::new(EffectSource {
                source_file: "source".repeat(4096),
                site: EffectSite {
                    action: ActionSite {
                        definition_id: "skill".into(),
                        action_id: "action".into(),
                    },
                    effect_id: "effect".into(),
                },
                field_path: "effects[0]".into(),
                origin_definition_id: "skill".into(),
                origin: EffectOrigin::Original,
            }),
            cast: Arc::new(SkillCastExecutionSource {
                skill_id: "skill".into(),
                caster_instance_id: Uuid::nil().into(),
                cast_seq: 1,
                ability_event_seq: 1,
                cause: BattleEventCause::default(),
                proc_origin: None,
            }),
        });
        let mut entry = end_entry(1, 10);
        entry.skill_effect_source = Some(source.clone());
        entry.source_command_id = Some("command".into());
        entry.event = BattleLogEvent::HpChanged {
            source_instance_id: None,
            target_instance_id: Uuid::nil().into(),
            delta: -1,
            hp_before: 2,
            hp_after: 1,
            reason: HpChangeReason::Command,
            damage_source: None,
            damage_type: None,
            raw_damage: None,
            final_damage: None,
            damage_breakdown: Some(vec![
                DamageBreakdown {
                    damage_type: DamageType::True,
                    raw_damage: 1,
                    final_damage: 1,
                    critical: false
                };
                3
            ]),
            critical: None,
            feedback_tags: vec![DamageFeedbackTag::FixedDamage; 2],
        };
        let original = BattleEventLog::from_entries(vec![entry]);
        // One Arc<Vec>, one entry, five nested elements, one source Arc.
        let mut candidate = original.clone();
        let limits = ExecutionLimits {
            max_clone_slots: 7,
            ..ExecutionLimits::default()
        };
        let failure = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || candidate.prepare_authoring_write(),
        )
        .unwrap_err();
        assert_eq!(failure.requested, Some(8));
        assert!(Arc::ptr_eq(&original.entries.0, &candidate.entries.0));

        let limits = ExecutionLimits {
            max_clone_slots: 8,
            max_work: 16,
            ..ExecutionLimits::default()
        };
        let failure = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || candidate.prepare_authoring_write(),
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_work_limit");
        assert_eq!(failure.requested, Some(17));
        assert!(Arc::ptr_eq(&original.entries.0, &candidate.entries.0));
        let limits = ExecutionLimits {
            max_work: 17,
            ..limits
        };
        let (result, usage) = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || candidate.prepare_authoring_write(),
        )
        .unwrap();
        result.unwrap();
        assert_eq!(usage.work, 17);
        assert!(Arc::ptr_eq(
            candidate.entries[0].skill_effect_source.as_ref().unwrap(),
            &source
        ));
        assert_eq!(
            candidate.to_json_string().unwrap(),
            original.to_json_string().unwrap()
        );
    }

    #[test]
    fn authoring_cow_cancellation_keeps_original_and_rejects_prepared_candidate() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let original = BattleEventLog::from_entries(vec![end_entry(1, 10)]);
        let mut candidate = original.clone();
        let cancellation = CancellationToken::default();
        let failure = control::run(
            &ExecutionLimits::default(),
            &cancellation,
            ExecutionPhase::TrialExecution,
            || {
                cancellation.cancel();
                assert!(candidate.prepare_authoring_write().is_err());
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_cancelled");
        assert!(Arc::ptr_eq(&original.entries.0, &candidate.entries.0));
        let cancellation = CancellationToken::default();
        let failure = control::run(
            &ExecutionLimits::default(),
            &cancellation,
            ExecutionPhase::TrialExecution,
            || {
                let mut candidate = original.clone();
                candidate.prepare_authoring_write().unwrap();
                candidate.append(end_entry(2, 20));
                cancellation.cancel();
                candidate
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_cancelled");
        assert_eq!(original.entries.len(), 1);
        assert_eq!(original.latest_seq(), 1);
    }

    #[test]
    fn count_cap_keeps_exact_window_and_requires_resync_before_floor() {
        let mut log = BattleEventLog::new();
        for seq in 1..=BATTLE_HISTORY_MAX_EVENTS as u64 + 1 {
            log.append(end_entry(seq, 0));
        }

        assert_eq!(log.entries.len(), BATTLE_HISTORY_MAX_EVENTS);
        assert_eq!(log.floor_seq(), 1);
        assert_eq!(log.latest_seq(), BATTLE_HISTORY_MAX_EVENTS as u64 + 1);
        assert_eq!(log.entries.first().map(|entry| entry.seq), Some(2));
        assert!(matches!(
            log.validate_cursor(0),
            Err(BattleHistoryCursorError::SnapshotResyncRequired {
                requested: 0,
                oldest_available_after_seq: 1,
                latest,
            }) if latest == BATTLE_HISTORY_MAX_EVENTS as u64 + 1
        ));
        assert_eq!(
            log.entries_after_seq(1)
                .iter()
                .map(|entry| entry.seq)
                .collect::<Vec<_>>(),
            (2..=BATTLE_HISTORY_MAX_EVENTS as u64 + 1).collect::<Vec<_>>()
        );
        let first = log
            .entries_after_seq(1)
            .into_iter()
            .map(|entry| entry.seq)
            .collect::<Vec<_>>();
        let retry = log
            .entries_after_seq(1)
            .into_iter()
            .map(|entry| entry.seq)
            .collect::<Vec<_>>();
        assert_eq!(first, retry);
    }

    #[test]
    fn encoded_byte_cap_allows_exact_limit_and_drops_oversized_diagnostics() {
        let mut exact = BattleEventLog::new();
        exact.append_with_encoded_len_for_test(
            end_entry(1, 0),
            Some(BATTLE_HISTORY_MAX_ENCODED_BYTES - 2),
        );
        assert_eq!(
            exact.encoded_entry_array_bytes(),
            BATTLE_HISTORY_MAX_ENCODED_BYTES
        );
        assert_eq!(exact.entries.len(), 1);

        let mut oversized = BattleEventLog::new();
        oversized.append_with_encoded_len_for_test(
            end_entry(1, 0),
            Some(BATTLE_HISTORY_MAX_ENCODED_BYTES - 1),
        );
        assert!(oversized.entries.is_empty());
        assert_eq!(oversized.floor_seq(), 1);
        assert_eq!(oversized.latest_seq(), 1);
        assert!(matches!(
            oversized.validate_cursor(0),
            Err(BattleHistoryCursorError::SnapshotResyncRequired { .. })
        ));

        oversized.append_with_encoded_len_for_test(end_entry(2, 1), None);
        assert!(oversized.entries.is_empty());
        assert_eq!(oversized.floor_seq(), 2);
        assert_eq!(oversized.latest_seq(), 2);
    }

    #[test]
    fn battle_time_cap_is_a_moving_window_not_a_timeout() {
        let mut log = BattleEventLog::new();
        log.append(end_entry(1, 0));
        log.append(end_entry(2, BATTLE_HISTORY_MAX_SPAN_MS));
        assert_eq!(log.entries.len(), 2);
        assert_eq!(log.retained_time_span_ms(), BATTLE_HISTORY_MAX_SPAN_MS);

        log.append(end_entry(3, BATTLE_HISTORY_MAX_SPAN_MS + 1));
        assert_eq!(log.floor_seq(), 1);
        assert_eq!(log.latest_seq(), 3);
        assert_eq!(
            log.entries
                .iter()
                .map(|entry| entry.seq)
                .collect::<Vec<_>>(),
            vec![2, 3]
        );
        assert_eq!(log.retained_time_span_ms(), 1);
        assert!(log.validate_cursor(1).is_ok());
        assert!(matches!(
            log.validate_cursor(4),
            Err(BattleHistoryCursorError::InvalidFuture {
                requested: 4,
                latest: 3
            })
        ));
    }

    #[test]
    fn hp_changed_serializes_feedback_tags_as_snake_case_array() {
        let source_instance_id = UnitInstanceId::from(Uuid::from_u128(0xA));
        let target_instance_id = UnitInstanceId::from(Uuid::from_u128(0xB));
        let event = BattleLogEvent::HpChanged {
            source_instance_id: Some(source_instance_id),
            target_instance_id,
            delta: -10,
            hp_before: 100,
            hp_after: 90,
            reason: HpChangeReason::Command,
            damage_source: Some(DamageSource::Ability),
            damage_type: Some(DamageType::Physical),
            raw_damage: Some(20),
            final_damage: Some(10),
            damage_breakdown: None,
            critical: Some(true),
            feedback_tags: vec![DamageFeedbackTag::Critical, DamageFeedbackTag::Mitigated],
        };

        let value = serde_json::to_value(event).expect("serialize HpChanged");

        assert_eq!(value["type"], "HpChanged");
        assert_eq!(
            value["feedback_tags"],
            serde_json::json!(["critical", "mitigated"])
        );
    }

    #[test]
    fn movement_stopped_serializes_explicit_stopped_at_ms() {
        let unit_instance_id = UnitInstanceId::from(Uuid::from_u128(0xA));
        let event = BattleLogEvent::MovementStopped {
            unit_instance_id,
            reason: MovementStopReason::CastStarted,
            world_position: EventLogVec2 {
                x_milli: 1_250,
                y_milli: 2_500,
            },
            stopped_at_ms: 1_234,
            until_ms: Some(2_000),
        };

        let value = serde_json::to_value(event).expect("serialize MovementStopped");

        assert_eq!(value["type"], "MovementStopped");
        assert_eq!(value["unit_instance_id"], unit_instance_id.to_string());
        assert_eq!(value["stopped_at_ms"], 1_234);
        assert_eq!(value["until_ms"], 2_000);
    }
}
