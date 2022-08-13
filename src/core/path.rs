use super::{AnyCollection, AnyKey, Collection, Directioned, Global, Key, Local, Ref};

/// A path from top to bottom.
/// Fetching bottom can cost.
pub trait PathRef<'a>: 'a {
    type Top: AnyCollection + ?Sized + 'a;
    type Bottom: AnyCollection + ?Sized + 'a;

    fn top(&self) -> &Self::Top;

    fn bottom(&self) -> &Self::Bottom;

    /// Converts top key to bottom key if it belongs to bottom.
    fn bottom_key<T: ?Sized + 'static>(&self, key: Key<T>) -> Option<Key<T>>
    where
        Self::Bottom: Collection<T>;

    fn bottom_key_any(&self, key: AnyKey) -> Option<AnyKey>;
}

impl<'a, C: AnyCollection + ?Sized + 'a> PathRef<'a> for &'a C {
    type Top = C;
    type Bottom = C;

    fn top(&self) -> &C {
        self
    }

    fn bottom(&self) -> &C {
        self
    }

    /// Converts top key to bottom key if it belongs to bottom.
    fn bottom_key<T: ?Sized + 'static>(&self, key: Key<T>) -> Option<Key<T>>
    where
        Self::Bottom: Collection<T>,
    {
        Some(key)
    }

    fn bottom_key_any(&self, key: AnyKey) -> Option<AnyKey> {
        Some(key)
    }
}

impl<'a, C: AnyCollection + ?Sized + 'a> PathRef<'a> for &'a mut C {
    type Top = C;
    type Bottom = C;

    fn top(&self) -> &C {
        self
    }

    fn bottom(&self) -> &C {
        self
    }

    /// Converts top key to bottom key if it belongs to bottom.
    fn bottom_key<T: ?Sized + 'static>(&self, key: Key<T>) -> Option<Key<T>>
    where
        Self::Bottom: Collection<T>,
    {
        Some(key)
    }

    fn bottom_key_any(&self, key: AnyKey) -> Option<AnyKey> {
        Some(key)
    }
}

pub trait PathMut<'a>: PathRef<'a> {
    fn top_mut(&mut self) -> &mut Self::Top;

    fn bottom_mut(&mut self) -> &mut Self::Bottom;
}

impl<'a, C: AnyCollection + ?Sized + 'a> PathMut<'a> for &'a mut C
where
    Self: PathRef<'a, Top = C, Bottom = C>,
{
    fn top_mut(&mut self) -> &mut C {
        self
    }

    fn bottom_mut(&mut self) -> &mut C {
        self
    }
}

// ********************** Convenient methods *************************** //

impl<D: Directioned, T: ?Sized + 'static> Ref<T, Global, D> {
    pub fn reverse<'a, F: ?Sized + 'static, P: PathRef<'a>>(
        self,
        _: &P,
        global_from: Key<F>,
    ) -> Ref<F, Global, D>
    where
        P::Top: Collection<F>,
    {
        Ref::new(global_from)
    }
}

impl<D: Directioned, T: ?Sized + 'static> Ref<T, Local, D> {
    /// None if key is not in local/bottom collection.
    pub fn reverse<'a, F: ?Sized + 'static, P: PathRef<'a>>(
        self,
        path: &P,
        global_from: Key<F>,
    ) -> Option<Ref<F, Local, D>>
    where
        P::Bottom: Collection<F>,
    {
        Some(Ref::new(path.bottom_key(global_from)?))
    }
}
