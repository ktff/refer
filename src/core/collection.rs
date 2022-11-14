use super::{Item, Key};
use crate::Allocator;

/// A collection of entities.
///
/// Entity is an item in a shell.
/// Entities are connected to each other through shells.
///
/// Collection can be split into collections of items and shells.
pub trait Collection<T: Item>: Allocator<T> {
    /// Err if collection is out of keys or if some of the references don't exist.
    fn add(&mut self, item: T) -> Result<Key<T>, T>
    where
        Self: Allocator<T, R = ()>,
    {
        self.add_with(item, ())
    }

    /// Err if collection is out of keys or if some of the references don't exist.
    fn add_with(&mut self, item: T, r: Self::R) -> Result<Key<T>, T>;

    /// Err if some of the references don't exist.
    fn set(&mut self, key: Key<T>, set: T) -> Result<T, T>;

    /// Some if it exists.
    fn take(&mut self, key: Key<T>) -> Option<T>;
}
