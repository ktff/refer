use crate::core::*;
use std::{any::TypeId, collections::HashSet};

pub type ItemIter<'a, C: Container<T> + 'static, T: AnyItem> =
    impl Iterator<Item = RefItemSlot<'a, T, <C as Container<T>>::GroupItem, C::Alloc>>;
pub type ItemMutIter<'a, C: Container<T> + 'static, T: AnyItem> =
    impl Iterator<Item = MutItemSlot<'a, T, <C as Container<T>>::GroupItem, C::Alloc>>;

pub type ShellsIter<'a, C: Container<T> + 'static, T: AnyItem> =
    impl Iterator<Item = RefShellSlot<'a, T, <C as Container<T>>::Shell, C::Alloc>>;
pub type ShellsMutIter<'a, C: Container<T> + 'static, T: AnyItem> =
    impl Iterator<Item = MutShellSlot<'a, T, <C as Container<T>>::Shell, C::Alloc>>;

pub type Iter<'a, C: Container<T> + 'static, T: AnyItem> =
    impl Iterator<Item = RefSlot<'a, T, <C as Container<T>>::GroupItem, C::Shell, C::Alloc>>;
pub type MutIter<'a, C: Container<T> + 'static, T: AnyItem> =
    impl Iterator<Item = MutSlot<'a, T, <C as Container<T>>::GroupItem, C::Shell, C::Alloc>>;

// TODO: Fuzzy test access/unsafe this.

/// Impl collection for provided container by having full ownership of it.
pub struct Owned<C: 'static>(C);

impl<C: 'static> Owned<C> {
    // TODO: Make this unsafe, or somehow enforce that we have full ownership of C.
    pub fn new(c: C) -> Self {
        Self(c)
    }

    pub fn inner(&self) -> &C {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut C {
        &mut self.0
    }
}

/// This is safe since Owned has full ownership of C.
unsafe impl<C: 'static> Sync for Owned<C> {}

impl<C: Allocator<T> + Container<T> + AnyContainer + 'static, T: Item> Collection<T> for Owned<C> {
    fn add_with(&mut self, item: T, r: Self::R) -> Result<Key<T>, T> {
        // Allocate slot
        let (key, _) = if let Some(key) = self.reserve(Some(&item), r) {
            key
        } else {
            return Err(item);
        };

        // Update connections
        let mut shells = self.shells_mut();
        if !super::util::add_references(&mut shells, key.key().into_key(), &item) {
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
        let mut slot = if let Some(item) = items.get_mut(key) {
            item
        } else {
            // No item
            return Err(set);
        };

        // Update connections
        if !super::util::update_diff(&mut shells, key, slot.item(), &set) {
            // Failed
            return Err(set);
        }

        // Replace item
        Ok(std::mem::replace(slot.item_mut(), set))
    }

    fn take(&mut self, key: Key<T>) -> Option<T> {
        let mut remove = Vec::new();

        // Update connections
        let (mut items, mut shells) = self.split_mut();
        super::util::notify_item_removed(&mut items, &mut shells, key.into(), &mut remove)?;
        // Deallocate
        let (item, _) = self.0.unfill(key.into())?;

        // Recursive remove
        while let Some(rf) = remove.pop() {
            // Update connections
            let (mut items, mut shells) = self.split_mut();
            if super::util::notify_item_removed(&mut items, &mut shells, rf, &mut remove).is_some()
            {
                // Deallocate
                self.0.unfill_any(rf.into());
            }
        }

        Some(item)
    }
}

impl<C: Allocator<T> + 'static, T: 'static> Allocator<T> for Owned<C> {
    type Alloc = C::Alloc;

    type R = C::R;

    fn reserve(&mut self, item: Option<&T>, r: Self::R) -> Option<(ReservedKey<T>, &Self::Alloc)> {
        self.0.reserve(item, r)
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

    fn unfill(&mut self, key: SubKey<T>) -> Option<(T, &Self::Alloc)>
    where
        T: Sized,
    {
        self.0.unfill(key)
    }
}

impl<C: Container<T> + 'static, T: AnyItem> Access<T> for Owned<C> {
    type GroupItem = <C as Container<T>>::GroupItem;

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

