use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashSet,
    marker::PhantomData,
};

use crate::core::*;

/// A pair of containers for type T and U.
pub struct ContainerPair<T, CT, U, CU> {
    ct: CT,
    /// Needs to be implemented manually.
    pub cu: CU,
    _t: PhantomData<T>,
    _u: PhantomData<U>,
}

impl<T, CT, U, CU> ContainerPair<T, CT, U, CU> {
    /// Creates a new pair of containers.
    pub fn new(ct: CT, cu: CU) -> Self {
        Self {
            ct,
            cu,
            _t: PhantomData,
            _u: PhantomData,
        }
    }
}

// TODO: How to enable default impl for U?
impl<T: 'static, CT: Allocator<T>, U, CU: AnyContainer> Allocator<T>
    for ContainerPair<T, CT, U, CU>
{
    type Alloc = CT::Alloc;

    type R = CT::R;

    fn reserve(&mut self, item: Option<&T>, r: Self::R) -> Option<(ReservedKey<T>, &Self::Alloc)> {
        self.ct.reserve(item, r)
    }

    fn cancel(&mut self, key: ReservedKey<T>) {
        self.ct.cancel(key)
    }

    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T> {
        self.ct.fulfill(key, item)
    }

    fn unfill(&mut self, key: SubKey<T>) -> Option<T>
    where
        T: Sized,
    {
        self.ct.unfill(key)
    }
}

impl<T: AnyItem, CT: Container<T>, U: AnyItem, CU: AnyContainer> Container<T>
    for ContainerPair<T, CT, U, CU>
{
    type GroupItem = CT::GroupItem;

    type Shell = CT::Shell;

    type SlotIter<'a> = CT::SlotIter<'a> where Self: 'a;

    fn get_slot(
        &self,
        key: SubKey<T>,
    ) -> Option<(
        (&UnsafeCell<T>, &Self::GroupItem),
        &UnsafeCell<Self::Shell>,
        &Self::Alloc,
    )> {
        self.ct.get_slot(key)
    }

    unsafe fn iter_slot(&self) -> Option<Self::SlotIter<'_>> {
        self.ct.iter_slot()
    }
}

impl<T: AnyItem, CT: AnyContainer, U: AnyItem, CU: AnyContainer> AnyContainer
    for ContainerPair<T, CT, U, CU>
{
    fn any_get_slot(
        &self,
        key: AnySubKey,
    ) -> Option<(
        (&UnsafeCell<dyn AnyItem>, &dyn Any),
        &UnsafeCell<dyn AnyShell>,
        &dyn std::alloc::Allocator,
    )> {
        if TypeId::of::<T>() == key.type_id() {
            self.ct.any_get_slot(key)
        } else if TypeId::of::<U>() == key.type_id() {
            self.cu.any_get_slot(key)
        } else {
            None
        }
    }

    fn unfill_any(&mut self, key: AnySubKey) {
        if TypeId::of::<T>() == key.type_id() {
            self.ct.unfill_any(key)
        } else if TypeId::of::<U>() == key.type_id() {
            self.cu.unfill_any(key)
        }
    }

    fn first(&self, key: TypeId) -> Option<AnySubKey> {
        if TypeId::of::<T>() == key {
            self.ct.first(key)
        } else if TypeId::of::<U>() == key {
            self.cu.first(key)
        } else {
            None
        }
    }

    fn next(&self, key: AnySubKey) -> Option<AnySubKey> {
        if TypeId::of::<T>() == key.type_id() {
            self.ct.next(key)
        } else if TypeId::of::<U>() == key.type_id() {
            self.cu.next(key)
        } else {
            None
        }
    }

    fn types(&self) -> HashSet<TypeId> {
        let mut set = HashSet::new();
        set.insert(TypeId::of::<T>());
        set.insert(TypeId::of::<U>());
        set
    }
}
