use std::marker::PhantomData;

use super::{AnyKey, Global, Key, Local, Locality, Localized, LocalizedData};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnyRef(pub Locality, pub Directionality, pub AnyKey);

impl AnyRef {
    pub fn locality(&self) -> Locality {
        self.0
    }

    pub fn directionality(&self) -> Directionality {
        self.1
    }

    /// Must be used in accordance of Locality.
    pub fn key(&self) -> LocalizedData<AnyKey> {
        match self.0 {
            Locality::Global => LocalizedData::Global(self.2),
            Locality::Local => LocalizedData::Local(self.2),
        }
    }
}

impl<L: Localized, D: Directioned, T: ?Sized + 'static> From<Ref<T, L, D>> for AnyRef {
    fn from(ref_: Ref<T, L, D>) -> Self {
        AnyRef(L::L, D::D, ref_.0.into())
    }
}

impl<T: ?Sized + 'static> From<TypedRef<T>> for AnyRef {
    fn from(ref_: TypedRef<T>) -> Self {
        AnyRef(ref_.0, ref_.1, ref_.2.into())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TypedRef<T: ?Sized>(pub Locality, pub Directionality, pub Key<T>);

impl<T: ?Sized> TypedRef<T> {
    pub fn locality(&self) -> Locality {
        self.0
    }

    pub fn directionality(&self) -> Directionality {
        self.1
    }

    /// Must be used in accordance of Locality.
    pub fn key(&self) -> LocalizedData<Key<T>> {
        match self.0 {
            Locality::Global => LocalizedData::Global(self.2),
            Locality::Local => LocalizedData::Local(self.2),
        }
    }
}

impl<T: ?Sized> Copy for TypedRef<T> {}

impl<T: ?Sized> Clone for TypedRef<T> {
    fn clone(&self) -> Self {
        TypedRef(self.0, self.1, self.2)
    }
}

impl<L: Localized, D: Directioned, T: ?Sized> From<Ref<T, L, D>> for TypedRef<T> {
    fn from(ref_: Ref<T, L, D>) -> Self {
        TypedRef(L::L, D::D, ref_.0)
    }
}

/// A reference to an item T that can be anywhere-Global or nearby-Local of which
/// T is at least aware of-Unidirectional and can also know origin item if Bidirectional.
///
/// Constructed through:
/// * `Ref::add` when an existing item is creating destination item.
/// * `Ref::bind` when an existing item is referencing existing destination item.
/// * `Ref::init` when a creating item is referencing existing destination item.
///
/// Used, and finally removed via call to `Ref::remove` either during mutation or during item removal.
#[derive(Debug, PartialEq, Eq)]
pub struct Ref<T: ?Sized, L: Localized = Global, D: Directioned = Uni>(
    pub(crate) Key<T>,
    pub(crate) PhantomData<(L, D)>,
);

impl<L: Localized, D: Directioned, T: ?Sized> Ref<T, L, D> {
    pub fn locality(&self) -> Locality {
        L::L
    }

    pub fn directionality(&self) -> Directionality {
        D::D
    }
}

impl<D: Directioned, T: ?Sized> Ref<T, Global, D> {
    pub fn key(&self) -> Key<T> {
        self.0
    }
}

impl<D: Directioned, T: ?Sized> Ref<T, Local, D> {
    pub fn key(&self) -> LocalizedData<Key<T>> {
        LocalizedData::Local(self.0)
    }
}

impl<L: Localized, D: Directioned, T: ?Sized> Copy for Ref<T, L, D> {}

impl<L: Localized, D: Directioned, T: ?Sized> Clone for Ref<T, L, D> {
    fn clone(&self) -> Self {
        Ref(self.0, PhantomData)
    }
}

// ********************* Directionality *********************

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Directionality {
    Uni,
    Bi,
}

pub trait Directioned {
    const D: Directionality;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Uni;

impl Directioned for Uni {
    const D: Directionality = Directionality::Uni;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bi;

impl Directioned for Bi {
    const D: Directionality = Directionality::Bi;
}
