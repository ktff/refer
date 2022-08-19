use super::{AnyKey, AnyRef, Key};
use std::any::TypeId;

/// A shell of an entity.
/// Shells are connected to each other.
pub trait AnyShell<'a>: 'a {
    fn type_id(&self) -> TypeId;

    fn key_any(&self) -> AnyKey;

    /// Bidirectional references.
    fn from_any(&self) -> Vec<AnyKey>;

    /// Number of items referencing this item.
    fn from_count(&self) -> usize;
}

pub trait Shell<'a> {
    type T: ?Sized + 'static;

    fn key(&self) -> Key<Self::T>;
}

pub trait RefShell<'a>: Shell<'a> + AnyShell<'a> {
    type Iter<T: ?Sized + 'static>: Iterator<Item = Key<T>> + 'a;

    /// Bidirectional references.
    fn from<T: ?Sized + 'static>(&self) -> Self::Iter<T>;
}

/// Changes can be delayed until drop.
pub trait MutShell<'a>: Shell<'a> {
    /// Expects original reference with key of item referencing this one.
    fn add_from(&mut self, from: AnyRef);

    /// Expects original reference with key of item referencing this one.
    fn remove_from(&mut self, from: AnyRef);
}
