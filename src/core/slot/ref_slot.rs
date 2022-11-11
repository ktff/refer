use std::any::Any;

use super::{RefAnySlot, RefItemSlot, RefShellSlot};
use crate::{
    core::{AnyItem, Key},
    Shell,
};
use getset::CopyGetters;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RefSlot<'a, T: AnyItem, G: Any, S: Shell<T = T>, A: std::alloc::Allocator + Any> {
    pub(super) key: Key<T>,
    pub(super) item: &'a T,
    pub(super) group_item: &'a G,
    pub(super) shell: &'a S,
    pub(super) alloc: &'a A,
}

impl<'a, T: AnyItem, G: Any, S: Shell<T = T>, A: std::alloc::Allocator + Any>
    RefSlot<'a, T, G, S, A>
{
    pub fn split(self) -> (RefItemSlot<'a, T, G, A>, RefShellSlot<'a, T, S, A>) {
        (
            RefItemSlot {
                key: self.key,
                item: self.item,
                group_item: self.group_item,
                alloc: self.alloc,
            },
            RefShellSlot {
                key: self.key,
                shell: self.shell,
                alloc: self.alloc,
            },
        )
    }

    pub fn upcast(self) -> RefAnySlot<'a> {
        RefAnySlot {
            key: self.key.upcast(),
            item: self.item,
            group_item: self.group_item,
            shell: self.shell,
            alloc: self.alloc,
            alloc_any: self.alloc,
        }
    }
}
