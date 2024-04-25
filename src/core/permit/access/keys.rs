use super::*;

impl<'a, C: AnyContainer + ?Sized, R: Permit, T: Item, K: KeyPermit> Access<'a, C, R, T, K> {
    pub fn key_try<K2: Copy>(self, key: Key<K2, T>) -> Option<Access<'a, C, R, T, Key<K2, T>>> {
        if K::allowed(&self.key_state, key) {
            // SAFETY: We just checked that we have permit for the key.
            Some(unsafe { self.unsafe_key_split(key) })
        } else {
            None
        }
    }
}

impl<'a, C: AnyContainer + ?Sized, R: Permit, TP: TypePermit, KEYS: KeyPermit + KeySet>
    Access<'a, C, R, TP, KEYS>
{
    pub fn take_key<K: Copy, T: DynItem + ?Sized>(
        &mut self,
        key: Key<K, T>,
    ) -> Option<Access<'a, C, R, TP, Key<K, T>>>
    where
        TP: Permits<T>,
    {
        if self.key_state.try_insert(key) {
            // SAFETY: We just checked that the key is not splitted.
            Some(unsafe { self.unsafe_key_split(key) })
        } else {
            None
        }
    }

    pub fn borrow_key<'b, K: Copy, T: DynItem + ?Sized>(
        &'b self,
        key: Key<K, T>,
    ) -> Option<Access<'b, C, R, TP, Key<K, T>>>
    where
        TP: Permits<T>,
    {
        if self.key_state.contains(key) {
            None
        } else {
            // SAFETY: We just checked that the key is not splitted and we are allowing it to live only for the lifetime of self.
            Some(unsafe { self.unsafe_key_split(key) })
        }
    }
}
