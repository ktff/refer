use super::{RefAnySlot, RefItemSlot, RefShellSlot};
use crate::core::{Access, Allocator, AnyItem, Items, Key, Shells};
use getset::CopyGetters;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RefSlot<'a, T: AnyItem, C: Access<T>> {
    pub(super) container: &'a C,
    pub(super) key: Key<T>,
    pub(super) item: &'a T,
    pub(super) group_item: &'a C::GroupItem,
    pub(super) shell: &'a C::Shell,
    pub(super) alloc: &'a C::Alloc,
}

impl<'a, T: AnyItem, C: Access<T>> RefSlot<'a, T, C> {
    pub fn split(self) -> (RefItemSlot<'a, T, C>, RefShellSlot<'a, T, C>)
    where
        C: Items<T, GroupItem = <C as Access<T>>::GroupItem, Alloc = <C as Allocator<T>>::Alloc>
            + Shells<T, Shell = <C as Access<T>>::Shell, Alloc = <C as Allocator<T>>::Alloc>,
    {
        (
            RefItemSlot {
                container: self.container,
                key: self.key,
                item: self.item,
                group_item: self.group_item,
                alloc: self.alloc,
            },
            RefShellSlot {
                container: self.container,
                key: self.key,
                shell: self.shell,
                alloc: self.alloc,
            },
        )
    }

    pub fn upcast(self) -> RefAnySlot<'a, C> {
        RefAnySlot {
            container: self.container,
            key: self.key.upcast(),
            item: self.item,
            group_item: self.group_item,
            shell: self.shell,
            alloc: self.alloc,
            alloc_any: self.alloc,
        }
    }
}
