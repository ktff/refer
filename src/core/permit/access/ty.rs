use super::*;

impl<'a, C: Container<T> + ?Sized, R: Into<permit::Ref>, T: Item> AccessPermit<'a, C, R, T, All> {
    pub fn types_split(self) -> AccessPermit<'a, C, R, Types, All> {
        self.type_transition(|()| {
            let mut type_state = Types::default();
            type_state.insert::<T>();
            type_state
        })
    }

    pub fn key<K: Clone>(self, key: Key<K, T>) -> AccessPermit<'a, C, R, T, Key<K, T>>
    where
        C: AnyContainer,
    {
        self.key_transition(|()| key)
    }

    pub fn keys_split(self) -> AccessPermit<'a, C, R, T, Keys> {
        self.key_transition(|()| Keys::default())
    }

    pub fn key_split<K: Copy>(
        self,
        key: Key<K, T>,
    ) -> (
        AccessPermit<'a, C, R, T, Key<K, T>>,
        AccessPermit<'a, C, R, T, Not<Key>>,
    ) {
        // SAFETY: Key and Not<Key> are disjoint.
        let key_split = unsafe { self.unsafe_split(|this| this.key_transition(|()| key)) };
        (key_split, self.key_transition(|()| key.ptr().any()))
    }
}

impl<'a, C: AnyContainer + ?Sized, R: Into<permit::Ref>, T: Item>
    AccessPermit<'a, C, R, Not<T>, All>
{
    pub fn types_split(self) -> AccessPermit<'a, C, R, Not<Types>, All> {
        self.type_transition(|()| {
            let mut type_state = Types::default();
            type_state.insert::<T>();
            type_state
        })
    }

    pub fn ty<D: Item>(self) -> Option<AccessPermit<'a, C, R, D, All>>
    where
        C: Container<D>,
    {
        if TypeId::of::<T>() != TypeId::of::<D>() {
            // SAFETY: We just checked that we have permit for the type.
            Some(unsafe { self.unsafe_type_split(()) })
        } else {
            None
        }
    }
}
