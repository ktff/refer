use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use crate::core::*;

use super::vec::VecContainerFamily;

pub struct AllContainer<F: ContainerFamily = VecContainerFamily> {
    /// T -> F::C<T>
    collections: HashMap<TypeId, Box<dyn AnyContainer>>,
    key_len: u32,
    _family: PhantomData<F>,
}

impl<F: ContainerFamily> AllContainer<F> {
    pub fn new(key_len: u32) -> Self {
        Self {
            collections: HashMap::new(),
            key_len,
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
        let key_len = self.key_len;
        (self
            .collections
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(F::new::<T>(key_len))) as &mut dyn Any)
            .downcast_mut()
            .expect("Should be correct type")
    }
}

impl<T: AnyItem, F: ContainerFamily> Allocator<T> for AllContainer<F>
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

impl<F: ContainerFamily> !Sync for AllContainer<F> {}

impl<T: AnyItem, F: ContainerFamily> Container<T> for AllContainer<F>
where
    F::C<T>: Container<T>,
{
    type Shell = <F::C<T> as Container<T>>::Shell;

    type SlotIter<'a>=<F::C<T> as Container<T>>::SlotIter<'a>
    where
        Self: 'a;

    fn get_slot(&self, key: SubKey<T>) -> Option<(&UnsafeCell<T>, &UnsafeCell<Self::Shell>)> {
        self.coll()?.get_slot(key)
    }

    unsafe fn iter_slot(&self) -> Option<Self::SlotIter<'_>> {
        self.coll()?.iter_slot()
    }
}

impl<F: ContainerFamily> AnyContainer for AllContainer<F> {
    fn any_get_slot(
        &self,
        key: AnySubKey,
    ) -> Option<(&UnsafeCell<dyn AnyItem>, &UnsafeCell<dyn AnyShell>)> {
        self.collections
            .get(&key.type_id())
            .map(|c| &**c)?
            .any_get_slot(key)
    }

    /// Frees if it exists.
    fn unfill_any(&mut self, key: AnySubKey) {
        self.collections
            .get_mut(&key.type_id())
            .map(|c| &mut **c)
            .map(|c| c.unfill_any(key));
    }

    fn first(&self, key: TypeId) -> Option<AnySubKey> {
        self.collections.get(&key).and_then(|c| c.first(key))
    }

    fn next(&self, key: AnySubKey) -> Option<AnySubKey> {
        self.collections
            .get(&key.type_id())
            .and_then(|c| c.next(key))
    }

    fn types(&self) -> HashSet<TypeId> {
        let mut set = HashSet::new();
        for c in self.collections.values() {
            set.extend(c.types());
        }
        set
    }
}
