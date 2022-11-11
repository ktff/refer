use std::{any::Any, ops::Deref};

use super::RefAnyItemSlot;
use crate::core::{AnyItem, Key};
use getset::CopyGetters;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RefItemSlot<'a, T: AnyItem, G: Any, A: std::alloc::Allocator + Any> {
    pub(super) key: Key<T>,
    pub(super) item: &'a T,
    pub(super) group_item: &'a G,
    pub(super) alloc: &'a A,
}

impl<'a, T: AnyItem, G: Any, A: std::alloc::Allocator + Any> RefItemSlot<'a, T, G, A> {
    pub fn upcast(self) -> RefAnyItemSlot<'a> {
        RefAnyItemSlot {
            key: self.key.upcast(),
            item: self.item,
            group_item: self.group_item,
            alloc: self.alloc,
            alloc_any: self.alloc,
        }
    }
}

impl<'a, T: AnyItem, G: Any, A: std::alloc::Allocator + Any> Deref for RefItemSlot<'a, T, G, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}
