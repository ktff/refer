use std::{
    any::{self, TypeId},
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    marker::PhantomData,
};

// NOTE: Key can't be ref since it's not possible for all but the basic library to statically guarantee that
// the key is valid so some kind of dynamic check is needed, hence the library needs to be able to check any key
// hence it needs to be able to know where something starts and ends which is not robustly possible for ref keys.

// NOTE: That leaves us with numerical keys.

// NOTE: Index could be larger than u64 so the possibility of changing that to u128 is left as an option.

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Index(pub u64);

impl Debug for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

pub struct Key<T>(Index, PhantomData<T>);

impl<T> Key<T> {
    pub(crate) fn new(index: Index) -> Self {
        Key(index, PhantomData)
    }
}

impl<T> Eq for Key<T> {}

impl<T> PartialEq for Key<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Hash for Key<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Copy for Key<T> {}

impl<T> Clone for Key<T> {
    fn clone(&self) -> Self {
        Key(self.0, PhantomData)
    }
}

impl<T> Debug for Key<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key<{}>({:?})", any::type_name::<T>(), self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnyKey(TypeId, Index);

impl AnyKey {
    pub(crate) fn new(ty_id: TypeId, index: Index) -> Self {
        AnyKey(ty_id, index)
    }

    pub fn downcast<T: 'static>(self) -> Option<Key<T>> {
        if self.0 == TypeId::of::<T>() {
            Some(Key::new(self.1))
        } else {
            None
        }
    }
}

impl std::fmt::Debug for AnyKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AnyKey<{:?}>({:?})", self.0, self.1)
    }
}

impl<T: 'static> From<Key<T>> for AnyKey {
    fn from(key: Key<T>) -> Self {
        AnyKey(TypeId::of::<T>(), key.0)
    }
}
