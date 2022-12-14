use std::ops::Deref;

use crate::core::*;

pub type RefIter<'a, T: AnyItem + ?Sized> = impl Iterator<Item = AnyRef> + 'a;

/// F --> T
pub struct Vertice<T: AnyItem + ?Sized>(Vec<Ref<T>>);

impl<T: AnyItem + ?Sized> Vertice<T> {
    pub fn new(refs: Vec<Ref<T>>) -> Self {
        Vertice(refs)
    }

    /// Connects this -> to in collection.
    pub fn connect(&mut self, collection: &mut impl ShellsMut<T>, this: AnyKey, to: Key<T>) {
        self.0.push(Ref::connect(this, to, collection));
    }

    /// Disconnects this -> to in collection.
    /// Panics if index is out of bounds.
    pub fn disconnect(&mut self, collection: &mut impl ShellsMut<T>, this: AnyKey, to: usize) {
        self[to].disconnect(this, collection);
        self.0.remove(to);
    }

    /// Iterates through T items pointing to this one.
    pub fn iter_from<'a>(&self, this: &'a impl Shell<T = T>) -> impl Iterator<Item = Key<T>> + 'a {
        this.from()
    }
}

impl<T: AnyItem + ?Sized> Deref for Vertice<T> {
    type Target = [Ref<T>];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: AnyItem + ?Sized> Item for Vertice<T> {
    type I<'a> = RefIter<'a, T>;

    fn references(&self, _: Index) -> Self::I<'_> {
        self.0.iter().copied().map(Into::into)
    }
}

impl<T: AnyItem + ?Sized> AnyItem for Vertice<T> {
    fn references_any<'a>(&'a self, this: Index) -> Option<Box<dyn Iterator<Item = AnyRef> + 'a>> {
        Some(Box::new(self.references(this)))
    }

    fn item_removed(&mut self, _: Index, key: AnyKey) -> bool {
        self.0.retain(|rf| rf.key().upcast() != key);

        true
    }
}

impl<T: AnyItem + ?Sized> Default for Vertice<T> {
    fn default() -> Self {
        Vertice(Vec::default())
    }
}
