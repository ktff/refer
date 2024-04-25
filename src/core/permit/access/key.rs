use super::*;
use log::warn;

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

impl<'a, C: AnyContainer + ?Sized, R: Permit, T: DynItem + ?Sized>
    Access<'a, C, R, All, Key<Ptr, T>>
{
    /// None if doesn't exist as such type.
    pub fn get_dyn_try(self) -> Option<DynSlot<'a, R, T>>
    where
        T: AnyDynItem,
    {
        let Self {
            container,
            key_state: key,
            permit,
            ..
        } = self;
        container
            .any_get_slot(key.any())
            // SAFETY: Type level logic of permit ensures that it has sufficient access for 'a to this slot.
            .and_then(|slot| {
                unsafe { DynSlot::new_any(slot, permit) }
                    .sidecast()
                    .map_err(|slot| {
                        warn!(
                            "Item at {:?}:{} is not {:?} which was assumed to be true.",
                            slot.locality().path(),
                            slot.type_name(),
                            std::any::type_name::<T>()
                        );
                    })
                    .ok()
            })
    }

    pub fn get_try(self) -> Option<Slot<'a, R, T>>
    where
        T: Item,
        C: Container<T>,
    {
        self.ty().get_try()
    }
}

impl<'a, C: Container<T> + ?Sized, R: Permit, T: Item> Access<'a, C, R, T, Key<Ptr, T>> {
    pub fn get_try(self) -> Option<Slot<'a, R, T>> {
        let Self {
            container,
            key_state: key,
            permit,
            ..
        } = self;
        container
            .get_slot(key)
            // SAFETY: Type level logic of permit ensures that it has sufficient access for 'a to this slot.
            .map(|slot| unsafe { Slot::new(slot, permit) })
    }
}

impl<'a, C: AnyContainer + ?Sized, R: Permit, T: DynItem + ?Sized>
    Access<'a, C, R, All, Key<Ref<'a>, T>>
{
    pub fn get_dyn(self) -> DynSlot<'a, R, T>
    where
        T: AnyDynItem,
    {
        self.key_transition(|key| key.ptr())
            .get_dyn_try()
            .expect("Reference is invalid for given container.")
    }

    pub fn get(self) -> Slot<'a, R, T>
    where
        T: Item,
        C: Container<T>,
    {
        self.ty().get()
    }
}

impl<'a, C: Container<T> + ?Sized, R: Permit, T: Item> Access<'a, C, R, T, Key<Ref<'a>, T>> {
    pub fn get(self) -> Slot<'a, R, T> {
        self.key_transition(|key| key.ptr())
            .get_try()
            .expect("Reference is invalid for given container.")
    }
}
