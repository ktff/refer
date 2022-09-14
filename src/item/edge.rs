use std::{any, fmt};

use crate::core::*;

pub type RefIter<'a, T: 'static> = impl Iterator<Item = AnyRef> + 'a;

/// Connects T <--Edge--> T.
pub struct Edge<T: 'static>([Ref<T>; 2]);

impl<T: 'static> Edge<T> {
    pub fn new(refs: [Ref<T>; 2]) -> Self {
        Edge(refs)
    }

    /// Panics if a or b don't exist.
    pub fn connect(coll: &mut impl ShellsMut<T>, this: AnyKey, a: Key<T>, b: Key<T>) -> Self {
        Self([
            Ref::connect(this.into(), a, coll),
            Ref::connect(this.into(), b, coll),
        ])
    }

    pub fn disconnect(self, coll: &mut impl ShellsMut<T>, this: AnyKey) {
        for rf in self.0 {
            rf.disconnect(this, coll);
        }
    }
}

impl<T: 'static> Item for Edge<T> {
    type I<'a> = RefIter<'a, T>;

    fn references(&self, _: Index) -> Self::I<'_> {
        self.0.iter().copied().map(Into::into)
    }
}

impl<T: 'static> AnyItem for Edge<T> {
    fn references_any<'a>(&'a self, this: Index) -> Option<Box<dyn Iterator<Item = AnyRef> + 'a>> {
        Some(Box::new(self.references(this)))
    }

    fn item_removed(&mut self, _: Index, key: AnyKey) -> bool {
        // Both references are crucial
        key != self.0[0].key().into() && key != self.0[1].key().into()
    }
}

impl<T: 'static> fmt::Debug for Edge<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Edge<{}>({:?} -- {:?})",
            any::type_name::<T>(),
            self.0[0],
            self.0[1]
        )
    }
}
