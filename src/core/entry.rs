use super::{
    AnyKey, AnyRef, BorrowPathMut, BorrowPathRef, Collection, Directioned, Error, Global, Key,
    Local, PathMut, PathRef, Ref, TypedRef,
};

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

    fn add_reference<T: ?Sized + 'static>(&mut self, reference: impl Into<TypedRef<T>>)
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

    /// Expects original reference with key of item referencing this one.
    fn add_from(&mut self, from: impl Into<AnyRef>);

    /// Expects original reference with key of item referencing this one.
    fn remove_from(&mut self, from: impl Into<AnyRef>);

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
}

// ************************ Convenient methods *************************** //

impl<D: Directioned, T: ?Sized + 'static> Ref<T, Global, D> {
    // Initializes T with provided init closure and adds self as reference.
    pub fn init<'a, P: PathMut<'a>, M: MutEntry<'a, P>>(
        from: &mut M,
        init: impl FnOnce(<P::Top as Collection<T>>::IE<'_, &mut P::Top>) -> Result<Key<T>, Error>,
    ) -> Result<Self, Error>
    where
        P::Top: Collection<T> + Collection<M::T>,
    {
        let key = init(Collection::<T>::add(from.path_mut().top_mut()))?;

        Self::new(from, key)
    }

    pub fn get<'a: 'b, 'b, P: PathRef<'a>, M: RefEntry<'a, P>>(
        self,
        from: &'b M,
    ) -> Result<<P::Top as Collection<T>>::RE<'b, &'b P::Top>, Error>
    where
        P::Top: Collection<T>,
    {
        Collection::<T>::get(from.path().top(), self.key())
    }

    pub fn get_mut<'a: 'b, 'b, P: PathMut<'a>, M: MutEntry<'a, P>>(
        self,
        from: &'b mut M,
    ) -> Result<<P::Top as Collection<T>>::ME<'b, &'b mut P::Top>, Error>
    where
        P::Top: Collection<T>,
    {
        Collection::<T>::get_mut(from.path_mut().top_mut(), self.key())
    }

    /// Returns Ref referencing from.
    pub fn from<'a, P: PathRef<'a>, M: RefEntry<'a, P>>(self, from: &M) -> Ref<M::T, Global, D>
    where
        P::Top: Collection<T> + Collection<M::T>,
    {
        self.reverse(from.path(), from.key())
    }

    /// Adds this reference to collection.
    pub fn add<'a, P: PathMut<'a>, M: MutEntry<'a, P>>(self, from: &mut M) -> Result<(), Error>
    where
        P::Top: Collection<T> + Collection<M::T>,
    {
        let from_ref = self.from(from);
        self.get_mut(from).map(|mut to| to.add_from(from_ref))
    }

    /// Removes this reference from collection.
    pub fn remove<'a, P: PathMut<'a>, M: MutEntry<'a, P>>(self, from: &mut M) -> Result<(), Error>
    where
        P::Top: Collection<T> + Collection<M::T>,
    {
        let from_ref = self.from(from);
        self.get_mut(from).map(|mut to| to.remove_from(from_ref))
    }
}

impl<D: Directioned, T: ?Sized + 'static> Ref<T, Local, D> {
    // Initializes T with provided init closure and adds self as reference.
    pub fn init<'a, P: PathMut<'a>, M: MutEntry<'a, P>>(
        from: &mut M,
        init: impl FnOnce(
            <P::Bottom as Collection<T>>::IE<'_, BorrowPathMut<'a, '_, P>>,
        ) -> Result<Key<T>, Error>,
    ) -> Result<Self, Error>
    where
        P::Bottom: Collection<T> + Collection<M::T>,
    {
        let key = init(Collection::<T>::add(from.path_mut().borrow_mut()))?;

        Self::new(from, key).map(|this| this.expect("Collection added item outside of path."))
    }

    pub fn get<'a: 'b, 'b, P: PathRef<'a>, M: RefEntry<'a, P>>(
        self,
        from: &'b M,
    ) -> Result<<P::Bottom as Collection<T>>::RE<'b, BorrowPathRef<'a, 'b, P>>, Error>
    where
        P::Bottom: Collection<T>,
    {
        <P::Bottom as Collection<T>>::get(from.path().borrow(), self.key())
    }

    pub fn get_mut<'a: 'b, 'b, P: PathMut<'a>, M: MutEntry<'a, P>>(
        self,
        from: &'b mut M,
    ) -> Result<<P::Bottom as Collection<T>>::ME<'b, BorrowPathMut<'a, 'b, P>>, Error>
    where
        P::Bottom: Collection<T>,
    {
        <P::Bottom as Collection<T>>::get_mut(from.path_mut().borrow_mut(), self.key())
    }

    pub fn from<'a, P: PathRef<'a>, M: RefEntry<'a, P>>(self, from: &M) -> Ref<M::T, Local, D>
    where
        P::Bottom: Collection<T> + Collection<M::T>,
    {
        self.reverse(from.path(), from.key())
            .expect("Entry returned key not from it's path.")
    }

    pub fn add<'a, P: PathMut<'a>, M: MutEntry<'a, P>>(self, from: &mut M) -> Result<(), Error>
    where
        P::Bottom: Collection<T> + Collection<M::T>,
    {
        let from_ref = self.from(from);
        self.get_mut(from).map(|mut to| to.add_from(from_ref))
    }

    pub fn remove<'a, P: PathMut<'a>, M: MutEntry<'a, P>>(self, from: &mut M) -> Result<(), Error>
    where
        P::Bottom: Collection<T> + Collection<M::T>,
    {
        let from_ref = self.from(from);
        self.get_mut(from).map(|mut to| to.remove_from(from_ref))
    }
}
