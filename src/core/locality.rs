use super::{Item, KeyPath, LeafPath, LocalityKey, Path};
use getset::{CopyGetters, Getters};
use std::any::Any;

#[derive(Getters, Debug)]
#[getset(get = "pub")]
pub struct Locality<T: Item> {
    locality_key: LocalityKey,
    data: T::LocalityData,
    allocator: T::Alloc,
}

impl<T: Item> Locality<T> {
    pub fn new(leaf: LeafPath, data: T::LocalityData, allocator: T::Alloc) -> Self {
        Self {
            locality_key: LocalityKey::new(leaf),
            data,
            allocator,
        }
    }

    pub fn new_default(leaf: LeafPath) -> Self
    where
        T: Item<Alloc = std::alloc::Global>,
        T::LocalityData: Default,
    {
        Self {
            locality_key: LocalityKey::new(leaf),
            data: T::LocalityData::default(),
            allocator: std::alloc::Global,
        }
    }

    pub fn slot_locality(&self) -> SlotLocality<'_, T> {
        SlotLocality {
            prefix: self.locality_key.into(),
            data: &self.data,
            allocator: &self.allocator,
        }
    }
}

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct SlotLocality<'a, T: Item> {
    // TODO: Key?, If KeyPath is really needed, then it can be a different kind of SlotLocality.
    prefix: KeyPath<T>,
    data: &'a T::LocalityData,
    allocator: &'a T::Alloc,
}

impl<'a, T: Item> SlotLocality<'a, T> {
    pub fn new(prefix: KeyPath<T>, data: &'a T::LocalityData, allocator: &'a T::Alloc) -> Self {
        Self {
            prefix,
            data,
            allocator,
        }
    }

    pub fn upcast(self) -> AnySlotLocality<'a> {
        AnySlotLocality {
            prefix: self.prefix.path(),
            data: self.data,
            allocator: self.allocator,
            alloc_any: self.allocator,
        }
    }
}

impl<'a, T: Item> Copy for SlotLocality<'a, T> {}

impl<'a, T: Item> Clone for SlotLocality<'a, T> {
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
pub struct AnySlotLocality<'a> {
    prefix: Path,
    #[getset(skip)]
    data: &'a (dyn Any + Send + Sync + 'static),
    allocator: &'a (dyn std::alloc::Allocator + Send + Sync + 'static),
    #[getset(skip)]
    alloc_any: &'a (dyn Any + Send + Sync + 'static),
}

impl<'a> AnySlotLocality<'a> {
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

    pub fn downcast<I: Item>(self) -> SlotLocality<'a, I> {
        self.downcast_try().expect("Unexpected item type")
    }

    pub fn downcast_try<I: Item>(self) -> Option<SlotLocality<'a, I>> {
        if let Some(data) = self.data.downcast_ref() {
            Some(SlotLocality {
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
