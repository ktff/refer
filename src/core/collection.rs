use super::{
    AnyCollection, AnyItemCollection, AnyKeyCollection, AnyShellCollection, Error, Item, Key,
    MutEntity, MutShell, RefEntity, RefShell,
};

/// Polly collections can split &mut self to multiple &mut views each with set of types that don't overlap.
/// Polly collection can implement this trait for each type.
pub trait Collection<T: Item + ?Sized>: KeyCollection<T> + AnyCollection {
    type Shells: ShellCollection<T>;
    type Items: ItemCollection<T>;

    type MEntity<'a>: MutEntity<'a, T = T>
    where
        Self: 'a;

    type REntity<'a>: RefEntity<'a, T = T>
    where
        Self: 'a;

    type ReIter<'a>: Iterator<Item = Self::REntity<'a>>
    where
        Self: 'a;

    type MiIter<'a>: Iterator<Item = Self::MEntity<'a>>
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

    fn entity(&self, key: Key<T>) -> Result<Self::REntity<'_>, Error>;

    fn entity_mut(&mut self, key: Key<T>) -> Result<Self::MEntity<'_>, Error>;

    fn iter_entity(&self) -> Self::ReIter<'_>;

    fn iter_entity_mut(&mut self) -> Self::MiIter<'_>;

    fn shells(&self) -> &Self::Shells;

    fn shells_mut(&mut self) -> &mut Self::Shells {
        self.split().1
    }

    fn items(&self) -> &Self::Items;

    fn items_mut(&mut self) -> &mut Self::Items {
        self.split().0
    }

    /// Splits view to items and shells
    fn split(&mut self) -> (&mut Self::Items, &mut Self::Shells);
}

/// Polly collections can split &mut self to multiple &mut views each with set of types that don't overlap.
/// Polly collection can implement this trait for each type.
pub trait ItemCollection<T: ?Sized + 'static>: KeyCollection<T> + AnyItemCollection {
    type RiIter<'a>: Iterator<Item = (Key<T>, &'a T)>
    where
        Self: 'a;

    type MiIter<'a>: Iterator<Item = (Key<T>, &'a mut T)>
    where
        Self: 'a;

    fn item(&self, key: Key<T>) -> Result<&T, Error>;

    fn item_mut(&mut self, key: Key<T>) -> Result<&mut T, Error>;

    /// Consistent ascending order.
    fn iter_item(&self) -> Self::RiIter<'_>;

    /// Consistent ascending order.
    fn iter_item_mut(&mut self) -> Self::MiIter<'_>;
}

/// Polly collections can split &mut self to multiple &mut views each with set of types that don't overlap.
pub trait ShellCollection<T: ?Sized + 'static>: KeyCollection<T> + AnyShellCollection {
    type RShell<'a>: RefShell<'a, T = T>
    where
        Self: 'a;

    type MShell<'a>: MutShell<'a, T = T>
    where
        Self: 'a;

    type MsIter<'a>: Iterator<Item = Self::MShell<'a>>
    where
        Self: 'a;

    type RsIter<'a>: Iterator<Item = Self::RShell<'a>>
    where
        Self: 'a;

    fn shell(&self, key: Key<T>) -> Result<Self::RShell<'_>, Error>;

    fn shell_mut(&mut self, key: Key<T>) -> Result<Self::MShell<'_>, Error>;

    /// Consistent ascending order.
    fn iter_shell(&self) -> Self::RsIter<'_>;

    /// Consistent ascending order.
    fn iter_shell_mut(&mut self) -> Self::MsIter<'_>;
}

pub trait KeyCollection<T: ?Sized + 'static>: AnyKeyCollection {
    fn first(&self) -> Option<Key<T>>;

    /// Returns following key after given in ascending order.
    fn next(&self, key: Key<T>) -> Option<Key<T>>;
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
