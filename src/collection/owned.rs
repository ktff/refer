use crate::core::*;
use std::{any::TypeId, collections::HashSet};

pub type ItemIter<'a, C: Container<T> + 'static, T: AnyItem> =
    impl Iterator<Item = (Key<T>, &'a T)>;
pub type ItemMutIter<'a, C: Container<T> + 'static, T: AnyItem> =
    impl Iterator<Item = (Key<T>, &'a mut T)>;

pub type ShellsIter<'a, C: Container<T> + 'static, T: AnyItem> =
    impl Iterator<Item = (Key<T>, &'a <C as Container<T>>::Shell)>;
pub type ShellsMutIter<'a, C: Container<T> + 'static, T: AnyItem> =
    impl Iterator<Item = (Key<T>, &'a mut <C as Container<T>>::Shell)>;

pub type Iter<'a, C: Container<T> + 'static, T: AnyItem> =
    impl Iterator<Item = (Key<T>, &'a T, &'a C::Shell)>;
pub type MutIter<'a, C: Container<T> + 'static, T: AnyItem> =
    impl Iterator<Item = (Key<T>, &'a mut T, &'a C::Shell)>;

/// Impl collection for provided container by having full ownership of it.
pub struct Owned<C: 'static>(C);

impl<C: 'static> Owned<C> {
    pub fn new(c: C) -> Self {
        Self(c)
    }
}

impl<C: Allocator<T> + Container<T> + AnyContainer + 'static, T: Item> Collection<T> for Owned<C> {
    fn add(&mut self, item: T) -> Result<Key<T>, T> {
        // Allocate slot
        let key = if let Some(key) = self.0.reserve(&item) {
            key
        } else {
            return Err(item);
        };

        // Update connections
        if !super::util::add_references(&mut self.shells_mut(), key.key().into_key(), &item) {
            // Failed

            // Deallocate slot
            self.0.cancel(key);

            return Err(item);
        }

        // Add item & shell
        Ok(self.0.fulfill(key, item).into_key())
    }

    fn set(&mut self, key: Key<T>, set: T) -> Result<T, T> {
        let (mut items, mut shells) = self.split_mut();
        let old = if let Some(item) = items.get_mut(key) {
            item
        } else {
            // No item
            return Err(set);
        };

        // Update connections
        if !super::util::update_diff(&mut shells, key, old, &set) {
            // Failed
            return Err(set);
        }

        // Replace item
        Ok(std::mem::replace(old, set))
    }

    fn take(&mut self, key: Key<T>) -> Option<T> {
        let mut remove = Vec::new();

        // Update connections
        super::util::remove_references(self, key.into(), &mut remove)?;
        // Deallocate
        let item = self.0.unfill(key.into()).expect("Should exist");

        // Recursive remove
        while let Some(rf) = remove.pop() {
            // Update connections
            if super::util::remove_references(self, rf, &mut remove).is_some() {
                // Deallocate
                let _ = self.0.any_unfill(rf.into());
            }
        }

        Some(item)
    }
}

impl<C: Allocator<T> + 'static, T: 'static> Allocator<T> for Owned<C> {
    fn reserve(&mut self, item: &T) -> Option<ReservedKey<T>> {
        self.0.reserve(item)
    }

    fn cancel(&mut self, key: ReservedKey<T>) {
        self.0.cancel(key)
    }

    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T>
    where
        T: Sized,
    {
        self.0.fulfill(key, item)
    }

    fn unfill(&mut self, key: SubKey<T>) -> Option<T>
    where
        T: Sized,
    {
        self.0.unfill(key)
    }
}

impl<C: Container<T> + 'static, T: AnyItem> Access<T> for Owned<C> {
    type Shell = <C as Container<T>>::Shell;

    type ItemsMut<'a> = AccessItemsMut<'a,C>
    where
        Self: 'a;

    type ShellsMut<'a> = AccessShellsMut<'a,C>
    where
        Self: 'a;

    type Items<'a> = AccessItems<'a,C>
    where
        Self: 'a;

    type Shells<'a> = AccessShells<'a,C>
    where
        Self: 'a;

    type Iter<'a> = Iter<'a,C,T>
    where
        Self: 'a;

    type MutIter<'a> = MutIter<'a,C,T>
    where
        Self: 'a;

    fn get(&self, key: Key<T>) -> Option<(&T, &Self::Shell)> {
        self.0.get_slot(key.into()).map(|(item, slot)| {
            // This is safe because Self has total access to C and
            // we borrow &self so we can't mutate any slot hence &slot is safe.
            unsafe { (&*item.get(), &*slot.get()) }
        })
    }

    /// Some if item exists.
    fn get_mut(&mut self, key: Key<T>) -> Option<(&mut T, &Self::Shell)> {
        self.0.get_slot(key.into()).map(|(item, slot)| {
            // This is safe because Self has total access to C and
            // we borrow &mut self so there is no other &mut slot hence &mut slot is safe.
            unsafe { (&mut *item.get(), &*slot.get()) }
        })
    }

    fn iter(&self) -> Self::Iter<'_> {
        // This is safe because Self has total access to C and
        // we borrow &self so we can't mutate any slot hence all &slot is safe.
        unsafe {
            self.0.iter_slot().into_iter().flat_map(|iter| {
                iter.map(|(key, item, slot)| (key.into_key(), &*item.get(), &*slot.get()))
            })
        }
    }

    fn iter_mut(&mut self) -> Self::MutIter<'_> {
        // This is safe because Self has total access to C and
        // we borrow &mut self so we can't alias any slot twice by calling
        // other *_mut. Additionally we have guarantee from
        // iter_slot that no slot is returned twice.
        unsafe {
            self.0.iter_slot().into_iter().flat_map(|iter| {
                iter.map(|(key, item, slot)| (key.into_key(), &mut *item.get(), &*slot.get()))
            })
        }
    }

    fn split(&self) -> (Self::Items<'_>, Self::Shells<'_>) {
        // This is safe because Self has total access to C and
        // we borrow &self so we can guarantee that no one will mut access
        // items nor shells in C collection during this lifetime.
        unsafe { (AccessItems::new(&self.0), AccessShells::new(&self.0)) }
    }

    fn split_mut(&mut self) -> (Self::ItemsMut<'_>, Self::ShellsMut<'_>) {
        // This is safe because Self has total access to C and
        // we borrow &mut self so we can guarantee that no one will access
        // items nor shells in C collection during this lifetime.
        unsafe { (AccessItemsMut::new(&self.0), AccessShellsMut::new(&self.0)) }
    }
}

