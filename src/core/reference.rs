use crate::core::*;
use std::fmt;

// TODO: Pinned outer references
// TODO: Make ref builded by adding ref fn on recipient.
#[derive(PartialEq, Eq)]
pub struct Ref<T: AnyItem>(Key<T>);

impl<T: AnyItem> Ref<T> {
    pub fn new(key: Key<T>) -> Self {
        Ref(key)
    }

    pub fn key(&self) -> Key<T> {
        self.0
    }
}

impl<T: AnyItem> Ref<T> {
    /// Panics if to doesn't exist.
    pub fn connect(from: AnyKey, to: Key<T>, collection: MutShells<T, impl Container<T>>) -> Self {
        let mut shell = collection.get(to).expect("Item doesn't exist");
        shell.add_from(from);
        Self::new(to)
    }

    pub fn disconnect(self, from: AnyKey, collection: MutShells<T, impl Container<T>>) {
        if let Some(mut shell) = collection.get(self.key()) {
            shell.remove_from(from)
        }
    }

    pub fn get<R, S, C: Container<T>>(
        self,
        coll: TypePermit<R, T, S, C>,
    ) -> Slot<T, C::GroupItem, C::Shell, C::Alloc, R, S> {
        coll.get(self.key())
            .unwrap_or_else(|| self.panic_dangling())
    }

    fn panic_dangling(&self) -> ! {
        panic!("Dangling reference {:?}", self);
    }
}

impl<T: AnyItem> Copy for Ref<T> {}

impl<T: AnyItem> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Ref(self.0)
    }
}

impl<T: AnyItem> From<Ref<T>> for Key<T> {
    fn from(ref_: Ref<T>) -> Self {
        ref_.0
    }
}

impl<T: AnyItem> fmt::Debug for Ref<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ref({:?})", self.0)
    }
}

impl<T: AnyItem> PartialEq<AnyKey> for Ref<T> {
    fn eq(&self, other: &AnyKey) -> bool {
        AnyKey::from(self.0) == *other
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AnyRef(AnyKey);

impl AnyRef {
    pub fn new(key: AnyKey) -> Self {
        Self(key)
    }

    pub fn key(&self) -> AnyKey {
        self.0
    }

    pub fn downcast<T: AnyItem>(self) -> Option<Ref<T>> {
        self.0.downcast().map(Ref)
    }

    /// Panics if to doesn't exist.
    pub fn connect(from: AnyKey, to: AnyKey, collection: MutAnyShells<impl AnyContainer>) -> Self {
        collection
            .get(to)
            .expect("Item doesn't exist")
            .add_from(from);
        Self::new(to)
    }

    pub fn disconnect(self, from: AnyKey, collection: MutAnyShells<impl AnyContainer>) {
        if let Some(mut slot) = collection.get(self.key()) {
            slot.shell_mut().remove_from(from);
        }
    }

    pub fn get<R, S, C: AnyContainer>(self, coll: AnyPermit<R, S, C>) -> AnySlot<R, S> {
        coll.get(self.key())
            .unwrap_or_else(|| self.panic_dangling())
    }

    fn panic_dangling(&self) -> ! {
        panic!("Dangling reference {:?}", self);
    }
}

impl<T: AnyItem> From<Ref<T>> for AnyRef {
    fn from(ref_: Ref<T>) -> Self {
        AnyRef(ref_.0.into())
    }
}
