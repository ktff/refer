use std::cell::UnsafeCell;

use super::{AnyItem, AnyKey, AnyShell, Key, Prefix, Shell};

pub trait Container<T: ?Sized + 'static>: AnyContainer {
    type Shell: Shell<T = T>;

    type CellIter<'a>: Iterator<Item = (Key<T>, &'a UnsafeCell<T>, &'a UnsafeCell<Self::Shell>)>
    where
        Self: 'a;

    /// Reserves slot for item.
    /// None if collection is out of keys.
    /// Must be eventually canceled or fulfilled, otherwise memory can be leaked.
    /// Consecutive calls without canceling or fulfilling have undefined behavior.
    fn reserve(&mut self) -> Option<Key<T>>;

    /// Cancels reservation for item.
    /// Panics if an item is present.
    fn cancel(&mut self, key: Key<T>);

    /// Fulfills reservation.
    /// Panics if there is no reservation, if it's already fulfilled,
    /// or may panic if this item differs from one during reservation.
    fn fulfill(&mut self, key: Key<T>, item: T)
    where
        T: Sized;

    /// Frees and returns item if it exists
    fn unfill(&mut self, key: Key<T>) -> Option<T>
    where
        T: Sized;

    fn get_slot(&self, key: Key<T>) -> Option<(&UnsafeCell<T>, &UnsafeCell<Self::Shell>)>;

    fn iter_slot(&self) -> Self::CellIter<'_>;
}

pub trait AnyContainer: KeyContainer {
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

    fn first<I: ?Sized + 'static>(&self) -> Option<Key<I>>;

    /// Returns following key after given in ascending order.
    fn next<I: ?Sized + 'static>(&self, key: Key<I>) -> Option<Key<I>>;
}
