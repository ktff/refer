mod collection;
mod entity;
mod item;
mod key;
mod reference;
mod shell;

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
   x. Item i Ostalo su odvojene stvari
      - Ovo je prakticno rjesenje za problem duplog MutEntry jedan posudenog od drugog nad istim itemom gdje unutrasnji
        MutEntry remova item te se vrati na vanjski MutEntry koji ocekuje valjani podatak ali ga nema.
      - Da se ovo fixa a da se omoguci followanje referenci,
   x. Izdvoji parrallelnost
   x. Ukloni Locality
   x. Ukloniti Path
*/

// #[derive(Debug, Clone, PartialEq, Eq, Copy)]
// pub enum Error {
//     /// Operation was not possible because key is not in use for any item
//     ///
//     /// A fatal logical error. Should panic.
//     KeyIsNotInUse(AnyKey),
//     // NOTE: Not loaded is just KeyIsNotInUse, while locked is probably only related to multiple MutShell which
//     //       should impl handle.
//     // /// It's unavailble for some reasons, like it isn't ~~loaded~~ or is locked or something else.
//     // /// But it's temporary.
//     // Unavailable(AnyKey),

//     // NOTE: Should be staticly checked by impl.
//     // /// Operation was not possible because item type is not supported
//     // ///
//     // /// A fatal logical error. Should panic.
//     // // UnsupportedType(TypeId),

//     // NOTE: Impl should panic or return KeyIsNotInUse.
//     // /// Key is not local.
//     // ///
//     // /// A fatal logical error. Should panic.
//     // NotLocalKey(AnyKey),
// }

// impl Error {
//     pub fn recoverable(self) -> bool {
//         match self {
//             Error::KeyIsNotInUse(_) => false,
//             Error::ItemIsReferenced(_) => true,
//             Error::ItemNotInMemory(_) => true,
//             Error::Locked => true,
//             Error::OutOfKeys => false,
//             Error::NotLocalKey(_) => false,
//         }
//     }

//     pub fn unrecoverable(self) -> bool {
//         !self.recoverable()
//     }
// }

// ************************ Convenient methods *************************** //

impl<T: ?Sized + 'static> Ref<T> {
    /// Some if to shells exist, otherwise None.
    pub fn connect<F: ?Sized + 'static>(
        from: Key<F>,
        to: Key<T>,
        collection: &mut impl ShellCollection,
    ) -> Option<Self> {
        let mut to_shell = collection.get_mut(to)?;
        to_shell.add_from(Ref::<F>::new(from).into());
        Some(Self::new(to))
    }

    /// True if there was reference to remove.
    pub fn disconnect<F: ?Sized + 'static>(
        self,
        from: Key<F>,
        collection: &mut impl ShellCollection,
    ) -> bool {
        if let Some(mut to_shell) = collection.get_mut(self.key()) {
            to_shell.remove_from(Ref::<F>::new(from).into())
        } else {
            false
        }
    }
}

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
