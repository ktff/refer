use super::permit::{self, ItemAccess, Permit, RefAccess, ShellAccess};
use crate::core::{
    AnyItem, AnyKey, AnyRef, AnyShell, AnySlotContext, AnyUnsafeSlot, Index, Item, KeyPrefix, Shell,
};
use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

pub struct AnySlot<'a, R, A> {
    key: AnyKey,
    slot: AnyUnsafeSlot<'a>,
    access: Permit<R, A>,
}

impl<'a, R, A> AnySlot<'a, R, A> {
    /// SAFETY: Caller must ensure that it has the correct access to the slot for the given 'a.
    pub unsafe fn new(key: AnyKey, slot: AnyUnsafeSlot<'a>, access: Permit<R, A>) -> Self {
        Self { key, slot, access }
    }

    pub fn key(&self) -> AnyKey {
        self.key
    }

    pub fn context(&self) -> AnySlotContext<'a> {
        self.slot.context()
    }
}

impl<'a, R: RefAccess, A: ItemAccess> AnySlot<'a, R, A> {
    pub fn item(&self) -> &dyn AnyItem {
        // SAFETY: We have at least read access to the item. R
        unsafe { &*self.slot.item().get() }
    }

    pub fn item_as<T: Item>(&self) -> Option<&T> {
        (self.item() as &dyn Any).downcast_ref::<T>()
    }

    pub fn iter_references_any(&self) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>> {
        self.item().iter_references_any(self.context())
    }

    /// Can panic if context isn't for this type.
    pub fn duplicate(&self, to: AnySlotContext) -> Option<Box<dyn Any>> {
        let context = self.context();
        self.item().duplicate_any(context, to)
    }
}

impl<'a, A: ItemAccess> AnySlot<'a, permit::Mut, A> {
    pub fn item_mut(&mut self) -> &mut dyn AnyItem {
        // SAFETY: We have mut access to the item.
        unsafe { &mut *self.slot.item().get() }
    }

    pub fn item_mut_as<T: Item>(&mut self) -> Option<&mut T> {
        (self.item_mut() as &mut dyn Any).downcast_mut::<T>()
    }

    pub fn remove_reference(&mut self, other: AnyKey) -> bool {
        let context = self.context();
        self.item_mut().remove_reference_any(context, other)
    }

    pub fn replace_reference(&mut self, other: AnyKey, to: Index) {
        let context = self.context();
        self.item_mut().replace_reference_any(context, other, to);
    }

    pub fn displace_reference(&mut self, other: AnyKey, to: Index) -> Option<KeyPrefix> {
        let context = self.context();
        self.item_mut().displace_reference_any(context, other, to)
    }

    pub fn duplicate_reference(&mut self, other: AnyKey, to: Index) -> Option<KeyPrefix> {
        let context = self.context();
        self.item_mut().duplicate_reference_any(context, other, to)
    }

    pub fn displace(&mut self) {
        let context = self.context();
        self.item_mut().displace_any(context, None);
    }
}

impl<'a, R: RefAccess, A: ShellAccess> AnySlot<'a, R, A> {
    pub fn shell(&self) -> &dyn AnyShell {
        // SAFETY: We have at least read access to the shell. R
        unsafe { &*self.slot.shell().get() }
    }

    pub fn shell_as<S: Shell>(&self) -> Option<&S> {
        (self.shell() as &dyn Any).downcast_ref::<S>()
    }
}

impl<'a, A: ShellAccess> AnySlot<'a, permit::Mut, A> {
    pub fn shell_mut(&mut self) -> &mut dyn AnyShell {
        // SAFETY: We have mut access to the shell.
        unsafe { &mut *self.slot.shell().get() }
    }

    pub fn shell_mut_as<S: Shell>(&mut self) -> Option<&mut S> {
        (self.shell_mut() as &mut dyn Any).downcast_mut::<S>()
    }

    pub fn add_from(&mut self, from: AnyKey) {
        let context = self.context();
        self.shell_mut().add_any(from, context);
    }

    pub fn add_in_shell_many(&mut self, from: AnyKey, count: usize) {
        let context = self.context();
        self.shell_mut().add_many_any(from, count, context);
    }

    pub fn replace_in_shell(&mut self, from: AnyKey, to: Index) {
        let context = self.context();
        self.shell_mut().replace_any(from, to, context);
    }

    pub fn remove_in_shell(&mut self, from: AnyKey) {
        self.shell_mut().remove_any(from);
    }

    pub fn clear_shell(&mut self) {
        let context = self.context();
        self.shell_mut().clear_any(context);
    }
}

impl<'a> AnySlot<'a, permit::Mut, permit::Slot> {
    pub fn split(&mut self) -> (&mut dyn AnyItem, &mut dyn AnyShell) {
        // SAFETY: We have mut access to the item and shell.
        unsafe { (&mut *self.slot.item().get(), &mut *self.slot.shell().get()) }
    }

    pub fn split_slot(
        self,
    ) -> (
        AnySlot<'a, permit::Mut, permit::Item>,
        AnySlot<'a, permit::Mut, permit::Shell>,
    ) {
        let (item_access, shell_access) = self.access.split();

        (
            AnySlot {
                key: self.key,
                slot: self.slot,
                access: item_access,
            },
            AnySlot {
                key: self.key,
                slot: self.slot,
                access: shell_access,
            },
        )
    }
}

impl<'a, R, A> Copy for AnySlot<'a, R, A> where Permit<R, A>: Copy {}

impl<'a, R, A> Clone for AnySlot<'a, R, A>
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

impl<'a, R: RefAccess> Deref for AnySlot<'a, R, permit::Item> {
    type Target = dyn AnyItem;

    fn deref(&self) -> &Self::Target {
        self.item()
    }
}

impl<'a> DerefMut for AnySlot<'a, permit::Mut, permit::Item> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item_mut()
    }
}

impl<'a, R: RefAccess> Deref for AnySlot<'a, R, permit::Slot> {
    type Target = dyn AnyItem;

    fn deref(&self) -> &Self::Target {
        self.item()
    }
}

impl<'a> DerefMut for AnySlot<'a, permit::Mut, permit::Slot> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item_mut()
    }
}

impl<'a, R: RefAccess> Deref for AnySlot<'a, R, permit::Shell> {
    type Target = dyn AnyShell;

    fn deref(&self) -> &Self::Target {
        self.shell()
    }
}

impl<'a> DerefMut for AnySlot<'a, permit::Mut, permit::Shell> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.shell_mut()
    }
}
