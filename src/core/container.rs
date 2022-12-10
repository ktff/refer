use super::{
    AnyItem, AnyKey, AnyKeyPrefix, AnySlotContext, AnyUnsafeSlot, Context, Index, Item, Key,
    KeyPrefix, Shell, SlotContext, UnsafeSlot,
};
use std::{
    alloc::Allocator,
    any::{Any, TypeId},
    collections::HashSet,
    fmt::Debug,
    path::Prefix,
};

/// A family of containers.
pub trait ContainerFamily: Send + Sync + 'static {
    type Container<T: Item>: Container<T>;

    fn new_container<T: Item>(key: Prefix) -> Self::Container<T>;
}

/// It's responsibility is to allocate/contain/deallocate items and shells, not to manage access to them or
/// to call their methods.
///
/// May panic if argument keys don't correspond to this container.
pub trait Container<T: Item>: AnyContainer {
    /// Shell of item.
    type Shell: Shell<T = T>;

    type SlotIter<'a>: Iterator<Item = (Key<T>, UnsafeSlot<'a, T, Self::Shell>)> + Send
    where
        Self: 'a;

    fn get_slot(&self, sub_key: Key<T>) -> Option<UnsafeSlot<T, Self::Shell>>;

    fn get_locality(&self, key: T::LocalityKey) -> Option<SlotContext<T>>;

    /// Iterates in ascending order of key for keys under/with given prefix.
    /// No slot is returned twice in returned iterator.
    fn iter_slot(&self, top_key: KeyPrefix) -> Option<Self::SlotIter<'_>>;

    /// None if there is no more place in locality.
    fn fill_slot(&mut self, key: T::LocalityKey, item: T) -> Result<Key<T>, T>;

    /// Fills locality
    fn fill_locality(&mut self, key: T::LocalityKey);

    /// Removes from container.
    fn unfill_slot(&mut self, sub_key: Key<T>) -> Option<(T, Self::Shell, SlotContext<T>)>;
}

pub trait AnyContainer: Any + Sync + Send {
    fn get_slot_any(&self, sub_key: AnyKey) -> Option<AnyUnsafeSlot>;

    /// None if such locality doesn't exist.
    fn get_locality_any(&self, key: AnyKeyPrefix) -> Option<AnySlotContext>;

    /// Returns first key for given type
    fn first(&self, key: TypeId) -> Option<AnyKey>;

    /// Returns following key after given in ascending order
    /// for the same type.
    fn next(&self, sub_key: AnyKey) -> Option<AnyKey>;

    /// Returns last key for given type
    fn last(&self, key: TypeId) -> Option<AnyKey>;

    /// All types in the container.
    fn types(&self) -> HashSet<TypeId>;

    /// Err if:
    /// - no more place in locality
    /// - type is unknown
    /// - locality is undefined
    /// - type mismatch
    fn fill_slot_any(&mut self, key: AnyKeyPrefix, item: Box<dyn Any>) -> Result<AnyKey, String>;

    /// Fills some locality under given prefix, or enclosing locality.
    fn fill_locality_any(&mut self, top_key: AnyKeyPrefix) -> AnyKeyPrefix;

    fn unfill_slot_any(&mut self, sub_key: AnyKey);
}
