use crate::core::*;
use std::ops::{Deref, DerefMut};

/// Adds Vec collection of edges
/// Self -D-> T
///
/// Passes through:
/// - impl Item
pub struct VecEdges<S = (), D = (), T: DynItem + ?Sized = dyn AnyItem> {
    inner: S,
    targets: Vec<(D, Key<Owned, T>)>,
}

impl<S: Item, D: Eq + Sync + Send + 'static, T: DynItem + ?Sized> VecEdges<S, D, T> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            targets: Vec::new(),
        }
    }
}

impl<S, D, T: DynItem + ?Sized> Deref for VecEdges<S, D, T> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S, D, T: DynItem + ?Sized> DerefMut for VecEdges<S, D, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<S: Item, D: Eq + Sync + Send + 'static, T: DynItem + ?Sized> Item for VecEdges<S, D, T> {
    type Alloc = S::Alloc;
    type LocalityData = S::LocalityData;

    type Edges<'a> = impl Iterator<Item = Key<Ref<'a>>>;

    fn iter_edges(&self, locality: ItemLocality<'_, Self>) -> Self::Edges<'_> {
        self.targets
            .iter()
            .map(|(_, key)| key.borrow().any())
            .chain(self.inner.iter_edges(locality.map_type()))
    }

    fn remove_edges<E: DynItem + ?Sized>(
        &mut self,
        locality: ItemLocality<'_, Self>,
        target: Key<Ptr, E>,
    ) -> Option<Removed<E>> {
        let multi_key = match self.inner.remove_edges(locality.map_type(), target) {
            Some(Removed::No(key)) => return Some(Removed::No(key)),
            Some(Removed::Yes(multi_key)) => Some(multi_key),
            None => None,
        };

        self.targets
            .extract_if(.., |(_, source)| *source == target)
            .fold(
                multi_key,
                |owned: Option<MultiOwned<E>>, (_, key): (D, Key<Owned, T>)| {
                    if let Some(mut owned) = owned {
                        owned.add(key.any().assume());
                        Some(owned)
                    } else {
                        Some(key.any().assume().into())
                    }
                },
            )
            .map(Removed::Yes)
    }

    fn localized_drop(self, locality: ItemLocality<'_, Self>) -> Vec<Key<Owned>> {
        let mut keys = self.inner.localized_drop(locality.map_type());
        keys.extend(self.targets.into_iter().map(|(_, key)| key.any()));
        keys
    }
}

impl<S: Item, D: Eq + Sync + Send + 'static, T: DynItem + ?Sized> EdgeContainer<D, T>
    for VecEdges<S, D, T>
{
    fn add_edge(&mut self, _: ItemLocality<'_, Self>, data: D, target: Key<Owned, T>) {
        self.targets.push((data, target));
    }

    fn remove_edge(
        &mut self,
        _: ItemLocality<'_, Self>,
        data: D,
        target: Key<Ptr, T>,
    ) -> Option<Key<Owned, T>> {
        // Find first occurrence of source in sources and remove it
        let index = self
            .targets
            .iter()
            .position(|(d, s)| *d == data && *s == target)?;
        Some(self.targets.remove(index).1)
    }
}
