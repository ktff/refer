use super::{AnyItem, Item};
use std::{any::Any, cell::SyncUnsafeCell, ptr::Pointee};

pub type AnyAlloc = (dyn Any + Send + Sync + 'static);
pub type AnyLocalityData = (dyn Any + Send + Sync + 'static);

pub trait AnyDynItem =
    DynItem<AnyType = dyn AnyItem, AnyLocalityData = AnyLocalityData, AnyAlloc = AnyAlloc>;

/// Marker trait for dyn compliant traits of items.
pub trait DynItem: Any + Pointee {
    type AnyType: AnyItem + ?Sized;
    type AnyLocalityData: Any + Send + Sync + 'static + ?Sized;
    type AnyAlloc: Any + Send + Sync + 'static + ?Sized;

    fn as_any_type(
        a: &SyncUnsafeCell<Self::AnyType>,
    ) -> &SyncUnsafeCell<<dyn AnyItem as DynItem>::AnyType>;

    fn as_any_locality_data(
        a: &Self::AnyLocalityData,
    ) -> &<dyn AnyItem as DynItem>::AnyLocalityData;

    fn as_any_alloc(a: &Self::AnyAlloc) -> &<dyn AnyItem as DynItem>::AnyAlloc;
}

impl<T: Any + Pointee + ?Sized> DynItem for T {
    default type AnyType = <dyn AnyItem as DynItem>::AnyType;
    default type AnyLocalityData = <dyn AnyItem as DynItem>::AnyLocalityData;
    default type AnyAlloc = <dyn AnyItem as DynItem>::AnyAlloc;

    default fn as_any_type(
        a: &SyncUnsafeCell<Self::AnyType>,
    ) -> &SyncUnsafeCell<<dyn AnyItem as DynItem>::AnyType> {
        cast_as_eq_type(a)
    }

    default fn as_any_locality_data(
        a: &Self::AnyLocalityData,
    ) -> &<dyn AnyItem as DynItem>::AnyLocalityData {
        cast_as_eq_type(a)
    }

    default fn as_any_alloc(a: &Self::AnyAlloc) -> &<dyn AnyItem as DynItem>::AnyAlloc {
        cast_as_eq_type(a)
    }
}

impl DynItem for dyn AnyItem {
    type AnyType = dyn AnyItem;
    type AnyLocalityData = AnyLocalityData;
    type AnyAlloc = AnyAlloc;

    fn as_any_type(
        a: &SyncUnsafeCell<Self::AnyType>,
    ) -> &SyncUnsafeCell<<dyn AnyItem as DynItem>::AnyType> {
        a
    }

    fn as_any_locality_data(
        a: &Self::AnyLocalityData,
    ) -> &<dyn AnyItem as DynItem>::AnyLocalityData {
        a
    }

    fn as_any_alloc(a: &Self::AnyAlloc) -> &<dyn AnyItem as DynItem>::AnyAlloc {
        a
    }
}

impl<T: Item> DynItem for T {
    type AnyType = T;
    type AnyLocalityData = T::LocalityData;
    type AnyAlloc = T::Alloc;

    fn as_any_type(
        a: &SyncUnsafeCell<Self::AnyType>,
    ) -> &SyncUnsafeCell<<dyn AnyItem as DynItem>::AnyType> {
        a as _
    }

    fn as_any_locality_data(
        a: &Self::AnyLocalityData,
    ) -> &<dyn AnyItem as DynItem>::AnyLocalityData {
        a as _
    }

    fn as_any_alloc(a: &Self::AnyAlloc) -> &<dyn AnyItem as DynItem>::AnyAlloc {
        a as _
    }
}

/// Panics if not eq
pub fn cast_as_eq_type<T: ?Sized + 'static, D: ?Sized + 'static>(data: &T) -> &D {
    assert_eq!(std::any::TypeId::of::<T>(), std::any::TypeId::of::<D>());

    // SAFETY: We know that the item is of type D, so we can safely cast it.
    unsafe {
        let metadata: <D as Pointee>::Metadata =
            std::mem::transmute_copy(&std::ptr::metadata(data));
        &*std::ptr::from_raw_parts(data as *const _ as *const (), metadata)
    }
}
