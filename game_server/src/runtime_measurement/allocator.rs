//! Continuous process requested-byte accounting, including allocations before a
//! measurement window. Hooks use only the underlying allocator and atomics.
use serde::Serialize;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct HeapSnapshot {
    pub instrumented: bool,
    pub live_requested_bytes: Option<u64>,
    pub lifetime_peak_requested_bytes: Option<u64>,
    pub successful_allocations: u64,
    pub successful_zeroed_allocations: u64,
    pub successful_reallocations: u64,
    pub failed_allocations: u64,
    pub failed_reallocations: u64,
    pub deallocations: u64,
    pub counter_overflowed: bool,
    /// Individual atomics are sampled; concurrent activity is not paused.
    pub concurrent_snapshot: bool,
}
#[derive(Debug, Clone, Copy, Serialize)]
pub struct HeapWindow {
    pub baseline: HeapSnapshot,
    pub end: HeapSnapshot,
    pub peak_requested_bytes: Option<u64>,
    pub live_delta_bytes: Option<i128>,
    pub retiring_hooks_at_end: usize,
    pub boundary_updates_unavailable: bool,
    pub scope: &'static str,
}

/// A bounded read of an already-running window; reading never opens a window.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ActiveHeapWindow {
    pub epoch: u64,
    pub baseline_live_requested_bytes: Option<u64>,
    pub current: HeapSnapshot,
    pub peak_requested_bytes: Option<u64>,
    pub live_delta_bytes: Option<i128>,
    pub inflight_hooks: usize,
    pub boundary_updates_unavailable: bool,
    pub concurrent_snapshot: bool,
}

