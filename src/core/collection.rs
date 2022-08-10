use super::{catch_error, AnyEntry, AnyRef, Error, Key, PollyCollection, Ref};

/// Polly collection can implement this trait for each type.
pub trait Collection<T: ?Sized + 'static>: PollyCollection {
    type IE<'a>: InitEntry<'a, T = T, Coll = Self>
    where
        Self: 'a;

    type RE<'a>: RefEntry<'a, T = T, Coll = Self>
    where
        Self: 'a;

    type ME<'a>: MutEntry<'a, T = T, Coll = Self>
    where
        Self: 'a;

    fn first_key(&self) -> Option<Key<T>>;

    /// Returns following key after given in ascending order.
    fn next_key(&self, key: Key<T>) -> Option<Key<T>>;

    fn add<'a>(&'a mut self) -> Self::IE<'a>;

    /// Errors:
    /// - KeyIsNotInUse
    fn get<'a>(&'a self, key: impl Into<Key<T>>) -> Result<Self::RE<'a>, Error>;

    // NOTE: Since Key is numerical hence countable and storage needs to be able ot check if a key is valid
    // hence iteration is always possible although maybe expensive.

    /// Errors:
    /// - KeyIsNotInUse
    fn get_mut<'a>(&'a mut self, key: impl Into<Key<T>>) -> Result<Self::ME<'a>, Error>;

    // NOTE: Posto se from mora mijenjati ovo se nemoze sigurno izvesti.
    // iz istog razloga se preporuca da kolekcija implementira ovo za sve tipove
    // na jednom structu.
    // fn iter_mut<'a>(&'a mut self) -> Self::IterMut<'a>;
}

/// Only once finished are changes committed.
pub trait InitEntry<'a> {
    type T: ?Sized + 'static;
    type Coll;

    fn add_reference<T: ?Sized + 'static>(&mut self, reference: impl Into<Ref<T>>)
    where
        Self::Coll: Collection<T>;

    fn add_from(&mut self, from: AnyRef);

    fn collection_mut(&mut self) -> &mut Self::Coll;

    fn finish(self, item: Self::T) -> Result<Key<Self::T>, Error>
    where
        Self::T: Sized;

    fn finish_copy(self, item: &Self::T) -> Result<Key<Self::T>, Error>
    where
        Self::T: Copy;
}

// Responsibilities of this trait shouldn't be delegated to T.
pub trait RefEntry<'a>: AnyEntry<'a> {
    type T: ?Sized;
    type Iter<T: ?Sized>: Iterator<Item = Key<T>> + 'a;

    fn key(&self) -> Key<Self::T>;

    fn item(&self) -> &Self::T;

    /// Bidirectional references.
    fn from<T: ?Sized>(&self) -> Self::Iter<T>;
}

pub trait MutEntry<'a>: RefEntry<'a> {
    fn item_mut(&mut self) -> &mut Self::T;

    fn add_from(&mut self, from: AnyRef);

    fn remove_from(&mut self, from: AnyRef);

    fn set_copy(&mut self, item: &Self::T)
    where
        Self::T: Copy;

    /// Errors:
    /// - ItemIsReferenced
    fn remove(self) -> Result<(), Error>;

    /// Errors:
    /// - ItemIsReferenced
    fn take(self) -> Result<Self::T, Error>
    where
        Self::T: Sized;

    fn collection_mut(&mut self) -> &mut Self::Coll;

    /// T as composite type now has one reference.
    fn add_reference<T: ?Sized + 'static>(&mut self, reference: impl Into<Ref<T>>)
    where
        Self::Coll: Collection<T>,
    {
        // Note: it's better to log the errors here then to propagate them higher and potentially
        // creating more issues.
        catch_error(|| {
            let reference = reference.into();
            let from = reference.from(self.key_any());
            let coll = self.collection_mut();
            coll.get_mut(reference.1)?.add_from(from);
            Ok(())
        });
    }

    /// T as composite type now doesn't have one reference.
    fn remove_reference<T: ?Sized + 'static>(&mut self, reference: impl Into<Ref<T>>)
    where
        Self::Coll: Collection<T>,
    {
        catch_error(|| {
            let reference = reference.into();
            let from = reference.from(self.key_any());
            let coll = self.collection_mut();
            coll.get_mut(reference.1)?.remove_from(from);
            Ok(())
        });
    }
}

impl<T: ?Sized> Key<T> {
    pub fn get<C: Collection<T>>(self, coll: &C) -> Result<C::RE<'_>, Error> {
        coll.get(self)
    }
}
