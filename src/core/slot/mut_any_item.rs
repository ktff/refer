use super::MutItemSlot;
use crate::core::{AnyItem, AnyItems, AnyKey, ItemsMut};
use getset::{CopyGetters, Getters};
use std::{any::Any, marker::PhantomData};

#[derive(Getters, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct MutAnyItemSlot<'a, C: AnyItems> {
    #[getset(skip)]
    pub(super) _container: PhantomData<&'a mut C>,
    pub(super) key: AnyKey,
    #[getset(skip)]
    pub(super) item: &'a mut dyn AnyItem,
    pub(super) group_item: &'a dyn Any,
    pub(super) alloc: &'a dyn std::alloc::Allocator,
    #[getset(skip)]
    pub(super) alloc_any: &'a dyn Any,
}

impl<'a, C: AnyItems> MutAnyItemSlot<'a, C> {
    pub fn item(&self) -> &dyn AnyItem {
        self.item
    }

    pub fn item_mut(&mut self) -> &mut dyn AnyItem {
        self.item
    }

    pub fn downcast<T: AnyItem>(self) -> Result<MutItemSlot<'a, T, C>, Self>
    where
        C: ItemsMut<T>,
    {
        if let Some(key) = self.key.downcast() {
            Ok(MutItemSlot {
                key,
                item: (self.item as &'a mut dyn Any)
                    .downcast_mut::<T>()
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
        } else {
            Err(self)
        }
    }
}
