use super::*;
use crate::core::{Container, KeyPath};
use std::ops::Deref;

pub struct PathPermit<'a, T: core::Item, R, A, C> {
    permit: TypePermit<'a, T, R, A, C>,
    path: KeyPath<T>,
}

impl<'a, R, T: core::Item, A, C: Container<T>> PathPermit<'a, T, R, A, C> {
    pub fn new(permit: TypePermit<'a, T, R, A, C>, path: KeyPath<T>) -> Self {
        Self { permit, path }
    }

    pub fn path(&self) -> KeyPath<T> {
        self.path
    }

    pub fn iter(self) -> impl Iterator<Item = core::Slot<'a, T, C::Shell, R, A>> {
        let Self { permit, path } = self;
        permit
            .iter_slot(path)
            .into_iter()
            .flat_map(|iter| iter)
            // SAFETY: Type level logic of Permit ensures that it has sufficient access for 'a to all slots of T under path.
            .map(move |(key, slot)| unsafe { core::Slot::new(key, slot, permit.access()) })
    }

    /// Splits on lower level, or returns self if level is higher.
    pub fn split_level(
        self,
        level: u32,
    ) -> Box<dyn ExactSizeIterator<Item = PathPermit<'a, T, R, A, C>> + 'a>
    where
        R: 'static,
        A: 'static,
    {
        if let Some(iter) = self.path.iter_level(level) {
            Box::new(
                // SAFETY: We depend on iter_level returning disjoint paths.
                iter.map(move |path| unsafe {
                    self.permit.unsafe_split(|permit| permit.path(path))
                }),
            )
        } else {
            Box::new(std::iter::once(self))
        }
    }
}

impl<'a, T: core::Item, A, C: Container<T>> PathPermit<'a, T, Mut, A, C> {
    pub fn borrow(&self) -> PathPermit<T, Ref, A, C> {
        PathPermit {
            permit: self.permit.borrow(),
            path: self.path,
        }
    }

    pub fn borrow_mut(&mut self) -> PathPermit<T, Mut, A, C> {
        PathPermit {
            permit: self.permit.borrow_mut(),
            path: self.path,
        }
    }
}

impl<'a, T: core::Item, R, A, C: Container<T>> Deref for PathPermit<'a, T, R, A, C> {
    type Target = &'a C;

    fn deref(&self) -> &Self::Target {
        &self.permit
    }
}

impl<'a, T: core::Item, A, C> Copy for PathPermit<'a, T, Ref, A, C> {}

impl<'a, T: core::Item, A, C> Clone for PathPermit<'a, T, Ref, A, C> {
    fn clone(&self) -> Self {
        Self {
            permit: self.permit,
            path: self.path,
        }
    }
}
