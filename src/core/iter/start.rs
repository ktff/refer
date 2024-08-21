use super::{DynItem, Key, Ref};
use std::collections::VecDeque;

pub struct Subset<'a, T: DynItem + ?Sized, K>(pub VecDeque<(K, Key<Ref<'a>, T>)>);

pub struct Root<'a, T: DynItem + ?Sized>(pub Option<Key<Ref<'a>, T>>);

pub trait Start<'a>: 'a {
    type T: DynItem + ?Sized;
    type K: Ord + Copy;

    fn pop(&mut self) -> Option<(Self::K, Key<Ref<'a>, Self::T>)>;

    fn iter<'b>(&'b self) -> impl Iterator<Item = (&'b Self::K, &'b Key<Ref<'a>, Self::T>)> + 'b
    where
        'a: 'b;
}

impl<'a, T: DynItem + ?Sized, K: Ord + Copy + 'static> Start<'a> for Subset<'a, T, K> {
    type T = T;
    type K = K;

    fn pop(&mut self) -> Option<(Self::K, Key<Ref<'a>, T>)> {
        self.0.pop_front()
    }

    fn iter<'b>(&'b self) -> impl Iterator<Item = (&'b Self::K, &'b Key<Ref<'a>, Self::T>)> + 'b
    where
        'a: 'b,
    {
        self.0.iter().map(|(k, v)| (k, v))
    }
}

impl<'a, T: DynItem + ?Sized> Start<'a> for Root<'a, T> {
    type T = T;
    type K = ();

    fn pop(&mut self) -> Option<(Self::K, Key<Ref<'a>, T>)> {
        self.0.take().map(|v| ((), v))
    }

    fn iter<'b>(&'b self) -> impl Iterator<Item = (&'b Self::K, &'b Key<Ref<'a>, Self::T>)> + 'b
    where
        'a: 'b,
    {
        self.0.iter().map(|v| (&(), v))
    }
}
