pub mod leaf;
pub mod region;
pub mod ty;

use super::{
    AnyKey, AnyPath, AnySlotContext, AnyUnsafeSlot, ExclusivePermit, Item, Key, KeyPath, Path,
    RegionPath, Shell, SlotContext, UnsafeSlot,
};
use std::{
    any::{Any, TypeId},
    collections::HashSet,
};

// TODO: Container returning inner Containers, & simplifications that can be done with it. Can this be traits?

// TODO: Unify naming of methods in various Container traits.

/// TODO: Macro impl for *Container

/// A family of containers.
pub trait ContainerFamily: Send + Sync + 'static {
    type Container<T: Item>: Container<T>;

    fn new_container<T: Item>(region: RegionPath) -> Self::Container<T>;
}

/// It's responsibility is to:
/// - allocate/contain/deallocate items and shells, not to manage access to them or to call their methods.
/// - expose internal containers as & and &mut.
/// - clear it self up during drop.
///
/// May panic if argument keys don't correspond to this container.
pub trait Container<T: Item>: AnyContainer {
    /// Shell of item.
    type Shell: Shell<T = T>;

    type SlotIter<'a>: Iterator<Item = (Key<T>, UnsafeSlot<'a, T, Self::Shell>)> + Send
    where
        Self: 'a;

    /// Implementations should have #[inline(always)]
    /// Bijection between keys and slots MUST be enforced.
    fn get_slot(&self, key: Key<T>) -> Option<UnsafeSlot<T, Self::Shell>>;

    fn get_context(&self, key: T::LocalityKey) -> Option<SlotContext<T>>;

    /// Iterates in ascending order of key for keys under/with given prefix.
    /// Iterator MUST NOT return the same slot more than once.
    fn iter_slot(&self, path: KeyPath<T>) -> Option<Self::SlotIter<'_>>;

    /// None if there is no more place in locality.
    fn fill_slot(&mut self, key: T::LocalityKey, item: T) -> Result<Key<T>, T>;

    /// Fills locality
    fn fill_context(&mut self, key: T::LocalityKey);

    /// Removes from container.
    fn unfill_slot(&mut self, key: Key<T>) -> Option<(T, Self::Shell, SlotContext<T>)>;
}

pub trait AnyContainer: Any + Sync + Send {
    /// Path of container shared for all items in the container.
    fn container_path(&self) -> Path;

    /// Implementations should have #[inline(always)]
    /// Bijection between keys and slots MUST be enforced.
    fn get_slot_any(&self, key: AnyKey) -> Option<AnyUnsafeSlot>;

    /// None if such locality doesn't exist.
    fn get_context_any(&self, path: AnyPath) -> Option<AnySlotContext>;

    /// Returns first key for given type
    fn first_key(&self, key: TypeId) -> Option<AnyKey>;

    /// Returns following key after given in ascending order
    /// for the same type.
    fn next_key(&self, key: AnyKey) -> Option<AnyKey>;

    /// Returns last key for given type
    fn last_key(&self, key: TypeId) -> Option<AnyKey>;

    /// All types in the container.
    fn types(&self) -> HashSet<TypeId>;

    /// Err if:
    /// - no more place in locality
    /// - type is unknown
    /// - locality is undefined
    /// - type mismatch
    fn fill_slot_any(&mut self, path: AnyPath, item: Box<dyn Any>) -> Result<AnyKey, String>;

    /// Fills some locality under given prefix, or enclosing locality.
    fn fill_context_any(&mut self, path: AnyPath) -> AnyPath;

    fn unfill_slot_any(&mut self, key: AnyKey);

    fn access(&mut self) -> ExclusivePermit<'_, Self> {
        ExclusivePermit::new(self)
    }
}
