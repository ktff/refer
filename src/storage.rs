use crate::field::ReadField;
use std::{marker::PhantomData, ops::Deref};

// ! node extensions. Ownership i Option to omogucavaju.
/*
* Ovo sve je više pristup programiranju graph problema nego biblioteka.

* Data je spremljen:
    * Plain kao T
    * Serijaliziran kao [u8]

* Each storage is also any map - extension
* Storage grained lock - extension


* Hierarchy of storages

* Ako postoji samo jedna ili dvije dobre implementacije, mogu se direktno koristiti bez apstrakcija.

* Konkretni storage:
    * Plain
    * Raw - tu postoji vise verzija radi backcompatibility

* Bitno je da se reference mogu pokzivati medu razlicitim storage-ovima.

* Svaka struktura se specijalizira za jedan od konkretnih storage.

* Kompozicija je glavni način ekstenzije.

* Nisu dozvoljeni ciklusi u usmjerenom graphu referenci. Reference koje održavaju storagi se neracunaju, from & owner.
* Omogucuje mutaciju svega ako se ima ogranici na samo jedan root, ili na mutaciju owner stabla ako se ogranici samo na
* rotove koji su vlasnistvo storage-a.

* Rooted owner trees are primary mode of mutation

* Storages should be independent, like they can be mutated at the same time.

* set(key: K, value: T) funckija je dobar primjer za kompoziciju i da nemora Storage ili neki drugi trait ovdje pokriti
* svaku mogucu opciju vec da treba omoguciti siri spektar opcija.

*/

// TODO: Mozda da read vraca Option te da je svaki key koji je storage ikad izdao je valjan, cak i ako je node removan s njega.

