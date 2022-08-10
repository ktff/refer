mod collection;
mod key;
mod polly;
mod reference;

use std::ops::{Deref, DerefMut};

pub use collection::{Collection, InitEntry, MutEntry, RefEntry};
pub use key::*;
pub use polly::{AnyEntry, PollyCollection};
pub use reference::*;

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
   X. Polymorphism
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

// /// Item that references to other items.
// pub trait Composite: 'static {
//     /// Calls for each reference.
//     fn visit_references(&self, call: impl FnMut(AnyRef));
// }

pub fn catch_error<F>(f: F)
where
    F: FnOnce() -> Result<(), Error>,
{
    if let Err(error) = f() {
        // TODO: Log error
        unimplemented!()
    }
}
