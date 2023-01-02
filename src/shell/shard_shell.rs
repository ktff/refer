#![allow(dead_code)]
use crate::{core::*, util::shard_vec::ShardVec};
use modular_bitfield::prelude::*;
use std::{any::TypeId, marker::PhantomData, ptr::Pointee};

/// If an Item knows which Types can reference it, it can implement
/// this to enable compressing references.
pub trait Referable: Send + Sync {
    /// TypeId order must be preserved.
    fn from_type(ty: TypeId) -> Option<u8>;

    fn from_index(index: u8) -> Option<<dyn AnyItem as Pointee>::Metadata>;
}

/// Meant for small shells for smallish keys.
pub struct ShardShell<T: Item + Referable> {
    from: ShardVec<ShardAnyRef, 1>,
    _data: PhantomData<T>,
}

impl<T: Item + Referable> ShardShell<T> {
    pub fn clear(&mut self, allocator: &impl std::alloc::Allocator) {
        self.from.clear(allocator);
    }

    fn to_ref(from: AnyKey) -> ShardAnyRef {
        if let Some(ty_index) = T::from_type(from.type_id()) {
            assert!(
                ty_index < 8,
                "Too many referable types for {}",
                std::any::type_name::<T>()
            );

            let mut r = ShardAnyRef::new();
            r.set_ty_index(ty_index);

            let key = from.index().get();
            assert_eq!(key >> 45, 0, "Key is too large to fit in AnyRef");
            r.set_index(key);

            r
        } else {
            panic!(
                "Unexpected reference type {:?} to {}",
                from.type_id(),
                std::any::type_name::<T>()
            );
        }
    }

    fn iter_all(&self) -> impl Iterator<Item = AnyRef> + '_ {
        self.from.iter().map(|r| {
            let metadata = T::from_index(r.ty_index()).expect("Should be a valid type index");
            let index = Index::new(r.index()).expect("Shouldn't be zero");
            AnyRef::new(AnyKey::new_with(index, metadata))
        })
    }
}

impl<T: Item + Referable> Shell for ShardShell<T> {
    type T = T;

    type Iter<'a>=impl Iterator<Item = AnyRef> + 'a
    where
        Self: 'a;

    type IterOf<'a, I: Item>=impl Iterator<Item = Ref<I>> + 'a
    where
        Self: 'a;

    fn new(_: &<Self::T as Item>::Alloc) -> Self {
        Self::default()
    }

    fn iter(&self) -> AscendingIterator<Self::Iter<'_>> {
        AscendingIterator::ascending(self.iter_all())
    }

    fn iter_of<I: Item>(&self) -> AscendingIterator<Self::IterOf<'_, I>> {
        AscendingIterator::ascending(self.iter_all().filter_map(AnyRef::downcast))
    }

    fn add(&mut self, from: impl Into<AnyKey>, alloc: &<Self::T as Item>::Alloc) {
        let add = Self::to_ref(from.into());
        let i = match self.from.as_slice_mut().binary_search(&add) {
            Ok(i) => i,
            Err(i) => i,
        };
        self.from.insert(i, add, alloc);
    }

    fn replace(&mut self, from: impl Into<AnyKey>, to: Index, _: &<Self::T as Item>::Alloc) {
        let from = from.into();
        let to = Self::to_ref(AnyKey::new_with(to, from.metadata()));
        let from = Self::to_ref(from);

        for r in self.from.iter_mut() {
            if r == &from {
                *r = to;
            }
        }
    }

    fn remove(&mut self, from: impl Into<AnyKey>) {
        let remove = Self::to_ref(from.into());
        if let Ok(i) = self.from.as_slice_mut().binary_search(&remove) {
            self.from.remove(i);
        }
    }

    fn clear(&mut self, alloc: &<Self::T as Item>::Alloc) {
        self.from.clear(alloc);
    }
}

impl<T: Item + Referable> Default for ShardShell<T> {
    fn default() -> Self {
        Self {
            from: ShardVec::new(),
            _data: PhantomData,
        }
    }
}

#[bitfield]
#[derive(Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
struct ShardAnyRef {
    ty_index: B3,
    index: B45,
}
