use super::{AnyContainer, AnyItem, AnyKey, AnyRef, Container, Item, Key, Prefix, Shell};

// NOTE: Generic naming is intentionally here so to trigger naming conflicts to discourage
//       implementations from implementing all *Collection traits on the same type.

/// A collection of entities.
///
/// Entity is an item in a shell.
/// Entities are connected to each other through shells.
///
/// Collection can be split into collections of items and shells.
pub trait Collection<T: AnyItem + ?Sized>: Container<T> + AnyCollection {
    type Items: ItemCollection<T>;

    type Shells: ShellCollection<T>;

    type Iter<'a>: Iterator<
        Item = (
            Key<T>,
            &'a T,
            &'a <Self::Shells as ShellCollection<T>>::Shell,
        ),
    >
    where
        Self: 'a;

    type MutIter<'a>: Iterator<
        Item = (
            Key<T>,
            &'a mut T,
            &'a <Self::Shells as ShellCollection<T>>::Shell,
        ),
    >
    where
        Self: 'a;

    /// Err if collection is out of keys.
    /// May panic if some of the references don't exist or if prefix doesn't exist.
    fn add(&mut self, prefix: Option<Prefix>, item: T) -> Result<Key<T>, T>
    where
        T: Item + Sized,
    {
        assert!(prefix.is_none(), "Not yet implemented");

        // Allocate slot
        let key = if let Some(key) = self.reserve() {
            key
        } else {
            return Err(item);
        };

        // Update connections
        if !super::util::add_references(self.shells_mut(), key.key(), &item) {
            // Failed

            // Deallocate slot
            self.cancel(key);

            return Err(item);
        }

        // Add item & shell
        Ok(self.fulfill(key, item))
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
    fn get(&self, key: Key<T>) -> Option<(&T, &<Self::Shells as ShellCollection<T>>::Shell)> {
        let (items, shells) = self.split();
        Some((items.get(key)?, shells.get(key)?))
    }

    /// Some if item exists.
    fn get_mut(
        &mut self,
        key: Key<T>,
    ) -> Option<(&mut T, &<Self::Shells as ShellCollection<T>>::Shell)> {
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

pub trait AnyCollection: AnyContainer {
    type AnyItems: AnyItemCollection;

    type AnyShells: AnyShellCollection;

    /// Frees if it exists.
    fn unfill_any(&mut self, key: AnyKey) -> bool {
        self.any_unfill(key)
    }

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

pub trait AnyItemCollection: AnyContainer {
    fn contains(&self, key: AnyKey) -> bool {
        self.any_get_slot(key).is_some()
    }

    /// None if item doesn't exist.
    fn references(&self, key: AnyKey) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>> {
        self.any_get_slot(key).map(|(item, _)| {
            // This is safe since self: AnyItemCollection has exclusive access to shells
            // and we have here shared access to self.
            let item = unsafe { &*item.get() };
            item.references_any()
        })
    }

    /// False if reference still exists.
    fn remove_reference(&mut self, key: AnyKey, rf: AnyKey) -> bool {
        self.any_get_slot(key).map_or(true, |(item, _)| {
            // This is safe since self: AnyItemCollection has exclusive access to shells
            // and we have here exclusive access to self.
            let item = unsafe { &mut *item.get() };
            item.remove_reference(rf)
        })
    }
}

/// Polly ShellCollection can't split this.
pub trait ShellCollection<T: ?Sized + 'static>: AnyShellCollection {
    type Shell: Shell<T = T>;

    type Iter<'a>: Iterator<Item = (Key<T>, &'a Self::Shell)>
    where
        Self: 'a;

    type MutIter<'a>: Iterator<Item = (Key<T>, &'a mut Self::Shell)>
    where
        Self: 'a;

    /// Some if item exists.
    fn get(&self, key: Key<T>) -> Option<&Self::Shell>;

    /// Some if item exists.
    fn get_mut(&mut self, key: Key<T>) -> Option<&mut Self::Shell>;

    /// Consistent ascending order.
    fn iter(&self) -> Self::Iter<'_>;

    /// Consistent ascending order.
    fn iter_mut(&mut self) -> Self::MutIter<'_>;
}

pub trait AnyShellCollection: AnyContainer {
    fn contains(&self, key: AnyKey) -> bool {
        self.any_get_slot(key).is_some()
    }

    /// Fails if key->shell doesn't exist.
    fn add_from(&mut self, key: AnyKey, rf: AnyKey) -> bool {
        self.any_get_slot(key)
            .map(|(_, shell)| {
                // This is safe since self: AnyShellCollection has exclusive access to shells
                // and we have here exclusive access to self.
                let shell = unsafe { &mut *shell.get() };
                shell.add_from(rf);
            })
            .is_some()
    }

    /// None if it doesn't exist.
    fn from(&self, key: AnyKey) -> Option<Box<dyn Iterator<Item = AnyKey> + '_>> {
        self.any_get_slot(key).map(|(_, shell)| {
            // This is safe since self: AnyShellCollection has exclusive access to shells
            // and we have here shared access to self.
            let shell = unsafe { &*shell.get() };
            shell.from_any()
        })
    }

    /// Fails if key->shell doesn't exist or if shell[rf] doesn't exist.
    fn remove_from(&mut self, key: AnyKey, rf: AnyKey) -> bool {
        self.any_get_slot(key).map_or(false, |(_, shell)| {
            // This is safe since self: AnyShellCollection has exclusive access to shells
            // and we have here exclusive access to self.
            let shell = unsafe { &mut *shell.get() };
            shell.remove_from(rf)
        })
    }
}
