use super::*;
use crate::core::{
    region::RegionContainer, ty::TypeContainer, AnyContainer, AnyItem, AnyKey, Container, Key,
    Result,
};
use std::{
    any::TypeId,
    ops::{Deref, RangeBounds},
};

pub struct AnyPermit<'a, R, A, C: ?Sized> {
    permit: Permit<R, A>,
    container: &'a C,
}

impl<'a, R, A, C: AnyContainer + ?Sized> AnyPermit<'a, R, A, C> {
    pub fn new(container: &'a mut C) -> Self {
        // SAFETY: We have exclusive access to the container so any Permit is valid.
        unsafe { Self::unsafe_new(Permit::new(), container) }
    }

    /// SAFETY: Caller must ensure that it has the correct R & S access to C for the given 'a.
    pub unsafe fn unsafe_new(permit: Permit<R, A>, container: &'a C) -> Self {
        Self { container, permit }
    }

    /// UNSAFE: Caller must ensure some kind of division between jurisdictions of the two permits.
    pub unsafe fn unsafe_split<T>(&self, map: impl FnOnce(Self) -> T) -> T {
        map(Self {
            container: self.container,
            permit: self.permit.access(),
        })
    }

    pub(super) fn access(&self) -> Permit<R, A> {
        self.permit.access()
    }

    pub fn container(&self) -> &'a C {
        self.container
    }

    pub fn slot<T: core::DynItem + ?Sized>(self, key: Key<T>) -> SlotPermit<'a, T, R, A, C>
    where
        C: AnyContainer,
    {
        self.ty().slot(key)
    }

    pub fn ty<T: core::DynItem + ?Sized>(self) -> TypePermit<'a, T, R, A, C>
    where
        C: AnyContainer,
    {
        TypePermit::new(self)
    }

    /// Iterates over valid slot permit of type in ascending order.
    pub fn iter(self, key: TypeId) -> impl Iterator<Item = SlotPermit<'a, dyn AnyItem, R, A, C>> {
        self.keys(key).map(move |key| {
            // SAFETY: First-next iteration ensures that we don't access the same slot twice.
            unsafe { self.unsafe_split(|permit| permit.slot(key)) }
        })
    }

    /// Iterates over valid keys of type in ascending order.
    pub fn keys(&self, key: TypeId) -> impl Iterator<Item = AnyKey> + 'a {
        let container = self.container;
        std::iter::successors(container.first_key(key), move |&key| {
            container.next_key(key)
        })
    }

    // None if a == b
    pub fn split_pair(
        self,
        a: AnyKey,
        b: AnyKey,
    ) -> Option<(
        SlotPermit<'a, dyn core::AnyItem, R, A, C>,
        SlotPermit<'a, dyn core::AnyItem, R, A, C>,
    )> {
        if a == b {
            None
        } else {
            // SAFETY: We just checked that a != b.
            Some(unsafe { (self.unsafe_split(|permit| permit.slot(a)), self.slot(b)) })
        }
    }
}

impl<'a, A: Into<Shell>, C: AnyContainer + ?Sized> AnyPermit<'a, Mut, A, C> {
    pub fn connect_dyn<T: core::DynItem + ?Sized>(
        &mut self,
        from: AnyKey,
        to: Key<T>,
    ) -> Result<core::DynRef<T>> {
        self.peek_dyn(to)?.shell_add(from);
        Ok(core::DynRef::new(to))
    }

    pub fn disconnect_dyn<T: core::DynItem + ?Sized>(
        &mut self,
        from: AnyKey,
        to: core::DynRef<T>,
    ) -> Result<()> {
        Ok(self.peek_dyn(to.key())?.shell_remove(from))
    }
}

impl<'a, C: AnyContainer + ?Sized> AnyPermit<'a, Mut, Slot, C> {
    pub fn split(self) -> (AnyPermit<'a, Mut, Item, C>, AnyPermit<'a, Mut, Shell, C>) {
        let (item, shell) = self.permit.split();
        (
            AnyPermit {
                permit: item,
                container: self.container,
            },
            AnyPermit {
                permit: shell,
                container: self.container,
            },
        )
    }
}

impl<'a, A, C: AnyContainer + ?Sized> AnyPermit<'a, Mut, A, C> {
    pub fn split_types(self) -> TypeSplitPermit<'a, A, C>
    where
        C: Sized,
    {
        TypeSplitPermit::new(self)
    }

    pub fn split_slots(self) -> SlotSplitPermit<'a, A, C> {
        SlotSplitPermit::new(self)
    }
}

