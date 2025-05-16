use super::{
    permit::{self},
    AnyDynItem, AnyItem, BiItem, DrainItem, DynItem, Item, Key, KeyPath, LeafPath, LocalityKey,
    LocalityPath, Owned, Path, Ref, Slot,
};
use getset::{CopyGetters, Getters};
use std::num::NonZeroUsize;

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

    pub fn container_locality(&self) -> LocalityRef<'_, KeyPath<T>, T> {
        LocalityRef::new(self.locality_key.into(), &self.data, &self.allocator)
    }
}

pub type ItemLocality<'a, T: DynItem + ?Sized = dyn AnyItem> = LocalityRef<'a, Key<Ref<'a>, T>, T>;
pub type ContainerLocality<'a, T: DynItem + ?Sized = dyn AnyItem> = LocalityRef<'a, KeyPath<T>, T>;

#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct LocalityRef<'a, K: LocalityPath + Copy = Path, T: DynItem + ?Sized = dyn AnyItem> {
    path: K,
    data: &'a T::AnyLocalityData,
    allocator: &'a T::AnyAlloc,
}

impl<'a, K: LocalityPath + Copy, T: DynItem + ?Sized> LocalityRef<'a, K, T> {
    pub fn new(path: K, data: &'a T::AnyLocalityData, allocator: &'a T::AnyAlloc) -> Self {
        Self {
            path,
            data,
            allocator,
        }
    }
}

impl<'a, T: DynItem + ?Sized> LocalityRef<'a, Key<Ref<'a>, T>, T> {
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
        drain.localized(|item, locality| item.add_drain_edge(locality, self.owned_key().any()));
        drain.locality().owned_key()
    }

    /// Adds drain edge to drain.
    /// UNSAFE: Caller must ensure to add returned key to this(source) Item edges as PartialEdge {object:key,side:Side::Source}.
    #[must_use]
    pub unsafe fn add_dyn_drain<D: DynItem + ?Sized>(
        &self,
        drain: &mut Slot<permit::Mut, D>,
    ) -> Option<Key<Owned, D>> {
        drain
            .any_localized(|item, locality| {
                item.any_add_drain_edge(locality, self.owned_key().any())
            })
            .map(|()| drain.locality().owned_key().any().assume())
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

    /// Removes self from drain.
    /// Edge should have come from self Item edges.
    /// Panics if edge is not in drain.
    pub fn remove_from_drain<D: DrainItem>(
        &self,
        drain: &mut Slot<permit::Mut, D>,
        drain_key: Key<Owned, D>,
    ) {
        drain
            .try_remove_drain_edge(drain_key, self.path().ptr())
            .expect("Method invariant broken");
    }

    /// Removes self from other.
    /// Panics if edge is not in other.
    pub fn remove_bi_edge<R, D: BiItem<R, T>>(
        &self,
        data: R,
        other: &mut Slot<permit::Mut, D>,
        other_key: Key<Owned, D>,
    ) {
        std::mem::forget(other_key);
        let owned = other
            .localized(|item, locality| item.try_remove_bi_edge(locality, data, self.path().ptr()));
        assert!(owned.is_some(), "BI edge should be present in both items");
        std::mem::forget(owned);
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
}

impl<'a, T: DynItem + ?Sized> LocalityRef<'a, Key<Ref<'a>, T>, T> {
    pub fn any(self) -> ItemLocality<'a> {
        ItemLocality {
            path: self.path.any(),
            data: T::as_any_locality_data(self.data),
            allocator: T::as_any_alloc(self.allocator),
        }
    }
}

impl<'a, T: DynItem + ?Sized> LocalityRef<'a, KeyPath<T>, T> {
    pub fn any(self) -> LocalityRef<'a> {
        LocalityRef {
            path: self.path.path(),
            data: T::as_any_locality_data(self.data),
            allocator: T::as_any_alloc(self.allocator),
        }
    }
}

impl<'a, T: AnyDynItem + ?Sized> ItemLocality<'a, T> {
    pub fn downcast<D: Item>(self) -> Option<ItemLocality<'a, D>> {
        if let Some(data) = self.data.downcast_ref() {
            Some(ItemLocality {
                path: self.path.any().assume(),
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

    pub fn sidecast<
        D: DynItem<AnyLocalityData = T::AnyLocalityData, AnyAlloc = T::AnyAlloc> + ?Sized,
    >(
        self,
    ) -> ItemLocality<'a, D> {
        ItemLocality {
            path: self.path.any().assume(),
            ..self
        }
    }
}

impl<'a, T: AnyDynItem + ?Sized> LocalityRef<'a, Path, T> {
    pub fn downcast<D: Item>(self) -> Option<LocalityRef<'a, KeyPath<D>, D>> {
        if let Some(data) = self.data.downcast_ref() {
            Some(LocalityRef {
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

impl<'a, T: Item> ItemLocality<'a, T> {
    /// Casting for a sub struct of this item.
    pub fn sub_cast<D: Item<Alloc = T::Alloc, LocalityData = T::LocalityData>>(
        self,
    ) -> ItemLocality<'a, D> {
        ItemLocality {
            path: self.path.any().assume(),
            ..self
        }
    }
}

impl<'a, K: LocalityPath + Copy, T: DynItem + ?Sized> Copy for LocalityRef<'a, K, T> {}

impl<'a, K: LocalityPath + Copy, T: DynItem + ?Sized> Clone for LocalityRef<'a, K, T> {
    fn clone(&self) -> Self {
        Self {
            path: self.path,
            data: self.data,
            allocator: self.allocator,
        }
    }
}
