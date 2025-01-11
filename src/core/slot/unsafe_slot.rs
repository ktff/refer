use crate::core::{
    cast_as_eq_type, AnyAlloc, AnyDynItem, AnyItem, AnyLocalityData, DynItem, Item, ItemLocality,
    LocalityRef,
};
use std::{
    any::TypeId,
    cell::SyncUnsafeCell,
    ptr::{DynMetadata, Pointee},
};

pub struct UnsafeSlot<'a, T: DynItem + ?Sized = dyn AnyItem> {
    metadata: T::Metadata,
    locality: ItemLocality<'a, T>,
    item: &'a SyncUnsafeCell<T::AnyType>,
}

impl<'a, T: Item> UnsafeSlot<'a, T> {
    pub fn new(locality: ItemLocality<'a, T>, item: &'a SyncUnsafeCell<T>) -> Self {
        let metadata = std::ptr::metadata(item.get());
        Self {
            metadata,
            locality,
            item,
        }
    }
}

impl<'a> UnsafeSlot<'a> {
    pub fn new_any(locality: ItemLocality<'a>, item: &'a SyncUnsafeCell<dyn AnyItem>) -> Self {
        let metadata = std::ptr::metadata(item.get());
        Self {
            metadata,
            locality,
            item,
        }
    }
}

impl<'a, T: DynItem + ?Sized> UnsafeSlot<'a, T> {
    pub fn item_type_id(&self) -> std::any::TypeId {
        self.item.get().item_type_id()
    }

    pub fn item_type_name(&self) -> &'static str {
        self.item.get().type_info().name
    }

    pub fn item_as_any(&self) -> &SyncUnsafeCell<dyn AnyItem> {
        T::as_any_type(self.item)
    }

    pub fn any(self) -> UnsafeSlot<'a> {
        UnsafeSlot::new_any(self.locality.any(), T::as_any_type(self.item))
    }

    pub fn item(&self) -> *const T {
        let ptr = self.item.get();

        // SAFETY: During construction and conversions we checked that the type of the item matches the type of the key.
        std::ptr::from_raw_parts(ptr as *const (), self.metadata)
    }

    pub fn item_mut(&mut self) -> *mut T {
        let ptr = self.item.get();

        // SAFETY: During construction and conversions we checked that the type of the item matches the type of the key.
        std::ptr::from_raw_parts_mut(ptr as *mut (), self.metadata)
    }

    pub fn locality(&self) -> ItemLocality<'a, T> {
        self.locality
    }

    fn metadata<D: DynItem + ?Sized>(&self) -> Option<D::Metadata> {
        PointeeMetadata::<D::Metadata>::any_metadata(self, TypeId::of::<D>())
    }
}

impl<'a, T: AnyDynItem + ?Sized> UnsafeSlot<'a, T> {
    pub fn anycast<D: DynItem + ?Sized>(self) -> Option<UnsafeSlot<'a, D>> {
        D::anycast(self)
    }

    pub fn sidecast<
        D: DynItem<AnyLocalityData = T::AnyLocalityData, AnyAlloc = T::AnyAlloc> + ?Sized,
    >(
        self,
    ) -> Option<UnsafeSlot<'a, D>> {
        let metadata = self.metadata::<D>()?;

        // We know that T and D both have same AnyType in AnyDynItem.
        Some(UnsafeSlot {
            locality: self.locality.sidecast(),
            item: cast_as_eq_type(self.item),
            metadata,
        })
    }

    pub fn downcast<D: Item>(self) -> Option<UnsafeSlot<'a, D>> {
        if TypeId::of::<D>() == self.item_type_id() {
            // SAFETY: We know that the item is of type D, so we can safely cast it.
            let item = unsafe {
                &*(self.item as *const SyncUnsafeCell<T::AnyType> as *const SyncUnsafeCell<D>)
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

impl<'a, T: DynItem + ?Sized> Copy for UnsafeSlot<'a, T> {}

impl<'a, T: DynItem + ?Sized> Clone for UnsafeSlot<'a, T> {
    fn clone(&self) -> Self {
        Self {
            locality: self.locality,
            item: self.item,
            metadata: self.metadata,
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

// ********************* Internal Helpers

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

trait AnyCast: DynItem {
    fn anycast<T: AnyDynItem + ?Sized>(slot: UnsafeSlot<T>) -> Option<UnsafeSlot<Self>>;
}

impl<D: DynItem + ?Sized> AnyCast for D {
    default fn anycast<T: AnyDynItem + ?Sized>(slot: UnsafeSlot<T>) -> Option<UnsafeSlot<Self>> {
        // Since there is an impl for Item we know that if this gets called
        // That it's a AnyDynItem.
        let metadata = slot.metadata::<D>()?;

        // We know that T and D both have same types in AnyDynItem.
        Some(UnsafeSlot {
            metadata,
            locality: LocalityRef::new(
                slot.locality.path().any().assume(),
                cast_as_eq_type(slot.locality.data()),
                cast_as_eq_type(slot.locality.allocator()),
            ),
            item: cast_as_eq_type(slot.item),
        })
    }
}

impl<D: Item> AnyCast for D {
    fn anycast<T: AnyDynItem + ?Sized>(slot: UnsafeSlot<T>) -> Option<UnsafeSlot<Self>> {
        slot.downcast()
    }
}
