use crate::{MutItemSlot, RefItemSlot};

use super::{AnyKey, AnyRef, Index, Key, MutAnyItemSlot, RefAnyItemSlot};
use std::any::Any;

/// An item of a model.
pub trait Item: AnyItem {
    type I<'a>: Iterator<Item = AnyRef>;

    /// All internal references.
    fn references(&self, this: Index) -> Self::I<'_>;
}

pub trait AnyItem: Any + Sync + Send {
    // TODO: Remove this items: &I,
    /// All internal references.
    fn references_any(&self, this: Index) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>>;

    // TODO: To be able to say that some other item also needs to be updated.
    /// Item of given key has/is been removed.
    ///
    /// This item should return true if it's ok with it.
    /// If false, this item will also be removed.
    ///
    /// Should be called for its references.
    fn item_removed(&mut self, this: Index, key: AnyKey) -> bool;

    /// Old and new must be of same type.
    fn item_moved(&mut self, old: AnyKey, new: AnyKey);
}

pub trait ItemsMut<T: AnyItem>: Items<T> {
    type MutIter<'a>: Iterator<Item = MutItemSlot<'a, T, Self::GroupItem, Self::Alloc>> + Send
    where
        Self: 'a;

    /// Some if it exists.
    fn get_mut(&mut self, key: Key<T>) -> Option<MutItemSlot<T, Self::GroupItem, Self::Alloc>>;

    /// Ascending order.
    fn iter_mut(&mut self) -> Self::MutIter<'_>;
}

pub trait Items<T: AnyItem> {
    type Alloc: std::alloc::Allocator + 'static;

    type GroupItem: Any;

    type Iter<'a>: Iterator<Item = RefItemSlot<'a, T, Self::GroupItem, Self::Alloc>> + Send
    where
        Self: 'a;

    /// Some if it exists.
    fn get(&self, key: Key<T>) -> Option<RefItemSlot<T, Self::GroupItem, Self::Alloc>>;

    /// Ascending order.
    fn iter(&self) -> Self::Iter<'_>;
}

pub trait AnyItems {
    fn get_any(&self, key: AnyKey) -> Option<RefAnyItemSlot>;

    fn get_mut_any(&mut self, key: AnyKey) -> Option<MutAnyItemSlot>;
}
