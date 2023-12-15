use std::marker::PhantomData;

use mcrl2_sys::{atermpp::ffi, cxx::UniquePtr};


/// Provides access to the mCRL2 busy forbidden protocol. For more efficient
/// shared mutex implementation see bf-sharedmutex.
struct BfTermPool<T> {
    object: T
}

struct BfTermPoolRead<'a> {
    guard: UniquePtr<ffi::shared_guard>,
    _marker: PhantomData<&'a ()>,
}

struct BfTermPoolWrite<'a> {
    guard: UniquePtr<ffi::lock_guard>,
    _marker: PhantomData<&'a ()>,
}



impl<'a, T> BfTermPool<T> {

    pub fn read(&'a self) -> BfTermPoolRead<'a> {
        BfTermPoolRead {
            guard: ffi::lock_shared(),
            _marker: Default::default()
        }
    }

    pub fn write(&'a self) -> BfTermPoolWrite<'a> {
        BfTermPoolWrite {
            guard: ffi::lock_exclusive(),
            _marker: Default::default()
        }
    }

}