use crate::core::{
    permit, AnyKey, AnyShell, AnySlot, Item, Key, Path, Permit, Shell, SlotLocality, UnsafeSlot,
};
use std::ops::{Deref, DerefMut};

pub struct Slot<'a, T: Item, S: Shell<T = T>, R, A> {
    key: Key<T>,
    slot: UnsafeSlot<'a, T, S>,
    access: Permit<R, A>,
}

impl<'a, T: Item, S: Shell<T = T>, R, A> Slot<'a, T, S, R, A> {
    /// SAFETY: Caller must ensure that it has the correct access to the slot for the given 'a.
    pub unsafe fn new(key: Key<T>, slot: UnsafeSlot<'a, T, S>, access: Permit<R, A>) -> Self {
        debug_assert!(slot.prefix().contains_key(key));
        Self { key, slot, access }
    }

    pub fn key(&self) -> Key<T> {
        self.key
    }

    pub fn locality(&self) -> SlotLocality<'a, T> {
        self.slot.locality()
    }

    pub fn upcast(self) -> AnySlot<'a, R, A> {
        // SAFETY: We have the same access to the slot.
        unsafe { AnySlot::new_any(self.key.upcast(), self.slot.upcast(), self.access) }
    }

    pub fn downgrade<F, B>(self) -> Slot<'a, T, S, F, B>
    where
        Permit<R, A>: Into<Permit<F, B>>,
    {
        Slot {
            key: self.key,
            slot: self.slot,
            access: self.access.into(),
        }
    }
}

impl<'a, T: Item, S: Shell<T = T>, R: Into<permit::Ref>, A: Into<permit::Item>>
    Slot<'a, T, S, R, A>
{
    pub fn item(&self) -> &T {
        // SAFETY: We have at least read access to the item. R
        unsafe { &*self.slot.item().get() }
    }

    pub fn iter_references(&self) -> T::Iter<'_> {
        self.item().iter_references(self.locality())
    }

    /// Can panic if locality isn't for this type.
    pub fn duplicate(&self, to: SlotLocality<T>) -> Option<T> {
        let locality = self.locality();
        self.item().duplicate(locality, to)
    }
}

impl<'a, T: Item, S: Shell<T = T>, A: Into<permit::Item>> Slot<'a, T, S, permit::Ref, A> {
    pub fn to_item(&self) -> &'a T {
        // SAFETY: We have read access to the item for lifetime of 'a.
        unsafe { &*self.slot.item().get() }
    }
}

impl<'a, T: Item, S: Shell<T = T>, A: Into<permit::Item>> Slot<'a, T, S, permit::Mut, A> {
    pub fn item_mut(&mut self) -> &mut T {
        // SAFETY: We have mut access to the item.
        unsafe { &mut *self.slot.item().get() }
    }

    pub fn replace_reference(&mut self, other: AnyKey, to: AnyKey) {
        let locality = self.locality();
        self.item_mut().replace_reference(locality, other, to);
    }

    pub fn displace_reference(&mut self, other: AnyKey, to: AnyKey) -> Option<Path> {
        let locality = self.locality();
        self.item_mut().displace_reference(locality, other, to)
    }

    pub fn duplicate_reference(&mut self, other: AnyKey, to: AnyKey) -> Option<Path> {
        let locality = self.locality();
        self.item_mut().duplicate_reference(locality, other, to)
    }

    pub fn displace(&mut self) {
        let locality = self.locality();
        self.item_mut().displace(locality, None)
    }
}

impl<'a, T: Item, S: Shell<T = T>, R: Into<permit::Ref>, A: Into<permit::Shell>>
    Slot<'a, T, S, R, A>
{
    pub fn shell(&self) -> &S {
        // SAFETY: We have at least read access to the shell. R
        unsafe { &*self.slot.shell().get() }
    }
}

impl<'a, T: Item, S: Shell<T = T>, A: Into<permit::Shell>> Slot<'a, T, S, permit::Ref, A> {
    pub fn to_shell(&self) -> &'a S {
        // SAFETY: We have read access to the shell for 'a lifetime.
        unsafe { &*self.slot.shell().get() }
    }
}

impl<'a, T: Item, S: Shell<T = T>, A: Into<permit::Shell>> Slot<'a, T, S, permit::Mut, A> {
    pub fn shell_mut(&mut self) -> &mut S {
        // SAFETY: We have mut access to the shell.
        unsafe { &mut *self.slot.shell().get() }
    }

    pub fn shell_add(&mut self, from: AnyKey) {
        let alloc = self.slot.allocator();
        self.shell_mut().add(from, alloc);
    }

    pub fn shell_add_many(&mut self, from: AnyKey, count: usize) {
        let locality = self.locality();
        self.shell_mut()
            .add_many_any(from, count, locality.upcast());
    }

    pub fn shell_replace(&mut self, from: AnyKey, to: AnyKey) {
        let alloc = self.slot.allocator();
        self.shell_mut().replace(from, to, alloc);
    }

    pub fn shell_remove(&mut self, from: AnyKey) {
        self.shell_mut().remove(from);
    }
}

impl<'a, T: Item, S: Shell<T = T>> Slot<'a, T, S, permit::Mut, permit::Slot> {
    pub fn split(&mut self) -> (&mut T, &mut S) {
        // SAFETY: We have mut access to the item and shell.
        unsafe { (&mut *self.slot.item().get(), &mut *self.slot.shell().get()) }
    }

    pub fn split_slot(
        self,
    ) -> (
        Slot<'a, T, S, permit::Mut, permit::Item>,
        Slot<'a, T, S, permit::Mut, permit::Shell>,
    ) {
        let (item_access, shell_access) = self.access.split();

        (
            Slot {
                key: self.key,
                slot: self.slot,
                access: item_access,
            },
            Slot {
                key: self.key,
                slot: self.slot,
                access: shell_access,
            },
        )
    }
}

impl<'a, T: Item, S: Shell<T = T>, A> Slot<'a, T, S, permit::Mut, A> {
    pub fn borrow(&self) -> Slot<T, S, permit::Ref, A> {
        // SAFETY: We have mut access to the item.
        unsafe { Slot::new(self.key, self.slot, self.access.borrow()) }
    }
}

impl<'a, T: Item, S: Shell<T = T>, R, A> Copy for Slot<'a, T, S, R, A> where Permit<R, A>: Copy {}

impl<'a, T: Item, S: Shell<T = T>, R, A> Clone for Slot<'a, T, S, R, A>
where
    Permit<R, A>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            slot: self.slot,
            access: self.access.clone(),
        }
    }
}

impl<'a, T: Item, S: Shell<T = T>, R: Into<permit::Ref>> Deref for Slot<'a, T, S, R, permit::Item> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item()
    }
}

impl<'a, T: Item, S: Shell<T = T>> DerefMut for Slot<'a, T, S, permit::Mut, permit::Item> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item_mut()
    }
}

impl<'a, T: Item, S: Shell<T = T>, R: Into<permit::Ref>> Deref for Slot<'a, T, S, R, permit::Slot> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item()
    }
}

impl<'a, T: Item, S: Shell<T = T>> DerefMut for Slot<'a, T, S, permit::Mut, permit::Slot> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item_mut()
    }
}

impl<'a, T: Item, S: Shell<T = T>, R: Into<permit::Ref>> Deref
    for Slot<'a, T, S, R, permit::Shell>
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        self.shell()
    }
}

impl<'a, T: Item, S: Shell<T = T>> DerefMut for Slot<'a, T, S, permit::Mut, permit::Shell> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.shell_mut()
    }
}
