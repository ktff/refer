use crate::core::*;
use std::{
    alloc,
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashSet,
    num::NonZeroU64,
};

use super::item::{SizedShell, Slot};

pub type SlotIter<'a, T: AnyItem, S: Shell<T = T> + Default, A: alloc::Allocator + 'static> =
    impl Iterator<Item = (SubKey<T>, (&'a UnsafeCell<T>, &'a ()), &'a UnsafeCell<S>)>;

pub struct VecContainerFamily;

impl ContainerFamily for VecContainerFamily {
    type C<T: AnyItem> = VecContainer<T>;

    fn new<T: AnyItem>(key_len: u32) -> Self::C<T> {
        VecContainer::new(key_len)
    }
}

/// A simple vec container of items of the same type.
pub struct VecContainer<
    T: 'static,
    S: Shell<T = T> + Default = SizedShell<T>,
    A: alloc::Allocator + 'static = alloc::Global,
> {
    slots: Vec<Slot<T, S>, A>,
    free: Vec<Index, A>,
    key_len: u32,
}

impl<T: 'static, S: Shell<T = T> + Default> VecContainer<T, S, alloc::Global> {
    pub fn new(key_len: u32) -> Self {
        Self {
            slots: vec![Slot::Free],
            free: Vec::new(),
            key_len: key_len,
        }
    }
}

impl<T: 'static, S: Shell<T = T> + Default, A: alloc::Allocator + Clone + 'static>
    VecContainer<T, S, A>
{
    pub fn new_in(key_len: u32, allocator: A) -> Self {
        Self {
            slots: Vec::new_in(allocator.clone()),
            free: Vec::new_in(allocator),
            key_len: key_len,
        }
    }
}

impl<T: 'static, S: Shell<T = T> + Default, A: alloc::Allocator + 'static> VecContainer<T, S, A> {
    fn first_from(&self, i: usize) -> Option<SubKey<T>> {
        self.slots
            .iter()
            .enumerate()
            .skip(i)
            .filter_map(|(i, slot)| match slot {
                Slot::Free | Slot::Reserved => None,
                Slot::Filled { .. } => Some(SubKey::new(
                    self.key_len,
                    Index(NonZeroU64::new(i as u64).expect("Zero index is allocated")),
                )),
            })
            .next()
    }
}

impl<T: 'static, S: Shell<T = T> + Default, A: alloc::Allocator + 'static> Allocator<T>
    for VecContainer<T, S, A>
{
    fn reserve(&mut self, _: &T) -> Option<ReservedKey<T>> {
        let index = if let Some(index) = self.free.pop() {
            debug_assert!(matches!(self.slots[index.as_usize()], Slot::Free));
            index
        } else {
            if self.slots.len().checked_shr(self.key_len).unwrap_or(0) >= 1 {
                // Out of keys
                return None;
            }

            let index = Index(NonZeroU64::new(self.slots.len() as u64).expect("Zero index"));
            self.slots.push(Slot::Free);
            index
        };

        self.slots[index.as_usize()].reserve();
        Some(ReservedKey::new(SubKey::new(self.key_len, index)))
    }

    fn cancel(&mut self, key: ReservedKey<T>) {
        let index = key.take().index(self.key_len);
        self.slots[index.as_usize()].cancel();
        self.free.push(index);
    }

    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T> {
        let key = key.take();
        dbg!(key);
        self.slots[key.index(self.key_len).as_usize()].fulfill(item);

        key
    }

    fn unfill(&mut self, key: SubKey<T>) -> Option<T>
    where
        T: Sized,
    {
        let index = key.index(self.key_len);

        self.slots[index.as_usize()].unfill().map(|item| {
            self.free.push(index);
            item
        })
    }
}

impl<T: AnyItem, S: Shell<T = T> + Default, A: alloc::Allocator + 'static> !Sync
    for VecContainer<T, S, A>
{
}

impl<T: AnyItem, S: Shell<T = T> + Default, A: alloc::Allocator + 'static> Container<T>
    for VecContainer<T, S, A>
{
    type GroupItem = ();

    type Shell = S;

    type SlotIter<'a> = SlotIter<'a, T, S, A> where Self: 'a;

    fn get_slot(
        &self,
        key: SubKey<T>,
    ) -> Option<((&UnsafeCell<T>, &()), &UnsafeCell<Self::Shell>)> {
        let i = key.index(self.key_len).as_usize();
        let slot = self.slots.get(i)?;
        match slot {
            Slot::Free | Slot::Reserved => None,
            Slot::Filled { item, shell } => Some(((item, &()), shell)),
        }
    }

    unsafe fn iter_slot(&self) -> Option<Self::SlotIter<'_>> {
        // This is safe since Vec::iter guarantees that each element
        // is returned only once.
        Some(
            self.slots
                .iter()
                .enumerate()
                .filter_map(|(i, slot)| match slot {
                    Slot::Free | Slot::Reserved => None,
                    Slot::Filled { item, shell } => Some((
                        SubKey::new(
                            self.key_len,
                            Index(NonZeroU64::new(i as u64).expect("Zero index")),
                        ),
                        (item, &()),
                        shell,
                    )),
                }),
        )
    }
}

