use crate::core::{leaf::LeafContainer, *};
use crate::leaf_container;
use crate::shell::vec_shell::VecShell;
use std::num::NonZeroUsize;
use std::ops::RangeBounds;
use std::{any::TypeId, cell::SyncUnsafeCell, collections::HashSet};

const ONE: NonZeroUsize = NonZeroUsize::new(1).expect("Not zero");

pub struct ItemContainerFamily;

impl<T: Item<Alloc = std::alloc::Global>> ContainerFamily<T> for ItemContainerFamily
where
    T::LocalityData: Default,
{
    type Container = ItemContainer<T>;

    fn new_container(region: RegionPath) -> Self::Container {
        ItemContainer::new(Context::new_default(
            region.path().leaf().expect("Too large path region"),
        ))
    }
}

/// A collection of 1 item.
pub struct ItemContainer<T: Item, S: Shell<T = T> = VecShell<T>> {
    context: Context<T>,
    slot: Option<(SyncUnsafeCell<T>, SyncUnsafeCell<S>)>,
}

impl<T: Item, S: Shell<T = T>> ItemContainer<T, S> {
    pub fn new(context: Context<T>) -> Self {
        Self {
            context,
            slot: None,
        }
    }

    fn key(&self) -> Key<T> {
        self.context.leaf_path().key_of(ONE)
    }
}

impl<T: Item, S: Shell<T = T>> LeafContainer<T> for ItemContainer<T, S> {
    /// Shell of item.
    type Shell = S;

    type Iter<'a>= impl Iterator<Item = (Key<T>, UnsafeSlot<'a, T, Self::Shell>)> + Send
   where
       Self: 'a;

    #[inline(always)]
    fn context(&self) -> &Context<T> {
        &self.context
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
    fn get(&self, index: usize) -> Option<UnsafeSlot<T, Self::Shell>> {
        self.slot
            .as_ref()
            .filter(|_| index == 1)
            .map(|(item, shell)| UnsafeSlot::new(self.context.slot_context(), item, shell))
    }

    fn iter(&self, range: impl RangeBounds<usize>) -> Option<Self::Iter<'_>> {
        self.slot
            .as_ref()
            .filter(|_| range.contains(&1))
            .map(|(item, shell)| {
                std::iter::once((
                    self.key(),
                    UnsafeSlot::new(self.context.slot_context(), item, shell),
                ))
            })
    }

    fn fill(&mut self, item: T) -> std::result::Result<NonZeroUsize, T> {
        if self.slot.is_some() {
            Err(item)
        } else {
            self.slot = Some((
                SyncUnsafeCell::new(item),
                SyncUnsafeCell::new(S::new_in(self.context.allocator())),
            ));
            Ok(ONE)
        }
    }

    fn unfill(&mut self, index: usize) -> Option<(T, Self::Shell)> {
        if index == 1 {
            self.slot
                .take()
                .map(|(item, shell)| (item.into_inner(), shell.into_inner()))
        } else {
            None
        }
    }
}

impl<T: Item, S: Shell<T = T>> Container<T> for ItemContainer<T, S> {
    leaf_container!(impl Container<T>);
}

impl<T: Item, S: Shell<T = T>> AnyContainer for ItemContainer<T, S> {
    leaf_container!(impl AnyContainer<T>);
}

impl<T: Item, S: Shell<T = T>> Drop for ItemContainer<T, S> {
    leaf_container!(impl Drop<T>);
}

#[cfg(feature = "base_u64")]
impl<T: Item<Alloc = std::alloc::Global>> Default for ItemContainer<T>
where
    T::LocalityData: Default,
{
    fn default() -> Self {
        Self::new(Context::new_default(
            LeafPath::new(Path::default()).expect("Base index larger than usize"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill() {
        let mut container = ItemContainer::default();

        let item = 42;
        let key = container.fill(item).unwrap();

        assert_eq!(
            unsafe { &*container.get(key.get()).unwrap().item().get() },
            &item
        );
        assert!(container
            .iter(..)
            .into_iter()
            .flatten()
            .map(|(key, slot)| (key, unsafe { &*slot.item().get() }))
            .eq(Some((container.context.leaf_path().key_of(key), &item))));
    }

    #[test]
    fn unfill() {
        let mut container = ItemContainer::default();

        let item = 42;
        let key = container.fill(item).unwrap();

        assert_eq!(
            unsafe { &*container.get(key.get()).unwrap().item().get() },
            &item
        );

        assert_eq!(
            container.unfill(key.get()).map(|(item, _)| item),
            Some(item)
        );

        assert!(container.get(key.get()).is_none());
        assert_eq!(container.iter(..).into_iter().flatten().count(), 0);
    }
}
