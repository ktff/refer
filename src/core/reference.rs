use super::{AnyKey, Key};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Directionality {
    Uni,
    Bi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnyRef(pub Directionality, pub AnyKey);

impl<T: ?Sized + 'static> From<UniRef<T>> for AnyRef {
    fn from(ref_: UniRef<T>) -> Self {
        AnyRef(Directionality::Uni, ref_.0.into())
    }
}

impl<T: ?Sized + 'static> From<BiRef<T>> for AnyRef {
    fn from(ref_: BiRef<T>) -> Self {
        AnyRef(Directionality::Bi, ref_.0.into())
    }
}

impl<T: ?Sized + 'static> From<Ref<T>> for AnyRef {
    fn from(ref_: Ref<T>) -> Self {
        AnyRef(ref_.0, ref_.1.into())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Ref<T: ?Sized>(pub Directionality, pub Key<T>);

impl<T: ?Sized> Ref<T> {
    pub fn from(self, key: AnyKey) -> AnyRef {
        AnyRef(self.0, key)
    }
}

impl<T: ?Sized> Copy for Ref<T> {}

impl<T: ?Sized> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Ref(self.0, self.1)
    }
}

impl<T: ?Sized + 'static> From<UniRef<T>> for Ref<T> {
    fn from(ref_: UniRef<T>) -> Self {
        Ref(Directionality::Uni, ref_.0.into())
    }
}

impl<T: ?Sized + 'static> From<BiRef<T>> for Ref<T> {
    fn from(ref_: BiRef<T>) -> Self {
        Ref(Directionality::Bi, ref_.0.into())
    }
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UniRef<T: ?Sized>(pub Key<T>);

impl<T: ?Sized> Copy for UniRef<T> {}

impl<T: ?Sized> Clone for UniRef<T> {
    fn clone(&self) -> Self {
        UniRef(self.0)
    }
}

impl<T: ?Sized> Into<Key<T>> for UniRef<T> {
    fn into(self) -> Key<T> {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BiRef<T: ?Sized>(pub Key<T>);

impl<T: ?Sized> Copy for BiRef<T> {}

impl<T: ?Sized> Clone for BiRef<T> {
    fn clone(&self) -> Self {
        BiRef(self.0)
    }
}

impl<T: ?Sized> Into<Key<T>> for BiRef<T> {
    fn into(self) -> Key<T> {
        self.0
    }
}
