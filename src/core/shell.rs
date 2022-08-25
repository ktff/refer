use super::{AnyKey, Key};
use std::any::TypeId;

/// A shell of an entity.
/// Shells are connected to each other.
pub trait AnyShell<'a>: 'a {
    fn item_ty(&self) -> TypeId;

    fn from_any(&self) -> Vec<AnyKey>;

    /// Number of items referencing this item.
    fn from_count(&self) -> usize {
        self.from_any().len()
    }
}

pub trait RefShell<'a>: AnyShell<'a> {
    type T: ?Sized + 'static;
    type Iter<F: ?Sized + 'static>: Iterator<Item = Key<F>> + 'a;

    fn from<F: ?Sized + 'static>(&self) -> Self::Iter<F>;
}

/// Changes can be delayed until drop.
pub trait MutShell<'a> {
    type T: ?Sized + 'static;

    /// Expects original reference with key of item referencing this one.
    fn add_from(&mut self, from: AnyKey);

    /// Expects original reference with key of item referencing this one.
    fn remove_from(&mut self, from: AnyKey) -> bool;
}
