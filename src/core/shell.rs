use super::{AnyKey, Key};
use std::any::TypeId;

/// A shell of an entity.
/// Shells are connected to each other.
pub trait AnyShell<'a>: 'a {
    fn item_ty(&self) -> TypeId;

    fn from_any(&self) -> Box<dyn Iterator<Item = AnyKey> + 'a>;

    /// Number of items referencing this item.
    fn from_count(&self) -> usize {
        self.from_any().count()
    }
}

pub trait RefShell<'a>: AnyShell<'a> {
    type T: ?Sized + 'static;
    type Iter<F: ?Sized + 'static>: Iterator<Item = Key<F>> + 'a;
    type AnyIter: Iterator<Item = AnyKey> + 'a;

    fn from<F: ?Sized + 'static>(&self) -> Self::Iter<F>;

    fn iter(&self) -> Self::AnyIter;
}

/// Changes can be delayed until drop.
pub trait MutShell<'a> {
    fn add_from(&mut self, from: AnyKey);

    fn remove_from(&mut self, from: AnyKey) -> bool;
}
