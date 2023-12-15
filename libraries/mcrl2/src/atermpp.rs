use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::{collections::VecDeque, fmt};

use mcrl2_sys::{atermpp::ffi, cxx::{Exception, UniquePtr}};

use crate::data::{BoolSort, DataApplication, DataFunctionSymbol, DataVariable};
use crate::symbol::{SymbolTrait, Symbol, SymbolRef};


/// Rust interface of a atermpp::aterm

pub trait ATermTrait<'a> {
    
    /// Returns the indexed argument of the term
    fn arg<'b: 'a>(&'a self, index: usize) -> ATermRef<'b>;

    /// Returns the list of arguments as a collection
    fn arguments(&'a self) -> ATermArgs;

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
    fn iter(&self) -> TermIterator;

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
    marker: PhantomData<&'a ()>
}

impl<'a> Default for ATermRef<'a> {
    fn default() -> Self {
        ATermRef { term: std::ptr::null(), marker: PhantomData::default() }
    }
}

impl<'a> ATermRef<'a> {
    pub fn protect(&self) -> ATerm { 
        unsafe {
            ATerm { term: Some(ffi::protect_aterm(self.term)) }
        }
    }
}

impl<'a> ATermRef<'a> {
    pub fn borrow<'b: 'a>(&self) -> ATermRef<'b> {
        ATermRef { term: self.term, marker: PhantomData::default() }
    }
}

