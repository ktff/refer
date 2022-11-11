//! Containers can panic, if you try to use a key that was not produced at any
//! point by that container.

use std::{
    any::{Any, TypeId},
    cell::SyncUnsafeCell,
    collections::HashSet,
};

use super::{AnyItem, AnyShell, AnySubKey, ReservedKey, Shell, SubKey};

/// A family of containers.
pub trait ContainerFamily: Send + Sync + 'static {
    type C<T: AnyItem>: AnyContainer + 'static;

    fn new<T: AnyItem>(key_len: u32) -> Self::C<T>;
}

/// It's responsibility is to contain items and shells, not to manage access to them.
pub trait Container<T: AnyItem>: Allocator<T> + AnyContainer {
    type GroupItem: Any;

    type Shell: Shell<T = T>;

    type SlotIter<'a>: Iterator<
            Item = (
                SubKey<T>,
                (&'a SyncUnsafeCell<T>, &'a Self::GroupItem),
                &'a SyncUnsafeCell<Self::Shell>,
                &'a Self::Alloc,
            ),
        > + Send
    where
        Self: 'a;

    fn get_slot(
        &self,
        key: SubKey<T>,
    ) -> Option<(
        (&SyncUnsafeCell<T>, &Self::GroupItem),
        &SyncUnsafeCell<Self::Shell>,
        &Self::Alloc,
    )>;

    /// Iterates in ascending order of key.
    /// UNSAFE: Guarantees no slot is returned twice in returned iterator.
    unsafe fn iter_slot(&self) -> Option<Self::SlotIter<'_>>;
}

pub trait AnyContainer: Any + Sync + Send {
    fn any_get_slot(
        &self,
        key: AnySubKey,
    ) -> Option<(
        (&SyncUnsafeCell<dyn AnyItem>, &dyn Any),
        &SyncUnsafeCell<dyn AnyShell>,
        &dyn std::alloc::Allocator,
    )>;

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
pub trait Allocator<T: 'static>: Send + Sync {
    /// Allocator used for items and shells.
    type Alloc: std::alloc::Allocator + 'static;

    // TODO: Some descriptive name
    /// Allocator can select placement for item based on this.
    type R: Copy;

    /// Reserves slot for item based on R.
    /// None if collection is out of keys or memory.
    fn reserve(&mut self, item: Option<&T>, r: Self::R) -> Option<(ReservedKey<T>, &Self::Alloc)>;

    /// Cancels reservation for item.
    /// Panics if there is no reservation.
    fn cancel(&mut self, key: ReservedKey<T>);

    /// Fulfills reservation.
    /// Panics if there is no reservation.
    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T>
    where
        T: Sized;

    /// Frees and returns item if it exists
    fn unfill(&mut self, key: SubKey<T>) -> Option<(T, &Self::Alloc)>
    where
        T: Sized;
}
