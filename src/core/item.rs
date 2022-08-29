use super::{AnyKey, AnyRef, Index};
use std::any::Any;

pub trait Item: AnyItem {
    type I<'a>: Iterator<Item = AnyRef>;

    /// All internal references. Must be stable.
    fn references(&self, this: Index) -> Self::I<'_>;
}

/// An item of entity.
pub trait AnyItem: Any + 'static {
    /// All internal references.
    /// Some can be empty.
    fn references_any(&self, this: Index) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>>;

    /// True if removed, false if not and this item should be removed as a result.
    /// May panic if not present.
    fn remove_reference(&mut self, this: Index, key: AnyKey) -> bool;
}
