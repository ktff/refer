use super::{AnyKey, AnyRef, Collection, Error, Key, PathMut, PathRef, Ref};

pub trait Entry<'a, P: PathRef<'a>>: 'a {
    fn path(&self) -> &P;
}

pub trait EntryMut<'a, P: PathMut<'a>>: Entry<'a, P> {
    fn path_mut(&mut self) -> &mut P;
}

pub trait AnyEntry<'a, P: PathRef<'a>>: Entry<'a, P> {
    type IterAny: Iterator<Item = AnyKey> + 'a;

    fn key_any(&self) -> AnyKey;

    /// Bidirectional references.
    fn from_any(&self) -> Self::IterAny;

    /// True if this item is referenced by other items.
    fn referenced(&self) -> bool;
}

/// Only once finished are changes committed.
pub trait InitEntry<'a, P: PathMut<'a>>: EntryMut<'a, P> {
    type T: ?Sized + 'static;

    fn add_reference<T: ?Sized + 'static>(&mut self, reference: impl Into<Ref<T>>)
    where
        P::Top: Collection<T>;

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
pub trait RefEntry<'a, P: PathRef<'a>>: AnyEntry<'a, P> {
    type T: ?Sized + 'static;
    type Iter<T: ?Sized + 'static>: Iterator<Item = Key<T>> + 'a;

    fn key(&self) -> Key<Self::T>;

    fn item(&self) -> &Self::T;

    /// Bidirectional references.
    fn from<T: ?Sized + 'static>(&self) -> Self::Iter<T>
    where
        P::Top: Collection<T>;
}

pub trait MutEntry<'a, P: PathMut<'a>>: RefEntry<'a, P> + EntryMut<'a, P> {
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
    fn add_reference<T: ?Sized + 'static>(
        &mut self,
        reference: impl Into<Ref<T>>,
    ) -> Result<(), Error>
    where
        P::Top: Collection<T>,
    {
        let reference = reference.into();
        let from = reference.from(self.key_any());
        let top = self.path_mut().top_mut();
        reference.get_mut(top)?.add_from(from);
        Ok(())
    }

    /// T as composite type now doesn't have one reference.
    fn remove_reference<T: ?Sized + 'static>(
        &mut self,
        reference: impl Into<Ref<T>>,
    ) -> Result<(), Error>
    where
        P::Top: Collection<T>,
    {
        let reference = reference.into();
        let from = reference.from(self.key_any());
        let top = self.path_mut().top_mut();
        reference.get_mut(top)?.remove_from(from);
        Ok(())
    }
}
