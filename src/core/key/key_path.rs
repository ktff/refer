use super::{Key, LeafPath, Path};
use std::{
    any::{self, TypeId},
    fmt::{self},
    hash::{Hash, Hasher},
    marker::Unsize,
    ptr::Pointee,
};

use crate::core::AnyItem;

pub type AnyPath = KeyPath<dyn AnyItem>;

pub struct KeyPath<T: Pointee + AnyItem + ?Sized>(Path, T::Metadata);

impl<T: Pointee<Metadata = ()> + AnyItem + ?Sized> KeyPath<T> {
    pub fn new(path: Path) -> Self {
        KeyPath(path, ())
    }
}

impl<T: Pointee + AnyItem + ?Sized> KeyPath<T> {
    pub fn new_with(path: Path, metadata: T::Metadata) -> Self {
        KeyPath(path, metadata)
    }
}

impl<T: Pointee + AnyItem + ?Sized> KeyPath<T> {
    pub fn type_id(&self) -> TypeId {
        self.key_type_id()
    }

    #[inline(always)]
    pub fn path(&self) -> Path {
        self.0
    }

    #[inline(always)]
    pub fn metadata(&self) -> T::Metadata {
        self.1
    }

    /// Returns longest path that covers both paths.
    pub fn or(self, other: impl Into<Path>) -> Self {
        Self(self.0.or(other), self.1)
    }

    /// Returns shortest path covered by both paths.
    pub fn and(self, other: impl Into<Path>) -> Option<Self> {
        self.0.and(other).map(|path| Self(path, self.1))
    }

    /// Iterates over children of given level, None if level is too high.
    pub fn iter_level(self, level: u32) -> Option<impl ExactSizeIterator<Item = Self>> {
        self.0
            .iter_level(level)
            .map(|iter| iter.map(move |path| Self(path, self.1)))
    }

    pub fn upcast<U: Pointee + AnyItem + ?Sized>(self) -> KeyPath<U>
    where
        T: Unsize<U>,
    {
        let Self(index, metadata) = self;
        let ptr = std::ptr::from_raw_parts::<T>(std::ptr::null(), metadata);
        let metadata = std::ptr::metadata(ptr as *const U);
        KeyPath(index, metadata)
    }

    pub fn downcast<U: Pointee<Metadata = ()> + AnyItem + ?Sized>(self) -> Option<KeyPath<U>> {
        if self.type_id() == TypeId::of::<U>() {
            Some(KeyPath(self.0, ()))
        } else {
            None
        }
    }

    /// Downcasts key if type matches and this is the start of the key.
    pub fn downcast_key<U: AnyItem + ?Sized>(self, key: Key<U>) -> Option<Key<T>> {
        if self.type_id() == key.type_id() {
            let key = Key::new_with(key.index(), self.1);
            if self.includes_key(key) {
                Some(key)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// True if self start of given path.
    pub fn includes(self, path: KeyPath<T>) -> bool {
        self.0.includes_path(path.0)
    }

    /// True if self start of given key.
    pub fn includes_key(self, key: Key<T>) -> bool {
        self.0.includes_index(key.index())
    }
}

impl<T: Pointee + AnyItem + ?Sized> Eq for KeyPath<T> {}

impl<T: Pointee + AnyItem + ?Sized, U: Pointee + AnyItem + ?Sized> PartialEq<KeyPath<U>>
    for KeyPath<T>
{
    default fn eq(&self, other: &KeyPath<U>) -> bool {
        self.0 == other.0 && self.type_id() == other.type_id()
    }
}

impl<T: Pointee<Metadata = ()> + AnyItem + ?Sized> PartialEq for KeyPath<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Pointee + AnyItem + ?Sized> Hash for KeyPath<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_id().hash(state);
        self.0.hash(state);
    }
}

impl<T: Pointee + AnyItem + ?Sized> Copy for KeyPath<T> {}

impl<T: Pointee + AnyItem + ?Sized> Clone for KeyPath<T> {
    fn clone(&self) -> Self {
        KeyPath(self.0, self.1)
    }
}

impl<T: Pointee + AnyItem + ?Sized> fmt::Debug for KeyPath<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "KeyPath<{}>({:?}, {:?})",
            any::type_name::<T>(),
            self.0,
            self.type_id()
        )
    }
}

// Default
impl<T: Pointee<Metadata = ()> + AnyItem + ?Sized> Default for KeyPath<T> {
    fn default() -> Self {
        KeyPath(Path::default(), ())
    }
}

impl<T: Pointee + AnyItem + ?Sized> From<Key<T>> for KeyPath<T> {
    fn from(key: Key<T>) -> Self {
        KeyPath(key.path(), key.metadata())
    }
}

impl<T: Pointee<Metadata = ()> + AnyItem + ?Sized> From<LeafPath> for KeyPath<T> {
    fn from(path: LeafPath) -> Self {
        Self(path.path(), ())
    }
}

trait KeyTypeId {
    fn key_type_id(&self) -> TypeId;
}

impl<T: Pointee + AnyItem + ?Sized> KeyTypeId for KeyPath<T> {
    default fn key_type_id(&self) -> TypeId {
        let ptr = std::ptr::from_raw_parts::<T>(std::ptr::null(), self.1);
        ptr.item_type_id()
    }
}

impl<T: Pointee<Metadata = ()> + AnyItem + ?Sized> KeyTypeId for KeyPath<T> {
    fn key_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}
