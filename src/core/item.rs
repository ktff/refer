use super::{AnyKey, AnyRef, Index, Key};
use std::any::Any;

// TODO: To allow references and references_any to accept &Items so to be able to trace it's tree of references?

/// An item of a model.
pub trait Item: AnyItem {
    type I<'a>: Iterator<Item = AnyRef>;

    /// All internal references.
    fn references<I: AnyItems + ?Sized>(&self, this: Index, items: &I) -> Self::I<'_>;
}

pub trait AnyItem: Any {
    /// All internal references.
    fn references_any(
        &self,
        this: Index,
        items: &dyn AnyItems,
    ) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>>;

    /// Item of given key has/is been removed.
    ///
    /// This item should return true if it's ok with it.
    /// If false, this item will also be removed.
    ///
    /// Should be called for its references.
    fn item_removed(&mut self, this: Index, key: AnyKey) -> bool;
}

pub trait ItemsMut<T: ?Sized + 'static>: Items<T> {
    type MutIter<'a>: Iterator<Item = (Key<T>, &'a mut T)>
    where
        Self: 'a;

    /// Some if it exists.
    fn get_mut(&mut self, key: Key<T>) -> Option<&mut T>;

    /// Ascending order.
    fn iter_mut(&mut self) -> Self::MutIter<'_>;
}

pub trait Items<T: ?Sized + 'static> {
    type Iter<'a>: Iterator<Item = (Key<T>, &'a T)>
    where
        Self: 'a;

    /// Some if it exists.
    fn get(&self, key: Key<T>) -> Option<&T>;

    /// Ascending order.
    fn iter(&self) -> Self::Iter<'_>;
}

pub trait AnyItems {
    fn get_any(&self, key: AnyKey) -> Option<&dyn AnyItem>;

    fn get_mut_any(&mut self, key: AnyKey) -> Option<&mut dyn AnyItem>;
}
