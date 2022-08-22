use super::{Item, Key, MutEntity, MutShell, RefEntity, RefShell};

// NOTE: Generic naming is intentionally here so to trigger naming conflicts to discourage
//       implementations from implementing all *Collection traits on the same type.

/// A collection of entities.
///
/// Entity is an item in a shell.
/// Entities are connected to each other through shells.
///
/// Collection can be split into collections of items and shells.
pub trait Collection: KeyCollection {
    type Items: ItemCollection;

    type Shells: ShellCollection;

    type Ref<'a, I: ?Sized + 'static>: RefEntity<'a, I>
    where
        Self: 'a;

    type Mut<'a, I: ?Sized + 'static>: MutEntity<'a, I>
    where
        Self: 'a;

    type Iter<'a, I: ?Sized + 'static>: Iterator<Item = Self::Ref<'a, I>>
    where
        Self: 'a;

    type MutIter<'a, I: ?Sized + 'static>: Iterator<Item = Self::Mut<'a, I>>
    where
        Self: 'a;

    /// Err if collection is out of keys.
    fn add<I: Item>(&mut self, item: I) -> Result<Key<I>, I>;

    /// None if collection is out of keys.
    fn add_copy<I: Item + Copy + ?Sized>(&mut self, item: &I) -> Option<Key<I>>;

    /// True Item existed so it was removed.
    fn remove<I: Item + ?Sized>(&mut self, key: Key<I>) -> bool;

    /// Some if item exists.
    fn take<I: Item>(&mut self, key: Key<I>) -> Option<I>;

    /// Some if item exists.
    fn get<I: ?Sized + 'static>(&self, key: Key<I>) -> Option<Self::Ref<'_, I>>;

    /// Some if item exists.
    fn get_mut<I: ?Sized + 'static>(&mut self, key: Key<I>) -> Option<Self::Mut<'_, I>>;

    /// Consistent ascending order.
    fn iter<I: ?Sized + 'static>(&self) -> Self::Iter<'_, I>;

    /// Consistent ascending order.
    fn iter_mut<I: ?Sized + 'static>(&mut self) -> Self::MutIter<'_, I>;

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
pub trait ItemCollection: KeyCollection {
    type Iter<'a, I: ?Sized + 'static>: Iterator<Item = (Key<I>, &'a I)>
    where
        Self: 'a;

    type MutIter<'a, I: ?Sized + 'static>: Iterator<Item = (Key<I>, &'a mut I)>
    where
        Self: 'a;

    /// Some if item exists.
    fn get<I: ?Sized + 'static>(&self, key: Key<I>) -> Option<&I>;

    /// Some if item exists.
    fn get_mut<I: ?Sized + 'static>(&mut self, key: Key<I>) -> Option<&mut I>;

    /// Consistent ascending order.
    fn iter<I: ?Sized + 'static>(&self) -> Self::Iter<'_, I>;

    /// Consistent ascending order.
    fn iter_mut<I: ?Sized + 'static>(&mut self) -> Self::MutIter<'_, I>;
}

/// Polly ShellCollection can't split this.
pub trait ShellCollection: KeyCollection {
    type MutColl<'a>: MutShellCollection<'a>
    where
        Self: 'a;

    type Ref<'a, I: ?Sized + 'static>: RefShell<'a, I>
    where
        Self: 'a;

    type Mut<'a, I: ?Sized + 'static>: MutShell<'a, I>
    where
        Self: 'a;

    type Iter<'a, I: ?Sized + 'static>: Iterator<Item = Self::Ref<'a, I>>
    where
        Self: 'a;

    type MutIter<'a, I: ?Sized + 'static>: Iterator<Item = Self::Mut<'a, I>>
    where
        Self: 'a;

    /// Some if item exists.
    fn get<I: ?Sized + 'static>(&self, key: Key<I>) -> Option<Self::Ref<'_, I>>;

    /// Some if item exists.
    fn get_mut<I: ?Sized + 'static>(&mut self, key: Key<I>) -> Option<Self::Mut<'_, I>>;

    /// Consistent ascending order.
    fn iter<I: ?Sized + 'static>(&self) -> Self::Iter<'_, I>;

    /// Consistent ascending order.
    fn iter_mut<I: ?Sized + 'static>(&mut self) -> Self::MutIter<'_, I>;

    fn mut_coll(&mut self) -> Self::MutColl<'_>;
}

/// Enables holding on to multiple MutShells at the same time.
/// To enable that it usually won't enable viewing of the data.
pub trait MutShellCollection<'a>: 'a {
    type Mut<I: ?Sized + 'static>: MutShell<'a, I>;

    /// Some if item exists.
    fn get_mut<I: ?Sized + 'static>(&self, key: Key<I>) -> Option<Self::Mut<I>>;
}

pub trait KeyCollection {
    fn first<I: ?Sized + 'static>(&self) -> Option<Key<I>>;

    /// Returns following key after given in ascending order.
    fn next<I: ?Sized + 'static>(&self, key: Key<I>) -> Option<Key<I>>;
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
