use std::{marker::Unsize, ops::Deref};

use super::*;
use crate::core::{permit, AnyContainer, AnyItem, AnyPermit, DynItem};

/// Edgeless reference.
/// Dropping this will cause item leak, release instead.
pub struct Grc<T: DynItem + ?Sized = dyn AnyItem>(Key<Owned, T>);

impl<T: DynItem + ?Sized> Grc<T> {
    /// Key should come from StandaloneItem.
    pub(in super::super) fn new(key: Key<Owned, T>) -> Self {
        Self(key)
    }

    pub fn upcast<U: DynItem + ?Sized>(self) -> Grc<U>
    where
        T: Unsize<U>,
    {
        Grc(self.into_owned_key().upcast())
    }

    pub fn any(self) -> Grc {
        Grc(self.into_owned_key().any())
    }

    /// Proper way of dropping this.
    pub fn release<C: AnyContainer>(self, access: AnyPermit<permit::Mut, C>) {
        // access.slot();
        // TODO: Unifed DynSlot & Slot would help here
        unimplemented!()
    }

    /// Callers should make sure that the key is properly disposed of, else T will leak.
    pub fn into_owned_key(self) -> Key<Owned, T> {
        let index = self.index();
        std::mem::forget(self);
        // SAFETY: We are effectively moving Key out of self.
        unsafe { Key::new_owned(index) }
    }
}

impl Grc {
    /// Make assumption that this is Grc for T.
    pub fn assume<T: DynItem + ?Sized>(self) -> Grc<T> {
        Grc(self.into_owned_key().assume())
    }
}

// Deref to Key<Owned, T>
impl<T: DynItem + ?Sized> Deref for Grc<T> {
    type Target = Key<Owned, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: DynItem + ?Sized> Eq for Grc<T> {}

impl<T: DynItem + ?Sized> Ord for Grc<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: DynItem + ?Sized, U: DynItem + ?Sized> PartialEq<Grc<U>> for Grc<T> {
    fn eq(&self, other: &Grc<U>) -> bool {
        &self.0 == &other.0
    }
}

impl<T: DynItem + ?Sized, U: DynItem + ?Sized> PartialOrd<Grc<U>> for Grc<T> {
    fn partial_cmp(&self, other: &Grc<U>) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<P, T: DynItem + ?Sized, U: DynItem + ?Sized> PartialEq<Key<P, U>> for Grc<T> {
    fn eq(&self, other: &Key<P, U>) -> bool {
        &self.0 == other
    }
}

impl<P, T: DynItem + ?Sized, U: DynItem + ?Sized> PartialEq<Grc<U>> for Key<P, T> {
    fn eq(&self, other: &Grc<U>) -> bool {
        self == &other.0
    }
}

impl<P, T: DynItem + ?Sized, U: DynItem + ?Sized> PartialOrd<Key<P, U>> for Grc<T> {
    fn partial_cmp(&self, other: &Key<P, U>) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<P, T: DynItem + ?Sized, U: DynItem + ?Sized> PartialOrd<Grc<U>> for Key<P, T> {
    fn partial_cmp(&self, other: &Grc<U>) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

// Hash
impl<T: DynItem + ?Sized> std::hash::Hash for Grc<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

// Debug
impl<T: DynItem + ?Sized> std::fmt::Debug for Grc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Grc({:?})", self.0)
    }
}

// Display
impl<T: DynItem + ?Sized> std::fmt::Display for Grc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// Drop
impl<T: DynItem + ?Sized> Drop for Grc<T> {
    fn drop(&mut self) {
        if cfg!(debug_assertions) {
            panic!("Grc should be disposed of properly. {:?}", self);
        } else {
            use log::*;
            warn!(
                "Grc should be disposed of properly. Leaking item {:?}",
                self
            );
        }
    }
}