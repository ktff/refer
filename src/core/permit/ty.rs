use super::*;
use crate::core::{
    container::LeafContainer, container::RegionContainer, container::TypeContainer, AnyContainer,
    Container, Key, Ptr,
};
use std::{
    any::TypeId,
    marker::PhantomData,
    ops::{Deref, RangeBounds},
};

pub struct TypePermit<'a, T: ?Sized, R, C: ?Sized> {
    permit: AnyPermit<'a, R, C>,
    _marker: PhantomData<&'a T>,
}

impl<'a, R, T: core::DynItem + ?Sized, C: AnyContainer + ?Sized> TypePermit<'a, T, R, C> {
    pub fn new(permit: AnyPermit<'a, R, C>) -> Self {
        Self {
            permit,
            _marker: PhantomData,
        }
    }

    /// UNSAFE: Caller must ensure some kind of division between jurisdictions of the two permits.
    pub unsafe fn unsafe_split<P>(&self, map: impl FnOnce(Self) -> P) -> P {
        map(self.permit.unsafe_split(|permit| permit.ty()))
    }

    pub(super) fn access(&self) -> Permit<R> {
        self.permit.access()
    }

    pub fn slot<K>(self, key: Key<K, T>) -> SlotPermit<'a, R, K, T, C> {
        SlotPermit::new(self, key)
    }
}

impl<'a, R, T: core::Item, C: ?Sized> TypePermit<'a, T, R, C> {
    pub fn step(self) -> Option<TypePermit<'a, T, R, C::Sub>>
    where
        C: TypeContainer<T>,
    {
        self.permit.step().map(TypePermit::new)
    }

    pub fn step_iter(self) -> impl Iterator<Item = SlotPermit<'a, R, core::Ref<'a>, T, C>>
    where
        C: LeafContainer<T> + AnyContainer,
    {
        // SAFETY: We are only accessing T type items.
        self.permit.iter()
    }

    /// Iterates over valid keys in ascending order.
    pub fn keys(&self) -> impl Iterator<Item = Key<Ptr, T>> + '_
    where
        C: AnyContainer,
    {
        self.permit.keys(TypeId::of::<T>()).map(|key| key.assume())
    }
}

impl<'a, R, T: core::DynItem + ?Sized, C: AnyContainer + ?Sized> TypePermit<'a, T, R, C> {
    pub fn step_into(self, index: usize) -> Option<TypePermit<'a, T, R, C::Sub>>
    where
        C: RegionContainer,
    {
        self.permit.step_into(index).map(TypePermit::new)
    }

    pub fn step_range(
        self,
        range: impl RangeBounds<usize>,
    ) -> Option<impl Iterator<Item = TypePermit<'a, T, R, C::Sub>>>
    where
        C: RegionContainer,
    {
        Some(self.permit.step_range(range)?.map(TypePermit::new))
    }
}

impl<'a, R, T: core::Item, C: Container<T> + ?Sized> TypePermit<'a, T, R, C> {
    pub fn on_path(self) -> PathPermit<'a, T, R, C> {
        PathPermit::new(self)
    }

    // None if a == b
    pub fn split_pair<A, B>(
        self,
        a: Key<A, T>,
        b: Key<B, T>,
    ) -> Option<(SlotPermit<'a, R, A, T, C>, SlotPermit<'a, R, B, T, C>)> {
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

// impl<'a, T: core::Item, A: Into<Shell>, C: Container<T> + ?Sized> TypePermit<'a, T, Mut, C> {
//     pub fn connect(&mut self, from: Key, to: Key<T>) -> core::Key<Refer,T> {
//         self.borrow_mut()
//             .slot(to)
//             .get()
//             .map_err(|error| {
//                 error!("Failed to connect {:?} -> {:?}, error: {}", from, to, error);
//                 error
//             })
//             .expect("Failed to connect")
//             .shell_add(from);

//         core::Ref::new(to)
//     }

//     pub fn disconnect(&mut self, from: Key, to: core::Key<Refer,T>) {
//         self.borrow_mut()
//             .slot(to.key())
//             .get()
//             .map_err(|error| {
//                 error!(
//                     "Failed to disconnect {:?} -> {:?}, error: {}",
//                     from,
//                     to.key(),
//                     error
//                 );
//                 error
//             })
//             .expect("Failed to disconnect")
//             .shell_remove(from)
//     }
// }

impl<'a, T: ?Sized, C: ?Sized> TypePermit<'a, T, Mut, C> {
    pub fn borrow(&self) -> TypePermit<T, Ref, C> {
        TypePermit {
            permit: (&self.permit).into(),
            _marker: PhantomData,
        }
    }

    pub fn borrow_mut(&mut self) -> TypePermit<T, Mut, C> {
        TypePermit {
            permit: (&mut self.permit).into(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T: ?Sized, R, C: ?Sized> Deref for TypePermit<'a, T, R, C> {
    type Target = &'a C;

    fn deref(&self) -> &Self::Target {
        &self.permit
    }
}

impl<'a, T: ?Sized, C: ?Sized> Copy for TypePermit<'a, T, Ref, C> {}

impl<'a, T: ?Sized, C: ?Sized> Clone for TypePermit<'a, T, Ref, C> {
    fn clone(&self) -> Self {
        Self {
            permit: self.permit,
            _marker: PhantomData,
        }
    }
}

impl<'a, T, C: ?Sized> From<TypePermit<'a, T, Mut, C>> for TypePermit<'a, T, Ref, C> {
    fn from(TypePermit { permit, .. }: TypePermit<'a, T, Mut, C>) -> Self {
        Self {
            permit: permit.into(),
            _marker: PhantomData,
        }
    }
}

impl<'a: 'b, 'b, T, R, C: ?Sized> From<&'b TypePermit<'a, T, R, C>> for TypePermit<'b, T, Ref, C>
where
    Permit<R>: Into<Permit<Ref>>,
{
    fn from(permit: &'b TypePermit<'a, T, R, C>) -> Self {
        Self {
            permit: (&permit.permit).into(),
            _marker: PhantomData,
        }
    }
}

impl<'a: 'b, 'b, T, C: ?Sized> From<&'b mut TypePermit<'a, T, Mut, C>>
    for TypePermit<'b, T, Mut, C>
{
    fn from(permit: &'b mut TypePermit<'a, T, Mut, C>) -> Self {
        Self {
            permit: (&mut permit.permit).into(),
            _marker: PhantomData,
        }
    }
}
