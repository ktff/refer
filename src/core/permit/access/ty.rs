use super::*;

impl<'a, C: Container<T> + ?Sized, R: Permit, T: Item> Access<'a, C, R, T, All> {
    pub fn types_split(self) -> Access<'a, C, R, Types, All> {
        self.type_transition(|()| {
            let mut type_state = Types::default();
            type_state.insert::<T>();
            type_state
        })
    }
}

impl<'a, C: AnyContainer + ?Sized, R: Permit, T: Item> Access<'a, C, R, Not<T>, All> {
    pub fn types_split(self) -> Access<'a, C, R, Not<Types>, All> {
        self.type_transition(|()| {
            let mut type_state = Types::default();
            type_state.insert::<T>();
            type_state
        })
    }

    pub fn ty<D: Item>(self) -> Option<Access<'a, C, R, D, All>>
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
