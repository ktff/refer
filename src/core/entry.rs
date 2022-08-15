use std::marker::PhantomData;

use super::{
    AnyKey, AnyRef, BorrowPathMut, BorrowPathRef, Collection, Directioned, Error, Global, Key,
    Local, PathMut, PathRef, Ref,
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

    /// Will try to localize item to locality of given key.
    /// Returns localized key.
    /// Errors:
    /// - NotLocalKey if entry was already localized to a different locality.
    fn localize<T: ?Sized + 'static>(&mut self, near: Key<T>) -> Result<Key<T>, Error>;

    /// Adds reference of this item.
    /// Errors:
    /// - NotLocalKey if entry was not localized and reference is local.
    fn add_to(&mut self, to: impl Into<AnyRef>) -> Result<(), Error>;

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

    // The point of this is T :?Sized which can have different sizes.
    fn set_copy(&mut self, item: &Self::T)
    where
        Self::T: Copy;

    /// Drop function must remove references in drop function.
    /// If `drop` is called, item will be removed and R returned.
    /// Errors:
    /// - ItemIsReferenced
    fn remove<R>(self, drop: impl FnOnce(&mut Self) -> R) -> Result<R, Error>;
}

// ************************ Convenient methods *************************** //

impl<D: Directioned, T: ?Sized + 'static> Ref<T, Global, D> {
    /// Initializes T with provided init closure and adds self as reference.
    pub fn add<'a, P: PathMut<'a>, M: MutEntry<'a, P>>(
        from: &mut M,
        init: impl FnOnce(<P::Top as Collection<T>>::IE<'_, &mut P::Top>) -> Result<Key<T>, Error>,
    ) -> Result<Self, Error>
    where
        P::Top: Collection<T> + Collection<M::T>,
    {
        let key = init(Collection::<T>::add(from.path_mut().top_mut()))?;

        Self::bind(from, key)
    }

    /// Creates new reference to the given item.
    pub fn bind<'a, P: PathMut<'a>, E: MutEntry<'a, P>>(
        from: &mut E,
        key: Key<T>,
    ) -> Result<Self, Error>
    where
        P::Top: Collection<T> + Collection<E::T>,
    {
        let this = Ref::<T, Global, D>(key, PhantomData);
        let from_ref = this.from(from);
        this.get_mut(from)?.add_from(from_ref);

        Ok(this)
    }

    pub fn init<'a, P: PathMut<'a>, E: InitEntry<'a, P>>(
        from: &mut E,
        key: Key<T>,
    ) -> Result<Self, Error>
    where
        P::Top: Collection<T> + Collection<E::T>,
    {
        let this = Ref::<T, Global, D>(key, PhantomData);
        from.add_to(this)?;

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

    /// Removes this reference from collection.
    pub fn remove<'a, P: PathMut<'a>, M: MutEntry<'a, P>>(self, from: &mut M) -> Result<(), Error>
    where
        P::Top: Collection<T> + Collection<M::T>,
    {
        let from_ref = self.from(from);
        self.get_mut(from).map(|mut to| to.remove_from(from_ref))
    }

    /// Returns Ref referencing from.
    fn from<'a, P: PathRef<'a>, M: RefEntry<'a, P>>(self, from: &M) -> Ref<M::T, Global, D>
    where
        P::Top: Collection<T> + Collection<M::T>,
    {
        Ref::<M::T, Global, D>(from.key(), PhantomData)
    }
}

impl<D: Directioned, T: ?Sized + 'static> Ref<T, Local, D> {
    /// Initializes T with provided init closure and adds self as reference.
    pub fn add<'a, P: PathMut<'a>, M: MutEntry<'a, P>>(
        from: &mut M,
        init: impl FnOnce(
            <P::Bottom as Collection<T>>::IE<'_, BorrowPathMut<'a, '_, P>>,
        ) -> Result<Key<T>, Error>,
    ) -> Result<Self, Error>
    where
        P::Bottom: Collection<T> + Collection<M::T>,
    {
        let from_key = from.key();
        let mut entry = Collection::<T>::add(from.path_mut().borrow_mut());
        let _ = entry.localize(from_key)?;
        let key = init(entry)?;

        Self::bind(from, key)
    }

    /// Creates new reference to the given item.
    /// None if key is not in local/bottom collection.
    ///
    /// Errors:
    /// - Keys are not local
    pub fn bind<'a, P: PathMut<'a>, E: MutEntry<'a, P>>(
        from: &mut E,
        key: Key<T>,
    ) -> Result<Self, Error>
    where
        P::Bottom: Collection<T> + Collection<E::T>,
    {
        let local_key = from
            .path()
            .bottom_key(key)
            .ok_or_else(|| Error::NotLocalKey(key.into()))?;
        let this = Ref::<T, Local, D>(local_key, PhantomData);
        let from_ref = this.from(from);
        this.get_mut(from)?.add_from(from_ref);

        Ok(this)
    }

    pub fn init<'a, P: PathMut<'a>, E: InitEntry<'a, P>>(
        from: &mut E,
        key: Key<T>,
    ) -> Result<Self, Error>
    where
        P::Bottom: Collection<T> + Collection<E::T>,
    {
        let local_key = from.localize(key)?;
        let this = Ref::<T, Local, D>(local_key, PhantomData);
        from.add_to(this)?;

        Ok(this)
    }

    pub fn get<'a: 'b, 'b, P: PathRef<'a>, M: RefEntry<'a, P>>(
        self,
        from: &'b M,
    ) -> Result<<P::Bottom as Collection<T>>::RE<'b, BorrowPathRef<'a, 'b, P>>, Error>
    where
        P::Bottom: Collection<T>,
    {
        <P::Bottom as Collection<T>>::get(from.path().borrow(), self.0)
    }

    pub fn get_mut<'a: 'b, 'b, P: PathMut<'a>, M: MutEntry<'a, P>>(
        self,
        from: &'b mut M,
    ) -> Result<<P::Bottom as Collection<T>>::ME<'b, BorrowPathMut<'a, 'b, P>>, Error>
    where
        P::Bottom: Collection<T>,
    {
        <P::Bottom as Collection<T>>::get_mut(from.path_mut().borrow_mut(), self.0)
    }

    pub fn remove<'a, P: PathMut<'a>, M: MutEntry<'a, P>>(self, from: &mut M) -> Result<(), Error>
    where
        P::Bottom: Collection<T> + Collection<M::T>,
    {
        let from_ref = self.from(from);
        self.get_mut(from).map(|mut to| to.remove_from(from_ref))
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
}
