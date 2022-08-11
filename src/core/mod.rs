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

TODO:

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

// pub fn catch_error<F>(f: F)
// where
//     F: FnOnce() -> Result<(), Error>,
// {
//     if let Err(error) = f() {
//         // TODO: Log error
//         unimplemented!()
//     }
// }

// pub struct CompositeCollection<C>(C);

// impl<C: AnyCollection> AnyCollection for CompositeCollection<C> {
//     type AE<'a, L: LayerRef<Down = Self> + 'a>=CompositeAnyEntry<'a,C,L>
//     where
//         Self: 'a;

//     fn first_key_any(&self) -> Option<AnyKey> {
//         self.0.first_key_any()
//     }

//     fn next_key_any(&self, key: AnyKey) -> Option<AnyKey> {
//         self.0.next_key_any(key)
//     }

//     fn get_any<'a, Tc: LayerRef<Down = Self> + 'a>(
//         top: &'a Tc,
//         key: AnyKey,
//     ) -> Result<Self::AE<'a, Tc>, Error> {
//         unimplemented!()
//     }

//     fn chunks_any(&self) -> Vec<(AnyKey, AnyKey)> {
//         self.0.chunks_any()
//     }
// }

// // impl<T: ?Sized + 'static, C: Collection<T>> Collection<T> for CompositeCollection<C> {
// //     type IE<'a> = C::IE<'a> where Self:'a;

// //     type RE<'a> = C::RE<'a> where Self:'a;

// //     type ME<'a> = C::ME<'a> where Self:'a;

// //     fn indices_bits(&self) -> usize {
// //         self.collection.indices_bits()
// //     }

// //     fn first_key(&self) -> Option<Key<T>> {
// //         self.collection.first_key()
// //     }

// //     fn next_key(&self, key: Key<T>) -> Option<Key<T>> {
// //         self.collection.next_key(key)
// //     }

// //     fn add<'a>(&'a mut self) -> Self::IE<'a> {
// //         self.collection.add()
// //     }

// //     fn get<'a>(&'a self, key: impl Into<Key<T>>) -> Result<Self::RE<'a>, Error> {
// //         self.collection.get(key)
// //     }

// //     fn get_mut<'a>(&'a mut self, key: impl Into<Key<T>>) -> Result<Self::ME<'a>, Error> {
// //         self.collection.get_mut(key)
// //     }

// //     fn chunks(&self) -> Vec<(Key<T>, Key<T>)> {
// //         self.collection.chunks()
// //     }
// // }

// pub struct CompositeAnyEntry<
//     'a,
//     C: AnyCollection + 'a,
//     L: LayerRef<Down = CompositeCollection<C>> + 'a,
// > {
//     entry: C::AE<'a, CompositeLayer<'a, L>>,
// }

// impl<'a, C: AnyCollection + 'a, L: LayerRef<Down = CompositeCollection<C>> + 'a> AnyEntry<'a>
//     for CompositeAnyEntry<'a, C, L>
// {
//     // TODO: Placeholder
//     type IterAny = <C::AE<'a, CompositeLayer<'a, L>> as AnyEntry<'a>>::IterAny;
//     type Coll = L;

//     fn key_any(&self) -> AnyKey {
//         self.entry.key_any()
//     }

//     fn from_any(&self) -> Self::IterAny {
//         self.entry.from_any()
//     }

//     fn referenced(&self) -> bool {
//         self.entry.referenced()
//     }

//     fn collection(&self) -> &Self::Coll {
//         self.entry.collection().layer
//     }
// }

// pub struct CompositeLayer<'a, L> {
//     layer: &'a L,
// }

// impl<'a, C, Tc: LayerRef<Down = CompositeCollection<C>>> LayerRef for CompositeLayer<'a, Tc> {
//     type Down = C;
//     fn down(&self) -> &Self::Down {
//         &self.layer.down().0
//     }
// }
