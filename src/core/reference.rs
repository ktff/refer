use std::marker::PhantomData;

use super::{AnyKey, Collection, Key, PathRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnyRef(pub Locality, pub Directionality, pub AnyKey);

impl AnyRef {
    pub fn locality(&self) -> Locality {
        self.0
    }

    pub fn directionality(&self) -> Directionality {
        self.1
    }

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
    pub Key<T>,
    PhantomData<(L, D)>,
);

impl<L: Localized, D: Directioned, T: ?Sized> Ref<T, L, D> {
    pub fn locality(&self) -> Locality {
        L::L
    }

    pub fn directionality(&self) -> Directionality {
        D::D
    }

    pub fn key(&self) -> Key<T> {
        self.0
    }
}

impl<D: Directioned, T: ?Sized + 'static> Ref<T, Global, D> {
    pub fn new<'a, P: PathRef<'a>>(_: &P, global_key: Key<T>) -> Self
    where
        P::Top: Collection<T>,
    {
        Ref(global_key, PhantomData)
    }

    pub fn reverse<'a, F: ?Sized + 'static, P: PathRef<'a>>(
        self,
        path: &P,
        global_from: Key<F>,
    ) -> Ref<F, Global, D>
    where
        P::Top: Collection<F>,
    {
        Ref::<F, Global, D>::new(path, global_from)
    }
}

impl<D: Directioned, T: ?Sized + 'static> Ref<T, Local, D> {
    /// None if key is not in local/bottom collection.
    pub fn new<'a, P: PathRef<'a>>(path: &P, global_key: Key<T>) -> Option<Self>
    where
        P::Bottom: Collection<T>,
    {
        Some(Ref(path.bottom_key(global_key)?, PhantomData))
    }

    /// None if key is not in local/bottom collection.
    pub fn reverse<'a, F: ?Sized + 'static, P: PathRef<'a>>(
        self,
        path: &P,
        global_from: Key<F>,
    ) -> Option<Ref<F, Local, D>>
    where
        P::Bottom: Collection<F>,
    {
        Ref::<F, Local, D>::new(path, global_from)
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
