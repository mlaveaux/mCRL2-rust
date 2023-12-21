use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::{collections::VecDeque, fmt};

use mcrl2_sys::{atermpp::ffi, cxx::UniquePtr};

use crate::data::{BoolSort, DataApplication, DataFunctionSymbol, DataVariable};
use crate::symbol::{SymbolRef, SymbolTrait};
use crate::aterm_pool::THREAD_TERM_POOL;

pub use crate::aterm_pool::*;

/// Rust interface of a atermpp::aterm

pub trait ATermTrait<'a> {
    /// Returns the indexed argument of the term
    fn arg(&self, index: usize) -> ATermRef;

    /// Returns the list of arguments as a collection
    fn arguments(&self) -> ATermArgs;

    /// Returns whether the term is the default term (not initialised)
    fn is_default(&self) -> bool;

    /// Returns true iff this is an aterm_list
    fn is_list(&self) -> bool;

    /// Returns true iff this is the empty aterm_list
    fn is_empty_list(&self) -> bool;

    /// Returns true iff this is a aterm_int
    fn is_int(&self) -> bool;

    /// Returns the head function symbol of the term.
    fn get_head_symbol(&'a self) -> SymbolRef<'a>;

    /// Returns an iterator over all arguments of the term that runs in pre order traversal of the term trees.
    fn iter(&'a self) -> TermIterator;

    // Recognizers for the data library

    /// Returns true iff this is a data::variable
    fn is_data_variable(&self) -> bool;

    /// Returns true iff this is a data::function_symbol
    fn is_data_function_symbol(&self) -> bool;

    fn is_data_where_clause(&self) -> bool;

    fn is_data_abstraction(&self) -> bool;

    fn is_data_untyped_identifier(&self) -> bool;

    /// Returns true iff the term is not default.
    fn require_valid(&self);
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ATermRef<'a> {
    pub(crate) term: *const ffi::_aterm,
    marker: PhantomData<&'a ()>,
}

unsafe impl<'a> Send for ATermRef<'a> {}

impl<'a> Default for ATermRef<'a> {
    fn default() -> Self {
        ATermRef {
            term: std::ptr::null(),
            marker: PhantomData,
        }
    }
}

impl<'a> ATermRef<'a> {
    pub fn protect(&self) -> ATerm {
        if self.is_default() {
            ATerm::default()
        } else {
            THREAD_TERM_POOL.with_borrow_mut(|tp| { tp.protect(self.term) })
        }
    }

    /// In some cases the lifetime analysis can not figure out transitive lifetimes, and this unsafe
    /// function can be used to extend the life time in that case.
    pub unsafe fn upgrade<'b: 'a>(&self) -> ATermRef<'b> {
        ATermRef::new(self.term)
    }
}

impl<'a> ATermRef<'a> {
    fn new(term: *const ffi::_aterm) -> ATermRef<'a> {
        ATermRef {
            term,
            marker: PhantomData,
        }
    }

    /// Borrows the term with a potentially shorter lifetime.
    pub fn borrow<'b: 'a>(&self) -> ATermRef<'b> {
        ATermRef::new(self.term)
    }
}

impl<'a> ATermTrait<'a> for ATermRef<'a> {
    fn arg(&self, index: usize) -> ATermRef {
        self.require_valid();
        debug_assert!(
            index < self.get_head_symbol().arity(),
            "arg({index}) is not defined for term {:?}",
            self
        );

        unsafe {
            ATermRef {
                term: ffi::get_term_argument(self.term, index),
                marker: PhantomData,
            }
        }
    }

    fn arguments(&self) -> ATermArgs {
        self.require_valid();

        ATermArgs::new(self.borrow())
    }

    fn is_default(&self) -> bool {
        self.term.is_null()
    }

    fn is_list(&self) -> bool {
        unsafe { ffi::aterm_is_list(self.term) }
    }

    fn is_empty_list(&self) -> bool {
        unsafe { ffi::aterm_is_empty_list(self.term) }
    }

    fn is_int(&self) -> bool {
        unsafe { ffi::aterm_is_int(self.term) }
    }

    fn get_head_symbol(&self) -> SymbolRef<'a> {
        self.require_valid();
        unsafe { ffi::get_aterm_function_symbol(self.term).into() }
    }

    fn iter(&self) -> TermIterator {
        TermIterator::new(self.protect())
    }

    fn is_data_variable(&self) -> bool {
        self.require_valid();
        unsafe { ffi::is_data_variable(self.term) }
    }

    fn is_data_function_symbol(&self) -> bool {
        self.require_valid();
        unsafe { ffi::is_data_function_symbol(self.term) }
    }

    fn is_data_where_clause(&self) -> bool {
        self.require_valid();
        unsafe { ffi::is_data_where_clause(self.term) }
    }

    fn is_data_abstraction(&self) -> bool {
        self.require_valid();
        unsafe { ffi::is_data_abstraction(self.term) }
    }

    fn is_data_untyped_identifier(&self) -> bool {
        self.require_valid();
        unsafe { ffi::is_data_untyped_identifier(self.term) }
    }

    fn require_valid(&self) {
        debug_assert!(
            !self.is_default(),
            "This function can only be called on valid terms, i.e., not default terms"
        );
    }
}