    fn get(&self, key: Key<T>) -> Option<RefSlot<T, Self::GroupItem, Self::Shell, Self::Alloc>> {
        self.0.get_slot(key.into()).map(|slot| {
            // This is safe because Self has total access to C and
            // we borrow &self so we can't mutate any slot hence &slot is safe.
            unsafe { slot.into_slot(key) }
        })
    }

    fn get_mut(
        &mut self,
        key: Key<T>,
    ) -> Option<MutSlot<T, Self::GroupItem, Self::Shell, Self::Alloc>> {
        self.0.get_slot(key.into()).map(|slot| {
            // This is safe because Self has total access to C and
            // we borrow &mut self so there is no other &mut slot hence &mut slot is safe.
            unsafe { slot.into_slot_mut(key) }
        })
    }

    fn iter(&self) -> Self::Iter<'_> {
        // This is safe because Self has total access to C and
        // we borrow &self so we can't mutate any slot hence all &slot is safe.
        unsafe {
            self.0
                .iter_slot()
                .into_iter()
                .flat_map(|iter| iter.map(|(key, slot)| slot.into_slot(key.into_key())))
        }
    }

    fn iter_mut(&mut self) -> Self::MutIter<'_> {
        // This is safe because Self has total access to C and
        // we borrow &mut self so we can't alias any slot twice by calling
        // other *_mut. Additionally we have guarantee from
        // iter_slot that no slot is returned twice.
        unsafe {
            self.0
                .iter_slot()
                .into_iter()
                .flat_map(|iter| iter.map(|(key, slot)| slot.into_slot_mut(key.into_key())))
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

    fn split_item_any(&mut self, key: AnyKey) -> Option<(MutAnyItemSlot, &mut dyn AnyShells)> {
        // TODO: Can we do better?
        // This is safe since we are just decoupling lifetimes of mut shells from the others.
        // We later in the function dereference it back to the original lifetime.
        let shells_ptr = &mut self.0 as *mut _;
        let item = self.0.any_get_slot(key.into()).map(|slot| {
            // This is safe because Self has total access to slots and
            // we borrow &mut self so we can't mutate any item hence &mut AnyItem is safe.
            unsafe { slot.into_item_mut(key) }
        })?;
        // This is safe since we are only referencing a mut item and group item at this point so it's safe to
        // give mut access to shells.
        let shells = AccessShellsAny::new(unsafe { &mut *shells_ptr });

        Some((item, shells))
    }

    fn split_shell_any(&mut self, key: AnyKey) -> Option<(&mut dyn AnyItems, MutAnyShellSlot)> {
        // TODO: Can we do better?
        // This is safe since we are just decoupling lifetimes of mut items from the others.
        // We later in the function dereference it back to the original lifetime.
        let items_ptr = &mut self.0 as *mut _;
        let shell = self.0.any_get_slot(key.into()).map(|slot| {
            // This is safe because Self has total access to slots and
            // we borrow &mut self so we can't mutate any shell hence &mut AnyShell is safe.
            unsafe { slot.into_shell_mut(key) }
        })?;
        // This is safe since we are only referencing a mut shell at this point so it's safe to
        // give mut access to items.
        let items = AccessItemsAny::new(unsafe { &mut *items_ptr });

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

/// This guarantees that it will fetch and mutably reference only items.
pub struct AccessItemsMut<'c, C: 'static>(AccessItems<'c, C>);

impl<'c, C: 'static> AccessItemsMut<'c, C> {
    /// UNSAFE: Caller must guarantee that no one else will access
    /// items in C collection during 'c lifetime.
    pub unsafe fn new(collection: &'c C) -> Self {
        Self(AccessItems::new(collection))
    }
}

impl<'c, C: Container<T> + 'static, T: AnyItem> ItemsMut<T> for AccessItemsMut<'c, C> {
    type MutIter<'a> = ItemMutIter<'a, C, T> where Self:'a;

    fn get_mut(&mut self, key: Key<T>) -> Option<MutItemSlot<T, Self::GroupItem, Self::Alloc>> {
        (self.0).0.get_slot(key.into()).map(|slot| {
            // This is safe because Self has total access to items and
            // we borrow &mut self so we can't alias T twice.
            unsafe { slot.into_item_mut(key) }
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
                .flat_map(|iter| iter.map(|(key, slot)| slot.into_item_mut(key.into_key())))
        }
    }
}

impl<'c, C: Container<T> + 'static, T: AnyItem> Items<T> for AccessItemsMut<'c, C> {
    type Alloc = C::Alloc;

    type GroupItem = <C as Container<T>>::GroupItem;

    type Iter<'a> = ItemIter<'a,C, T> where Self:'a;

    fn get(&self, key: Key<T>) -> Option<RefItemSlot<T, Self::GroupItem, Self::Alloc>> {
        self.0.get(key)
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.0.iter()
    }
}

impl<'c, C: AnyContainer + 'static> AnyItems for AccessItemsMut<'c, C> {
    fn get_any(&self, key: AnyKey) -> Option<RefAnyItemSlot> {
        (self.0).0.any_get_slot(key.into()).map(|slot| {
            // This is safe because Self has total access to items and
            // we borrow &self so we can't mutate any item hence &AnyItem is safe.
            unsafe { slot.into_item(key) }
        })
    }

    fn get_mut_any(&mut self, key: AnyKey) -> Option<MutAnyItemSlot> {
        (self.0).0.any_get_slot(key.into()).map(|slot| {
            // This is safe because Self has total access to items and
            // we borrow &mut self so there is no other &mut AnyItem hence &mut AnyItem is safe.
            unsafe { slot.into_item_mut(key) }
        })
    }
}

