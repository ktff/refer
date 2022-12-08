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

    pub fn slot_context(&self) -> SlotContext<'_, T> {
        SlotContext {
            prefix: self.prefix,
            data: &self.data,
            allocator: &self.allocator,
        }
    }
}

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct SlotContext<'a, T: Item> {
    prefix: KeyPrefix,
    data: &'a T::LocalityData,
    allocator: &'a T::Alloc,
}

impl<'a, T: Item> SlotContext<'a, T> {
    pub fn new(prefix: KeyPrefix, data: &'a T::LocalityData, allocator: &'a T::Alloc) -> Self {
        Self {
            prefix,
            data,
            allocator,
        }
    }

    pub fn upcast(self) -> AnySlotContext<'a> {
        AnySlotContext {
            ty: TypeId::of::<T>(),
            prefix: self.prefix,
            data: self.data,
            allocator: self.allocator,
            alloc_any: self.allocator,
        }
    }
}

impl<'a, T: Item> Copy for SlotContext<'a, T> {}

impl<'a, T: Item> Clone for SlotContext<'a, T> {
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
pub struct AnySlotContext<'a> {
    #[getset(skip)]
    ty: TypeId,
    prefix: KeyPrefix,
    #[getset(skip)]
    data: &'a dyn Any,
    allocator: &'a dyn std::alloc::Allocator,
    #[getset(skip)]
    alloc_any: &'a dyn Any,
}

impl<'a> AnySlotContext<'a> {
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

    pub fn downcast<I: Item>(self) -> SlotContext<'a, I> {
        self.downcast_try().expect("Unexpected item type")
    }

    pub fn downcast_try<I: Item>(self) -> Option<SlotContext<'a, I>> {
        if self.ty == TypeId::of::<I>() {
            Some(SlotContext {
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
