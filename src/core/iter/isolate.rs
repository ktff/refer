use std::marker::PhantomData;

use crate::core::*;
use iter::{start::Start, KeyPermit, KeySet, TypePermit};
use radix_heap::Radix;

pub trait IsolateTemplate<T: DynItem + ?Sized> {
    type Group: Radix + Ord + Copy;
    type C: AnyContainer + ?Sized;
    type R: Permit;
    type TP: TypePermit + Permits<T>;
    type Keys: KeyPermit + KeySet + Default;
    type Paused;
    type B<'a>: Isolate<
        'a,
        T,
        Group = Self::Group,
        C = Self::C,
        R = Self::R,
        TP = Self::TP,
        Keys = Self::Keys,
        Paused = Self::Paused,
    >;

    fn new<'a, 's: 'a>(
        access: Access<'a, Self::C, Self::R, Self::TP, All>,
        start: &impl Start<'s, T = T, K = Self::Group>,
    ) -> Self::B<'a>;

    fn resume<'a>(
        access: Access<'a, Self::C, Self::R, Self::TP, All>,
        state: Self::Paused,
    ) -> Self::B<'a>;
}

pub struct Shared<
    T: DynItem + ?Sized,
    C: AnyContainer + ?Sized,
    R: Permit,
    TP: TypePermit + Permits<T>,
    Keys: KeyPermit + KeySet + Default,
>(pub PhantomData<(&'static T, &'static C, R, TP, Keys)>);

impl<
        T: DynItem + ?Sized,
        C: AnyContainer + ?Sized,
        R: Permit,
        TP: TypePermit + Permits<T>,
        KEYS: KeyPermit + KeySet + Default,
    > IsolateTemplate<T> for Shared<T, C, R, TP, KEYS>
{
    type Group = ();
    type C = C;
    type R = R;
    type TP = TP;
    type Keys = KEYS;
    type Paused = KEYS;
    type B<'a> = Access<'a, C, R, TP, KEYS>;

    fn new<'a, 's: 'a>(
        access: Access<'a, Self::C, Self::R, Self::TP, All>,
        _: &impl Start<'s, T = T, K = Self::Group>,
    ) -> Self::B<'a> {
        access.keys_split_with(KEYS::default())
    }

    fn resume<'a>(
        access: Access<'a, Self::C, Self::R, Self::TP, All>,
        state: <Self::B<'a> as Isolate<'a, T>>::Paused,
    ) -> Self::B<'a> {
        access.keys_split_with(state)
    }
}

pub struct Isolated<
    T: DynItem + ?Sized,
    C: AnyContainer + ?Sized,
    TP: TypePermit + Permits<T>,
    Keys: KeyPermit + KeySet + Default,
>(pub PhantomData<(&'static T, &'static C, TP, Keys)>);

impl<
        T: DynItem + ?Sized,
        C: AnyContainer + ?Sized,
        TP: TypePermit + Permits<T>,
        Keys: KeyPermit + KeySet + Default,
    > IsolateTemplate<T> for Isolated<T, C, TP, Keys>
{
    type Group = usize;
    type C = C;
    type R = permit::Ref;
    type TP = TP;
    type Keys = Keys;
    type Paused = Vec<Keys>;
    type B<'a> = RefIsolate<'a, C, TP, Keys>;

    fn new<'a, 's: 'a>(
        access: Access<'a, Self::C, Self::R, Self::TP, All>,
        start: &impl Start<'s, T = T, K = Self::Group>,
    ) -> Self::B<'a> {
        let mut this = RefIsolate {
            access,
            isolates: Vec::new(),
        };
        if let Some(n) = start.iter().map(|(g, _)| *g).max() {
            while n >= this.isolates.len() {
                this.add_group();
            }
        }

        this
    }

    fn resume<'a>(
        access: Access<'a, Self::C, Self::R, Self::TP, All>,
        state: <Self::B<'a> as Isolate<'a, T>>::Paused,
    ) -> Self::B<'a> {
        RefIsolate {
            isolates: state
                .into_iter()
                .map(|keys| access.clone().keys_split_with(keys))
                .collect(),
            access,
        }
    }
}

