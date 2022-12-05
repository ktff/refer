use crate::{core::*, item::vertice::Vertice as Node};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

// TODO: methods useful for manipulating and using graph

pub type Id<T> = Key<Vertice<T>>;

/// A model of a graph.
///
/// Graph contains vertices and edges between them.
///
/// Vertice can carry data T.
pub struct Graph<T: Sync + Send + 'static, C: Collection<Vertice<T>>>(C, PhantomData<T>);

impl<T: Sync + Send + 'static, C: Collection<Vertice<T>>> Graph<T, C> {
    pub fn new(coll: C) -> Self {
        Graph(coll, PhantomData)
    }
}

impl<T: Sync + Send + 'static, C: Collection<Vertice<T>> + Default> Default for Graph<T, C> {
    fn default() -> Self {
        Graph::new(Default::default())
    }
}

pub struct Vertice<T: Sync + Send + 'static>(T, Node<Self>);

impl<T: Sync + Send + 'static> Item for Vertice<T> {
    type I<'a> = <Node<Self> as Item>::I<'a>;

    fn iter_references(&self, this: Index) -> Self::I<'_> {
        self.1.iter_references(this)
    }
}

impl<T: Sync + Send + 'static> AnyItem for Vertice<T> {
    fn iter_references_any<'a>(
        &'a self,
        this: Index,
    ) -> Option<Box<dyn Iterator<Item = AnyRef> + 'a>> {
        self.1.iter_references_any(this)
    }

    fn remove_reference(&mut self, this: Index, key: AnyKey) -> bool {
        self.1.remove_reference(this, key)
    }

    fn set_reference(&mut self, old: AnyKey, new: AnyKey) {
        self.1.set_reference(old, new)
    }
}

impl<T: Sync + Send + 'static> Deref for Vertice<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Sync + Send + 'static> DerefMut for Vertice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
