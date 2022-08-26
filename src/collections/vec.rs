use crate::core::*;
use std::{any::TypeId, cell::UnsafeCell, marker::PhantomData, mem::MaybeUninit, num::NonZeroU64};

pub type ItemIter<'a, T: 'static> = impl Iterator<Item = (Key<T>, &'a T)>;
pub type ItemMutIter<'a, T: 'static> = impl Iterator<Item = (Key<T>, &'a mut T)>;

pub type VecRefShellIter<'a, F: ?Sized + 'static> = impl Iterator<Item = Key<F>> + 'a;

pub type ShellIter<'a, T: ?Sized + 'static> = impl Iterator<Item = (Key<T>, VecRefShell<'a, T>)>;
pub type ShellMutIter<'a, T: ?Sized + 'static> = impl Iterator<Item = (Key<T>, VecMutShell<'a, T>)>;

pub type EntityIter<'a, T: 'static> = impl Iterator<Item = (Key<T>, &'a T, VecRefShell<'a, T>)>;
pub type EntityMutIter<'a, T: 'static> =
    impl Iterator<Item = (Key<T>, &'a mut T, VecRefShell<'a, T>)>;

/// A simple vec collection of items of the same type.
pub struct VecCollection<T: 'static> {
    items: VecItemCollection<T>,
    shells: VecShellCollection<T>,
    free: Vec<Key<T>>,
}

impl<T: 'static> VecCollection<T> {
    pub fn new() -> Self {
        Self {
            items: VecItemCollection(vec![None]),
            shells: VecShellCollection(vec![None]),
            free: Vec::new(),
        }
    }

    /// Removes entity if exists.
    /// Can add additional entity to remove list.
    fn remove_entity(&mut self, key: Key<T>, remove: &mut Vec<AnyKey>) -> Option<T>
    where
        T: Item,
    {
        // Deallocate
        let item = self
            .items
            .0
            .get_mut(key.as_usize())
            .and_then(|slot| slot.take())?;
        let shell = self
            .shells
            .0
            .get_mut(key.as_usize())
            .and_then(|slot| slot.take())
            .expect("Shell not found");
        self.free.push(key);

        // Update connections

        // item --> others
        for rf in item.references() {
            if let Some(mut other) = self
                .shells
                .get_mut(rf.downcast::<T>().expect("Should be T").key())
            {
                other.remove_from(key.into());
            }
        }

        // item <-- others
        for &rf in shell.from() {
            if let Some(other) = self.items.get_mut(rf.downcast::<T>().expect("Should be T")) {
                if !other.remove_reference(key.into(), &item) {
                    remove.push(rf);
                }
            }
        }

        Some(item)
    }
}

impl<T: 'static> Collection<T> for VecCollection<T> {
    type Items = VecItemCollection<T>;

    type Shells = VecShellCollection<T>;

    type Iter<'a> = EntityIter<'a,T> where Self: 'a;

    type MutIter<'a> = EntityMutIter<'a,T> where Self: 'a;

    unsafe fn allocate(&mut self, _: &T) -> Option<(Key<T>, &mut [MaybeUninit<u8>])> {
        unimplemented!()
    }

    unsafe fn free<R>(&mut self, _: Key<T>, _: impl FnOnce(&[MaybeUninit<u8>]) -> R) -> Option<R> {
        unimplemented!()
    }

    fn add(&mut self, prefix: Option<Prefix>, item: T) -> Result<Key<T>, T>
    where
        T: Item + Sized,
    {
        assert!(prefix.is_none(), "Not yet implemented");

        // Allocate slot
        let key = if let Some(key) = self.free.pop() {
            key
        } else {
            let key = Key::new(Index(
                NonZeroU64::new(self.items.0.len() as u64).expect("Zero index"),
            ));
            self.items.0.push(None);
            self.shells.0.push(None);
            key
        };

        // Update connections

        // item --> others
        let mut references = item.references().enumerate();
        for (i, rf) in &mut references {
            if let Some(rf) = rf.downcast::<T>() {
                if let Some(mut shell) = self.shells.get_mut(rf.key()) {
                    // Reference exists
                    shell.add_from(key.into());
                    continue;
                }
            }

            // Reference doesn't exist

            // Rollback and return error
            for rf in item.references().take(i) {
                let mut shell = self
                    .shells
                    .get_mut(rf.downcast::<T>().expect("Should be T").key())
                    .expect("Should exist");
                shell.remove_from(key.into());
            }

            // Deallocate slot
            self.free.push(key);

            drop(references);
            return Err(item);
        }
        drop(references);

        // Add item & shell
        self.items.0[key.as_usize()] = Some(item);
        self.shells.0[key.as_usize()] = Some(VecShell::new());

        Ok(key)
    }

    fn add_copy(&mut self, prefix: Option<Prefix>, item: &T) -> Option<Key<T>>
    where
        T: Item + Copy,
    {
        self.add(prefix, *item).ok()
    }

    fn set(&mut self, key: Key<T>, set: T) -> Result<T, T>
    where
        T: Item + Sized,
    {
        // Update connections
        let mut old = if let Some(item) = self.items.get(key) {
            item.references().collect::<Vec<_>>()
        } else {
            // No item
            return Err(set);
        };
        let mut new = set.references().collect::<Vec<_>>();
        old.sort();
        new.sort();

        // item --> others
        for (i, cmp) in crate::util::merge(&old, &new).enumerate() {
            match cmp {
                (Some(_), Some(_)) | (None, None) => (),
                (Some(rf), None) => {
                    if let Some(mut shell) = self
                        .shells
                        .get_mut(rf.downcast::<T>().expect("Should be T").key())
                    {
                        shell.remove_from(key.into());
                    }
                }
                (None, Some(rf)) => {
                    if let Some(mut shell) = self
                        .shells
                        .get_mut(rf.downcast::<T>().expect("Should be T").key())
                    {
                        shell.add_from(key.into());
                    } else {
                        // Reference doesn't exist

                        // Rollback and return error
                        for cmp in crate::util::merge(&old, &new).take(i) {
                            match cmp {
                                (Some(_), Some(_)) | (None, None) => (),
                                (Some(rf), None) => {
                                    let mut shell = self
                                        .shells
                                        .get_mut(rf.downcast::<T>().expect("Should be T").key())
                                        .expect("Should exist");
                                    shell.add_from(key.into());
                                }
                                (None, Some(rf)) => {
                                    let mut shell = self
                                        .shells
                                        .get_mut(rf.downcast::<T>().expect("Should be T").key())
                                        .expect("Should exist");
                                    shell.remove_from(key.into());
                                }
                            }
                        }

                        return Err(set);
                    }
                }
            }
        }

        // Replace item
        let item = std::mem::replace(
            self.items
                .0
                .get_mut(key.as_usize())
                .and_then(|slot| slot.as_mut())
                .expect("Item not found"),
            set,
        );

        Ok(item)
    }

    fn set_copy(&mut self, key: Key<T>, set: &T) -> bool
    where
        T: Item + Copy,
    {
        self.set(key, *set).is_ok()
    }

    fn remove(&mut self, key: Key<T>) -> bool
    where
        T: Item,
    {
        self.take(key).is_some()
    }

    fn take(&mut self, key: Key<T>) -> Option<T>
    where
        T: Item + Sized,
    {
        // First remove
        let mut remove = Vec::new();
        let item = self.remove_entity(key, &mut remove);

        // Recursive remove
        while let Some(rf) = remove.pop() {
            let _ = self.remove_entity(rf.downcast::<T>().expect("Should be T"), &mut remove);
        }

        item
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.items
            .iter()
            .zip(self.shells.iter())
            .map(|((k0, item), (k1, shell))| {
                debug_assert_eq!(k0, k1);
                (k0, item, shell)
            })
    }

    fn iter_mut(&mut self) -> Self::MutIter<'_> {
        self.items
            .iter_mut()
            .zip(self.shells.iter())
            .map(|((k0, item), (k1, shell))| {
                debug_assert_eq!(k0, k1);
                (k0, item, shell)
            })
    }

    fn split(&self) -> (&Self::Items, &Self::Shells) {
        (&self.items, &self.shells)
    }

    fn split_mut(&mut self) -> (&mut Self::Items, &mut Self::Shells) {
        (&mut self.items, &mut self.shells)
    }
}

