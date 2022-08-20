use std::ops::Deref;

use crate::core::*;

/// T --> T
pub struct Vertice<T: ?Sized + 'static>(Vec<Ref<T, Global, Bi>>);

impl<T: ?Sized + 'static> Vertice<T> {
    pub fn new() -> Self {
        Vertice(Vec::new())
    }

    /// Connects from/self --with-> to in collection.
    pub fn connect(
        &mut self,
        from: Key<T>,
        to: Key<T>,
        collection: &mut impl ShellCollection<T>,
    ) -> Result<(), Error> {
        self.0.push(Ref::connect(from, to, collection)?);
        Ok(())
    }

    /// Disconnects from/self --with-> to in collection.
    /// Panics if index is out of bounds.
    pub fn disconnect(
        &mut self,
        from: Key<T>,
        index: usize,
        collection: &mut impl ShellCollection<T>,
    ) -> Result<(), Error> {
        self[index].disconnect(from, collection)?;
        self.0.remove(index);
        Ok(())
    }

    /// Iterates through T items pointing to this one.
    pub fn iter_from<'a>(
        &self,
        shell: &impl RefShell<'a, T = T>,
    ) -> impl Iterator<Item = Key<T>> + 'a {
        shell.from()
    }
}

impl<T: ?Sized + 'static> Deref for Vertice<T> {
    type Target = [Ref<T, Global, Bi>];
    fn deref(&self) -> &Self::Target {
        &self.0
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
