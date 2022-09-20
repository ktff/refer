use crate::core::*;
use log::*;
use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashSet,
    marker::PhantomData,
    num::NonZeroU64,
};

pub type RefShellIter<'a, F: ?Sized + 'static> = impl Iterator<Item = Key<F>> + 'a;
pub type RefShellAnyIter<'a> = impl Iterator<Item = AnyKey> + 'a;

pub type SlotIter<'a, T: 'static> = impl Iterator<
    Item = (
        SubKey<T>,
        (&'a UnsafeCell<T>, &'a ()),
        &'a UnsafeCell<SizedShell<T>>,
        &'a std::alloc::Global,
    ),
>;

pub struct ItemContainerFamily;

impl ContainerFamily for ItemContainerFamily {
    type C<T: AnyItem> = ItemContainer<T>;

    fn new<T: AnyItem>(_: u32) -> Self::C<T> {
        ItemContainer::new()
    }
}

/// A collection of 1 item.
pub struct ItemContainer<T: 'static>(Slot<T>);

impl<T: 'static> ItemContainer<T> {
    pub fn new() -> Self {
        Self(Slot::Free)
    }

    const fn key() -> SubKey<T> {
        SubKey::new(1, Index(NonZeroU64::new(1).expect("Shouldn't be zero")))
    }
}

impl<T: 'static> Allocator<T> for ItemContainer<T> {
    type Alloc = std::alloc::Global;

    type R = ();

    fn reserve(&mut self, _: Option<&T>, _: Self::R) -> Option<(ReservedKey<T>, &Self::Alloc)> {
        match self.0 {
            Slot::Free => {
                self.0 = Slot::Reserved;
                Some((ReservedKey::new(Self::key()), &std::alloc::Global))
            }
            _ => None,
        }
    }

    fn cancel(&mut self, key: ReservedKey<T>) {
        self.0.cancel();
        key.take();
    }

    fn fulfill(&mut self, key: ReservedKey<T>, item: T) -> SubKey<T> {
        self.0.fulfill(item);

        key.take()
    }

    fn unfill(&mut self, _: SubKey<T>) -> Option<(T, &Self::Alloc)>
    where
        T: Sized,
    {
        self.0.unfill().map(|item| (item, &std::alloc::Global))
    }
}

impl<T: AnyItem> !Sync for ItemContainer<T> {}

impl<T: AnyItem> Container<T> for ItemContainer<T> {
    type GroupItem = ();

    type Shell = SizedShell<T>;

    type SlotIter<'a> = SlotIter<'a, T> where Self: 'a;

    fn get_slot(
        &self,
        _: SubKey<T>,
    ) -> Option<(
        (&UnsafeCell<T>, &()),
        &UnsafeCell<Self::Shell>,
        &Self::Alloc,
    )> {
        match &self.0 {
            Slot::Free => None,
            Slot::Reserved => {
                warn!("Reserved slot {:?} was accessed", Self::key());
                None
            }
            Slot::Filled { item, shell } => Some(((item, &()), shell, &std::alloc::Global)),
        }
    }

    unsafe fn iter_slot(&self) -> Option<Self::SlotIter<'_>> {
        // This is safe since we only return reference to a single slot.
        match &self.0 {
            Slot::Free => None,
            Slot::Reserved => {
                warn!("Reserved slot {:?} was accessed", Self::key());
                None
            }
            Slot::Filled { item, shell } => {
                Some(Some((Self::key(), (item, &()), shell, &std::alloc::Global)).into_iter())
            }
        }
    }
}

impl<T: AnyItem> AnyContainer for ItemContainer<T> {
    fn any_get_slot(
        &self,
        key: AnySubKey,
    ) -> Option<(
        (&UnsafeCell<dyn AnyItem>, &dyn Any),
        &UnsafeCell<dyn AnyShell>,
        &dyn std::alloc::Allocator,
    )> {
        key.downcast::<T>()?;
        match &self.0 {
            Slot::Free => None,
            Slot::Reserved => {
                warn!("Reserved slot {:?} was accessed", Self::key());
                None
            }
            Slot::Filled { item, shell } => Some(((item, &()), shell, &std::alloc::Global)),
        }
    }

    fn unfill_any(&mut self, key: AnySubKey) {
        if key.downcast::<T>().is_some() {
            self.0.unfill();
        }
    }

    fn first(&self, key: TypeId) -> Option<AnySubKey> {
        if key == TypeId::of::<T>() {
            if let Slot::Filled { .. } = &self.0 {
                Some(
                    SubKey::<T>::new(1, Index(NonZeroU64::new(1).expect("Shouldn't be zero")))
                        .into(),
                )
            } else {
                None
            }
        } else {
            None
        }
    }

    fn next(&self, _: AnySubKey) -> Option<AnySubKey> {
        None
    }

    fn types(&self) -> HashSet<TypeId> {
        let mut set = HashSet::new();
        set.insert(TypeId::of::<T>());
        set
    }
}

impl<T: 'static> Default for ItemContainer<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SizedShell<T: ?Sized + 'static> {
    from: Vec<AnyKey>,
    _data: PhantomData<T>,
}

impl<T: ?Sized + 'static> SizedShell<T> {
    pub fn new() -> Self {
        Self {
            from: Vec::new(),
            _data: PhantomData,
        }
    }
}

