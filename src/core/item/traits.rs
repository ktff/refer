use std::{
    any::TypeId,
    marker::Unsize,
    ptr::{DynMetadata, Pointee},
};

use super::{AnyItem, DynItem, Item};

pub type ItemTraits<I = dyn AnyItem> = &'static [ItemTrait<I>];

/// Trait info about a specific trait implemented I.
#[derive(Debug)]
pub struct ItemTrait<I: DynItem + ?Sized = dyn AnyItem> {
    trait_id: TypeId,
    /// Actually DynMetadata<T> where T is the trait with id trait_id.
    metadata: DynMetadata<()>,
    _marker: std::marker::PhantomData<&'static I>,
}

impl<I: Item> ItemTrait<I> {
    pub const fn new<T: Pointee<Metadata = DynMetadata<T>> + ?Sized + 'static>() -> Self
    where
        I: Unsize<T>,
    {
        let metadata: DynMetadata<T> = std::ptr::metadata(std::ptr::null::<I>() as *const T);
        // SAFETY: For now this is safe
        let metadata = unsafe { std::mem::transmute::<DynMetadata<T>, DynMetadata<()>>(metadata) };
        Self {
            trait_id: TypeId::of::<T>(),
            metadata,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn any(self) -> ItemTrait<dyn AnyItem> {
        ItemTrait {
            trait_id: self.trait_id,
            metadata: self.metadata,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<I: DynItem + ?Sized> ItemTrait<I> {
    pub fn is(self, trait_it: TypeId) -> bool {
        self.trait_id == trait_it
    }

    pub fn downcast<T: Pointee<Metadata = DynMetadata<T>> + ?Sized + 'static>(
        self,
    ) -> Option<DynMetadata<T>> {
        if self.trait_id == TypeId::of::<T>() {
            // SAFETY: We've checked that the metadata is for T.
            let metadata =
                unsafe { std::mem::transmute::<DynMetadata<()>, DynMetadata<T>>(self.metadata) };
            Some(metadata)
        } else {
            None
        }
    }
}

// Impl Copy & Clone
impl<I: DynItem + ?Sized> Copy for ItemTrait<I> {}

impl<I: DynItem + ?Sized> Clone for ItemTrait<I> {
    fn clone(&self) -> Self {
        Self {
            trait_id: self.trait_id,
            metadata: self.metadata,
            _marker: std::marker::PhantomData,
        }
    }
}
