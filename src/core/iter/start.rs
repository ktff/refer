use super::{DynItem, Key, Ref};
use std::collections::HashSet;

pub struct Subset<'a, T: DynItem + ?Sized>(pub HashSet<Key<Ref<'a>, T>>);

pub struct Root<'a, T: DynItem + ?Sized>(pub Key<Ref<'a>, T>);

pub trait Start<'a> {
    type T: DynItem + ?Sized;
}

impl<T: DynItem + ?Sized> Start<'_> for Subset<'_, T> {
    type T = T;
}

impl<T: DynItem + ?Sized> Start<'_> for Root<'_, T> {
    type T = T;
}
