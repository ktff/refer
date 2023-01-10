use super::*;
use crate::core::{AnyContainer, Container, Key, Result};

pub struct SlotPermit<'a, T: core::AnyItem + ?Sized, R, A, C: ?Sized> {
    permit: TypePermit<'a, T, R, A, C>,
    key: Key<T>,
}

impl<'a, R, T: core::DynItem + ?Sized, A, C: AnyContainer + ?Sized> SlotPermit<'a, T, R, A, C> {
    pub fn new(permit: TypePermit<'a, T, R, A, C>, key: Key<T>) -> Self {
        Self { permit, key }
    }

    pub fn get_dyn(self) -> Result<core::DynSlot<'a, T, R, A>> {
        let Self { permit, key } = self;
        permit
            .get_slot_any(key.upcast())
            // SAFETY: Type level logic of permit ensures that it has sufficient access for 'a to this slot.
            .map(|slot| unsafe { core::DynSlot::new(key, slot, permit.access()) })
            .ok_or_else(|| key.into())
    }
}

impl<'a, R, T: core::Item, A, C: Container<T>> SlotPermit<'a, T, R, A, C> {
    pub fn get(self) -> Result<core::Slot<'a, T, C::Shell, R, A>> {
        let Self { permit, key } = self;
        permit
            .get_slot(key)
            // SAFETY: Type level logic of permit ensures that it has sufficient access for 'a to this slot.
            .map(|slot| unsafe { core::Slot::new(key, slot, permit.access()) })
            .ok_or_else(|| key.into())
    }
}

impl<'a, T: core::AnyItem + ?Sized, A, C: ?Sized> SlotPermit<'a, T, Mut, A, C> {
    pub fn borrow(&self) -> SlotPermit<T, Ref, A, C> {
        SlotPermit {
            permit: self.permit.borrow(),
            key: self.key,
        }
    }

    pub fn borrow_mut(&mut self) -> SlotPermit<T, Mut, A, C> {
        SlotPermit {
            permit: self.permit.borrow_mut(),
            key: self.key,
        }
    }
}

impl<'a, T: core::AnyItem + ?Sized, A, C: ?Sized> Copy for SlotPermit<'a, T, Ref, A, C> {}

impl<'a, T: core::AnyItem + ?Sized, A, C: ?Sized> Clone for SlotPermit<'a, T, Ref, A, C> {
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}