impl<'a> ATermTrait<'a> for ATermRef<'a> {
    fn arg<'b: 'a>(&self, index: usize) -> ATermRef<'b> {
        self.require_valid();
        debug_assert!(
            index < self.get_head_symbol().arity(),
            "arg({index}) is not defined for term {:?}",
            self
        );

        unsafe {
            ATermRef {
                term: ffi::get_term_argument(self.term, index),
                marker: PhantomData::default(),
            }
        }
    }

    fn arguments(&self) -> ATermArgs {
        self.require_valid();

        ATermArgs::new(
            self.borrow()
        )
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
        unsafe {
            ffi::get_aterm_function_symbol(self.term).into()
        }
    }

    fn iter(&self) -> TermIterator {
        TermIterator::new(self.protect())
    }

    fn is_data_variable(&self) -> bool {
        self.require_valid();
        unsafe {
        ffi::is_data_variable(self.term)
        }
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


impl<'a> From<ATermRef<'a>> for ATerm {
    fn from(value: ATermRef<'a>) -> Self {
        value.protect()    
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
    pub(crate) term: Option<UniquePtr<ffi::aterm>>,
}

impl<'a> ATerm {
    pub fn borrow(&'a self) -> ATermRef<'a> {
        match &self.term {
            Some(t) => {
                ATermRef {
                    term: ffi::aterm_address(&t),
                    marker: PhantomData::default()
                }   
            },
            None => {
                ATermRef::default()
            }
        }     
    }
}

impl Default for ATerm {
    fn default() -> Self {
        ATerm { term: None }
    }
}

impl Clone for ATerm {
    fn clone(&self) -> Self {
        self.borrow().protect()
    }
}

impl From<UniquePtr<ffi::aterm>> for ATerm {
    fn from(value: UniquePtr<ffi::aterm>) -> Self {
        ATerm { term: Some(value) }
    }
}

impl From<&ffi::aterm> for ATerm {
    fn from(value: &ffi::aterm) -> Self {
        unsafe {
            ATerm { term: Some(ffi::protect_aterm(ffi::aterm_address(value))) }
        }
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

impl<'a, T: From<ATerm>> Iterator for ATermListIter<T> {
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

/// This is the thread local term pool.
pub struct TermPool {
    arguments: Vec<*const ffi::_aterm>,
    data_appl: Vec<Symbol>, // Function symbols to represent 'DataAppl' with any number of arguments.
}

impl TermPool {
    pub fn new() -> TermPool {
        // Initialise the C++ aterm library.
        ffi::initialise();

        TermPool {
            arguments: vec![],
            data_appl: vec![],
        }
    }

    /// Trigger a garbage collection
    pub fn collect(&mut self) {
        ffi::collect_garbage();
    }

    /// Print performance metrics
    pub fn print_metrics(&self) {
        ffi::print_metrics();
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

        unsafe {
            ATerm {
                term: Some(ffi::create_aterm(symbol.address(), arguments)),
            }
        }
    }

    /// Creates a function symbol with the given name and arity.
    pub fn create_symbol(&mut self, name: &str, arity: usize) -> Symbol {
        ffi::create_function_symbol(String::from(name), arity).into()
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

        let symbol = self.data_appl[arguments.len() + 1].clone();
        let term = self.create_head(&symbol, head, arguments);

        DataApplication { term }
    }

    pub fn create_variable(&mut self, name: &str) -> DataVariable {
        DataVariable {
            term: ATerm::from(ffi::create_data_variable(String::from(name))),
        }
    }

    pub fn create_data_function_symbol(&mut self, name: &str) -> DataFunctionSymbol {
        DataFunctionSymbol {
            term: ATerm::from(ffi::create_data_function_symbol(String::from(name))),
        }
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
    fn create_head(&mut self, symbol: &impl SymbolTrait, head: &ATerm, arguments: &[ATerm]) -> ATerm {
        let arguments = self.tmp_arguments_head(head, arguments);

        debug_assert_eq!(
            symbol.arity(),
            arguments.len(),
            "Number of arguments does not match arity"
        );

        unsafe {
            ATerm {
                term: Some(ffi::create_aterm(symbol.address(), arguments)),
            }
        }
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

impl Drop for TermPool {
    fn drop(&mut self) {
        self.print_metrics();
    }
}


pub struct ATermArgs<'a> {
    term: ATermRef<'a>,
    arity: usize,
    index: usize
}

impl<'a> ATermArgs<'a> {
    fn new(term: ATermRef<'a>) -> ATermArgs {
        let arity = term.get_head_symbol().arity();
        ATermArgs {
            term,
            arity,
            index: 0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.arity == 0
    }
}

impl<'a> Iterator for ATermArgs<'a> {
    type Item = ATerm;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.arity {
            let res = Some(self.term.arg(self.index).protect());     
            self.index += 1;
            res
        } else {
            None
        }
    }
}

impl<'a> DoubleEndedIterator for ATermArgs<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < self.arity {
            let res = Some(self.term.arg(self.arity - 1).protect());
            self.arity -= 1;
            res
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for ATermArgs<'a> {
    fn len(&self) -> usize {
        self.arity
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
                self.queue.push_back(argument);
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
        self.borrow().partial_cmp(&other.borrow())
    }
}

impl Ord for ATerm {
    fn cmp(&self, other: &Self) -> Ordering {
        self.borrow().cmp(&other.borrow())
    }
}

impl Eq for ATerm {}


impl<'a> ATermTrait<'a> for ATerm {
    fn arg<'b: 'a>(&'a self, index: usize) -> ATermRef<'b> {
        debug_assert!(
            index < self.get_head_symbol().arity(),
            "arg({index}) is not defined for term {:?}",
            self
        );
        
        match self.term.as_ref() {
            Some(t) => {        
                unsafe {
                    ATermRef {
                        term: ffi::get_term_argument(ffi::aterm_address(&t), index),
                        marker: PhantomData::default(),
                    }
                }
            }
            None => {
                panic!("Requires valid term");
            }
        }
    }

    fn arguments(&'a self) -> ATermArgs {
        ATermArgs::new(
            self.borrow()
        )
    }

    fn is_default(&self) -> bool {
        self.term.is_none()
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
        match self.term.as_ref() {
            Some(t) => {
                unsafe {
                    ffi::get_aterm_function_symbol(ffi::aterm_address(&t)).into()
                }
            }
            None => {
                panic!("Requires valid term");
            }
        }
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
        assert!(!self.term.is_none(), "Requires valid term");
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
