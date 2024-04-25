use crate::core::{
    permit::{self, Permit},
    AnyDynItem, AnyItem, AnyUnsafeSlot, BiItem, DrainItem, DynItem, Found, Grc, Item, Key,
    MultiOwned, Owned, PartialEdge, Ptr, Ref, Side, StandaloneItem, UniversalItemLocality,
    UnsafeSlot,
};
use log::*;
use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

pub struct Slot<'a, R, T: DynItem + ?Sized = dyn AnyItem> {
    metadata: T::Metadata,
    slot: UnsafeSlot<'a, T>,
    access: R,
}

impl<'a, R> Slot<'a, R> {
    /// SAFETY: Caller must ensure that it has the correct access to the slot for the given 'a.    
    pub unsafe fn new_any(slot: AnyUnsafeSlot<'a>, access: R) -> Self {
        let metadata = std::ptr::metadata(slot.item().get());
        Self {
            metadata,
            slot,
            access,
        }
    }

    pub fn type_name(&self) -> &'static str {
        self.slot.item().get().type_info().name
    }
}

// impl<'a, T: DynItem + ?Sized, R> Slot<'a, R, T> {
//     /// Key should correspond to the slot.
//     /// None if item doesn't implement T.
//     /// SAFETY: Caller must ensure that it has the correct access to the slot for the given 'a.
//     pub unsafe fn new_dyn(slot: UnsafeSlot<'a, T>, access: R) -> Option<Self> {
//         if let Some(metadata) = slot.metadata::<T>() {
//             Some(Self {
//                 metadata,
//                 slot,
//                 access,
//             })
//         } else {
//             warn!(
//                 "Item at {:?}:{} is not {:?} which was assumed to be true.",
//                 slot.locality().path(),
//                 slot.item().get().type_info().name,
//                 std::any::type_name::<T>()
//             );
//             None
//         }
//     }
// }

impl<'a, T: AnyDynItem + ?Sized, R> Slot<'a, R, T> {
    // pub fn upcast<U: DynItem + ?Sized>(self) -> Slot<'a, R, U>
    // where
    //     T: Unsize<U>,
    // {
    //     // Upcast metadata
    //     let metadata = {
    //         let ptr = std::ptr::from_raw_parts::<T>(std::ptr::null(), self.metadata) as *const U;
    //         std::ptr::metadata(ptr)
    //     };

    //     Slot {
    //         metadata,
    //         slot: self.slot,
    //         access: self.access,
    //     }
    // }

    pub fn sidecast<U: AnyDynItem + ?Sized>(self) -> Result<Slot<'a, R, U>, Self> {
        if let Some(metadata) = self.slot.metadata::<U>() {
            Ok(Slot {
                metadata,
                slot: self.slot.sidecast(),
                access: self.access,
            })
        } else {
            Err(self)
        }
    }

    pub fn downcast<U: Item>(self) -> Result<Slot<'a, R, U>, Self> {
        if let Some(slot) = self.slot.downcast() {
            // SAFETY: We have the same access to the slot.
            Ok(unsafe { Slot::new(slot, self.access) })
        } else {
            Err(self)
        }
    }
}

impl<'a, T: Item, R> Slot<'a, R, T> {
    /// SAFETY: Caller must ensure that it has the correct access to the slot for the given 'a.
    pub unsafe fn new(slot: UnsafeSlot<'a, T>, access: R) -> Self {
        let metadata = std::ptr::metadata(slot.item().get());
        Self {
            metadata,
            slot,
            access,
        }
    }

    pub fn any(self) -> Slot<'a, R> {
        // SAFETY: We have the same access to the slot.
        unsafe { Slot::new_any(self.slot.any(), self.access) }
    }
}

impl<'a, T: DynItem + ?Sized, R> Slot<'a, R, T> {
    pub fn downgrade<F>(self) -> Slot<'a, F, T>
    where
        R: Into<F>,
    {
        Slot {
            access: self.access.into(),
            ..self
        }
    }

    pub fn key(&self) -> Key<Ref<'a>, T> {
        self.locality().path()
    }

    pub fn item_type_id(&self) -> std::any::TypeId {
        self.slot.item_type_id()
    }

    pub fn locality(&self) -> UniversalItemLocality<'a, T> {
        self.slot.locality()
    }
}

impl<'a, T: Item, R: Into<permit::Ref>> Slot<'a, R, T> {
    pub fn iter_drains(&self) -> impl Iterator<Item = Key<Ref<'_>>> + '_ {
        self.iter_edges(Some(crate::core::Side::Source))
            .map(|edge| edge.object)
    }

    pub fn iter_sources(&self) -> impl Iterator<Item = Key<Ref<'_>>> + '_ {
        self.iter_edges(Some(crate::core::Side::Drain))
            .map(|edge| edge.object)
    }

