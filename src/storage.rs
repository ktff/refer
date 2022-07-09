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

*/

pub trait Storage<F: Family> {
    type K: Key;
    type C<'a>: Container<'a, T = F::I<Self::K>, K = Self::K>
    where
        Self: 'a;

    /// Expects that the data is present.
    fn read<'a>(&'a self, key: Self::K) -> Self::C<'a>;

    /// Iters over storage owned.
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (Self::K, Self::C<'a>)> + 'a>;

    /// May panic if adding taking ownership of something already owned.
    fn add(&mut self, data: F::I<Self::K>) -> Self::K;

    /// Panics if not owned by this storage.
    fn remove(&mut self, key: Self::K);

    /// Owner must remove from self key.
    /// May panic if key is not owned by this owner.
    fn remove_owned(&mut self, key: Self::K, owner: Self::K);

    // //? Note: Bilo bi dobro omoguciti da ide i izvan owned stabla s ref.
    // //? Mozda bi se dalo ako bi se onemogucilo write dok se ima takav ref.

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
}

pub trait Container<'a>: 'a {
    type K;
    type T: ?Sized;

    fn data(&self) -> &Self::T;

    fn owner(&self) -> Option<Self::K>;

    fn from(&self) -> Box<dyn Iterator<Item = Self::K> + 'a>;
}

pub trait Key: Copy + 'static {}

impl Key for usize {}

pub enum Relation {
    Owns,
    Ref,
}
pub trait Family: 'static {
    type I<K: Key>: Instance<K>;
}

pub trait Instance<K: Key>: 'static {
    fn iter(&self, call: impl FnMut(Relation, K));

    /// Must not be called for owners of key.
    /// Will be called as many times as iter returns it.
    fn remove_ref(&mut self, key: K) -> bool;
}

pub struct ReadStructure<'a, F: Family, Store: Storage<F> + ?Sized> {
    store: &'a Store,
    data: Store::C<'a>,
}

impl<'a, F: Family, Store: Storage<F> + ?Sized> ReadStructure<'a, F, Store> {
    fn new_key(store: &'a Store, key: Store::K) -> Self {
        Self::new_data(store, store.read(key))
    }

    fn new_data(store: &'a Store, data: Store::C<'a>) -> Self {
        ReadStructure { store, data }
    }

    /// Expects that the data is present.
    pub fn read(&self, key: Store::K) -> Self {
        Self::new_key(self.store, key)
    }

    pub fn owner(&self) -> Option<Store::K> {
        self.data.owner()
    }

    pub fn from(&self) -> impl Iterator<Item = Store::K> + 'a {
        self.data.from()
    }
}

impl<'a, F: Family, Store: Storage<F>> Deref for ReadStructure<'a, F, Store> {
    type Target = F::I<Store::K>;
    fn deref(&self) -> &Self::Target {
        self.data.data()
    }
}

// ****************************** PLAIN

pub struct PlainStorage<F: Family> {
    data: Vec<Slot<usize, F::I<usize>>>,
}

impl<F: Family> PlainStorage<F> {
    fn remove_slot(&mut self, remove: usize, owned: Option<usize>) {
        match std::mem::replace(&mut self.data[remove], Slot::Empty) {
            Slot::Occupied(Occupied { from, data, owner }) if owner == owned => {
                // Remove from
                from.into_iter()
                    .filter(|key| owned.map(|own| own != *key).unwrap_or(true))
                    .for_each(|key| match &mut self.data[key] {
                        Slot::Occupied(Occupied { data, .. }) => {
                            data.remove_ref(remove);
                        }
                        Slot::Empty => (),
                    });

                // Remove to
                data.iter(|relation, key| match (relation, &mut self.data[key]) {
                    (Relation::Ref, Slot::Occupied(Occupied { from, .. })) => {
                        let (i, _) = from
                            .iter()
                            .enumerate()
                            .find(|(_, &k)| k == remove)
                            .expect("Key is invalid");
                        from.remove(i);
                    }
                    (
                        Relation::Owns,
                        Slot::Occupied(Occupied {
                            owner: Some(owner), ..
                        }),
                    ) if *owner == remove => self.remove_slot(key, Some(remove)),
                    (Relation::Owns, Slot::Occupied(Occupied { .. })) => {
                        panic!("Own relation is invalid")
                    }
                    (Relation::Ref, Slot::Empty) => (),
                    (Relation::Owns, Slot::Empty) => panic!("Own relation is invalid"),
                });
            }
            Slot::Occupied(_) => panic!("Key is owned by something else"),
            Slot::Empty => (),
        }
    }
}

impl<F: Family> Storage<F> for PlainStorage<F> {
    type K = usize;
    type C<'a> = &'a Occupied<Self::K, F::I<usize>>;

    fn read<'a>(&'a self, key: usize) -> Self::C<'a> {
        match &self.data[key] {
            Slot::Occupied(node) => node,
            Slot::Empty => panic!("Key is invalid"),
        }
    }

    fn iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = (usize, &'a Occupied<Self::K, F::I<usize>>)> + 'a> {
        Box::new(
            self.data
                .iter()
                .flat_map(|data| match data {
                    Slot::Occupied(node @ Occupied { owner: None, .. }) => Some(node),
                    Slot::Empty | Slot::Occupied(Occupied { owner: Some(_), .. }) => None,
                })
                .enumerate(),
        )
    }

    fn add(&mut self, data: F::I<usize>) -> Self::K {
        // Allocate slot
        let n = self.data.len();

        // Add from
        data.iter(|relation, key| match (relation, &mut self.data[key]) {
            (Relation::Ref, Slot::Occupied(Occupied { from, .. })) => from.push(n),
            (Relation::Owns, Slot::Occupied(Occupied { owner: Some(a), .. })) if *a == n => (),
            (Relation::Owns, Slot::Occupied(Occupied { owner: Some(_), .. })) => {
                panic!("Already owned")
            }
            (Relation::Owns, Slot::Occupied(Occupied { owner, .. })) => *owner = Some(n),
            (_, Slot::Empty) => panic!("Key is invalid"),
        });

        // Add to slot
        self.data.push(Slot::Occupied(Occupied {
            data: data.into(),
            from: Vec::new(),
            owner: None,
        }));

        n
    }

    fn remove(&mut self, remove: Self::K) {
        self.remove_slot(remove, None);
    }

    fn remove_owned(&mut self, remove: Self::K, owner: Self::K) {
        self.remove_slot(remove, Some(owner));
    }
}

// *************************** Boxed

// TODO
// *************************** Multi Storage ********************** //

// ***************************** Helper *************************** //

enum Slot<K, T> {
    Empty,
    Occupied(Occupied<K, T>),
}

pub struct Occupied<K, T> {
    owner: Option<K>,
    // TODO: Eliminate K if T has it
    from: Vec<K>,
    data: T,
}

impl<'a, K: Key, T> Container<'a> for &'a Occupied<K, T> {
    type K = K;
    type T = T;

    fn data(&self) -> &Self::T {
        &self.data
    }

    fn owner(&self) -> Option<Self::K> {
        self.owner
    }

    fn from(&self) -> Box<dyn Iterator<Item = Self::K> + 'a> {
        Box::new(self.from.iter().copied()) as Box<_>
    }
}
