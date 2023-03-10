use super::*;
use crate::core::{
    container::RegionContainer, container::TypeContainer, AnyContainer, Container, Key, Ptr,
};

pub struct SlotPermit<'a, R, K, T: core::DynItem + ?Sized, C: ?Sized> {
    permit: TypePermit<'a, T, R, C>,
    key: Key<K, T>,
}

impl<'a, R, K, T: core::DynItem + ?Sized, C: AnyContainer + ?Sized> SlotPermit<'a, R, K, T, C> {
    pub fn new(permit: TypePermit<'a, T, R, C>, key: Key<K, T>) -> Self {
        Self { permit, key }
    }
}

impl<'a, R, T: core::DynItem + ?Sized, C: AnyContainer + ?Sized> SlotPermit<'a, R, Ptr, T, C> {
    /// None if doesn't exist.
    pub fn get_dyn(self) -> Option<core::DynSlot<'a, T, R>> {
        let Self { permit, key } = self;
        permit
            .get_slot_any(key.any())
            // SAFETY: Type level logic of permit ensures that it has sufficient access for 'a to this slot.
            .and_then(|slot| unsafe { core::DynSlot::new(slot, permit.access()) })
    }
}

impl<'a, R, T: core::DynItem + ?Sized, C: AnyContainer + ?Sized>
    SlotPermit<'a, R, core::Ref<'a>, T, C>
{
    pub fn get_dyn(self) -> core::DynSlot<'a, T, R> {
        SlotPermit::new(self.permit, self.key.ptr())
            .get_dyn()
            .expect("Reference is invalid for given container.")
    }
}

impl<'a, R, T: core::Item, C: Container<T> + ?Sized> SlotPermit<'a, R, Ptr, T, C> {
    pub fn get(self) -> Option<core::Slot<'a, T, R>> {
        let Self { permit, key } = self;
        permit
            .get_slot(key)
            // SAFETY: Type level logic of permit ensures that it has sufficient access for 'a to this slot.
            .map(|slot| unsafe { core::Slot::new(slot, permit.access()) })
    }
}

impl<'a, R, T: core::Item, C: Container<T> + ?Sized> SlotPermit<'a, R, core::Ref<'a>, T, C> {
    pub fn get(self) -> core::Slot<'a, T, R> {
        SlotPermit::new(self.permit, self.key.ptr())
            .get()
            .expect("Reference is invalid for given container.")
    }
}

impl<'a, R, K: Copy, T: core::Item, C: AnyContainer + ?Sized> SlotPermit<'a, R, K, T, C> {
    pub fn step(self) -> Option<SlotPermit<'a, R, K, T, C::Sub>>
    where
        C: TypeContainer<T>,
    {
        let Self { permit, key } = self;
        permit.step().map(|permit| SlotPermit::new(permit, key))
    }

    pub fn step_into(self) -> Option<SlotPermit<'a, R, K, T, C::Sub>>
    where
        C: RegionContainer,
    {
        let Self { permit, key } = self;
        let index = permit.region().index_of(key.ptr());
        permit
            .step_into(index)
            .map(|permit| SlotPermit::new(permit, key))
    }
}

impl<'a, T: core::AnyItem + ?Sized, C: ?Sized> SlotPermit<'a, Mut, Ptr, T, C> {
    pub fn borrow(&self) -> SlotPermit<Ref, Ptr, T, C> {
        SlotPermit {
            permit: self.permit.borrow(),
            key: self.key,
        }
    }

    pub fn borrow_mut(&mut self) -> SlotPermit<Mut, Ptr, T, C> {
        SlotPermit {
            permit: self.permit.borrow_mut(),
            key: self.key,
        }
    }
}

impl<'a, T: core::AnyItem + ?Sized, C: ?Sized> SlotPermit<'a, Mut, core::Ref<'a>, T, C> {
    pub fn borrow(&self) -> SlotPermit<Ref, core::Ref, T, C> {
        SlotPermit {
            permit: self.permit.borrow(),
            key: self.key.borrow(),
        }
    }

    pub fn borrow_mut(&mut self) -> SlotPermit<Mut, core::Ref, T, C> {
        SlotPermit {
            permit: self.permit.borrow_mut(),
            key: self.key.borrow(),
        }
    }
}

impl<'a, K: Copy, T: core::AnyItem + ?Sized, C: ?Sized> Copy for SlotPermit<'a, Ref, K, T, C> {}

impl<'a, K: Copy, T: core::AnyItem + ?Sized, C: ?Sized> Clone for SlotPermit<'a, Ref, K, T, C> {
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}
