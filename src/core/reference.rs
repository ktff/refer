use super::{AnyKey, Key};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnyRef(pub AnyKey);

impl AnyRef {
    pub fn key(&self) -> AnyKey {
        self.0
    }
}

impl<T: ?Sized + 'static> From<Ref<T>> for AnyRef {
    fn from(ref_: Ref<T>) -> Self {
        AnyRef(ref_.0.into())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Ref<T: ?Sized>(pub(crate) Key<T>);

impl<T: ?Sized> Ref<T> {
    pub(crate) fn new(key: Key<T>) -> Self {
        Ref(key)
    }

    pub fn key(&self) -> Key<T> {
        self.0
    }
}

impl<T: ?Sized> Copy for Ref<T> {}

impl<T: ?Sized> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Ref(self.0)
    }
}
