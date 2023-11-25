use std::cmp::Ordering;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::{collections::VecDeque, fmt};

use ahash::AHashSet;
use mcrl2_sys::{atermpp::ffi, cxx::{Exception, UniquePtr}};

use crate::data::{BoolSort, DataApplication, DataFunctionSymbol, DataVariable};

/// A Symbol references to an aterm function symbol, which has a name and an arity.
pub struct Symbol {
    function: UniquePtr<ffi::function_symbol>,
}

impl Symbol {
    pub fn name(&self) -> &str {
        ffi::get_function_symbol_name(&self.function)
    }

    pub fn arity(&self) -> usize {
        ffi::get_function_symbol_arity(&self.function)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if true {
            write!(f, "{}", self.name())
        } else {
            write!(
                f,
                "{}:{} [{}]",
                self.name(),
                self.arity(),
                ffi::function_symbol_address(&self.function)
            )
        }
    }
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(ffi::hash_function_symbol(&self.function));
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        ffi::equal_function_symbols(&self.function, &other.function)
    }
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        if ffi::less_function_symbols(&self.function, &other.function) {
            Ordering::Less
        } else if self == other {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }
}

impl Clone for Symbol {
    fn clone(&self) -> Self {
        Symbol {
            function: ffi::copy_function_symbol(&self.function),
        }
    }
}

impl Eq for Symbol {}

/// Rust representation of a atermpp::aterm
pub struct ATerm {
    pub(crate) term: UniquePtr<ffi::aterm>,
}

impl ATerm {
    /// Returns the indexed argument of the term
    pub fn arg(&self, index: usize) -> ATerm {
        self.require_valid();
        debug_assert!(
            index < self.get_head_symbol().arity(),
            "arg({index}) is not defined for term {:?}",
            self
        );

        ATerm {
            term: ffi::get_term_argument(&self.term, index),
        }
    }

    /// Returns the list of arguments.
    pub fn arguments(&self) -> Vec<ATerm> {
        self.require_valid();
        let mut result = vec![];
        for i in 0..self.get_head_symbol().arity() {
            result.push(self.arg(i));
        }
        result
    }

    /// Returns whether the term is the default term (not initialised)
    pub fn is_default(&self) -> bool {
        ffi::aterm_pointer(&self.term) == 0
    }

    /// Returns true iff this is an aterm_list
    pub fn is_list(&self) -> bool {
        ffi::aterm_is_list(&self.term)
    }

    /// Returns true iff this is the empty aterm_list
    pub fn is_empty_list(&self) -> bool {
        ffi::aterm_is_empty_list(&self.term)
    }

    /// Returns true iff this is a aterm_int
    pub fn is_int(&self) -> bool {
        ffi::aterm_is_int(&self.term)
    }

    /// Returns the head function symbol of the term.
    pub fn get_head_symbol(&self) -> Symbol {
        self.require_valid();
        Symbol {
            function: ffi::get_aterm_function_symbol(&self.term),
        }
    }

    /// Returns an iterator over all arguments of the term that runs in pre order traversal of the term trees.
    pub fn iter(&self) -> TermIterator {
        TermIterator::new(self.clone())
    }

    /// Returns true iff the term is not default.
    fn require_valid(&self) {
        debug_assert!(
            !self.is_default(),
            "This function can only be called on valid terms, i.e., not default terms"
        );
    }

    /// Get access to the underlying term
    pub(crate) fn get(&self) -> &ffi::aterm {
        self.require_valid();
        &self.term
    }

    // Recognizers for the data library

    /// Returns true iff this is a data::variable
    pub fn is_data_variable(&self) -> bool {
        self.require_valid();
        ffi::is_data_variable(&self.term)
    }

    /// Returns true iff this is a data::function_symbol
    pub fn is_data_function_symbol(&self) -> bool {
        self.require_valid();
        ffi::is_data_function_symbol(&self.term)
    }

    pub fn is_data_where_clause(&self) -> bool {
        self.require_valid();
        ffi::is_data_where_clause(&self.term)
    }

    pub fn is_data_abstraction(&self) -> bool {
        self.require_valid();
        ffi::is_data_abstraction(&self.term)
    }

    pub fn is_data_untyped_identifier(&self) -> bool {
        self.require_valid();
        ffi::is_data_untyped_identifier(&self.term)
    }
}

impl Default for ATerm {
    fn default() -> Self {
        ATerm {
            term: ffi::new_aterm(),
        }
    }
}

impl From<UniquePtr<ffi::aterm>> for ATerm {
    fn from(value: UniquePtr<ffi::aterm>) -> Self {
        ATerm { term: value }
    }
}

impl From<&ffi::aterm> for ATerm {
    fn from(value: &ffi::aterm) -> Self {
        ATerm {
            term: ffi::protect_aterm(value),
        }
    }
}

