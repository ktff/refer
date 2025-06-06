use super::{Index, IndexBase, Path, RegionPath, INDEX_BASE_BITS};
use std::{
    any::{self, TypeId},
    borrow::Borrow,
    fmt,
    hash::{Hash, Hasher},
    marker::{PhantomData, Unsize},
};

use crate::core::{AnyItem, Container, DynItem, Item, LocalityPath, LocalityRegion};

// NOTE: Key can't be ref since it's not possible for all but the basic library to statically guarantee that
// the key is valid so some kind of dynamic check is needed, hence the library needs to be able to check any key
// hence it needs to be able to know where something starts and ends which is not robustly possible for ref keys.
// That leaves us with numerical keys.

/// A pointer equivalent Key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ptr;

// Resolves to Ref if 'a can be proven.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Promise<'a>(PhantomData<&'a ()>);

/// A reference equivalent Key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ref<'a>(PhantomData<&'a ()>);

/// A shared owned equivalent Key.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Owned;

/// Key to an Item in a collection.
///
/// Key space is unified across types, that means that an item at some index determines its own type, not the key.
/// Another way of looking at it by drawing parallels with pointers:
/// - Key is a pointer to an item in constrained memory space as defined by a container.
///
/// - Guarantees of lifetime through which access and lifetime management is more ergonomic are:
///     - Ptr which is similar to *const T in that there is no guarantee.
///     - Ref which is similar to &'a T in that item should be alive for 'a lifetime. Enforced by the type system.
///     - Owned which is similar to Arc<T> in that the item should be alive while it's alive.
///       Partially enforced by traits, and partially by the type system, but at most cases it should be handled with care, else it can lead to item leaks.
///
/// - The item at some index has type, but a key with the same index can be cast to different types.
///
/// - Hence Key<T> is similar to *mut T where following checks are delegated to other parts of the library:
///     - Does *mut T exist? (In pointer terms: is it safe to dereference at all?) (Responsibility of Container system)
///     - Do we have exclusive or shared access? (In pointer terms: is it safe to dereference as & or &mut?) (Responsibility of Permit system)
///     - To which parts of the slot we have access? (In pointer terms: is it safe to access the item, the slot, or both?) (Responsibility of Permit system)
#[repr(transparent)]
pub struct Key<K = Ptr, T: DynItem + ?Sized = dyn AnyItem>(Index, PhantomData<(&'static T, K)>);

impl<T: DynItem + ?Sized> Key<Owned, T> {
    /// UNSAFE: This isn't unsafe per se since safety checks will still be made, but they can panic if
    /// caller allows for this key to outlive Item on index.
    ///
    /// Callers should also forget this Key<Owned> when they don't need to guarantee that T exists through it anymore.
    pub unsafe fn new_owned(index: Index) -> Self {
        Self(index, PhantomData)
    }
}

impl<'a, T: DynItem + ?Sized> Key<Ref<'a>, T> {
    /// UNSAFE: This isn't unsafe per se since safety checks will still be made, but they can panic if
    /// caller allow for this key to outlive Item on index.
    pub unsafe fn new_ref(index: Index) -> Self {
        Self(index, PhantomData)
    }

    /// UNSAFE: Caller must guarantee that T will be alive for 'b lifetime.
    pub unsafe fn extend<'b>(self) -> Key<Ref<'b>, T> {
        Key(self.0, PhantomData)
    }

    /// Item is guaranteed to be alive if you can later prove that 'b is valid.
    pub fn promise<'b>(self) -> Key<Promise<'b>, T> {
        Key(self.0, PhantomData)
    }
}

impl<'a, T: DynItem + ?Sized> Key<Promise<'a>, T> {
    /// UNSAFE: Caller must guarantee that 'a is a valid lifetime.
    pub unsafe fn as_ref(self) -> Key<Ref<'a>, T> {
        Key(self.0, PhantomData)
    }

    /// Fulfills promise by providing proof that 'b is a valid lifetime.
    /// Caller must ensure that both keys belong to the same graph.
    pub fn fulfill<D: DynItem + ?Sized>(self, _proof: Key<Ref<'a>, D>) -> Key<Ref<'a>, T> {
        // SAFETY: We have proof of a key existing with 'b lifetime therefor we can use that lifetime.
        unsafe { self.as_ref() }
    }
}

impl<T: Item> Key<Ptr, T> {
    /// Constructors of Key should strive to guarantee that T is indeed at Index.
    pub const fn new_ptr(index: Index) -> Self {
        Key(index, PhantomData)
    }

    /// Constructors of Key should strive to guarantee that T is indeed at Index.
    /// Panics if index is 0
    pub const fn new_const(index: IndexBase) -> Self {
        let Some(index) = Index::new(index) else {
            panic!("Index is 0");
        };
        Key(index, PhantomData)
    }

    /// UNSAFE: Caller must guarantee that it T will be alive for 'a lifetime.
    pub unsafe fn extend<'a>(self) -> Key<Ref<'a>, T> {
        Key(self.0, PhantomData)
    }

    /// Checked ref to T
    pub fn ref_from<'a>(self, container: &'a impl Container<T>) -> Option<Key<Ref<'a>, T>> {
        Some(self)
            .filter(|key| container.contains_slot(*key))
            // SAFETY: container contains slot and it will do so for at least 'a.
            .map(|key| unsafe { key.extend() })
    }
}

impl Key {
    pub fn new_any(index: Index) -> Self {
        Key(index, PhantomData)
    }
}

impl<P> Key<P> {
    /// Make assumption that this is Key for T.
    pub fn assume<T: DynItem + ?Sized>(self) -> Key<P, T> {
        Key(self.0, PhantomData)
    }
}