impl<T: 'static> KeyCollection for VecCollection<T> {
    fn prefix(&self) -> Option<Prefix> {
        None
    }

    fn first<I: ?Sized + 'static>(&self) -> Option<Key<I>> {
        if TypeId::of::<I>() == TypeId::of::<T>() {
            self.items
                .0
                .iter()
                .enumerate()
                .filter_map(|(i, slot)| {
                    slot.as_ref().map(|_| {
                        Key::new(Index(
                            NonZeroU64::new(i as u64).expect("Zero index is allocated"),
                        ))
                    })
                })
                .next()
        } else {
            None
        }
    }

    fn next<I: ?Sized + 'static>(&self, key: Key<I>) -> Option<Key<I>> {
        self.items
            .0
            .iter()
            .enumerate()
            .skip(key.as_usize())
            .filter_map(|(i, slot)| {
                slot.as_ref().map(|_| {
                    Key::new(Index(
                        NonZeroU64::new(i as u64).expect("Zero index is allocated"),
                    ))
                })
            })
            .next()
    }
}

pub struct VecItemCollection<T: 'static>(Vec<Option<T>>);

impl<T: 'static> ItemCollection<T> for VecItemCollection<T> {
    type Iter<'a> = ItemIter<'a, T>;

    type MutIter<'a> = ItemMutIter<'a, T>;

    fn get(&self, key: Key<T>) -> Option<&T> {
        self.0.get(key.as_usize()).and_then(|slot| slot.as_ref())
    }

    fn get_mut(&mut self, key: Key<T>) -> Option<&mut T> {
        self.0
            .get_mut(key.as_usize())
            .and_then(|slot| slot.as_mut())
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.0.iter().enumerate().filter_map(|(i, slot)| {
            slot.as_ref().map(|item| {
                (
                    Key::new(Index(
                        NonZeroU64::new(i as u64).expect("Zero index is allocated"),
                    )),
                    item,
                )
            })
        })
    }

    fn iter_mut(&mut self) -> Self::MutIter<'_> {
        self.0.iter_mut().enumerate().filter_map(|(i, slot)| {
            slot.as_mut().map(|item| {
                (
                    Key::new(Index(
                        NonZeroU64::new(i as u64).expect("Zero index is allocated"),
                    )),
                    item,
                )
            })
        })
    }
}

