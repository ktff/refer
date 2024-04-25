use crate::core::{
    cast_as_eq_type, AnyAlloc, AnyDynItem, AnyItem, AnyLocalityData, DynItem, Item, ItemLocality,
};
use getset::CopyGetters;
use std::{
    any::TypeId,
    cell::SyncUnsafeCell,
    ptr::{DynMetadata, Pointee},
};

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct UnsafeSlot<'a, T: DynItem + ?Sized = dyn AnyItem> {
    locality: ItemLocality<'a, T>,
    item: &'a SyncUnsafeCell<T::AnyType>,
}

impl<'a, T: DynItem + ?Sized> UnsafeSlot<'a, T> {
    pub fn new(locality: ItemLocality<'a, T>, item: &'a SyncUnsafeCell<T::AnyType>) -> Self {
        Self { locality, item }
    }

    pub fn metadata<D: DynItem + ?Sized>(&self) -> Option<D::Metadata> {
        PointeeMetadata::<D::Metadata>::any_metadata(self, TypeId::of::<D>())
    }

    pub fn item_type_id(&self) -> std::any::TypeId {
        self.item().get().item_type_id()
    }

    pub fn item_as_any(&self) -> &SyncUnsafeCell<dyn AnyItem> {
        T::as_any_type(self.item)
    }
}

impl<'a, T: AnyDynItem + ?Sized> UnsafeSlot<'a, T> {
    pub fn downcast<D: Item>(self) -> Option<UnsafeSlot<'a, D>> {
        if TypeId::of::<D>() == self.item_type_id() {
            // SAFETY: We know that the item is of type D, so we can safely cast it.
            let item = unsafe {
                &*(self.item as *const SyncUnsafeCell<T::AnyType> as *const SyncUnsafeCell<D>)
            };

            Some(UnsafeSlot::new(
                self.locality.downcast_universal().expect("Unexpected type"),
                item,
            ))
        } else {
            None
        }
    }

    pub fn sidecast<D: AnyDynItem + ?Sized>(self) -> UnsafeSlot<'a, D> {
        // We know that T and D both have same AnyType in AnyDynItem.
        UnsafeSlot {
            locality: self.locality.sidecast(),
            item: cast_as_eq_type(self.item),
        }
    }
}

impl<'a, T: Item> UnsafeSlot<'a, T> {
    pub fn any(self) -> UnsafeSlot<'a> {
        UnsafeSlot::new(self.locality.any_universal(), T::as_any_type(self.item))
    }
}

impl<'a, T: DynItem + ?Sized> Copy for UnsafeSlot<'a, T> {}

impl<'a, T: DynItem + ?Sized> Clone for UnsafeSlot<'a, T> {
    fn clone(&self) -> Self {
        Self {
            locality: self.locality,
            item: self.item,
        }
    }
}

// Deref to locality
impl<'a, T: DynItem + ?Sized> std::ops::Deref for UnsafeSlot<'a, T> {
    type Target = ItemLocality<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.locality
    }
}

trait PointeeMetadata<M> {
    fn any_metadata(&self, type_id: TypeId) -> Option<M>;
}

impl<'a, T: DynItem + ?Sized, D> PointeeMetadata<D> for UnsafeSlot<'a, T> {
    default fn any_metadata(&self, _: TypeId) -> Option<D> {
        None
    }
}

impl<
        'a,
        T: DynItem<AnyType = dyn AnyItem, AnyAlloc = AnyAlloc, AnyLocalityData = AnyLocalityData>
            + ?Sized,
        D: Pointee<Metadata = DynMetadata<D>> + ?Sized + 'static,
    > PointeeMetadata<DynMetadata<D>> for UnsafeSlot<'a, T>
{
    default fn any_metadata(&self, type_id: TypeId) -> Option<DynMetadata<D>> {
        let metadata = self.item.get().trait_metadata(type_id)?;
        metadata.downcast::<D>()
    }
}

impl<
        'a,
        T: DynItem<AnyType = dyn AnyItem, AnyAlloc = AnyAlloc, AnyLocalityData = AnyLocalityData>
            + ?Sized,
    > PointeeMetadata<DynMetadata<dyn AnyItem>> for UnsafeSlot<'a, T>
{
    fn any_metadata(&self, _: TypeId) -> Option<DynMetadata<dyn AnyItem>> {
        Some(std::ptr::metadata(self.item.get()))
    }
}

impl<'a, T: DynItem + ?Sized> PointeeMetadata<()> for UnsafeSlot<'a, T> {
    fn any_metadata(&self, type_id: TypeId) -> Option<()> {
        if type_id == self.item_type_id() {
            Some(())
        } else {
            None
        }
    }
}

// impl<
//         'a,
//         T: DynItem<AnyType = dyn AnyItem, AnyAlloc = AnyAlloc, AnyLocalityData = AnyLocalityData>
//             + ?Sized,
//     > PointeeMetadata<()> for AnyUnsafeSlot<'a, T>
// {
//     fn any_metadata(&self, type_id: TypeId) -> Option<()> {
//         if type_id == self.item_type_id() {
//             Some(())
//         } else {
//             None
//         }
//     }
// }
