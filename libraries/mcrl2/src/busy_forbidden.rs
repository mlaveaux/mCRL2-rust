use std::marker::PhantomData;

use mcrl2_sys::atermpp::ffi;

/// Provides access to the mCRL2 busy forbidden protocol. For more efficient
/// shared mutex implementation see bf-sharedmutex.
pub struct BfTermPool<T> {
    object: T
}

pub struct BfTermPoolRead<'a, T> {
    object: &'a T,
    _marker: PhantomData<&'a ()>,
}

impl<'a, T> Drop for BfTermPoolRead<'a, T> {
    fn drop(&mut self) {
        ffi::unlock_shared();        
    }
}

pub struct BfTermPoolWrite<'a, T> {
    object: &'a T,
    _marker: PhantomData<&'a ()>,
}

impl<'a, T> Drop for BfTermPoolWrite<'a, T> {
    fn drop(&mut self) {
        ffi::unlock_exclusive();        
    }
}

impl<'a, T> BfTermPool<T> {

    pub fn read(&'a self) -> BfTermPoolRead<'a, T> {
        ffi::lock_shared();
        BfTermPoolRead {
            object: &self.object,
            _marker: Default::default()
        }
    }

    pub fn write(&'a self) -> BfTermPoolWrite<'a, T> {
        ffi::lock_exclusive();
        BfTermPoolWrite {
            object: &self.object,
            _marker: Default::default()
        }
    }

}