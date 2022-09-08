//! Containers can panic, if you try to use a key that was not produced at any
//! point by that container.

use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashSet,
};

use super::{AnyItem, AnyShell, AnySubKey, ReservedKey, Shell, SubKey};

/// A family of containers.
pub trait ContainerFamily: 'static {
    type C<T: AnyItem>: AnyContainer + 'static;

    fn new<T: AnyItem>(key_len: u32) -> Self::C<T>;
}

/// It's responsibility is to contain items and shells, not to manage access to them.
/// UNSAFE: It is unsafe for Containers to be Sync.
pub trait Container<T: AnyItem>: AnyContainer {
    type Shell: Shell<T = T>;

    type SlotIter<'a>: Iterator<Item = (SubKey<T>, &'a UnsafeCell<T>, &'a UnsafeCell<Self::Shell>)>
    where
        Self: 'a;

    fn get_slot(&self, key: SubKey<T>) -> Option<(&UnsafeCell<T>, &UnsafeCell<Self::Shell>)>;

    /// Iterates in ascending order of key.
    /// UNSAFE: Guarantees no slot is returned twice in returned iterator.
    unsafe fn iter_slot(&self) -> Option<Self::SlotIter<'_>>;
}

/// UNSAFE: It is unsafe for Containers to be Sync.
pub trait AnyContainer: Any {
    fn any_get_slot(
        &self,
        key: AnySubKey,
    ) -> Option<(&UnsafeCell<dyn AnyItem>, &UnsafeCell<dyn AnyShell>)>;

    fn unfill_any(&mut self, key: AnySubKey);

    /// Returns first key for given type
    fn first(&self, key: TypeId) -> Option<AnySubKey>;

    /// Returns following key after given in ascending order
    /// for the same type.
    fn next(&self, key: AnySubKey) -> Option<AnySubKey>;

    /// All types in the container.
    fn types(&self) -> HashSet<TypeId>;
}

/// It's responsibility is to manage allocation/placement/deallocation of item
pub trait Allocator<T: 'static> {
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