impl<T: AnyItem, S: Shell<T = T> + Default, A: alloc::Allocator + 'static> AnyContainer
    for VecContainer<T, S, A>
{
    fn any_get_slot(
        &self,
        key: AnySubKey,
    ) -> Option<(
        (&UnsafeCell<dyn AnyItem>, &dyn Any),
        &UnsafeCell<dyn AnyShell>,
    )> {
        let i = key.downcast::<T>()?.index(self.key_len).as_usize();

        let slot = self.slots.get(i)?;
        match slot {
            Slot::Free | Slot::Reserved => None,
            Slot::Filled { item, shell } => Some(((item, &()), shell)),
        }
    }

    fn unfill_any(&mut self, key: AnySubKey) {
        if let Some(key) = key.downcast() {
            self.unfill(key);
        }
    }

    fn first(&self, key: TypeId) -> Option<AnySubKey> {
        if key == TypeId::of::<T>() {
            self.first_from(0).map(|key| key.into())
        } else {
            None
        }
    }

    fn next(&self, key: AnySubKey) -> Option<AnySubKey> {
        if let Some(key) = key.downcast::<T>() {
            self.first_from(key.index(self.key_len).as_usize() + 1)
                .map(|key| key.into())
        } else {
            None
        }
    }

    fn types(&self) -> HashSet<TypeId> {
        let mut set = HashSet::new();
        set.insert(TypeId::of::<T>());
        set
    }
}

impl<T: 'static, S: Shell<T = T> + Default> Default for VecContainer<T, S, alloc::Global> {
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
    fn add_items() {
        let n = 20;
        let mut container = Owned::new(VecContainer::<usize>::new(64));

        let keys = (0..n)
            .map(|i| container.add(i).unwrap())
            .collect::<Vec<_>>();

        for (i, key) in keys.iter().enumerate() {
            assert_eq!(container.get(*key).unwrap().0, &i);
        }
    }

    #[test]
    fn reserve_cancel() {
        let mut container = Owned::new(VecContainer::<usize>::new(1));

        let item = 42;
        let key = container.reserve(&item).unwrap();
        assert!(container.reserve(&item).is_none());

        container.cancel(key);
        assert!(container.reserve(&item).is_some());
    }

    #[test]
    fn add_unfill() {
        let mut container = Owned::new(VecContainer::<usize>::new(10));

        let item = 42;
        let key = container.add(item).unwrap();

        assert_eq!(container.items().get(key), Some(&item));
        assert_eq!(container.unfill(key.into()), Some(item));
        assert_eq!(container.items().get(key), None);
    }

    #[test]
    fn iter() {
        let n = 20;
        let mut container = Owned::new(VecContainer::<usize>::new(10));

        let mut keys = (0..n)
            .map(|i| (container.add(i).unwrap(), i))
            .collect::<Vec<_>>();

        keys.sort();

        assert_eq!(
            keys,
            container
                .items()
                .iter()
                .map(|(key, &item)| (key, item))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn get_any() {
        let mut container = Owned::new(VecContainer::<usize>::new(10));

        let item = 42;
        let key = container.add(item).unwrap();

        assert_eq!(
            (container.items_mut().get_any(key.into()).unwrap() as &dyn Any)
                .downcast_ref::<usize>(),
            Some(&item)
        );
    }

    #[test]
    fn unfill_any() {
        let mut container = VecContainer::<usize>::new(10);

        let item = 42;
        let key = container.reserve(&item).unwrap();
        let key = container.fulfill(key, item);

        container.unfill_any(key.into());
        assert!(container.get_slot(key.into()).is_none());
    }

    #[test]
    fn iter_keys() {
        let n = 20;
        let mut container = Owned::new(VecContainer::<usize>::new(8));

        let mut keys = (0..n)
            .map(|i| container.add(i).unwrap().into())
            .collect::<Vec<AnyKey>>();

        keys.sort();

        let any_keys = std::iter::successors(container.first(keys[0].type_id()), |key| {
            container.next(*key)
        })
        .take(30)
        .collect::<Vec<_>>();

        assert_eq!(keys, any_keys);
    }
}
