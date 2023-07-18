use crate::core::{
    permit::{self, Permit},
    AnyItem, DynItem, DynSlot, Found, Grc, Item, ItemLocality, Key, Owned, PartialEdge, Ptr, Ref,
    Side, StandaloneItem, UnsafeSlot,
};
use std::ops::{Deref, DerefMut};

pub struct Slot<'a, R, T: Item> {
    slot: UnsafeSlot<'a, T>,
    access: Permit<R>,
}

impl<'a, T: Item, R> Slot<'a, R, T> {
    /// SAFETY: Caller must ensure that it has the correct access to the slot for the given 'a.
    pub unsafe fn new(slot: UnsafeSlot<'a, T>, access: Permit<R>) -> Self {
        Self { slot, access }
    }

    pub fn key(&self) -> Key<Ref<'a>, T> {
        self.locality().path()
    }

    pub fn locality(&self) -> ItemLocality<'a, T> {
        self.slot.locality()
    }

    pub fn upcast(self) -> DynSlot<'a, R> {
        // SAFETY: We have the same access to the slot.
        unsafe { DynSlot::new_any(self.slot.any(), self.access) }
    }

    pub fn downgrade<F>(self) -> Slot<'a, F, T>
    where
        Permit<R>: Into<Permit<F>>,
    {
        Slot {
            slot: self.slot,
            access: self.access.into(),
        }
    }
}

impl<'a, T: Item, R: Into<permit::Ref>> Slot<'a, R, T> {
    pub fn item(&self) -> &T {
        // SAFETY: We have at least read access to the item. R
        unsafe { &*self.slot.item().get() }
    }

    pub fn drains(&self) -> impl Iterator<Item = Key<Ref<'_>>> + '_ {
        self.edges(Some(crate::core::Side::Source))
            .map(|edge| edge.object)
    }

    pub fn sources(&self) -> impl Iterator<Item = Key<Ref<'_>>> + '_ {
        self.edges(Some(crate::core::Side::Drain))
            .map(|edge| edge.object)
    }

    /// Edges where self is side.
    pub fn edges(&self, side: Option<Side>) -> T::Edges<'_> {
        self.item().edges(self.locality(), side)
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

    pub fn item_mut(&mut self) -> &mut T {
        // SAFETY: We have mut access to the item.
        unsafe { &mut *self.slot.item().get() }
    }

    pub fn localized<R>(&mut self, func: impl FnOnce(&mut T, ItemLocality<T>) -> R) -> R {
        let locality = self.locality();
        func(self.item_mut(), locality)
    }

    /// Ok success.
    /// Err if can't remove it.
    #[must_use]
    pub fn try_remove_edge<F: DynItem + ?Sized>(
        &mut self,
        this: Key<Owned, T>,
        edge: PartialEdge<Key<Ptr, F>>,
    ) -> Result<Key<Owned, F>, (Found, Key<Owned, T>)> {
        self.localized(|item, locality| item.try_remove_edge(locality, this, edge))
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

    #[must_use]
    pub fn remove_edge<F: DynItem + ?Sized>(
        &mut self,
        this: Key<Owned, T>,
        edge: PartialEdge<Key<Ptr, F>>,
    ) -> Key<Owned, F> {
        self.localized(|item, locality| item.remove_edge(locality, this, edge))
    }
}

impl<'a, T: Item, R> Copy for Slot<'a, R, T> where Permit<R>: Copy {}

impl<'a, T: Item, R> Clone for Slot<'a, R, T>
where
    Permit<R>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            slot: self.slot,
            access: self.access.clone(),
        }
    }
}

impl<'a, T: Item, R: Into<permit::Ref>> Deref for Slot<'a, R, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item()
    }
}

impl<'a, T: Item> DerefMut for Slot<'a, permit::Mut, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item_mut()
    }
}
