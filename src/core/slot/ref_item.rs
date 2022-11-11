use super::RefAnyItemSlot;
use crate::core::{AnyItem, AnyItems, Items, Key};
use getset::CopyGetters;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct RefItemSlot<'a, T: AnyItem, C: Items<T>> {
    pub(super) container: &'a C,
    pub(super) key: Key<T>,
    pub(super) item: &'a T,
    pub(super) group_item: &'a C::GroupItem,
    pub(super) alloc: &'a C::Alloc,
}

impl<'a, T: AnyItem, C: Items<T>> RefItemSlot<'a, T, C> {
    pub fn upcast(self) -> RefAnyItemSlot<'a, C>
    where
        C: AnyItems,
    {
        RefAnyItemSlot {
            container: self.container,
            key: self.key.upcast(),
            item: self.item,
            group_item: self.group_item,
            alloc: self.alloc,
            alloc_any: self.alloc,
        }
    }
}
