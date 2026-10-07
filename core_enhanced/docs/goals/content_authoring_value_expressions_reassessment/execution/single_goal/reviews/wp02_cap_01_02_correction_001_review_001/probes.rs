use super::*;
use crate::game::battle::runtime_control::{self as native, RuntimeFactLimits};

#[test]
fn review_cap_growth_matches_exhaustive_small_budget_oracle() {
    for len in 0..20_usize {
        for max_elements in [len + 1, len + 3, 40] {
            for bytes in 1..512_usize {
                for work in [1_u64, 10, 32, 100] {
                    let min = len + 1;
                    let preferred = (len + (len / 2).max(1)).max(2).max(min).min(max_elements);
                    let fits = |n: usize| n * 7 + 64 <= bytes && (n * 7 + 64).div_ceil(8) as u64 <= work;
                    let expected = (min..=preferred).rev().find(|n| fits(*n));
                    let result = native::run(RuntimeFactLimits { max_retained_bytes: bytes,
                        max_work: work, ..Default::default() }, || {
                        native::seed_retained_extent(0)?;
                        native::admitted_buffer_capacity(len, len, max_elements, 7, 64, "review_oracle")
                    });
                    assert_eq!(result.ok(), expected, "len={len},cap={max_elements},bytes={bytes},work={work}");
                }
            }
        }
    }
}

#[test]
fn review_cap_bucket_honors_container_cap_and_one_byte_short_refuses_before_reserve() {
    use crate::authoring::{CancellationToken, ExecutionLimits, ExecutionPhase};
    let mut bucket = Vec::new();
    control::run(&ExecutionLimits { max_bucket_events: 1, ..Default::default() },
        &CancellationToken::default(), ExecutionPhase::TrialExecution, || {
            reserve_timestamp_bucket_slot(&mut bucket, "review_bucket")
        }).unwrap().0.unwrap();
    assert_eq!(bucket.capacity(), 1);
    let mut bucket = Vec::new();
    let bytes = std::mem::size_of::<QueuedEvent>() + 64;
    let result = native::run(RuntimeFactLimits { max_retained_bytes: bytes - 1,
        ..Default::default() }, || reserve_timestamp_bucket_slot(&mut bucket, "review_bucket"));
    assert!(result.is_err());
    assert_eq!(bucket.capacity(), 0);
    native::run(RuntimeFactLimits { max_retained_bytes: bytes,
        ..Default::default() }, || reserve_timestamp_bucket_slot(&mut bucket, "review_bucket")).unwrap();
    assert_eq!(bucket.capacity(), 1);
}

#[test]
fn review_cap_equal_key_metadata_order_matches_old_exact_growth() {
    let mut current = EventQueue::new();
    let mut old = BinaryHeap::new();
    for i in 0..300 {
        let time_ms = (i % 7) as u64;
        let source = RuntimeFactSource { actor: Some(uuid::Uuid::from_u128(i + 1).into()),
            ..Default::default() };
        current.push_with_context(BattleEvent::ContinuousMovementTick { time_ms }, None, source.clone());
        if old.len() == old.capacity() { old.reserve_exact(1); }
        old.push(QueuedEvent { close_delivery_on_dispatch: false,
            event: PackedEvent::MovementTick { time_ms }, source: None,
            fact_source: QueuedFactSource::new(source).unwrap() });
    }
    while let Some(expected) = old.pop() {
        let actual = current.pop().unwrap();
        assert_eq!(actual.event.time_ms(), expected.event.time_ms());
        assert_eq!(actual.fact_source.actor, expected.fact_source.actor);
    }
    assert!(current.pop().is_none());
    let bytes = current.slot_extent().unwrap();
    assert!(bytes > 0);
    current.clear();
    assert_eq!(current.slot_extent().unwrap(), bytes);
    assert_eq!(current.clone().slot_extent().unwrap(), 0);
}
