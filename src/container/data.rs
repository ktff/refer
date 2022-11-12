use std::{
    any::{Any, TypeId},
    collections::HashSet,
};

use crate::core::*;

/// Wraps a `Container` with empty GroupData and passes additional data when fetching items.
pub struct ContainerData<D: Any, C: Send + Sync> {
    data: D,
    container: C,
}

impl<D: Any, C: Send + Sync> ContainerData<D, C> {
    pub fn new(data: D, container: C) -> Self {
        Self { data, container }
    }

    pub fn inner(&self) -> &C {
        &self.container
    }

    pub fn inner_mut(&mut self) -> &mut C {
        &mut self.container
    }

    pub fn inner_data(&self) -> &D {
        &self.data
    }
}

impl<D: Any + Send + Sync, C: Allocator<T>, T: 'static> Allocator<T> for ContainerData<D, C> {
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

    fn unfill(&mut self, key: SubKey<T>) -> Option<(T, &Self::Alloc)>
    where
        T: Sized,
    {
        self.container.unfill(key)
    }
}

impl<D: Any + Send + Sync, C: Container<T, GroupItem = ()>, T: AnyItem> Container<T>
    for ContainerData<D, C>
{
    type GroupItem = D;

    type Shell = <C as Container<T>>::Shell;

    type SlotIter<'a> = IterWithData<'a, T, D, C> where Self: 'a;

    fn get_slot(&self, key: SubKey<T>) -> Option<UnsafeSlot<T, D, Self::Shell, Self::Alloc>> {
        self.container
            .get_slot(key)
            .map(|slot| slot.with_group_item(&self.data))
    }

    fn iter_slot(&self) -> Option<Self::SlotIter<'_>> {
        self.container.iter_slot().map(|iter| IterWithData {
            iter,
            data: &self.data,
        })
    }
}

impl<D: Any + Send + Sync, C: AnyContainer> AnyContainer for ContainerData<D, C> {
    fn any_get_slot(&self, key: AnySubKey) -> Option<AnyUnsafeSlot> {
        self.container
            .any_get_slot(key)
            .map(|slot| slot.with_group_item(&self.data))
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

pub struct IterWithData<'a, T: AnyItem, D: Any + Send + Sync, C: Container<T, GroupItem = ()>> {
    iter: <C as Container<T>>::SlotIter<'a>,
    data: &'a D,
}

impl<'a, T: AnyItem, D: Any + Send + Sync, C: Container<T, GroupItem = ()>> Iterator
    for IterWithData<'a, T, D, C>
{
    type Item = (
        SubKey<T>,
        UnsafeSlot<'a, T, D, <C as Container<T>>::Shell, <C as Allocator<T>>::Alloc>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(key, slot)| (key, slot.with_group_item(self.data)))
    }
}
