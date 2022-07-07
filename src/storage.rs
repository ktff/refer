use crate::field::ReadField;
use std::{marker::PhantomData, ops::Deref};

pub trait Storage<S: Structure + ?Sized> {
    // ! Copy, not <'a>
    type K: Copy;
    type C<'a>: 'a + Deref<Target = S::Data<Self::K>>
    where
        Self: 'a;

    /// Expects that the data is present.
    fn read<'a>(&'a self, key: Self::K) -> Self::C<'a>;

    fn iter_read<'a>(&'a self) -> Box<dyn Iterator<Item = (Self::K, Self::C<'a>)> + 'a>;

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = ReadStructure<'a, S, Self>> + 'a> {
        Box::new(
            self.iter_read()
                .map(|(_, c)| ReadStructure::new_data(self, c)),
        )
    }

    /// May panic if adding taking ownership of something already owned.
    fn add(&mut self, data: S::T<Self::K>) -> Self::K;

    /// Panics if not owned by this storage.
    fn remove(&mut self, key: Self::K);
}

pub trait Structure: 'static {
    type T<K: Copy>: KeyStore<K>;

    type Fields;
    // ! not <'a>
    type Data<K: Copy>: KeyStore<K> + ?Sized;

    fn fields<S: Storage<Self> + ?Sized>(store: &Self::Data<S::K>) -> Self::Fields;
}

pub enum Relation {
    Owns,
    Ref,
}
pub trait KeyStore<K: Copy> {
    fn iter(&self, call: impl FnMut(Relation, K));

    /// May panic if this owns the key.
    /// Will be called as many times as iter returns it.
    fn remove(&self, key: K) -> bool;
}

pub struct ReadStructure<'a, S: Structure + ?Sized, Store: Storage<S> + ?Sized> {
    store: &'a Store,
    data: Store::C<'a>,
    fields: S::Fields,
}

impl<'a, S: Structure + ?Sized, Store: Storage<S> + ?Sized> ReadStructure<'a, S, Store> {
    #[doc(hidden)]
    pub fn new_key(store: &'a Store, key: Store::K) -> Self {
        Self::new_data(store, store.read(key))
    }

    #[doc(hidden)]
    pub fn new_data(store: &'a Store, data: Store::C<'a>) -> Self {
        let fields = S::fields::<Store>(&*data);

        ReadStructure {
            store,
            data,
            fields,
        }
    }

    #[doc(hidden)]
    pub fn read_data<'b, R: 'b>(
        &'b self,
        field: impl FnOnce(&S::Fields, &'b S::Data<<Store as Storage<S>>::K>) -> R,
    ) -> R {
        field(&self.fields, &*self.data)
    }

    #[doc(hidden)]
    pub fn read_store<'b, T: Structure>(
        &'b self,
        field: impl FnOnce(
            &S::Fields,
            &'b S::Data<<Store as Storage<S>>::K>,
        ) -> <Store as Storage<T>>::K,
    ) -> ReadStructure<'a, T, Store>
    where
        Store: Storage<T>,
    {
        ReadStructure::new_key(self.store, self.read_data(field))
    }

    #[doc(hidden)]
    pub fn read_store_optional<'b, T: Structure>(
        &'b self,
        field: impl FnOnce(
            &S::Fields,
            &'b S::Data<<Store as Storage<S>>::K>,
        ) -> Option<<Store as Storage<T>>::K>,
    ) -> Option<ReadStructure<'a, T, Store>>
    where
        Store: Storage<T>,
    {
        Some(ReadStructure::new_key(self.store, self.read_data(field)?))
    }
}

// ****************************** PLAIN

pub struct PlainStorage<S: Structure>
where
    S::Data<usize>: Sized,
{
    data: Vec<Slot<usize, S::Data<usize>>>,
}