/// This guarantees that it will fetch and reference only items.
pub struct AccessItems<'c, C: 'static>(&'c C);

impl<'c, C: 'static> AccessItems<'c, C> {
    /// UNSAFE: Caller must guarantee that no one will mut access
    /// items in C collection during 'c lifetime.
    pub unsafe fn new(collection: &'c C) -> Self {
        Self(collection)
    }
}

impl<'c, C: Container<T> + 'static, T: AnyItem> Items<T> for AccessItems<'c, C> {
    type Alloc = <C as Allocator<T>>::Alloc;

    type GroupItem = <C as Container<T>>::GroupItem;

    type Iter<'a> = ItemIter<'a,C, T> where Self:'a;

    fn get(&self, key: Key<T>) -> Option<RefItemSlot<T, Self::GroupItem, Self::Alloc>> {
        self.0.get_slot(key.into()).map(|slot| {
            // This is safe because Self has total access to items and
            // we borrow &self so we can't mutate the T hence &T is safe.
            unsafe { slot.into_item(key) }
        })
    }

    fn iter(&self) -> Self::Iter<'_> {
        // This is safe because Self has total access to items and
        // we borrow &self so we can't mutate any T hence all &T is safe.
        unsafe {
            self.0
                .iter_slot()
                .into_iter()
                .flat_map(|iter| iter.map(|(key, slot)| slot.into_item(key.into_key())))
        }
    }
}

/// This guarantees that it will fetch and mutably reference only items and shells won't touch.
#[repr(transparent)]
pub struct AccessItemsAny<C: 'static>(C);

impl<C: 'static> AccessItemsAny<C> {
    pub fn new(collection: &mut C) -> &mut Self {
        // This is safe since Self is a transparent wrapper around C
        unsafe { &mut *(collection as *mut C as *mut Self) }
    }
}

impl<C: AnyContainer + 'static> AnyItems for AccessItemsAny<C> {
    fn get_any(&self, key: AnyKey) -> Option<RefAnyItemSlot> {
        self.0.any_get_slot(key.into()).map(|slot| {
            // This is safe because Self has total access to items and
            // we borrow &self so we can't mutate any item hence &AnyItem is safe.
            unsafe { slot.into_item(key) }
        })
    }

    fn get_mut_any(&mut self, key: AnyKey) -> Option<MutAnyItemSlot> {
        self.0.any_get_slot(key.into()).map(|slot| {
            // This is safe because Self has total access to items and
            // we borrow &mut self so there is no other &mut AnyItem hence &mut AnyItem is safe.
            unsafe { slot.into_item_mut(key) }
        })
    }
}

