use super::AnyRef;
use std::any::Any;

pub trait Item: AnyItem {
    type I<'a>: Iterator<Item = AnyRef>;

    /// All internal references
    fn references(&self) -> Self::I<'_>;
}

/// An item of entity.
pub trait AnyItem: Any {
    /// All internal references
    fn references_any<'a>(&'a self) -> Box<dyn Iterator<Item = AnyRef> + 'a>;
}
