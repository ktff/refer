use super::permit::{self, ItemAccess, Permit, RefAccess, ShellAccess};
use crate::core::{
    AnyItem, AnyKey, AnyRef, AnyShell, AnySlotContext, AnyUnsafeSlot, DynItem, Index, Item, Key,
    Path, Shell,
};
use std::{
    any::Any,
    marker::Unsize,
    ops::{Deref, DerefMut},
};

pub type AnySlot<'a, R, A> = DynSlot<'a, dyn AnyItem, R, A>;

pub struct DynSlot<'a, T: DynItem + ?Sized, R, A> {
    key: Key<T>,
    slot: AnyUnsafeSlot<'a>,
    access: Permit<R, A>,
}

impl<'a, T: DynItem + ?Sized, R, A> DynSlot<'a, T, R, A> {
    /// Key should correspond to the slot.
    /// SAFETY: Caller must ensure that it has the correct access to the slot for the given 'a.    
    pub unsafe fn new(key: Key<T>, slot: AnyUnsafeSlot<'a>, access: Permit<R, A>) -> Self {
        assert_eq!(key.type_id(), slot.item().get().item_type_id());
        debug_assert!(slot.prefix().start_of_key(key.upcast()));
        Self { key, slot, access }
    }

    pub fn key(&self) -> Key<T> {
        self.key
    }

    pub fn context(&self) -> AnySlotContext<'a> {
        self.slot.context()
    }

    pub fn upcast<U: DynItem + ?Sized>(self) -> DynSlot<'a, U, R, A>
    where
        T: Unsize<U>,
    {
        DynSlot {
            key: self.key.upcast(),
            slot: self.slot,
            access: self.access,
        }
    }

    pub fn downcast<U: Item>(self) -> Result<DynSlot<'a, U, R, A>, Self> {
        if let Some(key) = self.key.downcast::<U>() {
            Ok(DynSlot {
                key,
                slot: self.slot,
                access: self.access,
            })
        } else {
            Err(self)
        }
    }
}

impl<'a, T: DynItem + ?Sized, R: RefAccess, A: ItemAccess> DynSlot<'a, T, R, A> {
    pub fn item(&self) -> &T {
        unsafe {
            let ptr = self.slot.item().get();

            // SAFETY: During construction we checked that the type of the item matches the type of the key.
            let ptr = std::ptr::from_raw_parts(ptr as *const (), self.key.metadata());

            // SAFETY: We have at least read access to the item. R
            &*ptr
        }
    }

    pub fn item_downcast<U: Item>(&self) -> Option<&U> {
        (self.item() as &dyn Any).downcast_ref::<U>()
    }

    pub fn iter_references_any(&self) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>> {
        self.item().iter_references_any(self.context())
    }

    /// Can panic if context isn't for this type.
    pub fn duplicate(&self, to: AnySlotContext) -> Option<Box<dyn AnyItem>> {
        let context = self.context();
        self.item().duplicate_any(context, to)
    }
}

impl<'a, T: DynItem + ?Sized, A: ItemAccess> DynSlot<'a, T, permit::Mut, A> {
    pub fn item_mut(&mut self) -> &mut T {
        unsafe {
            let ptr = self.slot.item().get();

            // SAFETY: During construction we checked that the type of the item matches the type of the key.
            let ptr = std::ptr::from_raw_parts_mut(ptr as *mut (), self.key.metadata());

            // SAFETY: We have mut access to the item.
            &mut *ptr
        }
    }

    pub fn item_mut_downcast<U: Item>(&mut self) -> Option<&mut U> {
        (self.item_mut() as &mut dyn Any).downcast_mut::<U>()
    }

    pub fn remove_reference(&mut self, other: AnyKey) -> bool {
        let context = self.context();
        self.item_mut().remove_reference_any(context, other)
    }

    pub fn replace_reference(&mut self, other: AnyKey, to: Index) {
        let context = self.context();
        self.item_mut().replace_reference_any(context, other, to);
    }

    pub fn displace_reference(&mut self, other: AnyKey, to: Index) -> Option<Path> {
        let context = self.context();
        self.item_mut().displace_reference_any(context, other, to)
    }

