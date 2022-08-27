use super::{AnyKey, Key};
use std::any::TypeId;

/// A shell of an entity.
/// Shells are connected to each other.
pub trait AnyShell {
    fn item_ty(&self) -> TypeId;

    fn from_any(&self) -> Box<dyn Iterator<Item = AnyKey> + '_>;

    /// Number of items referencing this item.
    fn from_count(&self) -> usize {
        self.from_any().count()
    }

    fn add_from(&mut self, from: AnyKey);

    fn remove_from(&mut self, from: AnyKey) -> bool;
}

pub trait Shell: AnyShell {
    // TODO: Is this really needed?
    type T: ?Sized + 'static;
    type Iter<'a, F: ?Sized + 'static>: Iterator<Item = Key<F>> + 'a
    where
        Self: 'a;
    type AnyIter<'a>: Iterator<Item = AnyKey> + 'a
    where
        Self: 'a;

    fn from<F: ?Sized + 'static>(&self) -> Self::Iter<'_, F>;

    fn iter(&self) -> Self::AnyIter<'_>;
}
