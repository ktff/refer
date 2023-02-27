use super::{Index, Path, RegionPath, INDEX_BASE_BITS};
use std::{
    any, fmt,
    hash::{Hash, Hasher},
    marker::{PhantomData, Unsize},
    ptr::Pointee,
};

use crate::core::{AnyItem, DynItem, LocalityPath, LocalityRegion};

// NOTE: Key can't be ref since it's not possible for all but the basic library to statically guarantee that
// the key is valid so some kind of dynamic check is needed, hence the library needs to be able to check any key
// hence it needs to be able to know where something starts and ends which is not robustly possible for ref keys.
// That leaves us with numerical keys.

pub type AnyKey = Key<dyn AnyItem>;

/// Key to an Item in a collection.
///
/// Key space is unified across types, that means that an item at some index determines its own type, not the key.
/// Another way of looking at it by drawing parallels with pointers:
/// - Key is a pointer to an item in constrained memory space as defined by a container.
/// - The item at some index has type, but a key with the same index can be cast to different types.
/// - Hence Key<T> is similar to *mut (T,Shell) where following checks are delegated to other parts of the library:
///     - Does *mut (T,Shell) exist? (In pointer terms: is it safe to dereference?) (Responsibility of Container system)
///     - Do we have exclusive or shared access? (In pointer terms: is it safe to dereference as & or &mut?) (Responsibility of Permit system)
///     - To which parts of the slot we have access? (In pointer terms: is it safe to access the item, the slot, or both?) (Responsibility of Permit system)
#[repr(transparent)]
pub struct Key<T: DynItem + ?Sized>(Index, PhantomData<&'static T>);

impl<T: Pointee<Metadata = ()> + DynItem + ?Sized> Key<T> {
    /// Constructors of Key should strive to guarantee that T is indeed at Index.
    pub fn new(index: Index) -> Self {
        Key(index, PhantomData)
    }
}

impl AnyKey {
    pub fn new_any(index: Index) -> Self {
        Key(index, PhantomData)
    }

    /// Make assumption that this is Key for T.
    pub fn assume<T: DynItem + ?Sized>(self) -> Key<T> {
        Key(self.0, PhantomData)
    }
}

impl<T: DynItem + ?Sized> Key<T> {
    pub fn path(&self) -> Path {
        Path::new_top(self.0.get(), INDEX_BASE_BITS.get())
    }

    #[inline(always)]
    pub fn index(&self) -> Index {
        self.0
    }

    pub fn upcast<U: DynItem + ?Sized>(self) -> Key<U>
    where
        T: Unsize<U>,
    {
        Key(self.0, PhantomData)
    }

    pub fn any(self) -> AnyKey {
        Key(self.0, PhantomData)
    }

    // pub fn upcast_array<U: DynItem + ?Sized>(array: &[Key<T>]) -> &[Key<U>]
    // where
    //     T: Unsize<U>,
    // {
    //     // SAFETY: This is safe since Key<T> and Key<U> have the same layout
    //     // and a single Key<T> can be upcast to a single Key<U>.
    //     unsafe { &*(array as *const [Key<T>] as *const [Key<U>]) }
    // }
}

impl<T: DynItem + ?Sized> LocalityPath for Key<T> {
    fn map(&self, region: RegionPath) -> Option<LocalityRegion> {
        Some(LocalityRegion::Index(region.index_of(*self)))
    }

    fn upcast(&self) -> &dyn LocalityPath {
        self
    }
}

impl<T: DynItem + ?Sized> Eq for Key<T> {}

impl<T: DynItem + ?Sized, U: DynItem + ?Sized> PartialEq<Key<U>> for Key<T> {
    fn eq(&self, other: &Key<U>) -> bool {
        self.0 == other.0
    }
}

impl<T: DynItem + ?Sized> Ord for Key<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: DynItem + ?Sized, U: DynItem + ?Sized> PartialOrd<Key<U>> for Key<T> {
    fn partial_cmp(&self, other: &Key<U>) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl<T: DynItem + ?Sized> Hash for Key<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// SAFETY: Key only contains Index, which is Send.
unsafe impl<T: DynItem + ?Sized> Send for Key<T> where Index: Send {}
/// SAFETY: Key only contains Index, which is Sync.
unsafe impl<T: DynItem + ?Sized> Sync for Key<T> where Index: Sync {}

impl<T: DynItem + ?Sized> Copy for Key<T> {}

impl<T: DynItem + ?Sized> Clone for Key<T> {
    fn clone(&self) -> Self {
        Key(self.0, self.1)
    }
}

impl<T: DynItem + ?Sized> fmt::Debug for Key<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}:{}", self.0, any::type_name::<T>())
    }
}

impl<T: DynItem + ?Sized> fmt::Display for Key<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}:{}", self.0, any::type_name::<T>())
    }
}
