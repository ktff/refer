use super::{Index, KeyPath, Path, INDEX_BASE_BITS};
use std::{
    any::{self, TypeId},
    fmt::{self},
    hash::{Hash, Hasher},
    marker::{PhantomData, Unsize},
    num::NonZeroU64,
    ops::CoerceUnsized,
    ptr::{DynMetadata, Pointee},
};

use crate::core::{AnyContainer, AnyItem, AnyPermit, AnySlot, Item};

// NOTE: Key can't be ref since it's not possible for all but the basic library to statically guarantee that
// the key is valid so some kind of dynamic check is needed, hence the library needs to be able to check any key
// hence it needs to be able to know where something starts and ends which is not robustly possible for ref keys.
// That leaves us with numerical keys.

pub type AnyKey = Key<dyn AnyItem>;

pub struct Key<T: Pointee + AnyItem + ?Sized>(Index, T::Metadata);

impl<T: Pointee<Metadata = ()> + AnyItem + ?Sized> Key<T> {
    pub fn new(index: Index) -> Self {
        Key(index, ())
    }
}

impl<T: Pointee + AnyItem + ?Sized> Key<T> {
    pub fn new_with(index: Index, metadata: T::Metadata) -> Self {
        Key(index, metadata)
    }
}

impl<T: Pointee + AnyItem + ?Sized> Key<T> {
    pub fn type_id(&self) -> TypeId {
        self.key_type_id()
    }

    pub fn path(&self) -> Path {
        Path::new(self.0.get(), INDEX_BASE_BITS.get())
    }

    pub fn index(&self) -> Index {
        self.0
    }

    pub fn metadata(&self) -> T::Metadata {
        self.1
    }

    pub fn upcast<U: Pointee + AnyItem + ?Sized>(self) -> Key<U>
    where
        T: Unsize<U>,
    {
        let Self(index, metadata) = self;
        let ptr = std::ptr::from_raw_parts::<T>(std::ptr::null(), metadata);
        let metadata = std::ptr::metadata(ptr as *const U);
        Key(index, metadata)
    }

    pub fn downcast<U: Pointee<Metadata = ()> + AnyItem + ?Sized>(self) -> Option<Key<U>> {
        if self.type_id() == TypeId::of::<U>() {
            Some(Key::new(self.0))
        } else {
            None
        }
    }

    // /// True if has given prefix.
    // pub fn of(self, prefix: impl Into<KeyPrefix>) -> bool {
    //     prefix.into().prefix_of((self.0).0)
    // }
}

impl<T: Pointee + AnyItem + ?Sized> Eq for Key<T> {}

impl<T: Pointee + AnyItem + ?Sized, U: Pointee + AnyItem + ?Sized> PartialEq<Key<U>> for Key<T> {
    default fn eq(&self, other: &Key<U>) -> bool {
        self.0 == other.0 && self.type_id() == other.type_id()
    }
}

impl<T: Pointee<Metadata = ()> + AnyItem + ?Sized> PartialEq for Key<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Pointee + AnyItem + ?Sized> Ord for Key<T> {
    default fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.type_id()
            .cmp(&other.type_id())
            .then_with(|| self.0.cmp(&other.0))
    }
}

impl<T: Pointee<Metadata = ()> + AnyItem + ?Sized> Ord for Key<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: Pointee + AnyItem + ?Sized, U: Pointee + AnyItem + ?Sized> PartialOrd<Key<U>> for Key<T> {
    default fn partial_cmp(&self, other: &Key<U>) -> Option<std::cmp::Ordering> {
        Some(
            self.type_id()
                .cmp(&other.type_id())
                .then_with(|| self.0.cmp(&other.0)),
        )
    }
}

impl<T: Pointee<Metadata = ()> + AnyItem + ?Sized> PartialOrd for Key<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl<T: Pointee + AnyItem + ?Sized> Hash for Key<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_id().hash(state);
        self.0.hash(state);
    }
}

impl<T: Pointee + AnyItem + ?Sized> Copy for Key<T> {}

impl<T: Pointee + AnyItem + ?Sized> Clone for Key<T> {
    fn clone(&self) -> Self {
        Key(self.0, self.1)
    }
}

impl<T: Pointee + AnyItem + ?Sized> fmt::Debug for Key<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Key<{}>({:?}, {:?})",
            any::type_name::<T>(),
            self.0,
            self.type_id()
        )
    }
}

trait KeyTypeId {
    fn key_type_id(&self) -> TypeId;
}

impl<T: Pointee + AnyItem + ?Sized> KeyTypeId for Key<T> {
    default fn key_type_id(&self) -> TypeId {
        let tmp_data: bool = false;
        let ptr = std::ptr::from_raw_parts::<T>(std::ptr::null(), self.1);
        ptr.item_type_id()
    }
}

impl<T: Pointee<Metadata = ()> + AnyItem + ?Sized> KeyTypeId for Key<T> {
    fn key_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}
