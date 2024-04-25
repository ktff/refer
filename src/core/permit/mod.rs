pub mod access;
pub mod add;
pub mod remove;

pub use access::{All, Keys, Not, Types};
pub use remove::ContainerExt;

use std::marker::PhantomData;

//? NOTE: Permit system is by no means complete, so it's fine to extend it.

// TODO: Test permit system, test compile failures?
// TODO: Unify Add with Access?

// Extension TODO: Like ExclusivePermit, SharedPermit could be constructed from ExclusivePermit for concurrent mutation.

pub trait Permit: Into<Ref> + 'static {
    /// UNSAFE: Caller must ensure one of:
    /// - permits represent disjoint set of keys
    /// - self is exclusively borrowed by the other
    unsafe fn copy(&self) -> Self;

    fn borrow(&self) -> Ref {
        Ref(PhantomData)
    }
}

pub struct Add(PhantomData<()>);

impl Add {
    /// UNSAFE: So that it's constructed sparingly.
    unsafe fn new() -> Self {
        Self(PhantomData)
    }
}

impl Permit for Add {
    unsafe fn copy(&self) -> Self {
        Self(PhantomData)
    }
}

impl From<Add> for Mut {
    fn from(_: Add) -> Self {
        Mut(PhantomData)
    }
}

impl From<Add> for Ref {
    fn from(_: Add) -> Self {
        Ref(PhantomData)
    }
}

pub struct Mut(PhantomData<()>);

impl Permit for Mut {
    unsafe fn copy(&self) -> Self {
        Self(PhantomData)
    }
}

impl From<Mut> for Ref {
    fn from(_: Mut) -> Self {
        Ref(PhantomData)
    }
}

#[derive(Clone, Copy)]
pub struct Ref(PhantomData<()>);

impl Permit for Ref {
    unsafe fn copy(&self) -> Self {
        Self(PhantomData)
    }
}
