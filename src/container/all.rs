use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashMap,
    marker::PhantomData,
};

use crate::core::*;

use super::vec::VecContainerFamily;

pub struct AllContainer<F: SizedContainerFamily = VecContainerFamily> {
    /// T -> F::C<T>
    collections: HashMap<TypeId, Box<dyn AnyContainer>>,
    _family: PhantomData<F>,
}

impl<F: SizedContainerFamily> AllContainer<F> {
    pub fn new() -> Self {
        Self {
            collections: HashMap::new(),
            _family: PhantomData,
        }
    }

    fn coll<T: AnyItem>(&self) -> Option<&F::C<T>> {
        self.collections.get(&TypeId::of::<T>()).map(|c| {
            (c as &dyn Any)
                .downcast_ref()
                .expect("Should be correct type")
        })
    }

    fn coll_mut<T: AnyItem>(&mut self) -> Option<&mut F::C<T>> {
        self.collections.get_mut(&TypeId::of::<T>()).map(|c| {
            (c as &mut dyn Any)
                .downcast_mut()
                .expect("Should be correct type")
        })
    }

    fn coll_or_insert<T: AnyItem>(&mut self) -> &mut F::C<T> {
        (self
            .collections
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(F::C::<T>::default())) as &mut dyn Any)
            .downcast_mut()
            .expect("Should be correct type")
    }
}

impl<T: AnyItem, F: SizedContainerFamily> Allocator<T> for AllContainer<F>
where
    F::C<T>: Allocator<T>,
{
    /// Reserves slot for item.
    /// None if collection is out of keys.
    fn reserve(&mut self, item: &T) -> Option<ReservedKey<T>> {
        self.coll_or_insert().reserve(item)
    }

    /// Cancels reservation for item.
    /// Panics if there is no reservation.
    fn cancel(&mut self, key: ReservedKey<T>) {
        self.coll_mut().expect("Invalid reserved key").cancel(key);
    }

    /// Fulfills reservation.
    /// Panics if there is no reservation.
    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T>
    where
        T: Sized,
    {
        self.coll_mut()
            .expect("Invalid reserved key")
            .fulfill(key, item)
    }

    /// Frees and returns item if it exists
    fn unfill(&mut self, key: SubKey<T>) -> Option<T>
    where
        T: Sized,
    {
        self.coll_mut()?.unfill(key)
    }
}

impl<T: AnyItem, F: SizedContainerFamily> Container<T> for AllContainer<F>
where
    F::C<T>: Container<T>,
{
    type Shell = <F::C<T> as Container<T>>::Shell;

    type CellIter<'a>=<F::C<T> as Container<T>>::CellIter<'a>
    where
        Self: 'a;

    fn get_slot(&self, key: SubKey<T>) -> Option<(&UnsafeCell<T>, &UnsafeCell<Self::Shell>)> {
        self.coll()?.get_slot(key)
    }

    fn iter_slot(&self) -> Option<Self::CellIter<'_>> {
        self.coll()?.iter_slot()
    }
}

impl<F: SizedContainerFamily> AnyContainer for AllContainer<F> {
    fn any_get_slot(
        &self,
        key: AnySubKey,
    ) -> Option<(&UnsafeCell<dyn AnyItem>, &UnsafeCell<dyn AnyShell>)> {
        self.collections
            .get(&key.ty_id())
            .map(|c| &**c)?
            .any_get_slot(key)
    }

    /// Frees if it exists.
    fn any_unfill(&mut self, key: AnySubKey) -> bool {
        self.collections
            .get_mut(&key.ty_id())
            .map(|c| &mut **c)
            .map(|c| c.any_unfill(key))
            .unwrap_or(false)
    }
}

impl<F: SizedContainerFamily> KeyContainer for AllContainer<F> {
    // fn prefix(&self) -> Option<Prefix> {
    //     unimplemented!()
    // }

    fn first<I: AnyItem>(&self) -> Option<SubKey<I>> {
        self.coll::<I>()?.first()
    }

    fn next<I: AnyItem>(&self, key: SubKey<I>) -> Option<SubKey<I>> {
        self.coll::<I>()?.next(key)
    }
}

/// Impl default
impl<F: SizedContainerFamily> Default for AllContainer<F> {
    fn default() -> Self {
        Self::new()
    }
}
