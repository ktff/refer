use super::GroupId;
use crate::core::*;
use iter::{start::Start, KeyPermit, KeySet, TypePermit};
use radix_heap::Radix;
use std::marker::PhantomData;

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

    fn access_paused(paused: &mut Self::Paused, group: Self::Group) -> Self::Keys;

    fn return_paused(paused: &mut Self::Paused, group: Self::Group, keys: Self::Keys);

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
    type Paused = Option<KEYS>;
    type B<'a, TP: 'a + TypePermit + Permits<T>> = Access<'a, C, R, TP, KEYS>;

    fn new<'a, 's: 'a, TP: 'a + TypePermit + Permits<T>>(
        access: Access<'a, Self::C, Self::R, TP, All>,
        _: &impl Start<'s, T = T, K = Self::Group>,
    ) -> Self::B<'a, TP> {
        access.keys_split_with(KEYS::default())
    }

    fn paused<'a, 's: 'a>(_: &impl Start<'s, T = T, K = Self::Group>) -> Self::Paused {
        None
    }

    fn access_paused(paused: &mut Self::Paused, _: Self::Group) -> Self::Keys {
        paused.take().unwrap_or_default()
    }

    fn return_paused(paused: &mut Self::Paused, _: Self::Group, keys: Self::Keys) {
        assert!(paused.is_none());
        *paused = Some(keys);
    }

    fn add_group_paused(_: &mut Self::Paused) -> Self::Group {
        ()
    }

    fn remove_group_paused(_: &mut Self::Paused, _: Self::Group) {}

    fn resume<'a, TP: 'a + TypePermit + Permits<T>>(
        access: Access<'a, Self::C, Self::R, TP, All>,
        state: <Self::B<'a, TP> as Isolate<'a, T>>::Paused,
    ) -> Self::B<'a, TP> {
        access.keys_split_with(state.unwrap_or_default())
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
    type Group = GroupId;
    type C = C;
    type R = permit::Ref;
    type Keys = Keys;
    type Paused = (Vec<Option<Keys>>, Vec<GroupId>);
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
        if let Some(n) = start.iter().map(|(g, _)| *g as usize).max() {
            while n >= this.isolates.len() {
                this.add_group();
            }
        }

        this
    }

    fn paused<'a, 's: 'a>(start: &impl Start<'s, T = T, K = Self::Group>) -> Self::Paused {
        let mut isolates = Vec::new();
        if let Some(n) = start.iter().map(|(g, _)| *g).max() {
            isolates.resize_with(n as usize + 1, || None);
        }
        (isolates, Vec::new())
    }

    fn access_paused(paused: &mut Self::Paused, group: Self::Group) -> Self::Keys {
        paused.0[group as usize].take().unwrap_or_default()
    }

    fn return_paused(paused: &mut Self::Paused, group: Self::Group, keys: Self::Keys) {
        paused.0[group as usize] = Some(keys);
    }

    fn add_group_paused(paused: &mut Self::Paused) -> Self::Group {
        if let Some(free) = paused.1.pop() {
            return free;
        }
        let group = paused.0.len();
        paused.0.push(None);
        group as u32
    }

    fn remove_group_paused(paused: &mut Self::Paused, group: Self::Group) {
        paused.1.push(group);
        paused.0[group as usize] = None;
    }

    fn resume<'a, TP: 'a + TypePermit + Permits<T>>(
        access: Access<'a, Self::C, Self::R, TP, All>,
        (state, free_slots): <Self::B<'a, TP> as Isolate<'a, T>>::Paused,
    ) -> Self::B<'a, TP> {
        RefIsolate {
            isolates: state
                .into_iter()
                .map(|keys| access.clone().keys_split_with(keys.unwrap_or_default()))
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
    type Paused = Option<KEYS>;

    fn pause(self) -> Self::Paused {
        Some(self.into_keys())
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
    free_slots: Vec<GroupId>,
}

impl<
        'a,
        T: DynItem + ?Sized,
        C: AnyContainer + ?Sized,
        TP: TypePermit + Permits<T>,
        KEYS: KeyPermit + KeySet + Default,
    > Isolate<'a, T> for RefIsolate<'a, C, TP, KEYS>
{
    type Group = GroupId;
    type C = C;
    type R = permit::Ref;
    type TP = TP;
    type Keys = KEYS;
    type Paused = (Vec<Option<KEYS>>, Vec<GroupId>);

    fn pause(self) -> Self::Paused {
        (
            self.isolates
                .into_iter()
                .map(|access| Some(access.into_keys()))
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
        group as u32
    }

    fn remove_group(&mut self, group: Self::Group) {
        self.free_slots.push(group);
        self.isolates[group as usize] = self.access.clone().keys_split_with(KEYS::default());
    }

    fn access_group(
        &mut self,
        group: Self::Group,
    ) -> &mut Access<'a, Self::C, Self::R, Self::TP, Self::Keys> {
        &mut self.isolates[group as usize]
    }

    fn take_key<K: Copy>(
        &mut self,
        group: GroupId,
        key: Key<K, T>,
    ) -> Option<Access<'a, Self::C, Self::R, Self::TP, Key<K, T>>> {
        Access::take_key(&mut self.isolates[group as usize], key)
    }

    fn borrow_key<'b, K: Copy>(
        &'b self,
        group: GroupId,
        key: Key<K, T>,
    ) -> Option<Access<'b, Self::C, Self::R, Self::TP, Key<K, T>>> {
        Access::borrow_key(&self.isolates[group as usize], key)
    }
}
