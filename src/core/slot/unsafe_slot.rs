use super::*;
use crate::core::{Item, Shell, SlotLocality};
use getset::CopyGetters;
use std::cell::SyncUnsafeCell;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct UnsafeSlot<'a, T: Item, S: Shell<T = T>> {
    locality: SlotLocality<'a, T>,
    item: &'a SyncUnsafeCell<T>,
    shell: &'a SyncUnsafeCell<S>,
}

impl<'a, T: Item, S: Shell<T = T>> UnsafeSlot<'a, T, S> {
    pub fn new(
        locality: SlotLocality<'a, T>,
        item: &'a SyncUnsafeCell<T>,
        shell: &'a SyncUnsafeCell<S>,
    ) -> Self {
        Self {
            locality,
            item,
            shell,
        }
    }

    pub fn upcast(self) -> AnyUnsafeSlot<'a> {
        AnyUnsafeSlot::new(self.locality.upcast(), self.item, self.shell)
    }
}

impl<'a, T: Item, S: Shell<T = T>> Copy for UnsafeSlot<'a, T, S> {}

impl<'a, T: Item, S: Shell<T = T>> Clone for UnsafeSlot<'a, T, S> {
    fn clone(&self) -> Self {
        Self {
            locality: self.locality,
            item: self.item,
            shell: self.shell,
        }
    }
}

// Deref to locality
impl<'a, T: Item, S: Shell<T = T>> std::ops::Deref for UnsafeSlot<'a, T, S> {
    type Target = SlotLocality<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.locality
    }
}