impl fmt::Display for ATerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_default() {
            write!(f, "{:?}", self)
        } else if self.is_data_function_symbol() {
            write!(
                f,
                "{}",
                <ATerm as Into<DataFunctionSymbol>>::into(self.clone())
            )
        // } else if self.is_data_application() {
        //     write!(
        //         f,
        //         "{}",
        //         <ATerm as Into<DataApplication>>::into(self.clone())
        //     )
        } else if self.is_data_variable() {
            write!(f, "{}", <ATerm as Into<DataVariable>>::into(self.clone()))
        } else {
            write!(f, "{:?}", self)
        }
    }
}

impl fmt::Debug for ATerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_default() {
            write!(f, "<default>")?;
        } else {
            write!(f, "{}", ffi::print_aterm(&self.term))?;
            //for term in self.iter() {
            //   write!(f, "{:?}: [{}]", term.get_head_symbol(), ffi::aterm_pointer(&self.term))?;
            //}
        }

        Ok(())
    }
}

impl Hash for ATerm {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(ffi::hash_aterm(&self.term));
    }
}

impl PartialEq for ATerm {
    fn eq(&self, other: &Self) -> bool {
        ffi::equal_aterm(&self.term, &other.term)
    }
}

impl PartialOrd for ATerm {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ATerm {
    fn cmp(&self, other: &Self) -> Ordering {
        if ffi::less_aterm(&self.term, &other.term) {
            Ordering::Less
        } else if self == other {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }
}

impl Clone for ATerm {
    fn clone(&self) -> Self {
        ATerm {
            term: ffi::copy_aterm(&self.term),
        }
    }
}

impl Eq for ATerm {}

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
        self.term.arg(0).into()
    }
}

impl<T> ATermList<T> {
    /// Returns true iff the list is empty.
    pub fn is_empty(&self) -> bool {
        self.term.is_empty_list()
    }

    /// Obtain the tail, i.e. the remainder, of the list.
    pub fn tail(&self) -> ATermList<T> {
        self.term.arg(1).into()
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

pub struct ATermListIter<T> {
    current: ATermList<T>,
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
        debug_assert!(value.is_list(), "Can only convert a aterm_list");
        ATermList::<T> {
            term: value,
            _marker: PhantomData,
        }
    }
}

/// This is the thread local term pool.
pub struct TermPool {
    arguments: Vec<ffi::aterm_ref>,
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

    /// Creates an ATerm from a string.
    pub fn from_string(&mut self, text: &str) -> Result<ATerm, Exception> {
        match ffi::aterm_from_string(String::from(text)) {
            Ok(term) => Ok(ATerm { term }),
            Err(exception) => Err(exception),
        }
    }

    /// Creates an [ATerm] with the given symbol and arguments.
    pub fn create(&mut self, symbol: &Symbol, arguments: &[ATerm]) -> ATerm {
        let arguments = self.tmp_arguments(arguments);

        debug_assert_eq!(
            symbol.arity(),
            arguments.len(),
            "Number of arguments does not match arity"
        );

        ATerm {
            term: ffi::create_aterm(&symbol.function, arguments),
        }
    }

    /// Creates a function symbol with the given name and arity.
    pub fn create_symbol(&mut self, name: &str, arity: usize) -> Symbol {
        Symbol {
            function: ffi::create_function_symbol(String::from(name), arity),
        }
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

        let symbol = &self.data_appl[arguments.len() + 1].clone();
        let term = self.create_head(symbol, head, arguments);

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
    pub fn is_data_application(&mut self, term: &ATerm) -> bool {
        term.require_valid();       
        
        let symbol = term.get_head_symbol();
        // It can be that data_applications are created without create_data_application in the mcrl2 ffi.
        while self.data_appl.len() <= symbol.arity() {
            let symbol = self.create_symbol("DataAppl", self.data_appl.len());
            self.data_appl.push(symbol);
        }

        symbol == self.data_appl[symbol.arity()]
    }

    /// Creates an [ATerm] with the given symbol, first and other arguments.
    fn create_head(&mut self, symbol: &Symbol, head: &ATerm, arguments: &[ATerm]) -> ATerm {
        let arguments = self.tmp_arguments_head(head, arguments);

        debug_assert_eq!(
            symbol.arity(),
            arguments.len(),
            "Number of arguments does not match arity"
        );

        ATerm {
            term: ffi::create_aterm(&symbol.function, arguments),
        }
    }

    /// Converts the [ATerm] slice into a [ffi::aterm_ref] slice.
    fn tmp_arguments(&mut self, arguments: &[ATerm]) -> &[ffi::aterm_ref] {
        // Make the temp vector sufficient length.
        while self.arguments.len() < arguments.len() {
            self.arguments.push(ffi::aterm_ref { index: 0 });
        }

        self.arguments.clear();
        for arg in arguments {
            self.arguments.push(ffi::aterm_ref {
                index: ffi::aterm_pointer(arg.get()),
            });
        }

        &self.arguments
    }

    /// Converts the [ATerm] slice into a [ffi::aterm_ref] slice.
    fn tmp_arguments_head(&mut self, head: &ATerm, arguments: &[ATerm]) -> &[ffi::aterm_ref] {
        // Make the temp vector sufficient length.
        while self.arguments.len() < arguments.len() + 1 {
            self.arguments.push(ffi::aterm_ref { index: 0 });
        }

        self.arguments.clear();
        self.arguments.push(ffi::aterm_ref {
            index: ffi::aterm_pointer(head.get()),
        });
        for arg in arguments {
            self.arguments.push(ffi::aterm_ref {
                index: ffi::aterm_pointer(arg.get()),
            });
        }

        &self.arguments
    }
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
            for argument in term.arguments().iter().rev() {
                self.queue.push_back(argument.clone());
            }

            Some(term)
        }
    }
}

