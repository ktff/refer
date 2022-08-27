use super::{AnyKey, AnyRef, Item, Key, MutShell, Prefix, RefShell};

// NOTE: Generic naming is intentionally here so to trigger naming conflicts to discourage
//       implementations from implementing all *Collection traits on the same type.

/// A collection of entities.
///
/// Entity is an item in a shell.
/// Entities are connected to each other through shells.
///
/// Collection can be split into collections of items and shells.
pub trait Collection<T: ?Sized + 'static>: AnyCollection {
    type Items: ItemCollection<T>;

    type Shells: ShellCollection<T>;

    type Iter<'a>: Iterator<Item = (Key<T>, &'a T, <Self::Shells as ShellCollection<T>>::Ref<'a>)>
    where
        Self: 'a;

    type MutIter<'a>: Iterator<
        Item = (
            Key<T>,
            &'a mut T,
            <Self::Shells as ShellCollection<T>>::Ref<'a>,
        ),
    >
    where
        Self: 'a;

    /// Reserves slot for item.
    /// None if collection is out of keys.
    /// Must be eventually canceled or fulfilled, otherwise memory can be leaked.
    /// Consecutive calls without canceling or fulfilling have undefined behavior.
    fn reserve(&mut self, item: &T) -> Option<Key<T>>;

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

    /// Err if collection is out of keys.
    /// May panic if some of the references don't exist or if prefix doesn't exist.
    fn add(&mut self, prefix: Option<Prefix>, item: T) -> Result<Key<T>, T>
    where
        T: Item + Sized,
    {
        assert!(prefix.is_none(), "Not yet implemented");

        // Allocate slot
        let key = if let Some(key) = self.reserve(&item) {
            key
        } else {
            return Err(item);
        };

        // Update connections
        if !super::util::add_references(self.shells_mut(), key, &item) {
            // Failed

            // Deallocate slot
            self.cancel(key);

            return Err(item);
        }

        // Add item & shell
        self.fulfill(key, item);

        Ok(key)
    }

    /// Err if some of the references don't exist.
    fn set(&mut self, key: Key<T>, set: T) -> Result<T, T>
    where
        T: Item + Sized,
    {
        let (items, shells) = self.split_mut();
        let old = if let Some(item) = items.get_mut(key) {
            item
        } else {
            // No item
            return Err(set);
        };

        // Update connections
        if !super::util::update_diff(shells, key, old, &set) {
            // Failed
            return Err(set);
        }

        // Replace item
        Ok(std::mem::replace(old, set))
    }

    /// Some if item exists.
    fn take(&mut self, key: Key<T>) -> Option<T>
    where
        T: Item + Sized,
    {
        let mut remove = Vec::new();

        // Update connections
        super::util::remove_references(self, key.into(), &mut remove)?;
        // Deallocate
        let item = self.unfill(key).expect("Should exist");

        // Recursive remove
        while let Some(rf) = remove.pop() {
            // Update connections
            if super::util::remove_references(self, rf, &mut remove).is_some() {
                // Deallocate
                let _ = self.unfill_any(rf);
            }
        }

        Some(item)
    }

    /// Some if item exists.
    fn get(&self, key: Key<T>) -> Option<(&T, <Self::Shells as ShellCollection<T>>::Ref<'_>)> {
        let (items, shells) = self.split();
        Some((items.get(key)?, shells.get(key)?))
    }

    /// Some if item exists.
    fn get_mut(
        &mut self,
        key: Key<T>,
    ) -> Option<(&mut T, <Self::Shells as ShellCollection<T>>::Ref<'_>)> {
        let (items, shells) = self.split_mut();
        Some((items.get_mut(key)?, shells.get(key)?))
    }

    /// Consistent ascending order.
    fn iter(&self) -> Self::Iter<'_>;

    /// Consistent ascending order.
    fn iter_mut(&mut self) -> Self::MutIter<'_>;

    fn shells(&self) -> &Self::Shells {
        self.split().1
    }

    fn shells_mut(&mut self) -> &mut Self::Shells {
        self.split_mut().1
    }

    fn items(&self) -> &Self::Items {
        self.split().0
    }

    fn items_mut(&mut self) -> &mut Self::Items {
        self.split_mut().0
    }

    fn split(&self) -> (&Self::Items, &Self::Shells);

    /// Splits to views of items and shells
    fn split_mut(&mut self) -> (&mut Self::Items, &mut Self::Shells);
}

pub trait AnyCollection: KeyCollection {
    type AnyItems: AnyItemCollection;

    type AnyShells: AnyShellCollection;

    /// Frees if it exists.
    fn unfill_any(&mut self, key: AnyKey) -> bool;

    /// Splits to views of items and shells
    fn split_any_mut(&mut self) -> (&mut Self::AnyItems, &mut Self::AnyShells);
}

/// Polly ItemCollection can split &mut self to multiple &mut views each with set of types that don't overlap.
pub trait ItemCollection<T: ?Sized + 'static>: AnyItemCollection {
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

pub trait AnyItemCollection {
    fn contains(&self, key: AnyKey) -> bool;

    /// None if item doesn't exist.
    fn references(&self, key: AnyKey) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>>;

    /// False if reference still exists.
    fn remove_reference(&mut self, key: AnyKey, rf: AnyKey) -> bool;
}

/// Polly ShellCollection can't split this.
pub trait ShellCollection<T: ?Sized + 'static>: AnyShellCollection {
    type Ref<'a>: RefShell<'a, T = T>
    where
        Self: 'a;

    type Mut<'a>: MutShell<'a>
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
}

pub trait AnyShellCollection {
    fn contains(&self, key: AnyKey) -> bool;

    /// Fails if key->shell doesn't exist.
    fn add_from(&mut self, key: AnyKey, rf: AnyKey) -> bool;

    /// None if it doesn't exist.
    fn from(&self, key: AnyKey) -> Option<Box<dyn Iterator<Item = AnyKey> + '_>>;

    /// Fails if key->shell doesn't exist or if shell[rf] doesn't exist.
    fn remove_from(&mut self, key: AnyKey, rf: AnyKey) -> bool;
}

pub trait KeyCollection {
    /// Prefix
    fn prefix(&self) -> Option<Prefix>;

    fn first<I: ?Sized + 'static>(&self) -> Option<Key<I>>;

    /// Returns following key after given in ascending order.
    fn next<I: ?Sized + 'static>(&self, key: Key<I>) -> Option<Key<I>>;
}