    pub fn duplicate_reference(&mut self, other: AnyKey, to: Index) -> Option<Path> {
        let context = self.context();
        self.item_mut().duplicate_reference_any(context, other, to)
    }

    pub fn displace(&mut self) {
        let context = self.context();
        self.item_mut().displace_any(context, None);
    }
}

impl<'a, T: DynItem + ?Sized, R: RefAccess, A: ShellAccess> DynSlot<'a, T, R, A> {
    pub fn shell(&self) -> &dyn AnyShell {
        // SAFETY: We have at least read access to the shell. R
        unsafe { &*self.slot.shell().get() }
    }

    pub fn shell_downcast<S: Shell>(&self) -> Option<&S> {
        (self.shell() as &dyn Any).downcast_ref::<S>()
    }
}

impl<'a, T: DynItem + ?Sized, A: ShellAccess> DynSlot<'a, T, permit::Mut, A> {
    pub fn shell_mut(&mut self) -> &mut dyn AnyShell {
        // SAFETY: We have mut access to the shell.
        unsafe { &mut *self.slot.shell().get() }
    }

    pub fn shell_mut_downcast<S: Shell>(&mut self) -> Option<&mut S> {
        (self.shell_mut() as &mut dyn Any).downcast_mut::<S>()
    }

    pub fn shell_add(&mut self, from: AnyKey) {
        let context = self.context();
        self.shell_mut().add_any(from, context);
    }

    pub fn shell_add_many(&mut self, from: AnyKey, count: usize) {
        let context = self.context();
        self.shell_mut().add_many_any(from, count, context);
    }

    pub fn shell_replace(&mut self, from: AnyKey, to: Index) {
        let context = self.context();
        self.shell_mut().replace_any(from, to, context);
    }

    pub fn shell_remove(&mut self, from: AnyKey) {
        self.shell_mut().remove_any(from);
    }

    pub fn shell_clear(&mut self) {
        let context = self.context();
        self.shell_mut().clear_any(context);
    }
}

impl<'a, T: DynItem + ?Sized> DynSlot<'a, T, permit::Mut, permit::Slot> {
    pub fn split(&mut self) -> (&mut dyn AnyItem, &mut dyn AnyShell) {
        // SAFETY: We have mut access to the item and shell.
        unsafe { (&mut *self.slot.item().get(), &mut *self.slot.shell().get()) }
    }

    pub fn split_slot(
        self,
    ) -> (
        DynSlot<'a, T, permit::Mut, permit::Item>,
        DynSlot<'a, T, permit::Mut, permit::Shell>,
    ) {
        let (item_access, shell_access) = self.access.split();

        (
            DynSlot {
                key: self.key,
                slot: self.slot,
                access: item_access,
            },
            DynSlot {
                key: self.key,
                slot: self.slot,
                access: shell_access,
            },
        )
    }
}

impl<'a, T: DynItem + ?Sized, R, A> Copy for DynSlot<'a, T, R, A> where Permit<R, A>: Copy {}

impl<'a, T: DynItem + ?Sized, R, A> Clone for DynSlot<'a, T, R, A>
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

impl<'a, T: DynItem + ?Sized, R: RefAccess> Deref for DynSlot<'a, T, R, permit::Item> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item()
    }
}

impl<'a, T: DynItem + ?Sized> DerefMut for DynSlot<'a, T, permit::Mut, permit::Item> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item_mut()
    }
}

impl<'a, T: DynItem + ?Sized, R: RefAccess> Deref for DynSlot<'a, T, R, permit::Slot> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item()
    }
}

impl<'a, T: DynItem + ?Sized> DerefMut for DynSlot<'a, T, permit::Mut, permit::Slot> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item_mut()
    }
}

impl<'a, T: DynItem + ?Sized, R: RefAccess> Deref for DynSlot<'a, T, R, permit::Shell> {
    type Target = dyn AnyShell;

    fn deref(&self) -> &Self::Target {
        self.shell()
    }
}

impl<'a, T: DynItem + ?Sized> DerefMut for DynSlot<'a, T, permit::Mut, permit::Shell> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.shell_mut()
    }
}
