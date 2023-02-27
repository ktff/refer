use crate::core::{
    permit, AnyItem, AnyRef, AnyShell, AnySlotLocality, AnyUnsafeSlot, DynItem, Item, Key, Path,
    Permit, ReferError, Shell, TypeInfo,
};

use std::{
    any::{Any, TypeId},
    marker::Unsize,
    ops::{Deref, DerefMut},
};

pub type AnySlot<'a, R, A> = DynSlot<'a, dyn AnyItem, R, A>;

pub struct DynSlot<'a, T: DynItem + ?Sized, R, A> {
    key: Key<T>,
    metadata: T::Metadata,
    slot: AnyUnsafeSlot<'a>,
    access: Permit<R, A>,
}

impl<'a, R, A> AnySlot<'a, R, A> {
    /// Key should correspond to the slot.
    /// SAFETY: Caller must ensure that it has the correct access to the slot for the given 'a.    
    pub unsafe fn new_any(
        key: Key<dyn AnyItem>,
        slot: AnyUnsafeSlot<'a>,
        access: Permit<R, A>,
    ) -> Self {
        debug_assert!(slot.prefix().contains_index(key.index()));
        let metadata = std::ptr::metadata(slot.item().get());

        Self {
            key,
            metadata,
            slot,
            access,
        }
    }
}

impl<'a, T: DynItem + ?Sized, R, A> DynSlot<'a, T, R, A> {
    /// Key should correspond to the slot.
    /// Err if item doesn't implement T.
    /// SAFETY: Caller must ensure that it has the correct access to the slot for the given 'a.    
    pub unsafe fn new(
        key: Key<T>,
        slot: AnyUnsafeSlot<'a>,
        access: Permit<R, A>,
    ) -> Result<Self, ReferError> {
        debug_assert!(slot.prefix().contains_index(key.index()));
        let metadata = slot
            .metadata::<T>()
            .ok_or_else(|| ReferError::InvalidCastType {
                expected: TypeInfo::of::<T>(),
                found: slot.item().get().type_info(),
                index: key.index(),
                container: slot.prefix(),
            })?;

        Ok(Self {
            key,
            metadata,
            slot,
            access,
        })
    }

    pub fn key(&self) -> Key<T> {
        self.key
    }

    pub fn item_type_id(&self) -> std::any::TypeId {
        self.slot.item_type_id()
    }

    pub fn locality(&self) -> AnySlotLocality<'a> {
        self.slot.locality()
    }

    pub fn upcast<U: DynItem + ?Sized>(self) -> DynSlot<'a, U, R, A>
    where
        T: Unsize<U>,
    {
        // Upcast metadata
        let metadata = {
            let ptr = std::ptr::from_raw_parts::<T>(std::ptr::null(), self.metadata) as *const U;
            std::ptr::metadata(ptr)
        };

        DynSlot {
            key: self.key.upcast(),
            metadata,
            slot: self.slot,
            access: self.access,
        }
    }

    pub fn sidecast<U: DynItem + ?Sized>(self) -> Result<DynSlot<'a, U, R, A>, Self> {
        if let Some(metadata) = self.slot.metadata::<U>() {
            Ok(DynSlot {
                key: self.key().any().assume(),
                metadata,
                slot: self.slot,
                access: self.access,
            })
        } else {
            Err(self)
        }
    }

    pub fn downcast<U: Item>(self) -> Result<DynSlot<'a, U, R, A>, Self> {
        if TypeId::of::<U>() == self.item_type_id() {
            Ok(DynSlot {
                key: Key::new(self.key.index()),
                metadata: (),
                slot: self.slot,
                access: self.access,
            })
        } else {
            Err(self)
        }
    }

    pub fn downgrade<F, B>(self) -> DynSlot<'a, T, F, B>
    where
        Permit<R, A>: Into<Permit<F, B>>,
    {
        DynSlot {
            key: self.key,
            metadata: self.metadata,
            slot: self.slot,
            access: self.access.into(),
        }
    }
}

impl<'a, T: DynItem + ?Sized, R: Into<permit::Ref>, A: Into<permit::Item>> DynSlot<'a, T, R, A> {
    pub fn any_item(&self) -> &dyn AnyItem {
        unsafe {
            let ptr = self.slot.item().get();

            // SAFETY: We have at least read access to the item. R
            &*ptr
        }
    }

    pub fn item(&self) -> &T {
        unsafe {
            let ptr = self.slot.item().get();

            // SAFETY: During construction we checked that the type of the item matches the type of the key.
            let ptr = std::ptr::from_raw_parts(ptr as *const (), self.metadata);

            // SAFETY: We have at least read access to the item. R
            &*ptr
        }
    }

    pub fn item_downcast<U: Item>(&self) -> Option<&U> {
        (self.any_item() as &dyn Any).downcast_ref::<U>()
    }

    pub fn iter_references_any(&self) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>> {
        self.any_item().iter_references_any(self.locality())
    }