impl<C: AnyContainer + 'static> AnyAccess for Owned<C> {
    fn first(&self, key: TypeId) -> Option<AnyKey> {
        self.0.first(key).map(|key| key.into_key())
    }

    fn next(&self, key: AnyKey) -> Option<AnyKey> {
        self.0.next(key.into()).map(|key| key.into_key())
    }

    fn types(&self) -> HashSet<TypeId> {
        self.0.types()
    }

    fn split_item_any(&mut self, key: AnyKey) -> Option<(&mut dyn AnyItem, &mut dyn AnyShells)> {
        let item = self.0.any_get_slot(key.into()).map(|(item, _)| {
            // This is safe because Self has total access to slots and
            // we borrow &mut self so we can't mutate any item hence &mut AnyItem is safe.
            unsafe { &mut *item.get() }
        })?;
        // This is safe since we are only referencing an item at this point so it's safe to
        // give mut access to shells.
        let shells = AccessShellsAny::new(&mut self.0);

        Some((item, shells))
    }

    fn split_shell_any(&mut self, key: AnyKey) -> Option<(&mut dyn AnyItems, &mut dyn AnyShell)> {
        let shell = self.0.any_get_slot(key.into()).map(|(_, shell)| {
            // This is safe because Self has total access to slots and
            // we borrow &mut self so we can't mutate any shell hence &mut AnyShell is safe.
            unsafe { &mut *shell.get() }
        })?;
        // This is safe since we are only referencing a shell at this point so it's safe to
        // give mut access to items.
        let items = AccessItemsAny::new(&mut self.0);

        Some((items, shell))
    }

    fn split_any(&mut self) -> (Box<dyn AnyItems + '_>, Box<dyn AnyShells + '_>) {
        // This is safe because Self has total access to C and
        // we borrow &mut self so we can guarantee that no one will access
        // items nor shells in C collection during this lifetime.
        unsafe {
            (
                Box::new(AccessItemsMut::new(&self.0)),
                Box::new(AccessShellsMut::new(&self.0)),
            )
        }
    }
}

/// This guarantees only Items will be fetched and mutably referenced.
pub struct AccessItemsMut<'c, C: 'static>(AccessItems<'c, C>);

impl<'c, C: 'static> AccessItemsMut<'c, C> {
    /// UNSAFE: Caller must guarantee that no one else will access
    /// items in C collection during 'c lifetime.
    pub unsafe fn new(collection: &'c C) -> Self {
        Self(AccessItems::new(collection))
    }
}

