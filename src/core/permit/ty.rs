use super::*;
use crate::core::{
    leaf::LeafContainer, region::RegionContainer, ty::TypeContainer, AnyContainer, AnyKey,
    Container, Key,
};
use log::*;
use std::{
    any::TypeId,
    marker::PhantomData,
    ops::{Deref, RangeBounds},
};

pub struct TypePermit<'a, T: ?Sized, R, A, C: ?Sized> {
    permit: AnyPermit<'a, R, A, C>,
    _marker: PhantomData<&'a T>,
}

impl<'a, R, T: core::DynItem + ?Sized, A, C: AnyContainer + ?Sized> TypePermit<'a, T, R, A, C> {
    pub fn new(permit: AnyPermit<'a, R, A, C>) -> Self {
        Self {
            permit,
            _marker: PhantomData,
        }
    }

    /// UNSAFE: Caller must ensure some kind of division between jurisdictions of the two permits.
    pub unsafe fn unsafe_split<P>(&self, map: impl FnOnce(Self) -> P) -> P {
        map(self.permit.unsafe_split(|permit| permit.ty()))
    }

    pub(super) fn access(&self) -> Permit<R, A> {
        self.permit.access()
    }

    pub fn slot(self, key: Key<T>) -> SlotPermit<'a, T, R, A, C> {
        SlotPermit::new(self, key)
    }
}

impl<'a, R, T: core::Item, A, C: ?Sized> TypePermit<'a, T, R, A, C> {
    pub fn step(self) -> Option<TypePermit<'a, T, R, A, C::Sub>>
    where
        C: TypeContainer<T>,
    {
        self.permit.step().map(TypePermit::new)
    }

    pub fn step_iter(self) -> impl Iterator<Item = SlotPermit<'a, T, R, A, C>>
    where
        C: LeafContainer<T> + AnyContainer,
    {
        // SAFETY: Iterator ensures that we don't access the same slot twice.
        self.keys()
            .map(move |key| unsafe { self.unsafe_split(|permit| permit.slot(key)) })
    }

    /// Iterates over valid keys in ascending order.
    pub fn keys(&self) -> impl Iterator<Item = Key<T>> + 'a
    where
        C: AnyContainer,
    {
        self.permit
            .keys(TypeId::of::<T>())
            .map(|key| Key::new(key.index()))
    }
}

impl<'a, R, T: core::DynItem + ?Sized, A, C: AnyContainer + ?Sized> TypePermit<'a, T, R, A, C> {
    pub fn step_into(self, index: usize) -> Option<TypePermit<'a, T, R, A, C::Sub>>
    where
        C: RegionContainer,
    {
        self.permit.step_into(index).map(TypePermit::new)
    }

    pub fn step_range(
        self,
        range: impl RangeBounds<usize>,
    ) -> Option<impl Iterator<Item = TypePermit<'a, T, R, A, C::Sub>>>
    where
        C: RegionContainer,
    {
        Some(self.permit.step_range(range)?.map(TypePermit::new))
    }
}

impl<'a, R, T: core::Item, A, C: Container<T> + ?Sized> TypePermit<'a, T, R, A, C> {
    pub fn path(self) -> PathPermit<'a, T, R, A, C> {
        PathPermit::new(self)
    }

    // None if a == b
    pub fn split_pair(
        self,
        a: Key<T>,
        b: Key<T>,
    ) -> Option<(SlotPermit<'a, T, R, A, C>, SlotPermit<'a, T, R, A, C>)> {
        if a == b {
            None
        } else {
            // SAFETY: We've checked that a != b so it's safe to split.
            Some(unsafe {
                (
                    self.permit.unsafe_split(|permit| permit.slot(a)),
                    self.slot(b),
                )
            })
        }
    }
}

impl<'a, T: core::Item, A: Into<Shell>, C: Container<T> + ?Sized> TypePermit<'a, T, Mut, A, C> {
    pub fn connect(&mut self, from: AnyKey, to: Key<T>) -> core::Ref<T> {
        self.borrow_mut()
            .slot(to)
            .get()
            .map_err(|error| {
                error!("Failed to connect {:?} -> {:?}, error: {}", from, to, error);
                error
            })
            .expect("Failed to connect")
            .shell_add(from);

        core::Ref::new(to)
    }

    pub fn disconnect(&mut self, from: AnyKey, to: core::Ref<T>) {
        self.borrow_mut()
            .slot(to.key())
            .get()
            .map_err(|error| {
                error!(
                    "Failed to disconnect {:?} -> {:?}, error: {}",
                    from,
                    to.key(),
                    error
                );
                error
            })
            .expect("Failed to disconnect")
            .shell_remove(from)
    }
}

impl<'a, T: ?Sized, A, C: ?Sized> TypePermit<'a, T, Mut, A, C> {
    pub fn borrow(&self) -> TypePermit<T, Ref, A, C> {
        TypePermit {
            permit: (&self.permit).into(),
            _marker: PhantomData,
        }
    }

    pub fn borrow_mut(&mut self) -> TypePermit<T, Mut, A, C> {
        TypePermit {
            permit: (&mut self.permit).into(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T: ?Sized, R, A, C: ?Sized> Deref for TypePermit<'a, T, R, A, C> {
    type Target = &'a C;

    fn deref(&self) -> &Self::Target {
        &self.permit
    }
}

impl<'a, T: ?Sized, A, C: ?Sized> Copy for TypePermit<'a, T, Ref, A, C> {}

impl<'a, T: ?Sized, A, C: ?Sized> Clone for TypePermit<'a, T, Ref, A, C> {
    fn clone(&self) -> Self {
        Self {
            permit: self.permit,
            _marker: PhantomData,
        }
    }
}

impl<'a, T, A: Into<B>, B, C: ?Sized> From<TypePermit<'a, T, Mut, A, C>>
    for TypePermit<'a, T, Ref, B, C>
{
    fn from(TypePermit { permit, .. }: TypePermit<'a, T, Mut, A, C>) -> Self {
        Self {
            permit: permit.into(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T, R, C: ?Sized> From<TypePermit<'a, T, R, Slot, C>> for TypePermit<'a, T, R, Item, C> {
    fn from(TypePermit { permit, .. }: TypePermit<'a, T, R, Slot, C>) -> Self {
        Self {
            permit: permit.into(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T, R, C: ?Sized> From<TypePermit<'a, T, R, Slot, C>> for TypePermit<'a, T, R, Shell, C> {
    fn from(TypePermit { permit, .. }: TypePermit<'a, T, R, Slot, C>) -> Self {
        Self {
            permit: permit.into(),
            _marker: PhantomData,
        }
    }
}

impl<'a: 'b, 'b, T, R, A, B, C: ?Sized> From<&'b TypePermit<'a, T, R, A, C>>
    for TypePermit<'b, T, Ref, B, C>
where
    Permit<R, A>: Into<Permit<Ref, B>>,
{
    fn from(permit: &'b TypePermit<'a, T, R, A, C>) -> Self {
        Self {
            permit: (&permit.permit).into(),
            _marker: PhantomData,
        }
    }
}

impl<'a: 'b, 'b, T, A, B, C: ?Sized> From<&'b mut TypePermit<'a, T, Mut, A, C>>
    for TypePermit<'b, T, Mut, B, C>
where
    Permit<Mut, A>: Into<Permit<Mut, B>>,
{
    fn from(permit: &'b mut TypePermit<'a, T, Mut, A, C>) -> Self {
        Self {
            permit: (&mut permit.permit).into(),
            _marker: PhantomData,
        }
    }
}
