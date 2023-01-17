use super::{Item, KeyPath, LeafPath, Path};
use getset::{CopyGetters, Getters};
use std::any::Any;

#[derive(Getters, Debug)]
#[getset(get = "pub")]
pub struct Context<T: Item> {
    leaf_path: LeafPath,
    data: T::LocalityData,
    allocator: T::Alloc,
}

impl<T: Item> Context<T> {
    pub fn new(leaf_path: LeafPath, data: T::LocalityData, allocator: T::Alloc) -> Self {
        Self {
            leaf_path,
            data,
            allocator,
        }
    }

    pub fn new_default(leaf_path: LeafPath) -> Self
    where
        T: Item<Alloc = std::alloc::Global>,
        T::LocalityData: Default,
    {
        Self {
            leaf_path,
            data: T::LocalityData::default(),
            allocator: std::alloc::Global,
        }
    }

    pub fn slot_context(&self) -> SlotContext<'_, T> {
        SlotContext {
            prefix: self.leaf_path.into(),
            data: &self.data,
            allocator: &self.allocator,
        }
    }
}

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct SlotContext<'a, T: Item> {
    prefix: KeyPath<T>,
    data: &'a T::LocalityData,
    allocator: &'a T::Alloc,
}

impl<'a, T: Item> SlotContext<'a, T> {
    pub fn new(prefix: KeyPath<T>, data: &'a T::LocalityData, allocator: &'a T::Alloc) -> Self {
        Self {
            prefix,
            data,
            allocator,
        }
    }

    pub fn upcast(self) -> AnySlotContext<'a> {
        AnySlotContext {
            prefix: self.prefix.path(),
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
    prefix: Path,
    #[getset(skip)]
    data: &'a (dyn Any + Send + Sync + 'static),
    allocator: &'a (dyn std::alloc::Allocator + Send + Sync + 'static),
    #[getset(skip)]
    alloc_any: &'a (dyn Any + Send + Sync + 'static),
}

impl<'a> AnySlotContext<'a> {
    pub fn new(
        prefix: Path,
        data: &'a (dyn Any + Send + Sync + 'static),
        allocator: &'a (dyn std::alloc::Allocator + Send + Sync + 'static),
        alloc_any: &'a (dyn Any + Send + Sync + 'static),
    ) -> Self {
        Self {
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
        if let Some(data) = self.data.downcast_ref() {
            Some(SlotContext {
                prefix: KeyPath::new(self.prefix),
                data,
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