impl<'a, R, A, C: ?Sized> AnyPermit<'a, R, A, C> {
    pub fn step<T: core::Item>(self) -> Option<AnyPermit<'a, R, A, C::Sub>>
    where
        C: TypeContainer<T>,
    {
        let Self { container, permit } = self;
        container
            .get()
            .map(|container| AnyPermit { container, permit })
    }

    pub fn step_into(self, index: usize) -> Option<AnyPermit<'a, R, A, C::Sub>>
    where
        C: RegionContainer,
    {
        let Self { container, permit } = self;
        container
            .get(index)
            .map(|container| AnyPermit { container, permit })
    }

    pub fn step_range(
        self,
        range: impl RangeBounds<usize>,
    ) -> Option<impl Iterator<Item = AnyPermit<'a, R, A, C::Sub>>>
    where
        C: RegionContainer,
    {
        let Self { container, permit } = self;
        Some(container.iter(range)?.map(move |(_, container)| AnyPermit {
            container,
            permit: permit.access(),
        }))
    }
}

impl<'a, A, C: ?Sized> AnyPermit<'a, Mut, A, C> {
    pub fn borrow(&self) -> AnyPermit<Ref, A, C> {
        self.into()
    }

    pub fn borrow_mut(&mut self) -> AnyPermit<Mut, A, C> {
        self.into()
    }

    pub fn peek<T: core::Item>(
        &mut self,
        key: Key<T>,
    ) -> Result<core::Slot<'_, T, C::Shell, Mut, A>>
    where
        C: Container<T>,
    {
        self.borrow_mut().slot(key).get()
    }

    pub fn peek_dyn<T: core::DynItem + ?Sized>(
        &mut self,
        key: Key<T>,
    ) -> Result<core::DynSlot<'_, T, Mut, A>>
    where
        C: AnyContainer,
    {
        self.borrow_mut().slot(key).get_dyn()
    }
}

impl<'a, A, C: ?Sized> AnyPermit<'a, Ref, A, C> {
    pub fn peek<T: core::Item>(self, key: Key<T>) -> Result<core::Slot<'a, T, C::Shell, Ref, A>>
    where
        C: Container<T>,
    {
        self.slot(key).get()
    }

    pub fn peek_dyn<T: core::DynItem + ?Sized>(
        self,
        key: Key<T>,
    ) -> Result<core::DynSlot<'a, T, Ref, A>>
    where
        C: AnyContainer,
    {
        self.slot(key).get_dyn()
    }
}

impl<'a, R, A, C: ?Sized> Deref for AnyPermit<'a, R, A, C> {
    type Target = &'a C;

    fn deref(&self) -> &Self::Target {
        &self.container
    }
}

impl<'a, A, C: ?Sized> Copy for AnyPermit<'a, Ref, A, C> {}

impl<'a, A, C: ?Sized> Clone for AnyPermit<'a, Ref, A, C> {
    fn clone(&self) -> Self {
        Self {
            permit: Permit { ..self.permit },
            ..*self
        }
    }
}

impl<'a, A: Into<B>, B, C: ?Sized> From<AnyPermit<'a, Mut, A, C>> for AnyPermit<'a, Ref, B, C> {
    fn from(AnyPermit { permit, container }: AnyPermit<'a, Mut, A, C>) -> Self {
        Self {
            permit: permit.into(),
            container,
        }
    }
}

impl<'a, R, C: ?Sized> From<AnyPermit<'a, R, Slot, C>> for AnyPermit<'a, R, Item, C> {
    fn from(AnyPermit { permit, container }: AnyPermit<'a, R, Slot, C>) -> Self {
        Self {
            permit: permit.into(),
            container,
        }
    }
}

impl<'a, R, C: ?Sized> From<AnyPermit<'a, R, Slot, C>> for AnyPermit<'a, R, Shell, C> {
    fn from(AnyPermit { permit, container }: AnyPermit<'a, R, Slot, C>) -> Self {
        Self {
            permit: permit.into(),
            container,
        }
    }
}

impl<'a: 'b, 'b, R, A, B, C: ?Sized> From<&'b AnyPermit<'a, R, A, C>> for AnyPermit<'b, Ref, B, C>
where
    Permit<R, A>: Into<Permit<Ref, B>>,
{
    fn from(permit: &'b AnyPermit<'a, R, A, C>) -> Self {
        Self {
            permit: permit.permit.access().into(),
            container: permit.container,
        }
    }
}

impl<'a: 'b, 'b, A, B, C: ?Sized> From<&'b mut AnyPermit<'a, Mut, A, C>>
    for AnyPermit<'b, Mut, B, C>
where
    Permit<Mut, A>: Into<Permit<Mut, B>>,
{
    fn from(permit: &'b mut AnyPermit<'a, Mut, A, C>) -> Self {
        Self {
            permit: permit.permit.access().into(),
            container: permit.container,
        }
    }
}
