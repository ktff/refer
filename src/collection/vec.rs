use crate::core::*;
use std::{any::TypeId, marker::PhantomData, num::NonZeroU64};

pub type ItemIter<'a, T: Item> = impl Iterator<Item = (Key<T>, &'a T)>;
pub type ItemMutIter<'a, T: Item> = impl Iterator<Item = (Key<T>, &'a mut T)>;

pub type VecRefShellIter<'a, F: ?Sized + 'static> = impl Iterator<Item = Key<F>> + 'a;
pub type VecRefShellAnyIter<'a> = impl Iterator<Item = AnyKey> + 'a;

pub type ShellIter<'a, T: Item + ?Sized> = impl Iterator<Item = (Key<T>, VecRefShell<'a, T>)>;
pub type ShellMutIter<'a, T: Item + ?Sized> = impl Iterator<Item = (Key<T>, VecMutShell<'a, T>)>;

pub type EntityIter<'a, T: Item> = impl Iterator<Item = (Key<T>, &'a T, VecRefShell<'a, T>)>;
pub type EntityMutIter<'a, T: Item> = impl Iterator<Item = (Key<T>, &'a mut T, VecRefShell<'a, T>)>;

/// A simple vec collection of items of the same type.
pub struct VecCollection<T: Item> {
    items: VecItemCollection<T>,
    shells: VecShellCollection<T>,
    free: Vec<Key<T>>,
}

impl<T: Item> VecCollection<T> {
    pub fn new() -> Self {
        Self {
            items: VecItemCollection(vec![None]),
            shells: VecShellCollection(vec![None]),
            free: Vec::new(),
        }
    }
}

impl<T: Item> Collection<T> for VecCollection<T> {
    type Items = VecItemCollection<T>;

    type Shells = VecShellCollection<T>;

    type Iter<'a> = EntityIter<'a,T> where Self: 'a;

    type MutIter<'a> = EntityMutIter<'a,T> where Self: 'a;

    fn reserve(&mut self) -> Option<Key<T>> {
        if self.free.is_empty() {
            self.free.push(Key::new(Index(
                NonZeroU64::new(self.items.0.len() as u64).expect("Zero index"),
            )));
            self.items.0.push(None);
            self.shells.0.push(None);
        }
        self.free.last().copied()
    }

    fn cancel(&mut self, key: Key<T>) {
        assert!(self.items().get(key).is_none());
        assert_eq!(self.free.last().copied(), Some(key));
    }

    fn fulfill(&mut self, key: Key<T>, item: T) {
        assert_eq!(self.free.pop(), Some(key));

        self.items.0[key.as_usize()] = Some(item);
        self.shells.0[key.as_usize()] = Some(VecShell {
            from: Vec::new(),
            _data: PhantomData,
        });
    }

    /// Frees and returns item if it exists
    fn unfill(&mut self, key: Key<T>) -> Option<T>
    where
        T: Sized,
    {
        let item = self
            .items
            .0
            .get_mut(key.as_usize())
            .and_then(|slot| slot.take())?;
        self.shells
            .0
            .get_mut(key.as_usize())
            .and_then(|slot| slot.take())
            .expect("Shell not found");
        self.free.push(key);

        Some(item)
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

impl<T: Item> AnyCollection for VecCollection<T> {
    type AnyItems = VecItemCollection<T>;

    type AnyShells = VecShellCollection<T>;

    fn split_any_mut(&mut self) -> (&mut Self::AnyItems, &mut Self::AnyShells) {
        (&mut self.items, &mut self.shells)
    }

    fn unfill_any(&mut self, key: AnyKey) -> bool {
        if let Some(key) = key.downcast() {
            self.unfill(key).is_some()
        } else {
            false
        }
    }
}

impl<T: Item> KeyCollection for VecCollection<T> {
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
        if TypeId::of::<I>() == TypeId::of::<T>() {
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
        } else {
            None
        }
    }
}

impl<T: Item> Default for VecCollection<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct VecItemCollection<T: Item>(Vec<Option<T>>);

impl<T: Item> ItemCollection<T> for VecItemCollection<T> {
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

impl<T: Item> AnyItemCollection for VecItemCollection<T> {
    fn contains(&self, key: AnyKey) -> bool {
        key.downcast().and_then(|key| self.get(key)).is_some()
    }

    fn references(&self, key: AnyKey) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>> {
        key.downcast()
            .and_then(|key| self.get(key))
            .map(|item| item.references_any())
    }

    fn remove_reference(&mut self, key: AnyKey, rf: AnyKey) -> bool {
        key.downcast()
            .and_then(|key| self.get_mut(key))
            .map(|item| item.remove_reference(rf))
            .unwrap_or(true)
    }
}

pub struct VecShellCollection<T: Item + ?Sized>(Vec<Option<VecShell<T>>>);

impl<T: Item + ?Sized> ShellCollection<T> for VecShellCollection<T> {
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
}

impl<T: Item + ?Sized> AnyShellCollection for VecShellCollection<T> {
    fn contains(&self, key: AnyKey) -> bool {
        key.downcast().and_then(|key| self.get(key)).is_some()
    }

    fn add_from(&mut self, key: AnyKey, rf: AnyKey) -> bool {
        key.downcast()
            .and_then(|key| self.get_mut(key))
            .map(|mut shell| shell.add_from(rf))
            .is_some()
    }

    fn from(&self, key: AnyKey) -> Option<Box<dyn Iterator<Item = AnyKey> + '_>> {
        key.downcast()
            .and_then(|key| self.get(key))
            .map(|shell| shell.from_any())
    }

    fn remove_from(&mut self, key: AnyKey, rf: AnyKey) -> bool {
        key.downcast()
            .and_then(|key| self.get_mut(key))
            .map(|mut shell| shell.remove_from(rf))
            .unwrap_or(false)
    }
}

pub struct VecRefShell<'a, T: Item + ?Sized>(&'a VecShell<T>);

impl<'a, T: Item + ?Sized> AnyShell<'a> for VecRefShell<'a, T> {
    fn item_ty(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn from_any(&self) -> Box<dyn Iterator<Item = AnyKey> + 'a> {
        Box::new(self.iter())
    }

    fn from_count(&self) -> usize {
        self.0.from.len()
    }
}

impl<'a, T: Item + ?Sized> RefShell<'a> for VecRefShell<'a, T> {
    type T = T;
    type Iter<F: ?Sized + 'static> = VecRefShellIter<'a, F>;
    type AnyIter = VecRefShellAnyIter<'a>;

    fn iter(&self) -> Self::AnyIter {
        self.0.from.iter().copied()
    }

    fn from<F: ?Sized + 'static>(&self) -> Self::Iter<F> {
        self.iter().filter_map(AnyKey::downcast)
    }
}

pub struct VecMutShell<'a, T: Item + ?Sized>(&'a mut VecShell<T>);

impl<'a, T: Item + ?Sized> MutShell<'a> for VecMutShell<'a, T> {
    fn add_from(&mut self, from: AnyKey) {
        self.0.from.push(from);
    }

    fn remove_from(&mut self, from: AnyKey) -> bool {
        // TODO: This will be really slow for large froms.
        if let Some((i, _)) = self
            .0
            .from
            .iter()
            .enumerate()
            .rev()
            .find(|(_, key)| key == &&from)
        {
            self.0.from.remove(i);
            true
        } else {
            false
        }
    }
}

struct VecShell<T: Item + ?Sized> {
    from: Vec<AnyKey>,
    _data: PhantomData<T>,
}
