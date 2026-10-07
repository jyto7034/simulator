//! Optional host process instrumentation. Core does not depend on allocator/OS IO.
pub mod allocator;
pub mod playback;
pub mod rss;
pub use allocator::{
    begin_process_window, process_active_window, process_allocator, process_heap, ActiveHeapWindow,
    CountingAllocator, HeapSnapshot, HeapWindow,
};
pub use rss::{process_rss, RssSample};
