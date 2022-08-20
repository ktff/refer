use std::ops::Deref;

use crate::core::*;

pub type RefIter<'a, T: ?Sized + 'static> = impl Iterator<Item = AnyRef> + 'a;

/// T --> T
pub struct Vertice<T: ?Sized + 'static>(Vec<Ref<T, Global, Bi>>);

impl<T: ?Sized + 'static> Vertice<T> {
    pub fn new(refs: Vec<Ref<T, Global, Bi>>) -> Self {
        Vertice(refs)
    }

    /// Connects this --with-> to in collection.
    pub fn connect(
        &mut self,
        collection: &mut impl ShellCollection<T>,
        this: Key<T>,
        to: Key<T>,
    ) -> Result<(), Error> {
        self.0.push(Ref::connect(this, to, collection)?);
        Ok(())
    }

    /// Disconnects this --with-> to in collection.
    /// Panics if index is out of bounds.
    pub fn disconnect(
        &mut self,
        collection: &mut impl ShellCollection<T>,
        this: Key<T>,
        to: usize,
    ) -> Result<(), Error> {
        self[to].disconnect(this, collection)?;
        self.0.remove(to);
        Ok(())
    }

    /// Iterates through T items pointing to this one.
    pub fn iter_from<'a>(
        &self,
        this: &impl RefShell<'a, T = T>,
    ) -> impl Iterator<Item = Key<T>> + 'a {
        this.from()
    }
}

impl<T: ?Sized + 'static> Deref for Vertice<T> {
    type Target = [Ref<T, Global, Bi>];
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
}

impl<T: ?Sized + 'static, D: Directioned> Ref<T, Global, D> {
    pub fn connect<F: ?Sized + 'static>(
        from: Key<F>,
        to: Key<T>,
        collection: &mut impl ShellCollection<T>,
    ) -> Result<Self, Error> {
        let mut to_shell = collection.get_mut(to)?;
        to_shell.add_from(Ref::<F, Global, D>::new(from).into());
        Ok(Self::new(to))
    }

    pub fn disconnect<F: ?Sized + 'static>(
        self,
        from: Key<F>,
        collection: &mut impl ShellCollection<T>,
    ) -> Result<(), Error> {
        let mut to_shell = collection.get_mut(self.key())?;
        to_shell.remove_from(Ref::<F, Global, D>::new(from).into());
        Ok(())
    }
}
