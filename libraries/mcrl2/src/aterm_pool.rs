use core::fmt;
use std::{cell::RefCell, fmt::Debug, sync::Arc};

use log::{info, trace};
use parking_lot::Mutex;

use mcrl2_sys::{
    atermpp::ffi,
    cxx::{Exception, UniquePtr},
};
use utilities::protection_set::ProtectionSet;

use crate::{
    aterm::{ATerm, ATermTrait},
    data::{DataApplication, DataFunctionSymbol, DataVariable},
    symbol::{Symbol, SymbolTrait},
};

// TODO: Fix some of this garbage
#[derive(Clone, Debug)]
struct ATermPtr {
    ptr: *const ffi::_aterm,
}

impl ATermPtr {
    fn new(ptr: *const ffi::_aterm) -> ATermPtr {
        ATermPtr { ptr }
    }
}

unsafe impl Send for ATermPtr {}

type SharedProtectionSet = Arc<Mutex<ProtectionSet<ATermPtr>>>;

/// This is the global set of protection sets, that are managed by the ThreadTermPool
static PROTECTION_SETS: Mutex<Vec<Option<SharedProtectionSet>>> = Mutex::new(vec![]);

/// Marks the terms in all protection sets.
fn mark_protection_sets() {
    let mut protected = 0;
    let mut total = 0;
    for set in PROTECTION_SETS.lock().iter().flatten() {
        let protection_set = set.lock();

        for term in protection_set.iter() {
            unsafe {
                ffi::aterm_mark_address(term.ptr);
            }
        }

        protected += protection_set.len();
        total += protection_set.maximum_size();
    }

    info!(
        "Collecting garbage, {} protected terms and {} insertions",
        protected, total
    );
}

/// Counts the number of terms in all protection sets.
fn protection_set_size() -> usize {
    let mut result = 0;
    for set in PROTECTION_SETS.lock().iter().flatten() {
        result += set.lock().len();
    }
    result
}

thread_local! {
    /// This is the thread specific term pool that manages the protection sets.
    pub(crate) static THREAD_TERM_POOL: RefCell<ThreadTermPool> = RefCell::new(ThreadTermPool::new());
}

pub struct ThreadTermPool {
    protection_set: SharedProtectionSet,
    _callback: UniquePtr<ffi::callback_container>,
    index: usize,
}

impl ThreadTermPool {
    pub fn new() -> ThreadTermPool {
        // Initialise the C++ aterm library, disables garbage collection so that it can be performed in Rust.
        ffi::initialise();

        // Register a protection set into the global set.
        let protection_set = Arc::new(Mutex::new(ProtectionSet::new()));

        let mut lock = PROTECTION_SETS.lock();
        lock.push(Some(protection_set.clone()));

        trace!("Registered ThreadTermPool {}", lock.len() - 1);
        ThreadTermPool {
            protection_set,
            _callback: ffi::register_mark_callback(mark_protection_sets, protection_set_size),
            index: lock.len() - 1,
        }
    }

    /// Protects the given aterm address and returns the term.
    pub fn protect(&mut self, term: *const ffi::_aterm) -> ATerm {
        debug_assert!(!term.is_null(), "Can only protect valid terms");
        let root = self.protection_set.lock().protect(ATermPtr::new(term));

        trace!(
            "Protected term {:?}, index {}, set {}",
            term,
            root,
            self.index
        );
        ATerm { term, root }
    }

    /// Removes the [ATerm] from the protection set.
    pub fn drop(&mut self, term: &ATerm) {
        term.require_valid();

        trace!(
            "Dropped term {:?}, index {}, set {}",
            term.term,
            term.root,
            self.index
        );
        self.protection_set.lock().unprotect(term.root);
    }
}

impl Default for ThreadTermPool {
    fn default() -> Self {
        ThreadTermPool::new()
    }
}

impl Debug for ThreadTermPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let protection_set = self.protection_set.lock();
        write!(
            f,
            "{} variables in root set ({} max, {} insertions)",
            protection_set.len(),
            protection_set.maximum_size(),
            protection_set.number_of_insertions()
        )
    }
}

impl Drop for ThreadTermPool {
    fn drop(&mut self) {
        PROTECTION_SETS.lock()[self.index] = None;
        trace!("Removed ThreadTermPool {}", self.index);
    }
}

/// This is the thread local term pool.
pub struct TermPool {
    arguments: Vec<*const ffi::_aterm>,
    data_appl: Vec<Symbol>, // Function symbols to represent 'DataAppl' with any number of arguments.
}

impl TermPool {
    pub fn new() -> TermPool {
        TermPool {
            arguments: vec![],
            data_appl: vec![],
        }
    }

