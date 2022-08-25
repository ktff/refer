use super::{AnyKey, AnyRef};
use std::any::Any;

pub trait Item: AnyItem {
    type I<'a>: Iterator<Item = AnyRef>;

    /// All internal references. Must be stable.
    fn references(&self) -> Self::I<'_>;
}

/// An item of entity.
pub trait AnyItem: Any + 'static {
    /// All internal references
    fn references_any<'a>(&'a self) -> Box<dyn Iterator<Item = AnyRef> + 'a>;

    /// True if removed, false if not and this item should be removed as a result.
    /// May panic if not present.
    fn remove_reference(&mut self, key: AnyKey, item: &impl Any) -> bool;
}
