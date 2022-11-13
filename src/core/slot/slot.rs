use super::permit::{self, Permit, Split};
use crate::core::{AnyItem, AnySlot, Key, UnsafeSlot};
use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

pub struct Slot<
    'a,
    T: AnyItem,
    G: Any,
    S: crate::Shell<T = T>,
    A: std::alloc::Allocator + Any,
    R,
    W,
> {
    key: Key<T>,
    slot: UnsafeSlot<'a, T, G, S, A>,
    access: Permit<R, W>,
}

impl<'a, T: AnyItem, G: Any, S: crate::Shell<T = T>, A: std::alloc::Allocator + Any, R, W>
    Slot<'a, T, G, S, A, R, W>
{
    /// SAFETY: Caller must ensure that it has the correct access to the slot for the given 'a.
    pub unsafe fn new(key: Key<T>, slot: UnsafeSlot<'a, T, G, S, A>, access: Permit<R, W>) -> Self {
        Self { key, slot, access }
    }

    pub fn key(&self) -> Key<T> {
        self.key
    }

    pub fn alloc(&self) -> &A {
        self.slot.alloc()
    }

    pub fn group_item(&self) -> &G {
        self.slot.group_item()
    }

    pub fn upcast(self) -> AnySlot<'a, R, W> {
        // SAFETY: We have the same access to the slot.
        unsafe { AnySlot::new(self.key.into(), self.slot.upcast(), self.access) }
    }
}

impl<'a, T: AnyItem, G: Any, S: crate::Shell<T = T>, A: std::alloc::Allocator + Any, R>
    Slot<'a, T, G, S, A, R, permit::Slot>
{
    pub fn item(&self) -> &T {
        // SAFETY: We have at least read access to the item. R
        unsafe { &*self.slot.item().get() }
    }

    pub fn shell(&self) -> &S {
        // SAFETY: We have at least read access to the shell. R
        unsafe { &*self.slot.shell().get() }
    }
}

impl<'a, T: AnyItem, G: Any, S: crate::Shell<T = T>, A: std::alloc::Allocator + Any>
    Slot<'a, T, G, S, A, permit::Mut, permit::Slot>
{
    pub fn item_mut(&mut self) -> &mut T {
        // SAFETY: We have mut access to the item.
        unsafe { &mut *self.slot.item().get() }
    }

    pub fn shell_mut(&mut self) -> &mut S {
        // SAFETY: We have mut access to the shell.
        unsafe { &mut *self.slot.shell().get() }
    }

    pub fn split_mut(&mut self) -> Split<&mut T, &mut S> {
        // SAFETY: We have mut access to the item and shell.
        unsafe {
            Split {
                items: &mut *self.slot.item().get(),
                shells: &mut *self.slot.shell().get(),
            }
        }
    }

    pub fn split(
        self,
    ) -> Split<
        Slot<'a, T, G, S, A, permit::Mut, permit::Item>,
        Slot<'a, T, G, S, A, permit::Mut, permit::Shell>,
    > {
        self.access.split().map(
            |access| Slot {
                key: self.key,
                slot: self.slot,
                access,
            },
            |access| Slot {
                key: self.key,
                slot: self.slot,
                access,
            },
        )
    }
}

impl<'a, T: AnyItem, G: Any, S: crate::Shell<T = T>, A: std::alloc::Allocator + Any, R>
    Slot<'a, T, G, S, A, R, permit::Item>
{
    pub fn item(&self) -> &T {
        // SAFETY: We have at least read access to the item. R
        unsafe { &*self.slot.item().get() }
    }
}

impl<'a, T: AnyItem, G: Any, S: crate::Shell<T = T>, A: std::alloc::Allocator + Any>
    Slot<'a, T, G, S, A, permit::Mut, permit::Item>
{
    pub fn item_mut(&mut self) -> &mut T {
        // SAFETY: We have mut access to the item.
        unsafe { &mut *self.slot.item().get() }
    }
}

impl<'a, T: AnyItem, G: Any, S: crate::Shell<T = T>, A: std::alloc::Allocator + Any, R>
    Slot<'a, T, G, S, A, R, permit::Shell>
{
    pub fn shell(&self) -> &S {
        // SAFETY: We have at least read access to the shell. R
        unsafe { &*self.slot.shell().get() }
    }
}

impl<'a, T: AnyItem, G: Any, S: crate::Shell<T = T>, A: std::alloc::Allocator + Any>
    Slot<'a, T, G, S, A, permit::Mut, permit::Shell>
{
    pub fn shell_mut(&mut self) -> &mut S {
        // SAFETY: We have mut access to the shell.
        unsafe { &mut *self.slot.shell().get() }
    }
}

impl<'a, T: AnyItem, G: Any, S: crate::Shell<T = T>, A: std::alloc::Allocator + Any, R, W> Copy
    for Slot<'a, T, G, S, A, R, W>
where
    Permit<R, W>: Copy,
{
}

impl<'a, T: AnyItem, G: Any, S: crate::Shell<T = T>, A: std::alloc::Allocator + Any, R, W> Clone
    for Slot<'a, T, G, S, A, R, W>
where
    Permit<R, W>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            slot: self.slot,
            access: self.access.clone(),
        }
    }
}

impl<'a, T: AnyItem, G: Any, S: crate::Shell<T = T>, A: std::alloc::Allocator + Any, R> Deref
    for Slot<'a, T, G, S, A, R, permit::Item>
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item()
    }
}

impl<'a, T: AnyItem, G: Any, S: crate::Shell<T = T>, A: std::alloc::Allocator + Any> DerefMut
    for Slot<'a, T, G, S, A, permit::Mut, permit::Item>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item_mut()
    }
}

impl<'a, T: AnyItem, G: Any, S: crate::Shell<T = T>, A: std::alloc::Allocator + Any, R> Deref
    for Slot<'a, T, G, S, A, R, permit::Shell>
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        self.shell()
    }
}

impl<'a, T: AnyItem, G: Any, S: crate::Shell<T = T>, A: std::alloc::Allocator + Any> DerefMut
    for Slot<'a, T, G, S, A, permit::Mut, permit::Shell>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.shell_mut()
    }
}
