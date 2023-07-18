use crate::{
    core::{
        container::{ContainerFamily, LeafContainer},
        *,
    },
    leaf_container,
};
use std::{
    any::TypeId,
    cell::SyncUnsafeCell,
    num::NonZeroUsize,
    ops::{Bound, RangeBounds},
};

#[derive(Default)]
pub struct VecContainerFamily;

impl<T: Item<Alloc = std::alloc::Global>> ContainerFamily<T> for VecContainerFamily
where
    T::LocalityData: Default,
{
    type Container = VecContainer<T>;

    fn new_container(&mut self, region: Path) -> Self::Container {
        VecContainer::new(Locality::new_default(
            region.leaf().expect("Too large path region"),
        ))
    }
}

/// A simple vec container of items of the same type.
/// Allocates by pushing to a vec, if there is no previously freed slot.
pub struct VecContainer<T: Item> {
    locality: Locality<T>,
    slots: Vec<Slot<T>, T::Alloc>,
    free_head: Option<NonZeroUsize>,
    count: usize,
}

impl<T: Item> VecContainer<T> {
    pub fn new(locality: Locality<T>) -> Self {
        let mut slots = Vec::new_in(locality.allocator().clone());
        slots.push(Slot::None(None));
        Self {
            slots,
            free_head: None,
            locality,
            count: 0,
        }
    }

    /// Number items in this collection
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Memory used directly by this container.
    pub fn used_memory(&self) -> usize {
        self.slots.capacity() * std::mem::size_of::<Slot<T>>()
    }

    pub fn shrink_to_fit(&mut self) {
        self.slots.shrink_to_fit();
    }

    fn first_from(&self, start: usize) -> Option<NonZeroUsize> {
        let start = NonZeroUsize::new(start + 1)?;
        self.slots[start.get()..]
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| match slot {
                Slot::None(_) => None,
                Slot::Some(_) => Some(start.saturating_add(i)),
            })
            .next()
    }

    fn iter_slice<'a>(
        &'a self,
        start: Bound<usize>,
        end: Bound<usize>,
    ) -> impl Iterator<Item = UnsafeSlot<'a, T>> + Send {
        let mut start = match start {
            Bound::Included(i) => i,
            Bound::Excluded(i) => i.saturating_add(1),
            Bound::Unbounded => 0,
        };
        let end = match end {
            // slice doesn't support inclusive end of usize::MAX nor the slice can contain it
            // so we need to adjust it to usize::MAX - 1.
            Bound::Included(i) => i.saturating_add(1).min(self.slots.len()),
            Bound::Excluded(i) => i.min(self.slots.len()),
            Bound::Unbounded => self.slots.len(),
        };
        if start > end {
            start = end;
        }

        self.slots[start..end]
            .iter()
            .enumerate()
            .filter_map(move |(i, slot)| {
                if let Slot::Some(ref item) = slot {
                    Some(UnsafeSlot::new(
                        self.locality.index_locality(
                            NonZeroUsize::new(start + i).expect("Zero index was allocated"),
                        ),
                        item,
                    ))
                } else {
                    None
                }
            })
    }
}

unsafe impl<T: Item> LeafContainer<T> for VecContainer<T> {
    type Iter<'a>= impl Iterator<Item = UnsafeSlot<'a, T>> + Send
    where
        Self: 'a;

    #[inline(always)]
    fn locality(&self) -> &Locality<T> {
        &self.locality
    }

    fn first(&self) -> Option<NonZeroUsize> {
        self.first_from(0)
    }

    fn next(&self, after: NonZeroUsize) -> Option<NonZeroUsize> {
        self.first_from(after.get())
    }

