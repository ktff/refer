use crate::core::*;
use auto_enums::auto_enum;
use std::ops::{Deref, DerefMut};

/// Vertice<T>: *E--> Vertice<T>
/// Item          --> Vertice<T>
#[derive(Debug)]
pub struct Vertice<T: Sync + Send + 'static, E: Sync + Send + 'static = ()> {
    inner: T,
    drains: Vec<(E, Key<Owned, Self>)>,
    sources: Vec<Key<Owned>>,
    owners: usize,
}

impl<T: Sync + Send + 'static, E: Sync + Send + 'static> Vertice<T, E> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            drains: Vec::new(),
            sources: Vec::new(),
            owners: 0,
        }
    }

    pub fn connect(source: &mut MutSlot<Self>, data: E, drain: &mut MutSlot<Self>) {
        // SAFETY: Drain key is added to drains which is exposed as Item::edges
        let drain_key = unsafe { source.locality().add_drain(drain) };
        source.drains.push((data, drain_key));
    }

    /// Disconnects edge at index.
    /// Panics if index is out of bounds.
    pub fn disconnect(
        source: &mut MutSlot<Self>,
        edge: usize,
        drains: ObjectAccess<impl Container<Self>, Self>,
    ) -> E {
        let (data, drain) = source.drains.remove(edge);
        let mut object = drains
            .key_try(drain.borrow())
            .expect("Should have access to everything but source")
            .get();
        source.locality().remove_from_drain(drain, &mut object);

        data
    }

    pub fn sources(&self) -> &[Key<Owned>] {
        &self.sources
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

impl<T: Sync + Send + 'static, E: Sync + Send + 'static> Item for Vertice<T, E> {
    type Alloc = std::alloc::Global;

    type LocalityData = ();

    type Edges<'a> = impl Iterator<Item = PartialEdge<Key<Ref<'a>>>>;

    const TRAITS: ItemTraits<Self> = &[];

    #[auto_enum(Iterator)]
    fn edges(&self, _: ItemLocality<'_, Self>, side: Option<Side>) -> Self::Edges<'_> {
        let drains = self
            .drains
            .iter()
            .map(|(_, drain)| Side::Source.object(drain.borrow().any()));
        let sources = self
            .sources
            .iter()
            .map(|source| Side::Drain.object(source.borrow().any()));

        match side {
            Some(Side::Source) => sources,
            Some(Side::Drain) => drains,
            Some(Side::Bi) => std::iter::empty(),
            None => drains.chain(sources),
        }
    }

    /// Should remove applicable (source,drain,bi) edges and return object refs.
    /// Ok success.
    /// Err if can't remove it, which may cause for this item to be removed.
    #[must_use]
    fn try_remove_edges<D: DynItem + ?Sized>(
        &mut self,
        _: ItemLocality<'_, Self>,
        this: Key<Owned, Self>,
        PartialEdge { subject, object }: PartialEdge<Key<Ptr, D>>,
    ) -> Result<MultiOwned<D>, (Found, Key<Owned, Self>)> {
        match subject {
            // Find all occurrence of object in sources and remove them
            Side::Drain => self
                .sources
                .drain_filter(|source| *source == object)
                .fold(None, |owned: Option<MultiOwned>, key| {
                    if let Some(mut owned) = owned {
                        owned.add(key);
                        Some(owned)
                    } else {
                        Some(key.into())
                    }
                })
                .map(|owned| owned.assume())
                .ok_or((Found::No, this)),
            // Find all occurrence of object in drains and remove them
            Side::Source => self
                .drains
                .drain_filter(|(_, drain)| *drain == object)
                .fold(None, |owned: Option<MultiOwned>, key| {
                    if let Some(mut owned) = owned {
                        owned.add(key.1.any());
                        Some(owned)
                    } else {
                        Some(key.1.any().into())
                    }
                })
                .map(|owned| owned.assume())
                .ok_or((Found::No, this)),
            Side::Bi => Err((Found::No, this)),
        }
    }

    fn localized_drop(self, _: ItemLocality<'_, Self>) -> Vec<PartialEdge<Key<Owned>>> {
        self.sources
            .into_iter()
            .map(|source| Side::Drain.object(source))
            .chain(
                self.drains
                    .into_iter()
                    .map(|(_, drain)| Side::Source.object(drain.any())),
            )
            .collect()
    }

    // item_traits_method!(Vertice<T, E>: dyn std::fmt::Debug);
}

unsafe impl<T: Sync + Send + 'static, E: Sync + Send + 'static> DrainItem for Vertice<T, E> {
    /// SAFETY: add_drain_edge MUST ensure to add PartialEdge{object: source,side: Side::Drain} to edges of self.
    #[must_use]
    fn add_drain_edge<D: DynItem + ?Sized>(
        &mut self,
        _: ItemLocality<'_, Self>,
        source: Key<Owned, D>,
    ) {
        self.sources.push(source.any());
    }

    /// Removes drain edge and returns object ref.
    /// Ok success.
    /// Err if doesn't exist.
    #[must_use]
    fn try_remove_drain_edge<D: DynItem + ?Sized>(
        &mut self,
        _: ItemLocality<'_, Self>,
        this: Key<Owned, Self>,
        source: Key<Ptr, D>,
    ) -> Result<Key<Owned, D>, Key<Owned, Self>> {
        // Find first occurrence of source in sources and remove it
        if let Some(index) = self.sources.iter().position(|s| *s == source.any()) {
            Ok(self.sources.remove(index).assume())
        } else {
            Err(this)
        }
    }
}

/// Item that doesn't depend on any edge so it can have Key<Owned> without edges.
impl<T: Sync + Send + 'static, E: Sync + Send + 'static> StandaloneItem for Vertice<T, E> {
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