impl<'a> fmt::Display for ATermRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_default() {
            write!(f, "{:?}", self)
        } else if self.is_data_function_symbol() {
            write!(
                f,
                "{}",
                <ATerm as Into<DataFunctionSymbol>>::into(self.protect())
            )
        // } else if self.is_data_application() {
        //     write!(
        //         f,
        //         "{}",
        //         <ATerm as Into<DataApplication>>::into(self.clone())
        //     )
        } else if self.is_data_variable() {
            write!(f, "{}", <ATerm as Into<DataVariable>>::into(self.protect()))
        } else {
            write!(f, "{:?}", self)
        }
    }
}

impl<'a> fmt::Debug for ATermRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_default() {
            write!(f, "<default>")?;
        } else {
            unsafe {
                write!(f, "{}", ffi::print_aterm(self.term))?;
            }
            //for term in self.iter() {
            //   write!(f, "{:?}: [{}]", term.get_head_symbol(), ffi::aterm_pointer(&self.term))?;
            //}
        }

        Ok(())
    }
}

pub struct ATerm {
    pub(crate) term: *const ffi::_aterm,
    pub(crate) root: usize,
}

impl<'a> ATerm {
    pub fn borrow(&self) -> ATermRef {
        ATermRef::new(self.term)
    }
}

impl Default for ATerm {
    fn default() -> Self {
        ATerm {
            term: std::ptr::null(),
            root: 0,
        }
    }
}

impl Drop for ATerm {
    fn drop(&mut self) {
        if !self.is_default() {
            THREAD_TERM_POOL.with_borrow_mut(|tp| {
                tp.drop(self);
            })
        }
    }
}

impl Clone for ATerm {
    fn clone(&self) -> Self {
        self.borrow().protect()
    }
}

impl From<UniquePtr<ffi::aterm>> for ATerm {
    fn from(value: UniquePtr<ffi::aterm>) -> Self {
        THREAD_TERM_POOL.with_borrow_mut(|tp| {
            unsafe { tp.protect(ffi::aterm_address(&value)) }
        })
    }
}

impl From<&ffi::aterm> for ATerm {
    fn from(value: &ffi::aterm) -> Self {
        THREAD_TERM_POOL.with_borrow_mut(|tp| {
            unsafe { tp.protect(ffi::aterm_address(value)) }
        })
    }
}

impl<'a> From<ATermRef<'a>> for ATerm {
    fn from(value: ATermRef<'a>) -> Self {
        value.protect()
    }
}

impl From<DataVariable> for ATerm {
    fn from(value: DataVariable) -> Self {
        value.term
    }
}

impl From<DataApplication> for ATerm {
    fn from(value: DataApplication) -> Self {
        value.term
    }
}

impl From<DataFunctionSymbol> for ATerm {
    fn from(value: DataFunctionSymbol) -> Self {
        value.term
    }
}

impl<T> From<ATermList<T>> for ATerm {
    fn from(value: ATermList<T>) -> Self {
        value.term
    }
}

impl From<BoolSort> for ATerm {
    fn from(value: BoolSort) -> Self {
        value.term
    }
}

pub struct ATermList<T> {
    term: ATerm,
    _marker: PhantomData<T>,
}

impl<T: From<ATerm>> ATermList<T> {
    /// Obtain the head, i.e. the first element, of the list.
    pub fn head(&self) -> T {
        self.term.arg(0).protect().into()
    }
}

impl<T> ATermList<T> {
    /// Returns true iff the list is empty.
    pub fn is_empty(&self) -> bool {
        self.term.borrow().is_empty_list()
    }

    /// Obtain the tail, i.e. the remainder, of the list.
    pub fn tail(&self) -> ATermList<T> {
        self.term.borrow().arg(1).into()
    }

    /// Returns an iterator over all elements in the list.
    pub fn iter(&self) -> ATermListIter<T> {
        ATermListIter {
            current: self.clone(),
        }
    }
}