pub struct AllocatorLedger {
    live: AtomicUsize,
    peak: AtomicUsize,
    allocations: AtomicU64,
    zeroed: AtomicU64,
    reallocations: AtomicU64,
    failed_allocations: AtomicU64,
    failed_reallocations: AtomicU64,
    deallocations: AtomicU64,
    observed: AtomicBool,
    overflow: AtomicBool,
    window_reserved: AtomicBool,
    window_epoch: AtomicU64,
    window_inflight: AtomicUsize,
    window_peak: AtomicUsize,
    window_baseline: AtomicU64,
    window_baseline_valid: AtomicBool,
    window_boundary_uncertain: AtomicBool,
    hook_sequence: AtomicU64,
}
impl Default for AllocatorLedger {
    fn default() -> Self {
        Self::new()
    }
}
impl AllocatorLedger {
    pub const fn new() -> Self {
        Self {
            live: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
            allocations: AtomicU64::new(0),
            zeroed: AtomicU64::new(0),
            reallocations: AtomicU64::new(0),
            failed_allocations: AtomicU64::new(0),
            failed_reallocations: AtomicU64::new(0),
            deallocations: AtomicU64::new(0),
            observed: AtomicBool::new(false),
            overflow: AtomicBool::new(false),
            window_reserved: AtomicBool::new(false),
            window_epoch: AtomicU64::new(0),
            window_inflight: AtomicUsize::new(0),
            window_peak: AtomicUsize::new(0),
            window_baseline: AtomicU64::new(0),
            window_baseline_valid: AtomicBool::new(false),
            window_boundary_uncertain: AtomicBool::new(false),
            hook_sequence: AtomicU64::new(0),
        }
    }
    fn increment(&self, counter: &AtomicU64) {
        if counter
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .is_err()
        {
            self.overflow.store(true, Ordering::Relaxed);
        }
    }
    fn change_live(&self, old: usize, new: usize) {
        self.change_live_with(old, new, || {});
    }
    fn change_live_with(&self, old: usize, new: usize, after_live: impl FnOnce()) {
        // Registration spans the live update and both peak publications. A
        // stalled update can never appear quiescent at a closing boundary.
        self.window_inflight.fetch_add(1, Ordering::SeqCst);
        if self
            .hook_sequence
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_add(1))
            .is_err()
        {
            self.overflow.store(true, Ordering::Relaxed);
        }
        let epoch = self.window_epoch.load(Ordering::SeqCst);
        let result = self
            .live
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |live| {
                if new >= old {
                    live.checked_add(new - old)
                } else {
                    live.checked_sub(old - new)
                }
            });
        match result {
            Ok(previous) => {
                let live = if new >= old {
                    previous + (new - old)
                } else {
                    previous - (old - new)
                };
                after_live();
                self.peak.fetch_max(live, Ordering::Relaxed);
                let current_epoch = self.window_epoch.load(Ordering::SeqCst);
                // Registered hooks may finish posting to their just-closed
                // window. Their registration prevents a successor from opening
                // until publication completes; finish waits or reports missing.
                if epoch & 1 != 0 && (current_epoch == epoch || current_epoch == epoch + 1) {
                    self.window_peak.fetch_max(live, Ordering::Relaxed);
                }
            }
            Err(_) => {
                self.overflow.store(true, Ordering::Relaxed);
            }
        }
        self.window_inflight.fetch_sub(1, Ordering::SeqCst);
    }
    #[cfg(test)]
    fn window_peak_update(&self, epoch: u64, live: usize, before_write: impl FnOnce()) {
        if epoch & 1 == 0 {
            return;
        }
        // SeqCst orders registration/recheck with close and the next begin's
        // retirement check. A delayed old epoch cannot write a new peak.
        self.window_inflight.fetch_add(1, Ordering::SeqCst);
        if self.window_epoch.load(Ordering::SeqCst) == epoch {
            before_write();
            self.window_peak.fetch_max(live, Ordering::Relaxed);
        }
        self.window_inflight.fetch_sub(1, Ordering::SeqCst);
    }
    pub fn snapshot(&self) -> HeapSnapshot {
        let overflow = self.overflow.load(Ordering::Relaxed);
        let instrumented = self.observed.load(Ordering::Relaxed);
        let live = self.live.load(Ordering::Relaxed) as u64;
        let peak = (self.peak.load(Ordering::Relaxed) as u64).max(live);
        HeapSnapshot {
            instrumented,
            live_requested_bytes: (instrumented && !overflow).then_some(live),
            lifetime_peak_requested_bytes: (instrumented && !overflow).then_some(peak),
            successful_allocations: self.allocations.load(Ordering::Relaxed),
            successful_zeroed_allocations: self.zeroed.load(Ordering::Relaxed),
            successful_reallocations: self.reallocations.load(Ordering::Relaxed),
            failed_allocations: self.failed_allocations.load(Ordering::Relaxed),
            failed_reallocations: self.failed_reallocations.load(Ordering::Relaxed),
            deallocations: self.deallocations.load(Ordering::Relaxed),
            counter_overflowed: overflow,
            concurrent_snapshot: true,
        }
    }
    pub fn begin_window(&self) -> Result<HeapWindowGuard<'_>, &'static str> {
        if self
            .window_reserved
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_err()
        {
            return Err("a process allocator window is already active");
        }
        let hook_sequence = self.hook_sequence.load(Ordering::SeqCst);
        if self.window_inflight.load(Ordering::SeqCst) != 0 {
            self.window_reserved.store(false, Ordering::Release);
            return Err("allocator hooks from the previous window are still retiring");
        }
        let epoch = self.window_epoch.load(Ordering::SeqCst);
        if epoch & 1 != 0 || epoch.checked_add(2).is_none() {
            self.window_reserved.store(false, Ordering::Release);
            return Err("allocator window epoch is exhausted or invalid");
        }
        let baseline = self.snapshot();
        self.window_boundary_uncertain
            .store(false, Ordering::Relaxed);
        self.window_baseline.store(
            baseline.live_requested_bytes.unwrap_or(0),
            Ordering::Relaxed,
        );
        self.window_baseline_valid
            .store(baseline.live_requested_bytes.is_some(), Ordering::Relaxed);
        self.window_peak.store(
            baseline.live_requested_bytes.unwrap_or(0) as usize,
            Ordering::Relaxed,
        );
        self.window_epoch.store(epoch + 1, Ordering::SeqCst);
        self.window_peak
            .fetch_max(self.live.load(Ordering::Relaxed), Ordering::Relaxed);
        // If a hook spans opening or completed during its snapshots, the
        // boundary is explicitly unavailable instead of claiming an exact peak.
        if hook_sequence != self.hook_sequence.load(Ordering::SeqCst)
            || self.window_inflight.load(Ordering::SeqCst) != 0
        {
            self.window_boundary_uncertain
                .store(true, Ordering::Relaxed);
        }
        Ok(HeapWindowGuard {
            ledger: self,
            baseline,
            active: true,
        })
    }
    pub fn active_window(&self) -> Option<ActiveHeapWindow> {
        for _ in 0..3 {
            let epoch = self.window_epoch.load(Ordering::SeqCst);
            if epoch & 1 == 0 {
                return None;
            }
            let baseline = self
                .window_baseline_valid
                .load(Ordering::Relaxed)
                .then(|| self.window_baseline.load(Ordering::Relaxed));
            let current = self.snapshot();
            let inflight = self.window_inflight.load(Ordering::SeqCst);
            let boundary = self.window_boundary_uncertain.load(Ordering::Relaxed);
            let peak = self.window_peak.load(Ordering::Relaxed) as u64;
            if epoch == self.window_epoch.load(Ordering::SeqCst) {
                return Some(ActiveHeapWindow {
                    epoch,
                    baseline_live_requested_bytes: baseline,
                    peak_requested_bytes: (inflight == 0
                        && !boundary
                        && current.instrumented
                        && !current.counter_overflowed)
                        .then(|| {
                            peak.max(baseline.unwrap_or(0))
                                .max(current.live_requested_bytes.unwrap_or(0))
                        }),
                    live_delta_bytes: baseline
                        .zip(current.live_requested_bytes)
                        .map(|(a, b)| i128::from(b) - i128::from(a)),
                    current,
                    inflight_hooks: inflight,
                    boundary_updates_unavailable: boundary,
                    concurrent_snapshot: true,
                });
            }
        }
        None
    }
}

