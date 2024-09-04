use std::marker::PhantomData;

use crate::core::*;
use iter::{start::Start, KeyPermit, KeySet, TypePermit};
use radix_heap::Radix;

pub trait IsolateTemplate<T: DynItem + ?Sized> {
    type Group: Radix + Ord + Copy;
    type C: AnyContainer + ?Sized;
    type R: Permit;
    // type TP: TypePermit + Permits<T>;
    type Keys: KeyPermit + KeySet + Default;
    type Paused: Default;
    type B<'a, TP: 'a + TypePermit + Permits<T>>: Isolate<
        'a,
        T,
        Group = Self::Group,
        C = Self::C,
        R = Self::R,
        TP = TP,
        Keys = Self::Keys,
        Paused = Self::Paused,
    >;

    fn new<'a, 's: 'a, TP: 'a + TypePermit + Permits<T>>(
        access: Access<'a, Self::C, Self::R, TP, All>,
        start: &impl Start<'s, T = T, K = Self::Group>,
    ) -> Self::B<'a, TP>;

    fn paused<'a, 's: 'a>(start: &impl Start<'s, T = T, K = Self::Group>) -> Self::Paused;

    fn access_paused(paused: &mut Self::Paused, group: Self::Group) -> &mut Self::Keys;

    fn add_group_paused(paused: &mut Self::Paused) -> Self::Group;

    fn remove_group_paused(paused: &mut Self::Paused, group: Self::Group);

    fn resume<'a, TP: 'a + TypePermit + Permits<T>>(
        access: Access<'a, Self::C, Self::R, TP, All>,
        state: Self::Paused,
    ) -> Self::B<'a, TP>;
}

pub struct Shared<
    T: DynItem + ?Sized,
    C: AnyContainer + ?Sized,
    R: Permit,
    Keys: KeyPermit + KeySet + Default,
>(pub PhantomData<(&'static T, &'static C, R, Keys)>);

impl<
        T: DynItem + ?Sized,
        C: AnyContainer + ?Sized,
        R: Permit,
        KEYS: KeyPermit + KeySet + Default,
    > IsolateTemplate<T> for Shared<T, C, R, KEYS>
{
    type Group = ();
    type C = C;
    type R = R;
    type Keys = KEYS;
    type Paused = KEYS;
    type B<'a, TP: 'a + TypePermit + Permits<T>> = Access<'a, C, R, TP, KEYS>;

    fn new<'a, 's: 'a, TP: 'a + TypePermit + Permits<T>>(
        access: Access<'a, Self::C, Self::R, TP, All>,
        _: &impl Start<'s, T = T, K = Self::Group>,
    ) -> Self::B<'a, TP> {
        access.keys_split_with(KEYS::default())
    }

    fn paused<'a, 's: 'a>(_: &impl Start<'s, T = T, K = Self::Group>) -> Self::Paused {
        KEYS::default()
    }

    fn access_paused(paused: &mut Self::Paused, _: Self::Group) -> &mut Self::Keys {
        paused
    }

    fn add_group_paused(_: &mut Self::Paused) -> Self::Group {
        ()
    }

    fn remove_group_paused(_: &mut Self::Paused, _: Self::Group) {}

    fn resume<'a, TP: 'a + TypePermit + Permits<T>>(
        access: Access<'a, Self::C, Self::R, TP, All>,
        state: <Self::B<'a, TP> as Isolate<'a, T>>::Paused,
    ) -> Self::B<'a, TP> {
        access.keys_split_with(state)
    }
}

pub struct Isolated<
    T: DynItem + ?Sized,
    C: AnyContainer + ?Sized,
    Keys: KeyPermit + KeySet + Default,
>(pub PhantomData<(&'static T, &'static C, Keys)>);

impl<T: DynItem + ?Sized, C: AnyContainer + ?Sized, Keys: KeyPermit + KeySet + Default>
    IsolateTemplate<T> for Isolated<T, C, Keys>
{
    type Group = usize;
    type C = C;
    type R = permit::Ref;
    type Keys = Keys;
    type Paused = (Vec<Keys>, Vec<usize>);
    type B<'a, TP: 'a + TypePermit + Permits<T>> = RefIsolate<'a, C, TP, Keys>;

    fn new<'a, 's: 'a, TP: 'a + TypePermit + Permits<T>>(
        access: Access<'a, Self::C, Self::R, TP, All>,
        start: &impl Start<'s, T = T, K = Self::Group>,
    ) -> Self::B<'a, TP> {
        let mut this = RefIsolate {
            access,
            isolates: Vec::new(),
            free_slots: Vec::new(),
        };
        if let Some(n) = start.iter().map(|(g, _)| *g).max() {
            while n >= this.isolates.len() {
                this.add_group();
            }
        }

        this
    }

    fn paused<'a, 's: 'a>(start: &impl Start<'s, T = T, K = Self::Group>) -> Self::Paused {
        let mut isolates = Vec::new();
        if let Some(n) = start.iter().map(|(g, _)| *g).max() {
            isolates.resize_with(n + 1, Keys::default);
        }
        (isolates, Vec::new())
    }

    fn access_paused(paused: &mut Self::Paused, group: Self::Group) -> &mut Self::Keys {
        &mut paused.0[group]
    }

    fn add_group_paused(paused: &mut Self::Paused) -> Self::Group {
        if let Some(free) = paused.1.pop() {
            return free;
        }
        let group = paused.0.len();
        paused.0.push(Keys::default());
        group
    }

    fn remove_group_paused(paused: &mut Self::Paused, group: Self::Group) {
        paused.1.push(group);
        paused.0[group] = Keys::default();
    }

    fn resume<'a, TP: 'a + TypePermit + Permits<T>>(
        access: Access<'a, Self::C, Self::R, TP, All>,
        (state, free_slots): <Self::B<'a, TP> as Isolate<'a, T>>::Paused,
    ) -> Self::B<'a, TP> {
        RefIsolate {
            isolates: state
                .into_iter()
                .map(|keys| access.clone().keys_split_with(keys))
                .collect(),
            access,
            free_slots,
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

    fn remove_group(&mut self, group: Self::Group);

    fn access_group(
        &mut self,
        group: Self::Group,
    ) -> &mut Access<'a, Self::C, Self::R, Self::TP, Self::Keys>;

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

    fn remove_group(&mut self, _: Self::Group) {}

    fn access_group(
        &mut self,
        _: Self::Group,
    ) -> &mut Access<'a, Self::C, Self::R, Self::TP, Self::Keys> {
        self
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
    free_slots: Vec<usize>,
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
    type Paused = (Vec<KEYS>, Vec<usize>);

    fn pause(self) -> Self::Paused {
        (
            self.isolates
                .into_iter()
                .map(|access| access.into_keys())
                .collect(),
            self.free_slots,
        )
    }

    fn add_group(&mut self) -> Self::Group {
        if let Some(free) = self.free_slots.pop() {
            return free;
        }
        let group = self.isolates.len();
        self.isolates
            .push(self.access.clone().keys_split_with(KEYS::default()));
        group
    }

    fn remove_group(&mut self, group: Self::Group) {
        self.free_slots.push(group);
        self.isolates[group] = self.access.clone().keys_split_with(KEYS::default());
    }

    fn access_group(
        &mut self,
        group: Self::Group,
    ) -> &mut Access<'a, Self::C, Self::R, Self::TP, Self::Keys> {
        &mut self.isolates[group]
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
