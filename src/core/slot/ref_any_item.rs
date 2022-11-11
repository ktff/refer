use super::RefItemSlot;
use crate::core::{AnyItem, AnyItems, AnyKey, Items};
use getset::CopyGetters;
use std::any::Any;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RefAnyItemSlot<'a, C: AnyItems> {
    pub(super) container: &'a C,
    pub(super) key: AnyKey,
    pub(super) item: &'a dyn AnyItem,
    pub(super) group_item: &'a dyn Any,
    pub(super) alloc: &'a dyn std::alloc::Allocator,
    #[getset(skip)]
    pub(super) alloc_any: &'a dyn Any,
}

impl<'a, C: AnyItems> RefAnyItemSlot<'a, C> {
    pub fn downcast<T: AnyItem>(&self) -> Option<RefItemSlot<'a, T, C>>
    where
        C: Items<T>,
    {
        let key = self.key.downcast()?;
        Some(RefItemSlot {
            container: self.container,
            key,
            item: (self.item as &'a dyn Any)
                .downcast_ref::<T>()
                .expect("Item of wrong type"),
            group_item: self
                .group_item
                .downcast_ref()
                .expect("Group item of wrong type"),
            alloc: self
                .alloc_any
                .downcast_ref()
                .expect("Allocator of wrong type"),
        })
    }
}
