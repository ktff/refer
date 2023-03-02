use crate::core::{
    permit::{self, SubjectPermit},
    AnyContainer, AnyItem, AnySlot, Item, Key, Permit, Ptr, Ref, Side, SlotLocality, UnsafeSlot,
};
use std::ops::{Deref, DerefMut};

use super::DynSlot;

// TODO: Swap places of T & R in Slots. So that it looks like Slot<'a,Ref,Item>
pub struct Slot<'a, T: Item, R> {
    // TODO: Use Ref
    key: Key<Ptr, T>,
    slot: UnsafeSlot<'a, T>,
    access: Permit<R>,
}

impl<'a, T: Item, R> Slot<'a, T, R> {
    /// SAFETY: Caller must ensure that it has the correct access to the slot for the given 'a.
    pub unsafe fn new(key: Key<Ptr, T>, slot: UnsafeSlot<'a, T>, access: Permit<R>) -> Self {
        debug_assert!(slot.prefix().contains_key(key));
        Self { key, slot, access }
    }

    pub fn key(&self) -> Key<Ptr, T> {
        self.key
    }

    pub fn locality(&self) -> SlotLocality<'a, T> {
        self.slot.locality()
    }

    pub fn upcast(self) -> AnySlot<'a, R> {
        // SAFETY: We have the same access to the slot.
        unsafe { AnySlot::new_any(self.key.upcast(), self.slot.upcast(), self.access) }
    }

    // TODO: Is this needed any more?
    pub fn downgrade<F>(self) -> Slot<'a, T, F>
    where
        Permit<R>: Into<Permit<F>>,
    {
        Slot {
            key: self.key,
            slot: self.slot,
            access: self.access.into(),
        }
    }
}

impl<'a, T: Item, R: Into<permit::Ref>> Slot<'a, T, R> {
    pub fn item(&self) -> &T {
        // SAFETY: We have at least read access to the item. R
        unsafe { &*self.slot.item().get() }
    }

    // TODO: rename
    pub fn iter_references(&self) -> impl Iterator<Item = Key<Ref<'_>>> + '_ {
        self.item()
            .edges(self.locality(), Some(crate::core::Side::Source))
            .map(|edge| edge.object)
    }

    pub fn edges(&self, side: Option<Side>) -> T::Edges<'_> {
        self.item().edges(self.locality(), side)
    }

    // /// Can panic if locality isn't for this type.
    // pub fn duplicate(&self, to: SlotLocality<T>) -> Option<T> {
    //     let locality = self.locality();
    //     self.item().duplicate(locality, to)
    // }
}

impl<'a, T: Item> Slot<'a, T, permit::Ref> {
    // TODO: Should this replace item() ?
    pub fn to_item(&self) -> &'a T {
        // SAFETY: We have read access to the item for lifetime of 'a.
        unsafe { &*self.slot.item().get() }
    }
}

impl<'a, T: Item> Slot<'a, T, permit::Mut> {
    pub fn borrow(&self) -> Slot<T, permit::Ref> {
        // SAFETY: We have mut access to the item.
        unsafe { Slot::new(self.key, self.slot, self.access.borrow()) }
    }

    pub fn item_mut(&mut self) -> &mut T {
        // SAFETY: We have mut access to the item.
        unsafe { &mut *self.slot.item().get() }
    }

    pub fn localized<R>(&mut self, func: impl FnOnce(&mut T, SlotLocality<T>) -> R) -> R {
        let locality = self.locality();
        func(self.item_mut(), locality)
    }

    // pub fn replace_reference(&mut self, other: Key, to: Key) {
    //     let locality = self.locality();
    //     self.item_mut().replace_reference(locality, other, to);
    // }

    // pub fn displace_reference(&mut self, other: Key, to: Key) -> Option<Path> {
    //     let locality = self.locality();
    //     self.item_mut().displace_reference(locality, other, to)
    // }

    // pub fn duplicate_reference(&mut self, other: Key, to: Key) -> Option<Path> {
    //     let locality = self.locality();
    //     self.item_mut().duplicate_reference(locality, other, to)
    // }

    // pub fn displace(&mut self) {
    //     let locality = self.locality();
    //     self.item_mut().displace(locality, None)
    // }
}

impl<'a, T: Item, R> Copy for Slot<'a, T, R> where Permit<R>: Copy {}

impl<'a, T: Item, R> Clone for Slot<'a, T, R>
where
    Permit<R>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            slot: self.slot,
            access: self.access.clone(),
        }
    }
}

impl<'a, T: Item, R: Into<permit::Ref>> Deref for Slot<'a, T, R> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item()
    }
}

impl<'a, T: Item> DerefMut for Slot<'a, T, permit::Mut> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item_mut()
    }
}
