mod all;
mod key;
mod key_permit;
mod keys;
mod path;
mod ty;
mod type_permit;
mod types;

pub use key::SlotAccess;
pub use key_permit::*;
pub use type_permit::*;

use std::{any::TypeId, collections::HashSet, marker::PhantomData, ops::Deref};

use crate::core::{
    permit, AnyContainer, AnyDynItem, Container, DynContainer, DynItem, Item, Key, Path, Ptr, Ref,
    Slot,
};

use super::{Mut, Permit};

pub struct All;

pub struct Not<T>(T);

/// 'a - Lifetime
/// C - Container type
/// R - Ref/Mut restriction
/// T - Type restriction
/// K - Key restriction
pub struct Access<'a, C: AnyContainer + ?Sized, P: Permit, T: TypePermit, K: KeyPermit> {
    container: &'a C,
    permit: P,
    type_state: T::State,
    key_state: K::State,
    _marker: PhantomData<Key<Ref<'a>>>,
}

impl<'a, C: AnyContainer + ?Sized> Access<'a, C, Mut, All, All> {
    pub fn new(container: &'a mut C) -> Self {
        // SAFETY: We have exclusive access to the container so any Permit is valid.
        let permit = unsafe { permit::Add::new().into() };
        Self {
            container: container,
            permit,
            type_state: Default::default(),
            key_state: Default::default(),
            _marker: PhantomData,
        }
    }
}

impl<'a, C: AnyContainer + ?Sized> Access<'a, C, permit::Ref, All, All> {
    pub fn new_ref(container: &'a mut C) -> Self {
        // SAFETY: We have exclusive access to the container so any Permit is valid.
        let permit = unsafe { permit::Add::new().into() };
        Self {
            container: container,
            permit,
            type_state: Default::default(),
            key_state: Default::default(),
            _marker: PhantomData,
        }
    }

    /// SAFETY: Caller must ensure that it has the correct Ref access to C for the given 'a and that
    ///         all keys are valid for 'a.
    pub unsafe fn unsafe_new(permit: permit::Ref, container: &'a C) -> Self {
        Self {
            container: container,
            permit,
            type_state: Default::default(),
            key_state: Default::default(),
            _marker: PhantomData,
        }
    }
}

impl<'a, C: AnyContainer + ?Sized, R: Permit, T: TypePermit, K: KeyPermit> Access<'a, C, R, T, K> {
    /// UNSAFE: Caller must ensure some kind of division between jurisdictions of the two permits.
    #[inline(always)]
    unsafe fn unsafe_split<P>(&self, map: impl FnOnce(Self) -> P) -> P {
        map(Self {
            container: self.container,
            permit: self.permit.copy(),
            type_state: self.type_state.clone(),
            key_state: self.key_state.clone(),
            _marker: PhantomData,
        })
    }

    /// UNSAFE: Caller must ensure type division between jurisdictions of the two permits.
    #[inline(always)]
    unsafe fn unsafe_type_split<P: TypePermit>(
        &self,
        type_state: P::State,
    ) -> Access<'a, C, R, P, K> {
        Access {
            container: self.container,
            permit: self.permit.copy(),
            type_state,
            key_state: self.key_state.clone(),
            _marker: PhantomData,
        }
    }

    /// UNSAFE: Caller must ensure key division between jurisdictions of the two permits.
    #[inline(always)]
    unsafe fn unsafe_key_split<P: KeyPermit>(&self, key_state: P::State) -> Access<'a, C, R, T, P> {
        Access {
            container: self.container,
            permit: self.permit.copy(),
            type_state: self.type_state.clone(),
            key_state,
            _marker: PhantomData,
        }
    }

    fn type_transition<P: TypePermit>(
        self,
        map: impl FnOnce(T::State) -> P::State,
    ) -> Access<'a, C, R, P, K> {
        Access {
            type_state: map(self.type_state),
            ..self
        }
    }

    fn key_transition<P: KeyPermit>(
        self,
        map: impl FnOnce(K::State) -> P::State,
    ) -> Access<'a, C, R, T, P> {
        Access {
            key_state: map(self.key_state),
            ..self
        }
    }

    pub fn extend<D: DynItem + ?Sized>(&self, key: Key<Ref<'_>, D>) -> Key<Ref<'a>, D> {
        // SAFETY: We have 'a lifetime guarantee no item will be removed so key is valid for 'a.
        unsafe { key.extend() }
    }

    // pub fn none(&self)-> AccessPermit<'a, C, R, T, K>
}