impl<T> Clone for ATermList<T> {
    fn clone(&self) -> Self {
        ATermList {
            term: self.term.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: From<ATerm>> Iterator for ATermListIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_empty() {
            None
        } else {
            let head = self.current.head();
            self.current = self.current.tail();
            Some(head)
        }
    }
}

impl<T> From<ATerm> for ATermList<T> {
    fn from(value: ATerm) -> Self {
        debug_assert!(value.borrow().is_list(), "Can only convert a aterm_list");
        ATermList::<T> {
            term: value,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> From<ATermRef<'a>> for ATermList<T> {
    fn from(value: ATermRef<'a>) -> Self {
        debug_assert!(value.is_list(), "Can only convert a aterm_list");
        ATermList::<T> {
            term: value.into(),
            _marker: PhantomData,
        }
    }
}

#[derive(Default)]
pub struct ATermArgs<'a> {
    term: ATermRef<'a>,
    arity: usize,
    index: usize,
}

impl<'a> ATermArgs<'a> {
    fn new(term: ATermRef<'a>) -> ATermArgs {
        let arity = term.get_head_symbol().arity();
        ATermArgs {
            term,
            arity,
            index: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.arity == 0
    }
}

impl<'a> Iterator for ATermArgs<'a> {
    type Item = ATermRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.arity {
            unsafe {
                let res = Some(self.term.arg(self.index).upgrade());
                self.index += 1;
                res
            }
        } else {
            None
        }
    }
}

impl<'a> DoubleEndedIterator for ATermArgs<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < self.arity {
            unsafe {
                let res = Some(self.term.arg(self.arity - 1).upgrade());
                self.arity -= 1;
                res
            }
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for ATermArgs<'a> {
    fn len(&self) -> usize {
        self.arity - self.index
    }
}

pub struct ATermListIter<T> {
    current: ATermList<T>,
}

/// An iterator over all subterms of the given [ATerm].
pub struct TermIterator {
    queue: VecDeque<ATerm>,
}

impl TermIterator {
    pub fn new(t: ATerm) -> TermIterator {
        TermIterator {
            queue: VecDeque::from([t]),
        }
    }
}

impl Iterator for TermIterator {
    type Item = ATerm;

    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.is_empty() {
            None
        } else {
            // Get a subterm to inspect
            let term = self.queue.pop_back().unwrap();

            // Put subterms in the queue
            for argument in term.arguments().rev() {
                self.queue.push_back(argument.protect());
            }

            Some(term)
        }
    }
}

/// TODO: These might be derivable
impl Hash for ATerm {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.borrow().hash(state)
    }
}

impl PartialEq for ATerm {
    fn eq(&self, other: &Self) -> bool {
        self.borrow().eq(&other.borrow())
    }
}

impl PartialOrd for ATerm {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.borrow().cmp(&other.borrow()))
    }
}

impl Ord for ATerm {
    fn cmp(&self, other: &Self) -> Ordering {
        self.borrow().cmp(&other.borrow())
    }
}

impl Eq for ATerm {}

impl<'a> ATermTrait<'a> for ATerm {
    fn arg(&self, index: usize) -> ATermRef {
        debug_assert!(
            index < self.get_head_symbol().arity(),
            "arg({index}) is not defined for term {:?}",
            self
        );

        self.require_valid();
        unsafe {
            ATermRef {
                term: ffi::get_term_argument(self.term, index),
                marker: PhantomData,
            }
        }
    }

    fn arguments(&self) -> ATermArgs {
        ATermArgs::new(self.borrow())
    }

    fn is_default(&self) -> bool {
        self.term.is_null()
    }

    fn is_list(&self) -> bool {
        self.borrow().is_list()
    }

    fn is_empty_list(&self) -> bool {
        self.borrow().is_empty_list()
    }

    fn is_int(&self) -> bool {
        self.borrow().is_int()
    }

    fn get_head_symbol(&'a self) -> SymbolRef<'a> {
        self.require_valid();

        unsafe { ffi::get_aterm_function_symbol(self.term).into() }
    }

    fn iter(&self) -> TermIterator {
        self.borrow().iter()
    }

    fn is_data_variable(&self) -> bool {
        self.borrow().is_data_variable()
    }

    fn is_data_function_symbol(&self) -> bool {
        self.borrow().is_data_function_symbol()
    }

    fn is_data_where_clause(&self) -> bool {
        self.borrow().is_data_where_clause()
    }

    fn is_data_abstraction(&self) -> bool {
        self.borrow().is_data_abstraction()
    }

    fn is_data_untyped_identifier(&self) -> bool {
        self.borrow().is_data_untyped_identifier()
    }

    fn require_valid(&self) {
        assert!(!self.is_default(), "Requires valid term");
    }
}

impl fmt::Display for ATerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.borrow())
    }
}

impl fmt::Debug for ATerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.borrow())
    }
}
