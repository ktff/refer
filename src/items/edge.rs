use std::any::Any;

use crate::core::*;

pub type RefIter<'a, T: ?Sized + 'static> = impl Iterator<Item = AnyRef> + 'a;

/// Connects T <--F--> T.
pub struct Edge<T: ?Sized + 'static>([Ref<T>; 2]);

impl<T: ?Sized + 'static> Edge<T> {
    pub fn new(refs: [Ref<T>; 2]) -> Self {
        Edge(refs)
    }

    /// Some if both shells exist.
    pub fn add<'a>(
        coll: &impl MutShellCollection<'a>,
        this: AnyKey,
        a: Key<T>,
        b: Key<T>,
    ) -> Option<Self> {
        let mut a_shell = coll.get_mut(a)?;
        let mut b_shell = coll.get_mut(b)?;

        a_shell.add_from(this);
        b_shell.add_from(this);

        Some(Self([Ref::new(a), Ref::new(b)]))
    }

    /// True if everything that should exist existed.
    pub fn remove(self, coll: &mut impl ShellCollection, this: AnyKey) -> bool {
        let mut success_a = false;
        if let Some(mut a_shell) = coll.get_mut(self.0[0].key()) {
            success_a = a_shell.remove_from(this);
        }

        let mut success_b = false;
        if let Some(mut b_shell) = coll.get_mut(self.0[1].key()) {
            success_b = b_shell.remove_from(this);
        }

        success_a && success_b
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

    fn remove_reference(&mut self, key: AnyKey, _: &impl Any) -> bool {
        // Both references are crucial so this removes it self.
        debug_assert!(self.0[0].key().upcast() == key || self.0[1].key().upcast() == key);

        false
    }
}
