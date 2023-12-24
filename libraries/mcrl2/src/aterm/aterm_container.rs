use std::{pin::Pin, sync::Arc, mem::transmute, marker::PhantomData};

use mcrl2_sys::atermpp::ffi;

use crate::aterm::{ATermRef, BfTermPool, BfTermPoolThreadWrite, THREAD_TERM_POOL};

use super::{BfTermPoolRead, ATermTrait};

/// A container of objects, typically either terms or objects containing terms, that are Markable.
/// These store ATermRef<'static> that are protected during garbage collection by the container itself.
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

    /// Provides mutable access to the underlying container.
    pub fn write<'a>(&'a mut self) -> BfTermPoolThreadWrite<'a, Vec<ATermRef<'a>>> {
        // The lifetime of ATermRef can be derived from self since it is protected by self, so transmute 'static into 'a.
        unsafe {
            transmute(self.container.write_exclusive(true))
        }
    }

    /// Provides immutable access to the underlying container.
    pub fn read<'a>(&'a self) -> BfTermPoolRead<'a, Vec<ATermRef<'a>>> {
        // The lifetime of ATermRef can be derived from self since it is protected by self, so transmute 'static into 'a.
        unsafe {
            transmute(self.container.read())
        }
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

impl<'a, T: Markable + ?Sized> BfTermPool<T> {
    pub fn mark(&self, mut todo: Pin<&mut ffi::term_mark_stack>) {
        // Marking will done while an exclusive lock is already held, also this
        // does not implement the Markable trait since self must be immutable
        // here.
        unsafe {
            self.write_exclusive(false).mark(todo.as_mut());
        }
    }
}

impl<C: Default> Default for TermContainer<C> {
    fn default() -> Self {
        TermContainer::new(Default::default())
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