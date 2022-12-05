use super::permit::{self, ItemAccess, Permit, RefAccess, ShellAccess};
use crate::core::{
    AnyItem, AnyItemContext, AnyKey, AnyRef, AnyShell, AnyUnsafeSlot, Index, KeyPrefix,
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

    pub fn alloc(&self) -> &'a dyn std::alloc::Allocator {
        self.slot.alloc()
    }

    pub fn group_item(&self) -> &'a dyn Any {
        self.slot.group_item()
    }

    pub fn context(&self) -> AnyItemContext<'a> {
        AnyItemContext::new(
            self.key.type_id(),
            self.slot.group_item(),
            self.slot.alloc(),
            self.slot.alloc_any(),
        )
    }

    // pub fn downcast<T: AnyItem, S: Shell<T = T>>(
    //     self,
    // ) -> Result<Slot<'a, T, S, R, W>, Self> {
    //     if let Some(key) = self.key.downcast() {
    //         if let Some(slot) = self.slot.downcast() {
    //             Ok(Slot {
    //                 key,
    //                 slot,
    //                 access: self.access,
    //             })
    //         } else {
    //             Err(self)
    //         }
    //     } else {
    //         Err(self)
    //     }
    // }
}

impl<'a, R: RefAccess, A: ItemAccess> AnySlot<'a, R, A> {
    pub fn item(&self) -> &dyn AnyItem {
        // SAFETY: We have at least read access to the item. R
        unsafe { &*self.slot.item().get() }
    }

    pub fn iter_references_any(&self) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>> {
        self.item().iter_references_any(self.context())
    }

    /// Can panic if context isn't for this type.
    pub fn duplicate(&self, to: AnyItemContext) -> Option<Box<dyn Any>> {
        let context = self.context();
        self.item().duplicate(context, to)
    }
}

impl<'a, A: ItemAccess> AnySlot<'a, permit::Mut, A> {
    pub fn item_mut(&mut self) -> &mut dyn AnyItem {
        // SAFETY: We have mut access to the item.
        unsafe { &mut *self.slot.item().get() }
    }

    pub fn replace_reference(&mut self, other: AnyKey, to: Index) {
        let context = self.context();
        self.item_mut().replace_reference(context, other, to);
    }

    pub fn displace_reference(&mut self, other: AnyKey, to: Index) -> Option<KeyPrefix> {
        let context = self.context();
        self.item_mut().displace_reference(context, other, to)
    }

    pub fn duplicate_reference(&mut self, other: AnyKey, to: Index) -> Option<KeyPrefix> {
        let context = self.context();
        self.item_mut().duplicate_reference(context, other, to)
    }
}

impl<'a, R: RefAccess, A: ShellAccess> AnySlot<'a, R, A> {
    pub fn shell(&self) -> &dyn AnyShell {
        // SAFETY: We have at least read access to the shell. R
        unsafe { &*self.slot.shell().get() }
    }
}

impl<'a, A: ShellAccess> AnySlot<'a, permit::Mut, A> {
    pub fn shell_mut(&mut self) -> &mut dyn AnyShell {
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
