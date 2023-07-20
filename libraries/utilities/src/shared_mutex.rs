use std::{pin::Pin, sync::{Mutex, Arc, atomic::AtomicBool}};


/// A shared mutex (readers-writer lock) implementation based on the
/// busy-forbidden protocol. Every thread has their own SharedMutex instance.
struct SharedMutex {
    busy: AtomicBool,
    forbidden: AtomicBool,

    /// The shared list of shared mutex objects.
    other: Arc<Mutex<Vec<Pin<SharedMutex>>>>,
}

impl SharedMutex {
    pub fn new() -> SharedMutex {
        SharedMutex {
            busy: false.into(),
            forbidden: false.into(),
            other: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Clone for SharedMutex {
    fn clone(&self) -> Self {
        let result = Self { 
            busy: false.into(), 
            forbidden: false.into(), 
            other: self.other.clone() };

        //result.other.lock().unwrap().push(result.pin());
        result
    }
}

unsafe impl Send for SharedMutex {}