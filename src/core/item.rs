use super::{AnyKey, AnyRef, Index};
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
