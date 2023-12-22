use std::{pin::Pin, sync::Arc, mem::transmute};

use mcrl2_sys::atermpp::ffi;

use crate::aterm::{ATermRef, BfTermPool, BfTermPoolThreadWrite, THREAD_TERM_POOL};

/// A container of objects that are Markable.
struct TermContainer<C: Markable> {
    container: Arc<BfTermPool<C>>,
    root: usize,
}

impl<C: Send + Markable> TermContainer<C> {

    pub fn new(container: C) -> TermContainer<C> {
        let mut container = Arc::new(BfTermPool::new(container));

        let root = THREAD_TERM_POOL.with_borrow_mut(|tp| {

            unsafe {
                let t: Arc<dyn Markable + 'static> = container.clone();
                tp.protect_container(t)
            }
        });

        TermContainer {
            container,
            root,
        }
    }
}

//     pub fn write<'b: 'a>(&'b self) -> BfTermPoolThreadWrite<'b, C> {
//         unsafe {
//             self.container.write_exclusive(true)      
//         }
//     }
// }

// impl<'a, C: Markable<'a>> Drop for TermContainer<'a, C> {

//     fn drop(&mut self) {
//         THREAD_TERM_POOL.with_borrow_mut(|tp| {
//             tp.drop_container(self.root);
//         });
//     }
// }

/// This trait should be used on all objects and containers related to storing unprotected terms.
pub trait Markable {
    
    /// Marks all the ATermRefs in the object as being reachable.
    fn mark(&mut self, todo: Pin<&mut ffi::term_mark_stack>);
}

// impl<'a> Markable<'a> for ATermRef<'a> {
//     fn mark(&self, todo: Pin<&mut ffi::term_mark_stack>) {
//         unsafe {
//             ffi::aterm_mark_address(self.term, todo);
//         }
//     }
// }

// impl<'a, T: Markable<'a>> Markable<'a> for Vec<T> {
//     fn mark(&self, mut todo: Pin<&mut ffi::term_mark_stack>) {
//         for value in self {
//             value.mark(todo.as_mut());
//         }
//     }
// }

impl<'a, T: Markable + ?Sized> Markable for BfTermPool<T> {
    fn mark(&mut self, mut todo: Pin<&mut ffi::term_mark_stack>) {
        // Marking will done while an exclusive lock is already held
        unsafe {
            self.write_exclusive(false).mark(todo.as_mut());
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     use crate::aterm::{ATermRef, TermPool};

//     #[test]
//     fn test_aterm_container() {
//         let mut tp = TermPool::new();
//         let t = tp.from_string("f(g(a),b)").unwrap();
        
//         // First test the trait for a standard container.
//         let container = TermContainer::<Vec::<ATermRef>>::new(vec![]);

//         for _ in 0..1000 {
//             container.write().push(t.borrow());
//         }
//     }
// }