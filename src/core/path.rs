use std::marker::PhantomData;

use super::{AnyCollection, AnyKey, Collection, Key};

// ****************************** Ref ********************************* //

/// A path from top to bottom.
/// Fetching bottom can cost.
pub trait PathRef<'a>: 'a {
    type Top: AnyCollection + ?Sized + 'a;
    type Bottom: AnyCollection + ?Sized + 'a;

    fn top(&self) -> &Self::Top;

    fn bottom(&self) -> &Self::Bottom;

    fn top_key_any(&self, bottom_key: AnyKey) -> AnyKey;

    fn bottom_key_any(&self, top_key: AnyKey) -> Option<AnyKey>;

    /// Converts bottom key to top key.
    fn top_key<T: ?Sized + 'static>(&self, bottom_key: Key<T>) -> Key<T>
    where
        Self::Top: Collection<T>,
    {
        Key::new(self.top_key_any(bottom_key.into()).index())
    }

    /// Converts top key to bottom key if it belongs to bottom.
    fn bottom_key<T: ?Sized + 'static>(&self, top_key: Key<T>) -> Option<Key<T>>
    where
        Self::Bottom: Collection<T>,
    {
        self.bottom_key_any(top_key.into())
            .map(|key| Key::new(key.index()))
    }

    fn borrow<'b>(&'b self) -> BorrowPathRef<'a, 'b, Self> {
        BorrowPathRef(self, PhantomData)
    }
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

    fn top_key_any(&self, key: AnyKey) -> AnyKey {
        key
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

    fn top_key_any(&self, key: AnyKey) -> AnyKey {
        key
    }

    fn bottom_key_any(&self, key: AnyKey) -> Option<AnyKey> {
        Some(key)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BorrowPathRef<'a: 'b, 'b, P: PathRef<'a> + ?Sized>(&'b P, PhantomData<&'a P::Top>);

impl<'a: 'b, 'b, P: PathRef<'a> + ?Sized> PathRef<'b> for BorrowPathRef<'a, 'b, P> {
    type Top = P::Top;
    type Bottom = P::Bottom;

    fn top(&self) -> &P::Top {
        self.0.top()
    }
    fn bottom(&self) -> &P::Bottom {
        self.0.bottom()
    }

    fn top_key_any(&self, key: AnyKey) -> AnyKey {
        self.0.top_key_any(key)
    }

    fn bottom_key_any(&self, key: AnyKey) -> Option<AnyKey> {
        self.0.bottom_key_any(key)
    }

    fn top_key<T: ?Sized + 'static>(&self, bottom_key: Key<T>) -> Key<T>
    where
        Self::Top: Collection<T>,
    {
        self.0.top_key(bottom_key)
    }

    fn bottom_key<T: ?Sized + 'static>(&self, key: Key<T>) -> Option<Key<T>>
    where
        Self::Bottom: Collection<T>,
    {
        self.0.bottom_key(key)
    }
}

// ***************************** Mut ***************************** //

pub trait PathMut<'a>: PathRef<'a> {
    fn top_mut(&mut self) -> &mut Self::Top;

    fn bottom_mut(&mut self) -> &mut Self::Bottom;

    fn borrow_mut<'b>(&'b mut self) -> BorrowPathMut<'a, 'b, Self> {
        BorrowPathMut(self, PhantomData)
    }
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

#[derive(Debug)]
pub struct BorrowPathMut<'a: 'b, 'b, P: PathMut<'a> + ?Sized>(&'b mut P, PhantomData<&'a P::Top>);

impl<'a: 'b, 'b, P: PathMut<'a> + ?Sized> PathRef<'b> for BorrowPathMut<'a, 'b, P> {
    type Top = P::Top;
    type Bottom = P::Bottom;

    fn top(&self) -> &P::Top {
        self.0.top()
    }
    fn bottom(&self) -> &P::Bottom {
        self.0.bottom()
    }

    fn top_key_any(&self, key: AnyKey) -> AnyKey {
        self.0.top_key_any(key)
    }

    fn bottom_key_any(&self, key: AnyKey) -> Option<AnyKey> {
        self.0.bottom_key_any(key)
    }

    fn top_key<T: ?Sized + 'static>(&self, bottom_key: Key<T>) -> Key<T>
    where
        Self::Top: Collection<T>,
    {
        self.0.top_key(bottom_key)
    }

    fn bottom_key<T: ?Sized + 'static>(&self, key: Key<T>) -> Option<Key<T>>
    where
        Self::Bottom: Collection<T>,
    {
        self.0.bottom_key(key)
    }
}

impl<'a: 'b, 'b, P: PathMut<'a> + ?Sized> PathMut<'b> for BorrowPathMut<'a, 'b, P> {
    fn top_mut(&mut self) -> &mut P::Top {
        self.0.top_mut()
    }
    fn bottom_mut(&mut self) -> &mut P::Bottom {
        self.0.bottom_mut()
    }
}
