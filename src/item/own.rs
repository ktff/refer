use crate::core::*;
use std::ops::{Deref, DerefMut};

/// Adds StandaloneItem to Item which allows for
/// * -> Self edges.
///
/// Passes through:
/// - impl Item
/// - impl EdgeContainer<*,*>
pub struct Own<T> {
    inner: T,
    owners: usize,
}

impl<T: Item + EdgeContainer> Own<T> {
    pub fn new(inner: T) -> Self {
        Self { inner, owners: 0 }
    }
}

impl<T> Deref for Own<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T> DerefMut for Own<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Item> Item for Own<T> {
    type Alloc = T::Alloc;
    type LocalityData = T::LocalityData;

    type Edges<'a> = impl Iterator<Item = Key<Ref<'a>>>;

    fn iter_edges(&self, locality: ItemLocality<'_, Self>) -> Self::Edges<'_> {
        self.inner.iter_edges(locality.map_type())
    }

    fn remove_edges<D: DynItem + ?Sized>(
        &mut self,
        locality: ItemLocality<'_, Self>,
        target: Key<Ptr, D>,
    ) -> Option<Removed<D>> {
        self.inner.remove_edges(locality.map_type(), target)
    }

    fn localized_drop(self, locality: ItemLocality<'_, Self>) -> Vec<Key<Owned>> {
        self.inner.localized_drop(locality.map_type())
    }
}

impl<T: EdgeContainer<D, O>, D, O: DynItem + ?Sized> EdgeContainer<D, O> for Own<T> {
    fn add_edge(&mut self, locality: ItemLocality<'_, Self>, data: D, target: Key<Owned, O>) {
        self.inner.add_edge(locality.map_type(), data, target);
    }

    fn remove_edge(
        &mut self,
        locality: ItemLocality<'_, Self>,
        data: D,
        target: Key<Ptr, O>,
    ) -> Option<Key<Owned, O>> {
        self.inner.remove_edge(locality.map_type(), data, target)
    }
}

/// Item that doesn't depend on any edge so it can have Key<Owned> without edges.
impl<T: EdgeContainer> StandaloneItem for Own<T> {
    fn inc_owners(&mut self, locality: ItemLocality<'_, Self>) -> Grc<Self> {
        self.owners = self.owners.checked_add(1).expect("Grc overflow");
        // SAFETY: We've just incremented counter.
        unsafe { Grc::new(locality.owned_key()) }
    }

    fn dec_owners(&mut self, locality: ItemLocality<'_, Self>, this: Grc<Self>) {
        assert_eq!(locality.path(), *this);
        self.owners = self.owners.checked_sub(1).expect("Grc underflow");
        std::mem::forget(this.into_owned_key());
    }

    /// True if there is counted Owned somewhere.
    fn has_owner(&self, _: ItemLocality<'_, Self>) -> bool {
        self.owners > 0
    }
}
