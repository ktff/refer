use super::*;

impl<
        'a,
        C: AnyContainer + ?Sized,
        R: Into<permit::Ref>,
        TP: TypePermit,
        K: Clone,
        T: DynItem + ?Sized,
    > AccessPermit<'a, C, R, TP, Key<K, T>>
{
    pub fn key(&self) -> Key<K, T> {
        self.key_state.clone()
    }
}

impl<'a, C: Container<T> + ?Sized, R: Into<permit::Ref>, K: Clone, T: Item>
    AccessPermit<'a, C, R, All, Key<K, T>>
{
    pub fn ty(self) -> AccessPermit<'a, C, R, T, Key<K, T>> {
        self.type_transition(|()| ())
    }
}

impl<'a, C: AnyContainer + ?Sized, R: Into<permit::Ref>, T: DynItem + ?Sized>
    AccessPermit<'a, C, R, All, Key<Ptr, T>>
{
    /// None if doesn't exist.
    pub fn get_dyn_try(self) -> Option<DynSlot<'a, R, T>> {
        let Self {
            container,
            key_state: key,
            permit,
            ..
        } = self;
        container
            .get_slot_any(key.any())
            // SAFETY: Type level logic of permit ensures that it has sufficient access for 'a to this slot.
            .and_then(|slot| unsafe { DynSlot::new(slot, permit) })
    }

    pub fn get_try(self) -> Option<Slot<'a, R, T>>
    where
        T: Item,
        C: Container<T>,
    {
        self.ty().get_try()
    }
}

impl<'a, C: Container<T> + ?Sized, R: Into<permit::Ref>, T: Item>
    AccessPermit<'a, C, R, T, Key<Ptr, T>>
{
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

impl<'a, C: AnyContainer + ?Sized, R: Into<permit::Ref>, T: DynItem + ?Sized>
    AccessPermit<'a, C, R, All, Key<Ref<'a>, T>>
{
    pub fn get_dyn(self) -> DynSlot<'a, R, T> {
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

impl<'a, C: Container<T> + ?Sized, R: Into<permit::Ref>, T: Item>
    AccessPermit<'a, C, R, T, Key<Ref<'a>, T>>
{
    pub fn get(self) -> Slot<'a, R, T> {
        self.key_transition(|key| key.ptr())
            .get_try()
            .expect("Reference is invalid for given container.")
    }
}
