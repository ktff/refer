use super::{
    permit, BiItem, DrainItem, DynItem, DynSlot, Item, Key, KeyPath, LeafPath, LocalityKey,
    LocalityPath, Owned, Path, Ref, Slot,
};
use getset::{CopyGetters, Getters};
use std::{any::Any, num::NonZeroUsize};

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

    pub fn index_locality<'a>(&'a self, index: NonZeroUsize) -> ItemLocality<'a, T> {
        let key = self.locality_key().key_of::<T>(index);
        // SAFETY: This is safe because locality does exist.
        let key: Key<Ref<'a>, T> = unsafe { Key::new_ref(key.index()) };
        self.item_locality(key)
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

impl<'a, T: DynItem + ?Sized, D: ?Sized, A: ?Sized> LocalityRef<'a, Key<Ref<'a>, T>, D, A> {
    /// UNSAFE: This isn't unsafe per se since safety checks will still be made, but they can panic if
    /// caller allows for this key to outlive this T.
    ///
    /// Callers should also forget this Key<Owned> when they don't need to guarantee that T exists through it anymore.
    #[must_use]
    pub unsafe fn owned_key(&self) -> Key<Owned, T> {
        Key::new_owned(self.path.index())
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
        drain.localized(|item, locality| item.add_drain_edge(locality, self.owned_key()));
        drain.locality().owned_key()
    }

    /// Adds drain edge to drain.
    /// UNSAFE: Caller must ensure to add returned key to this(source) Item edges as PartialEdge {object:key,side:Side::Source}.
    #[must_use]
    pub unsafe fn add_dyn_drain<D: DynItem + ?Sized>(
        &self,
        drain: &mut DynSlot<permit::Mut, D>,
    ) -> Option<Key<Owned, D>> {
        drain
            .any_localized(|item, locality| {
                item.any_add_drain_edge(locality, self.owned_key().any())
            })
            .map(|()| drain.locality().owned_key().assume())
            .map_err(std::mem::forget)
            .ok()
    }

    /// Adds bi edge to other.
    /// UNSAFE: Caller must ensure to add returned key to this Item edges as PartialEdge {object:key,side:Side::Bi}.
    #[must_use]
    pub unsafe fn add_bi<O, D: BiItem<O, T>>(
        &self,
        data: O,
        drain: &mut Slot<permit::Mut, D>,
    ) -> Key<Owned, D> {
        drain.localized(|item, locality| item.add_bi_edge(locality, data, self.owned_key()));
        drain.locality().owned_key()
    }

    // /// Removes edge from object.
    // /// Edge should have come from this Item edges.
    // /// Panics if edge is not in object.
    // pub fn remove_edge<D: StandaloneItem>(
    //     &self,
    //     edge: PartialEdge<Key<Owned, D>>,
    //     object: &mut Slot<permit::Mut, D>,
    // ) {
    //     let (object_key, edge) = edge.reverse(self.path().ptr());
    //     let subject_key = object.remove_edge(object_key, edge);
    //     std::mem::forget(subject_key);
    // }

    /// Removes this from drain.
    /// Edge should have come from this Item edges.
    /// Panics if edge is not in drain.
    pub fn remove_from_drain<D: DrainItem>(
        &self,
        drain_key: Key<Owned, D>,
        drain: &mut Slot<permit::Mut, D>,
    ) {
        drain
            .try_remove_drain_edge(drain_key, self.path().ptr())
            .expect("Method invariant broken");
    }

    // /// Removes edge from object.
    // /// Edge should have come from this Item edges.
    // /// Err if object can't remove it.
    // /// Panics if edge is not in object.
    // #[must_use]
    // pub fn try_remove_edge<D: Item>(
    //     &self,
    //     edge: PartialEdge<Key<Owned, D>>,
    //     object: &mut Slot<permit::Mut, D>,
    // ) -> Result<(), PartialEdge<Key<Owned, D>>> {
    //     let subject = edge.subject;
    //     let (object_key, edge) = edge.reverse(self.path().ptr());
    //     object
    //         .try_remove_edge(object_key, edge)
    //         .map(|subject_key| std::mem::forget(subject_key))
    //         .map_err(|(present, object_key)| {
    //             assert_eq!(present, Found::Yes);
    //             subject.object(object_key)
    //         })
    // }

    // /// Removes edge from object.
    // /// Edge should have come from this Item edges.
    // /// Err if object can't remove it.
    // /// Panics if edge is not in object.
    // #[must_use]
    // pub fn try_remove_dyn_edge<D: DynItem + ?Sized>(
    //     &self,
    //     edge: PartialEdge<Key<Owned, D>>,
    //     object: &mut DynSlot<permit::Mut, D>,
    // ) -> Result<(), PartialEdge<Key<Owned, D>>> {
    //     let subject = edge.subject;
    //     let (object_key, edge) = edge.reverse(self.path().ptr());
    //     object
    //         .remove_edge(object_key, edge)
    //         .map(|subject_key| std::mem::forget(subject_key))
    //         .map_err(|(present, object_key)| {
    //             assert_eq!(present, Found::Yes);
    //             subject.object(object_key)
    //         })
    // }

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