pub struct CountingAllocator<'a, A> {
    inner: A,
    ledger: &'a AllocatorLedger,
}
impl<'a, A> CountingAllocator<'a, A> {
    pub const fn new(inner: A, ledger: &'a AllocatorLedger) -> Self {
        Self { inner, ledger }
    }
}
unsafe impl<A: GlobalAlloc> GlobalAlloc for CountingAllocator<'_, A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = self.inner.alloc(layout);
        self.ledger.observed.store(true, Ordering::Relaxed);
        if pointer.is_null() {
            self.ledger.increment(&self.ledger.failed_allocations);
        } else {
            self.ledger.increment(&self.ledger.allocations);
            self.ledger.change_live(0, layout.size());
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = self.inner.alloc_zeroed(layout);
        self.ledger.observed.store(true, Ordering::Relaxed);
        if pointer.is_null() {
            self.ledger.increment(&self.ledger.failed_allocations);
        } else {
            self.ledger.increment(&self.ledger.allocations);
            self.ledger.increment(&self.ledger.zeroed);
            self.ledger.change_live(0, layout.size());
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        self.ledger.increment(&self.ledger.deallocations);
        self.ledger.change_live(layout.size(), 0);
        self.inner.dealloc(pointer, layout);
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let result = self.inner.realloc(pointer, layout, size);
        if result.is_null() {
            self.ledger.increment(&self.ledger.failed_reallocations);
        } else {
            self.ledger.increment(&self.ledger.reallocations);
            self.ledger.change_live(layout.size(), size);
        }
        result
    }
}

pub struct HeapWindowGuard<'a> {
    ledger: &'a AllocatorLedger,
    baseline: HeapSnapshot,
    active: bool,
}
impl HeapWindowGuard<'_> {
    pub fn finish(self) -> HeapWindow {
        self.finish_with(|| {})
    }
    fn finish_with(mut self, after_close: impl FnOnce()) -> HeapWindow {
        self.ledger.window_epoch.fetch_add(1, Ordering::SeqCst);
        after_close();
        for _ in 0..64 {
            if self.ledger.window_inflight.load(Ordering::SeqCst) == 0 {
                break;
            }
            std::hint::spin_loop();
        }
        let retiring_hooks_at_end = self.ledger.window_inflight.load(Ordering::SeqCst);
        let boundary_updates_unavailable = self
            .ledger
            .window_boundary_uncertain
            .load(Ordering::Relaxed);
        let end = self.ledger.snapshot();
        let peak = self.ledger.window_peak.load(Ordering::Relaxed) as u64;
        let peak_requested_bytes = if retiring_hooks_at_end == 0
            && !boundary_updates_unavailable
            && !end.counter_overflowed
            && end.instrumented
        {
            Some(
                peak.max(self.baseline.live_requested_bytes.unwrap_or(0))
                    .max(end.live_requested_bytes.unwrap_or(0)),
            )
        } else {
            None
        };
        self.ledger.window_reserved.store(false, Ordering::Release);
        self.active = false;
        HeapWindow { baseline: self.baseline, end, peak_requested_bytes, live_delta_bytes: self.baseline.live_requested_bytes.zip(end.live_requested_bytes).map(|(a,b)| i128::from(b) - i128::from(a)), retiring_hooks_at_end, boundary_updates_unavailable, scope: "process allocator requested bytes; concurrent boundary snapshots; includes observer and other sessions; excludes allocator fragmentation and OS RSS" }
    }
}
impl Drop for HeapWindowGuard<'_> {
    fn drop(&mut self) {
        if self.active {
            self.ledger.window_epoch.fetch_add(1, Ordering::SeqCst);
            self.ledger.window_reserved.store(false, Ordering::Release);
        }
    }
}

