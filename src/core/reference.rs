use crate::core::*;
use log::*;
use std::{
    fmt,
    hash::{Hash, Hasher},
};

pub type AnyRef = Ref<dyn AnyItem>;

#[repr(transparent)]
pub struct Ref<T: DynItem + ?Sized>(Key<T>);

impl<T: DynItem + ?Sized> Ref<T> {
    pub fn new(key: Key<T>) -> Self {
        Self(key)
    }

    pub fn key(&self) -> Key<T> {
        self.0
    }

    pub fn any(self) -> AnyRef {
        Ref::new(self.0.any())
    }

    pub fn get<R, S, C: Container<T> + ?Sized>(
        self,
        coll: TypePermit<T, R, S, C>,
    ) -> Slot<T, C::Shell, R, S>
    where
        T: Item,
    {
        coll.slot(self.key())
            .get()
            .map_err(|error| {
                error!("Failed to fetch {:?}, error: {}", self.0, error);
                error
            })
            .expect("Failed to fetch")
    }

    pub fn get_dyn<R, S, C: AnyContainer + ?Sized>(
        self,
        coll: AnyPermit<R, S, C>,
    ) -> DynSlot<T, R, S> {
        coll.slot(self.0)
            .get_dyn()
            .map_err(|error| {
                error!("Failed to fetch {:?}, error: {}", self.0, error);
                error
            })
            .expect("Failed to fetch")
    }
}

impl<T: DynItem + ?Sized> Eq for Ref<T> {}

impl<T: DynItem + ?Sized, U: DynItem + ?Sized> PartialEq<Ref<U>> for Ref<T> {
    fn eq(&self, other: &Ref<U>) -> bool {
        self.0 == other.0
    }
}

impl<T: DynItem + ?Sized, U: DynItem + ?Sized> PartialEq<Ref<U>> for Key<T> {
    fn eq(&self, other: &Ref<U>) -> bool {
        *self == other.key()
    }
}

impl<T: DynItem + ?Sized, U: DynItem + ?Sized> PartialEq<Key<U>> for Ref<T> {
    fn eq(&self, other: &Key<U>) -> bool {
        self.key() == *other
    }
}

impl<T: DynItem + ?Sized, U: DynItem + ?Sized> PartialOrd<Ref<U>> for Ref<T> {
    fn partial_cmp(&self, other: &Ref<U>) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: DynItem + ?Sized> Ord for Ref<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: DynItem + ?Sized> Hash for Ref<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T: DynItem + ?Sized> Copy for Ref<T> {}

impl<T: DynItem + ?Sized> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T: DynItem + ?Sized> From<Ref<T>> for Key<T> {
    fn from(ref_: Ref<T>) -> Self {
        ref_.0
    }
}

impl<T: DynItem + ?Sized> fmt::Debug for Ref<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynRef({:?})", self.0)
    }
}
