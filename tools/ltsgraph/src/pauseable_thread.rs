use std::{sync::{atomic::{AtomicBool, Ordering}, Arc, Condvar, Mutex}, thread::{Builder, JoinHandle}};

/// A thread that can be paused and quit, used for interactive parts of the UI.
pub struct PauseableThread {
    handle: Option<JoinHandle<()>>,
    shared: Arc<PauseableThreadShared>,
}

struct PauseableThreadShared {
    running: AtomicBool,
    paused: Mutex<bool>,
    cond_var: Condvar,
}

impl PauseableThread {

    /// Spawns a new thread that can be paused and stopped.
    pub fn new<F>(name: &str, loop_function: F) -> Result<PauseableThread, std::io::Error> 
        where F: Fn() -> () + Send + 'static,
        {
        
        let shared = Arc::new(PauseableThreadShared{
            running: AtomicBool::new(true),
            paused: Mutex::new(false),
            cond_var: Condvar::new(),
        });

        let thread = {
            let shared = shared.clone();
            Builder::new()
                .name(name.to_string())
                .spawn(move || {
                    while shared.running.load(std::sync::atomic::Ordering::Relaxed) {

                        // Check if paused is true and wait for it.
                        {
                            let mut paused = shared.paused.lock().unwrap();
                            while *paused {
                                paused = shared.cond_var.wait(paused).unwrap();
                            }
                        }

                        loop_function();
                    }
                })
        }?;

        Ok(PauseableThread {
            handle: Some(thread),
            shared
        })
    }

    /// Signal the thread to quit, will be joined when it is dropped.
    pub fn stop(&self) {
        self.resume();
        self.shared.running.store(false, Ordering::Relaxed)
    }

    /// Pause the thread on the next iteration.
    pub fn pause(&self) {
        *self.shared.paused.lock().unwrap() = true;
        // We notify the condvar that the value has changed.
        self.shared.cond_var.notify_one();
    }

    /// Resume the thread.
    pub fn resume(&self) {
        *self.shared.paused.lock().unwrap() = false;
        // We notify the condvar that the value has changed.
        self.shared.cond_var.notify_one();
    }
    
}


impl Drop for PauseableThread {
    fn drop(&mut self) {
        self.stop();

        // Joining consumes the handle
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_textcache() {
        let thread = PauseableThread::new("test", move || {
            // Do nothing.
        }).unwrap();

        thread.stop();
    }
}
