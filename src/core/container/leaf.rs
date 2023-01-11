use super::*;
use crate::core::Context;
use std::ops::RangeBounds;

pub trait LeafContainer<T: Item> {
    /// Shell of item.
    type Shell: Shell<T = T>;

    type Iter<'a>: Iterator<Item = (Key<T>, UnsafeSlot<'a, T, Self::Shell>)> + Send
    where
        Self: 'a;

    /// Implementations should have #[inline(always)]
    fn context(&self) -> &Context<T>;

    /// Returns first index with a slot.
    fn first(&self) -> Option<usize>;

    /// Returns following index after given in ascending order with a slot.
    fn next(&self, after: usize) -> Option<usize>;

    /// Returns last index with a slot.
    fn last(&self) -> Option<usize>;

    /// Implementations should have #[inline(always)]
    /// Bijection between index and slot MUST be enforced.
    fn get(&self, index: usize) -> Option<UnsafeSlot<T, Self::Shell>>;

    /// Iterates in ascending order for indices in range.
    /// Iterator MUST NOT return the same slot more than once.
    fn iter(&self, range: impl RangeBounds<usize>) -> Option<Self::Iter<'_>>;

    /// None if there is no more place in container.
    fn fill(&mut self, item: T) -> Result<Key<T>, T>;

    /// Removes from container.
    fn unfill(&mut self, index: usize) -> Option<(T, Self::Shell)>;
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

// impl<T: Item, C: LeafContainer<T> + AnyContainer> Container<T> for C {
//     type Shell = <C as LeafContainer<T>>::Shell;

//     type SlotIter<'a> = <C as LeafContainer<T>>::Iter<'a>;

//     #[inline(always)]
//     fn get_slot(&self, key: Key<T>) -> Option<UnsafeSlot<T, Self::Shell>> {
//         let index = self.context().leaf_path().index_of(key);
//         self.get(index)
//     }

//     fn get_context(&self, _: T::LocalityKey) -> Option<SlotContext<T>> {
//         Some(self.context().slot_context())
//     }

//     fn iter_slot(&self, path: KeyPath<T>) -> Option<Self::SlotIter<'_>> {
//         let range = self.context().leaf_path().range_of(path.path())?;
//         self.iter(range)
//     }

//     fn fill_slot(&mut self, _: T::LocalityKey, item: T) -> Result<Key<T>, T> {
//         self.fill(item)
//     }

//     fn fill_context(&mut self, _: T::LocalityKey) {}

//     fn unfill_slot(&mut self, key: Key<T>) -> Option<(T, Self::Shell, SlotContext<T>)> {
//         let index = self.context().leaf_path().index_of(key);
//         self.unfill(index)
//             .map(move |(item, shell)| (item, shell, self.context().slot_context()))
//     }
// }