pub trait Storage<T: 'static> {
    // ! FUNDAMENTAL, key is the reference, for value to be accessible a reference needs to exist
    type K: Key;
    // ! FUNDAMENTAL, to get value from storage and for it to provide metadata a storage selected struct is needed
    type C<'a>: Container<'a, T = T, K = Self::K>
    where
        Self: 'a;

    // ! FUNDAMENTAL, value must be accessible
    fn get<'a>(&'a self, key: Self::K) -> Option<ReadStructure<'a, T, Self>>;

    // //? Note: Bilo bi dobro omoguciti da ide i izvan owned stabla s ref.
    // //? Mozda bi se dalo ako bi se onemogucilo write dok se ima takav ref.

    // ?NOTE: Write da ima pristup kontekstu da moze odmah mijenjati
    // ?      reference.

    // // Panics if owner is not storage
    // // TODO: Mut
    // fn write<'a>(&'a mut self, key: Self::K) -> Self::C<'a> {
    //     unimplemented!()
    // }

    // /// May panic if key is not owned by this owner.
    // // TODO: Mut
    // fn write_owned<'a>(&'a mut self, key: Self::K, owner: Self::K) -> Self::C<'a> {
    //     unimplemented!()
    // }
    // ! FUNDAMENTAL, enables mutation and consequently building T from bottom down.
    fn get_mut<'a>(
        &'a mut self,
        key: Self::K,
        owner: Option<Self::K>,
    ) -> Option<WriteStructure<'a, T, Self>>;

    // ! FUNDAMENTAL, Those T that require Drop need to have Storage that knows where are they hence it knows to iterate.
    // ! OPTIONAL, For !Drop T
    // ? Replaces iter and iter_mut with an iterator over the keys. For mut this is an issue since consuming iterator requires & and consuming K requires &mut.
    // ?NOTE: Ovo pokazuje ovisnost seta kljuceva o mutaciji. Odnosno set kljuceva se moze promijeniti mutacijom. Stoga je ovaj problem legitiman te ce biti potrebna
    // ?      neka extra logika da se ovo handla.
    /// Iters over storage owned.
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = Self::K> + 'a>;

    // /// Iters over storage owned.
    // fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (Self::K, ReadStructure<'a, T, Self>)> + 'a>;

    // // TODO: Should be callable only once. An outer trait? Self::Partial
    // fn iter_mut<'a>(
    //     &'a mut self,
    // ) -> Box<dyn Iterator<Item = (Self::K, WriteStructure<'a, T, Self::Partial>)> + 'a>;

    // /// Iterates over storage owned.
    // fn iter_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = (Self::K, Self::C<'a>)> + 'a>;

    // TODO: Add i remove su za neke T opcionalne. Poput prostora kojem se pristupa s K.
    // TODO: Takva implementacija ili treba imati generator K->T u pozadini, ili treba omoguciti add(key: K,value: T)
    //
    // TODO: set(key: K, value: T), tada ovisi o implementaciji tko odreduje K. Ako nesto poput prostora, nema extra funkcija,
    // TODO:                       a ako je poput alokacije tada implementacija moze imati allocate/add funckiju. Preko toga se moze a i nemora apstrahirati,
    // TODO:                       to ovisi koliko ima smisla za specificnu implementaciju.

    // ! FUNDAMENTAL, for there to be value to access it must be set first
    // ? There are two subclases of storages for this:
    // ? * Allocate - kljuc nenosi podatak, moze realocirati ya potrebe memorije i optimizacije grupiranja
    // ? * Space - kljuc nosi podatak pa ga se nemože mijenjati, lokalnost kljuceva u njihovoj domeni se koristi za grupiranje
    // ? Subclases can add more constricted methods to there own traits or own impl. Ex. `fn set(&mut self, key: Self::K, value: T, owner: Option<Self::K>) -> Option<T>`
    // ? A specific impl can support both subclases.
    /// Returns previous value, and optionaly a different key for the new value
    /// May panic if this is taking ownership from someone else than storage or self.
    #[must_use]
    fn set(
        &mut self,
        key: impl Into<Option<Self::K>>,
        value: T,
        owner: Option<Self::K>,
    ) -> (Option<Self::K>, Option<T>)
    // TODO: Specialize impls if this is true
    where
        T: Instance<Self::K>;

    // ! FUNDAMENTAL, enables building T from bottom up.
    /// Returns Some if relocated value
    /// from
    /// None -> Panics if not owned by this storage.
    /// Some -> Panic if key is not owned by this owner
    ///      -> Owner must remove from self key.
    ///
    /// Panics if key not present.
    #[must_use]
    fn transfer(
        &mut self,
        key: Self::K,
        from: Option<Self::K>,
        to: Option<Self::K>,
    ) -> Option<Self::K>
    // TODO: Specialize impls if this is true
    where
        T: Instance<Self::K>;

    // ! FUNDAMENTAL, since  T can not exist once it can not exist multiple times
    /// None -> Panics if not owned by this storage.
    /// Some -> May panic if key is not owned by this owner
    ///      -> Owner must remove from self key.
    fn remove(&mut self, key: Self::K, owner: Option<Self::K>) -> Option<T>
    // TODO: Specialize impls if this is true
    where
        T: Instance<Self::K>;

    // ! CONVINENCE
    // /// As remove but removes only children without refs. Those that have are given to the storage.
    // fn remove_exclusive(&mut self, key: Self::K, owner: Option<Self::K>)
    // // TODO: Specialize impls if this is true
    // where
    //     T: Instance<Self::K>;
}

/// Meant to be used by a larger storage.
pub trait MinorStore<K: Key>: Keyed {
    type C<'a>: Container<'a, T = Self::T, K = K>
    where
        Self: 'a;

    ///! May panic:
    /// * if key is not valid.
    fn get_node<'a>(&'a self, key: Self::K) -> Self::C<'a>;

    fn iter_node<'a>(&'a self) -> Box<dyn Iterator<Item = (Self::K, Self::C<'a>)> + 'a>;

    fn add_node(&mut self, data: Self::T) -> Self::K;

    ///! May panic:
    /// * if key is not valid.
    fn set_node(&mut self, key: Self::K, node: Self::T);

    /// Key stops being valid after this.
    /// Returns data and edges to it.
    ///
    ///! May panic:
    /// * if key is not valid.
    fn remove_node(&mut self, key: Self::K) -> Option<(Option<Owner<K>>, Vec<K>, Self::T)>;

    ///! May panic:
    /// * if is taking ownership of something already owned.
    /// * if to is not valid.
    fn add_edge(&mut self, from: K, relation: Relation, to: Self::K);

    ///! May panic:
    /// * if from owns to
    /// * if from is not valid.
    fn remove_ref(&mut self, from: Self::K, to: K)
    where
        Self::T: Instance<K>;

    /// True if to was owned by from.
    ///
    /// ! May panic:
    /// * if is owners don't match
    /// * if to is not valid.
    /// * edge did not exist.
    fn remove_edge(&mut self, from: K, relation: Relation, to: Self::K) -> bool;
}

