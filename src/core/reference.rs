use crate::core::*;
use std::fmt;

// TODO: Pinned outer references

#[derive(PartialEq, Eq)]
pub struct Ref<T: AnyItem>(Key<T>);

impl<T: AnyItem> Ref<T> {
    pub fn new(key: Key<T>) -> Self {
        Ref(key)
    }

    pub fn key(&self) -> Key<T> {
        self.0
    }
}

impl<T: AnyItem> Ref<T> {
    /// Panics if to doesn't exist.
    pub fn connect(from: AnyKey, to: Key<T>, collection: &mut impl ShellsMut<T>) -> Self {
        let mut shell = collection.get_mut(to).expect("Item doesn't exist");
        shell.add_from(from);
        Self::new(to)
    }

    pub fn disconnect(self, from: AnyKey, collection: &mut impl ShellsMut<T>) {
        if let Some(mut shell) = collection.get_mut(self.key()) {
            shell.remove_from(from)
        }
    }

    fn panic_dangling(&self) -> ! {
        panic!("Dangling reference {:?}", self);
    }
}

impl<T: AnyItem> Ref<T> {
    pub fn get<C: Access<T>>(self, coll: &C) -> RefSlot<T, C::GroupItem, C::Shell, C::Alloc> {
        coll.get(self.key())
            .unwrap_or_else(|| self.panic_dangling())
    }

    pub fn get_mut<C: Access<T>>(
        self,
        coll: &mut C,
    ) -> MutSlot<T, C::GroupItem, C::Shell, C::Alloc> {
        coll.get_mut(self.key())
            .unwrap_or_else(|| self.panic_dangling())
    }

    pub fn item<C: ItemsMut<T>>(self, coll: &C) -> RefItemSlot<T, C::GroupItem, C::Alloc> {
        coll.get(self.key())
            .unwrap_or_else(|| self.panic_dangling())
    }

    pub fn item_mut<C: ItemsMut<T>>(self, coll: &mut C) -> MutItemSlot<T, C::GroupItem, C::Alloc> {
        coll.get_mut(self.key())
            .unwrap_or_else(|| self.panic_dangling())
    }

    pub fn shell<C: ShellsMut<T>>(self, coll: &C) -> RefShellSlot<T, C::Shell, C::Alloc> {
        coll.get(self.key())
            .unwrap_or_else(|| self.panic_dangling())
    }

    pub fn shell_mut<C: ShellsMut<T>>(self, coll: &mut C) -> MutShellSlot<T, C::Shell, C::Alloc> {
        coll.get_mut(self.key())
            .unwrap_or_else(|| self.panic_dangling())
    }
}

impl<T: AnyItem> Copy for Ref<T> {}

impl<T: AnyItem> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Ref(self.0)
    }
}

impl<T: AnyItem> From<Ref<T>> for Key<T> {
    fn from(ref_: Ref<T>) -> Self {
        ref_.0
    }
}

impl<T: AnyItem> fmt::Debug for Ref<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ref({:?})", self.0)
    }
}

impl<T: AnyItem> PartialEq<AnyKey> for Ref<T> {
    fn eq(&self, other: &AnyKey) -> bool {
        AnyKey::from(self.0) == *other
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

    pub fn downcast<T: AnyItem>(self) -> Option<Ref<T>> {
        self.0.downcast().map(Ref)
    }

    /// Panics if to doesn't exist.
    pub fn connect(from: AnyKey, to: AnyKey, collection: MutShells<impl AnyContainer>) -> Self {
        collection
            .get(to)
            .expect("Item doesn't exist")
            .add_from(from);
        Self::new(to)
    }

    pub fn disconnect(self, from: AnyKey, collection: MutShells<impl AnyContainer>) {
        if let Some(mut slot) = collection.get(self.key()) {
            slot.shell_mut().remove_from(from);
        }
    }
}

impl<T: AnyItem> From<Ref<T>> for AnyRef {
    fn from(ref_: Ref<T>) -> Self {
        AnyRef(ref_.0.into())
    }
}
