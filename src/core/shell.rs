use super::{AnyKey, Key};
use std::any::{Any, TypeId};

// TODO: Somehow enable for Shell to know self Key so that it can use DeltaKey,
// TODO: or just expose DeltaKey directly.

pub trait Shell: AnyShell {
    /// This is mostly used for type checking/constraining.
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

/// A shell of an entity.
/// Shells are connected to each other.
pub trait AnyShell: Any {
    fn item_ty(&self) -> TypeId;

    fn from_any(&self) -> Box<dyn Iterator<Item = AnyKey> + '_>;

    /// Number of items referencing this item.
    fn from_count(&self) -> usize {
        self.from_any().count()
    }

    fn add_from(&mut self, from: AnyKey);

    fn remove_from(&mut self, from: AnyKey) -> bool;
}

pub trait ShellsMut<T: ?Sized + 'static>: Shells<T> + AnyShells {
    type MutIter<'a>: Iterator<Item = (Key<T>, &'a mut Self::Shell)>
    where
        Self: 'a;

    /// Some if item exists.
    fn get_mut(&mut self, key: Key<T>) -> Option<&mut Self::Shell>;

    /// Consistent ascending order.
    fn iter_mut(&mut self) -> Self::MutIter<'_>;
}

pub trait Shells<T: ?Sized + 'static> {
    type Shell: Shell<T = T>;

    type Iter<'a>: Iterator<Item = (Key<T>, &'a Self::Shell)>
    where
        Self: 'a;

    /// Some if item exists.
    fn get(&self, key: Key<T>) -> Option<&Self::Shell>;

    /// Consistent ascending order.
    fn iter(&self) -> Self::Iter<'_>;
}

pub trait AnyShells {
    // TODO: Maybe _any suffix is too much?
    fn get_shell_any(&self, key: AnyKey) -> Option<&dyn AnyShell>;

    fn get_shell_mut_any(&mut self, key: AnyKey) -> Option<&mut dyn AnyShell>;
}
