use std::{marker::PhantomData, ops::{DerefMut, Deref}, cell::UnsafeCell};

use mcrl2_sys::atermpp::ffi;

/// Provides access to the mCRL2 busy forbidden protocol. For more efficient
/// shared mutex implementation see bf-sharedmutex.
pub struct BfTermPool<T> {
    object: UnsafeCell<T>
}

unsafe impl<T> Send for BfTermPool<T> {}
unsafe impl<T> Sync for BfTermPool<T> {}

impl<T> BfTermPool<T> {
    pub fn new(object: T) -> BfTermPool<T> {
        BfTermPool {
            object: UnsafeCell::new(object)
        }
    }
}


impl<'a, T> BfTermPool<T> {

    /// Provides read access to the underlying object.
    pub fn read(&'a self) -> BfTermPoolRead<'a, T> {
        ffi::lock_shared();
        BfTermPoolRead {
            mutex: self,
            _marker: Default::default()
        }
    }

    /// Provides write access to the underlying object
    /// 
    /// # Safety
    /// 
    /// This function must ONLY be called within a thread local access. Provides mutable access
    /// given that other threads use write exclusively.
    pub unsafe fn write_threadlocal(&'a self) -> BfTermPoolRead<'a, T> {
        ffi::lock_shared();
        BfTermPoolRead {
            mutex: self,
            _marker: Default::default()
        }
    }

    /// Provides write access to the underlying object.
    pub fn write(&'a self) -> BfTermPoolWrite<'a, T> {
        ffi::lock_exclusive();
        BfTermPoolWrite {
            mutex: self,
            _marker: Default::default()
        }
    }

}

pub struct BfTermPoolRead<'a, T> {
    mutex: &'a BfTermPool<T>,
    _marker: PhantomData<&'a ()>,
}

impl<'a, T> Deref for BfTermPoolRead<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // There can only be read guards.
        unsafe { &*self.mutex.object.get() }
    }
}

impl<'a, T> Drop for BfTermPoolRead<'a, T> {
    fn drop(&mut self) {
        ffi::unlock_shared();        
    }
}

pub struct BfTermPoolWrite<'a, T> {
    mutex: &'a BfTermPool<T>,
    _marker: PhantomData<&'a ()>,
}

impl<'a, T> Deref for BfTermPoolWrite<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // There can only be read guards.
        unsafe { &*self.mutex.object.get() }
    }
}


impl<'a, T> DerefMut for BfTermPoolWrite<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // We are the only guard after `write()`, so we can provide mutable access to the underlying object.
        unsafe { &mut *self.mutex.object.get() }
    }
}

impl<'a, T> Drop for BfTermPoolWrite<'a, T> {
    fn drop(&mut self) {
        ffi::unlock_exclusive();        
    }
}