pub struct VecShellCollection<T: 'static + ?Sized>(Vec<Option<VecShell<T>>>);

impl<T: ?Sized + 'static> ShellCollection<T> for VecShellCollection<T> {
    type MutColl<'a>=VecMutShellCollection<'a,T>
    where
        Self: 'a;

    type Ref<'a>= VecRefShell<'a, T>
    where
        Self: 'a;

    type Mut<'a>= VecMutShell<'a, T>
    where
        Self: 'a;

    type Iter<'a>=ShellIter<'a,T>
    where
        Self: 'a;

    type MutIter<'a>=ShellMutIter<'a,T>
    where
        Self: 'a;

    fn get(&self, key: Key<T>) -> Option<Self::Ref<'_>> {
        self.0
            .get(key.as_usize())
            .and_then(|slot| slot.as_ref())
            .map(VecRefShell)
    }

    fn get_mut(&mut self, key: Key<T>) -> Option<Self::Mut<'_>> {
        self.0
            .get_mut(key.as_usize())
            .and_then(|slot| slot.as_mut())
            .map(VecMutShell)
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.0.iter().enumerate().filter_map(|(i, slot)| {
            slot.as_ref().map(|shell| {
                (
                    Key::new(Index(
                        NonZeroU64::new(i as u64).expect("Zero index is allocated"),
                    )),
                    VecRefShell(shell),
                )
            })
        })
    }

    fn iter_mut(&mut self) -> Self::MutIter<'_> {
        self.0.iter_mut().enumerate().filter_map(|(i, slot)| {
            slot.as_mut().map(|shell| {
                (
                    Key::new(Index(
                        NonZeroU64::new(i as u64).expect("Zero index is allocated"),
                    )),
                    VecMutShell(shell),
                )
            })
        })
    }

    fn mut_coll(&mut self) -> Self::MutColl<'_> {
        VecMutShellCollection(self)
    }
}

