use std::marker::PhantomData;

use super::{
    AnyKey, AnyRef, BorrowPathMut, BorrowPathRef, Collection, Directionality, Directioned, Error,
    Global, Key, Local, Locality, PathMut, PathRef, Ref,
};

pub trait Entry<'a, P: PathRef<'a>>: 'a {
    fn path(&self) -> &P;
}

pub trait EntryMut<'a, P: PathMut<'a>>: Entry<'a, P> {
    fn path_mut(&mut self) -> &mut P;
}

pub trait AnyEntry<'a> {
    fn key_any(&self) -> AnyKey;

    /// Bidirectional references.
    fn from_any(&self) -> Vec<AnyKey>;

    /// True if this item is referenced by other items.
    fn referenced(&self) -> bool;
}

/// Only once finished are changes committed.
pub trait InitEntry<'a, P: PathMut<'a>>: EntryMut<'a, P> {
    type T: ?Sized + 'static;

    /// Adds reference of this item.
    /// Returns localized key.
    /// Errors:
    /// - NotLocalKey if this and some before local references aren't local.
    #[must_use]
    fn add_reference<T: ?Sized + 'static>(
        &mut self,
        locality: Locality,
        directionality: Directionality,
        global_key: Key<T>,
    ) -> Result<Key<T>, Error>
    where
        P::Top: Collection<T>;

    // fn add_from(&mut self, from: AnyRef);

    /// Added references are active once this is successful.
    /// Errors:
    /// - KeyIsNotInUse for any reference
    /// - OutOfKeys
    fn finish(self, item: Self::T) -> Result<Key<Self::T>, Error>
    where
        Self::T: Sized;

    /// Added references are active once this is successful.
    /// Errors:
    /// - KeyIsNotInUse for any reference
    /// - OutOfKeys
    fn finish_copy(self, item: &Self::T) -> Result<Key<Self::T>, Error>
    where
        Self::T: Copy;
}

// Responsibilities of this trait shouldn't be delegated to T.
pub trait RefEntry<'a, P: PathRef<'a>>: AnyEntry<'a> + Entry<'a, P> {
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
    /// Initializes T with provided init closure and adds self as reference.
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

    /// Creates new reference to the given item behind global key.
    pub fn new<'a, P: PathMut<'a>, E: MutEntry<'a, P>>(
        from: &mut E,
        global_key: Key<T>,
    ) -> Result<Self, Error>
    where
        P::Top: Collection<T> + Collection<E::T>,
    {
        let this = Ref::<T, Global, D>(global_key, PhantomData);
        this.add(from)?;

        Ok(this)
    }

    pub fn of<'a, P: PathMut<'a>, E: InitEntry<'a, P>>(
        from: &mut E,
        global_key: Key<T>,
    ) -> Result<Self, Error>
    where
        P::Top: Collection<T> + Collection<E::T>,
    {
        let key = from.add_reference(Locality::Global, D::D, global_key)?;
        let this = Ref::<T, Global, D>(key, PhantomData);

        Ok(this)
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
    fn from<'a, P: PathRef<'a>, M: RefEntry<'a, P>>(self, from: &M) -> Ref<M::T, Global, D>
    where
        P::Top: Collection<T> + Collection<M::T>,
    {
        Ref::<M::T, Global, D>(from.key(), PhantomData)
    }

    /// Adds this reference to collection.
    fn add<'a, P: PathMut<'a>, M: MutEntry<'a, P>>(self, from: &mut M) -> Result<(), Error>
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
    /// Initializes T with provided init closure and adds self as reference.
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

        Self::new(from, key)
    }

    /// Creates new reference to the given item behind global key.
    /// None if key is not in local/bottom collection.
    ///
    /// Errors:
    /// - Keys are not local
    pub fn new<'a, P: PathMut<'a>, E: MutEntry<'a, P>>(
        from: &mut E,
        global_key: Key<T>,
    ) -> Result<Self, Error>
    where
        P::Bottom: Collection<T> + Collection<E::T>,
    {
        let local_key = from
            .path()
            .bottom_key(global_key)
            .ok_or_else(|| Error::NotLocalKey(global_key.into()))?;
        let this = Ref::<T, Local, D>(local_key, PhantomData);
        this.add(from)?;

        Ok(this)
    }

    pub fn of<'a, P: PathMut<'a>, E: InitEntry<'a, P>>(
        from: &mut E,
        global_key: Key<T>,
    ) -> Result<Self, Error>
    where
        P::Top: Collection<T> + Collection<E::T>,
    {
        let key = from.add_reference(Locality::Local, D::D, global_key)?;
        let this = Ref::<T, Local, D>(key, PhantomData);

        Ok(this)
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

    fn from<'a, P: PathRef<'a>, M: RefEntry<'a, P>>(self, from: &M) -> Ref<M::T, Local, D>
    where
        P::Bottom: Collection<T> + Collection<M::T>,
    {
        Ref::<M::T, Local, D>(
            from.path()
                .bottom_key(from.key())
                .expect("Entry returned key not from it's path."),
            PhantomData,
        )
    }

    fn add<'a, P: PathMut<'a>, M: MutEntry<'a, P>>(self, from: &mut M) -> Result<(), Error>
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
