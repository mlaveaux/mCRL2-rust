use std::borrow::Borrow;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::{collections::VecDeque, fmt};

use mcrl2_sys::{atermpp::ffi, cxx::UniquePtr};

use crate::aterm::{THREAD_TERM_POOL, SymbolRef};

use super::global_aterm_pool::GLOBAL_TERM_POOL;

/// This represents a lifetime bound reference to an existing ATerm that is
/// protected somewhere. Can be 'static if the term is protected in a container.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ATermRef<'a> {
    term: *const ffi::_aterm,
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
    
    /// Protects the reference on the thread local protection pool.
    pub fn protect(&self) -> ATerm {
        if self.is_default() {
            ATerm::default()
        } else {
            THREAD_TERM_POOL.with_borrow_mut(|tp| { tp.protect(self.term) })
        }
    }
    
    /// Protects the reference on the global protection pool.
    pub fn protect_global(&self) -> ATermGlobal {
        if self.is_default() {
            ATermGlobal::default()
        } else {
            GLOBAL_TERM_POOL.lock().protect(self.term)
        }
    }

    /// This allows us to extend our borrowed lifetime from 'a to 'b based on
    /// existing parent term called `witness` which lives longer than us.
    /// 
    /// The main usecase is to establish transitive lifetimes. For example given
    /// a term t from which we borrow `u = t.arg(0)` then we cannot have
    /// u.arg(0) live as long as t since the intermediate temporary u is
    /// dropped. However, since we know that u.arg(0) is a subterm of `t` we can
    /// upgrade its lifetime to the lifetime of `t` using this function.
    /// 
    /// # Safety
    /// 
    /// This function might only be used if witness is a parent term of the
    /// current term.
    pub fn upgrade<'b: 'a>(&'a self, parent: &ATermRef<'b>) -> ATermRef<'b> {
        debug_assert!(parent.iter().any(|t| t.copy() == *self), "Upgrade has been used on a witness that is not a parent term");

        ATermRef::new(self.term)
    }

    /// A local unchecked version of [`ATermRef::upgrade`] since the above one uses the iterators.
    unsafe fn upgrade_unchecked<'b: 'a>(&'a self, _parent: &ATermRef<'b>) -> ATermRef<'b> {
        ATermRef::new(self.term)
    }

    /// Obtains the underlying pointer
    /// 
    /// # Safety 
    /// Should not be modified in any way.
    pub(crate) unsafe fn get(&self) -> *const ffi::_aterm {
        self.term
    }
}

impl<'a> ATermRef<'a> {
    pub(crate) fn new(term: *const ffi::_aterm) -> ATermRef<'a> {
        ATermRef {
            term,
            marker: PhantomData,
        }
    }
}

impl ATermRef<'_> {
    /// Returns the indexed argument of the term
    pub fn arg(&self, index: usize) -> ATermRef<'_> {
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

    /// Returns the list of arguments as a collection
    pub fn arguments(&self) -> ATermArgs<'_> {
        self.require_valid();

        ATermArgs::new(self.copy())
    }

    /// Makes a copy of the term with the same lifetime as itself.
    pub fn copy(&self) -> ATermRef<'_> {
        ATermRef::new(self.term)
    }

    /// Returns whether the term is the default term (not initialised)
    pub fn is_default(&self) -> bool {
        self.term.is_null()
    }

    /// Returns true iff this is an aterm_list
    pub fn is_list(&self) -> bool {
        unsafe { ffi::aterm_is_list(self.term) }
    }

    /// Returns true iff this is the empty aterm_list
    pub fn is_empty_list(&self) -> bool {
        unsafe { ffi::aterm_is_empty_list(self.term) }
    }

    /// Returns true iff this is a aterm_int
    pub fn is_int(&self) -> bool {
        unsafe { ffi::aterm_is_int(self.term) }
    }

    /// Returns the head function symbol of the term.
    pub fn get_head_symbol(&self) -> SymbolRef<'_> {
        self.require_valid();
        unsafe { ffi::get_aterm_function_symbol(self.term).into() }
    }
    
    /// Returns an iterator over all arguments of the term that runs in pre order traversal of the term trees.
    pub fn iter(&self) -> TermIterator<'_> {
        TermIterator::new(self.copy())
    }

    /// Panics if the term is default
    pub fn require_valid(&self) {
        debug_assert!(
            !self.is_default(),
            "This function can only be called on valid terms, i.e., not default terms"
        );
    }
}

impl<'a> fmt::Display for ATermRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.require_valid();        
        write!(f, "{:?}", self)
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
        }

        Ok(())
    }
}

/// The protected version of [ATermRef], mostly derived from it.
#[derive(Default)]
pub struct ATerm {
    pub(crate) term: ATermRef<'static>,
    pub(crate) root: usize,
}

