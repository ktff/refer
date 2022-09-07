use super::{Access, AnyItem, AnyKey, AnyShell, AnyShells, ItemsMut, Key, ShellsMut};
use std::fmt;

#[derive(PartialEq, Eq)]
pub struct Ref<T: ?Sized + 'static>(Key<T>);

impl<T: ?Sized + 'static> Ref<T> {
    pub fn new(key: Key<T>) -> Self {
        Ref(key)
    }

    pub fn key(&self) -> Key<T> {
        self.0
    }
}

impl<T: ?Sized + 'static> Ref<T> {
    /// Panics if to doesn't exist.
    pub fn connect(from: AnyKey, to: Key<T>, collection: &mut impl ShellsMut<T>) -> Self {
        let to_shell = collection.get_mut(to).expect("Item doesn't exist");
        to_shell.add_from(from);
        Self::new(to)
    }

    pub fn disconnect(self, from: AnyKey, collection: &mut impl ShellsMut<T>) {
        if let Some(to_shell) = collection.get_mut(self.key()) {
            to_shell.remove_from(from)
        }
    }

    fn panic_dangling(&self) -> ! {
        panic!("Dangling reference {:?}", self);
    }
}

impl<T: AnyItem + ?Sized> Ref<T> {
    pub fn get<C: Access<T>>(self, coll: &C) -> (&T, &C::Shell) {
        coll.get(self.key())
            .unwrap_or_else(|| self.panic_dangling())
    }

    pub fn get_mut<C: Access<T>>(self, coll: &mut C) -> (&mut T, &C::Shell) {
        coll.get_mut(self.key())
            .unwrap_or_else(|| self.panic_dangling())
    }

    pub fn item<C: ItemsMut<T>>(self, coll: &C) -> &T {
        coll.get(self.key())
            .unwrap_or_else(|| self.panic_dangling())
    }

    pub fn item_mut<C: ItemsMut<T>>(self, coll: &mut C) -> &mut T {
        coll.get_mut(self.key())
            .unwrap_or_else(|| self.panic_dangling())
    }

    pub fn shell<C: ShellsMut<T>>(self, coll: &C) -> &C::Shell {
        coll.get(self.key())
            .unwrap_or_else(|| self.panic_dangling())
    }

    pub fn shell_mut<C: ShellsMut<T>>(self, coll: &mut C) -> &mut C::Shell {
        coll.get_mut(self.key())
            .unwrap_or_else(|| self.panic_dangling())
    }
}

impl<T: ?Sized + 'static> Copy for Ref<T> {}

impl<T: ?Sized + 'static> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Ref(self.0)
    }
}

impl<T: ?Sized + 'static> From<Ref<T>> for Key<T> {
    fn from(ref_: Ref<T>) -> Self {
        ref_.0
    }
}

impl<T: ?Sized + 'static> fmt::Debug for Ref<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ref({:?})", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AnyRef(AnyKey);

impl AnyRef {
    pub fn new(key: AnyKey) -> Self {
        Self(key)
    }

    pub fn key(&self) -> AnyKey {
        self.0
    }

    pub fn downcast<T: ?Sized + 'static>(self) -> Option<Ref<T>> {
        self.0.downcast().map(Ref)
    }

    /// Panics if to doesn't exist.
    pub fn connect(from: AnyKey, to: AnyKey, collection: &mut (impl AnyShells + ?Sized)) -> Self {
        let to_shell = collection.get_mut_any(to).expect("Item doesn't exist");
        to_shell.add_from(from);
        Self::new(to)
    }

    pub fn disconnect(self, from: AnyKey, collection: &mut (impl AnyShells + ?Sized)) {
        if let Some(to_shell) = collection.get_mut_any(self.key()) {
            to_shell.remove_from(from);
        }
    }
}

impl<T: ?Sized + 'static> From<Ref<T>> for AnyRef {
    fn from(ref_: Ref<T>) -> Self {
        AnyRef(ref_.0.into())
    }
}
