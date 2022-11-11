use super::MutAnyItemSlot;
use crate::core::{AnyItem, Key};
use getset::{CopyGetters, Getters};
use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

#[derive(Getters, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MutItemSlot<'a, T: AnyItem, G: Any, A: std::alloc::Allocator + Any> {
    pub(super) key: Key<T>,
    #[getset(skip)]
    pub(super) item: &'a mut T,
    pub(super) group_item: &'a G,
    pub(super) alloc: &'a A,
}

impl<'a, T: AnyItem, G: Any, A: std::alloc::Allocator + Any> MutItemSlot<'a, T, G, A> {
    pub fn item(&self) -> &T {
        self.item
    }

    pub fn item_mut(&mut self) -> &mut T {
        self.item
    }

    pub fn upcast(self) -> MutAnyItemSlot<'a> {
        MutAnyItemSlot {
            key: self.key.upcast(),
            item: self.item,
            group_item: self.group_item,
            alloc: self.alloc,
            alloc_any: self.alloc,
        }
    }
}

impl<'a, T: AnyItem, G: Any, A: std::alloc::Allocator + Any> Deref for MutItemSlot<'a, T, G, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item
    }
}

impl<'a, T: AnyItem, G: Any, A: std::alloc::Allocator + Any> DerefMut for MutItemSlot<'a, T, G, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item
    }
}
