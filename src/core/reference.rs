use std::marker::PhantomData;

use super::{AnyKey, Key};

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
    pub fn key(&self) -> AnyKey {
        self.2
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
    pub fn key(&self) -> Key<T> {
        self.2
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

    /// Must be used in accordance of Locality.
    pub fn key(&self) -> Key<T> {
        self.0
    }
}

impl<L: Localized, D: Directioned, T: ?Sized> Copy for Ref<T, L, D> {}

impl<L: Localized, D: Directioned, T: ?Sized> Clone for Ref<T, L, D> {
    fn clone(&self) -> Self {
        Ref(self.0, PhantomData)
    }
}

// ********************* Locality *********************

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locality {
    /// Top
    Global,
    /// Bottom
    Local,
}

pub trait Localized {
    const L: Locality;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Global;

impl Localized for Global {
    const L: Locality = Locality::Global;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Local;

impl Localized for Local {
    const L: Locality = Locality::Local;
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
