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
    DynItem, Item, ItemLocality, ItemTraits, Key, KeyPath, LocalityKey, LocalityPath, LocalityRef,
    Owned, Path, Ptr, Ref, RegionPath, UnsafeSlot,
};
use log::warn;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/*
NOTES:
- Goal is to completely prevent memory errors, and to discourage logical errors.
- Containers are not to be Items since that creates non trivial recursions on type and logic levels.
*/

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
/// May panic if argument keys don't correspond to this container, in other words if key is not valid for this container.
///
/// UNSAFE: Implementations MUST follow get_slot & iter_slot SAFETY contracts.
pub unsafe trait Container<T: Item>: AnyContainer {
    type SlotIter<'a>: Iterator<Item = UnsafeSlot<'a, T>> + Send
    where
        Self: 'a;

    /// Implementations should have #[inline(always)]
    /// SAFETY: Bijection between keys and slots MUST be enforced.
    fn get_slot(&self, key: Key<Ptr, T>) -> Option<UnsafeSlot<T>>;

    fn get_locality(&self, key: &impl LocalityPath) -> Option<LocalityRef<KeyPath<T>, T>>;

    /// Iterates in ascending order of key for keys under/with given prefix.
    /// SAFETY: Iterator MUST NOT return the same slot more than once.
    fn iter_slot(&self, path: KeyPath<T>) -> Option<Self::SlotIter<'_>>;

    /// None if there is no more place in locality.
    fn fill_slot(&mut self, key: &impl LocalityPath, item: T) -> Result<Key<Ref, T>, T>;

    /// Fills locality
    /// None if there is no locality for T under given key.
    fn fill_locality(&mut self, key: &impl LocalityPath) -> Option<LocalityKey>;

    /// Removes from container.
    fn unfill_slot(&mut self, key: Key<Ptr, T>) -> Option<(T, ItemLocality<T>)>;

    /// True if slot for this key exists.
    /// Valid to ask for any key.
    fn contains_slot(&self, key: Key<Ptr, T>) -> bool;

    fn slot_count(&self) -> usize;
}

/// UNSAFE: Implementations MUST follow any_get_slot & next_key SAFETY contracts.
pub unsafe trait AnyContainer: Any + Sync + Send {
    /// Path of container shared for all items in the container.
    fn container_path(&self) -> Path;

    /// Returns first key for given type
    fn first_key(&self, key: TypeId) -> Option<Key<Ref>>;

    fn first_key_of<T: Any>(&self) -> Option<Key<Ref, T>>
    where
        Self: Sized,
    {
        self.first_key(TypeId::of::<T>()).map(Key::assume)
    }

    /// Returns following key after given in ascending order
    /// for the type at the key.
    ///
    /// SAFETY: MUST have bijection over input_key and output_key and input_key != output_key.
    fn next_key(&self, ty: TypeId, key: Key) -> Option<Key<Ref>>;

    fn next_key_of<P, T: Any>(&self, key: Key<P, T>) -> Option<Key<Ref, T>>
    where
        Self: Sized,
    {
        self.next_key(TypeId::of::<T>(), key.ptr().any())
            .map(Key::assume)
    }

    /// Returns last key for given type
    fn last_key(&self, key: TypeId) -> Option<Key<Ref>>;

    fn last_key_of<T: Any>(&self) -> Option<Key<Ref, T>>
    where
        Self: Sized,
    {
        self.last_key(TypeId::of::<T>()).map(Key::assume)
    }

    /// All types in the container.
    fn types(&self) -> HashMap<TypeId, ItemTraits>;

    /// Implementations should have #[inline(always)]
    /// SAFETY: Bijection between keys and slots MUST be enforced.
    fn any_get_slot(&self, key: Key) -> Option<UnsafeSlot>;

    /// None if there is no locality for given type under given key.
    fn any_get_locality(&self, key: &dyn LocalityPath, ty: TypeId) -> Option<LocalityRef>;

    /// Err if:
    /// - no more place in locality
    /// - type is unknown
    /// - locality is undefined
    /// - type mismatch
    fn any_fill_slot(
        &mut self,
        key: &dyn LocalityPath,
        item: Box<dyn Any>,
    ) -> Result<Key<Ref>, String>;

    /// Fills some locality under given key, or enclosing locality, for given type.
    /// None if there is no locality for given type under given key.
    fn any_fill_locality(&mut self, key: &dyn LocalityPath, ty: TypeId) -> Option<LocalityKey>;

    /// Panics if item is edgeless referenced.
    /// Caller should properly dispose of the edges.
    fn localized_drop(&mut self, key: Key) -> Option<Vec<Key<Owned>>>;
}

/// Abstracts over getting slots from Container and AnyContainer for all DynItem.
pub trait DynContainer<T: DynItem + ?Sized>: AnyContainer {
    /// Implementations should have #[inline(always)]
    /// SAFETY: Bijection between keys and slots MUST be enforced.
    fn unified_get_slot(&self, key: Key<Ptr, T>) -> Option<UnsafeSlot<T>>;
}

impl<C: AnyContainer + ?Sized, T: DynItem + ?Sized> DynContainer<T> for C {
    default fn unified_get_slot(&self, key: Key<Ptr, T>) -> Option<UnsafeSlot<T>> {
        self.any_get_slot(key.any())
            .and_then(|slot| match slot.anycast() {
                None => {
                    warn!(
                        "Item at {:?}:{} is not {:?} which was assumed to be true.",
                        slot.locality().path(),
                        slot.item_type_name(),
                        std::any::type_name::<T>()
                    );
                    None
                }
                slot => slot,
            })
    }
}

impl<C: Container<T> + ?Sized, T: Item> DynContainer<T> for C {
    fn unified_get_slot(&self, key: Key<Ptr, T>) -> Option<UnsafeSlot<T>> {
        self.get_slot(key)
    }
}
