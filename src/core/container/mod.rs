#[macro_use]
mod leaf;
#[macro_use]
mod region;
#[macro_use]
mod ty;

pub use leaf::*;
pub use region::*;
pub use ty::*;

use super::{
    permit::RemovePermit, AddPermit, AnyPermit, AnySlotLocality, AnyUnsafeSlot, Item, ItemTraits,
    Key, KeyPath, LocalityKey, LocalityPath, Owned, PartialEdge, Path, Ptr, Ref, RegionPath,
    SlotLocality, UnsafeSlot,
};
use crate::core::permit;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// A family of containers.
pub trait ContainerFamily<T: Item>: Send + Sync + 'static {
    type Container: Container<T>;

    fn new_container(&mut self, region: Path) -> Self::Container;
}

/// It's responsibility is to:
/// - allocate/contain/deallocate items, not to manage access to them or to call their methods.
/// - expose internal containers as & and &mut.
/// - clear it self up during drop.
///
/// TODO: Eliminate this sentence.
/// May panic if argument keys don't correspond to this container.
///
/// UNSAFE: Implementations MUST follow get_slot & iter_slot SAFETY contracts.
pub unsafe trait Container<T: Item>: AnyContainer {
    type SlotIter<'a>: Iterator<Item = (Key<Ref<'a>, T>, UnsafeSlot<'a, T>)> + Send
    where
        Self: 'a;

    /// Implementations should have #[inline(always)]
    /// SAFETY: Bijection between keys and slots MUST be enforced.
    fn get_slot(&self, key: Key<Ptr, T>) -> Option<UnsafeSlot<T>>;

    fn get_locality(&self, key: &impl LocalityPath) -> Option<SlotLocality<T>>;

    /// Iterates in ascending order of key for keys under/with given prefix.
    /// SAFETY: Iterator MUST NOT return the same slot more than once.
    fn iter_slot(&self, path: KeyPath<T>) -> Option<Self::SlotIter<'_>>;

    /// None if there is no more place in locality.
    fn fill_slot(&mut self, key: &impl LocalityPath, item: T) -> Result<Key<Ref, T>, T>;

    /// Fills locality
    /// None if there is no locality for T under given key.
    fn fill_locality(&mut self, key: &impl LocalityPath) -> Option<LocalityKey>;

    /// Removes from container.
    fn unfill_slot(&mut self, key: Key<Ptr, T>) -> Option<(T, SlotLocality<T>)>;
}

/// UNSAFE: Implementations MUST follow get_slot_any & next_key SAFETY contracts.
pub unsafe trait AnyContainer: Any + Sync + Send {
    /// Path of container shared for all items in the container.
    fn container_path(&self) -> Path;

    /// Returns first key for given type
    fn first_key(&self, key: TypeId) -> Option<Key<Ref>>;

    /// Returns following key after given in ascending order
    /// for the type at the key.
    ///
    /// SAFETY: MUST have bijection over input_key and output_key and input_key != output_key.
    fn next_key(&self, ty: TypeId, key: Key) -> Option<Key<Ref>>;

    /// Returns last key for given type
    fn last_key(&self, key: TypeId) -> Option<Key<Ref>>;

    /// All types in the container.
    fn types(&self) -> HashMap<TypeId, ItemTraits>;

    /// Implementations should have #[inline(always)]
    /// SAFETY: Bijection between keys and slots MUST be enforced.
    fn get_slot_any(&self, key: Key) -> Option<AnyUnsafeSlot>;

    /// None if there is no locality for given type under given key.
    fn get_locality_any(&self, key: &dyn LocalityPath, ty: TypeId) -> Option<AnySlotLocality>;

    /// Err if:
    /// - no more place in locality
    /// - type is unknown
    /// - locality is undefined
    /// - type mismatch
    fn fill_slot_any(
        &mut self,
        key: &dyn LocalityPath,
        item: Box<dyn Any>,
    ) -> Result<Key<Ref>, String>;

    /// Fills some locality under given key, or enclosing locality, for given type.
    /// None if there is no locality for given type under given key.
    fn fill_locality_any(&mut self, key: &dyn LocalityPath, ty: TypeId) -> Option<LocalityKey>;

    /// Panics if item is edgeless referenced.
    /// Caller should properly dispose of the edges.
    fn localized_drop(&mut self, key: Key) -> Option<Vec<PartialEdge<Key<Owned>>>>;

    fn access_add(&mut self) -> AddPermit<'_, Self>
    where
        Self: Sized,
    {
        AddPermit::new(self)
    }

    fn access_remove(&mut self) -> RemovePermit<'_, Self>
    where
        Self: Sized,
    {
        RemovePermit::new(self)
    }

    fn access_mut(&mut self) -> AnyPermit<permit::Mut, Self>
    where
        Self: Sized,
    {
        AnyPermit::new(self)
    }
}
