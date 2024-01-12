use std::alloc::{System, GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

struct AllocCounter;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

impl AllocCounter {
    pub fn number_of_allocations(&self) -> usize {
        ALLOCATED.load(Relaxed)
    }
}

unsafe impl GlobalAlloc for AllocCounter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            ALLOCATED.fetch_add(layout.size(), Relaxed);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED.fetch_sub(layout.size(), Relaxed);
    }
}