impl<'c, C: Container<T> + 'static, T: AnyItem> ItemsMut<T> for AccessItemsMut<'c, C> {
    type MutIter<'a> = ItemMutIter<'a,C, T> where Self:'a;

    fn get_mut(&mut self, key: Key<T>) -> Option<&mut T> {
        (self.0).0.get_slot(key.into()).map(|(item, _)| {
            // This is safe because Self has total access to items and
            // we borrow &mut self so we can't alias T twice.
            unsafe { &mut *item.get() }
        })
    }

    fn iter_mut(&mut self) -> Self::MutIter<'_> {
        // This is safe because Self has total access to items and
        // we borrow &mut self so we can't alias any T twice by calling
        // iter_mut or get_mut twice. Additionally we have guarantee from
        // iter_slot that no item is returned twice.
        unsafe {
            (self.0)
                .0
                .iter_slot()
                .into_iter()
                .flat_map(|iter| iter.map(|(key, item, _)| (key.into_key(), &mut *item.get())))
        }
    }
}

impl<'c, C: Container<T> + 'static, T: AnyItem> Items<T> for AccessItemsMut<'c, C> {
    type Iter<'a> = ItemIter<'a,C, T> where Self:'a;

    fn get(&self, key: Key<T>) -> Option<&T> {
        self.0.get(key)
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.0.iter()
    }
}

impl<'c, C: AnyContainer + 'static> AnyItems for AccessItemsMut<'c, C> {
    fn get_any(&self, key: AnyKey) -> Option<&dyn AnyItem> {
        (self.0).0.any_get_slot(key.into()).map(|(item, _)| {
            // This is safe because Self has total access to items and
            // we borrow &self so we can't mutate any item hence &AnyItem is safe.
            unsafe { &*item.get() }
        })
    }

    fn get_mut_any(&mut self, key: AnyKey) -> Option<&mut dyn AnyItem> {
        (self.0).0.any_get_slot(key.into()).map(|(item, _)| {
            // This is safe because Self has total access to items and
            // we borrow &mut self so there is no other &mut AnyItem hence &mut AnyItem is safe.
            unsafe { &mut *item.get() }
        })
    }
}

/// This guarantees only Items will be fetched and referenced.
pub struct AccessItems<'c, C: 'static>(&'c C);

impl<'c, C: 'static> AccessItems<'c, C> {
    /// UNSAFE: Caller must guarantee that no one will mut access
    /// items in C collection during 'c lifetime.
    pub unsafe fn new(collection: &'c C) -> Self {
        Self(collection)
    }
}

impl<'c, C: Container<T> + 'static, T: AnyItem> Items<T> for AccessItems<'c, C> {
    type Iter<'a> = ItemIter<'a,C, T> where Self:'a;

    fn get(&self, key: Key<T>) -> Option<&T> {
        self.0.get_slot(key.into()).map(|(item, _)| {
            // This is safe because Self has total access to items and
            // we borrow &self so we can't mutate the T hence &T is safe.
            unsafe { &*item.get() }
        })
    }

    fn iter(&self) -> Self::Iter<'_> {
        // This is safe because Self has total access to items and
        // we borrow &self so we can't mutate any T hence all &T is safe.
        unsafe {
            self.0
                .iter_slot()
                .into_iter()
                .flat_map(|iter| iter.map(|(key, item, _)| (key.into_key(), &*item.get())))
        }
    }
}

/// This guarantees only Items will be fetched and mutably referenced.
#[repr(transparent)]
pub struct AccessItemsAny<C: 'static>(C);

impl<C: 'static> AccessItemsAny<C> {
    pub fn new(collection: &mut C) -> &mut Self {
        // This is safe since Self is a transparent wrapper around C
        unsafe { &mut *(collection as *mut C as *mut Self) }
    }
}

impl<C: AnyContainer + 'static> AnyItems for AccessItemsAny<C> {
    fn get_any(&self, key: AnyKey) -> Option<&dyn AnyItem> {
        self.0.any_get_slot(key.into()).map(|(item, _)| {
            // This is safe because Self has total access to items and
            // we borrow &self so we can't mutate any item hence &AnyItem is safe.
            unsafe { &*item.get() }
        })
    }

    fn get_mut_any(&mut self, key: AnyKey) -> Option<&mut dyn AnyItem> {
        self.0.any_get_slot(key.into()).map(|(item, _)| {
            // This is safe because Self has total access to items and
            // we borrow &mut self so there is no other &mut AnyItem hence &mut AnyItem is safe.
            unsafe { &mut *item.get() }
        })
    }
}

/// This guarantees only Shells will be fetched and mutably referenced.
pub struct AccessShellsMut<'c, C: 'static>(AccessShells<'c, C>);

impl<'c, C: 'static> AccessShellsMut<'c, C> {
    /// UNSAFE: Caller must guarantee that no one else will access
    /// shells in C collection during 'c lifetime.
    pub unsafe fn new(collection: &'c C) -> Self {
        Self(AccessShells::new(collection))
    }
}