    fn last(&self) -> Option<NonZeroUsize> {
        self.slots
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, slot)| match slot {
                Slot::None(_) => None,
                Slot::Some(_) => Some(i),
            })
            .and_then(NonZeroUsize::new)
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<UnsafeSlot<T>> {
        self.slots.get(index).and_then(|slot| {
            if let Slot::Some(item) = slot {
                Some(UnsafeSlot::new(
                    self.locality.index_locality(
                        NonZeroUsize::new(index).expect("Zero index was allocated"),
                    ),
                    item,
                ))
            } else {
                None
            }
        })
    }

    fn iter(&self, range: impl RangeBounds<usize>) -> Self::Iter<'_> {
        self.iter_slice(range.start_bound().cloned(), range.end_bound().cloned())
    }

    fn fill(&mut self, item: T) -> std::result::Result<NonZeroUsize, T> {
        if let Some(index) = self.free_head.take() {
            match std::mem::replace(
                &mut self.slots[index.get()],
                Slot::Some(SyncUnsafeCell::new(item)),
            ) {
                Slot::None(next) => {
                    self.count += 1;
                    self.free_head = next;
                    Ok(index)
                }
                _ => unreachable!(),
            }
        } else {
            let index = NonZeroUsize::new(self.slots.len()).expect("Zero index");
            if self.locality.locality_key().contains(index) {
                self.slots.push(Slot::Some(SyncUnsafeCell::new(item)));
                self.count += 1;
                Ok(index)
            } else {
                // Out of keys
                Err(item)
            }
        }
    }

    /// Removes from container.
    fn unfill(&mut self, index: usize) -> Option<T> {
        self.slots.get_mut(index).and_then(|slot| {
            match std::mem::replace(slot, Slot::None(self.free_head)) {
                Slot::Some(item) => {
                    self.count -= 1;
                    self.free_head =
                        Some(NonZeroUsize::new(index).expect("Zero index was allocated"));
                    Some(item.into_inner())
                }
                other => {
                    *slot = other;
                    None
                }
            }
        })
    }
}

unsafe impl<T: Item> Container<T> for VecContainer<T> {
    leaf_container!(impl Container<T>);
}

unsafe impl<T: Item> AnyContainer for VecContainer<T> {
    leaf_container!(impl AnyContainer<T>);
}

impl<T: Item> Drop for VecContainer<T> {
    leaf_container!(impl Drop<T>);
}

#[cfg(feature = "base_u64")]
impl<T: Item<Alloc = std::alloc::Global>> Default for VecContainer<T>
where
    T::LocalityData: Default,
{
    fn default() -> Self {
        Self::new(Locality::new_default(
            LeafPath::new(Path::default()).expect("Base index larger than usize"),
        ))
    }
}

enum Slot<T> {
    None(Option<NonZeroUsize>),
    Some(SyncUnsafeCell<T>),
}

#[cfg(all(test, feature = "base_u64"))]
mod tests {
    use super::*;

    #[test]
    fn fill() {
        let n = 20;
        let mut container = VecContainer::default();

        let indices = (0..n)
            .map(|i| container.fill(i).unwrap())
            .collect::<Vec<_>>();

        for (i, index) in indices.iter().enumerate() {
            assert_eq!(
                unsafe { *container.get(index.get()).unwrap().item().get() },
                i
            );
        }
    }

    #[test]
    fn unfill() {
        let mut container = VecContainer::default();

        let item = 42;
        let index = container.fill(item).unwrap();

        assert_eq!(
            unsafe { *container.get(index.get()).unwrap().item().get() },
            item
        );
        assert_eq!(container.unfill(index.get()).unwrap(), item);
        assert!(container.get(index.get()).is_none());
    }

    #[test]
    fn iter() {
        let n = 20;
        let mut container = VecContainer::default();

        let mut indices = (0..n)
            .map(|i| (container.fill(i).unwrap().get() as u64, i))
            .collect::<Vec<_>>();

        indices.sort();

        assert_eq!(
            indices,
            container
                .iter(..)
                .map(|slot| (slot.locality().path().index().get(), unsafe {
                    *slot.item().get()
                }))
                .collect::<Vec<_>>()
        );
    }
}
