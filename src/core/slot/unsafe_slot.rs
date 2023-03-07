use super::*;
use crate::core::{Item, ItemLocality};
use getset::CopyGetters;
use std::cell::SyncUnsafeCell;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct UnsafeSlot<'a, T: Item> {
    locality: ItemLocality<'a, T>,
    item: &'a SyncUnsafeCell<T>,
}

impl<'a, T: Item> UnsafeSlot<'a, T> {
    pub fn new(locality: ItemLocality<'a, T>, item: &'a SyncUnsafeCell<T>) -> Self {
        Self { locality, item }
    }

    pub fn any(self) -> AnyUnsafeSlot<'a> {
        AnyUnsafeSlot::new(self.locality.any(), self.item)
    }
}

impl<'a, T: Item> Copy for UnsafeSlot<'a, T> {}

impl<'a, T: Item> Clone for UnsafeSlot<'a, T> {
    fn clone(&self) -> Self {
        Self {
            locality: self.locality,
            item: self.item,
        }
    }
}

// Deref to locality
impl<'a, T: Item> std::ops::Deref for UnsafeSlot<'a, T> {
    type Target = ItemLocality<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.locality
    }
}
