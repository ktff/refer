use super::*;
use crate::core::{
    container::RegionContainer, container::TypeContainer, AnyContainer, Container, Key, ReferError,
    Result,
};

pub struct SlotPermit<'a, T: core::DynItem + ?Sized, R, A, C: ?Sized> {
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
            .get_slot_any(key.any())
            .ok_or_else(|| ReferError::invalid_key(key, permit.container_path()))
            // SAFETY: Type level logic of permit ensures that it has sufficient access for 'a to this slot.
            .and_then(|slot| unsafe { core::DynSlot::new(key, slot, permit.access()) })
    }
}

impl<'a, R, T: core::Item, A, C: AnyContainer + ?Sized> SlotPermit<'a, T, R, A, C> {
    pub fn step(self) -> Option<SlotPermit<'a, T, R, A, C::Sub>>
    where
        C: TypeContainer<T>,
    {
        let Self { permit, key } = self;
        permit.step().map(|permit| SlotPermit::new(permit, key))
    }

    pub fn step_into(self) -> Option<SlotPermit<'a, T, R, A, C::Sub>>
    where
        C: RegionContainer,
    {
        let Self { permit, key } = self;
        let index = permit.region().index_of(key);
        permit
            .step_into(index)
            .map(|permit| SlotPermit::new(permit, key))
    }
}

impl<'a, R, T: core::Item, A, C: Container<T> + ?Sized> SlotPermit<'a, T, R, A, C> {
    pub fn get(self) -> Result<core::Slot<'a, T, C::Shell, R, A>> {
        let Self { permit, key } = self;
        permit
            .get_slot(key)
            // SAFETY: Type level logic of permit ensures that it has sufficient access for 'a to this slot.
            .map(|slot| unsafe { core::Slot::new(key, slot, permit.access()) })
            .ok_or_else(|| ReferError::invalid_key(key, permit.container_path()))
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
