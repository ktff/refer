use std::{
    any::{Any, TypeId},
    collections::HashSet,
};

use super::{
    AnyItem, AnyItems, AnyKey, AnyShell, AnyShells, Item, Items, ItemsMut, Key, Shell, Shells,
    ShellsMut,
};

/// A collection of entities.
///
/// Entity is an item in a shell.
/// Entities are connected to each other through shells.
///
/// Collection can be split into collections of items and shells.
pub trait Collection<T: Item>: Access<T> {
    /// Err if collection is out of keys or if some of the references don't exist.
    fn add(&mut self, item: T) -> Result<Key<T>, T>;

    /// Err if some of the references don't exist.
    fn set(&mut self, key: Key<T>, set: T) -> Result<T, T>;

    /// Some if it exists.
    fn take(&mut self, key: Key<T>) -> Option<T>;
}

pub trait Access<T: AnyItem + ?Sized>: AnyAccess {
    type Shell: Shell<T = T>;

    type ItemsMut<'a>: ItemsMut<T> + 'a
    where
        Self: 'a;

    type ShellsMut<'a>: ShellsMut<T, Shell = Self::Shell> + 'a
    where
        Self: 'a;

    type Items<'a>: Items<T> + 'a
    where
        Self: 'a;

    type Shells<'a>: Shells<T, Shell = Self::Shell> + 'a
    where
        Self: 'a;

    type Iter<'a>: Iterator<Item = (Key<T>, &'a T, &'a Self::Shell)>
    where
        Self: 'a;

    type MutIter<'a>: Iterator<Item = (Key<T>, &'a mut T, &'a Self::Shell)>
    where
        Self: 'a;

    /// Some if it exists.
    fn get(&self, key: Key<T>) -> Option<(&T, &Self::Shell)>;

    /// Some if it exists.
    fn get_mut(&mut self, key: Key<T>) -> Option<(&mut T, &Self::Shell)>;

    /// Ascending order.
    fn iter(&self) -> Self::Iter<'_>;

    /// Ascending order.
    fn iter_mut(&mut self) -> Self::MutIter<'_>;

    fn shells(&self) -> Self::Shells<'_> {
        self.split().1
    }

    fn shells_mut(&mut self) -> Self::ShellsMut<'_> {
        self.split_mut().1
    }

    fn items(&self) -> Self::Items<'_> {
        self.split().0
    }

    fn items_mut(&mut self) -> Self::ItemsMut<'_> {
        self.split_mut().0
    }

    /// Splits to views of items and shells
    fn split(&self) -> (Self::Items<'_>, Self::Shells<'_>);

    /// Splits to views of items and shells
    fn split_mut(&mut self) -> (Self::ItemsMut<'_>, Self::ShellsMut<'_>);
}

pub trait AnyAccess: Any {
    /// Returns first key for given type
    fn first(&self, key: TypeId) -> Option<AnyKey>;

    /// Returns following key after given in ascending order
    /// for the same type.
    fn next(&self, key: AnyKey) -> Option<AnyKey>;

    /// All types in the collection.
    fn types(&self) -> HashSet<TypeId>;

    fn split_item_any(&mut self, key: AnyKey) -> Option<(&mut dyn AnyItem, &mut dyn AnyShells)>;

    fn split_shell_any(&mut self, key: AnyKey) -> Option<(&mut dyn AnyItems, &mut dyn AnyShell)>;

    /// Splits to views of items and shells
    fn split_any(&mut self) -> (Box<dyn AnyItems + '_>, Box<dyn AnyShells + '_>);
}
