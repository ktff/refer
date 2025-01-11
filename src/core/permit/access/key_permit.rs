use super::*;

#[derive(Clone)]
pub struct Keys(HashSet<Key>);

impl Keys {
    pub fn new(iter: impl IntoIterator<Item = Key>) -> Self {
        Self(iter.into_iter().collect())
    }

    pub fn insert<K, T: DynItem + ?Sized>(&mut self, key: Key<K, T>) {
        self.0.insert(key.any().ptr());
    }

    pub fn try_insert<K, T: DynItem + ?Sized>(&mut self, key: Key<K, T>) -> bool {
        self.0.replace(key.any().ptr()).is_none()
    }

    pub fn contains<K, T: DynItem + ?Sized>(&self, key: Key<K, T>) -> bool {
        self.0.contains(&key.any().ptr())
    }

    pub fn remove<K, T: DynItem + ?Sized>(&mut self, key: Key<K, T>) -> bool {
        self.0.remove(&key.any().ptr())
    }
}

impl Default for Keys {
    fn default() -> Self {
        Self(HashSet::new())
    }
}

/// All keys bellow and including the top are contained.
#[derive(Clone, Copy)]
pub struct TopKey(IndexBase);

impl TopKey {
    pub fn new<K, T: DynItem + ?Sized>(top: Key<K, T>) -> Self {
        Self(top.index().get())
    }

    pub fn top(&self) -> IndexBase {
        self.0
    }

    pub fn key(&self) -> Option<Key> {
        Index::new(self.0).map(Key::new_any)
    }

    /// True if key wasn't present
    pub fn insert<K, T: DynItem + ?Sized>(&mut self, key: Key<K, T>) -> bool {
        if self.0 < key.index().get() {
            self.0 = key.index().get();
            true
        } else {
            false
        }
    }

    pub fn contains<K, T: DynItem + ?Sized>(&self, key: Key<K, T>) -> bool {
        key.index().get() <= self.0
    }
}

impl Default for TopKey {
    fn default() -> Self {
        Self(0)
    }
}

pub trait KeyPermit {
    fn allowed<K: Clone, T: DynItem + ?Sized>(&self, key: Key<K, T>) -> bool;
}

impl KeyPermit for All {
    fn allowed<K: Clone, T: DynItem + ?Sized>(&self, _: Key<K, T>) -> bool {
        true
    }
}

impl KeyPermit for Path {
    fn allowed<K: Clone, T: DynItem + ?Sized>(&self, key: Key<K, T>) -> bool {
        self.contains(key.path())
    }
}

impl<K: Clone, T: DynItem + ?Sized> KeyPermit for Key<K, T> {
    fn allowed<K2: Clone, T2: DynItem + ?Sized>(&self, key: Key<K2, T2>) -> bool {
        *self == key
    }
}

impl KeyPermit for Keys {
    fn allowed<K: Clone, T: DynItem + ?Sized>(&self, key: Key<K, T>) -> bool {
        !self.contains(key)
    }
}

impl KeyPermit for TopKey {
    fn allowed<K: Clone, T: DynItem + ?Sized>(&self, key: Key<K, T>) -> bool {
        !self.contains(key)
    }
}

impl<T: KeyPermit> KeyPermit for Not<T> {
    fn allowed<K: Clone, T2: DynItem + ?Sized>(&self, key: Key<K, T2>) -> bool {
        !self.0.allowed(key)
    }
}

impl<'a, T: KeyPermit> KeyPermit for &'a T {
    fn allowed<K: Clone, T2: DynItem + ?Sized>(&self, key: Key<K, T2>) -> bool {
        (**self).allowed(key)
    }
}

impl<'a, T: KeyPermit> KeyPermit for &'a mut T {
    fn allowed<K: Clone, T2: DynItem + ?Sized>(&self, key: Key<K, T2>) -> bool {
        (**self).allowed(key)
    }
}

pub trait PathPermit<C: AnyContainer + ?Sized>: KeyPermit + 'static {
    fn path(&self, container: &C) -> Path;
}

impl<C: AnyContainer + ?Sized> PathPermit<C> for All {
    fn path(&self, container: &C) -> Path {
        container.container_path()
    }
}

impl<C: AnyContainer + ?Sized> PathPermit<C> for Path {
    fn path(&self, _: &C) -> Path {
        *self
    }
}

pub trait KeySet {
    fn try_insert<K, T: DynItem + ?Sized>(&mut self, key: Key<K, T>) -> bool;

    fn contains<K, T: DynItem + ?Sized>(&self, key: Key<K, T>) -> bool;
}

impl KeySet for Keys {
    fn try_insert<K, T: DynItem + ?Sized>(&mut self, key: Key<K, T>) -> bool {
        self.0.replace(key.any().ptr()).is_none()
    }

    fn contains<K, T: DynItem + ?Sized>(&self, key: Key<K, T>) -> bool {
        self.0.contains(&key.any().ptr())
    }
}

impl KeySet for TopKey {
    fn try_insert<K, T: DynItem + ?Sized>(&mut self, key: Key<K, T>) -> bool {
        self.insert(key)
    }

    fn contains<K, T: DynItem + ?Sized>(&self, key: Key<K, T>) -> bool {
        self.contains(key)
    }
}

impl<'a, T: KeySet> KeySet for &'a mut T {
    fn try_insert<K, T2: DynItem + ?Sized>(&mut self, key: Key<K, T2>) -> bool {
        (**self).try_insert(key)
    }

    fn contains<K, T2: DynItem + ?Sized>(&self, key: Key<K, T2>) -> bool {
        (**self).contains(key)
    }
}
