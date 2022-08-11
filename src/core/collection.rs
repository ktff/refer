use super::{AnyCollection, Error, InitEntry, Key, MutEntry, PathMut, PathRef, Ref, RefEntry};

/// Polly collection can implement this trait for each type.
pub trait Collection<T: ?Sized + 'static>: AnyCollection {
    type IE<'a, P: PathMut<'a, Bottom = Self>>: InitEntry<'a, P, T = T>
    where
        Self: 'a,
        P::Top: Collection<T>;

    type RE<'a, P: PathRef<'a, Bottom = Self>>: RefEntry<'a, P, T = T>
    where
        Self: 'a,
        P::Top: Collection<T>;

    type ME<'a, P: PathMut<'a, Bottom = Self>>: MutEntry<'a, P, T = T>
    where
        Self: 'a,
        P::Top: Collection<T>;

    /// How many lower bits of indices can be used for keys.
    fn indices_bits(&self) -> usize;

    fn first_key(&self) -> Option<Key<T>>;

    /// Returns following key after given in ascending order.
    fn next_key(&self, key: Key<T>) -> Option<Key<T>>;

    fn add<'a, P: PathMut<'a, Bottom = Self>>(path: P) -> Self::IE<'a, P>
    where
        P::Top: Collection<T>;

    /// Errors:
    /// - KeyIsNotInUse
    fn get<'a, P: PathRef<'a, Bottom = Self>>(
        path: P,
        key: impl Into<Key<T>>,
    ) -> Result<Self::RE<'a, P>, Error>
    where
        P::Top: Collection<T>;

    // NOTE: Since Key is numerical hence countable and storage needs to be able ot check if a key is valid
    // hence iteration is always possible although maybe expensive.

    /// Errors:
    /// - KeyIsNotInUse
    fn get_mut<'a, P: PathMut<'a, Bottom = Self>>(
        path: P,
        key: impl Into<Key<T>>,
    ) -> Result<Self::ME<'a, P>, Error>
    where
        P::Top: Collection<T>;
}

// ********************** Convenience methods **********************

impl<T: ?Sized + 'static> Key<T> {
    pub fn get<C: Collection<T> + ?Sized>(self, coll: &C) -> Result<C::RE<'_, &C>, Error> {
        Collection::<T>::get(coll, self)
    }

    pub fn get_mut<C: Collection<T> + ?Sized>(
        self,
        coll: &mut C,
    ) -> Result<C::ME<'_, &mut C>, Error> {
        Collection::<T>::get_mut(coll, self)
    }

    pub fn next_key<C: Collection<T> + ?Sized>(self, coll: &C) -> Option<Key<T>> {
        Collection::<T>::next_key(coll, self)
    }
}

impl<T: ?Sized + 'static> Ref<T> {
    pub fn get<C: Collection<T> + ?Sized>(self, coll: &C) -> Result<C::RE<'_, &C>, Error> {
        Collection::<T>::get(coll, self.1)
    }

    pub fn get_mut<C: Collection<T> + ?Sized>(
        self,
        coll: &mut C,
    ) -> Result<C::ME<'_, &mut C>, Error> {
        Collection::<T>::get_mut(coll, self.1)
    }
}

pub trait CollectedType<C: Collection<Self> + ?Sized>: 'static {
    fn first_key(coll: &mut C) -> Option<Key<Self>>;

    fn add<'a>(coll: &'a mut C) -> C::IE<'a, &'a mut C>;
}

impl<T: ?Sized + 'static, C: Collection<T> + ?Sized> CollectedType<C> for T {
    fn first_key(coll: &mut C) -> Option<Key<T>> {
        coll.first_key()
    }

    fn add<'a>(coll: &'a mut C) -> C::IE<'a, &'a mut C> {
        Collection::<T>::add(coll)
    }
}
