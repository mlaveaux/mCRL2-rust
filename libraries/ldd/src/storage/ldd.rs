
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

/// Every Ldd points to its root node in the Storage instance for maximal
/// sharing. These Ldd instances can only be created from the storage.
pub struct Ldd
{
    index: usize, // Reference in the node table.
    root: usize, // Index in the root set.
    protection_set: Rc<RefCell<ProtectionSet>>,
}

impl Ldd
{
    pub fn new(protection_set: &Rc<RefCell<ProtectionSet>>, index: usize) -> Ldd
    {
        let root = protection_set.borrow_mut().protect(index);
        Ldd { protection_set: Rc::clone(protection_set), index, root }
    }

    pub fn index(&self) -> usize
    {
        self.index
    }

    /// Returns an [LddRef] with the same lifetime as this Ldd instance.
    pub fn borrow(&self) -> LddRef
    {
        LddRef::new(self.index)
    }
}

impl Clone for Ldd
{
    fn clone(&self) -> Self
    {
        Ldd::new(&self.protection_set, self.index())
    }
}

impl Drop for Ldd
{
    fn drop(&mut self)
    {
        self.protection_set.borrow_mut().unprotect(self.root);
    }
}

impl PartialEq for Ldd
{
    fn eq(&self, other: &Self) -> bool
    {
        debug_assert!(Rc::ptr_eq(&self.protection_set, &other.protection_set), "Both LDDs should refer to the same storage."); 
        self.index() == other.index()
    }
}

impl Debug for Ldd
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "index: {}", self.index())
    }
}

impl Hash for Ldd
{    
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index().hash(state);
    }
}

impl Eq for Ldd {}

/// The LddRef is a reference to an existing [Ldd] instance. This can be used to
/// avoid explicit protections that are performed when creating an [Ldd] instance.
#[derive(Hash, PartialEq, Eq, Debug)]
pub struct LddRef<'a>
{
    index: usize, // Index in the node table.
    marker: PhantomData<&'a ()>
}

impl<'a> LddRef<'a>
{
    /// TODO: This function should only be called by Storage and [Ldd]
    pub fn new(index: usize) -> LddRef<'a>
    {
        LddRef { index, marker: PhantomData }
    }

    pub fn index(&self) -> usize
    {
        self.index
    }
    
    /// Returns an LddRef with the same lifetime as itself.
    pub fn borrow(&self) -> LddRef
    {
        LddRef::new(self.index())        
    }
}

impl PartialEq<Ldd> for LddRef<'_>
{
    fn eq(&self, other: &Ldd) -> bool
    { 
        self.index == other.index()
    }
}