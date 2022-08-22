use std::marker::PhantomData;

use super::{AnyKey, Global, Key, Local, Locality, Localized, LocalizedData};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnyRef(pub Locality, pub AnyKey);

impl AnyRef {
    pub fn locality(&self) -> Locality {
        self.0
    }

    /// Must be used in accordance of Locality.
    pub fn key(&self) -> LocalizedData<AnyKey> {
        match self.0 {
            Locality::Global => LocalizedData::Global(self.1),
            Locality::Local => LocalizedData::Local(self.1),
        }
    }
}

impl<L: Localized, T: ?Sized + 'static> From<Ref<T, L>> for AnyRef {
    fn from(ref_: Ref<T, L>) -> Self {
        AnyRef(L::L, ref_.0.into())
    }
}

impl<T: ?Sized + 'static> From<TypedRef<T>> for AnyRef {
    fn from(ref_: TypedRef<T>) -> Self {
        AnyRef(ref_.0, ref_.1.into())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TypedRef<T: ?Sized>(pub Locality, pub Key<T>);

impl<T: ?Sized> TypedRef<T> {
    pub fn locality(&self) -> Locality {
        self.0
    }

    /// Must be used in accordance of Locality.
    pub fn key(&self) -> LocalizedData<Key<T>> {
        match self.0 {
            Locality::Global => LocalizedData::Global(self.1),
            Locality::Local => LocalizedData::Local(self.1),
        }
    }
}

impl<T: ?Sized> Copy for TypedRef<T> {}

impl<T: ?Sized> Clone for TypedRef<T> {
    fn clone(&self) -> Self {
        TypedRef(self.0, self.1)
    }
}

impl<L: Localized, T: ?Sized> From<Ref<T, L>> for TypedRef<T> {
    fn from(ref_: Ref<T, L>) -> Self {
        TypedRef(L::L, ref_.0)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Ref<T: ?Sized, L: Localized = Global>(pub(crate) Key<T>, PhantomData<L>);

impl<L: Localized, T: ?Sized> Ref<T, L> {
    pub(crate) fn new(key: Key<T>) -> Self {
        Ref(key, PhantomData)
    }

    pub fn locality(&self) -> Locality {
        L::L
    }
}

impl<T: ?Sized> Ref<T, Global> {
    pub fn key(&self) -> Key<T> {
        self.0
    }
}

impl<T: ?Sized> Ref<T, Local> {
    pub fn key(&self) -> LocalizedData<Key<T>> {
        LocalizedData::Local(self.0)
    }
}

impl<L: Localized, T: ?Sized> Copy for Ref<T, L> {}

impl<L: Localized, T: ?Sized> Clone for Ref<T, L> {
    fn clone(&self) -> Self {
        Ref(self.0, PhantomData)
    }
}