impl ATerm {
    /// Obtains the underlying pointer
    /// 
    /// # Safety 
    /// Should not be modified in any way.
    pub(crate) unsafe fn get(&self) -> *const ffi::_aterm {
        self.term.get()
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
        self.copy().protect()
    }
}

impl Deref for ATerm {
    type Target = ATermRef<'static>;

    fn deref(&self) -> &Self::Target {
        &self.term        
    }
}

impl<'a> Borrow<ATermRef<'a>> for ATerm {
    fn borrow(&self) -> &ATermRef<'a> {
        &self.term
    }
}

impl fmt::Debug for ATerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.copy())
    }
}

impl Hash for ATerm {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.term.hash(state)
    }
}

impl PartialEq for ATerm {
    fn eq(&self, other: &Self) -> bool {
        self.term.eq(&other.term)
    }
}

impl PartialOrd for ATerm {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.term.cmp(&other.term))
    }
}

impl Ord for ATerm {
    fn cmp(&self, other: &Self) -> Ordering {
        self.term.cmp(&other.term)
    }
}

impl Eq for ATerm {}

// Some convenient conversions.
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

impl<T> From<ATermList<T>> for ATerm {
    fn from(value: ATermList<T>) -> Self {
        value.term
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
        debug_assert!(value.term.is_list(), "Can only convert a aterm_list");
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
            term: value.protect(),
            _marker: PhantomData,
        }
    }
}

impl<T: From<ATerm>> IntoIterator for ATermList<T> {
    type IntoIter = ATermListIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: From<ATerm>> IntoIterator for &ATermList<T> {
    type IntoIter = ATermListIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// The same as [ATerm] but protected on the global protection set. This allows
/// the term to be Send and Sync among threads.
#[derive(Default)]
pub struct ATermGlobal {
    pub(crate) term: ATermRef<'static>,
    pub(crate) root: usize,
}

impl Drop for ATermGlobal {
    fn drop(&mut self) {
        if !self.is_default() {
            GLOBAL_TERM_POOL.lock().drop_term(self);
        }
    }
}

impl Clone for ATermGlobal {
    fn clone(&self) -> Self {
        self.copy().protect_global()
    }
}


/// An iterator over the arguments of a term.
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
            let res = unsafe {
               Some(self.term.arg(self.index).upgrade_unchecked(&self.term))
            };

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
            let res = unsafe {
                Some(self.term.arg(self.arity - 1).upgrade_unchecked(&self.term))
            };

            self.arity -= 1;
            res
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

/// An iterator over all subterms of the given [ATerm] in preorder traversal, i.e.,
/// for f(g(a), b) we visit f(g(a), b), g(a), a, b.
pub struct TermIterator<'a> {
    queue: VecDeque<ATermRef<'a>>,
}

impl TermIterator<'_> {
    pub fn new(t: ATermRef) -> TermIterator {
        TermIterator {
            queue: VecDeque::from([t]),
        }
    }
}

impl<'a> Iterator for TermIterator<'a> {
    type Item = ATermRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.queue.pop_back() {
            Some(term) => {
                // Put subterms in the queue
                for argument in term.arguments().rev() {
                    unsafe {
                        self.queue.push_back(argument.upgrade_unchecked(&term));
                    }
                }
    
                Some(term)
            },
            None => {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use crate::aterm::TermPool;

    use super::*;

    #[test]
    fn test_term_iterator() {
        let mut tp = TermPool::new();
        let t = tp.from_string("f(g(a),b)").unwrap();

        let mut result = t.iter();
        assert_eq!(result.next().unwrap(), tp.from_string("f(g(a),b)").unwrap().copy());
        assert_eq!(result.next().unwrap(), tp.from_string("g(a)").unwrap().copy());
        assert_eq!(result.next().unwrap(), tp.from_string("a").unwrap().copy());
        assert_eq!(result.next().unwrap(), tp.from_string("b").unwrap().copy());
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

impl fmt::Display for ATerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.term)
    }
}

impl Deref for ATermGlobal {
    type Target = ATermRef<'static>;

    fn deref(&self) -> &Self::Target {
        &self.term        
    }
}

impl Hash for ATermGlobal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.term.hash(state)
    }
}

impl PartialEq for ATermGlobal {
    fn eq(&self, other: &Self) -> bool {
        self.term.eq(&other.term)
    }
}

impl PartialOrd for ATermGlobal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.term.cmp(&other.term))
    }
}

impl Ord for ATermGlobal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.term.cmp(&other.term)
    }
}

impl Eq for ATermGlobal {}

impl fmt::Display for ATermGlobal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.copy())
    }
}

impl fmt::Debug for ATermGlobal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.copy())
    }
}