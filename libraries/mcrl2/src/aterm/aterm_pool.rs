use core::fmt;
use std::{cell::RefCell, sync::Arc, mem::ManuallyDrop};

use log::trace;

use mcrl2_sys::{
    atermpp::ffi,
    cxx::{Exception, UniquePtr},
};
use utilities::protection_set::ProtectionSet;

use crate::{aterm::{ATerm, ATermTrait, BfTermPoolThreadWrite, Symbol, SymbolTrait}, data::DataExpressionRef};

use super::{ATermRef, global_aterm_pool::{SharedProtectionSet, SharedContainerProtectionSet, ATermPtr, mark_protection_sets, protection_set_size, GLOBAL_TERM_POOL}, Markable};

thread_local! {
    /// This is the thread specific term pool that manages the protection sets.
    pub(crate) static THREAD_TERM_POOL: RefCell<ThreadTermPool> = RefCell::new(ThreadTermPool::new());
}

pub struct ThreadTermPool {
    protection_set: SharedProtectionSet,
    container_protection_set: SharedContainerProtectionSet,
    
    index: usize,

    /// Function symbols to represent 'DataAppl' with any number of arguments.
    data_appl: Vec<Symbol>,

    // TODO: On macOS destroying the callback causes issues. However, this should be fine since the related thread_aterm_pool is destroyed.
    _callback: ManuallyDrop<UniquePtr<ffi::callback_container>>,
}

/// Protects the given aterm address and returns the term.
///     - guard: An existing guard to the ThreadTermPool.protection_set.
///     - index: The index of the ThreadTermPool
fn protect_with(mut guard: BfTermPoolThreadWrite<'_, ProtectionSet<ATermPtr>>, index: usize, term: *const ffi::_aterm) -> ATerm {
    debug_assert!(!term.is_null(), "Can only protect valid terms");
    let aterm = ATermPtr::new(term);
    let root = guard.protect(aterm.clone());

    let term = ATermRef::new(term);
    trace!(
        "Protected term {:?}, index {}, protection set {}",
        term,
        root,
        index
    );

    ATerm { term, root }
}

impl ThreadTermPool {
    pub fn new() -> ThreadTermPool {

        // Register a protection set into the global set.
        let (protection_set, container_protection_set, index) = GLOBAL_TERM_POOL.lock().register_thread_term_pool();

        ThreadTermPool {
            protection_set,
            container_protection_set,
            index,
            _callback: ManuallyDrop::new(ffi::register_mark_callback(mark_protection_sets, protection_set_size)),
            data_appl: vec![],
        }
    }

    /// Protects the given aterm address and returns the term.
    pub fn protect(&mut self, term: *const ffi::_aterm) -> ATerm {
        unsafe {
            protect_with(self.protection_set.write_exclusive(true), self.index, term)
        }
    }

    /// Protects the given aterm address and returns the term.
    pub fn protect_container(&mut self, container: Arc<dyn Markable + Send + Sync>) -> usize {
        let root = unsafe {
            self.container_protection_set.write_exclusive(true).protect(container)
        };
    
        trace!(
            "Protected container index {}, protection set {}",
            root,
            self.index,
        );
    
        root
    }

    /// Removes the [ATerm] from the protection set.
    pub fn drop(&mut self, term: &ATerm) {
        term.require_valid();

        unsafe {
            let mut protection_set = self.protection_set.write_exclusive(true);
            trace!(
                "Dropped term {:?}, index {}, protection set {}",
                term.term,
                term.root,
                self.index
            );
            protection_set.unprotect(term.root);
        }
    }

    /// Removes the container from the protection set.
    pub fn drop_container(&mut self, container_root: usize) {
        
        unsafe {
            let mut container_protection_set = self.container_protection_set.write_exclusive(true);
            trace!(
                "Dropped container index {}, protection set {}",
                container_root,
                self.index
            );
            container_protection_set.unprotect(container_root);
        }
    }

    /// Returns true iff the given term is a data application.
    pub fn is_data_application(&mut self, term: &ATermRef<'_>) -> bool {     
        let symbol = term.get_head_symbol();   
        // It can be that data_applications are created without create_data_application in the mcrl2 ffi.
        while self.data_appl.len() <= symbol.arity() {
            let symbol = Symbol::take(ffi::create_function_symbol(String::from("DataAppl"), self.data_appl.len()));
            self.data_appl.push(symbol);
        }

        symbol == self.data_appl[symbol.arity()].copy()
    }
}

impl Default for ThreadTermPool {
    fn default() -> Self {
        ThreadTermPool::new()
    }
}

impl Drop for ThreadTermPool {
    fn drop(&mut self) {
        debug_assert!(self.protection_set.read().len() == 0, "The protection set should be empty");
        GLOBAL_TERM_POOL.lock().drop_thread_term_pool(self.index);
    }
}

/// This is the thread local term pool.
pub struct TermPool {
    arguments: Vec<*const ffi::_aterm>,
}

impl TermPool {
    pub fn new() -> TermPool {
        TermPool {
            arguments: vec![],
        }
    }

    /// Trigger a garbage collection explicitly.
    pub fn collect(&mut self) {
        ffi::collect_garbage();
    }

    /// Creates an ATerm from a string.
    pub fn from_string(&mut self, text: &str) -> Result<ATerm, Exception> {
        match ffi::aterm_from_string(String::from(text)) {
            Ok(term) => Ok(term.into()),
            Err(exception) => Err(exception),
        }
    }

