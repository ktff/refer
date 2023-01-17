use super::*;
use crate::core::{
    region::RegionContainer, ty::TypeContainer, AnyContainer, Container, KeyPath, Path, Result,
};
use std::{
    any::TypeId,
    ops::{Bound, Deref, RangeBounds},
};

pub struct PathPermit<'a, T: core::Item, R, A, C: ?Sized> {
    permit: TypePermit<'a, T, R, A, C>,
    path: KeyPath<T>,
}

impl<'a, R, T: core::Item, A, C: Container<T> + ?Sized> PathPermit<'a, T, R, A, C> {
    pub fn new(permit: TypePermit<'a, T, R, A, C>) -> Self {
        Self {
            path: permit.container_path().of(),
            permit,
        }
    }

    /// Panics if the path is not a subpath of container.
    pub fn new_with(permit: TypePermit<'a, T, R, A, C>, path: KeyPath<T>) -> Self {
        assert!(permit.container_path().of().contains(path));
        Self { path, permit }
    }

    pub fn path(&self) -> KeyPath<T> {
        self.path
    }

    /// Constrains the permit to the given path.
    /// None if they don't overlap.
    pub fn and(self, path: impl Into<Path>) -> Option<Self> {
        let Self { permit, path: p } = self;
        let path = p.and(path)?;
        Some(Self { permit, path })
    }

    pub fn iter(self) -> Result<impl Iterator<Item = core::Slot<'a, T, C::Shell, R, A>>> {
        let Self { permit, path } = self;
        assert!(permit.container_path().of().contains(path));
        Ok(permit
            .iter_slot(path)
            .into_iter()
            .flat_map(|iter| iter)
            // SAFETY: Type level logic of Permit ensures that it has sufficient access for 'a to all slots of T under path.
            .map(move |(key, slot)| unsafe { core::Slot::new(key, slot, permit.access()) }))
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
        // Compute common path of all keys in the iterator.
        let first = self.first_key(TypeId::of::<T>());
        let last = self.last_key(TypeId::of::<T>());
        let path = match (first, last) {
            (Some(first), Some(last)) => first.path().or(last.path()),
            (Some(_), None) | (None, Some(_)) => unreachable!(),
            // There is no slots to iterate so we can return self.
            (None, None) => return Box::new(std::iter::once(self)),
        };

        if let Some(iter) = path
            .and(self.path.path())
            .expect("Path out of Container path")
            .of()
            .iter_level(level)
        {
            Box::new(
                // SAFETY: We depend on iter_level returning disjoint paths.
                iter.map(move |path| unsafe {
                    self.permit.unsafe_split(|permit| Self { permit, path })
                }),
            )
        } else {
            Box::new(std::iter::once(self))
        }
    }
}

impl<'a, R, T: core::Item, A, C: Container<T> + ?Sized> PathPermit<'a, T, R, A, C> {
    pub fn step(self) -> Option<PathPermit<'a, T, R, A, C::Sub>>
    where
        C: TypeContainer<T>,
    {
        let Self { permit, path } = self;
        permit.step().map(|permit| PathPermit { permit, path })
    }

    pub fn step_into(self, index: usize) -> Option<PathPermit<'a, T, R, A, C::Sub>>
    where
        C: RegionContainer,
    {
        let path = self.region().path_of(index);
        let Self { permit, path } = self.and(path)?;
        permit
            .step_into(index)
            .map(|permit| PathPermit { permit, path })
    }

    pub fn step_range(
        self,
        range: impl RangeBounds<usize>,
    ) -> Option<impl Iterator<Item = PathPermit<'a, T, R, A, C::Sub>>>
    where
        C: RegionContainer,
    {
        let path_range = self
            .region()
            .range_of(self.path)
            .expect("Path out of Container path");

        // Intersect ranges, max start bound, min end bound.
        let start = (*path_range.start()).max(match range.start_bound() {
            Bound::Included(bound) => *bound,
            Bound::Excluded(bound) => bound.checked_add(1)?,
            Bound::Unbounded => 0,
        });
        let end = (*path_range.end()).min(match range.end_bound() {
            Bound::Included(bound) => *bound,
            Bound::Excluded(bound) => bound.checked_sub(1)?,
            Bound::Unbounded => usize::MAX,
        });
        let range = start..=end;

        let Self { permit, path } = self;
        permit.step_range(range).map(|iter| {
            iter.filter_map(move |permit| {
                Some(PathPermit {
                    path: path.and(permit.container_path())?,
                    permit,
                })
            })
        })
    }
}

impl<'a, T: core::Item, A, C: Container<T> + ?Sized> PathPermit<'a, T, Mut, A, C> {
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

impl<'a, T: core::Item, R, A, C: ?Sized> Deref for PathPermit<'a, T, R, A, C> {
    type Target = &'a C;

    fn deref(&self) -> &Self::Target {
        &self.permit
    }
}

impl<'a, T: core::Item, A, C: ?Sized> Copy for PathPermit<'a, T, Ref, A, C> {}

impl<'a, T: core::Item, A, C: ?Sized> Clone for PathPermit<'a, T, Ref, A, C> {
    fn clone(&self) -> Self {
        Self {
            permit: self.permit,
            path: self.path,
        }
    }
}

impl<'a, R, T: core::Item, A, C: Container<T> + ?Sized> From<TypePermit<'a, T, R, A, C>>
    for PathPermit<'a, T, R, A, C>
{
    fn from(permit: TypePermit<'a, T, R, A, C>) -> Self {
        Self::new(permit)
    }
}