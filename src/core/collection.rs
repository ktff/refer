use super::{permit, CollectionError, Item, ItemContext, Key, Slot};

/// A collection of entities.
///
/// Entity is an item in a shell.
/// Entities are connected to each other through shells.
///
/// Collection can be split into collections of items and shells.
///
/// Keys are brittle on this level so they should be used in/for limited amount.
/// For any other use case add an Item that will manage keys for you.
///
/// On Error, Collection must be in valid state.
/// On panic, Collection may be in invalid state.
///
/// On error caused by external state, returns Error. (Argument,call,op)
/// On error caused by internal state, panics.
pub trait Collection<T: Item> {
    // type Shell: Shell<T = T>;

    fn add(&mut self, locality: T::Locality, item: T) -> Result<Key<T>, CollectionError>;

    /// Removes previous item and sets new one in his place while updating references accordingly.
    fn replace_item(&mut self, key: Key<T>, item: T) -> Result<T, CollectionError>;

    fn displace(&mut self, key: Key<T>, locality: T::Locality) -> Result<Key<T>, CollectionError>;

    /// Moves item and removes shell.
    fn displace_item(
        &mut self,
        key: Key<T>,
        locality: T::Locality,
    ) -> Result<Key<T>, CollectionError>;

    /// Moves shell from `from` to `to` so that all references are now pointing to `to`.
    /// May have side effects that invalidate some Keys.
    fn displace_shell(&mut self, from: Key<T>, to: Key<T>) -> Result<(), CollectionError>;

    fn duplicate(&mut self, key: Key<T>, to: T::Locality) -> Result<Key<T>, CollectionError> {
        let to = self.duplicate_item(key, to)?;
        match self.duplicate_shell(key, to) {
            Ok(()) => Ok(to),
            Err(error) => {
                self.remove(to).expect("Should be valid key");
                Err(error)
            }
        }
    }

    /// Duplicates item to locality.
    fn duplicate_item(&mut self, key: Key<T>, to: T::Locality) -> Result<Key<T>, CollectionError>;

    /// Duplicates shell from `from` to `to` so that all references to `from` now also point to `to`.
    fn duplicate_shell(&mut self, from: Key<T>, to: Key<T>) -> Result<(), CollectionError>;

    /// T has it's local data dropped.
    /// May have side effects that invalidate some Keys.
    fn remove(&mut self, key: Key<T>) -> Result<T, CollectionError>;
}

// ************************************* WIP ************************************* //

pub trait RefAddition<T: Item> {
    //? NOTE: Issue is addition to any shell. To be of any use, shells of referenced items
    //?       also need to be RefAddition.
    fn add(
        &self,
        locality: T::Locality,
        builder: impl FnOnce(ItemContext<T>) -> T,
    ) -> Result<Key<T>, CollectionError>;
}

pub trait RefRemove<T: Item> {
    fn mark_for_removal(&self, key: Key<T>) -> Result<(), CollectionError>;

    fn remove_marked(&mut self);
}

//? IDEA: AliveKey<T> that can't be cloned or copied returned from adding collection methods can
//?       be a secure source of Refs. AliveKeys should be managed by Collection to allow only one
//?       AliveKey per item. For removal, AliveKey must be used.
//?       - A problematic case is when AliveKey is issued and item want's to remove itself because
//?         of some update.
