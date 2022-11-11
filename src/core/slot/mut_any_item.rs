use super::MutItemSlot;
use crate::{
    core::{AnyItem, AnyKey},
    Access,
};
use getset::{CopyGetters, Getters};
use std::any::Any;

#[derive(Getters, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MutAnyItemSlot<'a> {
    pub(super) key: AnyKey,
    #[getset(skip)]
    pub(super) item: &'a mut dyn AnyItem,
    pub(super) group_item: &'a dyn Any,
    pub(super) alloc: &'a dyn std::alloc::Allocator,
    #[getset(skip)]
    pub(super) alloc_any: &'a dyn Any,
}

impl<'a> MutAnyItemSlot<'a> {
    pub fn item(&self) -> &dyn AnyItem {
        self.item
    }

    pub fn item_mut(&mut self) -> &mut dyn AnyItem {
        self.item
    }

    /// Item of given key has/is been removed.
    ///
    /// This item should return true if it's ok with it.
    /// If false, this item will also be removed.
    ///
    /// Should be called for its references.
    pub fn item_removed(&mut self, key: AnyKey) -> bool {
        self.item.item_removed(self.key.index(), key)
    }

    pub fn downcast<T: AnyItem, C: Access<T>>(
        self,
    ) -> Result<MutItemSlot<'a, T, C::GroupItem, C::Alloc>, Self> {
        if let Some(key) = self.key.downcast() {
            if let Some(alloc) = self.alloc_any.downcast_ref() {
                if let Some(group_item) = self.group_item.downcast_ref() {
                    Ok(MutItemSlot {
                        key,
                        item: (self.item as &'a mut dyn Any)
                            .downcast_mut::<T>()
                            .expect("Mismatched key-item types"),
                        group_item,
                        alloc,
                    })
                } else {
                    Err(self)
                }
            } else {
                Err(self)
            }
        } else {
            Err(self)
        }
    }
}
