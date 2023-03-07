use crate::core::{AnyItem, AnyItemLocality, DynItem, Item};
use getset::CopyGetters;
use log::*;
use std::{any::TypeId, cell::SyncUnsafeCell};

use super::UnsafeSlot;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct AnyUnsafeSlot<'a> {
    locality: AnyItemLocality<'a>,
    item: &'a SyncUnsafeCell<dyn AnyItem>,
}

impl<'a> AnyUnsafeSlot<'a> {
    pub fn new(locality: AnyItemLocality<'a>, item: &'a SyncUnsafeCell<dyn AnyItem>) -> Self {
        Self { locality, item }
    }

    pub fn item_type_id(&self) -> std::any::TypeId {
        self.item().get().item_type_id()
    }

    pub fn metadata<T: DynItem + ?Sized>(&self) -> Option<T::Metadata> {
        let metadata = self.item.get().trait_metadata(TypeId::of::<T>())?;

        if let Some(&metadata) = metadata.downcast_ref::<T::Metadata>() {
            Some(metadata)
        } else {
            error!(
                "Item {:?} returned unexpected metadata for type {}. Expected: {}, got: {:?}",
                self.item.get().type_info(),
                std::any::type_name::<T>(),
                std::any::type_name::<T::Metadata>(),
                metadata.type_id(),
            );
            panic!("Metadata type mismatch");
        }
    }

    pub fn downcast<T: Item>(self) -> Option<UnsafeSlot<'a, T>> {
        if TypeId::of::<T>() == self.item_type_id() {
            // SAFETY: We know that the item is of type T, so we can safely cast it.
            let item = unsafe {
                &*(self.item as *const SyncUnsafeCell<dyn AnyItem> as *const SyncUnsafeCell<T>)
            };

            Some(UnsafeSlot::new(
                self.locality.downcast().expect("Unexpected type"),
                item,
            ))
        } else {
            None
        }
    }
}

impl<'a> Copy for AnyUnsafeSlot<'a> {}

impl<'a> Clone for AnyUnsafeSlot<'a> {
    fn clone(&self) -> Self {
        Self {
            locality: self.locality,
            item: self.item,
        }
    }
}

// Deref to locality
impl<'a> std::ops::Deref for AnyUnsafeSlot<'a> {
    type Target = AnyItemLocality<'a>;

    fn deref(&self) -> &Self::Target {
        &self.locality
    }
}
