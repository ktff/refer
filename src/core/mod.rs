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

// ************************ Convenient methods *************************** //

impl<T: ?Sized + 'static> Ref<T> {
    /// Some if to shells exist, otherwise None.
    pub fn connect(
        from: AnyKey,
        to: Key<T>,
        collection: &mut impl ShellCollection,
    ) -> Option<Self> {
        let mut to_shell = collection.get_mut(to)?;
        to_shell.add_from(from);
        Some(Self::new(to))
    }

    /// True if there was reference to remove.
    pub fn disconnect(self, from: AnyKey, collection: &mut impl ShellCollection) -> bool {
        if let Some(mut to_shell) = collection.get_mut(self.key()) {
            to_shell.remove_from(from)
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

//     /// Returns Ref referencing from.
//     fn from<'a, P: PathRef<'a>, M: RefItemEntry<'a, P>>(self, from: &M) -> Ref<M::T, Global, D>
//     where
//         P::Top: Collection<T> + Collection<M::T>,
//     {
//         Ref::<M::T, Global, D>(from.key(), PhantomData)
//     }
// }