// impl<'a, R, C: ?Sized> AnyPermit<'a, R, C> {
//     pub fn step<T: core::Item>(self) -> Option<AnyPermit<'a, R, C::Sub>>
//     where
//         C: TypeContainer<T>,
//     {
//         let Self { container, permit } = self;
//         container
//             .get()
//             .map(|container| AnyPermit { container, permit })
//     }

//     pub fn step_into(self, index: usize) -> Option<AnyPermit<'a, R, C::Sub>>
//     where
//         C: RegionContainer,
//     {
//         let Self { container, permit } = self;
//         container
//             .get(index)
//             .map(|container| AnyPermit { container, permit })
//     }

//     pub fn step_range(
//         self,
//         range: impl RangeBounds<usize>,
//     ) -> Option<impl Iterator<Item = AnyPermit<'a, R, C::Sub>>>
//     where
//         C: RegionContainer,
//     {
//         let Self { container, permit } = self;
//         Some(container.iter(range)?.map(move |container| AnyPermit {
//             container,
//             permit: permit.access(),
//         }))
//     }
// }

// impl<'a, R, T: core::Item, C: ?Sized> TypePermit<'a, T, R, C> {
//     pub fn step(self) -> Option<TypePermit<'a, T, R, C::Sub>>
//     where
//         C: TypeContainer<T>,
//     {
//         self.permit.step().map(TypePermit::new)
//     }

//     pub fn step_iter(self) -> impl Iterator<Item = SlotPermit<'a, R, core::Ref<'a>, T, C>>
//     where
//         C: LeafContainer<T> + AnyContainer,
//     {
//         // SAFETY: We are only accessing T type items.
//         self.permit.iter()
//     }

//     /// Iterates over valid keys in ascending order.
//     pub fn keys(&self) -> impl Iterator<Item = Key<Ptr, T>> + '_
//     where
//         C: AnyContainer,
//     {
//         self.permit.keys(TypeId::of::<T>()).map(|key| key.assume())
//     }
// }

// impl<'a, R, T: core::DynItem + ?Sized, C: AnyContainer + ?Sized> TypePermit<'a, T, R, C> {
//     pub fn step_into(self, index: usize) -> Option<TypePermit<'a, T, R, C::Sub>>
//     where
//         C: RegionContainer,
//     {
//         self.permit.step_into(index).map(TypePermit::new)
//     }

//     pub fn step_range(
//         self,
//         range: impl RangeBounds<usize>,
//     ) -> Option<impl Iterator<Item = TypePermit<'a, T, R, C::Sub>>>
//     where
//         C: RegionContainer,
//     {
//         Some(self.permit.step_range(range)?.map(TypePermit::new))
//     }
// }

// impl<'a, R, T: core::Item, C: Container<T> + ?Sized> PathPermit<'a, T, R, C> {
//     pub fn step(self) -> Option<PathPermit<'a, T, R, C::Sub>>
//     where
//         C: TypeContainer<T>,
//     {
//         let Self { permit, path } = self;
//         permit.step().map(|permit| PathPermit { permit, path })
//     }
// }

// impl<'a, R, T: DynItem + ?Sized, C: AnyContainer + ?Sized> PathPermit<'a, T, R, C> {
//     pub fn step_into(self, index: usize) -> Option<PathPermit<'a, T, R, C::Sub>>
//     where
//         C: RegionContainer,
//     {
//         let path = self.region().path_of(index);
//         let Self { permit, path } = self.and(path)?;
//         permit
//             .step_into(index)
//             .map(|permit| PathPermit { permit, path })
//     }

//     pub fn step_range(
//         self,
//         range: impl RangeBounds<usize>,
//     ) -> Option<impl Iterator<Item = PathPermit<'a, T, R, C::Sub>>>
//     where
//         C: RegionContainer,
//     {
//         let path_range = self
//             .region()
//             .range_of(self.path)
//             .expect("Path out of Container path");