impl Key<Owned> {
    /// Checked downcast to T
    pub fn downcast<T: Item>(self, container: &impl Container<T>) -> Option<Key<Owned, T>> {
        Some(self.assume()).filter(|key| container.contains_slot(key.ptr()))
    }
}

impl<'a> Key<Ref<'a>> {
    /// Checked downcast to T
    pub fn downcast<T: Item>(self, container: &impl Container<T>) -> Option<Key<Ref<'a>, T>> {
        Some(self.assume()).filter(|key| container.contains_slot(key.ptr()))
    }
}

impl Key<Ptr> {
    /// Checked downcast to T
    pub fn downcast<'a, T: Item>(
        self,
        container: &'a impl Container<T>,
    ) -> Option<Key<Ref<'a>, T>> {
        Some(self.assume())
            .filter(|key| container.contains_slot(key.ptr()))
            // SAFETY: container contains slot and it will do so for at least 'a.
            .map(|key| unsafe { key.extend() })
    }
}

impl<P, T: DynItem + ?Sized> Key<P, T> {
    // pub(super) unsafe fn new(index: Index) -> Self {
    //     Self(index, PhantomData)
    // }

    pub fn path(&self) -> Path {
        Path::new_top(self.0.get(), INDEX_BASE_BITS.get())
    }

    #[inline(always)]
    pub const fn index(&self) -> Index {
        self.0
    }

    pub fn ptr(&self) -> Key<Ptr, T> {
        Key(self.0, PhantomData)
    }

    pub fn upcast<U: DynItem + ?Sized>(self) -> Key<P, U>
    where
        T: Unsize<U>,
    {
        Key(self.0, PhantomData)
    }

    pub fn any(self) -> Key<P> {
        Key(self.0, PhantomData)
    }
}

impl<'a, T: DynItem + ?Sized> Key<Ref<'a>, T> {
    pub fn borrow<'b>(&self) -> Key<Ref<'b>, T> {
        Key(self.0, PhantomData)
    }
}

impl<T: DynItem + ?Sized> Key<Owned, T> {
    pub fn borrow<'a>(&self) -> Key<Ref<'a>, T> {
        Key(self.0, PhantomData)
    }
}

impl<P, T: DynItem + ?Sized> LocalityPath for Key<P, T> {
    default fn map(&self, region: RegionPath) -> Option<LocalityRegion> {
        Some(LocalityRegion::Index(region.index_of(self.ptr())))
    }

    default fn upcast(&self) -> &dyn LocalityPath {
        self
    }
}

impl<P, T: Item> LocalityPath for Key<P, T> {
    fn map(&self, region: RegionPath) -> Option<LocalityRegion> {
        Some(LocalityRegion::Id((
            TypeId::of::<T>(),
            region.index_of(self.ptr()),
        )))
    }

    fn upcast(&self) -> &dyn LocalityPath {
        self
    }
}

impl<P, T: DynItem + ?Sized> Eq for Key<P, T> {}

impl<PT, T: DynItem + ?Sized, PU, U: DynItem + ?Sized> PartialEq<Key<PU, U>> for Key<PT, T> {
    fn eq(&self, other: &Key<PU, U>) -> bool {
        self.0 == other.0
    }
}

impl<P, T: DynItem + ?Sized> Ord for Key<P, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<PT, T: DynItem + ?Sized, PU, U: DynItem + ?Sized> PartialOrd<Key<PU, U>> for Key<PT, T> {
    fn partial_cmp(&self, other: &Key<PU, U>) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl<P, T: DynItem + ?Sized> Hash for Key<P, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// SAFETY: Key only contains Index, which is Send.
unsafe impl<P, T: DynItem + ?Sized> Send for Key<P, T> where Index: Send {}
/// SAFETY: Key only contains Index, which is Sync.
unsafe impl<P, T: DynItem + ?Sized> Sync for Key<P, T> where Index: Sync {}

impl<P: Copy, T: DynItem + ?Sized> Copy for Key<P, T> {}

impl<P: Clone, T: DynItem + ?Sized> Clone for Key<P, T> {
    fn clone(&self) -> Self {
        Key(self.0, self.1)
    }
}

impl<P: KeySign, T: DynItem + ?Sized> fmt::Debug for Key<P, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}:{}{}", self.0, P::sign(), any::type_name::<T>())
    }
}

impl<P: KeySign, T: DynItem + ?Sized> fmt::Display for Key<P, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a, T: DynItem + ?Sized> Borrow<Key<Ptr, T>> for Key<Ref<'a>, T> {
    fn borrow(&self) -> &Key<Ptr, T> {
        // SAFETY: They both have same representation of Index
        unsafe { &*(self as *const Self as *const u8 as *const Key<Ptr, T>) }
    }
}

pub trait KeyMeta {
    fn is_owned(&self) -> bool;
}

default impl<K, T: 'static + ?Sized> KeyMeta for Key<K, T> {
    fn is_owned(&self) -> bool {
        false
    }
}

impl<T: 'static + ?Sized> KeyMeta for Key<Owned, T> {
    fn is_owned(&self) -> bool {
        true
    }
}

pub(super) trait KeySign {
    fn sign() -> &'static str;
}

impl<T> KeySign for T {
    default fn sign() -> &'static str {
        any::type_name::<T>()
    }
}

impl KeySign for Ptr {
    fn sign() -> &'static str {
        "*"
    }
}

impl KeySign for Owned {
    fn sign() -> &'static str {
        ""
    }
}

impl KeySign for Ref<'_> {
    fn sign() -> &'static str {
        "&"
    }
}
