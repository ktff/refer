use super::{Key, LeafPath, LocalityKey, LocalityPath, LocalityRegion, Path, Ptr, RegionPath};
use crate::core::{AnyItem, DynItem};
use std::{
    any::{self},
    fmt::{self},
    hash::{Hash, Hasher},
    marker::{PhantomData, Unsize},
};

pub type AnyPath = KeyPath<dyn AnyItem>;

#[repr(transparent)]
pub struct KeyPath<T: DynItem + ?Sized>(Path, PhantomData<&'static T>);

impl<T: AnyItem + ?Sized> KeyPath<T> {
    /// Constructors of KeyPath should strive to guarantee that T can indeed be on Path.
    pub fn new(path: Path) -> Self {
        KeyPath(path, PhantomData)
    }
}

impl AnyPath {
    pub fn new_any(path: Path) -> Self {
        Self(path, PhantomData)
    }
}

impl<T: DynItem + ?Sized> KeyPath<T> {
    #[inline(always)]
    pub fn path(&self) -> Path {
        self.0
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

    pub fn upcast<U: DynItem + ?Sized>(self) -> KeyPath<U>
    where
        T: Unsize<U>,
    {
        KeyPath(self.0, PhantomData)
    }

    /// True if self start of given path.
    pub fn contains(self, path: KeyPath<T>) -> bool {
        self.0.contains(path.0)
    }

    /// True if self start of given key.
    pub fn contains_key(self, key: Key<Ptr, T>) -> bool {
        self.0.contains_index(key.index())
    }
}

impl<T: DynItem + ?Sized> LocalityPath for KeyPath<T> {
    fn map(&self, region: RegionPath) -> Option<LocalityRegion> {
        self.0.map(region)
    }

    fn upcast(&self) -> &dyn LocalityPath {
        self
    }
}

impl<T: DynItem + ?Sized> Eq for KeyPath<T> {}

impl<T: DynItem + ?Sized, U: DynItem + ?Sized> PartialEq<KeyPath<U>> for KeyPath<T> {
    fn eq(&self, other: &KeyPath<U>) -> bool {
        self.0 == other.0
    }
}

impl<T: DynItem + ?Sized> Hash for KeyPath<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// SAFETY: KeyPath only contains Path, which is Send.
unsafe impl<T: DynItem + ?Sized> Send for KeyPath<T> where Path: Send {}
/// SAFETY: KeyPath only contains Path, which is Sync.
unsafe impl<T: DynItem + ?Sized> Sync for KeyPath<T> where Path: Sync {}

impl<T: DynItem + ?Sized> Copy for KeyPath<T> {}

impl<T: DynItem + ?Sized> Clone for KeyPath<T> {
    fn clone(&self) -> Self {
        KeyPath(self.0, self.1)
    }
}

impl<T: DynItem + ?Sized> fmt::Debug for KeyPath<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}:{}", self.0, any::type_name::<T>())
    }
}

impl<T: DynItem + ?Sized> fmt::Display for KeyPath<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.0, any::type_name::<T>())
    }
}

impl<T: DynItem + ?Sized> Default for KeyPath<T> {
    fn default() -> Self {
        KeyPath(Path::default(), PhantomData)
    }
}

impl<P: Copy, T: DynItem + ?Sized> From<Key<P, T>> for KeyPath<T> {
    fn from(key: Key<P, T>) -> Self {
        KeyPath(key.path(), PhantomData)
    }
}

impl<T: AnyItem + ?Sized> From<LeafPath> for KeyPath<T> {
    fn from(path: LeafPath) -> Self {
        Self::new(path.path())
    }
}

impl<T: AnyItem + ?Sized> From<LocalityKey> for KeyPath<T> {
    fn from(path: LocalityKey) -> Self {
        Self::new(path.path())
    }
}
