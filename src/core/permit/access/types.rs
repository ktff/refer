use super::*;

impl<'a, C: AnyContainer + ?Sized, TP: TypePermit> Access<'a, C, permit::Ref, TP, All> {
    pub fn ty_try<T: Item>(&self) -> Option<Access<'a, C, permit::Ref, T, All>>
    where
        C: Container<T>,
    {
        if TP::allowed::<T>(&self.type_state) {
            // SAFETY: We just checked that we have permit for the type.
            Some(unsafe { self.unsafe_type_split(()) })
        } else {
            None
        }
    }
}

impl<'a, C: AnyContainer + ?Sized> Access<'a, C, Mut, Types, All> {
    pub fn take_ty<T: Item>(&mut self) -> Option<Access<'a, C, Mut, T, All>>
    where
        C: Container<T>,
    {
        if self.type_state.remove::<T>() {
            // SAFETY: We just checked that we have permit for the type.
            Some(unsafe { self.unsafe_type_split(()) })
        } else {
            None
        }
    }
}

impl<'a, C: AnyContainer + ?Sized> Access<'a, C, Mut, Not<Types>, All> {
    pub fn take_ty<T: Item>(&mut self) -> Option<Access<'a, C, Mut, T, All>>
    where
        C: Container<T>,
    {
        if self.type_state.try_insert::<T>() {
            // SAFETY: We just checked that the type was not splitted of.
            Some(unsafe { self.unsafe_type_split(()) })
        } else {
            None
        }
    }
}