    /// Edges where self is side.
    pub fn iter_edges(&self, side: Option<Side>) -> T::Edges<'_> {
        self.item().iter_edges(self.locality(), side)
    }

    pub fn has_owners(&self) -> bool {
        self.item().any_has_owner(self.locality().any())
    }
}

impl<'a, T: Item> Slot<'a, permit::Ref, T> {
    pub fn to_item(&self) -> &'a T {
        // SAFETY: We have read access to the item for lifetime of 'a.
        unsafe { &*self.slot.item().get() }
    }
}

impl<'a, T: Item> Slot<'a, permit::Mut, T> {
    pub fn borrow(&self) -> Slot<permit::Ref, T> {
        // SAFETY: We have mut access to the item.
        unsafe { Slot::new(self.slot, self.access.borrow()) }
    }

    pub fn add_bi_edge<D, R, F: BiItem<R, T>>(
        &mut self,
        data: D,
        other_data: R,
        other: &mut Slot<permit::Mut, F>,
    ) where
        T: BiItem<D, F>,
    {
        // SAFETY: We are creating these keys in pair and adding them to their respective items.
        let (this_key, other_key) =
            unsafe { (self.locality().owned_key(), other.locality().owned_key()) };
        self.localized(|item, locality| item.add_bi_edge(locality, data, other_key));
        other.localized(|item, locality| item.add_bi_edge(locality, other_data, this_key));
    }

    pub fn try_remove_bi_edge<D, R, F: BiItem<R, T>>(
        &mut self,
        data: D,
        other_data: R,
        other: &mut Slot<permit::Mut, F>,
    ) -> bool
    where
        T: BiItem<D, F>,
    {
        let owned = self
            .localized(|item, locality| item.try_remove_bi_edge(locality, data, other.key().ptr()));
        if owned.is_some() {
            std::mem::forget(owned);
            let owned = other.localized(|item, locality| {
                item.try_remove_bi_edge(locality, other_data, self.key().ptr())
            });
            assert!(owned.is_some(), "BI edge should be present in both items");
            std::mem::forget(owned);
            true
        } else {
            false
        }
    }

    // /// Ok success.
    // /// Err if can't remove it.
    // #[must_use]
    // pub fn try_remove_edge<F: DynItem + ?Sized>(
    //     &mut self,
    //     this: Key<Owned, T>,
    //     edge: PartialEdge<Key<Ptr, F>>,
    // ) -> Result<Key<Owned, F>, (Found, Key<Owned, T>)> {
    //     self.localized(|item, locality| item.try_remove_edge(locality, this, edge))
    // }
}

impl<'a, T: DrainItem> Slot<'a, permit::Mut, T> {
    pub fn try_remove_drain_edge<F: DynItem + ?Sized>(
        &mut self,
        this: Key<Owned, T>,
        other: Key<Ptr, F>,
    ) -> Result<(), Key<Owned, T>> {
        if let Some(other) =
            self.localized(|item, locality| item.try_remove_drain_edge(locality, other))
        {
            std::mem::forget(other);
            std::mem::forget(this);
            Ok(())
        } else {
            Err(this)
        }
    }
}

impl<'a, T: StandaloneItem> Slot<'a, permit::Mut, T> {
    /// Caller should properly dispose of Grc once done with it.
    /// Proper disposal is:
    /// - Using it to construct an Item that will be added to a container.
    /// - Calling release() on Grc.
    pub fn own(&mut self) -> Grc<T> {
        self.localized(|item, locality| item.inc_owners(locality))
    }

    pub fn release(&mut self, grc: Grc<T>) {
        self.localized(|item, locality| item.dec_owners(locality, grc))
    }

    // #[must_use]
    // pub fn remove_edge<F: DynItem + ?Sized>(
    //     &mut self,
    //     this: Key<Owned, T>,
    //     edge: PartialEdge<Key<Ptr, F>>,
    // ) -> Key<Owned, F> {
    //     self.localized(|item, locality| item.remove_edge(locality, this, edge))
    // }
}

// impl<'a, T: Item, R: Into<permit::Ref>> Deref for Slot<'a, R, T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         self.item()
//     }
// }

// impl<'a, T: Item> DerefMut for Slot<'a, permit::Mut, T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         self.item_mut()
//     }
// }

impl<'a, R, T: Item> Eq for Slot<'a, R, T> {}

impl<'a, R, T: Item> PartialEq for Slot<'a, R, T> {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl<'a, R, T: Item> std::hash::Hash for Slot<'a, R, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().hash(state)
    }
}

impl<'a, T: DynItem + ?Sized, R: Into<permit::Ref>> Slot<'a, R, T> {
    pub fn item(&self) -> &T {
        unsafe {
            let ptr = self.slot.item().get();

            // SAFETY: During construction we checked that the type of the item matches the type of the key.
            let ptr = std::ptr::from_raw_parts(ptr as *const (), self.metadata);

            // SAFETY: We have at least read access to the item. R
            &*ptr
        }
    }
}