/// This guarantees that it will fetch and mutably reference only shells.
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

    fn get_mut(&mut self, key: Key<T>) -> Option<MutShellSlot<T, Self::Shell, Self::Alloc>> {
        (self.0).0.get_slot(key.into()).map(|slot| {
            // This is safe because Self has total access to shells and
            // we borrow &mut self so we can't alias Shell twice.
            unsafe { slot.into_shell_mut(key) }
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
                .flat_map(|iter| iter.map(|(key, slot)| slot.into_shell_mut(key.into_key())))
        }
    }
}

impl<'c, C: Container<T> + 'static, T: AnyItem> Shells<T> for AccessShellsMut<'c, C> {
    type Alloc = <C as Allocator<T>>::Alloc;

    type Shell = <C as Container<T>>::Shell;

    type Iter<'a> = ShellsIter<'a,C, T> where Self:'a;

    fn get(&self, key: Key<T>) -> Option<RefShellSlot<T, Self::Shell, Self::Alloc>> {
        self.0.get(key)
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.0.iter()
    }
}

impl<'c, C: AnyContainer + 'static> AnyShells for AccessShellsMut<'c, C> {
    fn get_any(&self, key: AnyKey) -> Option<RefAnyShellSlot> {
        (self.0).0.any_get_slot(key.into()).map(|slot| {
            // This is safe because Self has total access to shells and
            // we borrow &self so we can't mutate the Shell hence &Shell is safe.
            unsafe { slot.into_shell(key) }
        })
    }

    fn get_mut_any(&mut self, key: AnyKey) -> Option<MutAnyShellSlot> {
        (self.0).0.any_get_slot(key.into()).map(|slot| {
            // This is safe because Self has total access to shells and
            // we borrow &mut self so there is no other &mut Shell hence &mut Shell is safe.
            unsafe { slot.into_shell_mut(key) }
        })
    }
}

/// This guarantees that it will fetch and reference only shells.
pub struct AccessShells<'c, C: 'static>(&'c C);

impl<'c, C: 'static> AccessShells<'c, C> {
    /// UNSAFE: Caller must guarantee that no one will mut access
    /// shells in C collection during 'c lifetime.
    pub unsafe fn new(collection: &'c C) -> Self {
        Self(collection)
    }
}

impl<'c, C: Container<T> + 'static, T: AnyItem> Shells<T> for AccessShells<'c, C> {
    type Alloc = <C as Allocator<T>>::Alloc;

    type Shell = <C as Container<T>>::Shell;

    type Iter<'a> = ShellsIter<'a,C, T> where Self:'a;

    fn get(&self, key: Key<T>) -> Option<RefShellSlot<T, Self::Shell, Self::Alloc>> {
        self.0.get_slot(key.into()).map(|slot| {
            // This is safe because Self has total access to shells and
            // we borrow &self so we can't mutate the Shell hence &Shell is safe.
            unsafe { slot.into_shell(key) }
        })
    }

    fn iter(&self) -> Self::Iter<'_> {
        // This is safe because Self has total access to shells and
        // we borrow &self so we can't mutate any Shell hence all &Shell is safe.
        unsafe {
            self.0
                .iter_slot()
                .into_iter()
                .flat_map(|iter| iter.map(|(key, slot)| slot.into_shell(key.into_key())))
        }
    }
}

/// This guarantees that it will fetch and mutably reference only shells.
#[repr(transparent)]
pub struct AccessShellsAny<C: 'static>(C);

impl<C: 'static> AccessShellsAny<C> {
    pub fn new(collection: &mut C) -> &mut Self {
        // This is safe since Self is a transparent wrapper around C
        unsafe { &mut *(collection as *mut C as *mut Self) }
    }
}