impl<S: Structure> PlainStorage<S>
where
    S::T<usize>: Into<S::Data<usize>>,
    S::Data<usize>: Sized,
{
    fn remove_slot(&mut self, remove: usize, owned: Option<usize>) {
        match std::mem::replace(&mut self.data[remove], Slot::Empty) {
            Slot::Occupied {
                from,
                data,
                owner: owned,
            } => {
                // Remove from
                from.into_iter().for_each(|key| match &mut self.data[key] {
                    Slot::Occupied { data, .. } => {
                        data.remove(remove);
                    }
                    Slot::Empty => panic!("Key is invalid"),
                });

                // Remove to
                data.iter(|relation, key| match (relation, &mut self.data[key]) {
                    (Relation::Ref, Slot::Occupied { from, .. }) => {
                        let (i, _) = from
                            .iter()
                            .enumerate()
                            .find(|(_, &k)| k == remove)
                            .expect("Key is invalid");
                        from.remove(i);
                    }
                    (
                        Relation::Owns,
                        Slot::Occupied {
                            owner: Some(owner), ..
                        },
                    ) if *owner == remove => self.remove_slot(key, Some(remove)),
                    (Relation::Owns, Slot::Occupied { .. }) => {
                        panic!("Own relation is invalid")
                    }
                    (Relation::Ref, Slot::Empty) => (),
                    (Relation::Owns, Slot::Empty) => panic!("Own relation is invalid"),
                });
            }
            Slot::Occupied { .. } => panic!("Key is owned by something else"),
            Slot::Empty => (),
        }
    }
}

impl<S: Structure> Storage<S> for PlainStorage<S>
where
    S::T<usize>: Into<S::Data<usize>>,
    S::Data<usize>: Sized,
{
    type K = usize;
    type C<'a> = &'a S::Data<usize>;

    fn read<'a>(&'a self, key: usize) -> Self::C<'a> {
        match &self.data[key] {
            Slot::Occupied { data, .. } => data,
            Slot::Empty => panic!("Key is invalid"),
        }
    }

    fn iter_read<'a>(&'a self) -> Box<dyn Iterator<Item = (usize, &'a S::Data<usize>)> + 'a> {
        Box::new(
            self.data
                .iter()
                .flat_map(|data| match data {
                    Slot::Occupied {
                        data, owner: None, ..
                    } => Some(data),
                    Slot::Empty | Slot::Occupied { owner: Some(_), .. } => None,
                })
                .enumerate(),
        )
    }

    fn add(&mut self, data: S::T<Self::K>) -> Self::K {
        // Allocate slot
        let n = self.data.len();

        // Add from
        data.iter(|relation, key| match (relation, &mut self.data[key]) {
            (Relation::Ref, Slot::Occupied { from, .. }) => from.push(n),
            (Relation::Owns, Slot::Occupied { owner: Some(a), .. }) if *a == n => (),
            (Relation::Owns, Slot::Occupied { owner: Some(_), .. }) => {
                panic!("Already owned")
            }
            (Relation::Owns, Slot::Occupied { owner, .. }) => *owner = Some(n),
            (_, Slot::Empty) => panic!("Key is invalid"),
        });

        // Add to slot
        self.data.push(Slot::Occupied {
            data: data.into(),
            from: Vec::new(),
            owner: None,
        });

        n
    }

    fn remove(&mut self, remove: Self::K) {
        self.remove_slot(remove, None);
    }
}

// *************************** RAW

pub struct RawStorage<S: Structure> {
    data: Vec<Option<Box<S::Data<usize>>>>,
}

impl<S: Structure> Storage<S> for RawStorage<S>
where
    S::T<usize>: Into<Box<S::Data<usize>>>,
{
    type K = usize;
    type C<'a> = &'a S::Data<usize>;

    fn read<'a>(&'a self, key: usize) -> Self::C<'a> {
        self.data[key].as_ref().expect("Invalid key")
    }

    fn iter_read<'a>(&'a self) -> Box<dyn Iterator<Item = (usize, &'a S::Data<usize>)> + 'a> {
        Box::new(
            self.data
                .iter()
                .flat_map(|data| data.as_ref())
                .map(|c| c.as_ref())
                .enumerate(),
        )
    }

    fn add(&mut self, data: S::T<Self::K>) -> Self::K {
        let n = self.data.len();
        self.data.push(Some(data.into()));
        n
    }

    fn remove(&mut self, key: Self::K) {
        unimplemented!()
    }
}

// ***************************** Helper *************************** //

enum Slot<K, T> {
    Empty,
    // TODO: Expose owner & from keys
    Occupied {
        owner: Option<K>,
        from: Vec<K>,
        data: T,
    },
}
