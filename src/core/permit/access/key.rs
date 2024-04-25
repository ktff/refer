use crate::core::AnyItem;

use super::*;

impl<'a, C: AnyContainer + ?Sized, R: Permit, TP: TypePermit, K: Clone, T: DynItem + ?Sized>
    Access<'a, C, R, TP, Key<K, T>>
{
    pub fn key(&self) -> Key<K, T> {
        self.key_state.clone()
    }
}

impl<'a, C: Container<T> + ?Sized, R: Permit, K: Clone, T: Item> Access<'a, C, R, All, Key<K, T>> {
    pub fn ty(self) -> Access<'a, C, R, T, Key<K, T>> {
        self.type_transition(|()| ())
    }
}

impl<'a, C: AnyContainer + ?Sized, R: Permit, TP: Permits<T>, T: DynItem + ?Sized, K: Clone>
    Access<'a, C, R, TP, Key<K, T>>
{
    pub fn get_try(self) -> Option<Slot<'a, R, T>> {
        let Self {
            container,
            key_state: key,
            permit,
            ..
        } = self;
        container
            .unified_get_slot(key.ptr())
            // SAFETY: Type level logic of permit ensures that it has sufficient access for 'a to this slot.
            .map(|slot| unsafe { Slot::new(slot, permit) })
    }
}

impl<'a, C: AnyContainer + ?Sized, R: Permit, TP: Permits<T>, T: DynItem + ?Sized>
    Access<'a, C, R, TP, Key<Ref<'a>, T>>
{
    pub fn get(self) -> Slot<'a, R, T> {
        self.key_transition(|key| key.ptr())
            .get_try()
            .expect("Reference is invalid for given container.")
    }

    pub fn slot_access(self) -> SlotAccess<'a, C, R, T> {
        let Self {
            container,
            key_state: key,
            permit,
            ..
        } = self;
        SlotAccess {
            container,
            permit,
            key,
        }
    }
}

/// Provides access to one slot.
pub struct SlotAccess<
    'a,
    C: AnyContainer + ?Sized,
    P: Permit = permit::Ref,
    T: DynItem + ?Sized = dyn AnyItem,
> {
    container: &'a C,
    permit: P,
    key: Key<Ref<'a>, T>,
}

impl<'a, C: AnyContainer + ?Sized, P: Permit, T: DynItem + ?Sized> SlotAccess<'a, C, P, T> {
    pub fn get(self) -> Slot<'a, P, T> {
        self.container
            .unified_get_slot(self.key.ptr())
            .map(|slot| unsafe { Slot::new(slot, self.permit) })
            .expect("Reference is invalid for given container.")
    }

    pub fn borrow_mut<'b>(&'b mut self) -> SlotAccess<'b, C, P, T> {
        SlotAccess {
            container: self.container,
            // SAFETY: We are mutably borrowing original permit and this won't overlive that lifetime.
            permit: unsafe { self.permit.copy() },
            key: self.key,
        }
    }

    pub fn key(&self) -> Key<Ref<'a>, T> {
        self.key.clone()
    }
}

impl<'a, C: AnyContainer + ?Sized, P: Permit + Copy, T: DynItem + ?Sized> Copy
    for SlotAccess<'a, C, P, T>
{
}

impl<'a, C: AnyContainer + ?Sized, P: Permit + Clone, T: DynItem + ?Sized> Clone
    for SlotAccess<'a, C, P, T>
{
    fn clone(&self) -> Self {
        Self {
            container: self.container,
            permit: self.permit.clone(),
            key: self.key,
        }
    }
}
