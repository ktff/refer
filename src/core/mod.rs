mod any;
mod collection;
mod entity;
mod item;
mod key;
mod reference;
mod shell;

use std::any::TypeId;

pub use any::*;
pub use collection::*;
pub use entity::*;
pub use item::*;
pub use key::*;
pub use reference::*;
pub use shell::*;

/* NOTES

- Goal is to completely prevent memory errors, and to discourage logical errors.

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
   x. Reference lifetime
   x. Key<Locality>, no, it's the job of reference to track this.
   *. Item i Ostalo su odvojene stvari
      - Ovo je prakticno rjesenje za problem duplog MutEntry jedan posudenog od drugog nad istim itemom gdje unutrasnji
        MutEntry remova item te se vrati na vanjski MutEntry koji ocekuje valjani podatak ali ga nema.
      - Da se ovo fixa a da se omoguci followanje referenci,
   *. Izdvoji parrallelnost
   *. Ukloniti Path
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
    /// Key is not local.
    NotLocalKey(AnyKey),
    /// Item is being borrowed.
    Borrowed,
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
            Error::NotLocalKey(_) => false,
            Error::Borrowed => true,
        }
    }

    pub fn unrecoverable(self) -> bool {
        !self.recoverable()
    }
}

// ********************* Locality *********************

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locality {
    /// Top
    Global,
    /// Bottom
    Local,
}

pub trait Localized {
    const L: Locality;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Global;

impl Localized for Global {
    const L: Locality = Locality::Global;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Local;

impl Localized for Local {
    const L: Locality = Locality::Local;
}

/// Speed bump to encourage proper usage
pub enum LocalizedData<T> {
    Global(T),
    Local(T),
}

// ************************ Convenient methods *************************** //

// impl<D: Directioned, T: ?Sized + 'static> Ref<T, Global, D> {
//     /// Initializes T with provided init closure and adds self as reference.
//     pub fn add<'a, P: PathMut<'a>, M: MutEntry<'a, P>>(
//         from: &mut M,
//         init: impl FnOnce(<P::Top as Collection<T>>::IE<'_, &mut P::Top>) -> Result<Key<T>, Error>,
//     ) -> Result<Self, Error>
//     where
//         P::Top: Collection<T> + Collection<M::T>,
//     {
//         let key = init(Collection::<T>::add(from.path_mut().top_mut()))?;

//         Self::bind(from, key)
//     }

//     /// Creates new reference to the given item.
//     pub fn bind<'a, P: PathMut<'a>, E: MutEntry<'a, P>>(
//         from: &mut E,
//         key: Key<T>,
//     ) -> Result<Self, Error>
//     where
//         P::Top: Collection<T> + Collection<E::T>,
//     {
//         let this = Ref::<T, Global, D>(key, PhantomData);
//         let from_ref = this.from(from);
//         this.entry_mut(from)?.add_from(from_ref);

//         Ok(this)
//     }

//     pub fn init<'a, P: PathMut<'a>, E: InitItemEntry<'a, P>>(
//         from: &mut E,
//         key: Key<T>,
//     ) -> Result<Self, Error>
//     where
//         P::Top: Collection<T> + Collection<E::T>,
//     {
//         let this = Ref::<T, Global, D>(key, PhantomData);
//         from.add_to(this)?;

//         Ok(this)
//     }

//     pub fn entry<'a: 'b, 'b, P: PathRef<'a>, M: RefItemEntry<'a, P>>(
//         self,
//         from: &'b M,
//     ) -> Result<<P::Top as Collection<T>>::RE<'b, &'b P::Top>, Error>
//     where
//         P::Top: Collection<T>,
//     {
//         Collection::<T>::entry(from.path().top(), self.key())
//     }

//     pub fn entry_mut<'a: 'b, 'b, P: PathMut<'a>, M: MutEntry<'a, P>>(
//         self,
//         from: &'b mut M,
//     ) -> Result<<P::Top as Collection<T>>::ME<'b, &'b mut P::Top>, Error>
//     where
//         P::Top: Collection<T>,
//     {
//         Collection::<T>::entry_mut(from.path_mut().top_mut(), self.key())
//     }

//     pub fn get<'a: 'b, 'b, P: PathRef<'a>, M: RefItemEntry<'a, P>>(
//         self,
//         from: &'b M,
//     ) -> Result<<<P::Top as Collection<T>>::RE<'b, &'b P::Top>>::Item, Error>
//     where
//         P::Top: Collection<T>,
//     {
//         self.entry(from)?.item()
//     }

//     pub fn get_mut<'a: 'b, 'b, P: PathMut<'a>, M: MutEntry<'a, P>>(
//         self,
//         from: &'b mut M,
//     ) -> Result<<<P::Top as Collection<T>>::ME<'b, &'b mut P::Top>>::MutItem, Error>
//     where
//         P::Top: Collection<T>,
//     {
//         self.entry_mut(from)?.item_mut()
//     }

//     /// Removes this reference from collection.
//     pub fn remove<'a, P: PathMut<'a>, M: MutEntry<'a, P>>(self, from: &mut M) -> Result<(), Error>
//     where
//         P::Top: Collection<T> + Collection<M::T>,
//     {
//         let from_ref = self.from(from);
//         self.entry_mut(from).map(|mut to| to.remove_from(from_ref))
//     }

//     /// Returns Ref referencing from.
//     fn from<'a, P: PathRef<'a>, M: RefItemEntry<'a, P>>(self, from: &M) -> Ref<M::T, Global, D>
//     where
//         P::Top: Collection<T> + Collection<M::T>,
//     {
//         Ref::<M::T, Global, D>(from.key(), PhantomData)
//     }
// }

// impl<D: Directioned, T: ?Sized + 'static> Ref<T, Local, D> {
//     /// Initializes T with provided init closure and adds self as reference.
//     pub fn add<'a, P: PathMut<'a>, M: MutEntry<'a, P>>(
//         from: &mut M,
//         init: impl FnOnce(
//             <P::Bottom as Collection<T>>::IE<'_, BorrowPathMut<'a, '_, P>>,
//         ) -> Result<Key<T>, Error>,
//     ) -> Result<Self, Error>
//     where
//         P::Bottom: Collection<T> + Collection<M::T>,
//     {
//         let from_key = from.key();
//         let mut entry = Collection::<T>::add(from.path_mut().borrow_mut());
//         let _ = entry.localize(from_key)?;
//         let key = init(entry)?;

//         Self::bind(from, key)
//     }

//     /// Creates new reference to the given item.
//     /// None if key is not in local/bottom collection.
//     ///
//     /// Errors:
//     /// - Keys are not local
//     pub fn bind<'a, P: PathMut<'a>, E: MutEntry<'a, P>>(
//         from: &mut E,
//         key: Key<T>,
//     ) -> Result<Self, Error>
//     where
//         P::Bottom: Collection<T> + Collection<E::T>,
//     {
//         let local_key = from
//             .path()
//             .bottom_key(key)
//             .ok_or_else(|| Error::NotLocalKey(key.into()))?;
//         let this = Ref::<T, Local, D>(local_key, PhantomData);
//         let from_ref = this.from(from);
//         this.entry_mut(from)?.add_from(from_ref);

//         Ok(this)
//     }

//     pub fn init<'a, P: PathMut<'a>, E: InitItemEntry<'a, P>>(
//         from: &mut E,
//         key: Key<T>,
//     ) -> Result<Self, Error>
//     where
//         P::Bottom: Collection<T> + Collection<E::T>,
//     {
//         let local_key = from.localize(key)?;
//         let this = Ref::<T, Local, D>(local_key, PhantomData);
//         from.add_to(this)?;

//         Ok(this)
//     }

//     pub fn entry<'a: 'b, 'b, P: PathRef<'a>, M: RefItemEntry<'a, P>>(
//         self,
//         from: &'b M,
//     ) -> Result<<P::Bottom as Collection<T>>::RE<'b, BorrowPathRef<'a, 'b, P>>, Error>
//     where
//         P::Bottom: Collection<T>,
//     {
//         <P::Bottom as Collection<T>>::entry(from.path().borrow(), self.0)
//     }

//     pub fn entry_mut<'a: 'b, 'b, P: PathMut<'a>, M: MutEntry<'a, P>>(
//         self,
//         from: &'b mut M,
//     ) -> Result<<P::Bottom as Collection<T>>::ME<'b, BorrowPathMut<'a, 'b, P>>, Error>
//     where
//         P::Bottom: Collection<T>,
//     {
//         <P::Bottom as Collection<T>>::entry_mut(from.path_mut().borrow_mut(), self.0)
//     }

//     pub fn get<'a: 'b, 'b, P: PathRef<'a>, M: RefItemEntry<'a, P>>(
//         self,
//         from: &'b M,
//     ) -> Result<<<P::Bottom as Collection<T>>::RE<'b, BorrowPathRef<'a, 'b, P>>>::Item, Error>
//     where
//         P::Bottom: Collection<T>,
//     {
//         self.entry(from)?.item()
//     }

//     pub fn get_mut<'a: 'b, 'b, P: PathMut<'a>, M: MutEntry<'a, P>>(
//         self,
//         from: &'b mut M,
//     ) -> Result<<<P::Bottom as Collection<T>>::ME<'b, BorrowPathMut<'a, 'b, P>>>::MutItem, Error>
//     where
//         P::Bottom: Collection<T>,
//     {
//         self.entry_mut(from)?.item_mut()
//     }

//     pub fn remove<'a, P: PathMut<'a>, M: MutEntry<'a, P>>(self, from: &mut M) -> Result<(), Error>
//     where
//         P::Bottom: Collection<T> + Collection<M::T>,
//     {
//         let from_ref = self.from(from);
//         self.entry_mut(from).map(|mut to| to.remove_from(from_ref))
//     }

//     fn from<'a, P: PathRef<'a>, M: RefItemEntry<'a, P>>(self, from: &M) -> Ref<M::T, Local, D>
//     where
//         P::Bottom: Collection<T> + Collection<M::T>,
//     {
//         Ref::<M::T, Local, D>(
//             from.path()
//                 .bottom_key(from.key())
//                 .expect("Entry returned key not from it's path."),
//             PhantomData,
//         )
//     }
// }
