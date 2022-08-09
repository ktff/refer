mod coll_mut;
mod coll_ref;
mod key;

use std::ops::{Deref, DerefMut};

pub use coll_mut::{CollectionMut, MutEntry};
pub use coll_ref::{CollectionRef, RefEntry};
pub use key::*;

pub enum Error {
    KeyIsNotInUse,
    ItemIsReferenced,
}

/// Item that references other items.
pub trait Composite: 'static {
    /// Calls for each reference.
    fn references(&self, call: impl FnMut(AnyKey));
}
