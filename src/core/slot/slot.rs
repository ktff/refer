use super::permit::{self, ItemAccess, Permit, RefAccess, ShellAccess};
use crate::core::{
    AnyItemContext, AnyKey, AnySlot, Index, Item, ItemContext, Key, KeyPrefix, Shell, UnsafeSlot,
};
use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

pub struct Slot<'a, T: Item, S: Shell<T = T>, R, A> {
    key: Key<T>,
    slot: UnsafeSlot<'a, T, S>,
    access: Permit<R, A>,
}

impl<'a, T: Item, S: Shell<T = T>, R, A> Slot<'a, T, S, R, A> {
    /// SAFETY: Caller must ensure that it has the correct access to the slot for the given 'a.
    pub unsafe fn new(key: Key<T>, slot: UnsafeSlot<'a, T, S>, access: Permit<R, A>) -> Self {
        Self { key, slot, access }
    }

    pub fn key(&self) -> Key<T> {
        self.key
    }

    pub fn context(&self) -> ItemContext<'a, T> {
        self.slot.context()
    }

    pub fn upcast(self) -> AnySlot<'a, R, A> {
        // SAFETY: We have the same access to the slot.
        unsafe { AnySlot::new(self.key.into(), self.slot.upcast(), self.access) }
    }
}

impl<'a, T: Item, S: Shell<T = T>, R: RefAccess, A: ItemAccess> Slot<'a, T, S, R, A> {
    pub fn item(&self) -> &T {
        // SAFETY: We have at least read access to the item. R
        unsafe { &*self.slot.item().get() }
    }

    pub fn iter_references(&self) -> T::I<'_> {
        self.item().iter_references(self.context())
    }

    /// Can panic if context isn't for this type.
    pub fn duplicate(&self, to: AnyItemContext) -> Option<Box<dyn Any>> {
        let context = self.context();
        self.item().duplicate(context.upcast(), to)
    }
}

impl<'a, T: Item, S: Shell<T = T>, A: ItemAccess> Slot<'a, T, S, permit::Mut, A> {
    pub fn item_mut(&mut self) -> &mut T {
        // SAFETY: We have mut access to the item.
        unsafe { &mut *self.slot.item().get() }
    }

    pub fn replace_reference(&mut self, other: AnyKey, to: Index) {
        let context = self.context();
        self.item_mut()
            .replace_reference(context.upcast(), other, to);
    }

    pub fn displace_reference(&mut self, other: AnyKey, to: Index) -> Option<KeyPrefix> {
        let context = self.context();
        self.item_mut()
            .displace_reference(context.upcast(), other, to)
    }

    pub fn duplicate_reference(&mut self, other: AnyKey, to: Index) -> Option<KeyPrefix> {
        let context = self.context();
        self.item_mut()
            .duplicate_reference(context.upcast(), other, to)
    }
}

impl<'a, T: Item, S: Shell<T = T>, R: RefAccess, A: ShellAccess> Slot<'a, T, S, R, A> {
    pub fn shell(&self) -> &S {
        // SAFETY: We have at least read access to the shell. R
        unsafe { &*self.slot.shell().get() }
    }
}

impl<'a, T: Item, S: Shell<T = T>, A: ShellAccess> Slot<'a, T, S, permit::Mut, A> {
    pub fn shell_mut(&mut self) -> &mut S {
        // SAFETY: We have mut access to the shell.
        unsafe { &mut *self.slot.shell().get() }
    }

    pub fn add_from(&mut self, from: AnyKey) {
        let alloc = self.slot.allocator();
        self.shell_mut().add_from(from, alloc);
    }

    pub fn add_from_count(&mut self, from: AnyKey, count: usize) {
        let alloc = self.slot.allocator();
        self.shell_mut().add_from_count(from, count, alloc);
    }

    pub fn replace(&mut self, from: AnyKey, to: Index) {
        let alloc = self.slot.allocator();
        self.shell_mut().replace(from, to, alloc);
    }
}

impl<'a, T: Item, S: Shell<T = T>> Slot<'a, T, S, permit::Mut, permit::Slot> {
    pub fn split(&mut self) -> (&mut T, &mut S) {
        // SAFETY: We have mut access to the item and shell.
        unsafe { (&mut *self.slot.item().get(), &mut *self.slot.shell().get()) }
    }

    pub fn split_slot(
        self,
    ) -> (
        Slot<'a, T, S, permit::Mut, permit::Item>,
        Slot<'a, T, S, permit::Mut, permit::Shell>,
    ) {
        let (item_access, shell_access) = self.access.split();

        (
            Slot {
                key: self.key,
                slot: self.slot,
                access: item_access,
            },
            Slot {
                key: self.key,
                slot: self.slot,
                access: shell_access,
            },
        )
    }
}

impl<'a, T: Item, S: Shell<T = T>, A> Slot<'a, T, S, permit::Mut, A> {
    pub fn borrow(&self) -> Slot<T, S, permit::Ref, A> {
        // SAFETY: We have mut access to the item.
        unsafe { Slot::new(self.key, self.slot, self.access.borrow()) }
    }
}

impl<'a, T: Item, S: Shell<T = T>, R, A> Copy for Slot<'a, T, S, R, A> where Permit<R, A>: Copy {}

impl<'a, T: Item, S: Shell<T = T>, R, A> Clone for Slot<'a, T, S, R, A>
where
    Permit<R, A>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            slot: self.slot,
            access: self.access.clone(),
        }
    }
}

impl<'a, T: Item, S: Shell<T = T>, R: RefAccess> Deref for Slot<'a, T, S, R, permit::Item> {
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

impl<'a, T: Item, S: Shell<T = T>, R: RefAccess> Deref for Slot<'a, T, S, R, permit::Slot> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item()
    }
}

impl<'a, T: Item, S: Shell<T = T>> DerefMut for Slot<'a, T, S, permit::Mut, permit::Slot> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item_mut()
    }
}

impl<'a, T: Item, S: Shell<T = T>, R: RefAccess> Deref for Slot<'a, T, S, R, permit::Shell> {
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
