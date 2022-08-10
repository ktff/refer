use super::{AnyRef, CollectionRef, Error, Key, RefEntry};

/// Polly collection should implement this for multiple types.
pub trait CollectionMut<T: ?Sized + 'static>: CollectionRef<T> {
    type ME<'a>: MutEntry<'a, T = T>
    where
        Self: 'a;

    // Implementations should specialize this for Composite items.
    /// Returns previous item.
    /// Errors:
    /// - KeyIsNotInUse
    fn set(&mut self, key: impl Into<Key<T>>, item: T) -> Result<T, Error>
    where
        T: Sized;

    // Implementations should specialize this for Composite items.
    /// Errors:
    /// - KeyIsNotInUse
    fn set_copy(&mut self, key: impl Into<Key<T>>, item: &T) -> Result<(), Error>
    where
        T: Copy;

    /// Errors:
    /// - KeyIsNotInUse
    fn get_mut<'a>(&'a mut self, key: impl Into<Key<T>>) -> Result<Self::ME<'a>, Error>;

    // NOTE: Posto se from mora mijenjati ovo se nemoze sigurno izvesti.
    // iz istog razloga se preporuca da kolekcija implementira ovo za sve tipove
    // na jednom structu.
    // fn iter_mut<'a>(&'a mut self) -> Self::IterMut<'a>;

    fn first_key(&self) -> Option<Key<T>>;

    /// Returns following key after given in ascending order.
    fn next_key(&self, key: impl Into<Key<T>>) -> Option<Key<T>>;

    /// Errors:
    /// - KeyIsNotInUse
    /// - ItemIsReferenced
    fn remove(&mut self, key: impl Into<Key<T>>) -> Result<bool, Error>;

    /// Errors:
    /// - KeyIsNotInUse
    /// - ItemIsReferenced
    fn take(&mut self, key: impl Into<Key<T>>) -> Result<Option<T>, Error>
    where
        T: Sized;
}

pub trait MutEntry<'a>: RefEntry<'a> {
    fn item_mut(&mut self) -> &mut Self::T;

    /// T as composite type now has one reference.
    fn add_reference(&mut self, reference: AnyRef);

    /// T as composite type now doesn't have one reference.
    fn remove_reference(&mut self, reference: AnyRef);
}
