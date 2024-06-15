use std::sync::Mutex;
use std::sync::MutexGuard;

use once_cell::sync::Lazy;

pub type GlobalLockGuard = MutexGuard<'static, ()>;

/// A global lock for non thread safe FFI functions.
pub fn lock_global() -> GlobalLockGuard {
    GLOBAL_MUTEX.lock().expect("Failed to lock GLOBAL_MUTEX")
}

/// This is the global mutex used to guard non thread safe FFI functions.
pub(crate) static GLOBAL_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
