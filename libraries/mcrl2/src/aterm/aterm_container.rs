use std::{pin::Pin, sync::Arc, mem::transmute, marker::PhantomData, ops::{DerefMut, Deref}, hash::Hash, fmt::Debug, cell::RefCell};

use mcrl2_sys::atermpp::ffi;

use crate::aterm::{ATermRef, BfTermPool, BfTermPoolThreadWrite, THREAD_TERM_POOL};

use super::{BfTermPoolRead, ATermTrait};

/// A container of objects, typically either terms or objects containing terms,
/// that are of trait Markable. These store ATermRef<'static> that are protected
/// during garbage collection by being in the container itself.
pub struct Protected<C> {
    container: Arc<BfTermPool<C>>,
    root: usize,
    marker: PhantomData<C>,
}

impl<C: Markable + Send +'static> Protected<C> {

    pub fn new(container: C) -> Protected<C> {
        let shared = Arc::new(BfTermPool::new(container));

        let root = THREAD_TERM_POOL.with_borrow_mut(|tp| {
            tp.protect_container(shared.clone())
        });

        Protected {
            container: shared,
            root,
            marker: Default::default(),
        }
    }

    /// Provides mutable access to the underlying container. Use [protect] of
    /// the resulting guard to be able to insert terms into the container.
    /// Otherwise the borrow checker will note that the [ATermRef] do not
    /// outlive the guard, see [TermProtection].
    pub fn write(&mut self) -> Protector<'_, C> {
        // The lifetime of ATermRef can be derived from self since it is protected by self, so transmute 'static into 'a.
        unsafe {
            Protector::new(transmute(self.container.write_exclusive(true)))
        }
    }

    /// Provides immutable access to the underlying container.
    pub fn read(&self) -> BfTermPoolRead<'_, C> {
        // The lifetime of ATermRef can be derived from self since it is protected by self, so transmute 'static into 'a.
        unsafe {
            transmute(self.container.read())
        }
    }

}

impl<C: Default + Markable + Send + 'static> Default for Protected<C> {
    fn default() -> Self {
        Protected::new(Default::default())
    }
}

impl<C: Clone + Markable + Send + 'static> Clone for Protected<C> {
    fn clone(&self) -> Self {
        Protected::new(self.container.read().clone())
    }
}

impl<C: Hash + Markable> Hash for Protected<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.container.read().hash(state)
    }
}

impl<C: PartialEq + Markable> PartialEq for Protected<C> {
    fn eq(&self, other: &Self) -> bool {
        self.container.read().eq(&other.container.read())
    }
}

impl<C: PartialOrd + Markable> PartialOrd for Protected<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let c: &C = &other.container.read();
        self.container.read().partial_cmp(c)
    }
}

impl<C: Debug + Markable> Debug for Protected<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c: &C = &self.container.read();
        write!(f, "{:?}", c)
    }
}

impl<C: Eq + PartialEq + Markable> Eq for Protected<C> {}
impl<C: Ord + PartialEq + PartialOrd + Markable> Ord for Protected<C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let c: &C = &other.container.read();
        self.container.read().partial_cmp(c).unwrap()
    }
}

impl<C> Drop for Protected<C> {

    fn drop(&mut self) {
        THREAD_TERM_POOL.with_borrow_mut(|tp| {
            tp.drop_container(self.root);
        });
    }
}

/// A type for the todo queue.
pub type Todo<'a> = Pin<&'a mut ffi::term_mark_stack>;

/// This trait should be used on all objects and containers related to storing unprotected terms.
pub trait Markable {
    
    /// Marks all the ATermRefs to prevent them from being garbage collected.
    fn mark(&self, todo: Todo);

    /// Should return true iff the given term is contained in the object. Used for runtime checks.
    fn contains_term(&self, term: &ATermRef<'_>) -> bool;

    /// Returns the number of terms in the instance, used to delay garbage collection.
    fn len(&self) -> usize;

    /// Returns true iff the container is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> Markable for ATermRef<'a> {
    fn mark(&self, todo: Pin<&mut ffi::term_mark_stack>) {
        if !self.is_default() {
            unsafe {
                ffi::aterm_mark_address(self.get(), todo);
            }
        }
    }

    fn contains_term(&self, term: &ATermRef<'_>) -> bool {
        term == self
    }

    fn len(&self) -> usize {
        1
    }
}

impl<T: Markable> Markable for Vec<T> {
    fn mark(&self, mut todo: Pin<&mut ffi::term_mark_stack>) {
        for value in self {
            value.mark(todo.as_mut());
        }
    }

    fn contains_term(&self, term: &ATermRef<'_>) -> bool {
        self.iter().any(|v| { v.contains_term(term) })        
    }
    
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T: Markable + ?Sized> Markable for BfTermPool<T> {
    fn mark(&self, mut todo: Pin<&mut ffi::term_mark_stack>) {
        // Marking will done while an exclusive lock is already held, also this
        // does not implement the Markable trait since self must be immutable
        // here.
        unsafe {
            self.write_exclusive(false).mark(todo.as_mut());
        }
    }    

    fn contains_term(&self, term: &ATermRef<'_>) -> bool {
        self.read().contains_term(term)
    }

    fn len(&self) -> usize {
        self.read().len()
    }
}

/// This is a helper struct used by TermContainer to protected terms that are
/// inserted into the container before the guard is dropped.
/// 
/// The reason is that the ATermRef derive their lifetime from the
/// TermContainer. However, when inserting terms with shorter lifetimes we know
/// that their lifetime is extended by being in the container. This is enforced
/// by runtime checks during debug for containers that implement IntoIterator.
pub struct Protector<'a, C: Markable> {
    reference: BfTermPoolThreadWrite<'a, C>,
    
    #[cfg(debug_assertions)]
    protected: RefCell<Vec<ATermRef<'static>>>
}

impl<'a, C: Markable> Protector<'a, C> {
    fn new(reference: BfTermPoolThreadWrite<'a, C>) -> Protector<'_, C> {
        #[cfg(debug_assertions)]
        return Protector {
            reference,
            protected: RefCell::new(vec![]),
        };
        
        #[cfg(not(debug_assertions))]
        return Protector {
            reference,
        }
    }
    
    pub fn protect(&self, term: &ATermRef<'_>) -> ATermRef<'static>{

        unsafe {
            // Store terms that are marked as protected to check if they are
            // actually in the container when the protection is dropped.
            #[cfg(debug_assertions)]
            self.protected.borrow_mut().push(transmute(term.copy()));

            transmute(term.copy())
        }
    }
}

impl<'a, C: Markable> Deref for Protector<'a, C> {
    type Target = C;
    
    fn deref(&self) -> &Self::Target {
        &self.reference
    }
}

impl<'a, C: Markable> DerefMut for Protector<'a, C> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reference        
    }    
}

impl<'a, C: Markable> Drop for Protector<'a, C> {
    fn drop(&mut self) {
        // TODO: Implement this.
        #[cfg(debug_assertions)]
        {
            for term in self.protected.borrow().iter() {
                debug_assert!(self.reference.contains_term(term), "Term was protected but not actually inserted");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::aterm::TermPool;


    #[test]
    fn test_aterm_container() {
        let mut tp = TermPool::new();
        let t = tp.from_string("f(g(a),b)").unwrap();
        
        // First test the trait for a standard container.
        let mut container = Protected::<Vec::<ATermRef>>::new(vec![]);

        for _ in 0..1000 {
            let mut write = container.write();
            let u = write.protect(&t);
            write.push(u);
        }
    }
}