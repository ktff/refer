use crate::{
    core::*,
    item::{data, tagged_edge},
};
use std::marker::PhantomData;

// TODO: methods useful for manipulating and using graph
pub type Edge<T, D> = tagged_edge::Edge<D, Vertice<T>>;
pub type Vertice<T> = data::Data<T>;

pub type Id<T> = Key<Vertice<T>>;
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
    C: Collection<Vertice<T>> + Collection<Edge<T, D>>,
>(C, PhantomData<(T, D)>);

impl<
        T: Sync + Send + 'static,
        D: Sync + Send + 'static,
        C: Collection<Vertice<T>> + Collection<Edge<T, D>>,
    > TaggedGraph<T, D, C>
{
    pub fn new(coll: C) -> Self {
        TaggedGraph(coll, PhantomData)
    }
}

impl<
        T: Sync + Send + 'static,
        D: Sync + Send + 'static,
        C: Collection<Vertice<T>> + Collection<Edge<T, D>> + Default,
    > Default for TaggedGraph<T, D, C>
{
    fn default() -> Self {
        TaggedGraph::new(Default::default())
    }
}