impl<'c, C: Container<T> + 'static, T: AnyItem> ShellsMut<T> for AccessShellsMut<'c, C> {
    type MutIter<'a> = ShellsMutIter<'a,C, T> where Self:'a;

    fn get_mut(&mut self, key: Key<T>) -> Option<&mut Self::Shell> {
        (self.0).0.get_slot(key.into()).map(|(_, shell)| {
            // This is safe because Self has total access to shells and
            // we borrow &mut self so we can't alias Shell twice.
            unsafe { &mut *shell.get() }
        })
    }

    fn iter_mut(&mut self) -> Self::MutIter<'_> {
        // This is safe because Self has total access to shells and
        // we borrow &mut self so we can't alias any Shell twice by calling
        // iter_mut or get_mut twice. Additionally we have guarantee from
        // iter_slot that no shell is returned twice.
        unsafe {
            (self.0)
                .0
                .iter_slot()
                .into_iter()
                .flat_map(|iter| iter.map(|(key, _, shell)| (key.into_key(), &mut *shell.get())))
        }
    }
}

impl<'c, C: Container<T> + 'static, T: AnyItem> Shells<T> for AccessShellsMut<'c, C> {
    type Shell = <C as Container<T>>::Shell;

    type Iter<'a> = ShellsIter<'a,C, T> where Self:'a;

    fn get(&self, key: Key<T>) -> Option<&Self::Shell> {
        self.0.get(key)
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.0.iter()
    }
}

impl<'c, C: AnyContainer + 'static> AnyShells for AccessShellsMut<'c, C> {
    fn get_any(&self, key: AnyKey) -> Option<&dyn AnyShell> {
        (self.0).0.any_get_slot(key.into()).map(|(_, shell)| {
            // This is safe because Self has total access to shells and
            // we borrow &self so we can't mutate the Shell hence &Shell is safe.
            unsafe { &*shell.get() }
        })
    }

    fn get_mut_any(&mut self, key: AnyKey) -> Option<&mut dyn AnyShell> {
        (self.0).0.any_get_slot(key.into()).map(|(_, shell)| {
            // This is safe because Self has total access to shells and
            // we borrow &mut self so there is no other &mut Shell hence &mut Shell is safe.
            unsafe { &mut *shell.get() }
        })
    }
}

/// This guarantees only Shells will be fetched and referenced.
pub struct AccessShells<'c, C: 'static>(&'c C);

impl<'c, C: 'static> AccessShells<'c, C> {
    /// UNSAFE: Caller must guarantee that no one will mut access
    /// shells in C collection during 'c lifetime.
    pub unsafe fn new(collection: &'c C) -> Self {
        Self(collection)
    }
}

impl<'c, C: Container<T> + 'static, T: AnyItem> Shells<T> for AccessShells<'c, C> {
    type Shell = <C as Container<T>>::Shell;

    type Iter<'a> = ShellsIter<'a,C, T> where Self:'a;

    fn get(&self, key: Key<T>) -> Option<&Self::Shell> {
        self.0.get_slot(key.into()).map(|(_, shell)| {
            // This is safe because Self has total access to shells and
            // we borrow &self so we can't mutate the Shell hence &Shell is safe.
            unsafe { &*shell.get() }
        })
    }

    fn iter(&self) -> Self::Iter<'_> {
        // This is safe because Self has total access to shells and
        // we borrow &self so we can't mutate any Shell hence all &Shell is safe.
        unsafe {
            self.0
                .iter_slot()
                .into_iter()
                .flat_map(|iter| iter.map(|(key, _, shell)| (key.into_key(), &*shell.get())))
        }
    }
}

/// This guarantees only Shells will be fetched and mutably referenced.
#[repr(transparent)]
pub struct AccessShellsAny<C: 'static>(C);

impl<C: 'static> AccessShellsAny<C> {
    pub fn new(collection: &mut C) -> &mut Self {
        // This is safe since Self is a transparent wrapper around C
        unsafe { &mut *(collection as *mut C as *mut Self) }
    }
}

impl<C: AnyContainer + 'static> AnyShells for AccessShellsAny<C> {
    fn get_any(&self, key: AnyKey) -> Option<&dyn AnyShell> {
        self.0.any_get_slot(key.into()).map(|(_, shell)| {
            // This is safe because Self has total access to shells and
            // we borrow &self so we can't mutate the Shell hence &Shell is safe.
            unsafe { &*shell.get() }
        })
    }

    fn get_mut_any(&mut self, key: AnyKey) -> Option<&mut dyn AnyShell> {
        self.0.any_get_slot(key.into()).map(|(_, shell)| {
            // This is safe because Self has total access to shells and
            // we borrow &mut self so there is no other &mut Shell hence &mut Shell is safe.
            unsafe { &mut *shell.get() }
        })
    }
}