impl<C: AnyContainer + 'static> AnyShells for AccessShellsAny<C> {
    fn get_any(&self, key: AnyKey) -> Option<RefAnyShellSlot> {
        self.0.any_get_slot(key.into()).map(|slot| {
            // This is safe because Self has total access to shells and
            // we borrow &self so we can't mutate the Shell hence &Shell is safe.
            unsafe { slot.into_shell(key) }
        })
    }

    fn get_mut_any(&mut self, key: AnyKey) -> Option<MutAnyShellSlot> {
        self.0.any_get_slot(key.into()).map(|slot| {
            // This is safe because Self has total access to shells and
            // we borrow &mut self so there is no other &mut Shell hence &mut Shell is safe.
            unsafe { slot.into_shell_mut(key) }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        container::{all::AllContainer, vec::VecContainerFamily},
        item::{edge::Edge, vertice::Vertice},
    };

    #[test]
    fn reference_add() {
        let n = 10;
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let nodes = (0usize..n)
            .map(|i| collection.add_with(i, ()).unwrap())
            .collect::<Vec<_>>();

        let center = collection
            .add_with(
                Vertice::new(nodes.iter().copied().map(Ref::new).collect()),
                (),
            )
            .ok()
            .unwrap();

        for node in nodes {
            assert_eq!(
                collection
                    .get(node)
                    .unwrap()
                    .1
                    .from::<Vertice<usize>>()
                    .collect::<Vec<_>>(),
                vec![center]
            );
        }
    }

    #[test]
    fn reference_add_abort() {
        let n = 10;
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let nodes = (0usize..n)
            .map(|i| collection.add_with(i, ()).unwrap())
            .collect::<Vec<_>>();

        collection.take(nodes[n - 1]).unwrap();

        assert!(collection
            .add_with(
                Vertice::new(nodes.iter().copied().map(Ref::new).collect()),
                ()
            )
            .is_err());

        for &node in nodes.iter().take(n - 1) {
            assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
        }
    }

    #[test]
    fn reference_set() {
        let n = 10;
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let nodes = (0usize..n)
            .map(|i| collection.add(i).unwrap())
            .collect::<Vec<_>>();

        let center = collection
            .add_with(
                Vertice::new(nodes.iter().take(5).copied().map(Ref::new).collect()),
                (),
            )
            .ok()
            .unwrap();

        collection
            .set(
                center,
                Vertice::new(nodes.iter().skip(5).copied().map(Ref::new).collect()),
            )
            .ok()
            .unwrap();

        for &node in nodes.iter().take(5) {
            assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
        }

        for &node in nodes.iter().skip(5) {
            assert_eq!(
                collection
                    .get(node)
                    .unwrap()
                    .1
                    .from::<Vertice<usize>>()
                    .collect::<Vec<_>>(),
                vec![center]
            );
        }
    }

    #[test]
    fn reference_set_abort() {
        let n = 10;
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let nodes = (0usize..n)
            .map(|i| collection.add(i).unwrap())
            .collect::<Vec<_>>();

        collection.take(nodes[n - 1]).unwrap();

        let center = collection
            .add_with(
                Vertice::new(nodes.iter().take(5).copied().map(Ref::new).collect()),
                (),
            )
            .ok()
            .unwrap();

        assert!(collection
            .add_with(
                Vertice::new(nodes.iter().skip(5).copied().map(Ref::new).collect()),
                ()
            )
            .is_err());

        for &node in nodes.iter().take(5) {
            assert_eq!(
                collection
                    .get(node)
                    .unwrap()
                    .1
                    .from::<Vertice<usize>>()
                    .collect::<Vec<_>>(),
                vec![center]
            );
        }

        for &node in nodes.iter().skip(5).take(4) {
            assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
        }
    }

    #[test]
    fn reference_remove() {
        let n = 10;
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let nodes = (0usize..n)
            .map(|i| collection.add_with(i, ()).unwrap())
            .collect::<Vec<_>>();

        let center = collection
            .add_with(
                Vertice::new(nodes.iter().copied().map(Ref::new).collect()),
                (),
            )
            .ok()
            .unwrap();

        let _ = collection.take(center).unwrap();

        for node in nodes {
            assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
        }
    }

    #[test]
    fn cascading_remove() {
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let a = collection.add_with(0, ()).unwrap();
        let b = collection.add_with(1, ()).unwrap();
        let edge = collection
            .add_with(Edge::new([Ref::new(a), Ref::new(b)]), ())
            .unwrap();

        assert_eq!(collection.get(a).unwrap().1.from_count(), 1);
        assert_eq!(collection.get(b).unwrap().1.from_count(), 1);

        let _ = collection.take(a).unwrap();
        assert!(collection.get(edge).is_none());
        assert!(collection.get(a).is_none());
        assert!(collection.get(b).unwrap().0 == (&1, &()));
        assert_eq!(collection.get(b).unwrap().1.from_count(), 0);
    }
}
