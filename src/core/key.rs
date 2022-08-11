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
pub struct Index(pub u64);

impl Debug for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

/// It's the responsibility of collections to issue keys in a way that close by indices have close by items.
pub struct Key<T: ?Sized>(Index, PhantomData<T>);

impl<T: ?Sized> Key<T> {
    pub fn new(index: Index) -> Self {
        Key(index, PhantomData)
    }
}

impl<T: ?Sized> Eq for Key<T> {}

impl<T: ?Sized> PartialEq for Key<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: ?Sized + 'static> PartialEq<AnyKey> for Key<T> {
    fn eq(&self, other: &AnyKey) -> bool {
        &AnyKey::from(*self) == other
    }
}

impl<T: ?Sized> Ord for Key<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: ?Sized> PartialOrd for Key<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: ?Sized> Hash for Key<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T: ?Sized> Copy for Key<T> {}

impl<T: ?Sized> Clone for Key<T> {
    fn clone(&self) -> Self {
        Key(self.0, PhantomData)
    }
}

impl<T: ?Sized> Debug for Key<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key<{}>({:?})", any::type_name::<T>(), self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnyKey(TypeId, Index);

impl AnyKey {
    pub fn new(ty_id: TypeId, index: Index) -> Self {
        AnyKey(ty_id, index)
    }

    pub fn downcast<T: ?Sized + 'static>(self) -> Option<Key<T>> {
        if self.0 == TypeId::of::<T>() {
            Some(Key::new(self.1))
        } else {
            None
        }
    }
}

impl<T: ?Sized + 'static> PartialEq<Key<T>> for AnyKey {
    fn eq(&self, other: &Key<T>) -> bool {
        self == &Self::from(*other)
    }
}

impl std::fmt::Debug for AnyKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AnyKey<{:?}>({:?})", self.0, self.1)
    }
}

impl<T: ?Sized + 'static> From<Key<T>> for AnyKey {
    fn from(key: Key<T>) -> Self {
        AnyKey(TypeId::of::<T>(), key.0)
    }
}
