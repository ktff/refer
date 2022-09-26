use crate::{core::*, item::edge};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

// TODO: methods useful for manipulating and using graph

pub type Id<T, D> = Key<Vertice<T, D>>;
pub type EdgeId<T, D> = Key<Edge<T, D>>;

/// A model of a tagged graph.
///
/// Graph contains vertices and edges between them.
///
/// Vertice can carry data T.
/// Edge can carry data D.
pub struct TaggedGraph<
    T: Sync + Send + 'static,
    D: Sync + Send + 'static,
    C: Collection<Vertice<T, D>> + Collection<Edge<T, D>>,
>(C, PhantomData<(T, D)>);

impl<
        T: Sync + Send + 'static,
        D: Sync + Send + 'static,
        C: Collection<Vertice<T, D>> + Collection<Edge<T, D>>,
    > TaggedGraph<T, D, C>
{
    pub fn new(coll: C) -> Self {
        TaggedGraph(coll, PhantomData)
    }
}

impl<
        T: Sync + Send + 'static,
        D: Sync + Send + 'static,
        C: Collection<Vertice<T, D>> + Collection<Edge<T, D>> + Default,
    > Default for TaggedGraph<T, D, C>
{
    fn default() -> Self {
        TaggedGraph::new(Default::default())
    }
}

pub struct Vertice<T: Sync + Send + 'static, D: Sync + Send + 'static>(T, PhantomData<Edge<T, D>>);

impl<T: Sync + Send + 'static, D: Sync + Send + 'static> Item for Vertice<T, D> {
    type I<'a> = std::iter::Empty<AnyRef>;

    fn references(&self, _: Index) -> Self::I<'_> {
        std::iter::empty()
    }
}

impl<T: Sync + Send + 'static, D: Sync + Send + 'static> AnyItem for Vertice<T, D> {
    fn references_any<'a>(&'a self, _: Index) -> Option<Box<dyn Iterator<Item = AnyRef> + 'a>> {
        None
    }

    fn item_removed(&mut self, _: Index, _: AnyKey) -> bool {
        true
    }

    fn item_moved(&mut self, _: AnyKey, _: AnyKey) {}
}

impl<T: Sync + Send + 'static, D: Sync + Send + 'static> Deref for Vertice<T, D> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Sync + Send + 'static, D: Sync + Send + 'static> DerefMut for Vertice<T, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Edge<T: Sync + Send + 'static, D: Sync + Send + 'static>(D, edge::Edge<Vertice<T, D>>);

impl<T: Sync + Send + 'static, D: Sync + Send + 'static> Item for Edge<T, D> {
    type I<'a> = <edge::Edge<Vertice<T, D>> as Item>::I<'a>;

    fn references(&self, this: Index) -> Self::I<'_> {
        self.1.references(this)
    }
}

impl<T: Sync + Send + 'static, D: Sync + Send + 'static> AnyItem for Edge<T, D> {
    fn references_any<'a>(&'a self, this: Index) -> Option<Box<dyn Iterator<Item = AnyRef> + 'a>> {
        self.1.references_any(this)
    }

    fn item_removed(&mut self, this: Index, key: AnyKey) -> bool {
        self.1.item_removed(this, key)
    }

    fn item_moved(&mut self, from: AnyKey, to: AnyKey) {
        self.1.item_moved(from, to)
    }
}

impl<T: Sync + Send + 'static, D: Sync + Send + 'static> Deref for Edge<T, D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Sync + Send + 'static, D: Sync + Send + 'static> DerefMut for Edge<T, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
