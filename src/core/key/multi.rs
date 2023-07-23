use crate::core::{AnyItem, DynItem};

use super::{Key, Owned};

#[derive(Debug)]
pub struct MultiOwned<T: DynItem + ?Sized = dyn AnyItem> {
    key: Key<Owned, T>,
    count: usize,
}

impl<T: DynItem + ?Sized> MultiOwned<T> {
    pub fn new(key: Key<Owned, T>) -> Self {
        Self { key, count: 1 }
    }

    pub fn key(&self) -> &Key<Owned, T> {
        &self.key
    }

    pub fn count(&self) -> usize {
        self.count
    }

    /// Adds key, panics if not the same key.
    pub fn add(&mut self, key: Key<Owned, T>) {
        assert_eq!(self.key, key);
        std::mem::forget(key);
        self.count += 1;
    }

    /// Appends other to self.
    /// Panics if not the same key.
    pub fn append(&mut self, other: Self) {
        assert_eq!(self.key, other.key);
        self.count += other.count;
        std::mem::forget(other);
    }

    /// Removes one key, returns None if count is 1.
    pub fn take(&mut self) -> Option<Key<Owned, T>> {
        if self.count == 1 {
            None
        } else {
            self.count -= 1;
            // SAFETY: this is safe since for each one of this we've forgot one before.
            let key = unsafe { Key::new_owned(self.key.index()) };
            Some(key)
        }
    }

    pub fn sub(mut self) -> (Key<Owned, T>, Option<Self>) {
        if self.count == 1 {
            (self.key, None)
        } else {
            self.count -= 1;
            // SAFETY: this is safe since for each one of this we've forgot one before.
            let key = unsafe { Key::new_owned(self.key.index()) };
            (
                key,
                Some(Self {
                    key: self.key,
                    count: 1,
                }),
            )
        }
    }
}

impl MultiOwned {
    /// Make assumption that this is Key for T.
    pub fn assume<T: DynItem + ?Sized>(self) -> MultiOwned<T> {
        MultiOwned {
            key: self.key.assume(),
            count: self.count,
        }
    }
}

// From
impl<T: DynItem + ?Sized> From<Key<Owned, T>> for MultiOwned<T> {
    fn from(key: Key<Owned, T>) -> Self {
        Self::new(key)
    }
}

// Impl Eq,PartialEq,Ord, and PartialOrd for MultiOwned based on key
impl<T: DynItem + ?Sized> Eq for MultiOwned<T> {}

impl<T: DynItem + ?Sized> PartialEq for MultiOwned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T: DynItem + ?Sized> Ord for MultiOwned<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl<T: DynItem + ?Sized> PartialOrd for MultiOwned<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.key.cmp(&other.key))
    }
}
