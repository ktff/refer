use super::MutAnyItemSlot;
use crate::core::{AnyItem, AnyItems, ItemsMut, Key};
use getset::{CopyGetters, Getters};

#[derive(Getters, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MutItemSlot<'a, T: AnyItem, C: ItemsMut<T>> {
    pub(super) key: Key<T>,
    #[getset(skip)]
    pub(super) item: &'a mut T,
    pub(super) group_item: &'a C::GroupItem,
    pub(super) alloc: &'a C::Alloc,
}

impl<'a, T: AnyItem, C: ItemsMut<T>> MutItemSlot<'a, T, C> {
    pub fn item(&self) -> &T {
        self.item
    }

    pub fn item_mut(&mut self) -> &mut T {
        self.item
    }

    pub fn upcast(self) -> MutAnyItemSlot<'a, C>
    where
        C: AnyItems,
    {
        MutAnyItemSlot {
            _container: std::marker::PhantomData,
            key: self.key.upcast(),
            item: self.item,
            group_item: self.group_item,
            alloc: self.alloc,
            alloc_any: self.alloc,
        }
    }
}
