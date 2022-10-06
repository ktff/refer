use std::{any::TypeId, marker::PhantomData};

use crate::*;

// TODO: Add unreferable item which can have zero sized shell and static guarantee that nothing will have reference to it.

/// No shell.
/// Panics if anything tries to add reference.
#[derive(Debug, Clone, Copy)]
pub struct NoShell<T: AnyItem>(PhantomData<T>);

/// A shell of an item. In which references are recorded.
impl<T: AnyItem> Shell for NoShell<T> {
    type T = T;
    type Iter<'a, F: ?Sized + 'static> = impl Iterator<Item = Key<F>> + 'a
    where
        Self: 'a;
    type AnyIter<'a>= impl Iterator<Item = AnyKey> + 'a
    where
        Self: 'a;

    fn from<F: ?Sized + 'static>(&self) -> Self::Iter<'_, F> {
        None.into_iter()
    }

    fn iter(&self) -> Self::AnyIter<'_> {
        None.into_iter()
    }
}
impl<T: AnyItem> AnyShell for NoShell<T> {
    fn item_ty(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn from_any(&self) -> Box<dyn Iterator<Item = AnyKey> + '_> {
        Box::new(None.into_iter())
    }

    fn from_count(&self) -> usize {
        0
    }

    fn add_from(&mut self, from: AnyKey, _: &impl std::alloc::Allocator)
    where
        Self: Sized,
    {
        panic!("NoShell::add_from called for {:?}", from);
    }

    fn add_from_any(&mut self, from: AnyKey, _: &dyn std::alloc::Allocator) {
        panic!("NoShell::add_from_any called for {:?}", from);
    }

    fn remove_from(&mut self, _: AnyKey) {}
}

/// Default
impl<T: AnyItem> Default for NoShell<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