impl Default for TermPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Can be used to construct a term using two functions. Type I is used for the
/// input of transformer function, and C the type for the construct function.
#[derive(Default)]
pub struct TermBuilder<I, C> {
    // The stack of terms
    terms: Vec<ATerm>,
    configs: Vec<Config<I, C>>,
}

impl<I, C: fmt::Display> fmt::Display for TermBuilder<I, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Terms: [")?;
        for (i, term) in self.terms.iter().enumerate() {
            writeln!(f, "{}\t{}", i, term)?;
        }
        writeln!(f, "]")?;

        writeln!(f, "Configs: [")?;
        for config in &self.configs {
            writeln!(f, "\t{}", config)?;
        }
        write!(f, "]")
    }
}

impl<I, C: fmt::Display> fmt::Display for Config<I, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Config::Apply(_, result) => write!(f, "Apply({})", result),
            Config::Construct(symbol, arity, result) => {
                write!(f, "Construct({}, {}, {})", symbol, arity, result)
            }
        }
    }
}

/// Applies the given function to every subterm of the given term using the TermBuilder.
///     function(subterm) returns:
///         None   , in which case subterm is kept and it is recursed into its argments.
///         Some(x), in which case subterm is replaced by x.
pub fn apply<F>(tp: &mut TermPool, t: &ATerm, function: &F) -> ATerm
where
    F: Fn(&mut TermPool, &ATerm) -> Option<ATerm>,
{
    let mut builder = TermBuilder::<ATerm, Symbol>::new();

    builder
        .evaluate(
            tp,
            t.clone(),
            |tp, args, t| match function(tp, &t) {
                Some(result) => Ok(Yield::Term(result)),
                None => {
                    for arg in t.arguments() {
                        args.push(arg);
                    }

                    Ok(Yield::Construct(t.get_head_symbol()))
                }
            },
            |tp, symbol, args| Ok(tp.create(&symbol, args)),
        )
        .unwrap()
}

/// This can be used to construct a term from a given input of type (inductive
/// type) I, without using the system stack.
///
/// The `transformer` function is applied to every input I, which can put more
/// more inputs onto the argument stack and some instance C that is used to construct
/// the result term. Or yield a result term directly.
///
/// The `construct` function takes an instance C and the term arguments pushed to stack
/// where the transformer was applied.
///
///
/// A simple example could be to transform a term into another term using a
/// function f : ATerm -> Option<ATerm>. Then I will be ATerm since that is the
/// input, and C will be the Symbol from which we can construct the recursive
/// term.
///
/// `transformer` takes the input and applies f(input). Then either we return
/// Yield(x) when f returns some term, or Construct(head(input)) with the
/// arguments of the input term pushed to stack.
///
/// `construct` simply constructs the term from the symbol and the arguments on
/// the stack.
///
/// However, it can also be that I is some syntax tree from which we want to
/// construct a term.
impl<I, C: fmt::Display> TermBuilder<I, C> {
    pub fn new() -> TermBuilder<I, C> {
        TermBuilder {
            terms: vec![],
            configs: vec![],
        }
    }

