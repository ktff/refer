use super::{permit, Access, Split};
use crate::core::{AnyItem, Key, UnsafeSlot};
use std::any::Any;

pub struct Slot<
    'a,
    T: AnyItem,
    G: Any,
    S: crate::Shell<T = T>,
    A: std::alloc::Allocator + Any,
    R,
    W,
> {
    pub(super) key: Key<T>,
    pub(super) slot: UnsafeSlot<'a, T, G, S, A>,
    pub(super) access: Access<R, W>,
}

impl<'a, T: AnyItem, G: Any, S: crate::Shell<T = T>, A: std::alloc::Allocator + Any, R, W>
    Slot<'a, T, G, S, A, R, W>
{
    pub fn key(&self) -> Key<T> {
        self.key
    }

    pub fn alloc(&self) -> &A {
        self.slot.alloc()
    }

    pub fn group_item(&self) -> &G {
        self.slot.group_item()
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
                item: &mut *self.slot.item().get(),
                shell: &mut *self.slot.shell().get(),
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