pub trait Keyed {
    type K: Key;

    type T;
}

pub trait Container<'a>: Keyed + 'a {
    fn data(&self) -> &Self::T;

    fn owner(&self) -> Option<Owner<Self::K>>;

    fn from(&self) -> Box<dyn Iterator<Item = Self::K> + 'a>;
}

// TODO: KeyFamily{K<F:Family>}

// TODO: struct Clan, trait Member<C,F>{const N:usize}

pub trait Key: Eq + std::fmt::Debug + Copy + 'static {}

impl Key for usize {}

pub enum Relation {
    Owns, //{ anonymous: bool },
    Ref,
}
pub trait Instance<K: Key>: 'static {
    /// Calls for each ref
    fn for_each_ref(&self, call: impl FnMut(K));

    /// Calls for each owned
    fn for_each_owned(&self, call: impl FnMut(K));

    /// Will be returned as many times as for_each_ref returns it.
    fn remove_ref(&mut self, key: impl Iterator<Item = K>);
}

pub struct ReadStructure<'a, T: 'static, Store: Storage<T> + ?Sized> {
    store: &'a Store,
    data: Store::C<'a>,
}

impl<'a, T: 'static, Store: Storage<T> + ?Sized> ReadStructure<'a, T, Store> {
    fn new_key(store: &'a Store, key: Store::K) -> Self {
        store.get(key)
    }

    fn new_data(store: &'a Store, data: Store::C<'a>) -> Self {
        ReadStructure { store, data }
    }

    /// Expects that the data is present.
    pub fn read(&self, key: Store::K) -> Self {
        Self::new_key(self.store, key)
    }

    pub fn owner(&self) -> Option<Owner<Store::K>> {
        self.data.owner()
    }

    pub fn from(&self) -> impl Iterator<Item = Store::K> + 'a {
        self.data.from()
    }
}

impl<'a, T: 'static, Store: Storage<T> + ?Sized> Deref for ReadStructure<'a, T, Store> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.data.data()
    }
}

pub struct ReadStructure<'a, T: 'static, Store: Storage<T> + ?Sized> {
    store: &'a Store,
    data: Store::C<'a>,
}

impl<'a, T: 'static, Store: Storage<T> + ?Sized> ReadStructure<'a, T, Store> {
    fn new_key(store: &'a Store, key: Store::K) -> Self {
        store.get(key)
    }

    fn new_data(store: &'a Store, data: Store::C<'a>) -> Self {
        ReadStructure { store, data }
    }

    /// Expects that the data is present.
    pub fn read(&self, key: Store::K) -> Self {
        Self::new_key(self.store, key)
    }

    pub fn owner(&self) -> Option<Owner<Store::K>> {
        self.data.owner()
    }

    pub fn from(&self) -> impl Iterator<Item = Store::K> + 'a {
        self.data.from()
    }
}

impl<'a, T: 'static, Store: Storage<T> + ?Sized> Deref for ReadStructure<'a, T, Store> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.data.data()
    }
}

pub struct WriteStructure<'a, T: 'static, Store: Storage<T> + ?Sized> {
    // store: &'a mut Store,
    // data: Store::C<'a>,
}

