use crate::core::{AnyItem, DynItem};

use super::{Key, Owned};

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

    pub fn sub(mut self) -> (Key<Owned, T>, Option<Self>) {
        if self.count == 1 {
            (self.key, None)
        } else {
            self.count -= 1;
            // SAFETY: this is safe since for each of this we've forgot one before.
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

// From
impl<T: DynItem + ?Sized> From<Key<Owned, T>> for MultiOwned<T> {
    fn from(key: Key<Owned, T>) -> Self {
        Self::new(key)
    }
}
