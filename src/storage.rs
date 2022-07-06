use crate::field::ReadField;
use std::{marker::PhantomData, ops::Deref};

pub trait Storage<S: Structure + ?Sized> {
    // ! Copy, <'a>
    type K: Copy;
    type C<'a>: 'a + Deref<Target = S::Data<Self::K>>
    where
        Self: 'a;

    fn read<'a>(&'a self, key: Self::K) -> Self::C<'a>;

    fn iter_read<'a>(&'a self) -> Box<dyn Iterator<Item = (Self::K, Self::C<'a>)> + 'a>;

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = ReadStructure<'a, S, Self>> + 'a> {
        Box::new(
            self.iter_read()
                .map(|(_, c)| ReadStructure::new_data(self, c)),
        )
    }
}

pub trait Structure: 'static {
    type Fields;
    // ! <'a>
    type Data<K: Copy>: ?Sized;

    fn fields<S: Storage<Self> + ?Sized>(store: &Self::Data<S::K>) -> Self::Fields;
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
    pub fn read_data<'b, F: ReadField<S::Data<Store::K>>>(
        &'b self,
        field: impl FnOnce(&S::Fields) -> F,
    ) -> F::To<'b> {
        field(&self.fields).read(&*self.data)
    }

    #[doc(hidden)]
    pub fn read_store<'b, T: Structure, F: ReadField<S::Data<<Store as Storage<S>>::K>>>(
        &'b self,
        field: impl FnOnce(&S::Fields) -> F,
    ) -> ReadStructure<'a, T, Store>
    where
        Store: Storage<T>,
        F::To<'b>: Into<<Store as Storage<T>>::K>,
    {
        ReadStructure::new_key(self.store, self.read_data(field).into())
    }

    #[doc(hidden)]
    pub fn read_store_optional<'b, T: Structure, F: ReadField<S::Data<<Store as Storage<S>>::K>>>(
        &'b self,
        field: impl FnOnce(&S::Fields) -> F,
    ) -> Option<ReadStructure<'a, T, Store>>
    where
        Store: Storage<T>,
        F::To<'b>: Into<Option<<Store as Storage<T>>::K>>,
    {
        Some(ReadStructure::new_key(
            self.store,
            self.read_data(field).into()?,
        ))
    }
}

// ****************************** PLAIN

pub struct PlainStorage<S: Structure>
where
    S::Data<usize>: Sized,
{
    data: Vec<S::Data<usize>>,
}

impl<S: Structure> Storage<S> for PlainStorage<S>
where
    S::Data<usize>: Sized,
{
    type K = usize;
    type C<'a> = &'a S::Data<usize>;

    fn read<'a>(&'a self, key: usize) -> Self::C<'a> {
        &self.data[key]
    }

    fn iter_read<'a>(&'a self) -> Box<dyn Iterator<Item = (usize, &'a S::Data<usize>)> + 'a> {
        Box::new(self.data.iter().enumerate())
    }
}

// *************************** RAW

pub struct RawStorage<S: Structure> {
    data: Vec<Box<S::Data<usize>>>,
}

impl<S: Structure> Storage<S> for RawStorage<S> {
    type K = usize;
    type C<'a> = &'a S::Data<usize>;

    fn read<'a>(&'a self, key: usize) -> Self::C<'a> {
        &self.data[key]
    }

    fn iter_read<'a>(&'a self) -> Box<dyn Iterator<Item = (usize, &'a S::Data<usize>)> + 'a> {
        Box::new(self.data.iter().map(|c| c.as_ref()).enumerate())
    }
}
