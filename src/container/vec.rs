use crate::core::*;
use std::{any::TypeId, cell::UnsafeCell, marker::PhantomData, num::NonZeroU64};

pub type VecRefShellIter<'a, F: ?Sized + 'static> = impl Iterator<Item = Key<F>> + 'a;
pub type VecRefShellAnyIter<'a> = impl Iterator<Item = AnyKey> + 'a;

pub type CellIter<'a, T: AnyItem> =
    impl Iterator<Item = (Key<T>, &'a UnsafeCell<T>, &'a UnsafeCell<VecShell<T>>)>;

pub struct VecContainerFamily;

impl SizedContainerFamily for VecContainerFamily {
    type C<T: AnyItem> = VecContainer<T>;
}

/// A simple vec collection of items of the same type.
pub struct VecContainer<T: AnyItem> {
    items: Vec<Option<UnsafeCell<T>>>,
    shells: Vec<Option<UnsafeCell<VecShell<T>>>>,
    free: Vec<Key<T>>,
    reserved: Vec<Key<T>>,
}

impl<T: AnyItem> VecContainer<T> {
    pub fn new() -> Self {
        Self {
            items: vec![None],
            shells: vec![None],
            free: Vec::new(),
            reserved: Vec::new(),
        }
    }

    fn resolve_reservation(&mut self, key: ReservedKey<T>) -> Key<T> {
        let (i, _) = self
            .reserved
            .iter()
            .enumerate()
            .find(|(_, k)| **k == key.key())
            .expect("There is no reservation");
        self.reserved.remove(i);
        key.take()
    }
}

impl<T: AnyItem> Container<T> for VecContainer<T> {
    type Shell = VecShell<T>;

    type CellIter<'a> = CellIter<'a, T> where Self: 'a;

    fn reserve(&mut self) -> Option<ReservedKey<T>> {
        let key = if let Some(key) = self.free.pop() {
            key
        } else {
            let key = Key::new(Index(
                NonZeroU64::new(self.items.len() as u64).expect("Zero index"),
            ));
            self.items.push(None);
            self.shells.push(None);
            key
        };

        self.reserved.push(key);
        Some(ReservedKey::new(key))
    }

    fn cancel(&mut self, key: ReservedKey<T>) {
        let key = self.resolve_reservation(key);
        self.free.push(key);
    }

    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> Key<T> {
        let key = self.resolve_reservation(key);

        self.items[key.as_usize()] = Some(item.into());
        self.shells[key.as_usize()] = Some(
            VecShell {
                from: Vec::new(),
                _data: PhantomData,
            }
            .into(),
        );

        key
    }

    /// Frees and returns item if it exists
    fn unfill(&mut self, key: Key<T>) -> Option<T>
    where
        T: Sized,
    {
        let item = self
            .items
            .get_mut(key.as_usize())
            .and_then(|slot| slot.take())?;
        self.shells
            .get_mut(key.as_usize())
            .and_then(|slot| slot.take())
            .expect("Shell not found");
        self.free.push(key);

        Some(item.into_inner())
    }

    fn get_slot(&self, key: Key<T>) -> Option<(&UnsafeCell<T>, &UnsafeCell<Self::Shell>)> {
        let item = self
            .items
            .get(key.as_usize())
            .and_then(|slot| slot.as_ref())?;
        let shell = self
            .shells
            .get(key.as_usize())
            .and_then(|slot| slot.as_ref())
            .expect("Should exist");
        Some((item, shell))
    }

    fn iter_slot(&self) -> Option<Self::CellIter<'_>> {
        Some(
            self.items
                .iter()
                .zip(self.shells.iter())
                .enumerate()
                .filter_map(|(i, (item, shell))| {
                    let item = item.as_ref()?;
                    let shell = shell.as_ref().expect("Should exist");
                    Some((
                        Key::new(Index(NonZeroU64::new(i as u64).expect("Zero index"))),
                        item,
                        shell,
                    ))
                }),
        )
    }
}

impl<T: AnyItem> AnyContainer for VecContainer<T> {
    fn any_get_slot(
        &self,
        key: AnyKey,
    ) -> Option<(&UnsafeCell<dyn AnyItem>, &UnsafeCell<dyn AnyShell>)> {
        let item = self
            .items
            .get(key.downcast::<T>()?.as_usize())
            .and_then(|slot| slot.as_ref())
            .map(|item| item as &_)?;
        let shell = self
            .shells
            .get(key.downcast::<T>()?.as_usize())
            .and_then(|slot| slot.as_ref())
            .map(|shell| shell as &_)
            .expect("Should exist");

        Some((item, shell))
    }

    /// Frees if it exists.
    fn any_unfill(&mut self, key: AnyKey) -> bool {
        if let Some(key) = key.downcast() {
            self.unfill(key).is_some()
        } else {
            false
        }
    }
}

impl<T: AnyItem> KeyContainer for VecContainer<T> {
    fn prefix(&self) -> Option<Prefix> {
        None
    }

    fn first<I: AnyItem>(&self) -> Option<Key<I>> {
        if TypeId::of::<I>() == TypeId::of::<T>() {
            self.items
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

    fn next<I: AnyItem>(&self, key: Key<I>) -> Option<Key<I>> {
        if TypeId::of::<I>() == TypeId::of::<T>() {
            self.items
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

impl<T: AnyItem> Default for VecContainer<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct VecShell<T: AnyItem + ?Sized> {
    from: Vec<AnyKey>,
    _data: PhantomData<T>,
}

impl<T: AnyItem + ?Sized> AnyShell for VecShell<T> {
    fn item_ty(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn from_any(&self) -> Box<dyn Iterator<Item = AnyKey> + '_> {
        Box::new(self.iter())
    }

    fn from_count(&self) -> usize {
        self.from.len()
    }

    fn add_from(&mut self, from: AnyKey) {
        self.from.push(from);
    }

    fn remove_from(&mut self, from: AnyKey) -> bool {
        // TODO: This will be really slow for large froms.
        if let Some((i, _)) = self
            .from
            .iter()
            .enumerate()
            .rev()
            .find(|(_, key)| key == &&from)
        {
            self.from.remove(i);
            true
        } else {
            false
        }
    }
}

impl<T: AnyItem + ?Sized> Shell for VecShell<T> {
    type T = T;
    type Iter<'a, F: ?Sized + 'static> = VecRefShellIter<'a, F>;
    type AnyIter<'a> = VecRefShellAnyIter<'a>;

    fn iter(&self) -> Self::AnyIter<'_> {
        self.from.iter().copied()
    }

    fn from<F: ?Sized + 'static>(&self) -> Self::Iter<'_, F> {
        self.iter().filter_map(AnyKey::downcast)
    }
}
