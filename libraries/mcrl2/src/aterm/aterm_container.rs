use std::{pin::Pin, sync::Arc, mem::transmute, marker::PhantomData, ops::{DerefMut, Deref}};

use mcrl2_sys::atermpp::ffi;

use crate::aterm::{ATermRef, BfTermPool, BfTermPoolThreadWrite, THREAD_TERM_POOL};

use super::{BfTermPoolRead, ATermTrait};

/// A container of objects, typically either terms or objects containing terms,
/// that are of trait Markable. These store ATermRef<'static> that are protected
/// during garbage collection by being in the container itself.
pub struct TermContainer<C> {
    container: Arc<BfTermPool<Vec<ATermRef<'static>>>>,
    root: usize,
    marker: PhantomData<C>,
}

impl<C> TermContainer<C> {

    pub fn new(container: Vec<ATermRef<'static>>) -> TermContainer<C> {
        let shared = Arc::new(BfTermPool::new(container));

        let root = THREAD_TERM_POOL.with_borrow_mut(|tp| {
            tp.protect_container(shared.clone())
        });

        TermContainer {
            container: shared,
            root,
            marker: Default::default(),
        }
    }

    /// Provides mutable access to the underlying container. Use [protect] of
    /// the resulting guard to be able to insert terms into the container.
    /// Otherwise the borrow checker will note that the [ATermRef] do not
    /// outlive the guard, see [TermProtection].
    pub fn write(&mut self) -> TermProtection<'_, Vec<ATermRef<'_>>> {
        // The lifetime of ATermRef can be derived from self since it is protected by self, so transmute 'static into 'a.
        unsafe {
            TermProtection::new(transmute(self.container.write_exclusive(true)))
        }
    }

    /// Provides immutable access to the underlying container.
    pub fn read(&self) -> BfTermPoolRead<'_, Vec<ATermRef<'_>>> {
        // The lifetime of ATermRef can be derived from self since it is protected by self, so transmute 'static into 'a.
        unsafe {
            transmute(self.container.read())
        }
    }

}

impl<C: Default> Default for TermContainer<C> {
    fn default() -> Self {
        TermContainer::new(Default::default())
    }
}

impl<C> Drop for TermContainer<C> {

    fn drop(&mut self) {
        THREAD_TERM_POOL.with_borrow_mut(|tp| {
            tp.drop_container(self.root);
        });
    }
}

/// This trait should be used on all objects and containers related to storing unprotected terms.
pub trait Markable {
    
    /// Marks all the ATermRefs in the object as being reachable.
    fn mark(&mut self, todo: Pin<&mut ffi::term_mark_stack>);
}

impl<'a> Markable for ATermRef<'a> {
    fn mark(&mut self, todo: Pin<&mut ffi::term_mark_stack>) {
        if !self.is_default() {
            unsafe {
                ffi::aterm_mark_address(self.term, todo);
            }
        }
    }
}

impl<T: Markable> Markable for Vec<T> {
    fn mark(&mut self, mut todo: Pin<&mut ffi::term_mark_stack>) {
        for value in self {
            value.mark(todo.as_mut());
        }
    }
}

impl<T: Markable + ?Sized> BfTermPool<T> {
    pub fn mark(&self, mut todo: Pin<&mut ffi::term_mark_stack>) {
        // Marking will done while an exclusive lock is already held, also this
        // does not implement the Markable trait since self must be immutable
        // here.
        unsafe {
            self.write_exclusive(false).mark(todo.as_mut());
        }
    }
}



/// This is a helper struct used by TermContainer to protected terms that are
/// inserted into the container before the guard is dropped.
/// 
/// The reason is that the ATermRef derive their lifetime from the
/// TermContainer. However, when inserting terms with shorter lifetimes we know
/// that their lifetime is extended by being in the container. This is enforced
/// by runtime checks during debug for containers that implement IntoIterator.
pub struct TermProtection<'a, C: Markable> {
    reference: BfTermPoolThreadWrite<'a, C>,
    
    #[cfg(debug_assertions)]
    protected: Vec<ATermRef<'static>>
}

impl<'a, C: Markable> TermProtection<'a, C> {
    fn new(reference: BfTermPoolThreadWrite<'a, C>) -> TermProtection<'_, C> {
        #[cfg(debug_assertions)]
        return TermProtection {
            reference,
            protected: vec![]
        };
        
        #[cfg(not(debug_assertions))]
        return TermProtection {
            reference,
        }
    }
    
    pub fn protect(&mut self, term: ATermRef<'_>) -> ATermRef<'static>{

        unsafe {
            // Store terms that are marked as protected to check if they are
            // actually in the container when the protection is dropped.
            #[cfg(debug_assertions)]
            self.protected.push(transmute(term.borrow()));

            transmute(term)
        }
    }
}

impl<'a, C: Markable> Deref for TermProtection<'a, C> {
    type Target = C;
    
    fn deref(&self) -> &Self::Target {
        &self.reference
    }
}

impl<'a, C: Markable> DerefMut for TermProtection<'a, C> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reference        
    }    
}

impl<'a, C: Markable> Drop for TermProtection<'a, C> {
    fn drop(&mut self) {
        // TODO: Implement this.
        #[cfg(debug_assertions)]
        {
            // for term in &self.protected {
            //     debug_assert!(self.reference.into_iter().find(|t| t == term).is_some(), "Term was protected, but not actually inserted");
            // }
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
        let mut container = TermContainer::<Vec::<ATermRef>>::new(vec![]);

        for _ in 0..1000 {
            container.write().push(t.borrow());
        }
    }
}