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

    pub fn entry<C: Collection>(self, coll: &C) -> C::Ref<'_, T> {
        coll.get(self.key()).expect("Entry isn't present")
    }

    pub fn entry_mut<C: Collection>(self, coll: &mut C) -> C::Mut<'_, T> {
        coll.get_mut(self.key()).expect("Entry isn't present")
    }

    pub fn item<C: ItemCollection>(self, coll: &C) -> &T {
        coll.get(self.key()).expect("Item isn't present")
    }

    pub fn item_mut<C: ItemCollection>(self, coll: &mut C) -> &mut T {
        coll.get_mut(self.key()).expect("Item isn't present")
    }

    pub fn shell<C: ShellCollection>(self, coll: &C) -> C::Ref<'_, T> {
        coll.get(self.key()).expect("Shell isn't present")
    }

    pub fn shell_mut<C: ShellCollection>(self, coll: &mut C) -> C::Mut<'_, T> {
        coll.get_mut(self.key()).expect("Shell isn't present")
    }

    pub fn mut_shell<'a, C: MutShellCollection<'a>>(self, coll: &C) -> C::Mut<T> {
        coll.get_mut(self.key()).expect("Shell isn't present")
    }
}
