use refer::*;

/// An different variant of T.
///
/// Disables internal references and key updates.
/// Effectively archiving it.
pub struct Variant<T: Item> {
    /// Variant of
    of: Ref<T>,
    variant: T,
}

impl<T: Item> Variant<T> {
    pub fn new(of: Key<T>, variant: T) -> Self {
        Self {
            of: Ref::new(of),
            variant,
        }
    }

    pub fn of(&self) -> Key<T> {
        self.of.into()
    }

    pub fn variant(&self) -> &T {
        &self.variant
    }
}

impl<T: Item> Item for Variant<T> {
    type I<'a> = impl Iterator<Item = AnyRef> + 'a;

    fn references(&self, _: Index) -> Self::I<'_> {
        Some(self.of.into()).into_iter()
    }
}

impl<T: Item> AnyItem for Variant<T> {
    fn references_any(&self, this: Index) -> Option<Box<dyn Iterator<Item = AnyRef> + '_>> {
        Some(Box::new(self.references(this)))
    }

    fn item_removed(&mut self, _: Index, key: AnyKey) -> bool {
        self.of != key
    }

    fn item_moved(&mut self, old: AnyKey, new: AnyKey) {
        if self.of.key().upcast() == old {
            self.of = Ref::new(new.downcast().expect("Variant::item_moved: type mismatch"));
        }
    }
}
