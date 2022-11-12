use super::*;
use crate::core::{AnyItem, AnyKey, AnyShell};
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

    /// Caller must ensure that this slot belongs under the given container and key.
    ///
    /// UNSAFE: Caller must ensure that it has sufficient read access to this slot for lifetime 'a.
    pub unsafe fn into_slot(self, key: AnyKey) -> RefAnySlot<'a> {
        let Self {
            item,
            group_item,
            shell,
            alloc,
            alloc_any,
        } = self;

        // UNSAFE
        let item = &*item.get();
        let shell = &*shell.get();

        RefAnySlot {
            key,
            item,
            group_item,
            shell,
            alloc,
            alloc_any,
        }
    }

    /// Caller must ensure that this slot belongs under the given container and key.
    ///
    /// UNSAFE: Caller must ensure that it has sufficient read access to this item for lifetime 'a.
    pub unsafe fn into_item(self, key: AnyKey) -> RefAnyItemSlot<'a> {
        let Self {
            item,
            group_item,
            shell: _,
            alloc,
            alloc_any,
        } = self;

        // UNSAFE
        let item = &*item.get();

        RefAnyItemSlot {
            key,
            item,
            group_item,
            alloc,
            alloc_any,
        }
    }

    /// Caller must ensure that this slot belongs under the given container and key.
    ///
    /// UNSAFE: Caller must ensure that it has sufficient read access to this shell for lifetime 'a.
    pub unsafe fn into_shell(self, key: AnyKey) -> RefAnyShellSlot<'a> {
        let Self {
            item: _,
            group_item: _,
            shell,
            alloc,
            alloc_any,
        } = self;

        // UNSAFE
        let shell = &*shell.get();

        RefAnyShellSlot {
            key,
            shell,
            alloc,
            alloc_any,
        }
    }

    /// Caller must ensure that this slot belongs under the given key.
    ///
    /// UNSAFE: Caller must ensure that it has sufficient write access to this slot for lifetime 'a.
    pub unsafe fn into_slot_mut(self, key: AnyKey) -> MutAnySlot<'a> {
        let Self {
            item,
            group_item,
            shell,
            alloc,
            alloc_any,
        } = self;

        // UNSAFE
        let item = &mut *item.get();
        let shell = &mut *shell.get();

        MutAnySlot {
            key,
            item,
            group_item,
            shell,
            alloc,
            alloc_any,
        }
    }

    /// Caller must ensure that this slot belongs under the given key.
    ///
    /// UNSAFE: Caller must ensure that it has sufficient write access to this item for lifetime 'a.
    pub unsafe fn into_item_mut(self, key: AnyKey) -> MutAnyItemSlot<'a> {
        let Self {
            item,
            group_item,
            shell: _,
            alloc,
            alloc_any,
        } = self;

        // UNSAFE
        let item = &mut *item.get();

        MutAnyItemSlot {
            key,
            item,
            group_item,
            alloc,
            alloc_any,
        }
    }

    /// Caller must ensure that this slot belongs under the given key.
    ///
    /// UNSAFE: Caller must ensure that it has sufficient write access to this shell for lifetime 'a.
    pub unsafe fn into_shell_mut(self, key: AnyKey) -> MutAnyShellSlot<'a> {
        let Self {
            item: _,
            group_item: _,
            shell,
            alloc,
            alloc_any,
        } = self;

        // UNSAFE
        let shell = &mut *shell.get();

        MutAnyShellSlot {
            key,
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
}
