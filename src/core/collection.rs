use std::mem::MaybeUninit;

use super::{Item, Key, MutEntity, MutShell, Prefix, RefEntity, RefShell};

// NOTE: Generic naming is intentionally here so to trigger naming conflicts to discourage
//       implementations from implementing all *Collection traits on the same type.

/// A collection of entities.
///
/// Entity is an item in a shell.
/// Entities are connected to each other through shells.
///
/// Collection can be split into collections of items and shells.
pub trait Collection<T: ?Sized + 'static>: KeyCollection {
    type Items: ItemCollection<T>;

    type Shells: ShellCollection<T>;

    type Ref<'a>: RefEntity<'a, T = T>
    where
        Self: 'a;

    type Mut<'a>: MutEntity<'a, T = T>
    where
        Self: 'a;

    type Iter<'a>: Iterator<Item = (Key<T>, Self::Ref<'a>)>
    where
        Self: 'a;

    type MutIter<'a>: Iterator<Item = (Key<T>, Self::Mut<'a>)>
    where
        Self: 'a;

    /// None if collection is out of keys.
    ///
    /// Unsafe since caller must ensure to properly initialize data in all cases.
    /// Returned byte array is of size and alignment of the item.
    ///
    /// Mainly used for composite collections.
    unsafe fn allocate(&mut self, item: &T) -> Option<(Key<T>, &mut [MaybeUninit<u8>])>;

    /// Some if item exists.
    /// Will call free on the item and won't drop it, just free it's memory.
    /// Mainly used for composite collections.
    unsafe fn free<R>(
        &mut self,
        key: Key<T>,
        free: impl FnOnce(&[MaybeUninit<u8>]) -> R,
    ) -> Option<R>;

    /// Err if collection is out of keys.
    /// May panic if some of the references don't exist or if prefix doesn't exist.
    fn add(&mut self, prefix: Option<Prefix>, item: T) -> Result<Key<T>, T>
    where
        T: Item + Sized;

    /// None if collection is out of keys.
    fn add_copy(&mut self, prefix: Option<Prefix>, item: &T) -> Option<Key<T>>
    where
        T: Item + Copy;

    /// Err if some of the references don't exist.
    fn set(&mut self, key: Key<T>, set: T) -> Result<T, T>
    where
        T: Item + Sized;

    /// True if set. False if some of it's references don't exist.
    fn set_copy(&mut self, key: Key<T>, set: &T) -> bool
    where
        T: Item + Copy;

    /// True Item existed so it was removed.
    fn remove(&mut self, key: Key<T>) -> bool
    where
        T: Item;

    /// Some if item exists.
    fn take(&mut self, key: Key<T>) -> Option<T>
    where
        T: Item + Sized;

    /// Some if item exists.
    fn get(&self, key: Key<T>) -> Option<Self::Ref<'_>>;

    /// Some if item exists.
    fn get_mut(&mut self, key: Key<T>) -> Option<Self::Mut<'_>>;

    /// Consistent ascending order.
    fn iter(&self) -> Self::Iter<'_>;

    /// Consistent ascending order.
    fn iter_mut(&mut self) -> Self::MutIter<'_>;

    fn shells(&self) -> &Self::Shells;

    fn shells_mut(&mut self) -> &mut Self::Shells {
        self.split().1
    }

    fn items(&self) -> &Self::Items;

    fn items_mut(&mut self) -> &mut Self::Items {
        self.split().0
    }

    /// Splits to views of items and shells
    fn split(&mut self) -> (&mut Self::Items, &mut Self::Shells);
}

/// Polly ItemCollection can split &mut self to multiple &mut views each with set of types that don't overlap.
pub trait ItemCollection<T: ?Sized + 'static> {
    type Iter<'a>: Iterator<Item = (Key<T>, &'a T)>
    where
        Self: 'a;

    type MutIter<'a>: Iterator<Item = (Key<T>, &'a mut T)>
    where
        Self: 'a;

    /// Some if item exists.
    fn get(&self, key: Key<T>) -> Option<&T>;

    /// Some if item exists.
    fn get_mut(&mut self, key: Key<T>) -> Option<&mut T>;

    /// Consistent ascending order.
    fn iter(&self) -> Self::Iter<'_>;

    /// Consistent ascending order.
    fn iter_mut(&mut self) -> Self::MutIter<'_>;
}

/// Polly ShellCollection can't split this.
pub trait ShellCollection<T: ?Sized + 'static> {
    type MutColl<'a>: MutShellCollection<'a, T>
    where
        Self: 'a;

    type Ref<'a>: RefShell<'a, T = T>
    where
        Self: 'a;

    type Mut<'a>: MutShell<'a, T = T>
    where
        Self: 'a;

    type Iter<'a>: Iterator<Item = (Key<T>, Self::Ref<'a>)>
    where
        Self: 'a;

    type MutIter<'a>: Iterator<Item = (Key<T>, Self::Mut<'a>)>
    where
        Self: 'a;

    /// Some if item exists.
    fn get(&self, key: Key<T>) -> Option<Self::Ref<'_>>;

    /// Some if item exists.
    fn get_mut(&mut self, key: Key<T>) -> Option<Self::Mut<'_>>;

    /// Consistent ascending order.
    fn iter(&self) -> Self::Iter<'_>;

    /// Consistent ascending order.
    fn iter_mut(&mut self) -> Self::MutIter<'_>;

    fn mut_coll(&mut self) -> Self::MutColl<'_>;
}

/// Enables holding on to multiple MutShells at the same time.
/// To enable that it usually won't enable viewing of the data.
pub trait MutShellCollection<'a, T: ?Sized + 'static>: 'a {
    type Mut<'b>: MutShell<'b, T = T>
    where
        Self: 'b;

    /// Some if shell exists.
    fn get_mut(&self, key: Key<T>) -> Option<Self::Mut<'_>>;
}

pub trait KeyCollection {
    /// Prefix
    fn prefix(&self) -> Option<Prefix>;

    fn first<I: ?Sized + 'static>(&self) -> Option<Key<I>>;

    /// Returns following key after given in ascending order.
    fn next<I: ?Sized + 'static>(&self, key: Key<I>) -> Option<Key<I>>;
}
