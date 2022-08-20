use crate::core::*;

pub type RefIter<'a, T: ?Sized + 'static> = impl Iterator<Item = AnyRef> + 'a;

/// Connects T <--F--> T.
pub struct Edge<T: ?Sized + 'static>([Ref<T, Global, Bi>; 2]);

impl<T: ?Sized + 'static> Edge<T> {
    pub fn new(refs: [Ref<T, Global, Bi>; 2]) -> Self {
        Edge(refs)
    }

    pub fn add<'a, F: ?Sized + 'static>(
        coll: &impl MutShellCollection<'a, T>,
        this: Key<F>,
        a: Key<T>,
        b: Key<T>,
    ) -> Result<Self, Error> {
        let mut a_shell = coll.get_mut(a)?;
        let mut b_shell = coll.get_mut(b)?;

        a_shell.add_from(Ref::<F, Global, Bi>::new(this).into());
        b_shell.add_from(Ref::<F, Global, Bi>::new(this).into());

        Ok(Self([Ref::new(a), Ref::new(b)]))
    }

    pub fn remove<'a, F: ?Sized + 'static>(
        self,
        coll: &impl MutShellCollection<'a, T>,
        this: Key<F>,
    ) -> Result<(), Error> {
        let mut a_shell = coll.get_mut(self.0[0].key())?;
        let mut b_shell = coll.get_mut(self.0[1].key())?;

        a_shell.remove_from(Ref::<F, Global, Bi>::new(this).into());
        b_shell.remove_from(Ref::<F, Global, Bi>::new(this).into());

        Ok(())
    }
}

impl<T: ?Sized + 'static> Item for Edge<T> {
    type I<'a> = RefIter<'a, T>;

    fn references(&self) -> Self::I<'_> {
        self.0.iter().copied().map(Into::into)
    }
}

impl<T: ?Sized + 'static> AnyItem for Edge<T> {
    fn references_any<'a>(&'a self) -> Box<dyn Iterator<Item = AnyRef> + 'a> {
        Box::new(self.references())
    }
}