static PROCESS_LEDGER: AllocatorLedger = AllocatorLedger::new();
pub const fn process_allocator() -> CountingAllocator<'static, System> {
    CountingAllocator::new(System, &PROCESS_LEDGER)
}
pub fn process_heap() -> HeapSnapshot {
    PROCESS_LEDGER.snapshot()
}
pub fn process_active_window() -> Option<ActiveHeapWindow> {
    PROCESS_LEDGER.active_window()
}
pub fn begin_process_window() -> Result<HeapWindowGuard<'static>, &'static str> {
    PROCESS_LEDGER.begin_window()
}
#[cfg(test)]
pub(crate) fn process_live_bytes() -> usize {
    PROCESS_LEDGER.live.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct RefuseRealloc;
    unsafe impl GlobalAlloc for RefuseRealloc {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            System.alloc(layout)
        }
        unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
            System.dealloc(pointer, layout);
        }
        unsafe fn realloc(&self, _: *mut u8, _: Layout, _: usize) -> *mut u8 {
            std::ptr::null_mut()
        }
    }
    struct RefuseAlloc;
    unsafe impl GlobalAlloc for RefuseAlloc {
        unsafe fn alloc(&self, _: Layout) -> *mut u8 {
            std::ptr::null_mut()
        }
        unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
            panic!("failed allocation cannot be freed");
        }
    }
    #[test]
    fn cmrp_allocator_hook_retiring_after_close_still_publishes_old_window_peak() {
        use std::sync::{Arc, Barrier};
        let ledger = Arc::new(AllocatorLedger::new());
        ledger.observed.store(true, Ordering::Relaxed);
        ledger.live.store(100, Ordering::Relaxed);
        let window = ledger.begin_window().unwrap();
        let reached = Arc::new(Barrier::new(2));
        let resume = Arc::new(Barrier::new(2));
        let worker = {
            let (ledger, reached, resume) = (ledger.clone(), reached.clone(), resume.clone());
            std::thread::spawn(move || {
                ledger.change_live_with(0, 100, || {
                    reached.wait();
                    resume.wait();
                })
            })
        };
        reached.wait();
        ledger.change_live(100, 0);
        let end = window.finish_with(|| {
            resume.wait();
            worker.join().unwrap();
        });
        assert_eq!(end.retiring_hooks_at_end, 0);
        assert_eq!(end.end.live_requested_bytes, Some(100));
        assert_eq!(end.peak_requested_bytes, Some(200));
        assert_eq!(
            ledger.begin_window().unwrap().finish().peak_requested_bytes,
            Some(100)
        );
    }
    #[test]
    fn cmrp_allocator_registered_live_update_cannot_hide_transient_peak_at_close() {
        use std::sync::{Arc, Barrier};
        let ledger = Arc::new(AllocatorLedger::new());
        ledger.observed.store(true, Ordering::Relaxed);
        ledger.live.store(100, Ordering::Relaxed);
        let window = ledger.begin_window().unwrap();
        let reached = Arc::new(Barrier::new(2));
        let resume = Arc::new(Barrier::new(2));
        let worker = {
            let (ledger, reached, resume) = (ledger.clone(), reached.clone(), resume.clone());
            std::thread::spawn(move || {
                ledger.change_live_with(0, 100, || {
                    reached.wait();
                    resume.wait();
                })
            })
        };
        reached.wait();
        assert_eq!(ledger.live.load(Ordering::Relaxed), 200);
        // Free a different block already present at baseline, not the pending
        // allocation's unpublished pointer.
        ledger.change_live(100, 0);
        assert_eq!(ledger.live.load(Ordering::Relaxed), 100);
        assert!(ledger
            .active_window()
            .unwrap()
            .peak_requested_bytes
            .is_none());
        let end = window.finish();
        assert_eq!(end.retiring_hooks_at_end, 1);
        assert!(end.peak_requested_bytes.is_none());
        assert!(ledger.begin_window().is_err());
        resume.wait();
        worker.join().unwrap();
        let successor = ledger.begin_window().unwrap().finish();
        assert_eq!(successor.peak_requested_bytes, Some(100));
        assert_eq!(ledger.snapshot().lifetime_peak_requested_bytes, Some(200));
    }
    #[test]
    fn cmrp_allocator_active_view_reads_existing_epoch_and_marks_missing() {
        let ledger = AllocatorLedger::new();
        assert!(ledger.active_window().is_none());
        let window = ledger.begin_window().unwrap();
        let a = ledger.active_window().unwrap();
        assert!(a.baseline_live_requested_bytes.is_none());
        assert!(a.peak_requested_bytes.is_none());
        assert_eq!(ledger.active_window().unwrap().epoch, a.epoch);
        assert!(ledger.begin_window().is_err());
        drop(window);
        assert!(ledger.active_window().is_none());
        ledger.observed.store(true, Ordering::Relaxed);
        ledger.live.store(40, Ordering::Relaxed);
        let window = ledger.begin_window().unwrap();
        ledger.live.store(10, Ordering::Relaxed);
        let view = ledger.active_window().unwrap();
        assert_eq!(view.baseline_live_requested_bytes, Some(40));
        assert_eq!(view.peak_requested_bytes, Some(40));
        assert_eq!(view.live_delta_bytes, Some(-30));
        ledger.window_inflight.store(1, Ordering::SeqCst);
        assert!(ledger
            .active_window()
            .unwrap()
            .peak_requested_bytes
            .is_none());
        ledger.window_inflight.store(0, Ordering::SeqCst);
        window.finish();
        assert!(ledger.active_window().is_none());
    }
    #[test]
    fn cmrp_allocator_real_hooks_preserve_pre_window_free_zeroed_realloc_and_delta() {
        let ledger = AllocatorLedger::new();
        let allocator = CountingAllocator::new(System, &ledger);
        let failing_realloc = CountingAllocator::new(RefuseRealloc, &ledger);
        let failing_alloc = CountingAllocator::new(RefuseAlloc, &ledger);
        let layout = Layout::from_size_align(32, 8).unwrap();
        unsafe {
            let original = allocator.alloc(layout);
            assert!(!original.is_null());
            original.write(19);
            let window = ledger.begin_window().unwrap();
            assert_eq!(window.baseline.live_requested_bytes, Some(32));
            assert!(ledger.begin_window().is_err());
            let zeros = allocator.alloc_zeroed(Layout::from_size_align(64, 8).unwrap());
            assert!(!zeros.is_null());
            assert!(std::slice::from_raw_parts(zeros, 64)
                .iter()
                .all(|v| *v == 0));
            assert!(failing_alloc.alloc(layout).is_null());
            assert!(failing_realloc.realloc(original, layout, 128).is_null());
            assert_eq!(original.read(), 19);
            assert_eq!(ledger.snapshot().live_requested_bytes, Some(96));
            let grown = allocator.realloc(original, layout, 48);
            assert!(!grown.is_null());
            assert_eq!(grown.read(), 19);
            assert_eq!(ledger.snapshot().live_requested_bytes, Some(112));
            allocator.dealloc(grown, Layout::from_size_align(48, 8).unwrap());
            allocator.dealloc(zeros, Layout::from_size_align(64, 8).unwrap());
            let result = window.finish();
            assert_eq!(result.end.live_requested_bytes, Some(0));
            assert_eq!(result.live_delta_bytes, Some(-32));
            assert_eq!(result.peak_requested_bytes, Some(112));
            assert_eq!(result.retiring_hooks_at_end, 0);
            assert_eq!(result.end.successful_allocations, 2);
            assert_eq!(result.end.successful_zeroed_allocations, 1);
            assert_eq!(result.end.successful_reallocations, 1);
            assert_eq!(result.end.failed_reallocations, 1);
            assert_eq!(result.end.failed_allocations, 1);
            assert_eq!(result.end.deallocations, 2);
            drop(ledger.begin_window().unwrap());
            assert!(ledger.begin_window().is_ok());
        }
    }
    #[test]
    fn cmrp_allocator_concurrent_hooks_and_window_drop_are_finite_and_reusable() {
        let ledger = AllocatorLedger::new();
        let allocator = CountingAllocator::new(System, &ledger);
        let layout = Layout::from_size_align(32, 8).unwrap();
        unsafe {
            let warm = allocator.alloc(layout);
            assert!(!warm.is_null());
            allocator.dealloc(warm, layout);
        }
        let window = ledger.begin_window().unwrap();
        std::thread::scope(|scope| {
            for _ in 0..4 {
                let allocator = &allocator;
                scope.spawn(move || {
                    for _ in 0..100 {
                        unsafe {
                            let pointer = allocator.alloc(layout);
                            assert!(!pointer.is_null());
                            allocator.dealloc(pointer, layout);
                        }
                    }
                });
            }
        });
        let result = window.finish();
        assert_eq!(result.live_delta_bytes, Some(0));
        assert_eq!(result.end.successful_allocations, 401);
        assert_eq!(result.end.deallocations, 401);
        assert!((32..=128).contains(&result.peak_requested_bytes.unwrap()));
        let next = ledger.begin_window().unwrap().finish();
        assert_eq!(next.peak_requested_bytes, Some(0));
        assert!(next.end.lifetime_peak_requested_bytes.unwrap() >= 32);
    }
    #[test]
    fn cmrp_allocator_old_delayed_hook_cannot_write_into_successor_window() {
        let ledger = AllocatorLedger::new();
        ledger.observed.store(true, Ordering::Relaxed);
        let window = ledger.begin_window().unwrap();
        let old_epoch = ledger.window_epoch.load(Ordering::SeqCst);
        let entered = std::sync::Barrier::new(2);
        let released = std::sync::Barrier::new(2);
        std::thread::scope(|scope| {
            scope.spawn(|| {
                ledger.window_peak_update(old_epoch, 999, || {
                    entered.wait();
                    released.wait();
                })
            });
            entered.wait();
            let result = window.finish();
            assert_eq!(result.retiring_hooks_at_end, 1);
            assert_eq!(result.peak_requested_bytes, None);
            assert!(
                ledger.begin_window().is_err(),
                "the old accepted peak write is still in flight"
            );
            released.wait();
        });
        let new_window = ledger.begin_window().unwrap();
        ledger.window_peak_update(old_epoch, 9999, || {
            panic!("expired epoch cannot pass recheck")
        });
        assert_eq!(new_window.finish().peak_requested_bytes, Some(0));
    }
    #[test]
    fn cmrp_allocator_peak_includes_valid_baseline_and_end_when_hooks_lag() {
        let ledger = AllocatorLedger::new();
        ledger.observed.store(true, Ordering::Relaxed);
        ledger.live.store(90, Ordering::Relaxed);
        let window = ledger.begin_window().unwrap();
        ledger.live.store(120, Ordering::Relaxed);
        // Synthetic schedule: the live update completed before its peak hook.
        ledger.window_peak.store(1, Ordering::Relaxed);
        assert_eq!(window.finish().peak_requested_bytes, Some(120));
        ledger.live.store(90, Ordering::Relaxed);
        let window = ledger.begin_window().unwrap();
        ledger.live.store(0, Ordering::Relaxed);
        ledger.window_peak.store(1, Ordering::Relaxed);
        assert_eq!(window.finish().peak_requested_bytes, Some(90));
    }
    #[test]
    fn cmrp_allocator_missing_overflow_and_epoch_exhaustion_never_wrap_as_valid_zero() {
        let ledger = AllocatorLedger::new();
        let missing = ledger.snapshot();
        assert!(!missing.instrumented);
        assert!(missing.live_requested_bytes.is_none());
        assert!(missing.lifetime_peak_requested_bytes.is_none());
        ledger.observed.store(true, Ordering::Relaxed);
        ledger.live.store(usize::MAX, Ordering::Relaxed);
        ledger.change_live(0, 1);
        assert!(ledger.snapshot().counter_overflowed);
        assert!(ledger.snapshot().live_requested_bytes.is_none());
        assert!(ledger.snapshot().lifetime_peak_requested_bytes.is_none());
        ledger.window_epoch.store(u64::MAX - 1, Ordering::SeqCst);
        assert!(ledger.begin_window().is_err());
        assert!(!ledger.window_reserved.load(Ordering::Relaxed));
    }
}
