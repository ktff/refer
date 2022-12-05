use crate::core::{AnyItem, AnyShell};
use getset::CopyGetters;
use std::{any::Any, cell::SyncUnsafeCell};

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct AnyUnsafeSlot<'a> {
    item: &'a SyncUnsafeCell<dyn AnyItem>,
    group_item: &'a dyn Any,
    shell: &'a SyncUnsafeCell<dyn AnyShell>,
    alloc: &'a dyn std::alloc::Allocator,
    #[getset(skip)]
    alloc_any: &'a dyn Any,
}

impl<'a> AnyUnsafeSlot<'a> {
    pub fn new(
        item: &'a SyncUnsafeCell<dyn AnyItem>,
        group_item: &'a dyn Any,
        shell: &'a SyncUnsafeCell<dyn AnyShell>,
        alloc: &'a dyn std::alloc::Allocator,
        alloc_any: &'a dyn Any,
    ) -> Self {
        Self {
            item,
            group_item,
            shell,
            alloc,
            alloc_any,
        }
    }

    pub fn with_group_item<G: Any>(self, group_item: &'a G) -> AnyUnsafeSlot<'a> {
        let Self {
            item,
            group_item: _,
            shell,
            alloc,
            alloc_any,
        } = self;
        AnyUnsafeSlot {
            item,
            group_item,
            shell,
            alloc,
            alloc_any,
        }
    }

    pub(super) fn alloc_any(&self) -> &'a dyn Any {
        self.alloc_any
    }

    // pub fn downcast<T: AnyItem, G: Any, S: crate::Shell<T = T>, A: std::alloc::Allocator + Any>(
    //     &self,
    // ) -> Option<UnsafeSlot<'a, T, G, S, A>> {
    //     Some(UnsafeSlot::new(
    //         self.item.downcast_ref::<T>()?,
    //         self.group_item.downcast_ref()?,
    //         (self.shell as &'a dyn Any).downcast_ref()?,
    //         self.alloc_any.downcast_ref()?,
    //     ))
    // }
}

impl<'a> Copy for AnyUnsafeSlot<'a> {}

impl<'a> Clone for AnyUnsafeSlot<'a> {
    fn clone(&self) -> Self {
        Self {
            item: self.item,
            group_item: self.group_item,
            shell: self.shell,
            alloc: self.alloc,
            alloc_any: self.alloc_any,
        }
    }
}
