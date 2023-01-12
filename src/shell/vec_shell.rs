use crate::core::*;
use std::alloc::Global;

pub struct VecShell<T: Item> {
    from: Vec<AnyRef, T::Alloc>,
}

impl<T: Item> Shell for VecShell<T> {
    type T = T;

    type Iter<'a>= impl Iterator<Item = AnyRef> + 'a
    where
        Self: 'a;

    type IterOf<'a, I: Item>=impl Iterator<Item = Ref<I>> + 'a
    where
        Self: 'a;

    fn new_in(alloc: &<Self::T as Item>::Alloc) -> Self {
        Self {
            from: Vec::new_in(alloc.clone()),
        }
    }

    fn iter(&self) -> AscendingIterator<Self::Iter<'_>> {
        AscendingIterator::ascending(self.from.iter().copied())
    }

    fn iter_of<I: Item>(&self) -> AscendingIterator<Self::IterOf<'_, I>> {
        AscendingIterator::ascending(self.from.iter().filter_map(|r| r.downcast::<I>()))
    }

    fn add(&mut self, from: impl Into<AnyKey>, _: &<Self::T as Item>::Alloc) {
        let add = AnyRef::new(from.into());
        let i = match self.from.binary_search(&add) {
            Ok(i) => i,
            Err(i) => i,
        };
        self.from.insert(i, add);
    }

    fn replace(&mut self, from: impl Into<AnyKey>, to: Index, _: &<Self::T as Item>::Alloc) {
        let from = AnyRef::new(from.into());
        let to = AnyRef::new(AnyKey::new_with(to, from.key().metadata()));

        for r in self.from.iter_mut() {
            if r == &from {
                *r = to;
            }
        }
    }

    fn remove(&mut self, from: impl Into<AnyKey>) {
        let remove = AnyRef::new(from.into());
        if let Ok(i) = self.from.binary_search(&remove) {
            self.from.remove(i);
        }
    }

    fn clear(&mut self, alloc: &<Self::T as Item>::Alloc) {
        self.from = Vec::new_in(alloc.clone());
    }
}

impl<T: Item<Alloc = Global>> Default for VecShell<T> {
    fn default() -> Self {
        Self { from: Vec::new() }
    }
}
