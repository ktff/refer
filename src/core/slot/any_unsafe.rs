use crate::core::{AnyItem, AnyItemContext, AnyShell, KeyPrefix};
use getset::CopyGetters;
use std::{any::Any, cell::SyncUnsafeCell};

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct AnyUnsafeSlot<'a> {
    context: AnyItemContext<'a>,
    item: &'a SyncUnsafeCell<dyn AnyItem>,
    shell: &'a SyncUnsafeCell<dyn AnyShell>,
}

impl<'a> AnyUnsafeSlot<'a> {
    pub fn new(
        context: AnyItemContext<'a>,
        item: &'a SyncUnsafeCell<dyn AnyItem>,
        shell: &'a SyncUnsafeCell<dyn AnyShell>,
    ) -> Self {
        Self {
            context,
            item,
            shell,
        }
    }

    // TODO: Try to enable this

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
            context: self.context,
            item: self.item,
            shell: self.shell,
        }
    }
}

// Deref to context
impl<'a> std::ops::Deref for AnyUnsafeSlot<'a> {
    type Target = AnyItemContext<'a>;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
