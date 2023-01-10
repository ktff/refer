use super::{
    AnyKey, AnyPath, AnySlotContext, AnyUnsafeSlot, ExclusivePermit, Item, Key, KeyPath, Path,
    Shell, SlotContext, UnsafeSlot,
};
use std::{
    any::{Any, TypeId},
    collections::HashSet,
    path::Prefix,
};

// TODO: Container returning inner Containers, & simplifications that can be done with it.

/// A family of containers.
pub trait ContainerFamily: Send + Sync + 'static {
    type Container<T: Item>: Container<T>;

    fn new_container<T: Item>(key: Prefix) -> Self::Container<T>;
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

    fn get_slot(&self, sub_key: Key<T>) -> Option<UnsafeSlot<T, Self::Shell>>;

    fn get_context(&self, key: T::LocalityKey) -> Option<SlotContext<T>>;

    /// Iterates in ascending order of key for keys under/with given prefix.
    /// No slot is returned twice in returned iterator.
    fn iter_slot(&self, top_key: KeyPath<T>) -> Option<Self::SlotIter<'_>>;

    /// None if there is no more place in locality.
    fn fill_slot(&mut self, key: T::LocalityKey, item: T) -> Result<Key<T>, T>;

    /// Fills locality
    fn fill_context(&mut self, key: T::LocalityKey);

    /// Removes from container.
    fn unfill_slot(&mut self, sub_key: Key<T>) -> Option<(T, Self::Shell, SlotContext<T>)>;
}

pub trait AnyContainer: Any + Sync + Send {
    /// Path of container shared for all items in the container.
    fn container_path(&self) -> Path;

    fn get_slot_any(&self, sub_key: AnyKey) -> Option<AnyUnsafeSlot>;

    /// None if such locality doesn't exist.
    fn get_context_any(&self, key: AnyPath) -> Option<AnySlotContext>;

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
    fn fill_slot_any(&mut self, key: AnyPath, item: Box<dyn Any>) -> Result<AnyKey, String>;

    /// Fills some locality under given prefix, or enclosing locality.
    fn fill_context_any(&mut self, top_key: AnyPath) -> AnyPath;

    fn unfill_slot_any(&mut self, sub_key: AnyKey);

    fn access(&mut self) -> ExclusivePermit<'_, Self> {
        ExclusivePermit::new(self)
    }
}

// impl<'a, C: AnyContainer> Drop for Owned<'a, C> {
//     fn drop(&mut self) {
//         for ty in self.0.types() {
//             if let Some(mut key) = self.0.first(ty) {
//                 loop {
//                     // Drop slot
//                     match self.access_mut().slot(key).get_dyn() {
//                         Ok(mut slot) => {
//                             // Drop local
//                             slot.shell_clear();
//                             slot.displace();

//                             // Unfill
//                             self.0.unfill_slot_any(key);
//                         }
//                         Err(error) => warn!("Invalid key: {}", error),
//                     }

//                     // Next
//                     if let Some(next) = self.0.next(key) {
//                         key = next;
//                     } else {
//                         break;
//                     }
//                 }
//             }
//         }
//     }
// }
