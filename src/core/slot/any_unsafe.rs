use super::*;
use crate::core::{AnyAccess, AnyItem, AnyItems, AnyKey, AnyShell, AnyShells};
use std::{any::Any, cell::SyncUnsafeCell, marker::PhantomData};

pub struct AnyUnsafeSlot<'a> {
    item: &'a SyncUnsafeCell<dyn AnyItem>,
    group_item: &'a dyn Any,
    shell: &'a SyncUnsafeCell<dyn AnyShell>,
    alloc: &'a dyn std::alloc::Allocator,
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
    pub unsafe fn into_slot<C: AnyAccess>(
        self,
        key: AnyKey,
        container: &'a C,
    ) -> RefAnySlot<'a, C> {
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
            container,
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
    pub unsafe fn into_item<C: AnyItems>(
        self,
        key: AnyKey,
        container: &'a C,
    ) -> RefAnyItemSlot<'a, C> {
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
            container,
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
    pub unsafe fn into_shell<C: AnyShells>(
        self,
        key: AnyKey,
        container: &'a C,
    ) -> RefAnyShellSlot<'a, C> {
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
            container,
            key,
            shell,
            alloc,
            alloc_any,
        }
    }

    /// Caller must ensure that this slot belongs under the given key.
    ///
    /// UNSAFE: Caller must ensure that it has sufficient write access to this slot for lifetime 'a.
    pub unsafe fn into_slot_mut<C: AnyAccess>(self, key: AnyKey) -> MutAnySlot<'a, C> {
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
            _container: PhantomData,
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
    pub unsafe fn into_item_mut<C: AnyItems>(self, key: AnyKey) -> MutAnyItemSlot<'a, C> {
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
            _container: PhantomData,
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
    pub unsafe fn into_shell_mut<C: AnyShells>(self, key: AnyKey) -> MutAnyShellSlot<'a, C> {
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
            _container: PhantomData,
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
