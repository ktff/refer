use super::*;

impl<'a, C: AnyContainer + ?Sized, R: Into<permit::Ref>> AccessPermit<'a, C, R, All, All> {
    pub fn ty<T: Item>(self) -> AccessPermit<'a, C, R, T, All>
    where
        C: Container<T>,
    {
        self.type_transition(|()| ())
    }

    pub fn type_split<T: Item>(
        self,
    ) -> (
        AccessPermit<'a, C, R, T, All>,
        AccessPermit<'a, C, R, Not<T>, All>,
    )
    where
        C: Container<T>,
    {
        // SAFETY: T and Not<T> are disjoint.
        let not = unsafe { self.unsafe_type_split(()) };

        (self.ty(), not)
    }

    pub fn types_split(self) -> AccessPermit<'a, C, R, Not<Types>, All> {
        self.type_transition(|()| Types::default())
    }

    pub fn key<K: Clone, T: DynItem + ?Sized>(
        self,
        key: Key<K, T>,
    ) -> AccessPermit<'a, C, R, All, Key<K, T>>
    where
        C: AnyContainer,
    {
        self.key_transition(|()| key)
    }

    pub fn key_split<K: Copy, T: DynItem + ?Sized>(
        self,
        key: Key<K, T>,
    ) -> (
        AccessPermit<'a, C, R, All, Key<K, T>>,
        AccessPermit<'a, C, R, All, Not<Key>>,
    ) {
        // SAFETY: Key and Not<Key> are disjoint.
        let key_split = unsafe { self.unsafe_split(|this| this.key_transition(|()| key)) };
        (key_split, self.key_transition(|()| key.ptr().any()))
    }

    pub fn keys_split(self) -> AccessPermit<'a, C, R, All, Keys> {
        self.key_transition(|()| Keys::default())
    }

    // /// Iterates over valid slot permit of type in ascending order.
    // pub fn iter<T: core::Item>(
    //     self,
    // ) -> impl Iterator<Item = SlotPermit<'a, R, core::Ref<'a>, T, C>> {
    //     let container = self.container;
    //     std::iter::successors(container.first_key(TypeId::of::<T>()), move |&key| {
    //         container.next_key(TypeId::of::<T>(), key.ptr())
    //     })
    //     .map(move |key| {
    //         // SAFETY: First-next iteration ensures that we don't access the same slot twice.
    //         unsafe { self.unsafe_split(|permit| permit.slot(key.assume())) }
    //     })
    // }
}

impl<'a, C: AnyContainer + ?Sized, R: Into<permit::Ref>> AccessPermit<'a, C, R, All, Not<Key>> {
    pub fn key_try<K: Copy, T: DynItem + ?Sized>(
        self,
        key: Key<K, T>,
    ) -> Option<AccessPermit<'a, C, R, All, Key<K, T>>>
    where
        C: AnyContainer,
    {
        if self.key_state == key.any().ptr() {
            None
        } else {
            // SAFETY: We just checked that the key is not splitted.
            Some(self.key_transition(|_| key))
        }
    }

    pub fn keys_split(self) -> AccessPermit<'a, C, R, All, Keys> {
        self.key_transition(|key| Keys::new([key]))
    }
}
