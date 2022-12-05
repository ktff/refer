use super::permit::{self, Permit, Split};
use crate::core::{AnyKey, AnySlot, Index, Item, ItemBuilder, ItemContext, Key, Shell, UnsafeSlot};
use std::ops::{Deref, DerefMut};

pub struct Slot<'a, T: Item, S: Shell<T = T>, R, W> {
    key: Key<T>,
    slot: UnsafeSlot<'a, T, T::LocalityData, S, T::Alloc>,
    access: Permit<R, W>,
}

impl<'a, T: Item, S: Shell<T = T>, R, W> Slot<'a, T, S, R, W> {
    /// SAFETY: Caller must ensure that it has the correct access to the slot for the given 'a.
    pub unsafe fn new(
        key: Key<T>,
        slot: UnsafeSlot<'a, T, T::LocalityData, S, T::Alloc>,
        access: Permit<R, W>,
    ) -> Self {
        Self { key, slot, access }
    }

    pub fn key(&self) -> Key<T> {
        self.key
    }

    pub fn alloc(&self) -> &'a T::Alloc {
        self.slot.alloc()
    }

    pub fn group_item(&self) -> &'a T::LocalityData {
        self.slot.group_item()
    }

    pub fn context(&self) -> ItemContext<'a, T> {
        ItemContext::new((self.slot.group_item(), self.slot.alloc()))
    }

    pub fn upcast(self) -> AnySlot<'a, R, W> {
        // SAFETY: We have the same access to the slot.
        unsafe { AnySlot::new(self.key.into(), self.slot.upcast(), self.access) }
    }
}

impl<'a, T: Item, S: Shell<T = T>, R> Slot<'a, T, S, R, permit::Slot> {
    pub fn item(&self) -> &T {
        // SAFETY: We have at least read access to the item. R
        unsafe { &*self.slot.item().get() }
    }

    pub fn shell(&self) -> &S {
        // SAFETY: We have at least read access to the shell. R
        unsafe { &*self.slot.shell().get() }
    }

    pub fn iter_references(&self) -> T::I<'_> {
        self.item().iter_references(self.context())
    }

    pub fn duplicate(&self) -> Option<ItemBuilder> {
        let context = self.context();
        self.item().duplicate(context.upcast())
    }
}

impl<'a, T: Item, S: Shell<T = T>> Slot<'a, T, S, permit::Mut, permit::Slot> {
    pub fn item_mut(&mut self) -> &mut T {
        // SAFETY: We have mut access to the item.
        unsafe { &mut *self.slot.item().get() }
    }

    pub fn shell_mut(&mut self) -> &mut S {
        // SAFETY: We have mut access to the shell.
        unsafe { &mut *self.slot.shell().get() }
    }

    pub fn split_mut(&mut self) -> Split<&mut T, &mut S> {
        // SAFETY: We have mut access to the item and shell.
        unsafe {
            Split {
                items: &mut *self.slot.item().get(),
                shells: &mut *self.slot.shell().get(),
            }
        }
    }

    pub fn split(
        self,
    ) -> Split<Slot<'a, T, S, permit::Mut, permit::Item>, Slot<'a, T, S, permit::Mut, permit::Shell>>
    {
        self.access.split().map(
            |access| Slot {
                key: self.key,
                slot: self.slot,
                access,
            },
            |access| Slot {
                key: self.key,
                slot: self.slot,
                access,
            },
        )
    }

    // pub fn displace(&mut self) -> Option<ItemBuilder> {
    //     let context = self.context();
    //     self.item_mut().displace(context.upcast())
    // }
}

impl<'a, T: Item, S: Shell<T = T>, R> Slot<'a, T, S, R, permit::Item> {
    pub fn item(&self) -> &T {
        // SAFETY: We have at least read access to the item. R
        unsafe { &*self.slot.item().get() }
    }

    pub fn iter_references(&self) -> T::I<'_> {
        self.item().iter_references(self.context())
    }
}

impl<'a, T: Item, S: Shell<T = T>> Slot<'a, T, S, permit::Mut, permit::Item> {
    pub fn item_mut(&mut self) -> &mut T {
        // SAFETY: We have mut access to the item.
        unsafe { &mut *self.slot.item().get() }
    }

    pub fn replace_reference(&mut self, other: AnyKey, to: Index) {
        let context = self.context();
        self.item_mut()
            .replace_reference(context.upcast(), other, to);
    }

    pub fn duplicate_reference(&mut self, other: AnyKey, to: Index) -> bool {
        let context = self.context();
        self.item_mut()
            .duplicate_reference(context.upcast(), other, to)
    }
}

impl<'a, T: Item, S: Shell<T = T>, R> Slot<'a, T, S, R, permit::Shell> {
    pub fn shell(&self) -> &S {
        // SAFETY: We have at least read access to the shell. R
        unsafe { &*self.slot.shell().get() }
    }
}

impl<'a, T: Item, S: Shell<T = T>> Slot<'a, T, S, permit::Mut, permit::Shell> {
    pub fn shell_mut(&mut self) -> &mut S {
        // SAFETY: We have mut access to the shell.
        unsafe { &mut *self.slot.shell().get() }
    }

    pub fn add_from(&mut self, from: AnyKey) {
        let alloc = self.alloc();
        self.shell_mut().add_from(from, alloc);
    }

    pub fn add_from_count(&mut self, from: AnyKey, count: usize) {
        let alloc = self.alloc();
        self.shell_mut().add_from_count(from, count, alloc);
    }

    pub fn replace(&mut self, from: AnyKey, to: Index) {
        let alloc = self.alloc();
        self.shell_mut().replace(from, to, alloc);
    }
}

impl<'a, T: Item, S: Shell<T = T>, W> Slot<'a, T, S, permit::Mut, W> {
    pub fn borrow(&self) -> Slot<T, S, permit::Ref, W> {
        // SAFETY: We have mut access to the item.
        unsafe { Slot::new(self.key, self.slot, self.access.borrow()) }
    }
}

impl<'a, T: Item, S: Shell<T = T>, R, W> Copy for Slot<'a, T, S, R, W> where Permit<R, W>: Copy {}

impl<'a, T: Item, S: Shell<T = T>, R, W> Clone for Slot<'a, T, S, R, W>
where
    Permit<R, W>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            slot: self.slot,
            access: self.access.clone(),
        }
    }
}

impl<'a, T: Item, S: Shell<T = T>, R> Deref for Slot<'a, T, S, R, permit::Item> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item()
    }
}

impl<'a, T: Item, S: Shell<T = T>> DerefMut for Slot<'a, T, S, permit::Mut, permit::Item> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item_mut()
    }
}

impl<'a, T: Item, S: Shell<T = T>, R> Deref for Slot<'a, T, S, R, permit::Shell> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        self.shell()
    }
}

impl<'a, T: Item, S: Shell<T = T>> DerefMut for Slot<'a, T, S, permit::Mut, permit::Shell> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.shell_mut()
    }
}
