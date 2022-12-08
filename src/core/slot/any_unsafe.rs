use crate::core::{AnyItem, AnyShell, AnySlotContext, Container, Item, KeyPrefix};
use getset::CopyGetters;
use std::{any::Any, cell::SyncUnsafeCell};

use super::UnsafeSlot;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct AnyUnsafeSlot<'a> {
    context: AnySlotContext<'a>,
    item: &'a SyncUnsafeCell<dyn AnyItem>,
    shell: &'a SyncUnsafeCell<dyn AnyShell>,
}

impl<'a> AnyUnsafeSlot<'a> {
    pub fn new(
        context: AnySlotContext<'a>,
        item: &'a SyncUnsafeCell<dyn AnyItem>,
        shell: &'a SyncUnsafeCell<dyn AnyShell>,
    ) -> Self {
        Self {
            context,
            item,
            shell,
        }
    }
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
    type Target = AnySlotContext<'a>;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
