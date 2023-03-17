use super::*;

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
