use crate::core::*;
use auto_enums::auto_enum;
use std::ops::{Deref, DerefMut};

/// Item --> Data<T>
#[derive(Debug)]
pub struct Data<T: Sync + Send + 'static> {
    inner: T,
    sources: Vec<Key<Owned>>,
    owners: usize,
}

impl<T: Sync + Send + 'static> Data<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            sources: Vec::new(),
            owners: 0,
        }
    }

    pub fn sources(&self) -> &[Key<Owned>] {
        &self.sources
    }
}

impl<T: Sync + Send + 'static> Item for Data<T> {
    type Alloc = std::alloc::Global;

    type LocalityData = ();

    type Edges<'a> = impl Iterator<Item = PartialEdge<Key<Ref<'a>>>>;

    const TRAITS: ItemTraits<Self> = &[];

    #[auto_enum(Iterator)]
    fn edges(&self, _: ItemLocality<'_, Self>, side: Option<Side>) -> Self::Edges<'_> {
        let sources = self
            .sources
            .iter()
            .map(|source| Side::Drain.object(source.borrow().any()));

        match side {
            Some(Side::Source) => sources,
            Some(Side::Drain) => std::iter::empty(),
            None => sources,
        }
    }

    /// Should remove edge and return object ref.
    /// Ok success.
    /// Err if can't remove it, which may cause for this item to be removed.
    #[must_use]
    fn try_remove_edge<D: DynItem + ?Sized>(
        &mut self,
        _: ItemLocality<'_, Self>,
        this: Key<Owned, Self>,
        PartialEdge { subject, object }: PartialEdge<Key<Ptr, D>>,
    ) -> Result<Key<Owned, D>, (Found, Key<Owned, Self>)> {
        match subject {
            // Find first occurrence of object in sources and remove it
            Side::Drain => {
                if let Some(index) = self.sources.iter().position(|source| *source == object) {
                    Ok(self.sources.remove(index).assume())
                } else {
                    Err((Found::No, this))
                }
            }
            // Find first occurrence of object in drains and remove it
            Side::Source => Err((Found::No, this)),
        }
    }

    fn localized_drop(self, _: ItemLocality<'_, Self>) -> Vec<PartialEdge<Key<Owned>>> {
        self.sources
            .into_iter()
            .map(|source| Side::Drain.object(source))
            .collect()
    }

    // item_traits_method!(Vertice<T>: dyn std::fmt::Debug);
}

unsafe impl<T: Sync + Send + 'static> DrainItem for Data<T> {
    /// SAFETY: add_drain_edge MUST ensure to add PartialEdge{object: source,side: Side::Drain} to edges of self.
    #[must_use]
    fn add_drain_edge<D: DynItem + ?Sized>(
        &mut self,
        _: ItemLocality<'_, Self>,
        source: Key<Owned, D>,
    ) {
        self.sources.push(source.any());
    }
}

/// Item that doesn't depend on any edge so it can have Key<Owned> without edges.
impl<T: Sync + Send + 'static> StandaloneItem for Data<T> {
    #[must_use]
    fn inc_owners(&mut self, locality: ItemLocality<'_, Self>) -> Grc<Self> {
        self.owners.checked_add(1).expect("Grc overflow");
        // SAFETY: We've just incremented counter.
        unsafe { Grc::new(locality.owned_key()) }
    }

    fn dec_owners(&mut self, locality: ItemLocality<'_, Self>, this: Grc<Self>) {
        assert_eq!(locality.path(), *this);
        self.owners.checked_sub(1).expect("Grc underflow");
        std::mem::forget(this.into_owned_key());
    }

    /// True if there is counted Owned somewhere.
    fn has_owner(&self, _: ItemLocality<'_, Self>) -> bool {
        self.owners > 0
    }
}

impl<T: Sync + Send + 'static> Deref for Data<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Sync + Send + 'static> DerefMut for Data<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
