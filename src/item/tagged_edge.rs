use crate::{core::*, item::edge};
use std::ops::{Deref, DerefMut};

pub struct Edge<D: Sync + Send + 'static, I: AnyItem>(D, edge::Edge<I>);

impl<D: Sync + Send + 'static, I: AnyItem> Edge<D, I> {
    pub fn new(data: D, edge: [Ref<I>; 2]) -> Self {
        Edge(data, edge::Edge::new(edge))
    }
}

impl<D: Sync + Send + 'static, I: AnyItem> Item for Edge<D, I> {
    type I<'a> = <edge::Edge<I> as Item>::I<'a>;

    fn references(&self, this: Index) -> Self::I<'_> {
        self.1.references(this)
    }
}

impl<D: Sync + Send + 'static, I: AnyItem> AnyItem for Edge<D, I> {
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

impl<D: Sync + Send + 'static, I: AnyItem> Deref for Edge<D, I> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<D: Sync + Send + 'static, I: AnyItem> DerefMut for Edge<D, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
