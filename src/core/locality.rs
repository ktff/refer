use super::{
    permit, DrainItem, DynItem, DynSlot, Item, Key, KeyPath, LeafPath, LocalityKey, LocalityPath,
    Owned, PartialEdge, Path, Ref, Slot, StandaloneItem,
};
use getset::{CopyGetters, Getters};
use std::any::Any;

#[derive(Getters, Debug)]
#[getset(get = "pub")]
pub struct Locality<T: Item> {
    locality_key: LocalityKey,
    data: T::LocalityData,
    allocator: T::Alloc,
}

impl<T: Item> Locality<T> {
    pub fn new(leaf: LeafPath, data: T::LocalityData, allocator: T::Alloc) -> Self {
        Self {
            locality_key: LocalityKey::new(leaf),
            data,
            allocator,
        }
    }

    pub fn new_default(leaf: LeafPath) -> Self
    where
        T: Item<Alloc = std::alloc::Global>,
        T::LocalityData: Default,
    {
        Self {
            locality_key: LocalityKey::new(leaf),
            data: T::LocalityData::default(),
            allocator: std::alloc::Global,
        }
    }

    pub fn item_locality<'a>(&'a self, key: Key<Ref<'a>, T>) -> ItemLocality<'a, T> {
        ItemLocality::new(key, &self.data, &self.allocator)
    }

    pub fn container_locality(&self) -> ContainerLocality<'_, T> {
        ContainerLocality::new(self.locality_key.into(), &self.data, &self.allocator)
    }
}

pub type ItemLocality<'a, T: Item> = LocalityRef<'a, Key<Ref<'a>, T>, T::LocalityData, T::Alloc>;
pub type AnyItemLocality<'a> = LocalityRef<'a, Key<Ref<'a>>>;
pub type ContainerLocality<'a, T: Item> = LocalityRef<'a, KeyPath<T>, T::LocalityData, T::Alloc>;
pub type AnyContainerLocality<'a> = LocalityRef<'a>;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct LocalityRef<
    'a,
    K: LocalityPath + Copy = Path,
    D: ?Sized = (dyn Any + Send + Sync + 'static),
    A: ?Sized = (dyn Any + Send + Sync + 'static),
> {
    path: K,
    data: &'a D,
    allocator: &'a A,
}

impl<'a, K: LocalityPath + Copy, D: ?Sized, A: ?Sized> LocalityRef<'a, K, D, A> {
    pub fn new(path: K, data: &'a D, allocator: &'a A) -> Self {
        Self {
            path,
            data,
            allocator,
        }
    }
}

impl<'a, T: Item> ItemLocality<'a, T> {
    /// Adds drain edge to drain.
    /// UNSAFE: Caller must ensure to add returned key to this(source) Item edges as PartialEdge {object:key,side:Side::Source}.
    #[must_use]
    pub unsafe fn add_drain<D: DrainItem>(
        &self,
        drain: &mut Slot<permit::Mut, D>,
    ) -> Key<Owned, D> {
        let source = Key::<Owned, T>::new_owned(self.path.index());
        drain.add_drain_edge(source)
    }

    /// Adds drain edge to drain.
    /// UNSAFE: Caller must ensure to add returned key to this(source) Item edges as PartialEdge {object:key,side:Side::Source}.
    #[must_use]
    pub unsafe fn add_dyn_drain<D: DynItem + ?Sized>(
        &self,
        drain: &mut DynSlot<permit::Mut, D>,
    ) -> Option<Key<Owned, D>> {
        let source = Key::<Owned, T>::new_owned(self.path.index());
        drain
            .add_drain_edge(source)
            .map(Key::assume)
            .map_err(std::mem::forget)
            .ok()
    }

    /// Removes edge from object.
    /// Edge should have come from this Item edges.
    /// Panics if edge is not in object.
    pub fn remove_edge<D: StandaloneItem>(
        &self,
        edge: PartialEdge<Key<Owned, D>>,
        object: &mut Slot<permit::Mut, D>,
    ) {
        let (object_key, edge) = edge.reverse(self.path().ptr());
        let subject_key = object.remove_edge(object_key, edge);
        std::mem::forget(subject_key);
    }

    /// Removes edge from object.
    /// Edge should have come from this Item edges.
    /// Err if object can't remove it.
    /// Panics if edge is not in object.
    #[must_use]
    pub fn try_remove_edge<D: Item>(
        &self,
        edge: PartialEdge<Key<Owned, D>>,
        object: &mut Slot<permit::Mut, D>,
    ) -> Result<(), PartialEdge<Key<Owned, D>>> {
        let subject = edge.subject;
        let (object_key, edge) = edge.reverse(self.path().ptr());
        object
            .try_remove_edge(object_key, edge)
            .map(|subject_key| std::mem::forget(subject_key))
            .map_err(|object_key| subject.object(object_key))
    }

    /// Removes edge from object.
    /// Edge should have come from this Item edges.
    /// Err if object can't remove it.
    /// Panics if edge is not in object.
    #[must_use]
    pub fn try_remove_dyn_edge<D: DynItem + ?Sized>(
        &self,
        edge: PartialEdge<Key<Owned, D>>,
        object: &mut DynSlot<permit::Mut, D>,
    ) -> Result<(), PartialEdge<Key<Owned, D>>> {
        let subject = edge.subject;
        let (object_key, edge) = edge.reverse(self.path().ptr());
        object
            .remove_edge(object_key, edge)
            .map(|subject_key| std::mem::forget(subject_key))
            .map_err(|object_key| subject.object(object_key))
    }

    pub fn any(self) -> AnyItemLocality<'a> {
        AnyItemLocality {
            path: self.path.any(),
            data: self.data,
            allocator: self.allocator,
        }
    }
}

impl<'a, T: Item> ContainerLocality<'a, T> {
    pub fn any(self) -> AnyContainerLocality<'a> {
        AnyContainerLocality {
            path: self.path.path(),
            data: self.data,
            allocator: self.allocator,
        }
    }
}

impl<'a> AnyItemLocality<'a> {
    pub fn downcast<T: Item>(self) -> Option<ItemLocality<'a, T>> {
        if let Some(data) = self.data.downcast_ref() {
            Some(ItemLocality {
                path: self.path.assume(),
                data,
                allocator: self
                    .allocator
                    .downcast_ref()
                    .expect("Mismatched allocator type"),
            })
        } else {
            None
        }
    }
}

impl<'a> AnyContainerLocality<'a> {
    pub fn downcast<T: Item>(self) -> Option<ContainerLocality<'a, T>> {
        if let Some(data) = self.data.downcast_ref() {
            Some(ContainerLocality {
                path: KeyPath::new(self.path),
                data,
                allocator: self
                    .allocator
                    .downcast_ref()
                    .expect("Mismatched allocator type"),
            })
        } else {
            None
        }
    }
}

impl<'a, K: LocalityPath + Copy, D: ?Sized, A: ?Sized> Copy for LocalityRef<'a, K, D, A> {}

impl<'a, K: LocalityPath + Copy, D: ?Sized, A: ?Sized> Clone for LocalityRef<'a, K, D, A> {
    fn clone(&self) -> Self {
        Self {
            path: self.path,
            data: self.data,
            allocator: self.allocator,
        }
    }
}
