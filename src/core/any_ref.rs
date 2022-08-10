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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UniRef<T: ?Sized>(pub Key<T>);

impl<T: ?Sized> Into<Key<T>> for UniRef<T> {
    fn into(self) -> Key<T> {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BiRef<T: ?Sized>(pub Key<T>);

impl<T: ?Sized> Into<Key<T>> for BiRef<T> {
    fn into(self) -> Key<T> {
        self.0
    }
}
