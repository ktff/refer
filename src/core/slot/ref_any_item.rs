use super::RefItemSlot;
use crate::{
    core::{AnyItem, AnyKey},
    Access,
};
use getset::CopyGetters;
use std::any::Any;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RefAnyItemSlot<'a> {
    pub(super) key: AnyKey,
    pub(super) item: &'a dyn AnyItem,
    pub(super) group_item: &'a dyn Any,
    pub(super) alloc: &'a dyn std::alloc::Allocator,
    #[getset(skip)]
    pub(super) alloc_any: &'a dyn Any,
}

impl<'a> RefAnyItemSlot<'a> {
    pub fn downcast<T: AnyItem, C: Access<T>>(
        &self,
    ) -> Option<RefItemSlot<'a, T, C::GroupItem, C::Alloc>> {
        let key = self.key.downcast()?;
        Some(RefItemSlot {
            key,
            item: (self.item as &'a dyn Any).downcast_ref::<T>()?,
            group_item: self.group_item.downcast_ref()?,
            alloc: self.alloc_any.downcast_ref()?,
        })
    }
}
