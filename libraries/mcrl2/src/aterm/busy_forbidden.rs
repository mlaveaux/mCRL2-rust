use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use mcrl2_sys::atermpp::ffi;

/// Provides access to the mCRL2 busy forbidden protocol, where there
/// are thread local busy flags and one central storage for the forbidden
/// flags. Care must be taken to avoid deadlocks since the FFI also uses
/// the same flags.
pub struct BfTermPool<T: ?Sized> {
    object: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for BfTermPool<T> {}
unsafe impl<T: Send> Sync for BfTermPool<T> {}

impl<T> BfTermPool<T> {
    pub fn new(object: T) -> BfTermPool<T> {
        BfTermPool {
            object: UnsafeCell::new(object),
        }
    }
}

impl<'a, T: ?Sized> BfTermPool<T> {
    /// Provides read access to the underlying object.
    pub fn read(&'a self) -> BfTermPoolRead<'a, T> {
        ffi::lock_shared();
        BfTermPoolRead {
            mutex: self,
            _marker: Default::default(),
        }
    }

    /// Provides write access to the underlying object
    ///
    /// # Safety
    ///
    /// Provides mutable access given that other threads use write exclusively. If we are already in an exclusive context
    /// then lock can be set to false.
    pub unsafe fn write_exclusive(&'a self, lock: bool) -> BfTermPoolThreadWrite<'a, T> {
        // This is a lock shared, but assuming that only ONE thread uses this function.
        if lock {
            ffi::lock_shared();
        }
        BfTermPoolThreadWrite {
            mutex: self,
            locked: lock,
            _marker: Default::default(),
        }
    }

    /// Provides write access to the underlying object.
    pub fn write(&'a self) -> BfTermPoolWrite<'a, T> {
        ffi::lock_exclusive();
        BfTermPoolWrite {
            mutex: self,
            _marker: Default::default(),
        }
    }
}

pub struct BfTermPoolRead<'a, T: ?Sized> {
    mutex: &'a BfTermPool<T>,
    _marker: PhantomData<&'a ()>,
}

impl<'a, T: ?Sized> Deref for BfTermPoolRead<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // There can only be read guards.
        unsafe { &*self.mutex.object.get() }
    }
}

impl<'a, T: ?Sized> Drop for BfTermPoolRead<'a, T> {
    fn drop(&mut self) {
        ffi::unlock_shared();
    }
}

pub struct BfTermPoolWrite<'a, T: ?Sized> {
    mutex: &'a BfTermPool<T>,
    _marker: PhantomData<&'a ()>,
}

impl<'a, T: ?Sized> Deref for BfTermPoolWrite<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // There can only be read guards.
        unsafe { &*self.mutex.object.get() }
    }
}

impl<'a, T: ?Sized> DerefMut for BfTermPoolWrite<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // We are the only guard after `write()`, so we can provide mutable access to the underlying object.
        unsafe { &mut *self.mutex.object.get() }
    }
}

impl<'a, T: ?Sized> Drop for BfTermPoolWrite<'a, T> {
    fn drop(&mut self) {
        ffi::unlock_exclusive();
    }
}

pub struct BfTermPoolThreadWrite<'a, T: ?Sized> {
    mutex: &'a BfTermPool<T>,
    locked: bool,
    _marker: PhantomData<&'a ()>,
}

impl<'a, T: ?Sized> Deref for BfTermPoolThreadWrite<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // There can only be read guards.
        unsafe { &*self.mutex.object.get() }
    }
}

impl<'a, T: ?Sized> DerefMut for BfTermPoolThreadWrite<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // We are the only guard after `write()`, so we can provide mutable access to the underlying object.
        unsafe { &mut *self.mutex.object.get() }
    }
}

impl<'a, T: ?Sized> Drop for BfTermPoolThreadWrite<'a, T> {
    fn drop(&mut self) {
        if self.locked {
            ffi::unlock_shared();
        }
    }
}
