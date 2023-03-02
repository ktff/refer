use super::*;
use crate::core::{AnyContainer, Key};
use std::collections::HashSet;

pub struct SlotSplitPermit<'a, C: ?Sized> {
    permit: AnyPermit<'a, Mut, C>,
    splitted: HashSet<Key>,
}

impl<'a, C: ?Sized> SlotSplitPermit<'a, C> {
    pub fn new(permit: AnyPermit<'a, Mut, C>) -> Self {
        Self {
            permit,
            splitted: HashSet::new(),
        }
    }

    pub fn slot<K: Copy, T: core::DynItem + ?Sized>(
        &mut self,
        key: Key<K, T>,
    ) -> Option<SlotPermit<'a, Mut, K, T, C>>
    where
        C: AnyContainer,
    {
        if self.splitted.insert(key.any().ptr()) {
            // SAFETY: We just checked that the key is not splitted.
            Some(unsafe { self.permit.unsafe_split(|permit| permit.slot(key)) })
        } else {
            None
        }
    }
}