impl<T: ?Sized + 'static> AnyShell for SizedShell<T> {
    fn item_ty(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn from_any(&self) -> Box<dyn Iterator<Item = AnyKey> + '_> {
        Box::new(self.iter())
    }

    fn from_count(&self) -> usize {
        self.from.len()
    }

    fn add_from(&mut self, from: AnyKey, _: &impl std::alloc::Allocator) {
        self.from.push(from);
    }

    fn add_from_any(&mut self, from: AnyKey, _: &dyn std::alloc::Allocator) {
        self.from.push(from);
    }

    fn remove_from(&mut self, from: AnyKey) {
        // TODO: This will be really slow for large self.from
        if let Some((i, _)) = self
            .from
            .iter()
            .enumerate()
            .rev()
            .find(|(_, key)| key == &&from)
        {
            self.from.remove(i);
        }
    }
}

impl<T: ?Sized + 'static> Shell for SizedShell<T> {
    type T = T;
    type Iter<'a, F: ?Sized + 'static> = RefShellIter<'a, F>;
    type AnyIter<'a> = RefShellAnyIter<'a>;

    fn iter(&self) -> Self::AnyIter<'_> {
        self.from.iter().copied()
    }

    fn from<F: ?Sized + 'static>(&self) -> Self::Iter<'_, F> {
        self.iter().filter_map(AnyKey::downcast)
    }
}

impl<T: ?Sized + 'static> Default for SizedShell<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub enum Slot<T: 'static, S: Default = SizedShell<T>> {
    Free,
    Reserved,
    Filled {
        item: UnsafeCell<T>,
        shell: UnsafeCell<S>,
    },
}

impl<T: 'static, S: Default> Slot<T, S> {
    pub fn new(item: T) -> Self {
        Slot::Filled {
            item: UnsafeCell::new(item),
            shell: UnsafeCell::new(S::default()),
        }
    }

    pub fn reserve(&mut self) {
        debug_assert!(matches!(self, Slot::Free));
        *self = Slot::Reserved;
    }

    pub fn cancel(&mut self) {
        assert!(matches!(self, Slot::Reserved));
        *self = Slot::Free;
    }

    pub fn fulfill(&mut self, item: T) {
        assert!(matches!(self, Slot::Reserved));
        *self = Slot::new(item);
    }

    /// Frees and returns item if it exists
    pub fn unfill(&mut self) -> Option<T> {
        match std::mem::replace(self, Slot::Free) {
            Slot::Free => None,
            Slot::Reserved => {
                error!("Reserved Slot<{}> is unfilled", std::any::type_name::<T>());
                None
            }
            Slot::Filled { item, .. } => Some(item.into_inner()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::owned::Owned;
    use std::any::Any;

    #[test]
    fn allocate_item() {
        let mut container = Owned::new(ItemContainer::<usize>::new());

        let item = 42;
        let (key, _) = container.reserve(Some(&item), ()).unwrap();
        let key = container.fulfill(key, item).into_key();

        assert_eq!(container.items().get(key), Some((&item, &())));
        assert!(container.reserve(Some(&item), ()).is_none());
    }

    #[test]
    fn allocate_cancel() {
        let mut container = Owned::new(ItemContainer::<usize>::new());

        let item = 42;
        let (rkey, _) = container.reserve(Some(&item), ()).unwrap();
        let key = rkey.key().into_key();
        container.cancel(rkey);

        assert_eq!(container.items().get(key), None);
        assert!(container.reserve(Some(&item), ()).is_some());
    }

    #[test]
    fn allocate_unfill() {
        let mut container = Owned::new(ItemContainer::<usize>::new());

        let item = 42;
        let (key, _) = container.reserve(Some(&item), ()).unwrap();
        let key = container.fulfill(key, item).into_key();

        assert_eq!(container.items().get(key), Some((&item, &())));
        assert_eq!(container.unfill(key.into()), Some(item));
        assert_eq!(container.items().get(key), None);
        assert!(container.reserve(Some(&item), ()).is_some());
    }

    #[test]
    fn iter() {
        let mut container = Owned::new(ItemContainer::<usize>::new());

        let item = 42;
        let key = container.add_with(item, ()).unwrap();

        assert_eq!(container.items().iter().count(), 1);
        assert_eq!(
            container.items().iter().next().unwrap(),
            (key, (&item, &()))
        );
    }

    #[test]
    fn get_any() {
        let mut container = Owned::new(ItemContainer::<usize>::new());

        let item = 42;
        let key = container.add_with(item, ()).unwrap();

        assert_eq!(
            (container.items_mut().get_any(key.into()).unwrap().0 as &dyn Any)
                .downcast_ref::<usize>(),
            Some(&item)
        );
    }

    #[test]
    fn unfill_any() {
        let mut container = ItemContainer::<usize>::new();

        let item = 42;
        let (key, _) = container.reserve(Some(&item), ()).unwrap();
        let key = container.fulfill(key, item);

        container.unfill_any(key.into());
        assert!(container.get_slot(key.into()).is_none());
    }

    #[test]
    fn iter_keys() {
        let mut container = Owned::new(ItemContainer::<usize>::new());

        let item = 42;
        let key = container.add_with(item, ()).unwrap();

        let k = container.first(key.type_id());
        assert_eq!(k, Some(key.into()));
        assert!(container.next(k.unwrap()).is_none());
    }

    #[test]
    fn shell() {
        let mut container = Owned::new(ItemContainer::<usize>::new());

        let item = 42;
        let key = container.add_with(item, ()).unwrap();

        let mut shells = container.shells_mut();
        let (shell, alloc) = shells.get_mut(key).unwrap();
        shell.add_from(key.into(), alloc);

        assert_eq!(shell.from_count(), 1);
        assert_eq!(shell.from::<usize>().collect::<Vec<_>>(), vec![key]);
        assert_eq!(shell.from_any().collect::<Vec<_>>(), vec![key.into()]);
        shell.remove_from(key.into());

        assert_eq!(shell.from_count(), 0);
        assert_eq!(shell.from::<usize>().collect::<Vec<_>>(), vec![]);
    }
}
