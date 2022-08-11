use super::{AnyCollection, Error, InitEntry, Key, LayerMut, LayerRef, MutEntry, Ref, RefEntry};

/// Polly collection can implement this trait for each type.
pub trait Collection<T: ?Sized + 'static>: AnyCollection {
    type IE<'a, L: Collection<T> + LayerMut<Down = Self> + ?Sized + 'a>: InitEntry<
        'a,
        T = T,
        Coll = L,
    >
    where
        Self: 'a;

    type RE<'a, L: Collection<T> + LayerRef<Down = Self> + ?Sized + 'a>: RefEntry<
        'a,
        T = T,
        Coll = L,
    >
    where
        Self: 'a;

    type ME<'a, L: Collection<T> + LayerMut<Down = Self> + ?Sized + 'a>: MutEntry<
        'a,
        T = T,
        Coll = L,
    >
    where
        Self: 'a;

    /// How many lower bits of indices can be used for keys.
    fn indices_bits(&self) -> usize;

    fn first_key(&self) -> Option<Key<T>>;

    /// Returns following key after given in ascending order.
    fn next_key(&self, key: Key<T>) -> Option<Key<T>>;

    fn add<'a, L: Collection<T> + LayerMut<Down = Self> + ?Sized + 'a>(
        top: &'a mut L,
    ) -> Self::IE<'a, L>;

    /// Errors:
    /// - KeyIsNotInUse
    fn get<'a, L: Collection<T> + LayerRef<Down = Self> + ?Sized + 'a>(
        top: &'a L,
        key: impl Into<Key<T>>,
    ) -> Result<Self::RE<'a, L>, Error>;

    // NOTE: Since Key is numerical hence countable and storage needs to be able ot check if a key is valid
    // hence iteration is always possible although maybe expensive.

    /// Errors:
    /// - KeyIsNotInUse
    fn get_mut<'a, L: Collection<T> + LayerMut<Down = Self> + ?Sized + 'a>(
        top: &'a mut L,
        key: impl Into<Key<T>>,
    ) -> Result<Self::ME<'a, L>, Error>;

    /// A list of (first,last) keys representing in memory grouped items.
    /// In order of first -> next keys
    fn chunks(&self) -> Vec<(Key<T>, Key<T>)>;
}

impl<T: ?Sized + 'static> Key<T> {
    pub fn get<C: Collection<T> + ?Sized>(self, coll: &C) -> Result<C::RE<'_, C>, Error> {
        Collection::<T>::get(coll, self)
    }

    pub fn get_mut<C: Collection<T> + ?Sized>(self, coll: &mut C) -> Result<C::ME<'_, C>, Error> {
        Collection::<T>::get_mut(coll, self)
    }

    pub fn next_key<C: Collection<T> + ?Sized>(self, coll: &C) -> Option<Key<T>> {
        Collection::<T>::next_key(coll, self)
    }
}

impl<T: ?Sized + 'static> Ref<T> {
    pub fn get<'a, C: Collection<T> + ?Sized + 'a>(
        self,
        coll: &'a C,
    ) -> Result<C::RE<'_, C>, Error> {
        Collection::<T>::get(coll, self.1)
    }

    pub fn get_mut<'a, C: Collection<T> + ?Sized + 'a>(
        self,
        coll: &'a mut C,
    ) -> Result<C::ME<'_, C>, Error> {
        Collection::<T>::get_mut(coll, self.1)
    }
}

pub trait CollectedType<C: Collection<Self> + ?Sized>: 'static {
    fn first_key(coll: &mut C) -> Option<Key<Self>>;

    fn add<'a>(coll: &'a mut C) -> C::IE<'a, C>;
}

impl<T: ?Sized + 'static, C: Collection<T> + ?Sized> CollectedType<C> for T {
    fn first_key(coll: &mut C) -> Option<Key<T>> {
        coll.first_key()
    }

    fn add<'a>(coll: &'a mut C) -> C::IE<'a, C> {
        Collection::<T>::add(coll)
    }
}