pub struct VecRefShell<'a, T: ?Sized + 'static>(&'a VecShell<T>);

impl<'a, T: ?Sized + 'static> AnyShell<'a> for VecRefShell<'a, T> {
    fn item_ty(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn from_any(&self) -> Vec<AnyKey> {
        self.0.from().clone()
    }

    fn from_count(&self) -> usize {
        self.0.from().len()
    }
}

impl<'a, T: ?Sized + 'static> RefShell<'a> for VecRefShell<'a, T> {
    type T = T;
    type Iter<F: ?Sized + 'static> = VecRefShellIter<'a, F>;

    fn from<F: ?Sized + 'static>(&self) -> Self::Iter<F> {
        self.0.from().iter().copied().filter_map(AnyKey::downcast)
    }
}

pub struct VecMutShell<'a, T: ?Sized + 'static>(&'a mut VecShell<T>);

impl<'a, T: ?Sized + 'static> MutShell<'a> for VecMutShell<'a, T> {
    type T = T;

    fn add_from(&mut self, from: AnyKey) {
        self.0.from.get_mut().push(from);
    }

    fn remove_from(&mut self, from: AnyKey) -> bool {
        let vec = self.0.from.get_mut();

        // TODO: This will be really slow for large froms.
        if let Some((i, _)) = vec.iter().enumerate().rev().find(|(_, key)| key == &&from) {
            vec.remove(i);
            true
        } else {
            false
        }
    }
}

pub struct VecMutShellCollection<'a, T: ?Sized + 'static>(&'a mut VecShellCollection<T>);

impl<'a, T: ?Sized + 'static> MutShellCollection<'a, T> for VecMutShellCollection<'a, T> {
    type Mut<'b> = VecRefMutShell<'b, T> where Self: 'b;

    fn get_mut(&self, key: Key<T>) -> Option<Self::Mut<'_>> {
        (self.0)
            .0
            .get(key.as_usize())
            .and_then(|slot| slot.as_ref())
            .map(VecRefMutShell)
    }
}

pub struct VecRefMutShell<'a, T: ?Sized + 'static>(&'a VecShell<T>);

impl<'a, T: ?Sized + 'static> VecRefMutShell<'a, T> {
    /// This function is safe to call with combination of &'a mut of shell collection
    /// being bounded in VecMutShellCollection and that all uses of this field don't overlap.
    /// This exclusivity is achieved by the caller ensuring that:
    /// - This isn't leaked outside of it.
    /// - It doesn't allow in any way creation of additional reference for this field.
    unsafe fn enter(&self) -> &mut Vec<AnyKey> {
        // Caller must ensure no other references to this field exist nor
        // will there exist for this lifetime.
        &mut *self.0.from.get()
    }
}

impl<'a, T: ?Sized + 'static> MutShell<'a> for VecRefMutShell<'a, T> {
    type T = T;

    fn add_from(&mut self, from: AnyKey) {
        // By the contract of enter this reference isn't leaked and there is no
        // way for following operations to create additional references.
        let vec = unsafe { self.enter() };

        vec.push(from);
    }

    fn remove_from(&mut self, from: AnyKey) -> bool {
        // By the contract of enter this reference isn't leaked and there is no
        // way for following operations to create additional references.
        let vec = unsafe { self.enter() };

        // TODO: This will be really slow for large froms.
        if let Some((i, _)) = vec.iter().enumerate().rev().find(|(_, key)| key == &&from) {
            vec.remove(i);
            true
        } else {
            false
        }
    }
}

struct VecShell<T: ?Sized + 'static> {
    from: UnsafeCell<Vec<AnyKey>>,
    _data: PhantomData<T>,
}

impl<T: ?Sized + 'static> VecShell<T> {
    fn new() -> Self {
        Self {
            from: UnsafeCell::new(Vec::new()),
            _data: PhantomData,
        }
    }

    fn from(&self) -> &Vec<AnyKey> {
        // This is safe on it's own.
        unsafe { &*self.from.get() }
    }
}
