use super::{AnyKey, AnyRef, AnySlotLocality, Item};
pub use crate::util::ord_iter::AscendingIterator;
use std::any::Any;

/// A shell of an item. In which references are recorded.
pub trait Shell: Sized + Any + Sync + Send {
    // TODO: T se cini samo kao smetnja, mozda je bolje da se ukloni.
    type T: Item;

    type Iter<'a>: Iterator<Item = AnyRef> + 'a
    where
        Self: 'a;

    fn new_in(alloc: &<Self::T as Item>::Alloc) -> Self;

    fn iter(&self) -> AscendingIterator<Self::Iter<'_>>;

    /// Additive if called for same `from` multiple times.
    fn add(&mut self, from: impl Into<AnyKey>, alloc: &<Self::T as Item>::Alloc);

    fn replace(&mut self, from: impl Into<AnyKey>, to: AnyKey, alloc: &<Self::T as Item>::Alloc);

    /// Subtracts if called for same `from` multiple times.
    fn remove(&mut self, from: impl Into<AnyKey>);

    /// Should clear itself and shrink.
    /// Must not have any remaining local allocation.
    fn clear(&mut self, alloc: &<Self::T as Item>::Alloc);
}

/// Methods correspond 1 to 1 to Item methods.
pub trait AnyShell: Any {
    fn iter_any(&self) -> Option<AscendingIterator<Box<dyn Iterator<Item = AnyRef> + '_>>>;

    fn add_any(&mut self, from: AnyKey, locality: AnySlotLocality);

    fn add_many_any(&mut self, from: AnyKey, count: usize, locality: AnySlotLocality);

    fn replace_any(&mut self, from: AnyKey, to: AnyKey, locality: AnySlotLocality);

    fn remove_any(&mut self, from: AnyKey);

    fn clear_any(&mut self, locality: AnySlotLocality);
}

impl<T: Shell> AnyShell for T {
    fn iter_any(&self) -> Option<AscendingIterator<Box<dyn Iterator<Item = AnyRef> + '_>>> {
        let iter = self.iter();
        if let (0, Some(0)) = iter.size_hint() {
            None
        } else {
            Some(iter.map_internal(|iter| Box::new(iter) as Box<dyn Iterator<Item = AnyRef> + '_>))
        }
    }

    fn add_any(&mut self, from: AnyKey, locality: AnySlotLocality) {
        let locality = locality.downcast::<T::T>();
        self.add(from, locality.allocator());
    }

    fn add_many_any(&mut self, from: AnyKey, count: usize, locality: AnySlotLocality) {
        let locality = locality.downcast::<T::T>();
        for _ in 0..count {
            self.add(from, locality.allocator());
        }
    }

    fn replace_any(&mut self, from: AnyKey, to: AnyKey, locality: AnySlotLocality) {
        let locality = locality.downcast::<T::T>();
        self.replace(from, to, locality.allocator());
    }

    fn remove_any(&mut self, from: AnyKey) {
        self.remove(from);
    }

    fn clear_any(&mut self, locality: AnySlotLocality) {
        let locality = locality.downcast::<T::T>();
        self.clear(locality.allocator());
    }
}
