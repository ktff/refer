#![allow(dead_code)]
use crate::{core::*, util::shard_vec::ShardVec};
use modular_bitfield::prelude::*;
use std::marker::PhantomData;

// TODO: Version of this that stores key inline if it fits, else stores on heap.

/// Meant for small shells for smallish keys of max used lower 48 bits.
pub struct ShardShell<T: Item> {
    from: ShardVec<ShardAnyRef, 1>,
    _data: PhantomData<T>,
}

impl<T: Item> ShardShell<T> {
    pub fn clear(&mut self, allocator: &impl std::alloc::Allocator) {
        self.from.clear(allocator);
    }

    fn to_ref(from: AnyKey) -> ShardAnyRef {
        let mut r = ShardAnyRef::new();

        let key = from.index().get();
        assert_eq!(key >> 48, 0, "Key is too large to fit in ShardShell");
        r.set_index(key as u64);

        r
    }

    fn iter_all(&self) -> impl Iterator<Item = AnyRef> + '_ {
        self.from.iter().map(|r| {
            let index = Index::new(r.index() as IndexBase).expect("Shouldn't be zero");
            AnyRef::new(AnyKey::new_any(index))
        })
    }
}

impl<T: Item> Shell for ShardShell<T> {
    type T = T;

    type Iter<'a>=impl Iterator<Item = AnyRef> + 'a
    where
        Self: 'a;

    fn new_in(_: &<Self::T as Item>::Alloc) -> Self {
        Self::default()
    }

    fn iter(&self) -> AscendingIterator<Self::Iter<'_>> {
        AscendingIterator::ascending(self.iter_all())
    }

    fn add(&mut self, from: impl Into<AnyKey>, alloc: &<Self::T as Item>::Alloc) {
        let add = Self::to_ref(from.into());
        let i = match self.from.as_slice_mut().binary_search(&add) {
            Ok(i) => i,
            Err(i) => i,
        };
        self.from.insert(i, add, alloc);
    }

    fn replace(&mut self, from: impl Into<AnyKey>, to: AnyKey, _: &<Self::T as Item>::Alloc) {
        let from = from.into();
        let to = Self::to_ref(to);
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

impl<T: Item> Default for ShardShell<T> {
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
    index: B48,
}
