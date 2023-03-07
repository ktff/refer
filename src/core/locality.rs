use super::{Item, Key, KeyPath, LeafPath, LocalityKey, LocalityPath, Path, Ref};
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
    pub fn downcast<T: Item>(self) -> ItemLocality<'a, T> {
        self.downcast_try().expect("Unexpected item type")
    }

    pub fn downcast_try<T: Item>(self) -> Option<ItemLocality<'a, T>> {
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
    pub fn downcast<T: Item>(self) -> ContainerLocality<'a, T> {
        self.downcast_try().expect("Unexpected item type")
    }

    pub fn downcast_try<T: Item>(self) -> Option<ContainerLocality<'a, T>> {
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
