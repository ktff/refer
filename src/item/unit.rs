use crate::core::{AnyItem, AnyKey, AnyRef, Index, Item};

/// An unit type useful for type testing.
pub type Unit = ();

impl Item for Unit {
    type I<'a> = std::iter::Empty<AnyRef>;

    fn references(&self, _: Index) -> Self::I<'_> {
        std::iter::empty()
    }
}

impl AnyItem for Unit {
    fn references_any<'a>(&'a self, _: Index) -> Option<Box<dyn Iterator<Item = AnyRef> + 'a>> {
        None
    }

    fn remove_reference(&mut self, _: Index, _: AnyKey) -> bool {
        true
    }
}
