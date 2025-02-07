use std::cell::Cell;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static THREAD_COUNTER: AtomicUsize = AtomicUsize::new(1);

thread_local! {
    static THREAD_ID: Cell<usize> = Cell::new(THREAD_COUNTER.fetch_add(1, Ordering::SeqCst));
}

/// Provides a unique number for the current thread.
pub fn thread_id() -> usize {
    THREAD_ID.with(Cell::get)
}
