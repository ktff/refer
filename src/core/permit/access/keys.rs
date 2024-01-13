use super::*;

impl<'a, C: AnyContainer + ?Sized, R: Into<permit::Ref>, T: Item, K: KeyPermit>
    AccessPermit<'a, C, R, T, K>
{
    pub fn key_try<K2: Copy>(
        self,
        key: Key<K2, T>,
    ) -> Option<AccessPermit<'a, C, R, T, Key<K2, T>>> {
        if K::allowed(&self.key_state, key) {
            // SAFETY: We just checked that we have permit for the key.
            Some(unsafe { self.unsafe_key_split(key) })
        } else {
            None
        }
    }
}

impl<'a, C: AnyContainer + ?Sized, R: Into<permit::Ref>> AccessPermit<'a, C, R, All, Keys> {
    pub fn take_key<K: Copy, T: DynItem + ?Sized>(
        &mut self,
        key: Key<K, T>,
    ) -> Option<AccessPermit<'a, C, R, All, Key<K, T>>>
    where
        C: AnyContainer,
    {
        if self.key_state.try_insert(key) {
            // SAFETY: We just checked that the key is not splitted.
            Some(unsafe { self.unsafe_key_split(key) })
        } else {
            None
        }
    }
}

impl<'a, C: AnyContainer + ?Sized, R: Into<permit::Ref>, T: Item> AccessPermit<'a, C, R, T, Keys> {
    pub fn take_key<K: Copy>(
        &mut self,
        key: Key<K, T>,
    ) -> Option<AccessPermit<'a, C, R, T, Key<K, T>>>
    where
        C: AnyContainer,
    {
        if self.key_state.try_insert(key) {
            // SAFETY: We just checked that the key is not splitted.
            Some(unsafe { self.unsafe_key_split(key) })
        } else {
            None
        }
    }

    pub fn borrow_key<'b, K: Copy>(
        &'b self,
        key: Key<K, T>,
    ) -> Option<AccessPermit<'b, C, R, T, Key<K, T>>> {
        if self.key_state.contains(key) {
            None
        } else {
            // SAFETY: We just checked that the key is not splitted and we are allowing it to live only for the lifetime of self.
            Some(unsafe { self.unsafe_key_split(key) })
        }
    }
}
