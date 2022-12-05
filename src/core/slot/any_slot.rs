use super::permit::{self, Permit, Split};
use crate::core::{
    AnyItem, AnyItemContext, AnyKey, AnyRef, AnyShell, AnyUnsafeSlot, Index, ItemBuilder,
};
use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

pub struct AnySlot<'a, R, W> {
    key: AnyKey,
    slot: AnyUnsafeSlot<'a>,
    access: Permit<R, W>,
}

impl<'a, R, W> AnySlot<'a, R, W> {
    /// SAFETY: Caller must ensure that it has the correct access to the slot for the given 'a.
    pub unsafe fn new(key: AnyKey, slot: AnyUnsafeSlot<'a>, access: Permit<R, W>) -> Self {
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

impl<'a, R> AnySlot<'a, R, permit::Slot> {
    pub fn item(&self) -> &dyn AnyItem {
        // SAFETY: We have at least read access to the item. R
        unsafe { &*self.slot.item().get() }
    }

    pub fn iter_references_any(&self) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>> {
        self.item().iter_references_any(self.context())
    }

    pub fn duplicate(&self) -> Option<ItemBuilder> {
        let context = self.context();
        self.item().duplicate(context)
    }

    pub fn shell(&self) -> &dyn AnyShell {
        // SAFETY: We have at least read access to the shell. R
        unsafe { &*self.slot.shell().get() }
    }
}

impl<'a> AnySlot<'a, permit::Mut, permit::Slot> {
    pub fn item_mut(&mut self) -> &mut dyn AnyItem {
        // SAFETY: We have mut access to the item.
        unsafe { &mut *self.slot.item().get() }
    }

    pub fn shell_mut(&mut self) -> &mut dyn AnyShell {
        // SAFETY: We have mut access to the shell.
        unsafe { &mut *self.slot.shell().get() }
    }

    pub fn split_mut(&mut self) -> Split<&mut dyn AnyItem, &mut dyn AnyShell> {
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
    ) -> Split<AnySlot<'a, permit::Mut, permit::Item>, AnySlot<'a, permit::Mut, permit::Shell>>
    {
        self.access.split().map(
            |access| AnySlot {
                key: self.key,
                slot: self.slot,
                access,
            },
            |access| AnySlot {
                key: self.key,
                slot: self.slot,
                access,
            },
        )
    }

    pub fn replace_reference(&mut self, other: AnyKey, to: Index) {
        let context = self.context();
        self.item_mut().replace_reference(context, other, to);
    }
}

impl<'a, R> AnySlot<'a, R, permit::Item> {
    pub fn item(&self) -> &dyn AnyItem {
        // SAFETY: We have at least read access to the item. R
        unsafe { &*self.slot.item().get() }
    }

    pub fn iter_references_any(&self) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>> {
        self.item().iter_references_any(self.context())
    }

    pub fn duplicate(&self) -> Option<ItemBuilder> {
        let context = self.context();
        self.item().duplicate(context)
    }
}

impl<'a> AnySlot<'a, permit::Mut, permit::Item> {
    pub fn item_mut(&mut self) -> &mut dyn AnyItem {
        // SAFETY: We have mut access to the item.
        unsafe { &mut *self.slot.item().get() }
    }

    pub fn replace_reference(&mut self, other: AnyKey, to: Index) {
        let context = self.context();
        self.item_mut().replace_reference(context, other, to);
    }

    pub fn duplicate_reference(&mut self, other: AnyKey, to: Index) -> bool {
        let context = self.context();
        self.item_mut().duplicate_reference(context, other, to)
    }
}

impl<'a, R> AnySlot<'a, R, permit::Shell> {
    pub fn shell(&self) -> &dyn AnyShell {
        // SAFETY: We have at least read access to the shell. R
        unsafe { &*self.slot.shell().get() }
    }
}

impl<'a> AnySlot<'a, permit::Mut, permit::Shell> {
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

impl<'a, R, S> Copy for AnySlot<'a, R, S> where Permit<R, S>: Copy {}

impl<'a, R, S> Clone for AnySlot<'a, R, S>
where
    Permit<R, S>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            slot: self.slot,
            access: self.access.clone(),
        }
    }
}

impl<'a, R> Deref for AnySlot<'a, R, permit::Item> {
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

impl<'a, R> Deref for AnySlot<'a, R, permit::Shell> {
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
