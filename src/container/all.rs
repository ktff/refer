use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use crate::core::*;

use super::vec::VecContainerFamily;

/// A container of all types backed by container family F.
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
            ((&**c) as &dyn Any)
                .downcast_ref()
                .expect("Should be correct type")
        })
    }

    fn coll_mut<T: AnyItem>(&mut self) -> Option<&mut F::C<T>> {
        self.collections.get_mut(&TypeId::of::<T>()).map(|c| {
            ((&mut **c) as &mut dyn Any)
                .downcast_mut()
                .expect("Should be correct type")
        })
    }

    fn coll_or_insert<T: AnyItem>(&mut self) -> &mut F::C<T> {
        let key_len = self.key_len;
        (&mut **self
            .collections
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(F::new::<T>(key_len))) as &mut dyn Any)
            .downcast_mut::<F::C<T>>()
            .expect("Should be correct type")
    }
}

impl<T: AnyItem, F: ContainerFamily> Allocator<T> for AllContainer<F>
where
    F::C<T>: Allocator<T>,
{
    fn reserve(&mut self, item: &T) -> Option<ReservedKey<T>> {
        self.coll_or_insert::<T>().reserve(item)
    }

    fn cancel(&mut self, key: ReservedKey<T>) {
        self.coll_mut::<T>()
            .expect("Invalid reserved key")
            .cancel(key);
    }

    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T>
    where
        T: Sized,
    {
        self.coll_mut::<T>()
            .expect("Invalid reserved key")
            .fulfill(key, item)
    }

    fn unfill(&mut self, key: SubKey<T>) -> Option<T>
    where
        T: Sized,
    {
        self.coll_mut::<T>()?.unfill(key)
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
        self.coll::<T>()?.get_slot(key)
    }

    unsafe fn iter_slot(&self) -> Option<Self::SlotIter<'_>> {
        self.coll::<T>()?.iter_slot()
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

impl<F: ContainerFamily> Default for AllContainer<F> {
    fn default() -> Self {
        Self::new(MAX_KEY_LEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::owned::Owned;
    use std::any::Any;

    #[test]
    fn allocate_multi_type_item() {
        let mut container = Owned::new(AllContainer::<VecContainerFamily>::default());

        let key_a = container.add(42).unwrap();
        let key_b = container.add(true).unwrap();
        let key_c = container.add("Hello").unwrap();

        assert_eq!(container.get(key_a).map(|(item, _)| item), Some(&42));
        assert_eq!(container.get(key_b).map(|(item, _)| item), Some(&true));
        assert_eq!(container.get(key_c).map(|(item, _)| item), Some(&"Hello"));
    }

    #[test]
    fn get_any() {
        let mut container = Owned::new(AllContainer::<VecContainerFamily>::default());

        let key_a = container.add(42).unwrap();
        let key_b = container.add(true).unwrap();
        let key_c = container.add("Hello").unwrap();

        assert_eq!(
            (container
                .split_item_any(key_a.into())
                .map(|(item, _)| item)
                .unwrap() as &dyn Any)
                .downcast_ref(),
            Some(&42)
        );
        assert_eq!(
            (container
                .split_item_any(key_b.into())
                .map(|(item, _)| item)
                .unwrap() as &dyn Any)
                .downcast_ref(),
            Some(&true)
        );
        assert_eq!(
            (container
                .split_item_any(key_c.into())
                .map(|(item, _)| item)
                .unwrap() as &dyn Any)
                .downcast_ref(),
            Some(&"Hello")
        );
    }

    #[test]
    fn unfill_any() {
        let mut container = Owned::new(AllContainer::<VecContainerFamily>::default());

        let key_a = container.add(42).unwrap();
        let key_b = container.add(true).unwrap();
        let key_c = container.add("Hello").unwrap();

        assert_eq!(container.take(key_b), Some(true));

        assert_eq!(container.get(key_a).map(|(item, _)| item), Some(&42));
        assert!(container.get(key_b).is_none());
        assert_eq!(container.get(key_c).map(|(item, _)| item), Some(&"Hello"));
    }
}
