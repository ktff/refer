use crate::core::{
    permit, AnyItem, AnyItemLocality, AnyUnsafeSlot, DynItem, Item, Key, Owned, PartialEdge,
    Permit, Ptr, Ref, ReferError, Side, TypeInfo,
};

use std::{
    any::{Any, TypeId},
    marker::Unsize,
    ops::{Deref, DerefMut},
};

pub type AnySlot<'a, R> = DynSlot<'a, dyn AnyItem, R>;

pub struct DynSlot<'a, T: DynItem + ?Sized, R> {
    metadata: T::Metadata,
    slot: AnyUnsafeSlot<'a>,
    access: Permit<R>,
}

impl<'a, R> AnySlot<'a, R> {
    /// SAFETY: Caller must ensure that it has the correct access to the slot for the given 'a.    
    pub unsafe fn new_any(slot: AnyUnsafeSlot<'a>, access: Permit<R>) -> Self {
        let metadata = std::ptr::metadata(slot.item().get());
        Self {
            metadata,
            slot,
            access,
        }
    }
}

impl<'a, T: DynItem + ?Sized, R> DynSlot<'a, T, R> {
    /// Key should correspond to the slot.
    /// Err if item doesn't implement T.
    /// SAFETY: Caller must ensure that it has the correct access to the slot for the given 'a.    
    pub unsafe fn new(slot: AnyUnsafeSlot<'a>, access: Permit<R>) -> Result<Self, ReferError> {
        let metadata = slot
            .metadata::<T>()
            .ok_or_else(|| ReferError::InvalidCastType {
                expected: TypeInfo::of::<T>(),
                found: slot.item().get().type_info(),
                index: slot.locality().path().index(),
            })?;

        Ok(Self {
            metadata,
            slot,
            access,
        })
    }

    pub fn key(&self) -> Key<Ref<'a>, T> {
        self.locality().path().assume()
    }

    pub fn item_type_id(&self) -> std::any::TypeId {
        self.slot.item_type_id()
    }

    pub fn locality(&self) -> AnyItemLocality<'a> {
        self.slot.locality()
    }

    pub fn any(self) -> AnySlot<'a, R> {
        // SAFETY: We have the same access to the slot.
        unsafe { AnySlot::new_any(self.slot, self.access) }
    }

    pub fn upcast<U: DynItem + ?Sized>(self) -> DynSlot<'a, U, R>
    where
        T: Unsize<U>,
    {
        // Upcast metadata
        let metadata = {
            let ptr = std::ptr::from_raw_parts::<T>(std::ptr::null(), self.metadata) as *const U;
            std::ptr::metadata(ptr)
        };

        DynSlot {
            metadata,
            slot: self.slot,
            access: self.access,
        }
    }

    pub fn sidecast<U: DynItem + ?Sized>(self) -> Result<DynSlot<'a, U, R>, Self> {
        if let Some(metadata) = self.slot.metadata::<U>() {
            Ok(DynSlot {
                metadata,
                slot: self.slot,
                access: self.access,
            })
        } else {
            Err(self)
        }
    }

    pub fn downcast<U: Item>(self) -> Result<DynSlot<'a, U, R>, Self> {
        if TypeId::of::<U>() == self.item_type_id() {
            Ok(DynSlot {
                metadata: (),
                slot: self.slot,
                access: self.access,
            })
        } else {
            Err(self)
        }
    }

    pub fn downgrade<F>(self) -> DynSlot<'a, T, F>
    where
        Permit<R>: Into<Permit<F>>,
    {
        DynSlot {
            metadata: self.metadata,
            slot: self.slot,
            access: self.access.into(),
        }
    }
}

impl<'a, T: DynItem + ?Sized, R: Into<permit::Ref>> DynSlot<'a, T, R> {
    pub fn any_item(&self) -> &dyn AnyItem {
        unsafe {
            let ptr = self.slot.item().get();

            // SAFETY: We have at least read access to the item. R
            &*ptr
        }
    }

    pub fn item(&self) -> &T {
        unsafe {
            let ptr = self.slot.item().get();

            // SAFETY: During construction we checked that the type of the item matches the type of the key.
            let ptr = std::ptr::from_raw_parts(ptr as *const (), self.metadata);

            // SAFETY: We have at least read access to the item. R
            &*ptr
        }
    }

    pub fn item_downcast<U: Item>(&self) -> Option<&U> {
        (self.any_item() as &dyn Any).downcast_ref::<U>()
    }

    // TODO: rename
    pub fn iter_references_any(&self) -> impl Iterator<Item = Key<Ref<'_>>> + '_ {
        self.any_item()
            .edges_any(self.locality(), Some(Side::Source))
            .into_iter()
            .flatten()
            .map(|PartialEdge { object, .. }| object)
    }

