use crate::field::ReadField;
use std::{marker::PhantomData, ops::Deref};

pub trait Storage<S: Structure + ?Sized> {
    // ! Copy, <'a>
    type K: Copy;
    //TODO: Deref
    type C<'a>: 'a + Deref<Target = S::Data<Self>>
    where
        Self: 'a;

    fn read<'a>(&'a self, key: Self::K) -> Self::C<'a>;
}

// impl<S: Structure, Store: Storage<S>> Storage<Option<S>> for Store {
//     type K<'a> = Option<<Store as Storage<S>>::K<'a>>;

//     fn read<'a>(&'a self, key: Self::K<'a>) -> Option<S::Container<'a>> {
//         key.map(|k| self.read(k))
//     }
// }

pub trait Structure: 'static {
    // type Container<'a>;
    type Fields;
    // ! <'a>
    type Data<S: Storage<Self> + ?Sized>: ?Sized;

    fn fields<S: Storage<Self> + ?Sized>(store: &Self::Data<S>) -> Self::Fields;
}

// impl<S: Structure> Structure for Option<S> {
//     type Container<'a> = Option<S::Container<'a>>;
//     type Fields = Option<S::Fields>;

//     fn fields<'a>(store: &Self::Container<'a>) -> Self::Fields {
//         store.as_ref().map(S::fields)
//     }
// }

pub struct ReadStructure<'a, S: Structure, Store: Storage<S>> {
    store: &'a Store,
    data: Store::C<'a>,
    fields: S::Fields,
}

impl<'a, S: Structure, Store: Storage<S>> ReadStructure<'a, S, Store> {
    #[doc(hidden)]
    pub fn new(store: &'a Store, key: Store::K) -> Self {
        let data = store.read(key);
        let fields = S::fields(&*data);

        ReadStructure {
            store,
            data,
            fields,
        }
    }

    #[doc(hidden)]
    pub fn read_data<'b, F: ReadField<S::Data<Store>>>(
        &'b self,
        field: impl FnOnce(&S::Fields) -> F,
    ) -> F::To<'b> {
        field(&self.fields).read(&*self.data)
    }

    #[doc(hidden)]
    pub fn read_store<'b, T: Structure, F: ReadField<S::Data<Store>>>(
        &'b self,
        field: impl FnOnce(&S::Fields) -> F,
    ) -> ReadStructure<'a, T, Store>
    where
        Store: Storage<T>,
        F::To<'b>: Into<<Store as Storage<T>>::K>,
    {
        ReadStructure::new(self.store, self.read_data(field).into())
    }

    #[doc(hidden)]
    pub fn read_store_optional<'b, T: Structure, F: ReadField<S::Data<Store>>>(
        &'b self,
        field: impl FnOnce(&S::Fields) -> F,
    ) -> Option<ReadStructure<'a, T, Store>>
    where
        Store: Storage<T>,
        F::To<'b>: Into<Option<<Store as Storage<T>>::K>>,
    {
        Some(ReadStructure::new(
            self.store,
            self.read_data(field).into()?,
        ))
    }
}

// ****************************** PLAIN

pub struct PlainStorage<S: Structure>
where
    S::Data<Self>: Sized,
{
    data: Vec<S::Data<Self>>,
}

// pub enum PlainContainer<'a, T> {
//     Inlined(Box<T>),
//     Ref(&'a T),
// }

// impl<'a, T> PlainContainer<'a, T> {
//     pub fn to_ref<'b>(&'b self) -> PlainContainer<'b, T> {
//         match self {
//             PlainContainer::Inlined(b) => PlainContainer::Ref(b),
//             PlainContainer::Ref(r) => PlainContainer::Ref(r),
//         }
//     }
// }

// impl<'a, T> AsRef<T> for PlainContainer<'a, T> {
//     fn as_ref(&self) -> &T {
//         match self {
//             PlainContainer::Inlined(b) => b.as_ref(),
//             PlainContainer::Ref(r) => r,
//         }
//     }
// }

impl<S: Structure> Storage<S> for PlainStorage<S>
where
    S::Data<Self>: Sized,
{
    type K = usize;
    type C<'a> = &'a S::Data<Self>;

    fn read<'a>(&'a self, key: Self::K) -> Self::C<'a> {
        &self.data[key]
    }
}

// *************************** RAW

pub struct RawStorage<S: Structure> {
    data: Vec<Box<S::Data<Self>>>,
}

// pub struct RawStorage<T>(PhantomData<T>);

// pub struct Raw<T>(PhantomData<T>, [u8]);

// pub enum RawContainer<'a, T> {
//     Inlined(Box<Raw<T>>),
//     Ref(&'a Raw<T>),
// }

// impl<'a, T> RawContainer<'a, T> {
//     pub fn get(&self) -> &Raw<T> {
//         match self {
//             RawContainer::Inlined(b) => b.as_ref(),
//             RawContainer::Ref(r) => r,
//         }
//     }

//     pub fn as_ref<'b>(&'b self) -> RawContainer<'b, T> {
//         match self {
//             RawContainer::Inlined(b) => RawContainer::Ref(b),
//             RawContainer::Ref(r) => RawContainer::Ref(r),
//         }
//     }
// }

impl<S: Structure> Storage<S> for RawStorage<S> {
    type K = usize;
    type C<'a> = &'a S::Data<Self>;

    fn read<'a>(&'a self, key: Self::K) -> Self::C<'a> {
        &self.data[key]
    }
}
