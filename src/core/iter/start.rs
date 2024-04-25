use std::collections::VecDeque;
use super::{DynItem, Key, Ref};

pub struct Subset<'a, T: DynItem + ?Sized>(pub VecDeque<Key<Ref<'a>, T>>);

pub struct Root<'a, T: DynItem + ?Sized>(pub Option<Key<Ref<'a>, T>>);

pub trait Start<'a>: 'a {
    type T: DynItem + ?Sized;

    fn pop(&mut self) -> Option<Key<Ref<'a>, Self::T>>;
}

impl<'a, T: DynItem + ?Sized> Start<'a> for Subset<'a, T> {
    type T = T;

    fn pop(&mut self) -> Option<Key<Ref<'a>, T>> {
        self.0.pop_front()
    }
}

impl<'a, T: DynItem + ?Sized> Start<'a> for Root<'a, T> {
    type T = T;

    fn pop(&mut self) -> Option<Key<Ref<'a>, T>> {
        self.0.take()
    }
}
