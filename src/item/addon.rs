use crate::core::*;
use auto_enums::auto_enum;
use std::ops::{Deref, DerefMut};

/// T: n --> T
#[derive(Debug)]
pub struct Addon<T: Sync + Send + 'static, const N: usize, D: DynItem + ?Sized> {
    inner: T,
    drains: [Grc<D>; N],
}

impl<T: Sync + Send + 'static, const N: usize, D: DynItem + ?Sized> Addon<T, N, D> {
    pub fn new(inner: T, drains: [Grc<D>; N]) -> Self {
        Self { inner, drains }
    }

    pub fn drains(&self) -> &[Grc<D>; N] {
        &self.drains
    }
}

impl<T: Sync + Send + 'static, const N: usize, D: DynItem + ?Sized> Item for Addon<T, N, D> {
    type Alloc = std::alloc::Global;

    type LocalityData = ();

    type Edges<'a> = impl Iterator<Item = PartialEdge<Key<Ref<'a>>>>;

    const TRAITS: ItemTraits<Self> = &[];

    #[auto_enum(Iterator)]
    fn edges(&self, _: ItemLocality<'_, Self>, side: Option<Side>) -> Self::Edges<'_> {
        let drains = self
            .drains
            .iter()
            .map(|drain| Side::Source.object(drain.borrow().any()));

        match side {
            Some(Side::Source) => std::iter::empty(),
            Some(Side::Drain) => drains,
            None => drains,
        }
    }

    /// Should remove edge and return object ref.
    /// Ok success.
    /// Err if can't remove it, which may cause for this item to be removed.
    #[must_use]
    fn try_remove_edge<D2: DynItem + ?Sized>(
        &mut self,
        _: ItemLocality<'_, Self>,
        this: Key<Owned, Self>,
        PartialEdge { subject, object }: PartialEdge<Key<Ptr, D2>>,
    ) -> Result<Key<Owned, D2>, (Found, Key<Owned, Self>)> {
        match subject {
            Side::Drain => Err((Found::No, this)),
            Side::Source => Err((
                Found::found(self.drains.iter().any(|drain| *drain == object)),
                this,
            )),
        }
    }

    fn localized_drop(self, _: ItemLocality<'_, Self>) -> Vec<PartialEdge<Key<Owned>>> {
        self.drains
            .into_iter()
            .map(|drain| Side::Source.object(drain.into_owned_key().any()))
            .collect()
    }

    // item_traits_method!(Addon<T, E>: dyn std::fmt::Debug);
}

impl<T: Sync + Send + 'static, const N: usize, D: DynItem + ?Sized> Deref for Addon<T, N, D> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Sync + Send + 'static, const N: usize, D: DynItem + ?Sized> DerefMut for Addon<T, N, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
