use std::ops::Deref;

use crate::core::*;

pub type RefIter<'a, T: ?Sized + 'static> = impl Iterator<Item = AnyRef> + 'a;

/// F --> T
pub struct Vertice<T: ?Sized + 'static>(Vec<Ref<T>>);

impl<T: ?Sized + 'static> Vertice<T> {
    pub fn new(refs: Vec<Ref<T>>) -> Self {
        Vertice(refs)
    }

    /// Connects this --with-> to in collection.
    /// Fails if any of the shells don't exist.
    pub fn connect(
        &mut self,
        collection: &mut impl ShellCollection<T>,
        this: AnyKey,
        to: Key<T>,
    ) -> Option<()> {
        self.0.push(Ref::connect(this, to, collection)?);
        Some(())
    }

    /// Disconnects this --with-> to in collection.
    /// Panics if index is out of bounds.
    /// True if removed. False if there was nothing to remove.
    pub fn disconnect(
        &mut self,
        collection: &mut impl ShellCollection<T>,
        this: AnyKey,
        to: usize,
    ) -> bool {
        let success = self[to].disconnect(this, collection);
        self.0.remove(to);
        success
    }

    /// Iterates through T items pointing to this one.
    pub fn iter_from<'a>(&self, this: &'a impl Shell<T = T>) -> impl Iterator<Item = Key<T>> + 'a {
        this.from()
    }
}

impl<T: ?Sized + 'static> Deref for Vertice<T> {
    type Target = [Ref<T>];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ?Sized + 'static> Item for Vertice<T> {
    type I<'a> = RefIter<'a, T>;

    fn references(&self) -> Self::I<'_> {
        self.0.iter().copied().map(Into::into)
    }
}

impl<T: ?Sized + 'static> AnyItem for Vertice<T> {
    fn references_any<'a>(&'a self) -> Box<dyn Iterator<Item = AnyRef> + 'a> {
        Box::new(self.references())
    }

    fn remove_reference(&mut self, key: AnyKey) -> bool {
        let key = key.downcast::<T>().expect("Key is not of T");
        let (i, _) = self
            .0
            .iter()
            .enumerate()
            .find(|(_, ref_)| ref_.key() == key)
            .expect("Key is not in Vertice");
        self.0.remove(i);
        true
    }
}
