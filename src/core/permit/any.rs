use super::*;
use crate::core::{AnyContainer, AnyKey, Key};
use log::*;
use std::{any::TypeId, ops::Deref};

pub struct AnyPermit<'a, R, A, C: ?Sized> {
    permit: Permit<R, A>,
    container: &'a C,
}

impl<'a, R, A, C: AnyContainer + ?Sized> AnyPermit<'a, R, A, C> {
    /// SAFETY: Caller must ensure that it has the correct R & S access to C for the given 'a.
    pub unsafe fn new(container: &'a C) -> Self {
        Self {
            container,
            permit: Permit::new(),
        }
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

    pub fn iter(self, key: TypeId) -> impl Iterator<Item = core::AnySlot<'a, R, A>> {
        let Self { container, permit } = self;
        std::iter::successors(container.first(key), move |&key| container.next(key)).map(
            move |key| {
                container
                    .get_slot_any(key)
                    // SAFETY: Type level logic of AnyPermit ensures that it has sufficient access for 'a to all slots.
                    //         Furthermore first-next iteration ensures that we don't access the same slot twice.
                    .map(|slot| unsafe { core::AnySlot::new(key, slot, permit.access()) })
                    .expect("Should be valid key")
            },
        )
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
    ) -> core::DynRef<T> {
        self.borrow_mut()
            .slot(to)
            .get_dyn()
            .map_err(|error| {
                error!("Failed to connect {:?} -> {:?}, error: {}", from, to, error);
                error
            })
            .expect("Failed to connect")
            .shell_add(from);

        core::DynRef::new(to)
    }

    pub fn disconnect_dyn<T: core::DynItem + ?Sized>(&mut self, from: AnyKey, to: core::DynRef<T>) {
        self.borrow_mut()
            .slot(to.key())
            .get_dyn()
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

impl<'a, C: AnyContainer + ?Sized> AnyPermit<'a, Mut, Slot, C> {
    pub fn split_parts(self) -> (AnyPermit<'a, Mut, Item, C>, AnyPermit<'a, Mut, Shell, C>) {
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

impl<'a, A, C: ?Sized> AnyPermit<'a, Mut, A, C> {
    pub fn borrow(&self) -> AnyPermit<Ref, A, C> {
        self.into()
    }

    pub fn borrow_mut(&mut self) -> AnyPermit<Mut, A, C> {
        self.into()
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
