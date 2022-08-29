use crate::core::*;
use std::{any::TypeId, cell::UnsafeCell, marker::PhantomData, num::NonZeroU64};

pub type VecRefShellIter<'a, F: ?Sized + 'static> = impl Iterator<Item = Key<F>> + 'a;
pub type VecRefShellAnyIter<'a> = impl Iterator<Item = AnyKey> + 'a;

pub type CellIter<'a, T: AnyItem> =
    impl Iterator<Item = (SubKey<T>, &'a UnsafeCell<T>, &'a UnsafeCell<VecShell<T>>)>;

pub struct VecContainerFamily;

impl SizedContainerFamily for VecContainerFamily {
    type C<T: AnyItem> = VecContainer<T>;
}

/// A simple vec collection of items of the same type.
pub struct VecContainer<T: AnyItem> {
    items: Vec<Option<UnsafeCell<T>>>,
    shells: Vec<Option<UnsafeCell<VecShell<T>>>>,
    free: Vec<Index>,
    reserved: Vec<Index>,
    key_len: u32,
}

impl<T: AnyItem> VecContainer<T> {
    pub fn new(key_len: u32) -> Self {
        Self {
            items: vec![None],
            shells: vec![None],
            free: Vec::new(),
            reserved: Vec::new(),
            key_len: key_len,
        }
    }

    fn resolve_reservation(&mut self, key: ReservedKey<T>) -> SubKey<T> {
        let (i, _) = self
            .reserved
            .iter()
            .enumerate()
            .find(|(_, k)| **k == key.key().index(self.key_len))
            .expect("There is no reservation");
        self.reserved.remove(i);
        key.take()
    }
}

impl<T: AnyItem> Allocator<T> for VecContainer<T> {
    fn reserve(&mut self, _: &T) -> Option<ReservedKey<T>> {
        let index = if let Some(index) = self.free.pop() {
            index
        } else {
            if self.items.len() >= (1 << self.key_len) {
                // Out of keys
                return None;
            }

            let index = Index(NonZeroU64::new(self.items.len() as u64).expect("Zero index"));
            self.items.push(None);
            self.shells.push(None);
            index
        };

        self.reserved.push(index);
        Some(ReservedKey::new(SubKey::new(self.key_len, index)))
    }

    fn cancel(&mut self, key: ReservedKey<T>) {
        let key = self.resolve_reservation(key);
        self.free.push(key.index(self.key_len));
    }

    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T> {
        let key = self.resolve_reservation(key);

        let index = key.index(self.key_len);
        self.items[index.as_usize()] = Some(item.into());
        self.shells[index.as_usize()] = Some(
            VecShell {
                from: Vec::new(),
                _data: PhantomData,
            }
            .into(),
        );

        key
    }

    /// Frees and returns item if it exists
    fn unfill(&mut self, key: SubKey<T>) -> Option<T>
    where
        T: Sized,
    {
        let index = key.index(self.key_len);
        let item = self
            .items
            .get_mut(index.as_usize())
            .and_then(|slot| slot.take())?;
        self.shells
            .get_mut(index.as_usize())
            .and_then(|slot| slot.take())
            .expect("Shell not found");
        self.free.push(index);

        Some(item.into_inner())
    }
}

impl<T: AnyItem> Container<T> for VecContainer<T> {
    type Shell = VecShell<T>;

    type CellIter<'a> = CellIter<'a, T> where Self: 'a;

    fn get_slot(&self, key: SubKey<T>) -> Option<(&UnsafeCell<T>, &UnsafeCell<Self::Shell>)> {
        let i = key.as_usize(self.key_len);
        let item = self.items.get(i).and_then(|slot| slot.as_ref())?;
        let shell = self
            .shells
            .get(i)
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
                        SubKey::new(
                            self.key_len,
                            Index(NonZeroU64::new(i as u64).expect("Zero index")),
                        ),
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
        key: AnySubKey,
    ) -> Option<(&UnsafeCell<dyn AnyItem>, &UnsafeCell<dyn AnyShell>)> {
        let i = key.downcast::<T>()?.as_usize(self.key_len);
        let item = self
            .items
            .get(i)
            .and_then(|slot| slot.as_ref())
            .map(|item| item as &_)?;
        let shell = self
            .shells
            .get(i)
            .and_then(|slot| slot.as_ref())
            .map(|shell| shell as &_)
            .expect("Should exist");

        Some((item, shell))
    }

    /// Frees if it exists.
    fn any_unfill(&mut self, key: AnySubKey) -> bool {
        if let Some(key) = key.downcast() {
            self.unfill(key).is_some()
        } else {
            false
        }
    }
}

impl<T: AnyItem> KeyContainer for VecContainer<T> {
    fn first<I: AnyItem>(&self) -> Option<SubKey<I>> {
        if TypeId::of::<I>() == TypeId::of::<T>() {
            self.items
                .iter()
                .enumerate()
                .filter_map(|(i, slot)| {
                    slot.as_ref().map(|_| {
                        SubKey::new(
                            self.key_len,
                            Index(NonZeroU64::new(i as u64).expect("Zero index is allocated")),
                        )
                    })
                })
                .next()
        } else {
            None
        }
    }

    fn next<I: AnyItem>(&self, key: SubKey<I>) -> Option<SubKey<I>> {
        if TypeId::of::<I>() == TypeId::of::<T>() {
            self.items
                .iter()
                .enumerate()
                .skip(key.as_usize(self.key_len))
                .filter_map(|(i, slot)| {
                    slot.as_ref().map(|_| {
                        SubKey::new(
                            self.key_len,
                            Index(NonZeroU64::new(i as u64).expect("Zero index is allocated")),
                        )
                    })
                })
                .next()
        } else {
            None
        }
    }
}

impl<T: AnyItem> Item for VecContainer<T> {
    type I<'a> = std::iter::Empty<AnyRef>;

    fn references(&self, _: Index) -> Self::I<'_> {
        std::iter::empty()
    }
}

impl<T: AnyItem> AnyItem for VecContainer<T> {
    fn references_any(&self, _: Index) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>> {
        None
    }

    fn remove_reference(&mut self, _: Index, _: AnyKey) -> bool {
        true
    }
}

impl<T: AnyItem> Default for VecContainer<T> {
    fn default() -> Self {
        Self::new(MAX_KEY_LEN)
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
