use super::*;
use crate::core::{AnyContainer, DynItem, Key};

pub struct SubjectPermit<'a, C: ?Sized> {
    permit: AnyPermit<'a, Mut, C>,
    subject: Key,
}

impl<'a, C: AnyContainer + ?Sized> SubjectPermit<'a, C> {
    pub fn new<K: Copy, T: DynItem + ?Sized>(
        permit: AnyPermit<'a, Mut, C>,
        subject: Key<K, T>,
    ) -> (SlotPermit<'a, Mut, K, T, C>, Self) {
        let key = subject.any().ptr();
        // SAFETY: We ensure in the rest of this struct that this key is not accessed again.
        let slot = unsafe { permit.unsafe_split(|permit| permit.key(subject)) };
        (
            slot,
            Self {
                permit,
                subject: key,
            },
        )
    }

    pub fn subject(&self) -> Key {
        self.subject
    }

    /// None if it's the subject.
    pub fn slot<'b, K: Copy, T: core::DynItem + ?Sized>(
        &'b mut self,
        key: Key<K, T>,
    ) -> Option<SlotPermit<'b, Mut, K, T, C>> {
        if self.subject == key.any().ptr() {
            None
        } else {
            // SAFETY: We just checked that the key is not splitted.
            Some(self.permit.borrow_mut().key(key))
        }
    }
}
