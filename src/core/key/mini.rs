use super::{key::KeySign, Key, Ptr};
use crate::core::{AnyItem, DynItem};
use std::{
    any, fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    num::NonZeroU32,
};

#[repr(transparent)]
pub struct U32Key<K = Ptr, T: DynItem + ?Sized = dyn AnyItem>(
    NonZeroU32,
    PhantomData<(&'static T, K)>,
);

impl<P, T: DynItem + ?Sized> Eq for U32Key<P, T> {}

impl<PT, T: DynItem + ?Sized, PU, U: DynItem + ?Sized> PartialEq<U32Key<PU, U>> for U32Key<PT, T> {
    fn eq(&self, other: &U32Key<PU, U>) -> bool {
        self.0 == other.0
    }
}

impl<P, T: DynItem + ?Sized> Ord for U32Key<P, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<PT, T: DynItem + ?Sized, PU, U: DynItem + ?Sized> PartialOrd<U32Key<PU, U>> for U32Key<PT, T> {
    fn partial_cmp(&self, other: &U32Key<PU, U>) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl<P, T: DynItem + ?Sized> Hash for U32Key<P, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// SAFETY: Key only contains Index, which is Send.
unsafe impl<P, T: DynItem + ?Sized> Send for U32Key<P, T> where NonZeroU32: Send {}
/// SAFETY: Key only contains Index, which is Sync.
unsafe impl<P, T: DynItem + ?Sized> Sync for U32Key<P, T> where NonZeroU32: Sync {}

impl<P: Copy, T: DynItem + ?Sized> Copy for U32Key<P, T> {}

impl<P: Clone, T: DynItem + ?Sized> Clone for U32Key<P, T> {
    fn clone(&self) -> Self {
        U32Key(self.0, self.1)
    }
}

impl<P: KeySign, T: DynItem + ?Sized> fmt::Debug for U32Key<P, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}:{}{}", self.0, P::sign(), any::type_name::<T>())
    }
}

impl<P: KeySign, T: DynItem + ?Sized> fmt::Display for U32Key<P, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}:{}{}", self.0, P::sign(), any::type_name::<T>())
    }
}

impl<K, T: DynItem + ?Sized> TryFrom<Key<K, T>> for U32Key<K, T> {
    type Error = Key<K, T>;

    fn try_from(value: Key<K, T>) -> Result<Self, Self::Error> {
        if let Ok(index) = value.index().try_into() {
            std::mem::forget(value);
            Ok(U32Key(index, PhantomData))
        } else {
            Err(value)
        }
    }
}

// Into Key
impl<K, T: DynItem + ?Sized> From<U32Key<K, T>> for Key<K, T> {
    fn from(value: U32Key<K, T>) -> Self {
        // SAFETY: This came from Key so this is valid.
        unsafe { Self::new(value.0.into()) }
    }
}
