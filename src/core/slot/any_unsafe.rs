use crate::core::{AnyItem, AnyItemLocality, DynItem, Item};
use getset::CopyGetters;
use std::{
    any::TypeId,
    cell::SyncUnsafeCell,
    ptr::{DynMetadata, Pointee},
};

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
        PointeeMetadata::<T::Metadata>::any_metadata(self, TypeId::of::<T>())
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

trait PointeeMetadata<M> {
    fn any_metadata(&self, type_id: TypeId) -> Option<M>;
}

impl<'a, M> PointeeMetadata<M> for AnyUnsafeSlot<'a> {
    default fn any_metadata(&self, _: TypeId) -> Option<M> {
        None
    }
}

impl<'a, T: Pointee<Metadata = DynMetadata<T>> + DynItem + ?Sized> PointeeMetadata<DynMetadata<T>>
    for AnyUnsafeSlot<'a>
{
    default fn any_metadata(&self, type_id: TypeId) -> Option<DynMetadata<T>> {
        let metadata = self.item.get().trait_metadata(type_id)?;
        metadata.downcast::<T>()
    }
}

impl<'a> PointeeMetadata<DynMetadata<dyn AnyItem>> for AnyUnsafeSlot<'a> {
    fn any_metadata(&self, _: TypeId) -> Option<DynMetadata<dyn AnyItem>> {
        Some(std::ptr::metadata(self.item.get()))
    }
}

impl<'a> PointeeMetadata<()> for AnyUnsafeSlot<'a> {
    fn any_metadata(&self, type_id: TypeId) -> Option<()> {
        if type_id == self.item_type_id() {
            Some(())
        } else {
            None
        }
    }
}
