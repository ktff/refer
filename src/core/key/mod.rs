mod index;
mod prefix;
mod sub;

use std::{
    any::{self, TypeId},
    fmt::{self},
    hash::{Hash, Hasher},
    marker::PhantomData,
};

pub use index::*;
pub use prefix::*;
pub use sub::*;

use crate::core::{AnyContainer, AnyPermit, AnySlot};

// TODO: Revisit, refactor, Any X {Key,Prefix,SubKey} and Index concepts, interaction, etc.
// TODO: BottomKey ?
// TODO: Test this whole stack

// NOTE: Key can't be ref since it's not possible for all but the basic library to statically guarantee that
// the key is valid so some kind of dynamic check is needed, hence the library needs to be able to check any key
// hence it needs to be able to know where something starts and ends which is not robustly possible for ref keys.
// That leaves us with numerical keys.

pub struct Key<T: ?Sized + 'static>(Index, PhantomData<T>);

impl<T: ?Sized + 'static> Key<T> {
    pub fn new(index: Index) -> Self {
        Key(index, PhantomData)
    }

    pub fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    pub fn index(&self) -> Index {
        self.0
    }

    pub fn as_u64(&self) -> u64 {
        self.index().0.get()
    }

    pub fn upcast(self) -> AnyKey
    where
        T: 'static,
    {
        self.into()
    }

    pub fn order(array: [Self; 2]) -> [Self; 2] {
        if array[0].as_u64() <= array[1].as_u64() {
            array
        } else {
            [array[1], array[0]]
        }
    }
}

impl<T: ?Sized + 'static> Eq for Key<T> {}

impl<T: ?Sized + 'static> PartialEq for Key<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: ?Sized + 'static> Ord for Key<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: ?Sized + 'static> PartialOrd for Key<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: ?Sized + 'static> Hash for Key<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T: ?Sized + 'static> Copy for Key<T> {}

impl<T: ?Sized + 'static> Clone for Key<T> {
    fn clone(&self) -> Self {
        Key(self.0, PhantomData)
    }
}

impl<T: ?Sized + 'static> fmt::Debug for Key<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key<{}>({:?})", any::type_name::<T>(), self.0)
    }
}

impl<T: ?Sized + 'static> fmt::Display for Key<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{:?}", any::type_name::<T>(), self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AnyKey(TypeId, Index);

impl AnyKey {
    pub fn new(ty_id: TypeId, index: Index) -> Self {
        AnyKey(ty_id, index)
    }

    pub fn type_id(&self) -> TypeId {
        self.0
    }

    pub fn index(&self) -> Index {
        self.1
    }

    pub fn as_u64(&self) -> u64 {
        self.index().0.get()
    }

    pub fn downcast<T: ?Sized + 'static>(self) -> Option<Key<T>> {
        if self.0 == TypeId::of::<T>() {
            Some(Key::new(self.1))
        } else {
            None
        }
    }
}

impl fmt::Debug for AnyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnyKey<{:?}>({:?})", self.0, self.1)
    }
}

impl fmt::Display for AnyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}#{}", self.0, self.1)
    }
}

impl<T: ?Sized + 'static> From<Key<T>> for AnyKey {
    fn from(key: Key<T>) -> Self {
        AnyKey(TypeId::of::<T>(), key.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DynKey<T: ?Sized + 'static>(AnyKey, PhantomData<T>);
