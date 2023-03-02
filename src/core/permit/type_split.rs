use super::*;
use crate::core::Container;
use std::any::TypeId;

pub struct TypeSplitPermit<'a, C: ?Sized> {
    permit: AnyPermit<'a, Mut, C>,
    splitted: Vec<TypeId>,
}

impl<'a, C: ?Sized> TypeSplitPermit<'a, C> {
    pub fn new(permit: AnyPermit<'a, Mut, C>) -> Self {
        Self {
            permit,
            splitted: Vec::new(),
        }
    }

    pub fn ty<T: core::Item>(&mut self) -> Option<TypePermit<'a, T, Mut, C>>
    where
        C: Container<T>,
    {
        if self.splitted.contains(&TypeId::of::<T>()) {
            None
        } else {
            self.splitted.push(TypeId::of::<T>());
            // SAFETY: We just checked that the type is not splitted.
            Some(unsafe { self.permit.unsafe_split(|permit| permit.ty()) })
        }
    }
}
