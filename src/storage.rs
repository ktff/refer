use crate::field::ReadField;
use std::{marker::PhantomData, ops::Deref};

pub trait Storage<S: Structure + ?Sized> {
    // ! Copy, not <'a>
    type K: Copy;
    type C<'a>: Container<'a, T = S::Data<Self::K>, K = Self::K>
    where
        Self: 'a;

    /// Expects that the data is present.
    fn read<'a>(&'a self, key: Self::K) -> Self::C<'a>;

    /// Iters over storage owned.
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

    /// Owner must remove from self key.
    /// May panic if key is not owned by this owner.
    fn remove_owned(&mut self, key: Self::K, owner: Self::K);
}

pub trait Container<'a>: 'a {
    type K;
    type T: ?Sized;

    fn data(&self) -> &Self::T;
}

pub trait Structure: 'static {
    type T<K: Copy>: KeyStore<K>;

    // ! not <'a>
    /// Form of T when stored
    type Data<K: Copy>: KeyStore<K> + ?Sized;

    /// Form of T when partially read from Data
    type Cache;

    fn cache<K: Copy>(store: &Self::Data<K>) -> Self::Cache;
}

impl<S: Structure> Structure for Box<S> {
    type T<K: Copy> = S::T<K>;
    type Data<K: Copy> = Box<S::Data<K>>;
    type Cache = S::Cache;

    fn cache<K: Copy>(store: &Self::Data<K>) -> Self::Cache {
        S::cache::<K>(store.as_ref())
    }
}

pub enum Relation {
    Owns,
    Ref,
}
pub trait KeyStore<K: Copy> {
    fn iter(&self, call: impl FnMut(Relation, K));

    /// Must not be called for owners of key.
    /// Will be called as many times as iter returns it.
    fn remove_ref(&mut self, key: K) -> bool;
}

impl<K: Copy, T: KeyStore<K> + ?Sized> KeyStore<K> for Box<T> {
    fn iter(&self, call: impl FnMut(Relation, K)) {
        self.as_ref().iter(call);
    }

    fn remove_ref(&mut self, key: K) -> bool {
        self.as_mut().remove_ref(key)
    }
}

pub struct ReadStructure<'a, S: Structure + ?Sized, Store: Storage<S> + ?Sized> {
    store: &'a Store,
    data: Store::C<'a>,
    cache: S::Cache,
}

impl<'a, S: Structure + ?Sized, Store: Storage<S> + ?Sized> ReadStructure<'a, S, Store> {
    #[doc(hidden)]
    pub fn new_key(store: &'a Store, key: Store::K) -> Self {
        Self::new_data(store, store.read(key))
    }

    #[doc(hidden)]
    pub fn new_data(store: &'a Store, data: Store::C<'a>) -> Self {
        let cache = S::cache::<Store::K>(data.data());

        ReadStructure { store, data, cache }
    }

    // pub fn owner_key(&self) -> Option<Store::K> {
    //     // TODO: Nije nužno da je owner ovog tipa
    //     unimplemented!()
    // }

    // pub fn ref_keys(&self) ->

    #[doc(hidden)]
    pub fn read_data<'b, R: 'b>(
        &'b self,
        field: impl FnOnce(&S::Cache, &'b S::Data<<Store as Storage<S>>::K>) -> R,
    ) -> R {
        field(&self.cache, self.data.data())
    }

    #[doc(hidden)]
    pub fn read_store<'b, T: Structure>(
        &'b self,
        field: impl FnOnce(&S::Cache, &'b S::Data<<Store as Storage<S>>::K>) -> <Store as Storage<T>>::K,
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
            &S::Cache,
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

impl<S: Structure> Storage<S> for PlainStorage<S>
where
    S::T<usize>: Into<S::Data<usize>>,
    S::Data<usize>: Sized,
{
    type K = usize;
    type C<'a> = &'a Occupied<Self::K, S::Data<usize>>;

    fn read<'a>(&'a self, key: usize) -> Self::C<'a> {
        match &self.data[key] {
            Slot::Occupied(node) => node,
            Slot::Empty => panic!("Key is invalid"),
        }
    }

    fn iter_read<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = (usize, &'a Occupied<Self::K, S::Data<usize>>)> + 'a> {
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

    fn add(&mut self, data: S::T<Self::K>) -> Self::K {
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

pub struct BoxStorage<S: Structure> {
    store: PlainStorage<Box<S>>,
}

impl<S: Structure> Storage<S> for BoxStorage<S>
where
    S::T<usize>: Into<Box<S::Data<usize>>>,
{
    type K = <PlainStorage<Box<S>> as Storage<Box<S>>>::K;
    type C<'a> = BoxContainer<'a, <PlainStorage<Box<S>> as Storage<Box<S>>>::C<'a>, S::Data<usize>>;

    fn read<'a>(&'a self, key: Self::K) -> Self::C<'a> {
        self.store.read(key).into()
    }

    fn iter_read<'a>(&'a self) -> Box<dyn Iterator<Item = (usize, Self::C<'a>)> + 'a> {
        Box::new(self.store.iter_read().map(|(key, node)| (key, node.into()))) as Box<_>
    }

    fn add(&mut self, data: S::T<Self::K>) -> Self::K {
        self.store.add(data)
    }

    fn remove(&mut self, key: Self::K) {
        self.store.remove(key);
    }

    fn remove_owned(&mut self, remove: Self::K, owner: Self::K) {
        self.store.remove_owned(remove, owner);
    }
}

pub struct BoxContainer<'a, C, T: ?Sized> {
    container: C,
    marker: PhantomData<&'a T>,
}

impl<'a, C: Container<'a, T = Box<T>>, T: ?Sized> Container<'a> for BoxContainer<'a, C, T> {
    type K = <C as Container<'a>>::K;
    type T = T;

    fn data(&self) -> &Self::T {
        &**self.container.data()
    }
}

impl<'a, C, T: ?Sized> From<C> for BoxContainer<'a, C, T> {
    fn from(container: C) -> Self {
        BoxContainer {
            container,
            marker: PhantomData,
        }
    }
}

// ***************************** Helper *************************** //

enum Slot<K, T> {
    Empty,
    // TODO: Expose owner & from keys
    Occupied(Occupied<K, T>),
}

pub struct Occupied<K, T> {
    owner: Option<K>,
    from: Vec<K>,
    data: T,
}

impl<'a, K, T> Container<'a> for &'a Occupied<K, T> {
    type K = K;
    type T = T;

    fn data(&self) -> &Self::T {
        &self.data
    }
}
