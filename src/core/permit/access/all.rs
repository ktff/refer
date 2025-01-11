use super::*;

impl<'a, C: AnyContainer + ?Sized, R: Permit> Access<'a, C, R, All, All> {
    pub fn ty<T: Item>(self) -> Access<'a, C, R, T, All>
    where
        C: Container<T>,
    {
        self.type_transition(|()| ())
    }

    pub fn type_split<T: Item>(self) -> (Access<'a, C, R, T, All>, Access<'a, C, R, Not<T>, All>)
    where
        C: Container<T>,
    {
        // SAFETY: T and Not<T> are disjoint.
        let not = unsafe { self.unsafe_type_split(()) };

        (self.ty(), not)
    }

    pub fn types_split(self) -> Access<'a, C, R, Not<Types>, All> {
        self.type_transition(|()| Types::default())
    }
}

impl<'a, C: AnyContainer + ?Sized, R: Permit, TP: TypePermit> Access<'a, C, R, TP, All> {
    pub fn key<K: Clone, T: DynItem + ?Sized>(
        self,
        key: Key<K, T>,
    ) -> Access<'a, C, R, TP, Key<K, T>>
    where
        TP: Permits<T>,
    {
        self.key_transition(|All| key)
    }

    pub fn key_split<K: Copy, T: DynItem + ?Sized>(
        self,
        key: Key<K, T>,
    ) -> (
        Access<'a, C, R, TP, Key<K, T>>,
        Access<'a, C, R, TP, Not<Key>>,
    )
    where
        TP: Permits<T>,
    {
        // SAFETY: Key and Not<Key> are disjoint.
        let key_split = unsafe { self.unsafe_key_split(key) };
        (key_split, self.key_transition(|_| Not(key.ptr().any())))
    }

    pub fn keys_split(self) -> Access<'a, C, R, TP, Keys> {
        self.key_transition(|All| Keys::default())
    }

    pub fn top_key_split(self) -> Access<'a, C, R, TP, TopKey> {
        self.key_transition(|All| TopKey::default())
    }

    pub fn keys_split_with<K: KeyPermit>(self, keys: K) -> Access<'a, C, R, TP, K> {
        self.key_transition(|All| keys)
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

impl<'a, C: AnyContainer + ?Sized, R: Permit> Access<'a, C, R, All, Not<Key>> {
    pub fn key_try<K: Copy, T: DynItem + ?Sized>(
        self,
        key: Key<K, T>,
    ) -> Option<Access<'a, C, R, All, Key<K, T>>>
    where
        C: AnyContainer,
    {
        if self.key_state.0 == key.any().ptr() {
            None
        } else {
            // SAFETY: We just checked that the key is not splitted.
            Some(self.key_transition(|_| key))
        }
    }

    pub fn keys_split(self) -> Access<'a, C, R, All, Keys> {
        self.key_transition(|key| Keys::new([key.0]))
    }
}

impl Default for All {
    fn default() -> Self {
        Self
    }
}
