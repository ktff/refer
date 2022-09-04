use super::{AnyItem, AnyKey, AnyShell, Collection, ItemsMut, Key, ShellsMut};

#[derive(Debug, PartialEq, Eq)]
pub struct Ref<T: ?Sized>(Key<T>);

impl<T: ?Sized> Ref<T> {
    pub fn new(key: Key<T>) -> Self {
        Ref(key)
    }

    pub fn key(&self) -> Key<T> {
        self.0
    }
}

impl<T: AnyItem + ?Sized> Ref<T> {
    /// Some if to shells exist, otherwise None.
    pub fn connect(from: AnyKey, to: Key<T>, collection: &mut impl ShellsMut<T>) -> Option<Self> {
        let to_shell = collection.get_mut(to)?;
        to_shell.add_from(from);
        Some(Self::new(to))
    }

    /// True if there was reference to remove.
    pub fn disconnect(self, from: AnyKey, collection: &mut impl ShellsMut<T>) -> bool {
        if let Some(to_shell) = collection.get_mut(self.key()) {
            to_shell.remove_from(from)
        } else {
            false
        }
    }

    pub fn get<C: Collection<T>>(self, coll: &C) -> (&T, &C::Shell) {
        coll.get(self.key()).expect("Entry isn't present")
    }

    pub fn get_mut<C: Collection<T>>(self, coll: &mut C) -> (&mut T, &C::Shell) {
        coll.get_mut(self.key()).expect("Entry isn't present")
    }

    pub fn item<C: ItemsMut<T>>(self, coll: &C) -> &T {
        coll.get(self.key()).expect("Item isn't present")
    }

    pub fn item_mut<C: ItemsMut<T>>(self, coll: &mut C) -> &mut T {
        coll.get_mut(self.key()).expect("Item isn't present")
    }

    pub fn shell<C: ShellsMut<T>>(self, coll: &C) -> &C::Shell {
        coll.get(self.key()).expect("Shell isn't present")
    }

    pub fn shell_mut<C: ShellsMut<T>>(self, coll: &mut C) -> &mut C::Shell {
        coll.get_mut(self.key()).expect("Shell isn't present")
    }
}

impl<T: ?Sized> Copy for Ref<T> {}

impl<T: ?Sized> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Ref(self.0)
    }
}

impl<T: ?Sized> From<Ref<T>> for Key<T> {
    fn from(ref_: Ref<T>) -> Self {
        ref_.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AnyRef(pub AnyKey);

impl AnyRef {
    pub fn key(&self) -> AnyKey {
        self.0
    }

    pub fn downcast<T: ?Sized + 'static>(self) -> Option<Ref<T>> {
        self.0.downcast().map(Ref)
    }
}

impl<T: ?Sized + 'static> From<Ref<T>> for AnyRef {
    fn from(ref_: Ref<T>) -> Self {
        AnyRef(ref_.0.into())
    }
}
