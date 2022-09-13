use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashSet,
    marker::PhantomData,
};

use crate::core::*;

/// A pair of containers for type T and U.
pub struct ContainerPair<T, CT, U, CU> {
    pub ct: CT,
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

/// Implements Allocator and Container for type T and U for any ContainerPair
#[macro_export]
macro_rules! impl_container_pair {
    ($t:ty,$u:ty) => {
        impl<CT: $crate::Allocator<$t>, CU: $crate::AnyContainer> $crate::Allocator<$t>
            for $crate::container::ContainerPair<$t, CT, $u, CU>
        {
            type Alloc = CT::Alloc;

            type R = CT::R;

            fn reserve(
                &mut self,
                item: Option<&$t>,
                r: Self::R,
            ) -> Option<($crate::ReservedKey<$t>, &Self::Alloc)> {
                self.ct.reserve(item, r)
            }

            fn cancel(&mut self, key: $crate::ReservedKey<$t>) {
                self.ct.cancel(key)
            }

            fn fulfill(&mut self, key: $crate::ReservedKey<$t>, item: $t) -> $crate::SubKey<$t>
            where
                $t: Sized,
            {
                self.ct.fulfill(key, item)
            }

            fn unfill(&mut self, key: $crate::SubKey<$t>) -> Option<$t>
            where
                $t: Sized,
            {
                self.ct.unfill(key)
            }
        }

        impl<CT: $crate::Container<$t>, CU: $crate::AnyContainer> $crate::Container<$t>
            for $crate::container::ContainerPair<$t, CT, $u, CU>
        {
            type GroupItem = CT::GroupItem;

            type Shell = CT::Shell;

            type SlotIter<'a> = CT::SlotIter<'a> where Self: 'a;

            fn get_slot(
                &self,
                key: $crate::SubKey<$t>,
            ) -> Option<(
                (&std::cell::UnsafeCell<$t>, &Self::GroupItem),
                &std::cell::UnsafeCell<Self::Shell>,
                &Self::Alloc,
            )> {
                self.ct.get_slot(key)
            }

            unsafe fn iter_slot(&self) -> Option<Self::SlotIter<'_>> {
                self.ct.iter_slot()
            }
        }

        impl<CT: $crate::AnyContainer, CU: $crate::Allocator<$u>> $crate::Allocator<$u>
            for $crate::container::ContainerPair<$t, CT, $u, CU>
        {
            type Alloc = CU::Alloc;

            type R = CU::R;

            fn reserve(
                &mut self,
                item: Option<&$u>,
                r: Self::R,
            ) -> Option<($crate::ReservedKey<$u>, &Self::Alloc)> {
                self.cu.reserve(item, r)
            }

            fn cancel(&mut self, key: $crate::ReservedKey<$u>) {
                self.cu.cancel(key)
            }

            fn fulfill(&mut self, key: $crate::ReservedKey<$u>, item: $u) -> $crate::SubKey<$u>
            where
                $u: Sized,
            {
                self.cu.fulfill(key, item)
            }

            fn unfill(&mut self, key: $crate::SubKey<$u>) -> Option<$u>
            where
                $u: Sized,
            {
                self.cu.unfill(key)
            }
        }

        impl<CT: $crate::AnyContainer, CU: $crate::Container<$u>> $crate::Container<$u>
            for $crate::container::ContainerPair<$t, CT, $u, CU>
        {
            type GroupItem = CU::GroupItem;

            type Shell = CU::Shell;

            type SlotIter<'a> = CU::SlotIter<'a> where Self: 'a;

            fn get_slot(
                &self,
                key: $crate::SubKey<$u>,
            ) -> Option<(
                (&std::cell::UnsafeCell<$u>, &Self::GroupItem),
                &std::cell::UnsafeCell<Self::Shell>,
                &Self::Alloc,
            )> {
                self.cu.get_slot(key)
            }

            unsafe fn iter_slot(&self) -> Option<Self::SlotIter<'_>> {
                self.cu.iter_slot()
            }
        }
    };
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
