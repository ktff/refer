use std::{any::Any, cell::UnsafeCell, mem::forget};

use super::{AnyItem, AnyKey, AnyShell, Key, Prefix, Shell};

/// It's responsibility is to contain items and shells, not to manage access to them.
pub trait Container<T: AnyItem + ?Sized>: AnyContainer + KeyContainer {
    type Shell: Shell<T = T>;

    type CellIter<'a>: Iterator<Item = (Key<T>, &'a UnsafeCell<T>, &'a UnsafeCell<Self::Shell>)>
    where
        Self: 'a;

    /// Reserves slot for item.
    /// None if collection is out of keys.
    fn reserve(&mut self) -> Option<ReservedKey<T>>;

    /// Cancels reservation for item.
    /// Panics if there is no reservation.
    fn cancel(&mut self, key: ReservedKey<T>);

    /// Fulfills reservation.
    /// Panics if there is no reservation.
    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> Key<T>
    where
        T: Sized;

    /// Frees and returns item if it exists
    fn unfill(&mut self, key: Key<T>) -> Option<T>
    where
        T: Sized;

    fn get_slot(&self, key: Key<T>) -> Option<(&UnsafeCell<T>, &UnsafeCell<Self::Shell>)>;

    /// Even if some it may be empty.
    fn iter_slot(&self) -> Option<Self::CellIter<'_>>;
}

pub trait AnyContainer: Any {
    fn any_get_slot(
        &self,
        key: AnyKey,
    ) -> Option<(&UnsafeCell<dyn AnyItem>, &UnsafeCell<dyn AnyShell>)>;

    /// Frees if it exists.
    fn any_unfill(&mut self, key: AnyKey) -> bool;
}

pub trait KeyContainer {
    /// Prefix
    fn prefix(&self) -> Option<Prefix>;

    // TODO: Enable for I: ?Sized once you have that figured out
    fn first<I: AnyItem>(&self) -> Option<Key<I>>;

    // TODO: Enable for I: ?Sized once you have that figured out
    /// Returns following key after given in ascending order.
    fn next<I: AnyItem>(&self, key: Key<I>) -> Option<Key<I>>;
}

/// Helps to make allocate process easier to do correctly.
pub struct ReservedKey<T: ?Sized>(Key<T>);

impl<T: ?Sized> ReservedKey<T> {
    /// Should only be constructed by Containers.
    pub fn new(key: Key<T>) -> Self {
        Self(key)
    }

    pub fn key(&self) -> Key<T> {
        self.0
    }

    /// Only proper way to finish reservation.
    pub fn take(self) -> Key<T> {
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
