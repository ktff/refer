use crate::core::*;

pub type RefIter<'a, T: ?Sized + 'static> = impl Iterator<Item = AnyRef> + 'a;

/// Connects T <--F--> T.
pub struct Edge<T: ?Sized + 'static>([Ref<T>; 2]);

impl<T: ?Sized + 'static> Edge<T> {
    pub fn new(refs: [Ref<T>; 2]) -> Self {
        Edge(refs)
    }

    /// Panics if a or b don't exist.
    pub fn add(coll: &mut impl ShellCollection<T>, this: AnyKey, a: Key<T>, b: Key<T>) -> Self {
        coll.get_mut(a).expect("Should exist").add_from(this.into());
        coll.get_mut(b).expect("Should exist").add_from(this.into());

        Self([Ref::new(a), Ref::new(b)])
    }

    /// True if everything that should exist existed.
    pub fn remove(self, coll: &mut impl ShellCollection<T>, this: AnyKey) -> bool {
        let mut success_a = false;
        if let Some(a_shell) = coll.get_mut(self.0[0].key()) {
            success_a = a_shell.remove_from(this);
        }

        let mut success_b = false;
        if let Some(b_shell) = coll.get_mut(self.0[1].key()) {
            success_b = b_shell.remove_from(this);
        }

        success_a && success_b
    }
}

impl<T: ?Sized + 'static> Item for Edge<T> {
    type I<'a> = RefIter<'a, T>;

    fn references(&self, _: Index) -> Self::I<'_> {
        self.0.iter().copied().map(Into::into)
    }
}

impl<T: ?Sized + 'static> AnyItem for Edge<T> {
    fn references_any<'a>(&'a self, this: Index) -> Option<Box<dyn Iterator<Item = AnyRef> + 'a>> {
        Some(Box::new(self.references(this)))
    }

    fn remove_reference(&mut self, _: Index, key: AnyKey) -> bool {
        // Both references are crucial so this removes it self.
        debug_assert!(key == self.0[0].key().into() || key == self.0[1].key().into());

        false
    }
}
