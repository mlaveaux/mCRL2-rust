//! Helper functions for handling the Windows console from a GUI context.
//!
//! Windows subsystem applications must explicitly attach to an existing console
//! before stdio works, and if not available, create their own if they wish to
//! print anything.
//!
//! These functions enable that, primarily for the purposes of displaying Rust
//! panics.

#[cfg(windows)]
use winapi::um::consoleapi::AllocConsole;

#[cfg(windows)]
use winapi::um::wincon::{AttachConsole, FreeConsole, GetConsoleWindow, ATTACH_PARENT_PROCESS};

pub struct Console {
    #[cfg(windows)]
    attached: bool,
}

/// Initialises the console. On Windows this either attaches to the
pub fn init() -> anyhow::Result<Console> {
    
    #[cfg(windows)]
    unsafe {

        // Check if we're attached to an existing Windows console
        if GetConsoleWindow().is_null() {
            // Try to attach to an existing Windows console.
            //
            // It's normally a no-brainer to call this - it just makes println! and friends
            // work as expected, without cluttering the screen with a console in the general
            // case.
            if AttachConsole(ATTACH_PARENT_PROCESS) == 0 {
                // Try to attach to a console, and if not, allocate ourselves a new one.
                if AllocConsole() != 0 {
                    Ok(Console {
                        attached: false
                    })
                } else {
                    anyhow::bail!("Failed to attach to a console, and to create one")
                }
            } else {
                // We attached to an existing console.
                Ok(Console {
                    attached: true
                })
            }
        } else {
            // The program was started with a console attached.
            Ok(Console {
                attached: true
            })  
        }    
    }

    #[cfg(not(windows))]
    {
        Ok(Console { })
    }
}

impl Drop for Console {
    fn drop(&mut self) {    
        // Free the allocated console, when it was not attached.
        #[cfg(windows)]
        if !self.attached {
            unsafe { FreeConsole() };
        }
    }
}