impl<'a, T: 'static, Store: Storage<T> + ?Sized> WriteStructure<'a, T, Store> {
    // fn new_key(store: &'a Store, key: Store::K) -> Self {
    //     store.get(key)
    // }

    // fn new_data(store: &'a Store, data: Store::C<'a>) -> Self {
    //     ReadStructure { store, data }
    // }

    pub fn get<'b>(&'b self, key: Store::K) -> ReadStructure<'b, T, Store> {
        unimplemented!()
    }

    pub fn get_mut<'b>(&'b mut self, key: Store::K) -> Self {
        unimplemented!()
    }

    pub fn register_ref(&mut self, key: Store::K) {
        unimplemented!()
    }

    pub fn unregister_ref(&mut self, key: Store::K) {
        unimplemented!()
    }

    pub fn owner(&self) -> Option<Owner<Store::K>> {
        self.data.owner()
    }

    pub fn from(&self) -> impl Iterator<Item = Store::K> + 'a {
        self.data.from()
    }
}

impl<'a, T: 'static, Store: Storage<T> + ?Sized> Deref for WriteStructure<'a, T, Store> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.data.data()
    }
}

impl<'a, T: 'static, Store: Storage<T> + ?Sized> DerefMut for WriteStructure<'a, T, Store> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.data_mut()
    }
}

// ****************************** PLAIN

pub struct PlainStorage<K: Key, T> {
    data: Vec<Slot<K, T>>,
    _k: PhantomData<K>,
}

impl<K: Key, T: 'static> Keyed for PlainStorage<K, T> {
    type T = T;

    type K = usize;
}

impl<K: Key, T: 'static> MinorStore<K> for PlainStorage<K, T> {
    type C<'a> = &'a Occupied<K, T>;

    fn get_node<'a>(&'a self, key: usize) -> Self::C<'a> {
        match &self.data[key] {
            Slot::Occupied(node) => node,
            Slot::Empty => panic!("Key is invalid"),
        }
    }

    fn iter_node<'a>(&'a self) -> Box<dyn Iterator<Item = (usize, Self::C<'a>)> + 'a> {
        Box::new(
            self.data
                .iter()
                .enumerate()
                .flat_map(|(i, data)| match data {
                    Slot::Occupied(node @ Occupied { owner: None, .. }) => Some((i, node)),
                    Slot::Empty | Slot::Occupied(Occupied { owner: Some(_), .. }) => None,
                }),
        )
    }

    fn add_edge(&mut self, key: K, relation: Relation, to: Self::K) {
        match (relation, &mut self.data[to]) {
            (Relation::Ref, Slot::Occupied(Occupied { from, .. })) => from.push(key),
            (
                Relation::Owns { anonymous: false },
                Slot::Occupied(Occupied {
                    owner: Some(Owner::Known(a)),
                    ..
                }),
            ) if *a == key => (),

            (Relation::Owns { .. }, Slot::Occupied(Occupied { owner: Some(_), .. })) => {
                panic!("Already owned")
            }
            (Relation::Owns { anonymous: false }, Slot::Occupied(Occupied { owner, .. })) => {
                *owner = Some(Owner::Known(key))
            }
            (Relation::Owns { anonymous: true }, Slot::Occupied(Occupied { owner, .. })) => {
                *owner = Some(Owner::Anonymous)
            }
            (_, Slot::Empty) => panic!("Key is invalid"),
        }
    }

    fn remove_ref(&mut self, from: Self::K, remove: K)
    where
        T: Instance<K>,
    {
        match &mut self.data[from] {
            Slot::Occupied(Occupied { data, .. }) => {
                data.remove_ref(remove);
            }
            Slot::Empty => (),
        }
    }

    fn remove_edge(&mut self, remove: K, relation: Relation, to: Self::K) -> bool {
        match (relation, &mut self.data[to]) {
            (Relation::Ref, Slot::Occupied(Occupied { from, .. })) => {
                let (i, _) = from
                    .iter()
                    .enumerate()
                    .find(|(_, &k)| k == remove)
                    .expect("Key is invalid");
                from.remove(i);
                false
            }
            (Relation::Owns { anonymous }, Slot::Occupied(Occupied { owner, .. })) => {
                match (anonymous, owner.take()) {
                    // Ok
                    (false, Some(Owner::Known(a))) if a == remove => (),
                    // Ok
                    (true, Some(Owner::Anonymous)) => (),
                    // Error
                    _ => panic!("Own relation is invalid"),
                }
                true
            }
            (Relation::Ref, Slot::Empty) => false,
            (Relation::Owns { .. }, Slot::Empty) => {
                panic!("Own relation is invalid")
            }
        }
    }

    fn add_node(&mut self, data: T) -> Self::K {
        let n = self.data.len();

        self.data.push(Slot::Occupied(Occupied {
            data,
            from: Vec::new(),
            owner: None,
        }));

        n
    }

    fn set_node(&mut self, key: Self::K, data: T) {
        match &mut self.data[key] {
            Slot::Occupied(Occupied { data: old, .. }) => {
                *old = data;
            }
            Slot::Empty => panic!("Illegal key"),
        }
    }

    fn remove_node(&mut self, key: Self::K) -> Option<(Option<Owner<K>>, Vec<K>, Self::T)> {
        match std::mem::replace(&mut self.data[key], Slot::Empty) {
            Slot::Occupied(Occupied { from, data, owner }) => Some((owner, from, data)),
            Slot::Empty => None,
        }
    }
}

