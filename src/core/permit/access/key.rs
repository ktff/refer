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

impl<'a, C: AnyContainer + ?Sized, R: Permit, TP: Permits<T>, T: DynItem + ?Sized>
    Access<'a, C, R, TP, Key<Ptr, T>>
{
    pub fn get_try(self) -> Option<Slot<'a, R, T>> {
        let Self {
            container,
            key_state: key,
            permit,
            ..
        } = self;
        container
            .unified_get_slot(key)
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
}
