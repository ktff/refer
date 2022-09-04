use crate::core::*;
use std::{any::TypeId, cell::UnsafeCell, collections::HashSet, num::NonZeroU64};

use super::item::{SizedShell, Slot};

pub type SlotIter<'a, T: AnyItem> =
    impl Iterator<Item = (SubKey<T>, &'a UnsafeCell<T>, &'a UnsafeCell<SizedShell<T>>)>;

pub struct VecContainerFamily;

impl SizedContainerFamily for VecContainerFamily {
    type C<T: AnyItem> = VecContainer<T>;

    fn new<T: AnyItem>(key_len: u32) -> Self::C<T> {
        VecContainer::new(key_len)
    }
}

/// A simple vec container of items of the same type.
pub struct VecContainer<T: 'static> {
    slots: Vec<Slot<T>>,
    free: Vec<Index>,
    key_len: u32,
}

impl<T: 'static> VecContainer<T> {
    pub fn new(key_len: u32) -> Self {
        Self {
            slots: vec![Slot::Free],
            free: Vec::new(),
            key_len: key_len,
        }
    }

    fn first_from(&self, i: usize) -> Option<SubKey<T>> {
        self.slots
            .iter()
            .enumerate()
            .skip(i)
            .filter_map(|(i, slot)| match slot {
                Slot::Free | Slot::Reserved => None,
                Slot::Filled { .. } => Some(SubKey::new(
                    self.key_len,
                    Index(NonZeroU64::new(i as u64).expect("Zero index is allocated")),
                )),
            })
            .next()
    }
}

impl<T: 'static> Allocator<T> for VecContainer<T> {
    fn reserve(&mut self, _: &T) -> Option<ReservedKey<T>> {
        let index = if let Some(index) = self.free.pop() {
            debug_assert!(matches!(self.slots[index.as_usize()], Slot::Free));
            index
        } else {
            if self.slots.len() >= (1 << self.key_len) {
                // Out of keys
                return None;
            }

            let index = Index(NonZeroU64::new(self.slots.len() as u64).expect("Zero index"));
            self.slots.push(Slot::Free);
            index
        };

        self.slots[index.as_usize()].reserve();
        Some(ReservedKey::new(SubKey::new(self.key_len, index)))
    }

    fn cancel(&mut self, key: ReservedKey<T>) {
        let index = key.take().index(self.key_len);
        self.slots[index.as_usize()].cancel();
        self.free.push(index);
    }

    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T> {
        let key = key.take();
        self.slots[key.index(self.key_len).as_usize()].fulfill(item);

        key
    }

    /// Frees and returns item if it exists
    fn unfill(&mut self, key: SubKey<T>) -> Option<T>
    where
        T: Sized,
    {
        let index = key.index(self.key_len);

        self.slots[index.as_usize()].unfill().map(|item| {
            self.free.push(index);
            item
        })
    }
}

impl<T: AnyItem> !Sync for VecContainer<T> {}

impl<T: AnyItem> Container<T> for VecContainer<T> {
    type Shell = SizedShell<T>;

    type SlotIter<'a> = SlotIter<'a, T> where Self: 'a;

    fn get_slot(&self, key: SubKey<T>) -> Option<(&UnsafeCell<T>, &UnsafeCell<Self::Shell>)> {
        let i = key.as_usize(self.key_len);
        let slot = self.slots.get(i)?;
        match slot {
            Slot::Free | Slot::Reserved => None,
            Slot::Filled { item, shell } => Some((item, shell)),
        }
    }

    unsafe fn iter_slot(&self) -> Option<Self::SlotIter<'_>> {
        // This is safe since Vec::iter guarantees that each element
        // is returned only once.
        Some(
            self.slots
                .iter()
                .enumerate()
                .filter_map(|(i, slot)| match slot {
                    Slot::Free | Slot::Reserved => None,
                    Slot::Filled { item, shell } => Some((
                        SubKey::new(
                            self.key_len,
                            Index(NonZeroU64::new(i as u64).expect("Zero index")),
                        ),
                        item,
                        shell,
                    )),
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

        let slot = self.slots.get(i)?;
        match slot {
            Slot::Free | Slot::Reserved => None,
            Slot::Filled { item, shell } => Some((item, shell)),
        }
    }

    fn any_unfill(&mut self, key: AnySubKey) -> bool {
        if let Some(key) = key.downcast() {
            self.unfill(key).is_some()
        } else {
            false
        }
    }

    fn first(&self, key: TypeId) -> Option<AnySubKey> {
        if key == TypeId::of::<T>() {
            self.first_from(0).map(|key| key.into())
        } else {
            None
        }
    }

    fn next(&self, key: AnySubKey) -> Option<AnySubKey> {
        if let Some(key) = key.downcast::<T>() {
            self.first_from(key.as_usize(self.key_len))
                .map(|key| key.into())
        } else {
            None
        }
    }

    /// All types in the container.
    fn types(&self) -> HashSet<TypeId> {
        let mut set = HashSet::new();
        set.insert(TypeId::of::<T>());
        set
    }
}
