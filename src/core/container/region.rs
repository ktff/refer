use super::*;
use std::ops::RangeBounds;

pub trait LocalityContainer<T: Item> {
    /// Index of locality
    fn locality(&self, key: T::LocalityKey) -> Option<usize>;

    /// Index of locality
    fn fill_locality(&mut self, key: T::LocalityKey) -> usize;
}

pub trait RegionContainer {
    type Sub: AnyContainer;

    type Iter<'a>: Iterator<Item = (usize, &'a Self::Sub)> + Send
    where
        Self: 'a;

    type IterMut<'a>: Iterator<Item = (usize, &'a mut Self::Sub)> + Send
    where
        Self: 'a;

    /// Implementations should have #[inline(always)]
    fn region(&self) -> RegionPath;

    /// Returns first index with a sub container.
    fn first(&self) -> Option<usize>;

    /// Returns following index after given in ascending order with a sub container.
    fn next(&self, after: usize) -> Option<usize>;

    /// Returns last index with a sub container.
    fn last(&self) -> Option<usize>;

    /// Implementations should have #[inline(always)]
    /// Bijection between index and container MUST be enforced.
    fn get(&self, index: usize) -> Option<&Self::Sub>;

    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Sub>;

    /// Iterates in ascending order for indices in range.
    /// Iterator MUST NOT return the same container more than once.
    fn iter(&self, range: impl RangeBounds<usize>) -> Option<Self::Iter<'_>>;

    /// Iterates in ascending order for indices in range.
    fn iter_mut(&mut self, range: impl RangeBounds<usize>) -> Option<Self::IterMut<'_>>;

    /// None if index is out of region.
    fn fill(&mut self, index: usize) -> Option<&mut Self::Sub>;
}

// *************************** Blankets ***************************

// impl<T: Item, C: RegionContainer<T> + AnyContainer> Container<T> for C {
//     type Shell = <C::Sub as Container<T>>::Shell;

//     type SlotIter<'a> = impl Iterator<Item = (Key<T>, UnsafeSlot<'a, T, Self::Shell>)> + Send
//     where
//         Self: 'a;

//     #[inline(always)]
//     fn get_slot(&self, key: Key<T>) -> Option<UnsafeSlot<T, Self::Shell>> {
//         let index = self.region().index_of(key);
//         self.get(index).and_then(|c| c.get_slot(key))
//     }

//     fn get_context(&self, key: T::LocalityKey) -> Option<SlotContext<T>> {
//         self.locality(key).and_then(|c| c.get_context(key))
//     }

//     /// Iterates in ascending order of key for keys under/with given prefix.
//     /// No slot is returned twice in returned iterator.
//     fn iter_slot(&self, path: KeyPath<T>) -> Option<Self::SlotIter<'_>> {
//         let range = self.region().range_of(path.path())?;
//         self.iter(range).map(|iter| {
//             iter.flat_map(move |(_, container)| container.iter_slot(path).into_iter().flatten())
//         })
//     }

//     fn fill_slot(&mut self, key: T::LocalityKey, item: T) -> Result<Key<T>, T> {
//         self.fill(key).fill_slot(key, item)
//     }

//     fn fill_context(&mut self, key: T::LocalityKey) {
//         self.fill(key).fill_context(key)
//     }

//     fn unfill_slot(&mut self, key: Key<T>) -> Option<(T, Self::Shell, SlotContext<T>)> {
//         let index = self.region().index_of(key);
//         self.get_mut(index).and_then(|c| c.unfill_slot(key))
//     }
// }
