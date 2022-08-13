mod any;
mod collection;
mod entry;
mod key;
mod path;
mod reference;

use std::any::TypeId;

pub use any::AnyCollection;
pub use collection::Collection;
pub use entry::*;
pub use key::*;
pub use path::*;
pub use reference::*;

/* NOTES

- Nuzno je razdvojiti read step od write step zbog
bledanja side effecta izmjene from iz jedne mutacije u druge read/write funkcije.

- Ownership nema smisla kao feature za kolekcija. Ownership postoji za strukture koje
item normalno ima kao plain object, sve ostalo sto posotji kao item u kolekciji moze
biti referencirano od strane drugih te posto se nemoze removati ako postoji referenca
te da se nemoze sigurno izvesti mutacija ownera i onwed u isto vrijeme, ownership se
ne cini korisnim.

- Build on top:
   X. Polymorphism
   X. Chunked collections as one opaque collection derived from composite collection
        - Na konacnom kolekcijom je da napravi ovo
        - Moguce je pomoci s macro koji bi napravio chunked collection ze jedan tip
   x. Composite collections
        - Na konacnom kolekcijom je da napravi ovo
   x. Conncurrency
      - Kroz chunked collections se cini kao dobar put
      - Na korisniku je da doda lock na koleckije podatke ovisno o tome što joj treba.
   x. Key<Level> kako bi se razlikovalo kljuceve na razlicitim razinama.
      - Izvedeno pomocu Global/Local tipova
   x. Prostorni collection
      - Na korisnicima je da dodaju extra funkcije
*/

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Error {
    /// Operation was not possible because key is not in use for any item
    KeyIsNotInUse(AnyKey),
    /// Operation was not possible because item is referenced
    ItemIsReferenced(AnyKey),
    /// Operation was not possible because item is not in local memory.
    /// Generally can be thrown as when KeyIsNotInUse by collections that
    /// have lazy load.
    ItemNotInMemory(AnyKey),
    /// Operation was not possible because item type is not supported
    UnsupportedType(TypeId),
    /// Some collection or data is under some kind of lock.
    ///
    /// Some kind of probabilistic backoff is advised. ex. 50% to back off 50% to not.
    Locked,
    /// Operation was not possible because there are no more keys
    /// Generally collections is full and won't accept any more items
    OutOfKeys,
}

impl Error {
    pub fn recoverable(self) -> bool {
        match self {
            Error::KeyIsNotInUse(_) => false,
            Error::ItemIsReferenced(_) => true,
            Error::ItemNotInMemory(_) => true,
            Error::UnsupportedType(_) => false,
            Error::Locked => true,
            Error::OutOfKeys => false,
        }
    }

    pub fn unrecoverable(self) -> bool {
        !self.recoverable()
    }
}
