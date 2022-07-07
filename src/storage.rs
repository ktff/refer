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

    fn add(&mut self, data: S::T<Self::K>) -> Self::K;

    /// Panics if not present or if not owned.
    fn remove(&mut self, key: Self::K);
}

pub trait Structure: 'static {
    type T<K: Copy>: KeyStore<K>;

    type Fields;
    // ! not <'a>
    type Data<K: Copy>: KeyStore<K> + ?Sized;

    fn fields<S: Storage<Self> + ?Sized>(store: &Self::Data<S::K>) -> Self::Fields;
}

pub trait KeyStore<K: Copy> {
    fn iter(&self, call: impl FnMut(K));

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
                    Slot::Occupied { data, .. } => Some(data),
                    Slot::Empty => None,
                })
                .enumerate(),
        )
    }

    fn add(&mut self, data: S::T<Self::K>) -> Self::K {
        // Allocate slot
        let n = self.data.len();

        // Add from
        data.iter(|key| match &mut self.data[key] {
            Slot::Occupied { from, .. } => from.push(n),
            Slot::Empty => panic!("Key is invalid"),
        });

        // Add to slot
        self.data.push(Slot::Occupied {
            data: data.into(),
            from: Vec::new(),
        });

        n
    }

    fn remove(&mut self, remove: Self::K) {
        match std::mem::replace(&mut self.data[remove], Slot::Empty) {
            Slot::Occupied { from, data } => {
                // Remove from
                from.into_iter().for_each(|key| match &mut self.data[key] {
                    Slot::Occupied { data, .. } => {
                        data.remove(remove);
                    }
                    Slot::Empty => panic!("Key is invalid"),
                });

                // Remove to
                data.iter(|key| match &mut self.data[key] {
                    Slot::Occupied { from, .. } => {
                        let (i, _) = from
                            .iter()
                            .enumerate()
                            .find(|(_, &k)| k == remove)
                            .expect("Key is invalid");
                        from.remove(i);
                    }
                    Slot::Empty => panic!("Key is invalid"),
                });
            }
            Slot::Empty => panic!("Key is invalid"),
        }
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
    Occupied { from: Vec<K>, data: T },
}
