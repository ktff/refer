pub mod access;
mod add;
mod any;
mod path;
mod remove;
mod slot;
mod slot_split;
mod subject;
mod ty;
mod type_split;

pub use add::AddPermit;
pub use any::AnyPermit;
pub use path::PathPermit;
pub use remove::RemovePermit;
pub use slot::SlotPermit;
pub use slot_split::SlotSplitPermit;
pub use subject::SubjectPermit;
pub use ty::TypePermit;
pub use type_split::TypeSplitPermit;

use crate::core;
use std::marker::PhantomData;

//? NOTE: Permit system is by no means complete, so it's fine to extend it.

// TODO: Test permit system, test compile failures?

// TODO: Unify *Acccess under Access<R: Restriction>? Restrictions could be: Type<T>, Path, PathKey<T>, Key<T>, etc.

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
