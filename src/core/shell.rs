use crate::{AnyItem, MutShellSlot, RefShellSlot};

use super::{AnyKey, Key, MutAnyShellSlot, RefAnyShellSlot};
use std::any::{Any, TypeId};

/// A shell of an item. In which references are recorded.
pub trait Shell: AnyShell {
    // This is mostly used for type checking/constraining.
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

pub trait AnyShell: Any + Sync + Send {
    fn item_ty(&self) -> TypeId;

    fn from_any(&self) -> Box<dyn Iterator<Item = AnyKey> + '_>;

    /// Number of items referencing this item.
    fn from_count(&self) -> usize {
        self.from_any().count()
    }

    /// Additive if called for same `from` multiple times.
    fn add_from(&mut self, from: AnyKey, alloc: &impl std::alloc::Allocator)
    where
        Self: Sized;

    // TODO: Some better name
    /// Additive if called for same `from` multiple times.
    fn add_from_any(&mut self, from: AnyKey, alloc: &dyn std::alloc::Allocator);

    /// Subtracts if called for same `from` multiple times.
    fn remove_from(&mut self, from: AnyKey);
}

pub trait ShellsMut<T: AnyItem>: Shells<T> + AnyShells {
    type MutIter<'a>: Iterator<Item = MutShellSlot<'a, T, Self::Shell, Self::Alloc>> + Send
    where
        Self: 'a;

    fn get_mut(&mut self, key: Key<T>) -> Option<MutShellSlot<T, Self::Shell, Self::Alloc>>;

    /// Ascending order.
    fn iter_mut(&mut self) -> Self::MutIter<'_>;
}

pub trait Shells<T: AnyItem> {
    type Alloc: std::alloc::Allocator + 'static;

    type Shell: Shell<T = T>;

    type Iter<'a>: Iterator<Item = RefShellSlot<'a, T, Self::Shell, Self::Alloc>> + Send
    where
        Self: 'a;

    fn get(&self, key: Key<T>) -> Option<RefShellSlot<T, Self::Shell, Self::Alloc>>;

    /// Ascending order.
    fn iter(&self) -> Self::Iter<'_>;
}

pub trait AnyShells {
    fn get_any(&self, key: AnyKey) -> Option<RefAnyShellSlot>;

    fn get_mut_any(&mut self, key: AnyKey) -> Option<MutAnyShellSlot>;
}
