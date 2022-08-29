use super::{AnyItem, Index, SubKey};
use std::mem::forget;

/// It's responsibility is to manage allocation/placement/deallocation of item
pub trait Allocator<T: AnyItem + ?Sized> {
    /// Reserves slot for item.
    /// None if collection is out of keys or memory.
    fn reserve(&mut self, item: &T) -> Option<ReservedKey<T>>;

    /// Cancels reservation for item.
    /// Panics if there is no reservation.
    fn cancel(&mut self, key: ReservedKey<T>);

    /// Fulfills reservation.
    /// Panics if there is no reservation.
    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T>
    where
        T: Sized;

    /// Frees and returns item if it exists
    fn unfill(&mut self, key: SubKey<T>) -> Option<T>
    where
        T: Sized;
}

/// Helps to make allocate process easier to do correctly.
pub struct ReservedKey<T: ?Sized>(SubKey<T>);

impl<T: ?Sized> ReservedKey<T> {
    /// Should only be constructed by Containers.
    pub fn new(key: SubKey<T>) -> Self {
        Self(key)
    }

    pub fn key(&self) -> SubKey<T> {
        self.0
    }

    /// Adds prefix of given len.
    pub fn with_prefix(self, prefix_len: u32, prefix: Index) -> Self {
        Self(self.0.with_prefix(prefix_len, prefix))
    }

    /// Splits of prefix of given len and suffix.
    pub fn split_prefix(self, prefix_len: u32) -> (Index, Self) {
        let (prefix, suffix) = self.0.split_prefix(prefix_len);
        (prefix, Self(suffix))
    }

    /// Splits of prefix of given len and suffix.
    /// Fails if there is no suffix.
    pub fn split_prefix_try(self, prefix_len: u32) -> (Index, Option<Self>) {
        let (prefix, suffix) = self.0.split_prefix_try(prefix_len);
        (prefix, suffix.map(|suffix| Self(suffix)))
    }
    /// Only proper way to finish reservation.
    pub fn take(self) -> SubKey<T> {
        let key = self.0;
        forget(self);
        key
    }
}

impl<T: ?Sized> Drop for ReservedKey<T> {
    fn drop(&mut self) {
        // TODO: Log that it was leaked.
    }
}