    pub fn evaluate<F, G>(
        &mut self,
        tp: &mut TermPool,
        input: I,
        transformer: F,
        construct: G,
    ) -> Result<ATerm, Box<dyn Error>>
    where
        F: Fn(&mut TermPool, &mut ArgStack<I, C>, I) -> Result<Yield<C>, Box<dyn Error>>,
        G: Fn(&mut TermPool, C, &[ATerm]) -> Result<ATerm, Box<dyn Error>>,
    {
        self.terms.push(ATerm::default());
        self.configs.push(Config::Apply(input, 0));

        while let Some(config) = self.configs.pop() {
            match config {
                Config::Apply(input, result) => {
                    // Applies the given function to this input, and obtain a number of symbol and arguments.
                    let top_of_stack = self.configs.len();
                    let mut args = ArgStack::new(&mut self.terms, &mut self.configs);

                    match transformer(tp, &mut args, input)? {
                        Yield::Construct(input) => {
                            // This occurs before the other constructs.
                            let arity = args.len();
                            self.configs.reserve(1);
                            self.configs
                                .insert(top_of_stack, Config::Construct(input, arity, result));
                        }
                        Yield::Term(term) => {
                            self.terms[result] = term;
                        }
                    }
                }
                Config::Construct(input, arity, result) => {
                    let arguments = &self.terms[self.terms.len() - arity..];

                    self.terms[result] = construct(tp, input, arguments)?;

                    // Remove elements from the stack.
                    self.terms.drain(self.terms.len() - arity..);
                }
            }

            //println!("{}", self);
        }

        debug_assert!(
            self.terms.len() == 1,
            "Expect exactly one term on the result stack"
        );

        Ok(self
            .terms
            .pop()
            .expect("There should be at last one result"))
    }
}

enum Config<I, C> {
    Apply(I, usize),
    Construct(C, usize, usize),
}

pub enum Yield<C> {
    Term(ATerm),  // Yield this term as is.
    Construct(C), // Yield f(args) for every arg push to the argument stack, with the function applied to it.
}

/// This struct defines a local argument stack on the global stack.
pub struct ArgStack<'a, I, C> {
    terms: &'a mut Vec<ATerm>,
    configs: &'a mut Vec<Config<I, C>>,
    top_of_stack: usize,
}

impl<'a, I, C> ArgStack<'a, I, C> {
    fn new(terms: &'a mut Vec<ATerm>, configs: &'a mut Vec<Config<I, C>>) -> ArgStack<'a, I, C> {
        let top_of_stack = terms.len();
        ArgStack {
            terms,
            configs,
            top_of_stack,
        }
    }

    /// Returns the amount of arguments added.
    fn len(&self) -> usize {
        self.terms.len() - self.top_of_stack
    }

    /// Adds the term to the argument stack, will construct construct(C, args...) with the transformer applied to arguments.
    pub fn push(&mut self, input: I) {
        self.configs.push(Config::Apply(input, self.terms.len()));
        self.terms.push(ATerm::default());
    }
}

/// Create a random term consisting of the given symbol and constants. Performs
/// iterations number of constructions, and uses chance_duplicates to choose the
/// amount of subterms that are duplicated.
pub fn random_term(
    tp: &mut TermPool,
    symbols: &[(String, usize)],
    constants: &[String],
    iterations: usize,
) -> ATerm {
    use rand::prelude::IteratorRandom;

    debug_assert!(
        !constants.is_empty(),
        "We need constants to be able to create a term"
    );

    let mut subterms = AHashSet::<ATerm>::from_iter(constants.iter().map(|name| {
        let symbol = tp.create_symbol(name, 0);
        tp.create(&symbol, &[])
    }));

    let mut rng = rand::thread_rng();
    let mut result = ATerm::default();
    for _ in 0..iterations {
        let (symbol, arity) = symbols.iter().choose(&mut rng).unwrap();

        let mut arguments = vec![];
        for _ in 0..*arity {
            arguments.push(subterms.iter().choose(&mut rng).unwrap().clone());
        }

        let symbol = tp.create_symbol(symbol, *arity);
        result = tp.create(&symbol, &arguments);

        // Make this term available as another subterm that can be used.
        subterms.insert(result.clone());
    }

    result
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

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
    fn test_term_iterator() {
        let mut tp = TermPool::new();
        let t = tp.from_string("f(g(a),b)").unwrap();

        let mut result = t.iter();
        assert_eq!(result.next().unwrap(), tp.from_string("f(g(a),b)").unwrap());
        assert_eq!(result.next().unwrap(), tp.from_string("g(a)").unwrap());
        assert_eq!(result.next().unwrap(), tp.from_string("a").unwrap());
        assert_eq!(result.next().unwrap(), tp.from_string("b").unwrap());
    }

    #[test]
    fn test_thread_aterm_pool_parallel() {
        let mut threads = vec![];

        for _ in 0..20 {
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
    }

    #[test]
    fn test_aterm_list() {
        let mut tp = TermPool::new();
        let list: ATermList<ATerm> = tp.from_string("[f,g,h,i]").unwrap().into();

        assert!(!list.is_empty());

        // Convert into normal vector.
        let values: Vec<ATerm> = list.iter().collect();

        assert_eq!(values[0], tp.from_string("f").unwrap());
        assert_eq!(values[1], tp.from_string("g").unwrap());
        assert_eq!(values[2], tp.from_string("h").unwrap());
        assert_eq!(values[3], tp.from_string("i").unwrap());
    }
}
