use super::*;
use crate::core::{AnyContainer, AnyKey, Key};
use std::collections::HashSet;

pub struct SlotSplitPermit<'a, A, C: ?Sized> {
    permit: AnyPermit<'a, Mut, A, C>,
    splitted: HashSet<AnyKey>,
}

impl<'a, A, C: ?Sized> SlotSplitPermit<'a, A, C> {
    pub fn new(permit: AnyPermit<'a, Mut, A, C>) -> Self {
        Self {
            permit,
            splitted: HashSet::new(),
        }
    }

    pub fn slot<T: core::DynItem + ?Sized>(
        &mut self,
        key: Key<T>,
    ) -> Option<SlotPermit<'a, T, Mut, A, C>>
    where
        C: AnyContainer,
    {
        if self.splitted.insert(key.any()) {
            // SAFETY: We just checked that the key is not splitted.
            Some(unsafe { self.permit.unsafe_split(|permit| permit.slot(key)) })
        } else {
            None
        }
    }
}
