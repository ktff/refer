use std::ops::{Deref, DerefMut};

use refer::*;

/// Attaches some data D to T
pub struct Attach<T: Item, D: Send + Sync + 'static> {
    /// Attach to
    to: Ref<T>,
    data: D,
}

impl<T: Item, D: Send + Sync + 'static> Attach<T, D> {
    pub fn new(to: Key<T>, data: D) -> Self {
        Self {
            to: Ref::new(to),
            data,
        }
    }

    pub fn to(&self) -> Key<T> {
        self.to.into()
    }
}

impl<T: Item, D: Send + Sync + 'static> Item for Attach<T, D> {
    type I<'a> = impl Iterator<Item = AnyRef> + 'a;

    fn references(&self, _: Index) -> Self::I<'_> {
        Some(self.to.into()).into_iter()
    }
}

impl<T: Item, D: Send + Sync + 'static> AnyItem for Attach<T, D> {
    fn references_any(&self, this: Index) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>> {
        Some(Box::new(self.references(this)))
    }

    fn item_removed(&mut self, _: Index, key: AnyKey) -> bool {
        self.to != key
    }

    fn item_moved(&mut self, old: AnyKey, new: AnyKey) {
        if self.to.key().upcast() == old {
            self.to = Ref::new(new.downcast().expect("Variant::item_moved: type mismatch"));
        }
    }
}

// Deref
impl<T: Item, D: Send + Sync + 'static> Deref for Attach<T, D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

// DerefMut
impl<T: Item, D: Send + Sync + 'static> DerefMut for Attach<T, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
