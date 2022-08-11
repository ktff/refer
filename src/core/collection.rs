use super::{catch_error, AnyCollection, AnyEntry, AnyRef, Error, Key, Ref};

/// Polly collection can implement this trait for each type.
pub trait Collection<T: ?Sized + 'static>: AnyCollection {
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

    /// Errors:
    /// - KeyIsNotInUse for any reference
    /// - OutOfKeys
    fn finish(self, item: Self::T) -> Result<Key<Self::T>, Error>
    where
        Self::T: Sized;

    /// Errors:
    /// - KeyIsNotInUse for any reference
    /// - OutOfKeys
    fn finish_copy(self, item: &Self::T) -> Result<Key<Self::T>, Error>
    where
        Self::T: Copy;
}

// Responsibilities of this trait shouldn't be delegated to T.
pub trait RefEntry<'a>: AnyEntry<'a> {
    type T: ?Sized + 'static;
    type Iter<T: ?Sized + 'static>: Iterator<Item = Key<T>> + 'a;

    fn key(&self) -> Key<Self::T>;

    fn item(&self) -> &Self::T;

    /// Bidirectional references.
    fn from<T: ?Sized + 'static>(&self) -> Self::Iter<T>
    where
        Self::Coll: Collection<T>;
}

pub trait MutEntry<'a>: RefEntry<'a> {
    fn collection_mut(&mut self) -> &mut Self::Coll;

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

impl<T: ?Sized + 'static> Key<T> {
    pub fn get<C: Collection<T>>(self, coll: &C) -> Result<C::RE<'_>, Error> {
        coll.get(self)
    }

    pub fn get_mut<C: Collection<T>>(self, coll: &mut C) -> Result<C::ME<'_>, Error> {
        coll.get_mut(self)
    }

    pub fn next_key<C: Collection<T>>(self, coll: &C) -> Option<Key<T>> {
        coll.next_key(self)
    }
}

pub trait CollectedType<C: Collection<Self>>: 'static {
    fn first_key(coll: &mut C) -> Option<Key<Self>>;

    fn add<'a>(coll: &'a mut C) -> C::IE<'a>;
}

impl<T: ?Sized + 'static, C: Collection<T>> CollectedType<C> for T {
    fn first_key(coll: &mut C) -> Option<Key<T>> {
        coll.first_key()
    }

    fn add<'a>(coll: &'a mut C) -> C::IE<'a> {
        coll.add()
    }
}
