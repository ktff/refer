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

pub trait KeyPermit {
    type State: Clone;

    fn allowed<K: Clone, T: DynItem + ?Sized>(state: &Self::State, key: Key<K, T>) -> bool;
}

impl KeyPermit for All {
    type State = ();

    fn allowed<K: Clone, T: DynItem + ?Sized>(_: &Self::State, _: Key<K, T>) -> bool {
        true
    }
}

impl KeyPermit for Path {
    type State = Path;

    fn allowed<K: Clone, T: DynItem + ?Sized>(state: &Self::State, key: Key<K, T>) -> bool {
        state.contains(key.path())
    }
}

impl<K: Clone, T: DynItem + ?Sized> KeyPermit for Key<K, T> {
    type State = Key<K, T>;

    fn allowed<K2: Clone, T2: DynItem + ?Sized>(state: &Self::State, key: Key<K2, T2>) -> bool {
        *state == key
    }
}

impl KeyPermit for Keys {
    type State = Keys;

    fn allowed<K: Clone, T: DynItem + ?Sized>(state: &Self::State, key: Key<K, T>) -> bool {
        state.contains(key)
    }
}

impl<T: KeyPermit> KeyPermit for Not<T> {
    type State = T::State;

    fn allowed<K: Clone, T2: DynItem + ?Sized>(state: &Self::State, key: Key<K, T2>) -> bool {
        !T::allowed(state, key)
    }
}

pub trait PathPermit<C: AnyContainer + ?Sized>: KeyPermit + 'static {
    fn path(state: &Self::State, container: &C) -> Path;
}

impl<C: AnyContainer + ?Sized> PathPermit<C> for All {
    fn path(_: &Self::State, container: &C) -> Path {
        container.container_path()
    }
}

impl<C: AnyContainer + ?Sized> PathPermit<C> for Path {
    fn path(state: &Self::State, _: &C) -> Path {
        *state
    }
}
