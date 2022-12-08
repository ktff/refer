use crate::core::*;
use log::*;
use std::{
    fmt,
    hash::{Hash, Hasher},
};

pub struct Ref<T: Item>(Key<T>);

impl<T: Item> Ref<T> {
    pub fn new(key: Key<T>) -> Self {
        Ref(key)
    }

    pub fn key(&self) -> Key<T> {
        self.0
    }
}

impl<T: Item> Ref<T> {
    /// Panics if to doesn't exist.
    pub fn connect(from: AnyKey, to: Key<T>, collection: MutShells<T, impl Container<T>>) -> Self {
        let mut shell = collection
            .get(to)
            .map_err(|error| {
                error!("Failed to connect {} - {}, error: {}", from, to, error);
                error
            })
            .expect("Failed to connect");

        shell.add_to_shell(from);
        Self::new(to)
    }

    pub fn disconnect_from(self, from: AnyKey, collection: MutShells<T, impl Container<T>>) {
        collection
            .get(self.key())
            .map_err(|error| {
                error!(
                    "Failed to disconnect {} - {}, error: {}",
                    from, self.0, error
                );
                error
            })
            .expect("Failed to disconnect")
            .remove(from)
    }

    pub fn get<R, S, C: Container<T>>(
        self,
        coll: TypePermit<T, R, S, C>,
    ) -> Slot<T, C::Shell, R, S> {
        coll.get(self.key())
            .map_err(|error| {
                error!("Failed to fetch {}, error: {}", self.0, error);
                error
            })
            .expect("Failed to fetch")
    }
}

impl<T: Item> Key<T> {
    pub fn connect_from(
        self,
        from: impl Into<AnyKey>,
        collection: MutShells<T, impl Container<T>>,
    ) -> Ref<T> {
        Ref::connect(from.into(), self, collection)
    }

    pub fn connect_to(self, to: Key<T>, collection: MutShells<T, impl Container<T>>) -> Ref<T> {
        Ref::connect(self.into(), to, collection)
    }
}

impl<T: Item> Eq for Ref<T> {}

impl<T: Item> PartialEq for Ref<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Item> Ord for Ref<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: Item> PartialOrd for Ref<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Item> Hash for Ref<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T: Item> Copy for Ref<T> {}

impl<T: Item> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Ref(self.0)
    }
}

impl<T: Item> From<Ref<T>> for Key<T> {
    fn from(ref_: Ref<T>) -> Self {
        ref_.0
    }
}

impl<T: Item> fmt::Debug for Ref<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ref({:?})", self.0)
    }
}

impl<T: Item> PartialEq<AnyKey> for Ref<T> {
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

    pub fn downcast<T: Item>(self) -> Option<Ref<T>> {
        self.0.downcast().map(Ref)
    }

    /// Panics if to doesn't exist.
    pub fn connect(from: AnyKey, to: AnyKey, collection: MutAnyShells<impl AnyContainer>) -> Self {
        collection
            .get(to)
            .map_err(|error| {
                error!("Failed to connect {} - {}, error: {}", from, to, error);
                error
            })
            .expect("Failed to connect")
            .add_from(from);

        Self::new(to)
    }

    pub fn disconnect_from(self, from: AnyKey, collection: MutAnyShells<impl AnyContainer>) {
        collection
            .get(self.key())
            .map_err(|error| {
                error!(
                    "Failed to disconnect {} - {}, error: {}",
                    from, self.0, error
                );
                error
            })
            .expect("Failed to disconnect")
            .shell_mut()
            .remove_any(from);
    }

    pub fn get<R, S, C: AnyContainer>(self, coll: AnyPermit<R, S, C>) -> AnySlot<R, S> {
        coll.get(self.key())
            .map_err(|error| {
                error!("Failed to fetch {}, error: {}", self.0, error);
                error
            })
            .expect("Failed to fetch")
    }
}

impl AnyKey {
    pub fn connect_from(
        self,
        from: impl Into<AnyKey>,
        collection: MutAnyShells<impl AnyContainer>,
    ) -> AnyRef {
        AnyRef::connect(from.into(), self, collection)
    }

    pub fn connect_to<T: Item>(
        self,
        to: Key<T>,
        collection: MutShells<T, impl Container<T>>,
    ) -> Ref<T> {
        Ref::connect(self, to, collection)
    }
}

impl<T: Item> From<Ref<T>> for AnyRef {
    fn from(ref_: Ref<T>) -> Self {
        AnyRef(ref_.0.into())
    }
}
