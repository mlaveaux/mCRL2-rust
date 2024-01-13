use std::alloc::{System, GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

/// An allocator that can be used globally to count metrics
/// on the allocations performed.
pub struct AllocCounter;

static NUMBER_OF_ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

impl AllocCounter {

    /// Returns the total number of allocations since program start.
    pub fn number_of_allocations(&self) -> usize {
        NUMBER_OF_ALLOCATIONS.load(Relaxed)
    }
}

unsafe impl GlobalAlloc for AllocCounter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            NUMBER_OF_ALLOCATIONS.fetch_add(1, Relaxed);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
    }
}

