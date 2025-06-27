use crate::core::{DynItem, Key};
use std::marker::PhantomData;

/// Creators of this guarantee no item will be removed for the lifetime 'a.
#[derive(Clone, Copy)]
pub struct Lifetime<'a>(PhantomData<&'a ()>);

impl<'a> Lifetime<'a> {
    pub unsafe fn new() -> Self {
        Self(PhantomData)
    }

    pub fn extend<D: DynItem + ?Sized>(
        self,
        key: Key<crate::core::Ref<'_>, D>,
    ) -> Key<crate::core::Ref<'a>, D> {
        // SAFETY: We have 'a lifetime guarantee no item will be removed so key is valid for 'a.
        unsafe { key.extend() }
    }
}
