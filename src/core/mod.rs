mod any_ref;
mod coll_mut;
mod coll_polly;
mod coll_ref;
mod key;

use std::ops::{Deref, DerefMut};

pub use any_ref::*;
pub use coll_mut::{CollectionMut, MutEntry};
pub use coll_polly::{AnyEntry, PollyCollection};
pub use coll_ref::{CollectionRef, RefEntry};
pub use key::*;

/* NOTES

- Nuzno je razdvojiti read step od write step zbog
bledanja side effecta izmjene from iz jedne mutacije u druge read/write funkcije.

- Ownership nema smisla kao feature za kolekcija. Ownership postoji za strukture koje
item normalno ima kao plain object, sve ostalo sto posotji kao item u kolekciji moze
biti referencirano od strane drugih te posto se nemoze removati ako postoji referenca
te da se nemoze sigurno izvesti mutacija ownera i onwed u isto vrijeme, ownership se
ne cini korisnim.

TODO:

- Build on top:
   1. Polymorphism
   2. Chunked collections as one opaque collection
   3. Conncurrency
      - Kroz chunked collections se cini kao dobar put
   - Composite collections

*/

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Error {
    KeyIsNotInUse,
    ItemIsReferenced,
    UnsupportedType,
}

/// Item that references to other items.
pub trait Composite: 'static {
    /// Calls for each reference.
    fn visit_references(&self, call: impl FnMut(AnyRef));
}
