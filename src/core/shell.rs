use crate::util::AscendingIterator;

use super::{AnyKey, AnyRef, Index, Item, Key, Ref};
use std::any::{Any, TypeId};

/// A shell of an item. In which references are recorded.
pub trait Shell: AnyShell {
    // This is mostly used for type checking/constraining.
    type T: Item;

    type Iter<'a, F: Item>: Iterator<Item = Ref<F>> + 'a
    where
        Self: 'a;

    type AnyIter<'a>: Iterator<Item = AnyRef> + 'a
    where
        Self: 'a;

    fn from<F: Item>(&self) -> AscendingIterator<Self::Iter<'_, F>>;

    fn iter(&self) -> AscendingIterator<Self::AnyIter<'_>>;
}

pub trait AnyShell: Any + Sync + Send {
    fn item_ty(&self) -> TypeId;

    /// Ascending order of keys.
    fn from_any(
        &self,
        ty: TypeId,
    ) -> Option<AscendingIterator<Box<dyn Iterator<Item = AnyRef> + '_>>>;

    /// Ascending order of keys.
    fn iter_any(&self) -> Option<AscendingIterator<Box<dyn Iterator<Item = AnyRef> + '_>>>;

    /// Number of typed items referencing this item.
    fn from_count(&self, ty: TypeId) -> usize {
        self.from_any(ty)
            .map(|iter| iter.count())
            .unwrap_or_default()
    }

    /// Number of items referencing this item.
    fn iter_count(&self) -> usize {
        self.iter_any().map(|iter| iter.count()).unwrap_or_default()
    }

    /// Additive if called for same `from` multiple times.
    /// T::Alloc
    fn add_from(&mut self, from: AnyKey, alloc: &dyn std::alloc::Allocator);

    /// T::Alloc
    fn add_from_count(&mut self, from: AnyKey, count: usize, alloc: &dyn std::alloc::Allocator) {
        for _ in 0..count {
            self.add_from(from, alloc);
        }
    }

    /// T::Alloc
    fn replace(&mut self, from: AnyKey, to: Index, alloc: &dyn std::alloc::Allocator);

    /// T::Alloc
    fn append(&mut self, other: &dyn AnyShell, alloc: &dyn std::alloc::Allocator) {
        if let Some(iter) = other.iter_any() {
            for key in iter {
                self.add_from(key.key(), alloc);
            }
        }
    }

    /// Subtracts if called for same `from` multiple times.
    fn remove_from(&mut self, from: AnyKey);

    fn clear(&mut self);

    /// Should clear itself and shrink.
    /// Must not have any remaining local allocation.
    /// T::Alloc
    fn dealloc(&mut self, alloc: &dyn std::alloc::Allocator);
}
