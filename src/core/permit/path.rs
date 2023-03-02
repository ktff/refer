use super::*;
use crate::core::{
    container::RegionContainer, container::TypeContainer, AnyContainer, Container, DynItem, Path,
};
use std::{
    any::TypeId,
    ops::{Bound, Deref, RangeBounds},
};

pub struct PathPermit<'a, T: DynItem + ?Sized, R, C: AnyContainer + ?Sized> {
    permit: TypePermit<'a, T, R, C>,
    path: Path,
}

impl<'a, R, T: DynItem + ?Sized, C: AnyContainer + ?Sized> PathPermit<'a, T, R, C> {
    pub fn new(permit: TypePermit<'a, T, R, C>) -> Self {
        Self {
            path: permit.container_path(),
            permit,
        }
    }

    /// Panics if the path is not a subpath of container.
    pub fn new_with(permit: TypePermit<'a, T, R, C>, path: impl Into<Path>) -> Self {
        let path = path.into();
        assert!(permit.container_path().contains(path));
        Self { path, permit }
    }

    pub fn path(&self) -> Path {
        self.path
    }

    /// Constrains the permit to the given path.
    /// None if they don't overlap.
    pub fn and(self, path: impl Into<Path>) -> Option<Self> {
        let Self { permit, path: p } = self;
        let path = p.and(path)?;
        Some(Self { permit, path })
    }

    // TODO
    // /// Iterates over valid keys in ascending order of types that have T as Item trait.
    // pub fn iter_dyn(self) -> Result<impl Iterator<Item = core::DynSlot<'a, T, R, A>>> {
    //     unimplemented!()
    // }

    // TODO
    // pub fn split_level_dyn
}

impl<'a, R, T: core::Item, C: Container<T> + ?Sized> PathPermit<'a, T, R, C> {
    pub fn iter(self) -> impl Iterator<Item = core::Slot<'a, T, R>> {
        let Self { permit, path } = self;
        assert!(permit.container_path().contains(path));
        permit
            .iter_slot(path.of())
            .into_iter()
            .flat_map(|iter| iter)
            // SAFETY: Type level logic of Permit ensures that it has sufficient access for 'a to all slots of T under path.
            .map(move |(key, slot)| unsafe { core::Slot::new(key.ptr(), slot, permit.access()) })
    }

    /// Splits on lower level, or returns self if level is higher.
    pub fn split_level(
        self,
        level: u32,
    ) -> Box<dyn ExactSizeIterator<Item = PathPermit<'a, T, R, C>> + 'a>
    where
        R: 'static,
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
            .and(self.path)
            .expect("Path out of Container path")
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

impl<'a, R, T: core::Item, C: Container<T> + ?Sized> PathPermit<'a, T, R, C> {
    pub fn step(self) -> Option<PathPermit<'a, T, R, C::Sub>>
    where
        C: TypeContainer<T>,
    {
        let Self { permit, path } = self;
        permit.step().map(|permit| PathPermit { permit, path })
    }
}

impl<'a, R, T: DynItem + ?Sized, C: AnyContainer + ?Sized> PathPermit<'a, T, R, C> {
    pub fn step_into(self, index: usize) -> Option<PathPermit<'a, T, R, C::Sub>>
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
    ) -> Option<impl Iterator<Item = PathPermit<'a, T, R, C::Sub>>>
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

impl<'a, T: DynItem + ?Sized, C: AnyContainer + ?Sized> PathPermit<'a, T, Mut, C> {
    pub fn borrow(&self) -> PathPermit<T, Ref, C> {
        PathPermit {
            permit: self.permit.borrow(),
            path: self.path,
        }
    }

    pub fn borrow_mut(&mut self) -> PathPermit<T, Mut, C> {
        PathPermit {
            permit: self.permit.borrow_mut(),
            path: self.path,
        }
    }
}

impl<'a, R, T: core::Item, C: Container<T> + ?Sized> IntoIterator for PathPermit<'a, T, R, C> {
    type Item = core::Slot<'a, T, R>;
    type IntoIter = impl Iterator<Item = core::Slot<'a, T, R>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: DynItem + ?Sized, R, C: AnyContainer + ?Sized> Deref for PathPermit<'a, T, R, C> {
    type Target = &'a C;

    fn deref(&self) -> &Self::Target {
        &self.permit
    }
}

impl<'a, T: DynItem + ?Sized, C: AnyContainer + ?Sized> Copy for PathPermit<'a, T, Ref, C> {}

impl<'a, T: DynItem + ?Sized, C: AnyContainer + ?Sized> Clone for PathPermit<'a, T, Ref, C> {
    fn clone(&self) -> Self {
        Self {
            permit: self.permit,
            path: self.path,
        }
    }
}

impl<'a, R, T: DynItem + ?Sized, C: AnyContainer + ?Sized> From<TypePermit<'a, T, R, C>>
    for PathPermit<'a, T, R, C>
{
    fn from(permit: TypePermit<'a, T, R, C>) -> Self {
        Self::new(permit)
    }
}
