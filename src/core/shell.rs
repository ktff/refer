use super::{AnyKey, AnyRef, AnySlotContext, Item};
pub use crate::util::ord_iter::AscendingIterator;
use std::any::Any;

/// A shell of an item. In which references are recorded.
pub trait Shell: Sized + Any + Sync + Send {
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

    fn add_any(&mut self, from: AnyKey, context: AnySlotContext);

    fn add_many_any(&mut self, from: AnyKey, count: usize, context: AnySlotContext);

    fn replace_any(&mut self, from: AnyKey, to: AnyKey, context: AnySlotContext);

    fn remove_any(&mut self, from: AnyKey);

    fn clear_any(&mut self, context: AnySlotContext);
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

    fn add_any(&mut self, from: AnyKey, context: AnySlotContext) {
        let context = context.downcast::<T::T>();
        self.add(from, context.allocator());
    }

    fn add_many_any(&mut self, from: AnyKey, count: usize, context: AnySlotContext) {
        let context = context.downcast::<T::T>();
        for _ in 0..count {
            self.add(from, context.allocator());
        }
    }

    fn replace_any(&mut self, from: AnyKey, to: AnyKey, context: AnySlotContext) {
        let context = context.downcast::<T::T>();
        self.replace(from, to, context.allocator());
    }

    fn remove_any(&mut self, from: AnyKey) {
        self.remove(from);
    }

    fn clear_any(&mut self, context: AnySlotContext) {
        let context = context.downcast::<T::T>();
        self.clear(context.allocator());
    }
}
