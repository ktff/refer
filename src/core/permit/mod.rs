pub mod access;
pub mod add;
pub mod remove;

pub use access::{All, Keys, Not, Types};

use std::marker::PhantomData;

//? NOTE: Permit system is by no means complete, so it's fine to extend it.

// TODO: Test permit system, test compile failures?

// Extension TODO: Like ExclusivePermit, SharedPermit could be constructed from ExclusivePermit for concurrent mutation.

pub struct Mut;

pub struct Ref;
impl From<Mut> for Ref {
    fn from(_: Mut) -> Self {
        Ref
    }
}

pub struct Permit<R> {
    _marker: PhantomData<R>,
}

impl<R> Permit<R> {
    /// UNSAFE: So that it's constructed sparingly.
    pub unsafe fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    // TODO: Restrict this
    pub fn access(&self) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl Permit<Mut> {
    pub fn borrow(&self) -> Permit<Ref> {
        Permit {
            _marker: PhantomData,
        }
    }
}

impl Copy for Permit<Ref> {}

impl Clone for Permit<Ref> {
    fn clone(&self) -> Self {
        Permit {
            _marker: PhantomData,
        }
    }
}

impl From<Permit<Mut>> for Permit<Ref> {
    fn from(_: Permit<Mut>) -> Self {
        Permit {
            _marker: PhantomData,
        }
    }
}