// *************************** [u8]

// TODO
// *************************** Multi Storage ********************** //

// ***************************** Helper *************************** //

enum Slot<K, T> {
    Empty,
    Occupied(Occupied<K, T>),
}

#[derive(Clone, Copy, Debug)]
pub enum Owner<K> {
    Anonymous,
    Known(K),
}

impl<K: Eq> Eq for Owner<K> {}

impl<K: PartialEq> PartialEq for Owner<K> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Owner::Anonymous, _) | (_, Owner::Anonymous) => true,
            (Owner::Known(a), Owner::Known(b)) => a == b,
        }
    }
}

impl<K: PartialEq> PartialEq<K> for Owner<K> {
    fn eq(&self, other: &K) -> bool {
        match self {
            Owner::Anonymous => true,
            Owner::Known(a) => a == other,
        }
    }
}

pub struct Occupied<K, T> {
    owner: Option<Owner<K>>,
    // TODO: Eliminate K if T has it
    from: Vec<K>,
    data: T,
}

impl<'a, K: Key, T> Keyed for &'a Occupied<K, T> {
    type T = T;

    type K = K;
}

impl<'a, K: Key, T> Container<'a> for &'a Occupied<K, T> {
    fn data(&self) -> &Self::T {
        &self.data
    }

    fn owner(&self) -> Option<Owner<Self::K>> {
        self.owner
    }

    fn from(&self) -> Box<dyn Iterator<Item = Self::K> + 'a> {
        Box::new(self.from.iter().copied()) as Box<_>
    }
}

// ********************** BLANKET ************************* //

impl<S: MinorStore<<S as Keyed>::K> + Keyed> Storage<S::T> for S
where
    S::T: 'static,
{
    type K = S::K;
    type C<'a> = S::C<'a> where S:'a;

    fn get<'a>(&'a self, key: Self::K) -> ReadStructure<'a, S::T, Self> {
        ReadStructure::new_data(self, self.get_node(key))
    }

    fn iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = (Self::K, ReadStructure<'a, S::T, Self>)> + 'a> {
        Box::new(
            self.iter_node()
                .map(|(key, node)| (key, ReadStructure::new_data(self, node))),
        )
    }

    fn add(&mut self, data: S::T) -> Self::K
    where
        S::T: Instance<Self::K>,
    {
        // Collect edges
        let mut edges = Vec::new();
        data.for_each_ref(|relation, to| edges.push((relation, to)));

        // Add node
        let key = self.add_node(data);

        // Add from
        edges
            .into_iter()
            .for_each(|(relation, to)| self.add_edge(key, relation, to));

        key
    }

    fn remove(&mut self, remove: Self::K, owned: Option<Self::K>)
    where
        S::T: Instance<Self::K>,
    {
        if let Some((owner, from, data)) = self.remove_node(remove) {
            assert_eq!(
                owner,
                owned.map(Owner::Known),
                "Key is owned by something else"
            );

            // Remove from
            from.into_iter()
                .filter(|key| owned.map(|own| own != *key).unwrap_or(true))
                .for_each(|key| self.remove_ref(key, remove));

            // Remove to
            data.for_each_ref(|relation, key| {
                if self.remove_edge(remove, relation, key) {
                    self.remove(key, Some(remove));
                }
            });
        }
    }
}
