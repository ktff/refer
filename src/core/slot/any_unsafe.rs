use crate::core::{AnyItem, AnyShell, AnySlotLocality, DynItem};
use getset::CopyGetters;
use log::*;
use std::{any::TypeId, cell::SyncUnsafeCell};

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct AnyUnsafeSlot<'a> {
    locality: AnySlotLocality<'a>,
    item: &'a SyncUnsafeCell<dyn AnyItem>,
    shell: &'a SyncUnsafeCell<dyn AnyShell>,
}

impl<'a> AnyUnsafeSlot<'a> {
    pub fn new(
        locality: AnySlotLocality<'a>,
        item: &'a SyncUnsafeCell<dyn AnyItem>,
        shell: &'a SyncUnsafeCell<dyn AnyShell>,
    ) -> Self {
        Self {
            locality,
            item,
            shell,
        }
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
}

impl<'a> Copy for AnyUnsafeSlot<'a> {}

impl<'a> Clone for AnyUnsafeSlot<'a> {
    fn clone(&self) -> Self {
        Self {
            locality: self.locality,
            item: self.item,
            shell: self.shell,
        }
    }
}

// Deref to locality
impl<'a> std::ops::Deref for AnyUnsafeSlot<'a> {
    type Target = AnySlotLocality<'a>;

    fn deref(&self) -> &Self::Target {
        &self.locality
    }
}