    /// Creates an [ATerm] with the given symbol and arguments.
    pub fn create(&mut self, symbol: &impl SymbolTrait, arguments: &[impl ATermTrait]) -> ATerm {
        // Make the temp vector sufficient length.
        while self.arguments.len() < arguments.len() {
            self.arguments.push(std::ptr::null());
        }

        self.arguments.clear();
        for arg in arguments {
            unsafe {
                self.arguments.push(arg.copy().get());
            }
        }

        debug_assert_eq!(
            symbol.arity(),
            self.arguments.len(),
            "Number of arguments does not match arity"
        );

        let result = THREAD_TERM_POOL.with_borrow_mut(|tp| {
            unsafe {
                // ThreadPool is not Sync, so only one has access.
                let protection_set = tp.protection_set.write_exclusive(true);
                let term: *const ffi::_aterm = ffi::create_aterm(symbol.address(), &self.arguments);
                protect_with(protection_set, tp.index, term)
            }
        });

        ffi::test_garbage_collection();
        result
    }   
    
     /// Creates an [ATerm] with the given symbol, head argument and other arguments.
    pub fn create_data_application(&mut self, head: &impl ATermTrait, arguments: &[impl ATermTrait]) -> ATerm { 
        // Make the temp vector sufficient length.
        while self.arguments.len() < arguments.len() {
            self.arguments.push(std::ptr::null());
        }

        self.arguments.clear();
        unsafe {
            self.arguments.push(head.copy().get());
            for arg in arguments {
                    self.arguments.push(arg.copy().get());
            }
        }    

        THREAD_TERM_POOL.with_borrow_mut(|tp| {
            while tp.data_appl.len() <= arguments.len() + 1 {
                let symbol = self.create_symbol("DataAppl", tp.data_appl.len());
                tp.data_appl.push(symbol);
            }

            let symbol = &tp.data_appl[arguments.len() + 1];

            debug_assert_eq!(
                symbol.arity(),
                self.arguments.len(),
                "Number of arguments does not match arity"
            );

            let result = unsafe {
                // ThreadPool is not Sync, so only one has access.
                let protection_set = tp.protection_set.write_exclusive(true);
                let term: *const ffi::_aterm = ffi::create_aterm(symbol.address(), &self.arguments);
                protect_with(protection_set, tp.index, term)
            };

            ffi::test_garbage_collection();
            result
        })
    }
    
     /// Creates an [ATerm] with the given symbol, head argument and other arguments.
     pub fn create_data_application2(&mut self, head: &impl ATermTrait, arguments: &[DataExpressionRef<'_>]) -> ATerm { 
        // Make the temp vector sufficient length.
        while self.arguments.len() < arguments.len() {
            self.arguments.push(std::ptr::null());
        }

        self.arguments.clear();
        unsafe {
            self.arguments.push(head.copy().get());
            for arg in arguments {
                    self.arguments.push(arg.copy().get());
            }
        }    

        THREAD_TERM_POOL.with_borrow_mut(|tp| {
            while tp.data_appl.len() <= arguments.len() + 1 {
                let symbol = self.create_symbol("DataAppl", tp.data_appl.len());
                tp.data_appl.push(symbol);
            }

            let symbol = &tp.data_appl[arguments.len() + 1];

            debug_assert_eq!(
                symbol.arity(),
                self.arguments.len(),
                "Number of arguments does not match arity"
            );

            // ThreadPool is not Sync, so only one has access.
            let result = unsafe {
                let protection_set = tp.protection_set.write_exclusive(true);
                let term: *const ffi::_aterm = ffi::create_aterm(symbol.address(), &self.arguments);
                protect_with(protection_set, tp.index, term)
            };

            ffi::test_garbage_collection();
            result
        })
    }

    /// Creates a function symbol with the given name and arity.
    pub fn create_symbol(&mut self, name: &str, arity: usize) -> Symbol {
        Symbol::take(ffi::create_function_symbol(String::from(name), arity))
    }

    /// Creates a term with the FFI while taking care of the protection and garbage collection.
    pub fn create_with<F>(&mut self, create: F) -> ATerm 
        where F: Fn() -> *const ffi::_aterm,
    {
        let result = THREAD_TERM_POOL.with_borrow_mut(|tp| {
            unsafe {
                // ThreadPool is not Sync, so only one has access.
                let protection_set = tp.protection_set.write_exclusive(true);
                protect_with(protection_set, tp.index, create())
            }
        });

        ffi::test_garbage_collection();
        result
    }
}

impl Default for TermPool {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TermPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: This will always print, but only depends on aterm_configuration.h
        ffi::print_metrics();

        write!(f, "{:?}", GLOBAL_TERM_POOL.lock())
    }
}


#[cfg(test)]
mod tests {
    use std::thread;

    use crate::aterm::random_term;

    use super::*;
    
    /// Make sure that the term has the same number of arguments as its arity.
    fn verify_term(term: &ATerm) {
        for subterm in term.iter() {
            assert_eq!(
                subterm.get_head_symbol().arity(),
                subterm.arguments().len(),
                "The arity matches the number of arguments."
            )
        }
    }

    #[test]
    fn test_thread_aterm_pool_parallel() {
        let mut threads = vec![];

        for _ in 0..2 {
            threads.push(thread::spawn(|| {
                let mut tp = TermPool::new();

                let terms: Vec<ATerm> = (0..100)
                    .map(|_| {
                        random_term(
                            &mut tp,
                            &[("f".to_string(), 2)],
                            &["a".to_string(), "b".to_string()],
                            10,
                        )
                    })
                    .collect();

                tp.collect();

                for term in &terms {
                    verify_term(term);
                }
            }));
        }

        // Join the threads
        for thread in threads {
            thread.join().unwrap();
        }
    }
}