use std::{any, fmt};

use crate::core::*;

pub type RefIter<'a, T: AnyItem> = impl Iterator<Item = AnyRef> + 'a;

/// Connects T <--Edge--> T.
pub struct Edge<T: AnyItem>([Ref<T>; 2]);

impl<T: AnyItem> Edge<T> {
    pub fn new(refs: [Ref<T>; 2]) -> Self {
        Edge(refs)
    }

    /// Panics if a or b don't exist.
    pub fn connect(
        mut coll: MutShells<T, impl Container<T>>,
        this: AnyKey,
        a: Key<T>,
        b: Key<T>,
    ) -> Self {
        Self([
            Ref::connect(this.into(), a, coll.borrow_mut()),
            Ref::connect(this.into(), b, coll),
        ])
    }

    pub fn disconnect(self, mut coll: MutShells<T, impl Container<T>>, this: AnyKey) {
        for rf in self.0 {
            rf.disconnect(this, coll.borrow_mut());
        }
    }

    pub fn refs(&self) -> &[Ref<T>; 2] {
        &self.0
    }
}

impl<T: AnyItem> Item for Edge<T> {
    type I<'a> = RefIter<'a, T>;

    fn references(&self, _: Index) -> Self::I<'_> {
        self.0.iter().copied().map(Into::into)
    }
}

impl<T: AnyItem> AnyItem for Edge<T> {
    fn references_any<'a>(&'a self, this: Index) -> Option<Box<dyn Iterator<Item = AnyRef> + 'a>> {
        Some(Box::new(self.references(this)))
    }

    fn item_removed(&mut self, _: Index, key: AnyKey) -> bool {
        // Both references are crucial
        key != self.0[0].key().into() && key != self.0[1].key().into()
    }

    fn item_moved(&mut self, old: AnyKey, new: AnyKey) {
        if let Some(old) = old.downcast::<T>() {
            for rf in &mut self.0 {
                if rf.key() == old {
                    *rf = Ref::new(new.downcast().expect("New key is not T"));
                }
            }
        }
    }
}

impl<T: AnyItem> fmt::Debug for Edge<T> {
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