pub trait Isolate<'a, T: DynItem + ?Sized> {
    type Group: Radix + Ord + Copy;
    type C: AnyContainer + ?Sized;
    type R: Permit;
    type TP: TypePermit + Permits<T>;
    type Keys: KeyPermit + KeySet + Default;
    type Paused;

    fn pause(self) -> Self::Paused;

    fn add_group(&mut self) -> Self::Group;

    fn take_key<K: Copy>(
        &mut self,
        group: Self::Group,
        key: Key<K, T>,
    ) -> Option<Access<'a, Self::C, Self::R, Self::TP, Key<K, T>>>;

    fn borrow_key<'b, K: Copy>(
        &'b self,
        group: Self::Group,
        key: Key<K, T>,
    ) -> Option<Access<'b, Self::C, Self::R, Self::TP, Key<K, T>>>;
}

impl<
        'a,
        T: DynItem + ?Sized,
        C: AnyContainer + ?Sized,
        R: Permit,
        TP: TypePermit + Permits<T>,
        KEYS: KeyPermit + KeySet + Default,
    > Isolate<'a, T> for Access<'a, C, R, TP, KEYS>
{
    type Group = ();
    type C = C;
    type R = R;
    type TP = TP;
    type Keys = KEYS;
    type Paused = KEYS;

    fn pause(self) -> Self::Paused {
        self.into_keys()
    }

    fn add_group(&mut self) -> Self::Group {
        ()
    }

    fn take_key<K: Copy>(
        &mut self,
        _group: Self::Group,
        key: Key<K, T>,
    ) -> Option<Access<'a, Self::C, Self::R, Self::TP, Key<K, T>>> {
        Access::take_key(self, key)
    }

    fn borrow_key<'b, K: Copy>(
        &'b self,
        _group: Self::Group,
        key: Key<K, T>,
    ) -> Option<Access<'b, Self::C, Self::R, Self::TP, Key<K, T>>> {
        Access::borrow_key(self, key)
    }
}

pub struct RefIsolate<
    'a,
    C: AnyContainer + ?Sized,
    TP: TypePermit,
    KEYS: KeyPermit + KeySet + Default,
> {
    access: Access<'a, C, permit::Ref, TP, All>,
    isolates: Vec<Access<'a, C, permit::Ref, TP, KEYS>>,
}

impl<
        'a,
        T: DynItem + ?Sized,
        C: AnyContainer + ?Sized,
        TP: TypePermit + Permits<T>,
        KEYS: KeyPermit + KeySet + Default,
    > Isolate<'a, T> for RefIsolate<'a, C, TP, KEYS>
{
    type Group = usize;
    type C = C;
    type R = permit::Ref;
    type TP = TP;
    type Keys = KEYS;
    type Paused = Vec<KEYS>;

    fn pause(self) -> Self::Paused {
        self.isolates
            .into_iter()
            .map(|access| access.into_keys())
            .collect()
    }

    fn add_group(&mut self) -> Self::Group {
        let group = self.isolates.len();
        self.isolates
            .push(self.access.clone().keys_split_with(KEYS::default()));
        group
    }

    fn take_key<K: Copy>(
        &mut self,
        group: usize,
        key: Key<K, T>,
    ) -> Option<Access<'a, Self::C, Self::R, Self::TP, Key<K, T>>> {
        Access::take_key(&mut self.isolates[group], key)
    }

    fn borrow_key<'b, K: Copy>(
        &'b self,
        group: usize,
        key: Key<K, T>,
    ) -> Option<Access<'b, Self::C, Self::R, Self::TP, Key<K, T>>> {
        Access::borrow_key(&self.isolates[group], key)
    }
}
