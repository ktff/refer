use std::ops::{Deref, DerefMut};

use crate::core::*;

/// Adds EdgeComponent to Item which allows for
/// Self -> Item edges.
///
/// Passes through:
/// - impl Item
#[derive(Debug)]
pub struct Drain<T = ()> {
    inner: T,
    sources: Vec<Key<Owned>>,
}

impl<T: Item> Drain<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            sources: Vec::new(),
        }
    }

    pub fn sources(&self) -> &[Key<Owned>] {
        &self.sources
    }
}

impl<T> Deref for Drain<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Drain<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Item> Item for Drain<T> {
    type Alloc = T::Alloc;
    type LocalityData = T::LocalityData;

    type Edges<'a> = impl Iterator<Item = Key<Ref<'a>>>;

    fn iter_edges(&self, locality: ItemLocality<'_, Self>) -> Self::Edges<'_> {
        self.inner
            .iter_edges(locality.map_type())
            .chain(self.sources.iter().map(|key| key.borrow()))
    }

    fn remove_edges<D: DynItem + ?Sized>(
        &mut self,
        locality: ItemLocality<'_, Self>,
        target: Key<Ptr, D>,
    ) -> Option<Removed<D>> {
        let multi_key = match self.inner.remove_edges(locality.map_type(), target) {
            Some(Removed::No(key)) => return Some(Removed::No(key)),
            Some(Removed::Yes(multi_key)) => Some(multi_key),
            None => None,
        };

        self.sources
            .extract_if(.., |source| *source == target)
            .fold(multi_key, |owned: Option<MultiOwned<D>>, key| {
                if let Some(mut owned) = owned {
                    owned.add(key.assume());
                    Some(owned)
                } else {
                    Some(key.assume().into())
                }
            })
            .map(Removed::Yes)
    }

    fn localized_drop(mut self, locality: ItemLocality<'_, Self>) -> Vec<Key<Owned>> {
        let mut keys = self.inner.localized_drop(locality.map_type());
        keys.extend(self.sources.drain(..));
        keys
    }
}

impl<T: Item> EdgeContainer for Drain<T> {
    default fn add_edge(&mut self, _: ItemLocality<'_, Self>, _: (), source: Key<Owned>) {
        self.sources.push(source);
    }

    default fn remove_edge(
        &mut self,
        _: ItemLocality<'_, Self>,
        _: (),
        source: Key<Ptr>,
    ) -> Option<Key<Owned>> {
        // Find first occurrence of source in sources and remove it
        let index = self.sources.iter().position(|s| *s == source)?;
        Some(self.sources.remove(index))
    }
}