    /// Can panic if locality isn't for this type.
    pub fn duplicate(&self, to: AnySlotLocality) -> Option<Box<dyn AnyItem>> {
        let locality = self.locality();
        self.any_item().duplicate_any(locality, to)
    }
}

impl<'a, T: DynItem + ?Sized, A: Into<permit::Item>> DynSlot<'a, T, permit::Mut, A> {
    pub fn any_item_mut(&mut self) -> &mut dyn AnyItem {
        unsafe {
            let ptr = self.slot.item().get();

            // SAFETY: We have mut access to the item.
            &mut *ptr
        }
    }

    pub fn item_mut(&mut self) -> &mut T {
        unsafe {
            let ptr = self.slot.item().get();

            // SAFETY: During construction we checked that the type of the item matches the type of the key.
            let ptr = std::ptr::from_raw_parts_mut(ptr as *mut (), self.metadata);

            // SAFETY: We have mut access to the item.
            &mut *ptr
        }
    }

    pub fn item_mut_downcast<U: Item>(&mut self) -> Option<&mut U> {
        (self.any_item_mut() as &mut dyn Any).downcast_mut::<U>()
    }

    pub fn remove_reference<F: DynItem + ?Sized>(&mut self, other: Key<F>) -> bool {
        let locality = self.locality();
        self.any_item_mut()
            .remove_reference_any(locality, other.any())
    }

    pub fn replace_reference<F: DynItem + ?Sized>(&mut self, other: Key<F>, to: Key<F>) {
        let locality = self.locality();
        self.any_item_mut()
            .replace_reference_any(locality, other.any(), to.any());
    }

    pub fn displace_reference<F: DynItem + ?Sized>(
        &mut self,
        other: Key<F>,
        to: Key<F>,
    ) -> Option<Path> {
        let locality = self.locality();
        self.any_item_mut()
            .displace_reference_any(locality, other.any(), to.any())
    }

    pub fn duplicate_reference<F: DynItem + ?Sized>(
        &mut self,
        other: Key<F>,
        to: Key<F>,
    ) -> Option<Path> {
        let locality = self.locality();
        self.any_item_mut()
            .duplicate_reference_any(locality, other.any(), to.any())
    }

    pub fn displace(&mut self) {
        let locality = self.locality();
        self.any_item_mut().displace_any(locality, None);
    }
}

impl<'a, T: DynItem + ?Sized, R: Into<permit::Ref>, A: Into<permit::Shell>> DynSlot<'a, T, R, A> {
    pub fn shell(&self) -> &dyn AnyShell {
        // SAFETY: We have at least read access to the shell. R
        unsafe { &*self.slot.shell().get() }
    }

    pub fn shell_downcast<S: Shell>(&self) -> Option<&S> {
        (self.shell() as &dyn Any).downcast_ref::<S>()
    }
}

impl<'a, T: DynItem + ?Sized, A: Into<permit::Shell>> DynSlot<'a, T, permit::Mut, A> {
    pub fn shell_mut(&mut self) -> &mut dyn AnyShell {
        // SAFETY: We have mut access to the shell.
        unsafe { &mut *self.slot.shell().get() }
    }

    pub fn shell_mut_downcast<S: Shell>(&mut self) -> Option<&mut S> {
        (self.shell_mut() as &mut dyn Any).downcast_mut::<S>()
    }

    pub fn shell_add<F: DynItem + ?Sized>(&mut self, from: Key<F>) {
        let locality = self.locality();
        self.shell_mut().add_any(from.any(), locality);
    }

    pub fn shell_add_many<F: DynItem + ?Sized>(&mut self, from: Key<F>, count: usize) {
        let locality = self.locality();
        self.shell_mut().add_many_any(from.any(), count, locality);
    }

    pub fn shell_replace<F: DynItem + ?Sized>(&mut self, from: Key<F>, to: Key<F>) {
        let locality = self.locality();
        self.shell_mut().replace_any(from.any(), to.any(), locality);
    }

    pub fn shell_remove<F: DynItem + ?Sized>(&mut self, from: Key<F>) {
        self.shell_mut().remove_any(from.any());
    }

    pub fn shell_clear(&mut self) {
        let locality = self.locality();
        self.shell_mut().clear_any(locality);
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
                metadata: self.metadata,
                slot: self.slot,
                access: item_access,
            },
            DynSlot {
                key: self.key,
                metadata: self.metadata,
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
            metadata: self.metadata,
            slot: self.slot,
            access: self.access.clone(),
        }
    }
}

impl<'a, T: DynItem + ?Sized, R: Into<permit::Ref>> Deref for DynSlot<'a, T, R, permit::Item> {
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

impl<'a, T: DynItem + ?Sized, R: Into<permit::Ref>> Deref for DynSlot<'a, T, R, permit::Slot> {
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

impl<'a, T: DynItem + ?Sized, R: Into<permit::Ref>> Deref for DynSlot<'a, T, R, permit::Shell> {
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
