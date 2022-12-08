use super::{AnyKey, AnyRef, Index, Item, KeyPrefix};
use getset::{CopyGetters, Getters};
use std::{
    alloc::Allocator,
    any::{Any, TypeId},
    fmt::Debug,
};

#[derive(Getters)]
#[getset(get = "pub")]
pub struct Context<T: Item> {
    prefix: KeyPrefix,
    data: T::LocalityData,
    allocator: T::Alloc,
}

impl<T: Item> Context<T> {
    pub fn new(prefix: KeyPrefix, data: T::LocalityData, allocator: T::Alloc) -> Self {
        Self {
            prefix,
            data,
            allocator,
        }
    }

    pub fn item_context(&self) -> ItemContext<'_, T> {
        ItemContext {
            prefix: self.prefix,
            data: &self.data,
            allocator: &self.allocator,
        }
    }
}
// TODO: SlotContext?
#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct ItemContext<'a, T: Item> {
    prefix: KeyPrefix,
    data: &'a T::LocalityData,
    allocator: &'a T::Alloc,
}

impl<'a, T: Item> ItemContext<'a, T> {
    pub fn new(prefix: KeyPrefix, data: &'a T::LocalityData, allocator: &'a T::Alloc) -> Self {
        Self {
            prefix,
            data,
            allocator,
        }
    }

    pub fn upcast(self) -> AnyItemContext<'a> {
        AnyItemContext {
            ty: TypeId::of::<T>(),
            prefix: self.prefix,
            data: self.data,
            allocator: self.allocator,
            alloc_any: self.allocator,
        }
    }
}

impl<'a, T: Item> Copy for ItemContext<'a, T> {}

impl<'a, T: Item> Clone for ItemContext<'a, T> {
    fn clone(&self) -> Self {
        Self {
            prefix: self.prefix,
            data: self.data,
            allocator: self.allocator,
        }
    }
}

#[derive(Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct AnyItemContext<'a> {
    #[getset(skip)]
    ty: TypeId,
    prefix: KeyPrefix,
    #[getset(skip)]
    data: &'a dyn Any,
    allocator: &'a dyn std::alloc::Allocator,
    #[getset(skip)]
    alloc_any: &'a dyn Any,
}

impl<'a> AnyItemContext<'a> {
    pub fn new(
        ty: TypeId,
        prefix: KeyPrefix,
        data: &'a dyn Any,
        allocator: &'a dyn std::alloc::Allocator,
        alloc_any: &'a dyn Any,
    ) -> Self {
        Self {
            ty,
            prefix,
            data,
            allocator,
            alloc_any,
        }
    }

    pub fn downcast<I: Item>(self) -> ItemContext<'a, I> {
        self.downcast_try().expect("Unexpected item type")
    }

    pub fn downcast_try<I: Item>(self) -> Option<ItemContext<'a, I>> {
        if self.ty == TypeId::of::<I>() {
            Some(ItemContext {
                prefix: self.prefix,
                data: self
                    .data
                    .downcast_ref()
                    .expect("Mismatched locality data type"),
                allocator: self
                    .alloc_any
                    .downcast_ref()
                    .expect("Mismatched allocator type"),
            })
        } else {
            None
        }
    }
}
