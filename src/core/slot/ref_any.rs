use super::{RefAnyItemSlot, RefAnyShellSlot, RefSlot};
use crate::core::{Access, AnyItem, AnyKey, AnyShell};
use getset::CopyGetters;
use std::any::Any;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RefAnySlot<'a> {
    pub(super) key: AnyKey,
    pub(super) item: &'a dyn AnyItem,
    pub(super) group_item: &'a dyn Any,
    pub(super) shell: &'a dyn AnyShell,
    pub(super) alloc: &'a dyn std::alloc::Allocator,
    // The same as alloc but with a different dyn trait.
    #[getset(skip)]
    pub(super) alloc_any: &'a dyn Any,
}

impl<'a> RefAnySlot<'a> {
    pub fn split(self) -> (RefAnyItemSlot<'a>, RefAnyShellSlot<'a>) {
        (
            RefAnyItemSlot {
                key: self.key,
                item: self.item,
                group_item: self.group_item,
                alloc: self.alloc,
                alloc_any: self.alloc_any,
            },
            RefAnyShellSlot {
                key: self.key,
                shell: self.shell,
                alloc: self.alloc,
                alloc_any: self.alloc_any,
            },
        )
    }

    pub fn downcast<T: AnyItem, C: Access<T>>(
        &self,
    ) -> Option<RefSlot<'a, T, C::GroupItem, C::Shell, C::Alloc>> {
        let key = self.key.downcast()?;
        Some(RefSlot {
            key,
            item: (self.item as &'a dyn Any).downcast_ref::<T>()?,
            group_item: self.group_item.downcast_ref()?,
            shell: (self.shell as &'a dyn Any).downcast_ref()?,
            alloc: self.alloc_any.downcast_ref()?,
        })
    }
}
