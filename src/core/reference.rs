use crate::core::*;
use log::*;
use std::{
    fmt,
    hash::{Hash, Hasher},
};

// TODO: Try to unify Ref and DynRef through traits and impl on permits

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
    pub fn get<R, S, C: Container<T> + ?Sized>(
        self,
        coll: TypePermit<T, R, S, C>,
    ) -> Slot<T, C::Shell, R, S> {
        coll.slot(self.key())
            .get()
            .map_err(|error| {
                error!("Failed to fetch {:?}, error: {}", self.0, error);
                error
            })
            .expect("Failed to fetch")
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

impl<T: Item, U: AnyItem + ?Sized> PartialEq<Key<U>> for Ref<T> {
    fn eq(&self, other: &Key<U>) -> bool {
        &self.0 == other
    }
}

pub type AnyRef = DynRef<dyn AnyItem>;

pub struct DynRef<T: DynItem + ?Sized>(Key<T>);

impl<T: DynItem + ?Sized> DynRef<T> {
    pub fn new(key: Key<T>) -> Self {
        Self(key)
    }

    pub fn key(&self) -> Key<T> {
        self.0
    }

    pub fn downcast<U: Item>(self) -> Option<Ref<U>> {
        self.0.downcast().map(Ref)
    }

    pub fn get<R, S, C: AnyContainer + ?Sized>(self, coll: AnyPermit<R, S, C>) -> DynSlot<T, R, S> {
        coll.slot(self.0)
            .get_dyn()
            .map_err(|error| {
                error!("Failed to fetch {:?}, error: {}", self.0, error);
                error
            })
            .expect("Failed to fetch")
    }
}

impl<T: DynItem + ?Sized> Eq for DynRef<T> {}

impl<T: DynItem + ?Sized> PartialEq for DynRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: DynItem + ?Sized> Ord for DynRef<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: DynItem + ?Sized> PartialOrd for DynRef<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: DynItem + ?Sized> Hash for DynRef<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T: DynItem + ?Sized> Copy for DynRef<T> {}

impl<T: DynItem + ?Sized> Clone for DynRef<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T: DynItem + ?Sized> From<DynRef<T>> for Key<T> {
    fn from(ref_: DynRef<T>) -> Self {
        ref_.0
    }
}

impl<T: DynItem + ?Sized> fmt::Debug for DynRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynRef({:?})", self.0)
    }
}
