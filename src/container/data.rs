use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashSet,
};

use crate::core::*;

/// Wraps a `Container` with empty GroupData and passes additional data when fetching items.
pub struct ContainerData<D: Any, C> {
    data: D,
    container: C,
}

impl<D: Any, C> ContainerData<D, C> {
    pub fn new(data: D, container: C) -> Self {
        Self { data, container }
    }

    pub fn inner(&self) -> &C {
        &self.container
    }

    pub fn inner_data(&self) -> &D {
        &self.data
    }
}

impl<D: Any, C: Allocator<T>, T: 'static> Allocator<T> for ContainerData<D, C> {
    type Alloc = C::Alloc;

    type R = C::R;

    fn reserve(&mut self, item: Option<&T>, r: Self::R) -> Option<(ReservedKey<T>, &Self::Alloc)> {
        self.container.reserve(item, r)
    }

    fn cancel(&mut self, key: ReservedKey<T>) {
        self.container.cancel(key)
    }

    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T> {
        self.container.fulfill(key, item)
    }

    fn unfill(&mut self, key: SubKey<T>) -> Option<T>
    where
        T: Sized,
    {
        self.container.unfill(key)
    }
}

impl<D: Any, C: Container<T, GroupItem = ()>, T: AnyItem> Container<T> for ContainerData<D, C> {
    type GroupItem = D;

    type Shell = <C as Container<T>>::Shell;

    type SlotIter<'a> = IterWithData<'a, T, D, C> where Self: 'a;

    fn get_slot(
        &self,
        key: SubKey<T>,
    ) -> Option<((&UnsafeCell<T>, &D), &UnsafeCell<Self::Shell>, &Self::Alloc)> {
        self.container
            .get_slot(key)
            .map(|((item, _), shell, alloc)| ((item, &self.data), shell, alloc))
    }

    unsafe fn iter_slot(&self) -> Option<Self::SlotIter<'_>> {
        self.container.iter_slot().map(|iter| IterWithData {
            iter,
            data: &self.data,
        })
    }
}

impl<D: Any, C: AnyContainer> AnyContainer for ContainerData<D, C> {
    fn any_get_slot(
        &self,
        key: AnySubKey,
    ) -> Option<(
        (&UnsafeCell<dyn AnyItem>, &dyn Any),
        &UnsafeCell<dyn AnyShell>,
        &dyn std::alloc::Allocator,
    )> {
        self.container
            .any_get_slot(key)
            .map(|((item, _), shell, alloc)| ((item, &self.data as &dyn Any), shell, alloc))
    }

    fn unfill_any(&mut self, key: AnySubKey) {
        self.container.unfill_any(key)
    }

    fn first(&self, key: TypeId) -> Option<AnySubKey> {
        self.container.first(key)
    }

    fn next(&self, key: AnySubKey) -> Option<AnySubKey> {
        self.container.next(key)
    }

    fn types(&self) -> HashSet<TypeId> {
        self.container.types()
    }
}

pub struct IterWithData<'a, T: AnyItem, D, C: Container<T>> {
    iter: <C as Container<T>>::SlotIter<'a>,
    data: &'a D,
}

impl<'a, T: AnyItem, D, C: Container<T>> Iterator for IterWithData<'a, T, D, C> {
    type Item = (
        SubKey<T>,
        (&'a UnsafeCell<T>, &'a D),
        &'a UnsafeCell<<C as Container<T>>::Shell>,
        &'a <C as Allocator<T>>::Alloc,
    );

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(key, (item, _), shell, alloc)| (key, (item, self.data), shell, alloc))
    }
}
