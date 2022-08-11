use super::{catch_error, AnyKey, AnyRef, Collection, Error, Key, Ref};

pub trait Entry<'a>: 'a {
    type Coll: ?Sized;

    fn collection(&self) -> &Self::Coll;
}

pub trait EntryMut<'a>: Entry<'a> {
    fn collection_mut(&mut self) -> &mut Self::Coll;
}

pub trait AnyEntry<'a>: Entry<'a> {
    type IterAny: Iterator<Item = AnyKey> + 'a;

    fn key_any(&self) -> AnyKey;

    /// Bidirectional references.
    fn from_any(&self) -> Self::IterAny;

    /// True if this item is referenced by other items.
    fn referenced(&self) -> bool;
}

/// Only once finished are changes committed.
pub trait InitEntry<'a>: EntryMut<'a> {
    type T: ?Sized + 'static;

    fn add_reference<T: ?Sized + 'static>(&mut self, reference: impl Into<Ref<T>>)
    where
        Self::Coll: Collection<T>;

    fn add_from(&mut self, from: AnyRef);

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

pub trait MutEntry<'a>: RefEntry<'a> + EntryMut<'a> {
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
            reference.get_mut(coll)?.add_from(from);
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
            reference.get_mut(coll)?.remove_from(from);
            Ok(())
        });
    }
}