    // TODO: This lifetime is best for mut, but for ref it's possible to extend to 'a.
    pub fn edges(
        &self,
        side: Option<Side>,
    ) -> impl Iterator<Item = PartialEdge<Key<Ref<'_>>>> + '_ {
        self.any_item()
            .edges_any(self.locality(), side)
            .into_iter()
            .flatten()
    }

    // /// Can panic if locality isn't for this type.
    // pub fn duplicate(&self, to: AnySlotLocality) -> Option<Box<dyn AnyItem>> {
    //     let locality = self.locality();
    //     self.any_item().duplicate_any(locality, to)
    // }
}

impl<'a, T: DynItem + ?Sized> DynSlot<'a, T, permit::Mut> {
    pub fn any_item_mut(&mut self) -> &mut dyn AnyItem {
        unsafe {
            let ptr = self.slot.item().get();

            // SAFETY: We have mut access to the item.
            &mut *ptr
        }
    }

    pub fn item_mut(&mut self) -> &mut T {
        unsafe {
            let ptr = self.slot.item().get();

            // SAFETY: During construction we checked that the type of the item matches the type of the key.
            let ptr = std::ptr::from_raw_parts_mut(ptr as *mut (), self.metadata);

            // SAFETY: We have mut access to the item.
            &mut *ptr
        }
    }

    pub fn item_mut_downcast<U: Item>(&mut self) -> Option<&mut U> {
        (self.any_item_mut() as &mut dyn Any).downcast_mut::<U>()
    }

    pub fn localized<R>(&mut self, func: impl FnOnce(&mut T, AnyItemLocality) -> R) -> R {
        let locality = self.locality();
        func(self.item_mut(), locality)
    }

    pub fn any_localized<R>(
        &mut self,
        func: impl FnOnce(&mut dyn AnyItem, AnyItemLocality) -> R,
    ) -> R {
        let locality = self.locality();
        func(self.any_item_mut(), locality)
    }

    /// Ok with key to self.
    /// Err with provided source.
    /// Err if self isn't drain item so it wasn't added.
    #[must_use]
    pub fn add_drain_edge<F: DynItem + ?Sized>(
        &mut self,
        source: Key<Owned, F>,
    ) -> Result<Key<Owned>, Key<Owned, F>> {
        self.any_localized(|item, locality| item.add_drain_edge_any(locality, source.any()))
            .map_err(|source| source.assume())
    }

    /// Ok success.
    /// Err if can't remove it.
    #[must_use]
    pub fn remove_edge<F: DynItem + ?Sized>(
        &mut self,
        this: Key<Owned, T>,
        edge: PartialEdge<Key<Ptr, F>>,
    ) -> Result<Key<Owned, F>, Key<Owned, T>> {
        self.any_localized(|item, locality| {
            item.remove_edge_any(locality, this.any(), edge.map(|key| key.any()))
        })
        .map(Key::assume)
        .map_err(Key::assume)
    }

    pub fn any_delete_ref(&mut self, this: Key<Owned>) {
        assert_eq!(self.key(), this, "Provided key isn't of this slot");
        self.any_localized(|item, locality| item.any_dec_owners(locality, this))
    }

    // pub fn remove_reference<F: DynItem + ?Sized>(&mut self, other: Key<F>) -> bool {
    //     let locality = self.locality();
    //     self.any_item_mut()
    //         .remove_reference_any(locality, other.any())
    // }

    // pub fn replace_reference<F: DynItem + ?Sized>(&mut self, other: Key<F>, to: Key<F>) {
    //     let locality = self.locality();
    //     self.any_item_mut()
    //         .replace_reference_any(locality, other.any(), to.any());
    // }

    // pub fn displace_reference<F: DynItem + ?Sized>(
    //     &mut self,
    //     other: Key<F>,
    //     to: Key<F>,
    // ) -> Option<Path> {
    //     let locality = self.locality();
    //     self.any_item_mut()
    //         .displace_reference_any(locality, other.any(), to.any())
    // }

    // pub fn duplicate_reference<F: DynItem + ?Sized>(
    //     &mut self,
    //     other: Key<F>,
    //     to: Key<F>,
    // ) -> Option<Path> {
    //     let locality = self.locality();
    //     self.any_item_mut()
    //         .duplicate_reference_any(locality, other.any(), to.any())
    // }

    // pub fn displace(&mut self) {
    //     let locality = self.locality();
    //     self.any_item_mut().displace_any(locality, None);
    // }
}

impl<'a, T: DynItem + ?Sized, R> Copy for DynSlot<'a, T, R> where Permit<R>: Copy {}

impl<'a, T: DynItem + ?Sized, R> Clone for DynSlot<'a, T, R>
where
    Permit<R>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            metadata: self.metadata,
            slot: self.slot,
            access: self.access.clone(),
        }
    }
}

impl<'a, T: DynItem + ?Sized, R: Into<permit::Ref>> Deref for DynSlot<'a, T, R> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item()
    }
}

impl<'a, T: DynItem + ?Sized> DerefMut for DynSlot<'a, T, permit::Mut> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item_mut()
    }
}