    /// Trigger a garbage collection if necessary, we disable global garbage collection so ffi will never garbage collect themselves.
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
    pub fn create(&mut self, symbol: &impl SymbolTrait, arguments: &[ATerm]) -> ATerm {
        let arguments = self.tmp_arguments(arguments);

        debug_assert_eq!(
            symbol.arity(),
            arguments.len(),
            "Number of arguments does not match arity"
        );

        let result = THREAD_TERM_POOL.with_borrow_mut(|tp| {
            let mut protection_set = tp.protection_set.lock();

            unsafe {
                let term: *const ffi::_aterm = ffi::create_aterm(symbol.address(), &arguments);
                let root = protection_set.protect(ATermPtr::new(term));
                trace!(
                    "Protected term {:?}, index {}, set {}",
                    term,
                    root,
                    tp.index
                );

                ATerm { term, root }
            }
        });

        self.collect();
        result
    }

    /// Creates a function symbol with the given name and arity.
    pub fn create_symbol(&mut self, name: &str, arity: usize) -> Symbol {
        Symbol::take(ffi::create_function_symbol(String::from(name), arity))
    }

    /// Creates a data application of head applied to the given arguments.
    pub fn create_data_application(
        &mut self,
        head: &ATerm,
        arguments: &[ATerm],
    ) -> DataApplication {
        // The ffi function to create a DataAppl is not thread safe, so implemented here locally.
        while self.data_appl.len() <= arguments.len() + 1 {
            let symbol = self.create_symbol("DataAppl", self.data_appl.len());
            self.data_appl.push(symbol);
        }

        let symbol = self.data_appl[arguments.len() + 1].borrow();
        let term = self.create_head(&symbol, head, arguments);

        DataApplication { term }
    }

    /// Creates a data variable with the given name.
    pub fn create_variable(&mut self, name: &str) -> DataVariable {
        let result = THREAD_TERM_POOL.with_borrow_mut(|tp| {
            let mut protection_set = tp.protection_set.lock();

            let term = ffi::create_data_variable(name.to_string());
            let root = protection_set.protect(ATermPtr::new(term));
            trace!(
                "Protected term {:?}, index {}, set {}",
                term,
                root,
                tp.index
            );

            DataVariable {
                term: ATerm { term, root },
            }
        });

        self.collect();
        result
    }

    /// Creates a data function symbol with the given name.
    pub fn create_data_function_symbol(&mut self, name: &str) -> DataFunctionSymbol {
        let result = THREAD_TERM_POOL.with_borrow_mut(|tp| {
            let mut protection_set = tp.protection_set.lock();

            let term = ffi::create_data_function_symbol(name.to_string());
            let root = protection_set.protect(ATermPtr::new(term));
            trace!(
                "Protected term {:?}, index {}, set {}",
                term,
                root,
                tp.index
            );

            DataFunctionSymbol {
                term: ATerm { term, root },
            }
        });

        self.collect();
        result
    }

    /// Returns true iff this is a data::application
    pub fn is_data_application<'a>(&mut self, term: &'a impl ATermTrait<'a>) -> bool {
        term.require_valid();

        let symbol = term.get_head_symbol();
        // It can be that data_applications are created without create_data_application in the mcrl2 ffi.
        while self.data_appl.len() <= symbol.arity() {
            let symbol = self.create_symbol("DataAppl", self.data_appl.len());
            self.data_appl.push(symbol);
        }

        symbol == self.data_appl[symbol.arity()].borrow()
    }

    /// Creates an [ATerm] with the given symbol, first and other arguments.
    fn create_head(
        &mut self,
        symbol: &impl SymbolTrait,
        head: &ATerm,
        arguments: &[ATerm],
    ) -> ATerm {
        let arguments = self.tmp_arguments_head(head, arguments);

        debug_assert_eq!(
            symbol.arity(),
            arguments.len(),
            "Number of arguments does not match arity"
        );

        let result = THREAD_TERM_POOL.with_borrow_mut(|tp| {
            let mut protection_set = tp.protection_set.lock();

            unsafe {
                let term = ffi::create_aterm(symbol.address(), &arguments);
                let root = protection_set.protect(ATermPtr::new(term));
                trace!(
                    "Protected term {:?}, index {}, set {}",
                    term,
                    root,
                    tp.index
                );

                ATerm { term, root }
            }
        });

        self.collect();
        result
    }

    /// Converts the [ATerm] slice into a [ffi::aterm_ref] slice.
    fn tmp_arguments(&mut self, arguments: &[ATerm]) -> &[*const ffi::_aterm] {
        // Make the temp vector sufficient length.
        while self.arguments.len() < arguments.len() {
            self.arguments.push(std::ptr::null());
        }

        self.arguments.clear();
        for arg in arguments {
            self.arguments.push(arg.borrow().term);
        }

        &self.arguments
    }

    /// Converts the [ATerm] slice into a [ffi::aterm_ref] slice.
    fn tmp_arguments_head(&mut self, head: &ATerm, arguments: &[ATerm]) -> &[*const ffi::_aterm] {
        // Make the temp vector sufficient length.
        while self.arguments.len() < arguments.len() + 1 {
            self.arguments.push(std::ptr::null());
        }

        self.arguments.clear();
        self.arguments.push(head.borrow().term);
        for arg in arguments {
            self.arguments.push(arg.borrow().term);
        }

        &self.arguments
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

        THREAD_TERM_POOL.with_borrow(|tp| write!(f, "{:?}", tp))
    }
}
