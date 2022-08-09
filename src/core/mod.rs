mod coll_mut;
mod coll_ref;
mod key;

use std::ops::{Deref, DerefMut};

pub use coll_mut::{CollectionMut, MutEntry};
pub use coll_ref::{CollectionRef, RefEntry};
pub use key::*;

/* NOTES

- Nuzno je razdvojiti read step od write step zbog
bledanja side effecta izmjene from iz jedne mutacije u druge read/write funkcije.

*/

pub enum Error {
    KeyIsNotInUse,
    ItemIsReferenced,
    DifferentOwner,
}

/// Item that references other items.
pub trait Composite: 'static {
    /// Calls for each reference.
    fn references(&self, call: impl FnMut(AnyKey));
}

// pub trait CollectionOwn<T: ?Sized + 'static>: CollectionMut<T> {
//     type OE<'a>: OwnEntry<'a, T = T>
//     where
//         Self: 'a;

//     type IterOwn<'a>: Iterator<Item = Self::OE<'a>> + 'a
//     where
//         Self: 'a;

//     /// Will error if the key is not in use or if collection is not owner.
//     fn get_own<'a>(&'a self, key: Key<T>) -> Result<Self::OE<'a>, Error>;

//     /// Iters collection owned items.
//     fn iter_own<'a>(&'a self) -> Self::IterOwn<'a>;
// }

// pub trait OwnEntry<'a>: MutEntry<'a> {

//     fn get_own<'b,T>(&'b mut self,key: Key<T>)-> Result<>
// }
