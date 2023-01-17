use crate::core::*;
use log::*;
use std::marker::PhantomData;

/// No shell.
/// Panics if anything tries to add reference.
#[derive(Debug, Clone, Copy)]
pub struct NoShell<T: Item>(PhantomData<T>);

/// A shell of an item. In which references are recorded.
impl<T: Item> Shell for NoShell<T> {
    type T = T;

    type Iter<'a> = impl Iterator<Item = AnyRef> + 'a
    where
        Self: 'a;

    fn new_in(_: &<Self::T as Item>::Alloc) -> Self {
        Self::default()
    }

    fn iter(&self) -> AscendingIterator<Self::Iter<'_>> {
        AscendingIterator::ascending(std::iter::empty())
    }

    fn add(&mut self, from: impl Into<AnyKey>, _: &<Self::T as Item>::Alloc) {
        error!("NoShell::add(from: {:?}) called", from.into());
    }

    fn replace(&mut self, _: impl Into<AnyKey>, _: AnyKey, _: &<Self::T as Item>::Alloc) {}

    fn remove(&mut self, _: impl Into<AnyKey>) {}

    fn clear(&mut self, _: &<Self::T as Item>::Alloc) {}
}

impl<T: Item> Default for NoShell<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
