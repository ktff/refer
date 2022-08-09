use super::{AnyKey, CollectionRef, Error, Key, RefEntry};

/// Polly collection should implement this for multiple types.
pub trait CollectionMut<T: ?Sized + 'static>: CollectionRef<T> {
    type ME<'a>: MutEntry<'a, T = T>
    where
        Self: 'a;

    // type IterMut<'a>: Iterator<Item = Self::ME<'a>> + 'a
    // where
    //     Self: 'a;

    // Implementations should specialize this for Composite items.
    /// Will error if the key is not in use.
    /// Returns previous item.
    fn set(&mut self, key: Key<T>, item: T) -> Result<T, Error>
    where
        T: Sized;

    // Implementations should specialize this for Composite items.
    /// Will error if the key is not in use.
    fn set_clone(&mut self, key: Key<T>, item: &T) -> Result<(), Error>
    where
        T: Clone;

    /// Will error if the key is not in use.
    fn get_mut<'a>(&'a mut self, key: Key<T>) -> Result<Self::ME<'a>, Error>;

    // NOTE: Posto se from mora mijenjati ovo se nemoze sigurno izvesti.
    // iz istog razloga se preporuca da kolekcija implementira ovo za sve tipove
    // na jednom structu.
    // fn iter_mut<'a>(&'a mut self) -> Self::IterMut<'a>;

    fn first_key(&self) -> Option<Key<T>>;

    /// Returns following key after given in ascending order.
    fn next_key(&self, key: Key<T>) -> Option<Key<T>>;

    /// Will error if there is any reference to it or key is not in use.
    fn remove(&mut self, key: Key<T>) -> Result<bool, Error>;

    /// Will error if there is any reference to it or key is not in use.
    fn take(&mut self, key: Key<T>) -> Result<Option<T>, Error>
    where
        T: Sized;
}

pub trait MutEntry<'a>: RefEntry<'a> {
    fn item_mut(&mut self) -> &mut Self::T;

    /// T as composite type now has one reference.
    fn add_reference(&mut self, key: AnyKey);

    /// T as composite type now doesn't have one reference.
    fn remove_reference(&mut self, key: AnyKey);
}
