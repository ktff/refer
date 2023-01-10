mod any;
mod path;
mod slot;
mod slot_split;
mod ty;
mod type_split;

pub use any::AnyPermit;
pub use path::PathPermit;
pub use slot::SlotPermit;
pub use slot_split::SlotSplitPermit;
pub use ty::TypePermit;
pub use type_split::TypeSplitPermit;

use crate::core;
use std::marker::PhantomData;

// TODO: Test permit system, test compile failures?

pub struct Mut;

pub struct Ref;
impl From<Mut> for Ref {
    fn from(_: Mut) -> Self {
        Ref
    }
}

pub struct Slot;

pub struct Item;
impl From<Slot> for Item {
    fn from(_: Slot) -> Self {
        Item
    }
}

pub struct Shell;
impl From<Slot> for Shell {
    fn from(_: Slot) -> Self {
        Shell
    }
}

pub struct Permit<R, A> {
    _marker: PhantomData<(R, A)>,
}

impl<R, A> Permit<R, A> {
    /// UNSAFE: So that it's constructed sparingly.
    pub unsafe fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn access(&self) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<R> Permit<R, Slot> {
    pub fn split(self) -> (Permit<R, Item>, Permit<R, Shell>) {
        (
            Permit {
                _marker: PhantomData,
            },
            Permit {
                _marker: PhantomData,
            },
        )
    }
}

impl<A> Permit<Mut, A> {
    pub fn borrow(&self) -> Permit<Ref, A> {
        Permit {
            _marker: PhantomData,
        }
    }
}

impl<A> Copy for Permit<Ref, A> {}

impl<A> Clone for Permit<Ref, A> {
    fn clone(&self) -> Self {
        Permit {
            _marker: PhantomData,
        }
    }
}

impl<A: Into<B>, B> From<Permit<Mut, A>> for Permit<Ref, B> {
    fn from(_: Permit<Mut, A>) -> Self {
        Permit {
            _marker: PhantomData,
        }
    }
}

impl<R> From<Permit<R, Slot>> for Permit<R, Item> {
    fn from(_: Permit<R, Slot>) -> Self {
        Permit {
            _marker: PhantomData,
        }
    }
}

impl<R> From<Permit<R, Slot>> for Permit<R, Shell> {
    fn from(_: Permit<R, Slot>) -> Self {
        Permit {
            _marker: PhantomData,
        }
    }
}
