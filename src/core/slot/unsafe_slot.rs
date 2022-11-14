use super::*;
use crate::core::{AnyItem, Shell};
use getset::CopyGetters;
use std::{any::Any, cell::SyncUnsafeCell};

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct UnsafeSlot<'a, T: AnyItem, G: Any, S: Shell<T = T>, A: std::alloc::Allocator + Any> {
    item: &'a SyncUnsafeCell<T>,
    group_item: &'a G,
    shell: &'a SyncUnsafeCell<S>,
    alloc: &'a A,
}

impl<'a, T: AnyItem, G: Any, S: Shell<T = T>, A: std::alloc::Allocator + Any>
    UnsafeSlot<'a, T, G, S, A>
{
    pub fn new(
        item: &'a SyncUnsafeCell<T>,
        group_item: &'a G,
        shell: &'a SyncUnsafeCell<S>,
        alloc: &'a A,
    ) -> Self {
        Self {
            item,
            group_item,
            shell,
            alloc,
        }
    }

    pub fn upcast(self) -> AnyUnsafeSlot<'a> {
        AnyUnsafeSlot::new(
            self.item,
            self.group_item,
            self.shell,
            self.alloc,
            self.alloc,
        )
    }
}

impl<'a, T: AnyItem, S: Shell<T = T>, A: std::alloc::Allocator + Any> UnsafeSlot<'a, T, (), S, A> {
    pub fn with_group_item<G: Any>(self, group_item: &'a G) -> UnsafeSlot<'a, T, G, S, A> {
        let Self {
            item,
            group_item: _,
            shell,
            alloc,
        } = self;
        UnsafeSlot {
            item,
            group_item,
            shell,
            alloc,
        }
    }
}

impl<'a, T: AnyItem, G: Any, S: Shell<T = T>, A: std::alloc::Allocator + Any> Copy
    for UnsafeSlot<'a, T, G, S, A>
{
}

impl<'a, T: AnyItem, G: Any, S: Shell<T = T>, A: std::alloc::Allocator + Any> Clone
    for UnsafeSlot<'a, T, G, S, A>
{
    fn clone(&self) -> Self {
        Self {
            item: self.item,
            group_item: self.group_item,
            shell: self.shell,
            alloc: self.alloc,
        }
    }
}