impl<'a, T: DynItem + ?Sized> Slot<'a, permit::Mut, T> {
    pub fn item_mut(&mut self) -> &mut T {
        unsafe {
            let ptr = self.slot.item().get();

            // SAFETY: During construction we checked that the type of the item matches the type of the key.
            let ptr = std::ptr::from_raw_parts_mut(ptr as *mut (), self.metadata);

            // SAFETY: We have mut access to the item.
            &mut *ptr
        }
    }

    pub fn any_item_mut(&mut self) -> &mut dyn AnyItem {
        unsafe {
            let ptr = self.slot.item_as_any().get();

            // SAFETY: We have mut access to the item.
            &mut *ptr
        }
    }

    pub fn any_localized<R>(
        &mut self,
        func: impl FnOnce(&mut dyn AnyItem, UniversalItemLocality<'a>) -> R,
    ) -> R {
        let locality = self.locality().any_universal();
        func(self.any_item_mut(), locality)
    }

    pub fn item_mut_downcast<U: Item>(&mut self) -> Option<&mut U> {
        (self.any_item_mut() as &mut dyn Any).downcast_mut::<U>()
    }

    pub fn localized<R>(
        &mut self,
        func: impl FnOnce(&mut T, UniversalItemLocality<'a, T>) -> R,
    ) -> R {
        let locality = self.locality();
        func(self.item_mut(), locality)
    }

    /// Ok success.
    /// Err if can't remove it.
    #[must_use]
    pub fn remove_edges<F: DynItem + ?Sized>(
        &mut self,
        this: Key<Owned, T>,
        edge: PartialEdge<Key<Ptr, F>>,
    ) -> Result<MultiOwned<F>, (Found, Key<Owned, T>)> {
        match self
            .any_localized(|item, locality| {
                item.any_remove_edges(locality, edge.map(|key| key.any()))
            })
            .map(MultiOwned::assume)
        {
            Ok(owned) => {
                std::mem::forget(this);
                Ok(owned)
            }
            Err(present) => Err((present, this)),
        }
    }
}

impl<'a, R: Into<permit::Ref>, T: AnyDynItem + ?Sized> Slot<'a, R, T> {
    pub fn any_item(&self) -> &dyn AnyItem {
        unsafe {
            let ptr = self.slot.item().get();

            // SAFETY: We have at least read access to the item. R
            &*ptr
        }
    }

    pub fn item_downcast<D: Item>(&self) -> Option<&D> {
        (self.any_item() as &dyn Any).downcast_ref::<D>()
    }

    pub fn edges_dyn(
        &self,
        side: Option<Side>,
    ) -> impl Iterator<Item = PartialEdge<Key<Ref<'_>>>> + '_ {
        self.any_item()
            .any_iter_edges(self.locality().any_universal(), side)
            .into_iter()
            .flatten()
    }

    pub fn drains_dyn(&self) -> impl Iterator<Item = Key<Ref<'_>>> + '_ {
        self.edges_dyn(Some(Side::Source)).map(|edge| edge.object)
    }

    pub fn sources_dyn(&self) -> impl Iterator<Item = Key<Ref<'_>>> + '_ {
        self.edges_dyn(Some(Side::Drain)).map(|edge| edge.object)
    }
}

impl<'a, T: AnyDynItem + ?Sized> Slot<'a, permit::Mut, T> {
    /// Caller should properly dispose of Grc once done with it.
    /// Proper disposal is:
    /// - Using it to construct an Item that will be added to a container.
    /// - Calling release() on Grc.
    ///
    /// None if the item doesn't support ownership.
    pub fn own_dyn(&mut self) -> Option<Grc<T>> {
        self.any_localized(|item, locality| item.any_inc_owners(locality).map(|grc| grc.assume()))
    }

    pub fn release_dyn(&mut self, grc: Grc<T>) {
        self.any_localized(|item, locality| item.any_dec_owners(locality, grc.any()))
    }
}

impl<'a, T: DynItem + ?Sized, R> Copy for Slot<'a, R, T> where R: Copy {}

impl<'a, T: DynItem + ?Sized, R> Clone for Slot<'a, R, T>
where
    R: Clone,
{
    fn clone(&self) -> Self {
        Self {
            metadata: self.metadata,
            slot: self.slot,
            access: self.access.clone(),
        }
    }
}

impl<'a, T: DynItem + ?Sized, R: Into<permit::Ref>> Deref for Slot<'a, R, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item()
    }
}

impl<'a, T: DynItem + ?Sized> DerefMut for Slot<'a, permit::Mut, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item_mut()
    }
}
