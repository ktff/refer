use std::any::Any;

use super::{MutAnySlot, MutItemSlot, MutShellSlot};
use crate::core::{AnyItem, Key, Shell};
use getset::{CopyGetters, Getters};

#[derive(Getters, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MutSlot<'a, T: AnyItem, G: Any, S: Shell<T = T>, A: std::alloc::Allocator + Any> {
    pub(super) key: Key<T>,
    #[getset(skip)]
    pub(super) item: &'a mut T,
    pub(super) group_item: &'a G,
    #[getset(skip)]
    pub(super) shell: &'a mut S,
    pub(super) alloc: &'a A,
}

impl<'a, T: AnyItem, G: Any, S: Shell<T = T>, A: std::alloc::Allocator + Any>
    MutSlot<'a, T, G, S, A>
{
    pub fn item(&self) -> &T {
        self.item
    }

    pub fn item_mut(&mut self) -> &mut T {
        self.item
    }

    pub fn shell(&self) -> &S {
        self.shell
    }

    pub fn shell_mut(&mut self) -> &mut S {
        self.shell
    }

    pub fn split_mut(&mut self) -> (&mut T, &mut S) {
        (self.item, self.shell)
    }

    pub fn split(self) -> (MutItemSlot<'a, T, G, A>, MutShellSlot<'a, T, S, A>) {
        (
            MutItemSlot {
                key: self.key,
                item: self.item,
                group_item: self.group_item,
                alloc: self.alloc,
            },
            MutShellSlot {
                key: self.key,
                shell: self.shell,
                alloc: self.alloc,
            },
        )
    }

    pub fn upcast(self) -> MutAnySlot<'a> {
        MutAnySlot {
            key: self.key.upcast(),
            item: self.item,
            group_item: self.group_item,
            shell: self.shell,
            alloc: self.alloc,
            alloc_any: self.alloc,
        }
    }
}
