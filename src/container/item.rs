use crate::core::{
    container::{ContainerFamily, LeafContainer},
    *,
};
use crate::leaf_container;
use std::num::NonZeroUsize;
use std::ops::RangeBounds;
use std::{any::TypeId, cell::SyncUnsafeCell};

const ONE: NonZeroUsize = NonZeroUsize::new(1).expect("Not zero");

#[derive(Default)]
pub struct ItemContainerFamily;

impl<T: Item<Alloc = std::alloc::Global>> ContainerFamily<T> for ItemContainerFamily
where
    T::LocalityData: Default,
{
    type Container = ItemContainer<T>;

    fn new_container(&mut self, region: Path) -> Self::Container {
        ItemContainer::new(Locality::new_default(
            region.leaf().expect("Too large path region"),
        ))
    }
}

/// A collection of 1 item.
pub struct ItemContainer<T: Item> {
    locality: Locality<T>,
    slot: Option<SyncUnsafeCell<T>>,
}

impl<T: Item> ItemContainer<T> {
    pub fn new(locality: Locality<T>) -> Self {
        Self {
            locality,
            slot: None,
        }
    }
}

unsafe impl<T: Item> LeafContainer<T> for ItemContainer<T> {
    type Iter<'a>= impl Iterator<Item = UnsafeSlot<'a, T>> + Send
   where
       Self: 'a;

    #[inline(always)]
    fn locality(&self) -> &Locality<T> {
        &self.locality
    }

    fn first(&self) -> Option<NonZeroUsize> {
        self.slot.as_ref().map(|_| ONE)
    }

    fn next(&self, _: NonZeroUsize) -> Option<NonZeroUsize> {
        None
    }

    fn last(&self) -> Option<NonZeroUsize> {
        self.slot.as_ref().map(|_| ONE)
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<UnsafeSlot<T>> {
        self.slot
            .as_ref()
            .filter(|_| index == 1)
            .map(|item| UnsafeSlot::new(self.locality.index_locality(ONE), item))
    }

    #[inline(always)]
    fn contains(&self, index: usize) -> bool {
        self.slot.as_ref().filter(|_| index == 1).is_some()
    }

    fn iter(&self, range: impl RangeBounds<usize>) -> Self::Iter<'_> {
        self.slot
            .as_ref()
            .filter(|_| range.contains(&1))
            .map(|item| UnsafeSlot::new(self.locality.index_locality(ONE), item))
            .into_iter()
    }

    fn fill(&mut self, item: T) -> std::result::Result<NonZeroUsize, T> {
        if self.slot.is_some() {
            Err(item)
        } else {
            self.slot = Some(SyncUnsafeCell::new(item));
            Ok(ONE)
        }
    }

    fn unfill(&mut self, index: usize) -> Option<T> {
        if index == 1 {
            self.slot.take().map(|item| item.into_inner())
        } else {
            None
        }
    }
}

unsafe impl<T: Item> Container<T> for ItemContainer<T> {
    leaf_container!(impl Container<T>);
}

unsafe impl<T: Item> AnyContainer for ItemContainer<T> {
    leaf_container!(impl AnyContainer<T>);
}

impl<T: Item> Drop for ItemContainer<T> {
    leaf_container!(impl Drop<T>);
}

#[cfg(feature = "base_u64")]
impl<T: Item<Alloc = std::alloc::Global>> Default for ItemContainer<T>
where
    T::LocalityData: Default,
{
    fn default() -> Self {
        Self::new(Locality::new_default(
            LeafPath::new(Path::default()).expect("Base index larger than usize"),
        ))
    }
}

#[cfg(all(test, feature = "base_u64"))]
mod tests {
    use super::*;

    #[test]
    fn fill() {
        let mut container = ItemContainer::default();

        let item = 42;
        let key = container.fill(item).unwrap();

        assert_eq!(
            unsafe { *container.get(key.get()).unwrap().item().get() },
            item
        );
        assert!(container
            .iter(..)
            .map(|slot| (slot.locality().path().index().get(), unsafe {
                *slot.item().get()
            }))
            .eq(Some((key.get() as u64, item))));
    }

    #[test]
    fn unfill() {
        let mut container = ItemContainer::default();

        let item = 42;
        let key = container.fill(item).unwrap();

        assert_eq!(
            unsafe { *container.get(key.get()).unwrap().item().get() },
            item
        );

        assert_eq!(container.unfill(key.get()), Some(item));

        assert!(container.get(key.get()).is_none());
        assert_eq!(container.iter(..).count(), 0);
    }
}
