use super::{RefAnyItemSlot, RefAnyShellSlot, RefSlot};
use crate::core::{Access, AnyAccess, AnyItem, AnyItems, AnyKey, AnyShell, AnyShells};
use getset::CopyGetters;
use std::any::Any;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RefAnySlot<'a, C: AnyAccess> {
    pub(super) container: &'a C,
    pub(super) key: AnyKey,
    pub(super) item: &'a dyn AnyItem,
    pub(super) group_item: &'a dyn Any,
    pub(super) shell: &'a dyn AnyShell,
    pub(super) alloc: &'a dyn std::alloc::Allocator,
    // The same as alloc but with a different dyn trait.
    #[getset(skip)]
    pub(super) alloc_any: &'a dyn Any,
}

impl<'a, C: AnyAccess> RefAnySlot<'a, C> {
    pub fn split(self) -> (RefAnyItemSlot<'a, C>, RefAnyShellSlot<'a, C>)
    where
        C: AnyItems + AnyShells,
    {
        (
            RefAnyItemSlot {
                container: self.container,
                key: self.key,
                item: self.item,
                group_item: self.group_item,
                alloc: self.alloc,
                alloc_any: self.alloc_any,
            },
            RefAnyShellSlot {
                container: self.container,
                key: self.key,
                shell: self.shell,
                alloc: self.alloc,
                alloc_any: self.alloc_any,
            },
        )
    }

    pub fn downcast<T: AnyItem>(&self) -> Option<RefSlot<'a, T, C>>
    where
        C: Access<T>,
    {
        let key = self.key.downcast()?;
        Some(RefSlot {
            container: self.container,
            key,
            item: (self.item as &'a dyn Any)
                .downcast_ref::<T>()
                .expect("Item of wrong type"),
            group_item: self
                .group_item
                .downcast_ref()
                .expect("Group item of wrong type"),
            shell: (self.shell as &'a dyn Any)
                .downcast_ref()
                .expect("Shell of wrong type"),
            alloc: self
                .alloc_any
                .downcast_ref()
                .expect("Allocator of wrong type"),
        })
    }
}
