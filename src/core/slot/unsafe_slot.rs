use super::*;
use crate::core::{AnyItem, Key, Shell};
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

    /// Caller must ensure that this slot belongs under the given container and key.
    ///
    /// UNSAFE: Caller must ensure that it has sufficient read access to this slot for lifetime 'a.
    pub unsafe fn into_slot(self, key: Key<T>) -> RefSlot<'a, T, G, S, A> {
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
    pub unsafe fn into_item(self, key: Key<T>) -> RefItemSlot<'a, T, G, A> {
        let Self {
            item,
            group_item,
            shell: _,
            alloc,
        } = self;

        // UNSAFE CASTS
        let item = &*item.get();

        RefItemSlot {
            key,
            item,
            group_item,
            alloc,
        }
    }

    /// Caller must ensure that this slot belongs under the given container and key.
    ///
    /// UNSAFE: Caller must ensure that it has sufficient read access to this shell for lifetime 'a.
    pub unsafe fn into_shell(self, key: Key<T>) -> RefShellSlot<'a, T, S, A> {
        let Self {
            item: _,
            group_item: _,
            shell,
            alloc,
        } = self;

        // UNSAFE CASTS
        let shell = &*shell.get();

        RefShellSlot { key, shell, alloc }
    }

    /// Caller must ensure that this slot belongs under the given key.
    ///
    /// UNSAFE: Caller must ensure that it has sufficient write access to this slot for lifetime 'a.
    pub unsafe fn into_slot_mut(self, key: Key<T>) -> MutSlot<'a, T, G, S, A> {
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
    pub unsafe fn into_item_mut(self, key: Key<T>) -> MutItemSlot<'a, T, G, A> {
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
    pub unsafe fn into_shell_mut(self, key: Key<T>) -> MutShellSlot<'a, T, S, A> {
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
