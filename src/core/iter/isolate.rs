use crate::core::*;
use iter::{start::Start, KeyPermit, KeySet, TypePermit};
use radix_heap::Radix;

pub trait Isolate<'a, T: DynItem + ?Sized> {
    type Group: Radix + Ord + Copy;
    type C: AnyContainer + ?Sized;
    type R: Permit;
    type TP: TypePermit + Permits<T>;
    type Keys: KeyPermit + KeySet + Default;

    fn new(
        access: Access<'a, Self::C, Self::R, Self::TP, All>,
        start: &impl Start<'a, T = T, K = Self::Group>,
    ) -> Self;

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

    fn new(access: Access<'a, C, R, TP, All>, _: &impl Start<'a, T = T, K = Self::Group>) -> Self {
        access.keys_split_with(KEYS::default())
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

    fn new(
        access: Access<'a, C, permit::Ref, TP, All>,
        start: &impl Start<'a, T = T, K = Self::Group>,
    ) -> Self {
        let mut this = Self {
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
