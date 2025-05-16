use super::{drain::Drain, itemize::Itemize};
use crate::core::*;
use std::ops::{Deref, DerefMut};

/// Vertice<T>: *E--> Vertice<T>
/// Item          --> Item
#[derive(Debug)]
pub struct Vertice<T: Sync + Send + 'static, E: Sync + Send + 'static = ()> {
    inner: Drain<Itemize<T>>,
    drains: Vec<(E, Key<Owned, Self>)>,
}

impl<T: Sync + Send + 'static, E: Eq + Sync + Send + 'static> Vertice<T, E> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Drain::new(Itemize::new(inner)),
            drains: Vec::new(),
        }
    }

    pub fn connect(source: &mut MutSlot<Self>, data: E, drain: &mut MutSlot<Self>) {
        source.link_any::<E, Self>(data, drain);
    }

    /// Disconnects edge with data to drain.
    /// Panics if index is out of bounds.
    pub fn disconnect(source: &mut MutSlot<Self>, data: E, drain: &mut MutSlot<Self>) {
        source.unlink_any(data, drain);
    }

    pub fn drains(&self) -> &[(E, Key<Owned, Self>)] {
        &self.drains
    }

    pub fn drains_mut(&mut self) -> impl Iterator<Item = (&mut E, Key<Ref<'_>, Self>)> + '_ {
        self.drains
            .iter_mut()
            .map(|(data, drain)| (data, drain.borrow()))
    }

    pub fn get_drain_mut(&mut self, index: usize) -> Option<(&mut E, Key<Ref<'_>, Self>)> {
        self.drains
            .get_mut(index)
            .map(|(data, drain)| (data, drain.borrow()))
    }
}

impl<T: Sync + Send + 'static, E: Sync + Send + 'static> Deref for Vertice<T, E> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Sync + Send + 'static, E: Sync + Send + 'static> DerefMut for Vertice<T, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Sync + Send + 'static, E: Sync + Send + 'static> Item for Vertice<T, E> {
    type Edges<'a> = impl Iterator<Item = Key<Ref<'a>>>;

    fn iter_edges(&self, locality: ItemLocality<'_, Self>) -> Self::Edges<'_> {
        self.drains
            .iter()
            .map(|(_, drain)| drain.borrow().any())
            .chain(self.inner.iter_edges(locality.map_type()))
    }

    /// Err if can't remove it, which may cause for this item to be removed.
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

        self.drains
            .extract_if(.., |(_, drain)| *drain == target)
            .map(|(_, drain)| drain.any())
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

    fn localized_drop(self, locality: ItemLocality<'_, Self>) -> Vec<Key<Owned>> {
        let mut keys = self.inner.localized_drop(locality.map_type());
        keys.extend(self.drains.into_iter().map(|(_, drain)| drain.any()));
        keys
    }

    // item_traits_method!(Vertice<T, E>: dyn std::fmt::Debug);
}

impl<T: Sync + Send + 'static, E: Sync + Send + 'static> EdgeContainer for Vertice<T, E> {
    fn add_edge(&mut self, locality: ItemLocality<'_, Self>, data: (), other: Key<Owned>) {
        self.inner.add_edge(locality.map_type(), data, other);
    }

    fn remove_edge(
        &mut self,
        locality: ItemLocality<'_, Self>,
        data: (),
        other: Key<Ptr>,
    ) -> Option<Key<Owned>> {
        self.inner.remove_edge(locality.map_type(), data, other)
    }
}

impl<T: Sync + Send + 'static, E: Eq + Sync + Send + 'static> EdgeContainer<E, Self>
    for Vertice<T, E>
{
    fn add_edge(&mut self, _: ItemLocality<'_, Self>, data: E, source: Key<Owned, Self>) {
        self.drains.push((data, source));
    }

    fn remove_edge(
        &mut self,
        _: ItemLocality<'_, Self>,
        data: E,
        source: Key<Ptr, Self>,
    ) -> Option<Key<Owned, Self>> {
        // Find first occurrence of source in sources and remove it
        let index = self
            .drains
            .iter()
            .position(|(d, s)| *d == data && *s == source)?;
        Some(self.drains.remove(index).1)
    }
}
