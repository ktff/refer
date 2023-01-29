use crate::{
    core::{
        container::{ContainerFamily, LeafContainer},
        *,
    },
    leaf_container,
    shell::vec_shell::VecShell,
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
pub struct VecContainer<T: Item, S: Shell<T = T> = VecShell<T>> {
    locality: Locality<T>,
    slots: Vec<Slot<T, S>, T::Alloc>,
    free_head: Option<NonZeroUsize>,
    count: usize,
}

impl<T: Item, S: Shell<T = T>> VecContainer<T, S> {
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
        self.slots.capacity() * std::mem::size_of::<Slot<T, S>>()
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
                Slot::Some(_, _) => Some(start.saturating_add(i)),
            })
            .next()
    }

    fn iter_slice<'a>(
        &'a self,
        start: Bound<usize>,
        end: Bound<usize>,
    ) -> impl Iterator<Item = (NonZeroUsize, UnsafeSlot<'a, T, S>)> + Send {
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
                if let Slot::Some(ref item, ref shell) = slot {
                    Some((
                        NonZeroUsize::new(start + i).expect("Zero index was allocated"),
                        UnsafeSlot::new(self.locality.slot_locality(), item, shell),
                    ))
                } else {
                    None
                }
            })
    }
}

impl<T: Item, S: Shell<T = T>> LeafContainer<T> for VecContainer<T, S> {
    type Shell = S;

    type Iter<'a>= impl Iterator<Item = (NonZeroUsize, UnsafeSlot<'a, T, Self::Shell>)> + Send
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
                Slot::Some(_, _) => Some(i),
            })
            .and_then(NonZeroUsize::new)
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<UnsafeSlot<T, Self::Shell>> {
        self.slots.get(index).and_then(|slot| {
            if let Slot::Some(item, shell) = slot {
                Some(UnsafeSlot::new(self.locality.slot_locality(), item, shell))
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
                Slot::Some(
                    SyncUnsafeCell::new(item),
                    SyncUnsafeCell::new(S::new_in(self.locality.allocator())),
                ),
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
                self.slots.push(Slot::Some(
                    SyncUnsafeCell::new(item),
                    SyncUnsafeCell::new(S::new_in(self.locality.allocator())),
                ));
                self.count += 1;
                Ok(index)
            } else {
                // Out of keys
                Err(item)
            }
        }
    }

    /// Removes from container.
    fn unfill(&mut self, index: usize) -> Option<(T, Self::Shell)> {
        self.slots.get_mut(index).and_then(|slot| {
            match std::mem::replace(slot, Slot::None(self.free_head)) {
                Slot::Some(item, shell) => {
                    self.count -= 1;
                    self.free_head =
                        Some(NonZeroUsize::new(index).expect("Zero index was allocated"));
                    Some((item.into_inner(), shell.into_inner()))
                }
                other => {
                    *slot = other;
                    None
                }
            }
        })
    }
}

impl<T: Item, S: Shell<T = T>> Container<T> for VecContainer<T, S> {
    leaf_container!(impl Container<T>);
}

impl<T: Item, S: Shell<T = T>> AnyContainer for VecContainer<T, S> {
    leaf_container!(impl AnyContainer<T>);
}

impl<T: Item, S: Shell<T = T>> Drop for VecContainer<T, S> {
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

enum Slot<T, S> {
    None(Option<NonZeroUsize>),
    Some(SyncUnsafeCell<T>, SyncUnsafeCell<S>),
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
        assert_eq!(container.unfill(index.get()).unwrap().0, item);
        assert!(container.get(index.get()).is_none());
    }

    #[test]
    fn iter() {
        let n = 20;
        let mut container = VecContainer::default();

        let mut indices = (0..n)
            .map(|i| (container.fill(i).unwrap(), i))
            .collect::<Vec<_>>();

        indices.sort();

        assert_eq!(
            indices,
            container
                .iter(..)
                .map(|(key, slot)| (key, unsafe { *slot.item().get() }))
                .collect::<Vec<_>>()
        );
    }
}
