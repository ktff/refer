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
