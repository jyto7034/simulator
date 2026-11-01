//! Optional encoded observations. These limits and the attempt ledger never
//! participate in execution control, mandatory receipts, or binding inputs.
use std::{cell::Cell, io, sync::Arc};

use serde::Serialize;

use super::{clone_shape::CloneShape, control::CloneMeter};
use crate::game::{battle::core::authoring_observation::ApplicationWitness, behavior::GameError};

#[derive(Debug, Clone, Copy)]
pub struct TraceLimits {
    pub max_rows: usize,
    pub max_bytes: usize,
    pub max_row_bytes: usize,
    pub max_attempt_work: u64,
}

impl Default for TraceLimits {
    fn default() -> Self {
        Self {
            max_rows: 1024,
            max_bytes: 1024 * 1024,
            max_row_bytes: 16 * 1024,
            max_attempt_work: 16 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TraceOptions {
    pub limits: Option<TraceLimits>,
}
super::clone_shape::copy_shape!(TraceOptions);

impl TraceOptions {
    pub(crate) fn is_valid(self) -> bool {
        self.limits.is_none_or(|limits| {
            (1..=8192).contains(&limits.max_rows)
                && (1..=8 * 1024 * 1024).contains(&limits.max_bytes)
                && (1..=256 * 1024).contains(&limits.max_row_bytes)
                && limits.max_row_bytes <= limits.max_bytes
                && (1..=1 << 30).contains(&limits.max_attempt_work)
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub(crate) struct TraceUsage {
    pub work: u64,
    pub exhausted: bool,
}

#[derive(Clone, Copy)]
struct Budget {
    limit: u64,
    usage: TraceUsage,
}
thread_local! {
    static ATTEMPT: Cell<Option<Budget>> = const { Cell::new(None) };
}

/// Installed once outside a whole preparation/step control scope. Internal
/// gameplay rollback cannot replenish it. No data/source owner lives in TLS.
pub(crate) struct TraceAttempt(std::marker::PhantomData<std::rc::Rc<()>>);
impl TraceAttempt {
    pub(crate) fn enter(options: TraceOptions) -> Result<Self, ()> {
        if !options.is_valid() {
            return Err(());
        }
        ATTEMPT.with(|slot| {
            if slot.get().is_some() {
                return Err(());
            }
            slot.set(Some(Budget {
                limit: options.limits.map_or(0, |limits| limits.max_attempt_work),
                usage: TraceUsage::default(),
            }));
            Ok(Self(std::marker::PhantomData))
        })
    }

    pub(crate) fn usage(&self) -> TraceUsage {
        ATTEMPT.with(|slot| slot.get().expect("live optional attempt").usage)
    }
}
impl Drop for TraceAttempt {
    fn drop(&mut self) {
        ATTEMPT.with(|slot| slot.set(None));
    }
}

fn charge(amount: Option<u64>) -> Result<(), TraceGapReason> {
    ATTEMPT.with(|slot| {
        let Some(mut budget) = slot.get() else {
            return Err(TraceGapReason::MissingAttempt);
        };
        let next = amount.and_then(|amount| budget.usage.work.checked_add(amount));
        if budget.usage.exhausted || next.is_none_or(|next| next > budget.limit) {
            budget.usage.exhausted = true;
            slot.set(Some(budget));
            return Err(TraceGapReason::WorkLimit);
        }
        budget.usage.work = next.expect("checked optional work");
        slot.set(Some(budget));
        Ok(())
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceGapReason {
    RowLimit,
    ByteLimit,
    RowByteLimit,
    WorkLimit,
    MissingAttempt,
    Allocation,
    Encoding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TraceGap {
    pub reason: TraceGapReason,
    pub first_ordinal: u64,
    pub dropped_rows: u64,
    pub counter_overflow: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct OptionalTrace {
    options: TraceOptions,
    bytes: Option<Arc<Vec<u8>>>,
    retained_rows: u64,
    last_retained_ordinal: Option<u64>,
    gap: Option<TraceGap>,
}

impl CloneShape for OptionalTrace {
    fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
        let Self {
            options,
            bytes,
            retained_rows,
            last_retained_ordinal,
            gap,
        } = self;
        // Fixed reservation for the record, Copy fields and optional Arc handle.
        // Changing a field's ownership type requires revisiting this proof.
        let _: (
            &TraceOptions,
            &Option<Arc<Vec<u8>>>,
            &u64,
            &Option<u64>,
            &Option<TraceGap>,
        ) = (options, bytes, retained_rows, last_retained_ordinal, gap);
        meter.scan_storage(Some(12))
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TraceView<'a> {
    pub enabled: bool,
    /// Complete JSON rows, each followed by exactly one newline. No partial row.
    pub ndjson: &'a [u8],
    pub retained_rows: u64,
    pub last_retained_ordinal: Option<u64>,
    pub gap: Option<TraceGap>,
}

impl OptionalTrace {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let storage = match self.bytes.as_ref() {
            Some(bytes) => bytes.capacity().checked_add(64)?,
            None => 0,
        };
        std::mem::size_of::<Self>().checked_add(storage)
    }

    pub(crate) fn new(options: TraceOptions) -> Self {
        Self {
            options,
            ..Self::default()
        }
    }

    pub(crate) fn options(&self) -> TraceOptions {
        self.options
    }

    pub(crate) fn view(&self) -> TraceView<'_> {
        TraceView {
            enabled: self.options.limits.is_some(),
            ndjson: self.bytes.as_deref().map_or(&[], Vec::as_slice),
            retained_rows: self.retained_rows,
            last_retained_ordinal: self.last_retained_ordinal,
            gap: self.gap,
        }
    }

    fn drop_row(&mut self, ordinal: u64, reason: TraceGapReason) {
        if let Some(gap) = self.gap.as_mut() {
            if let Some(next) = gap.dropped_rows.checked_add(1) {
                gap.dropped_rows = next;
            } else {
                gap.counter_overflow = true;
            }
        } else {
            self.gap = Some(TraceGap {
                reason,
                first_ordinal: ordinal,
                dropped_rows: 1,
                counter_overflow: false,
            });
        }
    }

    pub(crate) fn record(&mut self, witness: &ApplicationWitness) {
        let Some(limits) = self.options.limits else {
            return;
        };
        if let Some(gap) = self.gap {
            self.drop_row(witness.ordinal, gap.reason);
            return;
        }
        if self.retained_rows >= limits.max_rows as u64 {
            self.drop_row(witness.ordinal, TraceGapReason::RowLimit);
            return;
        }
        // serde_json scans a string before emitting its final write fragment.
        // The closed witness graph must be admitted before that scan starts.
        if let Err(reason) = charge(witness.trace_scan_work()) {
            self.drop_row(witness.ordinal, reason);
            return;
        }
        let start = self.bytes.as_ref().map_or(0, |bytes| bytes.len());
        let mut writer = RowWriter {
            trace: self,
            limits,
            start,
            failure: None,
        };
        let result = serde_json::to_writer(&mut writer, witness)
            .map_err(|_| ())
            .and_then(|()| io::Write::write_all(&mut writer, b"\n").map_err(|_| ()));
        if result.is_err() {
            let reason = writer.failure.unwrap_or(TraceGapReason::Encoding);
            // No shared buffer was mutated. A partial row, if any, owns a unique
            // candidate buffer; shrinking it needs no allocation or detach.
            if let Some(bytes) = writer.trace.bytes.as_mut().and_then(Arc::get_mut) {
                bytes.truncate(start);
            }
            writer.trace.drop_row(witness.ordinal, reason);
            return;
        }
        writer.trace.retained_rows += 1; // Bounded above by validated max_rows.
        writer.trace.last_retained_ordinal = Some(witness.ordinal);
    }
}

struct RowWriter<'a> {
    trace: &'a mut OptionalTrace,
    limits: TraceLimits,
    start: usize,
    failure: Option<TraceGapReason>,
}

impl RowWriter<'_> {
    fn append(&mut self, bytes: &[u8]) -> Result<(), TraceGapReason> {
        let len = self.trace.bytes.as_ref().map_or(0, |buffer| buffer.len());
        let next = len
            .checked_add(bytes.len())
            .ok_or(TraceGapReason::ByteLimit)?;
        if next > self.limits.max_bytes {
            return Err(TraceGapReason::ByteLimit);
        }
        if next - self.start > self.limits.max_row_bytes {
            return Err(TraceGapReason::RowByteLimit);
        }
        let must_copy = self.trace.bytes.as_mut().is_none_or(|buffer| {
            Arc::get_mut(buffer).is_none_or(|buffer| buffer.capacity() < next)
        });
        if must_copy {
            let capacity = next
                .max(len.saturating_mul(2))
                .max(64)
                .min(self.limits.max_bytes);
            // Allocate a new buffer explicitly: charge its capacity, the live
            // prefix copy and the new bytes before any of these operations.
            charge(
                (capacity as u64)
                    .checked_add(len as u64)
                    .and_then(|cost| cost.checked_add(bytes.len() as u64))
                    .and_then(|cost| cost.checked_add(1)),
            )?;
            let mut buffer = Vec::new();
            buffer
                .try_reserve_exact(capacity)
                .map_err(|_| TraceGapReason::Allocation)?;
            if let Some(previous) = self.trace.bytes.as_ref() {
                buffer.extend_from_slice(previous);
            }
            buffer.extend_from_slice(bytes);
            self.trace.bytes = Some(Arc::new(buffer));
        } else {
            charge(Some(bytes.len() as u64))?;
            Arc::get_mut(self.trace.bytes.as_mut().expect("existing buffer"))
                .expect("unique admitted buffer")
                .extend_from_slice(bytes);
        }
        Ok(())
    }
}

impl io::Write for RowWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self.append(bytes) {
            Ok(()) => Ok(bytes.len()),
            Err(reason) => {
                self.failure.get_or_insert(reason);
                Err(io::Error::other("optional trace stopped"))
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::battle::{
        core::authoring_observation::AppliedChange, event_log::HpChangeReason,
    };

    fn witness(ordinal: u64) -> ApplicationWitness {
        ApplicationWitness {
            fragment: None,
            ordinal,
            time_ms: 17,
            source_unit: None,
            target_unit: uuid::Uuid::from_u128(19).into(),
            change: AppliedChange::Hp {
                before: 100,
                after: 83,
                reason: HpChangeReason::Command,
            },
            source: None,
            authored: None,
            area_source: None,
            equipment: None,
        }
    }
    fn enabled(limits: TraceLimits) -> TraceOptions {
        TraceOptions {
            limits: Some(limits),
        }
    }

    #[test]
    fn optional_trace_exact_caps_partial_row_and_clone_gap_preserve_complete_prefix() {
        let first = witness(1);
        let mut encoded = serde_json::to_vec(&first).unwrap();
        encoded.push(b'\n');
        let options = enabled(TraceLimits {
            max_bytes: encoded.len() * 2,
            max_row_bytes: encoded.len(),
            ..Default::default()
        });
        let attempt = TraceAttempt::enter(options).unwrap();
        let mut trace = OptionalTrace::new(options);
        trace.record(&first);
        assert_eq!(trace.view().ndjson, encoded);
        let original = trace.clone();
        assert!(Arc::ptr_eq(
            trace.bytes.as_ref().unwrap(),
            original.bytes.as_ref().unwrap()
        ));
        trace.record(&witness(2));
        assert_eq!(trace.view().retained_rows, 2);
        assert_eq!(trace.view().ndjson.len(), encoded.len() * 2);
        assert_eq!(original.view().ndjson, encoded);
        let before = trace.view().ndjson.to_vec();
        trace.record(&witness(3));
        trace.record(&witness(4));
        assert_eq!(trace.view().ndjson, before);
        assert_eq!(
            trace.view().gap.unwrap(),
            TraceGap {
                reason: TraceGapReason::ByteLimit,
                first_ordinal: 3,
                dropped_rows: 2,
                counter_overflow: false,
            }
        );
        assert!(original.view().gap.is_none());
        assert!(attempt.usage().work > encoded.len() as u64);
        drop(attempt);

        // Fail on the final newline after a complete JSON value was written.
        for row_limit in [true, false] {
            let options = enabled(TraceLimits {
                max_bytes: if row_limit {
                    encoded.len() * 2
                } else {
                    encoded.len() - 1
                },
                max_row_bytes: encoded.len() - 1,
                ..Default::default()
            });
            let _attempt = TraceAttempt::enter(options).unwrap();
            let mut trace = OptionalTrace::new(options);
            trace.record(&first);
            assert!(trace.view().ndjson.is_empty());
            assert_eq!(trace.view().retained_rows, 0);
            assert_eq!(trace.view().last_retained_ordinal, None);
            assert_eq!(
                trace.view().gap.unwrap().reason,
                if row_limit {
                    TraceGapReason::RowByteLimit
                } else {
                    TraceGapReason::ByteLimit
                }
            );
        }
        let options = enabled(TraceLimits {
            max_rows: 1,
            ..Default::default()
        });
        let _attempt = TraceAttempt::enter(options).unwrap();
        let mut trace = OptionalTrace::new(options);
        trace.record(&first);
        let prefix = trace.view().ndjson.to_vec();
        trace.record(&witness(2));
        assert_eq!(trace.view().gap.unwrap().reason, TraceGapReason::RowLimit);
        assert_eq!(trace.view().ndjson, prefix);
    }

    #[test]
    fn optional_trace_attempt_work_is_not_restored_by_candidate_drop() {
        let first = witness(1);
        let options = enabled(TraceLimits::default());
        let measured = {
            let attempt = TraceAttempt::enter(options).unwrap();
            let mut trace = OptionalTrace::new(options);
            trace.record(&first);
            assert!(trace.view().gap.is_none());
            attempt.usage().work
        };
        for (work, succeeds) in [(measured, true), (measured - 1, false)] {
            let options = enabled(TraceLimits {
                max_attempt_work: work,
                ..Default::default()
            });
            let attempt = TraceAttempt::enter(options).unwrap();
            assert!(TraceAttempt::enter(options).is_err());
            let original = OptionalTrace::new(options);
            let mut candidate = original.clone();
            candidate.record(&first);
            assert_eq!(candidate.view().gap.is_none(), succeeds);
            if !succeeds {
                assert!(candidate.view().ndjson.is_empty());
            }
            drop(candidate);
            let consumed = attempt.usage().work;
            assert!(consumed > 0);
            let mut retry = original.clone();
            retry.record(&first);
            assert_eq!(retry.view().gap.unwrap().reason, TraceGapReason::WorkLimit);
            assert!(original.view().ndjson.is_empty());
            assert!(original.view().gap.is_none());
            assert!(attempt.usage().exhausted);
        }
        let mut missing = OptionalTrace::new(options);
        missing.record(&first);
        assert_eq!(
            missing.view().gap.unwrap().reason,
            TraceGapReason::MissingAttempt
        );
        assert!(missing.view().ndjson.is_empty());
        let invalid = enabled(TraceLimits {
            max_bytes: usize::MAX,
            ..Default::default()
        });
        assert!(TraceAttempt::enter(invalid).is_err());
        // The prior failures/drops left no stale TLS owner.
        assert!(TraceAttempt::enter(options).is_ok());
    }

    #[test]
    fn optional_trace_preflights_source_scan_and_keeps_escaped_utf8_rows() {
        use crate::game::{
            ability::authoring::{ActionSite, EffectOrigin, EffectSite, EffectSource},
            battle::skill_source::{SkillCastExecutionSource, SkillEffectExecutionSource},
        };
        let mut row = witness(1);
        let text = "한글\n\"\\".repeat(128);
        row.source = Some(Arc::new(SkillEffectExecutionSource {
            authored: Arc::new(EffectSource {
                source_file: text.clone(),
                site: EffectSite {
                    action: ActionSite {
                        definition_id: "definition".into(),
                        action_id: "action".into(),
                    },
                    effect_id: "effect".into(),
                },
                field_path: "field".into(),
                origin_definition_id: "origin".into(),
                origin: EffectOrigin::Added {
                    source_definition_id: "added".into(),
                },
            }),
            cast: Arc::new(SkillCastExecutionSource {
                skill_id: "cast".into(),
                caster_instance_id: row.target_unit,
                cast_seq: 3,
                ability_event_seq: 5,
                cause: Default::default(),
                proc_origin: None,
            }),
        }));
        let scan = 256
            + text.len() as u64
            + [
                "definition",
                "action",
                "effect",
                "field",
                "origin",
                "added",
                "cast",
            ]
            .iter()
            .map(|s| s.len() as u64)
            .sum::<u64>();
        assert_eq!(row.trace_scan_work(), Some(scan));
        {
            let options = enabled(TraceLimits {
                max_attempt_work: scan - 1,
                ..Default::default()
            });
            let attempt = TraceAttempt::enter(options).unwrap();
            let mut trace = OptionalTrace::new(options);
            trace.record(&row);
            assert_eq!(trace.view().gap.unwrap().reason, TraceGapReason::WorkLimit);
            assert!(trace.bytes.is_none());
            assert_eq!(attempt.usage().work, 0);
        }
        let options = enabled(TraceLimits::default());
        let _attempt = TraceAttempt::enter(options).unwrap();
        let mut trace = OptionalTrace::new(options);
        trace.record(&row);
        assert!(trace.view().gap.is_none());
        assert_eq!(
            trace.view().ndjson.iter().filter(|b| **b == b'\n').count(),
            1
        );
        let parsed: serde_json::Value = serde_json::from_slice(trace.view().ndjson).unwrap();
        assert_eq!(parsed["source"]["authored"]["source_file"], text);
        assert_eq!(parsed, serde_json::to_value(row).unwrap());
    }

    #[test]
    fn composition_area_effect_source_trace_bounds_and_utf8() {
        let mut row = witness(1);
        row.area_source = Some(Arc::new(crate::authoring::view::AreaEffectSource {
            generation: None, captured_cast: None, area_id: uuid::Uuid::from_u128(1), skill_id: "area-skill".into(),
            step_id: "장판\n\"".repeat(128), cast_sequence: 3, status_index: 1,
            definition: "생성한 취약성".into(), modifier_source: Some(crate::game::ability::sustained::HitModifierSource {
                copy_uuid: uuid::Uuid::from_u128(2), definition: "broken-balance".into(), rule: "성공한 기절".into(),
            }),
        }));
        {
            let options = enabled(TraceLimits { max_attempt_work: 512, ..Default::default() });
            let _attempt = TraceAttempt::enter(options).unwrap();
            let mut trace = OptionalTrace::new(options); trace.record(&row);
            assert_eq!(trace.view().gap.unwrap().reason, TraceGapReason::WorkLimit);
            assert!(trace.view().ndjson.is_empty());
        }
        let options = enabled(TraceLimits::default());
        let _attempt = TraceAttempt::enter(options).unwrap();
        let mut trace = OptionalTrace::new(options); trace.record(&row);
        assert!(trace.view().gap.is_none());
        let saved: serde_json::Value = serde_json::from_slice(trace.view().ndjson).unwrap();
        assert_eq!(saved["area_source"]["definition"], "생성한 취약성");
        assert_eq!(saved["area_source"]["step_id"], "장판\n\"".repeat(128));
        assert!(saved["source"].is_null() && saved.get("authored").is_none());
    }
}
