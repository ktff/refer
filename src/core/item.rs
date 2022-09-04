use super::{AnyKey, AnyRef, Index, Key};
use std::any::Any;

pub trait Item: AnyItem {
    type I<'a>: Iterator<Item = AnyRef>;

    /// All internal references. Must be stable.
    fn references(&self, this: Index) -> Self::I<'_>;
}

/// An item of entity.
pub trait AnyItem: Any {
    /// All internal references.
    /// Some can be empty.
    fn references_any(&self, this: Index) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>>;

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

    /// Some if item exists.
    fn get_mut(&mut self, key: Key<T>) -> Option<&mut T>;

    /// Consistent ascending order.
    fn iter_mut(&mut self) -> Self::MutIter<'_>;
}

pub trait Items<T: ?Sized + 'static> {
    type Iter<'a>: Iterator<Item = (Key<T>, &'a T)>
    where
        Self: 'a;

    /// Some if item exists.
    fn get(&self, key: Key<T>) -> Option<&T>;

    /// Consistent ascending order.
    fn iter(&self) -> Self::Iter<'_>;
}

pub trait AnyItems {
    fn get_any(&self, key: AnyKey) -> Option<&dyn AnyItem>;

    fn get_mut_any(&mut self, key: AnyKey) -> Option<&mut dyn AnyItem>;
}