//         // Intersect ranges, max start bound, min end bound.
//         let start = (*path_range.start()).max(match range.start_bound() {
//             Bound::Included(bound) => *bound,
//             Bound::Excluded(bound) => bound.checked_add(1)?,
//             Bound::Unbounded => 0,
//         });
//         let end = (*path_range.end()).min(match range.end_bound() {
//             Bound::Included(bound) => *bound,
//             Bound::Excluded(bound) => bound.checked_sub(1)?,
//             Bound::Unbounded => usize::MAX,
//         });
//         let range = start..=end;

//         let Self { permit, path } = self;
//         permit.step_range(range).map(|iter| {
//             iter.filter_map(move |permit| {
//                 Some(PathPermit {
//                     path: path.and(permit.container_path())?,
//                     permit,
//                 })
//             })
//         })
//     }
// }

// impl<'a, R, K: Copy, T: core::Item, C: AnyContainer + ?Sized> SlotPermit<'a, R, K, T, C> {
//     pub fn step(self) -> Option<SlotPermit<'a, R, K, T, C::Sub>>
//     where
//         C: TypeContainer<T>,
//     {
//         let Self { permit, key } = self;
//         permit.step().map(|permit| SlotPermit::new(permit, key))
//     }

//     pub fn step_into(self) -> Option<SlotPermit<'a, R, K, T, C::Sub>>
//     where
//         C: RegionContainer,
//     {
//         let Self { permit, key } = self;
//         let index = permit.region().index_of(key.ptr());
//         permit
//             .step_into(index)
//             .map(|permit| SlotPermit::new(permit, key))
//     }
// }

impl<'a, C: AnyContainer + ?Sized, T: TypePermit, K: KeyPermit> Access<'a, C, permit::Ref, T, K> {
    pub fn borrow(&self) -> Access<'a, C, permit::Ref, T, K> {
        self.clone()
    }
}

impl<'a, C: AnyContainer + ?Sized, T: TypePermit, K: KeyPermit> Access<'a, C, Mut, T, K> {
    pub fn borrow_mut(&mut self) -> Access<'_, C, Mut, T, K> {
        Access {
            container: self.container,
            // SAFETY: We are borrowing exclusive access to self.
            permit: unsafe { self.permit.copy() }.into(),
            type_state: self.type_state.clone(),
            key_state: self.key_state.clone(),
            _marker: PhantomData,
        }
    }

    pub fn as_ref(&self) -> Access<'_, C, permit::Ref, T, K> {
        Access {
            container: self.container,
            permit: self.permit.borrow(),
            type_state: self.type_state.clone(),
            key_state: self.key_state.clone(),
            _marker: PhantomData,
        }
    }
}

impl<'a, C: AnyContainer + ?Sized, R: Permit, T: TypePermit, K: KeyPermit> Deref
    for Access<'a, C, R, T, K>
{
    type Target = &'a C;

    fn deref(&self) -> &Self::Target {
        &self.container
    }
}

impl<'a, C: AnyContainer + ?Sized, T: TypePermit, K: KeyPermit> Copy
    for Access<'a, C, permit::Ref, T, K>
where
    T::State: Copy,
    K::State: Copy,
{
}

impl<'a, C: AnyContainer + ?Sized, T: TypePermit, K: KeyPermit> Clone
    for Access<'a, C, permit::Ref, T, K>
{
    fn clone(&self) -> Self {
        Self {
            container: self.container,
            type_state: self.type_state.clone(),
            key_state: self.key_state.clone(),
            permit: self.permit.clone(),
            _marker: PhantomData,
        }
    }
}

impl<'a, C: AnyContainer + ?Sized, T: TypePermit, K: KeyPermit> From<Access<'a, C, Mut, T, K>>
    for Access<'a, C, permit::Ref, T, K>
{
    fn from(permit: Access<'a, C, Mut, T, K>) -> Self {
        Self {
            container: permit.container,
            type_state: permit.type_state,
            key_state: permit.key_state,
            permit: permit.permit.into(),
            _marker: PhantomData,
        }
    }
}
