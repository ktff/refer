#![allow(unused)]
use super::inline_vec::InlineVec;
use crate::core::*;
use modular_bitfield::prelude::*;
use std::{any::TypeId, marker::PhantomData, num::NonZeroU64};

/// If an Item knows which Types can reference it, it can implement
/// this to enable compressing references.
pub trait Referable {
    fn from_type(ty: TypeId) -> Option<u8>;

    fn from_type_index(index: u8) -> Option<TypeId>;
}

pub struct InlineShell<T: Referable + ?Sized + 'static> {
    from: InlineVec<AnyRef, 1>,
    _data: PhantomData<T>,
}

impl<T: Referable + ?Sized + 'static> InlineShell<T> {
    pub fn new() -> Self {
        Self {
            from: InlineVec::new(),
            _data: PhantomData,
        }
    }

    fn to_ref(from: AnyKey) -> AnyRef {
        if let Some(ty_index) = T::from_type(from.type_id()) {
            assert!(
                ty_index < 8,
                "Too many referable types for {}",
                std::any::type_name::<T>()
            );

            let mut r = AnyRef::default();
            r.set_ty_index(ty_index);

            let key = from.as_u64();
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
}

impl<T: Referable + ?Sized + 'static> AnyShell for InlineShell<T> {
    fn item_ty(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn from_any(&self) -> Box<dyn Iterator<Item = AnyKey> + '_> {
        Box::new(self.iter())
    }

    fn from_count(&self) -> usize {
        self.from.len()
    }

    fn add_from(&mut self, from: AnyKey, alloc: &impl std::alloc::Allocator) {
        self.from.push(Self::to_ref(from), alloc);
    }

    fn add_from_any(&mut self, from: AnyKey, alloc: &dyn std::alloc::Allocator) {
        self.from.push(Self::to_ref(from), alloc);
    }

    fn remove_from(&mut self, from: AnyKey) {
        let remove = Self::to_ref(from);
        let index = self
            .from
            .iter()
            .enumerate()
            .find(|(_, &r)| r == remove)
            .map(|(i, _)| i);
        if let Some(i) = index {
            self.from.swap_remove(i);
        }
    }
}

impl<T: Referable + ?Sized + 'static> Shell for InlineShell<T> {
    type T = T;
    type Iter<'a, F: ?Sized + 'static> = impl Iterator<Item = Key<F>> + 'a;
    type AnyIter<'a> = impl Iterator<Item = AnyKey> + 'a;

    fn iter(&self) -> Self::AnyIter<'_> {
        self.from.iter().map(|r| {
            let ty = T::from_type_index(r.ty_index()).expect("Should be a valid type index");
            let index = Index(NonZeroU64::new(r.index()).expect("Shouldn't be zero"));
            AnyKey::new(ty, index)
        })
    }

    fn from<F: ?Sized + 'static>(&self) -> Self::Iter<'_, F> {
        self.iter().filter_map(AnyKey::downcast)
    }
}

impl<T: Referable + ?Sized + 'static> Default for InlineShell<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[bitfield]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
struct AnyRef {
    ty_index: B3,
    index: B45,
}
