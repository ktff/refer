use super::*;
use crate::core::{
    container::RegionContainer, container::TypeContainer, AnyContainer, AnyItem, Container, Key,
    Ptr, Side,
};
use std::{
    any::TypeId,
    borrow::BorrowMut,
    ops::{Deref, RangeBounds},
};

pub struct AnyPermit<'a, R, C: ?Sized> {
    permit: Permit<R>,
    container: &'a C,
}

impl<'a, R, C: AnyContainer + ?Sized> AnyPermit<'a, R, C> {
    pub fn new(container: &'a mut C) -> Self {
        // SAFETY: We have exclusive access to the container so any Permit is valid.
        unsafe { Self::unsafe_new(Permit::new(), container) }
    }

    /// SAFETY: Caller must ensure that it has the correct R & S access to C for the given 'a.
    pub unsafe fn unsafe_new(permit: Permit<R>, container: &'a C) -> Self {
        Self { container, permit }
    }

    /// UNSAFE: Caller must ensure some kind of division between jurisdictions of the two permits.
    pub unsafe fn unsafe_split<T>(&self, map: impl FnOnce(Self) -> T) -> T {
        map(Self {
            container: self.container,
            permit: self.permit.access(),
        })
    }

    pub(super) fn access(&self) -> Permit<R> {
        self.permit.access()
    }

    pub fn container(&self) -> &'a C {
        self.container
    }

    pub fn slot<K, T: core::DynItem + ?Sized>(self, key: Key<K, T>) -> SlotPermit<'a, R, K, T, C>
    where
        C: AnyContainer,
    {
        self.ty().slot(key)
    }

    pub fn ty<T: core::DynItem + ?Sized>(self) -> TypePermit<'a, T, R, C>
    where
        C: AnyContainer,
    {
        TypePermit::new(self)
    }

    pub fn on_key<K, T: core::DynItem + ?Sized>(
        self,
        key: Key<K, T>,
    ) -> SlotPermit<'a, R, K, T, C> {
        self.slot(key)
    }

    /// Iterates over valid slot permit of type in ascending order.
    pub fn iter<T: core::Item>(
        self,
    ) -> impl Iterator<Item = SlotPermit<'a, R, core::Ref<'a>, T, C>> {
        let container = self.container;
        std::iter::successors(container.first_key(TypeId::of::<T>()), move |&key| {
            container.next_key(TypeId::of::<T>(), key.ptr())
        })
        .map(move |key| {
            // SAFETY: First-next iteration ensures that we don't access the same slot twice.
            unsafe { self.unsafe_split(|permit| permit.slot(key.assume())) }
        })
    }

    /// Iterates over valid slot permit of type in ascending order.
    pub fn iter_dyn(
        self,
        ty: TypeId,
    ) -> impl Iterator<Item = SlotPermit<'a, R, core::Ref<'a>, dyn AnyItem, C>> {
        let container = self.container;
        std::iter::successors(container.first_key(ty), move |&key| {
            container.next_key(ty, key.ptr())
        })
        .map(move |key| {
            // SAFETY: First-next iteration ensures that we don't access the same slot twice.
            unsafe { self.unsafe_split(|permit| permit.slot(key)) }
        })
    }

    /// Iterates over valid keys of type in ascending order.
    pub fn keys(&self, ty: TypeId) -> impl Iterator<Item = Key> + 'a {
        let container = self.container;
        std::iter::successors(container.first_key(ty).map(|key| key.ptr()), move |&key| {
            container.next_key(ty, key).map(|key| key.ptr())
        })
    }

    // None if a == b
    pub fn split_pair<A, B>(
        self,
        a: Key<A>,
        b: Key<B>,
    ) -> Option<(
        SlotPermit<'a, R, A, dyn AnyItem, C>,
        SlotPermit<'a, R, B, dyn AnyItem, C>,
    )> {
        if a == b {
            None
        } else {
            // SAFETY: We just checked that a != b.
            Some(unsafe { (self.unsafe_split(|permit| permit.slot(a)), self.slot(b)) })
        }
    }
}

// impl<'a, A: Into<Shell>, C: AnyContainer + ?Sized> AnyPermit<'a, Mut, A, C> {
//     pub fn connect_dyn<T: core::DynItem + ?Sized>(
//         &mut self,
//         from: Key,
//         to: Key<T>,
//     ) -> Result<core::Key<Refer,T>> {
//         self.peek_dyn(to)?.shell_add(from);
//         Ok(core::Ref::new(to))
//     }

//     pub fn disconnect_dyn<T: core::DynItem + ?Sized>(
//         &mut self,
//         from: Key,
//         to: core::Key<Refer,T>,
//     ) -> Result<()> {
//         Ok(self.peek_dyn(to.key())?.shell_remove(from))
//     }
// }

impl<'a, C: AnyContainer + ?Sized> AnyPermit<'a, Mut, C> {
    pub fn split_types(self) -> TypeSplitPermit<'a, C>
    where
        C: Sized,
    {
        TypeSplitPermit::new(self)
    }

    pub fn split_slots(self) -> SlotSplitPermit<'a, C> {
        SlotSplitPermit::new(self)
    }

    pub fn split_of<K: Copy, T: core::DynItem + ?Sized>(
        self,
        key: Key<K, T>,
    ) -> (SlotPermit<'a, Mut, K, T, C>, SubjectPermit<'a, C>) {
        SubjectPermit::new(self, key)
    }
}

