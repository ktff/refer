use super::*;
use crate::core::{Access, AnyItem, Items, ItemsMut, Key, Shell, Shells, ShellsMut};
use std::{any::Any, cell::SyncUnsafeCell};

pub struct UnsafeSlot<'a, T: AnyItem, G: Any, S: Shell<T = T>, A: std::alloc::Allocator> {
    item: &'a SyncUnsafeCell<T>,
    group_item: &'a G,
    shell: &'a SyncUnsafeCell<S>,
    alloc: &'a A,
}

impl<'a, T: AnyItem, G: Any, S: Shell<T = T>, A: std::alloc::Allocator> UnsafeSlot<'a, T, G, S, A> {
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

    /// Caller must ensure that this slot belongs under the given container and key.
    ///
    /// UNSAFE: Caller must ensure that it has sufficient read access to this slot for lifetime 'a.
    pub unsafe fn into_slot<C: Access<T, GroupItem = G, Shell = S, Alloc = A>>(
        self,
        key: Key<T>,
        container: &'a C,
    ) -> RefSlot<'a, T, C> {
        let Self {
            item,
            group_item,
            shell,
            alloc,
        } = self;

        // UNSAFE CASTS
        let item = &*item.get();
        let shell = &*shell.get();

        RefSlot {
            container,
            key,
            item,
            group_item,
            shell,
            alloc,
        }
    }

    /// Caller must ensure that this slot belongs under the given container and key.
    ///
    /// UNSAFE: Caller must ensure that it has sufficient read access to this item for lifetime 'a.
    pub unsafe fn into_item<C: Items<T, GroupItem = G, Alloc = A>>(
        self,
        key: Key<T>,
        container: &'a C,
    ) -> RefItemSlot<'a, T, C> {
        let Self {
            item,
            group_item,
            shell: _,
            alloc,
        } = self;

        // UNSAFE CASTS
        let item = &*item.get();

        RefItemSlot {
            container,
            key,
            item,
            group_item,
            alloc,
        }
    }

    /// Caller must ensure that this slot belongs under the given container and key.
    ///
    /// UNSAFE: Caller must ensure that it has sufficient read access to this shell for lifetime 'a.
    pub unsafe fn into_shell<C: Shells<T, Shell = S, Alloc = A>>(
        self,
        key: Key<T>,
        container: &'a C,
    ) -> RefShellSlot<'a, T, C> {
        let Self {
            item: _,
            group_item: _,
            shell,
            alloc,
        } = self;

        // UNSAFE CASTS
        let shell = &*shell.get();

        RefShellSlot {
            container,
            key,
            shell,
            alloc,
        }
    }

    /// Caller must ensure that this slot belongs under the given key.
    ///
    /// UNSAFE: Caller must ensure that it has sufficient write access to this slot for lifetime 'a.
    pub unsafe fn into_slot_mut<C: Access<T, GroupItem = G, Shell = S, Alloc = A>>(
        self,
        key: Key<T>,
    ) -> MutSlot<'a, T, C> {
        let Self {
            item,
            group_item,
            shell,
            alloc,
        } = self;

        // UNSAFE CASTS
        let item = &mut *item.get();
        let shell = &mut *shell.get();

        MutSlot {
            key,
            item,
            group_item,
            shell,
            alloc,
        }
    }

    /// Caller must ensure that this slot belongs under the given key.
    ///
    /// UNSAFE: Caller must ensure that it has sufficient write access to this item for lifetime 'a.
    pub unsafe fn into_item_mut<C: ItemsMut<T, GroupItem = G, Alloc = A>>(
        self,
        key: Key<T>,
    ) -> MutItemSlot<'a, T, C> {
        let Self {
            item,
            group_item,
            shell: _,
            alloc,
        } = self;

        // UNSAFE CASTS
        let item = &mut *item.get();

        MutItemSlot {
            key,
            item,
            group_item,
            alloc,
        }
    }

    /// Caller must ensure that this slot belongs under the given key.
    ///
    /// UNSAFE: Caller must ensure that it has sufficient write access to this shell for lifetime 'a.
    pub unsafe fn into_shell_mut<C: ShellsMut<T, Shell = S, Alloc = A>>(
        self,
        key: Key<T>,
    ) -> MutShellSlot<'a, T, C> {
        let Self {
            item: _,
            group_item: _,
            shell,
            alloc,
        } = self;

        // UNSAFE CASTS
        let shell = &mut *shell.get();

        MutShellSlot { key, shell, alloc }
    }
}

impl<'a, T: AnyItem, S: Shell<T = T>, A: std::alloc::Allocator> UnsafeSlot<'a, T, (), S, A> {
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
