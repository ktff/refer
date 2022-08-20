use super::{
    AnyEntity, AnyKey, AnyShell, Error, Item, Key, MutEntity, MutShell, RefEntity, RefShell,
};
use std::any::Any;

// NOTE: Generic naming is intentionally here so to trigger naming conflicts to discourage
//       implementations from implementing all *Collection traits on the same type.

/// A collection of entities.
///
/// Entity is an item in a shell.
/// Entities are connected to each other through shells.
///
/// Collection can be split into collections of items and shells.
pub trait Collection<T: Item + ?Sized>: KeyCollection<T> + PollyCollection {
    type Ref<'a>: RefEntity<'a, T = T>
    where
        Self: 'a;

    type Mut<'a>: MutEntity<'a, T = T>
    where
        Self: 'a;

    type Iter<'a>: Iterator<Item = Self::Ref<'a>>
    where
        Self: 'a;

    type MutIter<'a>: Iterator<Item = Self::Mut<'a>>
    where
        Self: 'a;

    fn add(&mut self, item: T) -> Result<Key<T>, Error>
    where
        T: Sized;

    fn add_copy(&mut self, item: &T) -> Result<Key<T>, Error>;

    /// Fails if item is referenced by other items.
    fn remove(&mut self, key: Key<T>) -> Result<(), Error>;

    /// Fails if item is referenced by other items.
    fn take(&mut self, key: Key<T>) -> Result<T, Error>
    where
        T: Sized;

    fn get(&self, key: Key<T>) -> Result<Self::Ref<'_>, Error>;

    fn get_mut(&mut self, key: Key<T>) -> Result<Self::Mut<'_>, Error>;

    /// Consistent ascending order.
    fn iter(&self) -> Self::Iter<'_>;

    /// Consistent ascending order.
    fn iter_mut(&mut self) -> Self::MutIter<'_>;
}

/// An entity collection of one or more types.
pub trait PollyCollection: AnyCollection {
    /// Should implement ItemCollection<T> for all Collection<T>
    type Items: AnyItemCollection;

    /// Should implement ShellCollection<T> for all Collection<T>
    type Shells: AnyShellCollection;

    fn shells(&self) -> &Self::Shells;

    fn shells_mut(&mut self) -> &mut Self::Shells {
        self.split().1
    }

    fn items(&self) -> &Self::Items;

    fn items_mut(&mut self) -> &mut Self::Items {
        self.split().0
    }

    /// Splits to views of items and shells
    fn split(&mut self) -> (&mut Self::Items, &mut Self::Shells);
}

/// Polly ItemCollection can split &mut self to multiple &mut views each with set of types that don't overlap.
pub trait ItemCollection<T: ?Sized + 'static>: KeyCollection<T> + AnyItemCollection {
    type Iter<'a>: Iterator<Item = (Key<T>, &'a T)>
    where
        Self: 'a;

    type MutIter<'a>: Iterator<Item = (Key<T>, &'a mut T)>
    where
        Self: 'a;

    fn get(&self, key: Key<T>) -> Result<&T, Error>;

    fn get_mut(&mut self, key: Key<T>) -> Result<&mut T, Error>;

    /// Consistent ascending order.
    fn iter(&self) -> Self::Iter<'_>;

    /// Consistent ascending order.
    fn iter_mut(&mut self) -> Self::MutIter<'_>;
}

/// Polly ShellCollection can't split this.
pub trait ShellCollection<T: ?Sized + 'static>: KeyCollection<T> + PollyShellCollection {
    type Ref<'a>: RefShell<'a, T = T>
    where
        Self: 'a;

    type Mut<'a>: MutShell<'a, T = T>
    where
        Self: 'a;

    type Iter<'a>: Iterator<Item = Self::Ref<'a>>
    where
        Self: 'a;

    type MutIter<'a>: Iterator<Item = Self::Mut<'a>>
    where
        Self: 'a;

    fn get(&self, key: Key<T>) -> Result<Self::Ref<'_>, Error>;

    fn get_mut(&mut self, key: Key<T>) -> Result<Self::Mut<'_>, Error>;

    /// Consistent ascending order.
    fn iter(&self) -> Self::Iter<'_>;

    /// Consistent ascending order.
    fn iter_mut(&mut self) -> Self::MutIter<'_>;
}

pub trait PollyShellCollection: AnyShellCollection {
    /// Should implement MutShellCollection<T> for all ShellCollection<T>
    type MutColl<'a>: 'a
    where
        Self: 'a;

    fn mut_coll(&mut self) -> Self::MutColl<'_>;
}

/// Enables holding on to multiple MutShells at the same time.
/// To enable that it usually won't enable viewing of the data.
pub trait MutShellCollection<'a, T: ?Sized + 'static>: AnyKeyCollection + 'a {
    type Mut: MutShell<'a, T = T>;

    fn get_mut(&self, key: Key<T>) -> Result<Self::Mut, Error>;
}

pub trait KeyCollection<T: ?Sized + 'static>: AnyKeyCollection {
    fn first(&self) -> Option<Key<T>>;

    /// Returns following key after given in ascending order.
    fn next(&self, key: Key<T>) -> Option<Key<T>>;
}

pub trait AnyCollection: AnyKeyCollection {
    fn any(&self, key: AnyKey) -> Result<Box<dyn AnyEntity<'_>>, Error>;
}

pub trait AnyItemCollection: AnyKeyCollection {
    fn any(&self, key: AnyKey) -> Result<&dyn Any, Error>;
}

pub trait AnyShellCollection: AnyKeyCollection {
    fn any(&self, key: AnyKey) -> Result<Box<dyn AnyShell<'_>>, Error>;
}

pub trait AnyKeyCollection {
    fn first_any(&self) -> Option<AnyKey>;

    /// Returns following key after given with indices in ascending order.
    /// Order according to type is undefined.
    fn next_any(&self, key: AnyKey) -> Option<AnyKey>;
}

// ********************** Convenience methods **********************

// impl<T: ?Sized + 'static> Key<T> {
//     pub fn entry<C: Collection<T> + ?Sized>(self, coll: &C) -> Result<C::RE<'_, &C>, Error> {
//         Collection::<T>::entry(coll, self)
//     }

//     pub fn entry_mut<C: Collection<T> + ?Sized>(
//         self,
//         coll: &mut C,
//     ) -> Result<C::ME<'_, &mut C>, Error> {
//         Collection::<T>::entry_mut(coll, self)
//     }

//     pub fn next_key<C: Collection<T> + ?Sized>(self, coll: &C) -> Option<Key<T>> {
//         Collection::<T>::next_key(coll, self)
//     }
// }

// pub trait CollectedType<C: Collection<Self> + ?Sized>: 'static {
//     fn first_key(coll: &mut C) -> Option<Key<Self>>;

//     fn add<'a>(coll: &'a mut C) -> C::IE<'a, &'a mut C>;
// }

// impl<T: ?Sized + 'static, C: Collection<T> + ?Sized> CollectedType<C> for T {
//     fn first_key(coll: &mut C) -> Option<Key<T>> {
//         coll.first_key()
//     }

//     fn add<'a>(coll: &'a mut C) -> C::IE<'a, &'a mut C> {
//         Collection::<T>::add(coll)
//     }
// }
