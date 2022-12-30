use super::*;
use crate::core::{Item, Shell, SlotContext};
use getset::CopyGetters;
use std::{any::Any, cell::SyncUnsafeCell};

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct UnsafeSlot<'a, T: Item, S: Shell<T = T>> {
    context: SlotContext<'a, T>,
    item: &'a SyncUnsafeCell<T>,
    shell: &'a SyncUnsafeCell<S>,
}

impl<'a, T: Item, S: Shell<T = T>> UnsafeSlot<'a, T, S> {
    pub fn new(
        context: SlotContext<'a, T>,
        item: &'a SyncUnsafeCell<T>,
        shell: &'a SyncUnsafeCell<S>,
    ) -> Self {
        Self {
            context,
            item,
            shell,
        }
    }

    pub fn upcast(self) -> AnyUnsafeSlot<'a> {
        AnyUnsafeSlot::new(self.context.upcast(), self.item, self.shell)
    }
}

impl<'a, T: Item, S: Shell<T = T>> Copy for UnsafeSlot<'a, T, S> {}

impl<'a, T: Item, S: Shell<T = T>> Clone for UnsafeSlot<'a, T, S> {
    fn clone(&self) -> Self {
        Self {
            context: self.context,
            item: self.item,
            shell: self.shell,
        }
    }
}

// Deref to context
impl<'a, T: Item, S: Shell<T = T>> std::ops::Deref for UnsafeSlot<'a, T, S> {
    type Target = SlotContext<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
