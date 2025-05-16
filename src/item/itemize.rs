use crate::core::*;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
/// Wrapper that turns regular struct into an item.
///
/// Passes through nothing.
pub struct Itemize<T> {
    inner: T,
}

impl<T> Itemize<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> Deref for Itemize<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Itemize<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Sync + Send + 'static> Item for Itemize<T> {
    type Edges<'a> = impl Iterator<Item = Key<Ref<'a>>>;

    fn iter_edges(&self, _: ItemLocality<'_, Self>) -> Self::Edges<'_> {
        std::iter::empty()
    }

    fn remove_edges<D: DynItem + ?Sized>(
        &mut self,
        _: ItemLocality<'_, Self>,
        _: Key<Ptr, D>,
    ) -> Option<Removed<D>> {
        None
    }

    fn localized_drop(self, _: ItemLocality<'_, Self>) -> Vec<Key<Owned>> {
        Vec::new()
    }
}
