use crate::core::*;
use std::{
    any::TypeId, cell::UnsafeCell, collections::HashSet, marker::PhantomData, num::NonZeroU64,
};

pub type RefShellIter<'a, F: ?Sized + 'static> = impl Iterator<Item = Key<F>> + 'a;
pub type RefShellAnyIter<'a> = impl Iterator<Item = AnyKey> + 'a;

pub type SlotIter<'a, T: 'static> =
    impl Iterator<Item = (SubKey<T>, &'a UnsafeCell<T>, &'a UnsafeCell<SizedShell<T>>)>;

pub struct ItemContainerFamily;

impl ContainerFamily for ItemContainerFamily {
    type C<T: AnyItem> = ItemContainer<T>;

    fn new<T: AnyItem>(_: u32) -> Self::C<T> {
        ItemContainer::new()
    }
}

/// A collection of 1 sized item.
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
    fn reserve(&mut self, _: &T) -> Option<ReservedKey<T>> {
        match self.0 {
            Slot::Free => {
                self.0 = Slot::Reserved;
                Some(ReservedKey::new(Self::key()))
            }
            _ => None,
        }
    }

    fn cancel(&mut self, _: ReservedKey<T>) {
        self.0.cancel();
    }

    fn fulfill(&mut self, _: ReservedKey<T>, item: T) -> SubKey<T> {
        self.0.fulfill(item);

        Self::key()
    }

    /// Frees and returns item if it exists
    fn unfill(&mut self, _: SubKey<T>) -> Option<T>
    where
        T: Sized,
    {
        self.0.unfill()
    }
}

impl<T: AnyItem> !Sync for ItemContainer<T> {}

impl<T: AnyItem> Container<T> for ItemContainer<T> {
    type Shell = SizedShell<T>;

    type SlotIter<'a> = SlotIter<'a, T> where Self: 'a;

    fn get_slot(&self, _: SubKey<T>) -> Option<(&UnsafeCell<T>, &UnsafeCell<Self::Shell>)> {
        match &self.0 {
            Slot::Free => None,
            Slot::Reserved => panic!("Reserved slot"),
            Slot::Filled { item, shell } => Some((item, shell)),
        }
    }

    unsafe fn iter_slot(&self) -> Option<Self::SlotIter<'_>> {
        // This is safe since we only return reference to a single slot.
        match &self.0 {
            Slot::Free => None,
            Slot::Reserved => panic!("Reserved slot"),
            Slot::Filled { item, shell } => Some(Some((Self::key(), item, shell)).into_iter()),
        }
    }
}

impl<T: AnyItem> AnyContainer for ItemContainer<T> {
    fn any_get_slot(
        &self,
        key: AnySubKey,
    ) -> Option<(&UnsafeCell<dyn AnyItem>, &UnsafeCell<dyn AnyShell>)> {
        key.downcast::<T>()?;
        match &self.0 {
            Slot::Free => None,
            Slot::Reserved => panic!("Reserved slot"),
            Slot::Filled { item, shell } => Some((item, shell)),
        }
    }

    fn any_unfill(&mut self, key: AnySubKey) -> bool {
        key.downcast::<T>().is_some() && self.0.unfill().is_some()
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

    fn add_from(&mut self, from: AnyKey) {
        self.from.push(from);
    }

    fn remove_from(&mut self, from: AnyKey) -> bool {
        // TODO: This will be really slow for large froms.
        if let Some((i, _)) = self
            .from
            .iter()
            .enumerate()
            .rev()
            .find(|(_, key)| key == &&from)
        {
            self.from.remove(i);
            true
        } else {
            false
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

pub enum Slot<T: 'static> {
    Free,
    Reserved,
    Filled {
        item: UnsafeCell<T>,
        shell: UnsafeCell<SizedShell<T>>,
    },
}

impl<T: 'static> Slot<T> {
    pub fn new(item: T) -> Self {
        Slot::Filled {
            item: UnsafeCell::new(item),
            shell: UnsafeCell::new(SizedShell::new()),
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
            Slot::Reserved => panic!("Reserved slot is unfilled"),
            Slot::Filled { item, .. } => Some(item.into_inner()),
        }
    }
}
