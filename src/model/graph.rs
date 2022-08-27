use crate::{collection::vec::VecCollection, core::*, item::vertice::Vertice as Node};
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
pub struct Graph<T: 'static = (), C: Collection<Vertice<T>> = VecCollection<Vertice<T>>>(
    C,
    PhantomData<T>,
);

impl<T: 'static, C: Collection<Vertice<T>>> Graph<T, C> {
    pub fn new(coll: C) -> Self {
        Graph(coll, PhantomData)
    }
}

impl<T: 'static, C: Collection<Vertice<T>> + Default> Default for Graph<T, C> {
    fn default() -> Self {
        Graph::new(Default::default())
    }
}

pub struct Vertice<T: 'static>(T, Node<Self>);

impl<T: 'static> Item for Vertice<T> {
    type I<'a> = <Node<Self> as Item>::I<'a>;

    fn references(&self) -> Self::I<'_> {
        self.1.references()
    }
}

impl<T: 'static> AnyItem for Vertice<T> {
    fn references_any<'a>(&'a self) -> Box<dyn Iterator<Item = AnyRef> + 'a> {
        self.1.references_any()
    }

    fn remove_reference(&mut self, key: AnyKey) -> bool {
        self.1.remove_reference(key)
    }
}

impl<T: 'static> Deref for Vertice<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static> DerefMut for Vertice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
