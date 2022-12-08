use crate::util::AscendingIterator;

use super::{AnyItemContext, AnyKey, AnyRef, Index, Item, Key, Ref};
use std::any::{Any, TypeId};

/// A shell of an item. In which references are recorded.
pub trait Shell: Any + Sync + Send {
    type T: Item;

    type Iter<'a>: Iterator<Item = AnyRef> + 'a
    where
        Self: 'a;

    type IterOf<'a, I: Item>: Iterator<Item = Ref<I>> + 'a
    where
        Self: 'a;

    fn iter(&self) -> AscendingIterator<Self::Iter<'_>>;

    fn iter_of<I: Item>(&self) -> AscendingIterator<Self::IterOf<'_, I>>;

    /// Number of I items referencing this item.
    fn len_of<I: Item>(&self) -> usize {
        self.iter_of::<I>().count()
    }

    /// Additive if called for same `from` multiple times.
    fn add(&mut self, from: AnyKey, alloc: &<Self::T as Item>::Alloc);

    fn replace(&mut self, from: AnyKey, to: Index, alloc: &<Self::T as Item>::Alloc);

    /// Subtracts if called for same `from` multiple times.
    fn remove(&mut self, from: AnyKey);

    /// Should clear itself and shrink.
    /// Must not have any remaining local allocation.
    fn clear(&mut self, alloc: &<Self::T as Item>::Alloc);
}

/// Methods correspond 1 to 1 to Item methods.
pub trait AnyShell: Any + Sync + Send {
    fn iter_any(&self) -> Option<AscendingIterator<Box<dyn Iterator<Item = AnyRef> + '_>>>;

    fn add_any(&mut self, from: AnyKey, context: AnyItemContext);

    fn add_many_any(&mut self, from: AnyKey, count: usize, context: AnyItemContext);

    fn replace_any(&mut self, from: AnyKey, to: Index, context: AnyItemContext);

    fn remove_any(&mut self, from: AnyKey);

    fn clear_any(&mut self, context: AnyItemContext);
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

    fn add_any(&mut self, from: AnyKey, context: AnyItemContext) {
        let context = context.downcast::<T::T>();
        self.add(from, context.allocator());
    }

    fn add_many_any(&mut self, from: AnyKey, count: usize, context: AnyItemContext) {
        let context = context.downcast::<T::T>();
        for _ in 0..count {
            self.add(from, context.allocator());
        }
    }

    fn replace_any(&mut self, from: AnyKey, to: Index, context: AnyItemContext) {
        let context = context.downcast::<T::T>();
        self.replace(from, to, context.allocator());
    }

    fn remove_any(&mut self, from: AnyKey) {
        self.remove(from);
    }

    fn clear_any(&mut self, context: AnyItemContext) {
        let context = context.downcast::<T::T>();
        self.clear(context.allocator());
    }
}
