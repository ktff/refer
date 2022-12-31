use super::{
    permit, AnyContainer, AnyPermit, CollectionError, Container, Item, Key, MutSlot, RefSlot,
    SlotContext,
};

pub type Result<T> = std::result::Result<T, CollectionError>;

/// A collection of slots.
///
/// Slot is an item in a shell.
/// Slots are connected to each other through shells.
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
pub trait Collection<T: Item>: Access<Self::Model> {
    type Model: Container<T>;

    fn add(&mut self, locality: T::LocalityKey, item: T) -> Result<Key<T>>;

    /// Removes previous item and sets new one in his place while updating references accordingly.
    fn replace_item(&mut self, key: Key<T>, item: T) -> Result<T>;

    /// T has it's local data dropped.
    /// May have side effects that invalidate some Keys.
    fn remove(&mut self, key: Key<T>) -> Result<T>;

    /// Displaces item to locality and displaces shell.
    /// May have side effects that invalidate some Keys.
    fn displace(&mut self, from: Key<T>, to: T::LocalityKey) -> Result<Key<T>>;

    /// Displaces item and removes shell.
    /// May have side effects that invalidate some Keys.
    fn displace_item(&mut self, from: Key<T>, to: T::LocalityKey) -> Result<Key<T>>;

    /// Displaces shell from `from` to `to` so that all references are now pointing to `to`.
    /// May have side effects that invalidate some Keys.
    fn displace_shell(&mut self, from: Key<T>, to: Key<T>) -> Result<()>;

    // Duplicates item to locality and duplicates shell.
    fn duplicate(&mut self, key: Key<T>, to: T::LocalityKey) -> Result<Key<T>>;

    /// Duplicates item to locality.
    fn duplicate_item(&mut self, key: Key<T>, to: T::LocalityKey) -> Result<Key<T>>;

    /// Duplicates shell from `from` to `to` so that all references to `from` now also point to `to`.
    fn duplicate_shell(&mut self, from: Key<T>, to: Key<T>) -> Result<()>;

    fn get(&self, key: Key<T>) -> Result<RefSlot<T, <Self::Model as Container<T>>::Shell>> {
        self.access().slot(key).get()
    }

    fn get_mut(&mut self, key: Key<T>) -> Result<MutSlot<T, <Self::Model as Container<T>>::Shell>> {
        self.access_mut().slot(key).get()
    }

    fn context(&mut self, locality: T::LocalityKey) -> SlotContext<T>;

    fn existing_context(&self, locality: T::LocalityKey) -> Option<SlotContext<T>>;
}

pub trait Access<M: AnyContainer> {
    fn access(&self) -> AnyPermit<permit::Ref, permit::Slot, M>;

    fn access_mut(&mut self) -> AnyPermit<permit::Mut, permit::Slot, M>;
}