impl<'a, R, C: ?Sized> AnyPermit<'a, R, C> {
    pub fn step<T: core::Item>(self) -> Option<AnyPermit<'a, R, C::Sub>>
    where
        C: TypeContainer<T>,
    {
        let Self { container, permit } = self;
        container
            .get()
            .map(|container| AnyPermit { container, permit })
    }

    pub fn step_into(self, index: usize) -> Option<AnyPermit<'a, R, C::Sub>>
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
    ) -> Option<impl Iterator<Item = AnyPermit<'a, R, C::Sub>>>
    where
        C: RegionContainer,
    {
        let Self { container, permit } = self;
        Some(container.iter(range)?.map(move |container| AnyPermit {
            container,
            permit: permit.access(),
        }))
    }
}

impl<'a, C: ?Sized> AnyPermit<'a, Mut, C> {
    pub fn borrow(&self) -> AnyPermit<Ref, C> {
        self.into()
    }

    pub fn borrow_mut(&mut self) -> AnyPermit<Mut, C> {
        self.into()
    }

    // TODO: Move peek to Key.

    pub fn peek<'b, T: core::Item>(
        &'b mut self,
        key: Key<core::Ref<'b>, T>,
    ) -> core::Slot<'b, T, Mut>
    where
        C: Container<T>,
    {
        self.borrow_mut().slot(key).get()
    }

    pub fn peek_dyn<'b, T: core::DynItem + ?Sized>(
        &'b mut self,
        key: Key<core::Ref<'b>, T>,
    ) -> core::DynSlot<'b, T, Mut>
    where
        C: AnyContainer,
    {
        self.borrow_mut().slot(key).get_dyn()
    }

    /// Connects subjects source side edges to drain Items.
    /// UNSAFE: Caller must ensure that this is called only once, when subject was put into the slot.
    pub unsafe fn connect_source_edges<T: core::Item>(&mut self, subject: Key<core::Ref, T>)
    where
        C: Container<T>,
    {
        let (subject, mut others) = self.borrow_mut().split_of(subject);
        let subject = subject.get();
        for edge in subject.edges(Some(Side::Source)) {
            if let Some(drain) = others.slot(edge.object) {
                // SAFETY: Subject,source,this exists at least for the duration of this function.
                //         By adding it(Key) to the drain, anyone dropping the drain will know that
                //         this subject needs to be notified. This ensures that edge in subject is
                //         valid for it's lifetime.
                let source = unsafe { Key::<_, T>::new_owned(subject.key().index()) };
                let mut drain = drain.get_dyn();
                let excess_key = match drain.add_drain_edge(source){
                    Ok (key) => key,
                    Err(_) => panic!(
                        "Invalid item edge: subject {} -> object {}, object not drain, but owned reference of him exists.",
                        subject.key(), drain.key(),
                    )
                };
                drain.any_delete_ref(excess_key);
            } else {
                // We skip self references
            }
        }
    }
}

impl<'a, C: ?Sized> AnyPermit<'a, Ref, C> {
    pub fn peek<T: core::Item>(self, key: Key<core::Ref<'a>, T>) -> core::Slot<'a, T, Ref>
    where
        C: Container<T>,
    {
        self.slot(key).get()
    }

    pub fn peek_dyn<T: core::DynItem + ?Sized>(
        self,
        key: Key<core::Ref<'a>, T>,
    ) -> core::DynSlot<'a, T, Ref>
    where
        C: AnyContainer,
    {
        self.slot(key).get_dyn()
    }
}

impl<'a, R, C: ?Sized> Deref for AnyPermit<'a, R, C> {
    type Target = &'a C;

    fn deref(&self) -> &Self::Target {
        &self.container
    }
}

impl<'a, C: ?Sized> Copy for AnyPermit<'a, Ref, C> {}

impl<'a, C: ?Sized> Clone for AnyPermit<'a, Ref, C> {
    fn clone(&self) -> Self {
        Self {
            permit: Permit { ..self.permit },
            ..*self
        }
    }
}

impl<'a, C: ?Sized> From<AnyPermit<'a, Mut, C>> for AnyPermit<'a, Ref, C> {
    fn from(AnyPermit { permit, container }: AnyPermit<'a, Mut, C>) -> Self {
        Self {
            permit: permit.into(),
            container,
        }
    }
}

impl<'a: 'b, 'b, R, C: ?Sized> From<&'b AnyPermit<'a, R, C>> for AnyPermit<'b, Ref, C>
where
    Permit<R>: Into<Permit<Ref>>,
{
    fn from(permit: &'b AnyPermit<'a, R, C>) -> Self {
        Self {
            permit: permit.permit.access().into(),
            container: permit.container,
        }
    }
}

impl<'a: 'b, 'b, C: ?Sized> From<&'b mut AnyPermit<'a, Mut, C>> for AnyPermit<'b, Mut, C> {
    fn from(permit: &'b mut AnyPermit<'a, Mut, C>) -> Self {
        Self {
            permit: permit.permit.access().into(),
            container: permit.container,
        }
    }
}
