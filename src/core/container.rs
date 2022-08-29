use std::{any::Any, cell::UnsafeCell};

use super::{AnyItem, AnyShell, AnySubKey, Shell, SubKey};

/// It's responsibility is to contain items and shells, not to manage access to them.
pub trait Container<T: AnyItem + ?Sized>: AnyContainer + KeyContainer {
    type Shell: Shell<T = T>;

    type CellIter<'a>: Iterator<Item = (SubKey<T>, &'a UnsafeCell<T>, &'a UnsafeCell<Self::Shell>)>
    where
        Self: 'a;

    fn get_slot(&self, key: SubKey<T>) -> Option<(&UnsafeCell<T>, &UnsafeCell<Self::Shell>)>;

    /// Even if some it may be empty.
    fn iter_slot(&self) -> Option<Self::CellIter<'_>>;
}

pub trait AnyContainer: Any {
    fn any_get_slot(
        &self,
        key: AnySubKey,
    ) -> Option<(&UnsafeCell<dyn AnyItem>, &UnsafeCell<dyn AnyShell>)>;

    /// Frees if it exists.
    fn any_unfill(&mut self, key: AnySubKey) -> bool;
}

pub trait KeyContainer {
    // TODO: Enable for I: ?Sized once you have that figured out
    fn first<I: AnyItem>(&self) -> Option<SubKey<I>>;

    // TODO: Enable for I: ?Sized once you have that figured out
    /// Returns following key after given in ascending order.
    fn next<I: AnyItem>(&self, key: SubKey<I>) -> Option<SubKey<I>>;
}
