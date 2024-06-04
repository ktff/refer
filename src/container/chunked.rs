use crate::{
    core::{container::*, *},
    region_container,
};
use std::{
    any::TypeId,
    collections::HashMap,
    ops::{Bound, Range, RangeBounds},
};

pub type Iter<'a, C: AnyContainer> = impl DoubleEndedIterator<Item = &'a C> + Send + 'a;

pub type IterMut<'a, C: AnyContainer> = impl DoubleEndedIterator<Item = &'a mut C> + Send + 'a;

/// A container that chunks items into separate containers according to items locality key.
pub struct VecChunkedContainer<C: AnyContainer> {
    region: RegionPath,
    builder: Box<dyn FnMut(&dyn LocalityPath, Path) -> Option<C> + Send + Sync>,
    /// Maps Locality IDs to path indices.
    map: HashMap<(TypeId, usize), usize>,
    chunks: Vec<C>,
}

impl<C: AnyContainer> VecChunkedContainer<C> {
    /// Builder should build for given locality path and path in this region Container C, or return None if it can do so for given locality path.
    pub fn new(
        region: RegionPath,
        builder: impl FnMut(&dyn LocalityPath, Path) -> Option<C> + Send + Sync + 'static,
    ) -> Self {
        Self {
            region,
            builder: Box::new(builder) as Box<_>,
            map: HashMap::new(),
            chunks: Vec::new(),
        }
    }

    fn iter_slice<'a>(&'a self, start: Bound<usize>, end: Bound<usize>) -> Option<Iter<'a, C>> {
        let range = self.normalize_bounds(start, end)?;
        Some(self.chunks[range].iter())
    }
    fn iter_slice_mut<'a>(
        &'a mut self,
        start: Bound<usize>,
        end: Bound<usize>,
    ) -> Option<IterMut<'a, C>> {
        let range = self.normalize_bounds(start, end)?;
        Some(self.chunks[range].iter_mut())
    }

    fn normalize_bounds(&self, start: Bound<usize>, end: Bound<usize>) -> Option<Range<usize>> {
        let start = match start {
            Bound::Included(start) => start,
            Bound::Excluded(start) => start.checked_add(1)?,
            Bound::Unbounded => 0,
        };
        let end = match end {
            Bound::Included(end) => end.saturating_add(1).min(self.chunks.len()),
            Bound::Excluded(end) => end.min(self.chunks.len()),
            Bound::Unbounded => self.chunks.len(),
        };
        if start >= end {
            None
        } else {
            Some(start..end)
        }
    }
}

unsafe impl<C: AnyContainer> RegionContainer for VecChunkedContainer<C> {
    type Sub = C;

    type Iter<'a> = Iter<'a,C>
    where
        Self: 'a;

    type IterMut<'a> = IterMut<'a, C>
    where
        Self: 'a;

    #[inline(always)]
    fn region(&self) -> RegionPath {
        self.region
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<&Self::Sub> {
        self.chunks.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Sub> {
        self.chunks.get_mut(index)
    }

    fn iter(&self, range: impl RangeBounds<usize>) -> Option<Self::Iter<'_>> {
        self.iter_slice(range.start_bound().cloned(), range.end_bound().cloned())
    }

    fn iter_mut(&mut self, range: impl RangeBounds<usize>) -> Option<Self::IterMut<'_>> {
        self.iter_slice_mut(range.start_bound().cloned(), range.end_bound().cloned())
    }

    fn locality<P: LocalityPath + ?Sized>(&self, key: &P) -> Option<&Self::Sub> {
        match key.map(self.region)? {
            LocalityRegion::Id(id) => self.get(*self.map.get(&id)?),
            LocalityRegion::Index(index) => self.get(index),
            LocalityRegion::Any => self.get(0),
            LocalityRegion::Indices(range) if range.start() <= range.end() => {
                self.get(*range.start())
            }
            LocalityRegion::Indices(_) => None,
        }
    }

    fn fill<P: LocalityPath + ?Sized>(&mut self, key: &P) -> Option<&mut Self::Sub> {
        match key.map(self.region)? {
            LocalityRegion::Id(id) => {
                if let Some(&index) = self.map.get(&id) {
                    Some(&mut self.chunks[index])
                } else {
                    let index = self.chunks.len();
                    let container = (self.builder)(key.upcast(), self.region.path_of(index))?;
                    self.chunks.push(container);
                    self.map.insert(id, index);
                    self.chunks.last_mut()
                }
            }
            LocalityRegion::Index(index) => self.get_mut(index),
            LocalityRegion::Any => self.get_mut(0),
            LocalityRegion::Indices(range) if range.start() <= range.end() => {
                self.get_mut(*range.start())
            }
            LocalityRegion::Indices(_) => None,
        }
    }
}

unsafe impl<C: Container<T>, T: Item> Container<T> for VecChunkedContainer<C> {
    region_container!(impl Container<T>);
}

unsafe impl<C: AnyContainer> AnyContainer for VecChunkedContainer<C> {
    region_container!(impl AnyContainer);

    /// All types in the container.
    fn types(&self) -> HashMap<TypeId, ItemTraits> {
        //? NOTE: This will be slow for many chunks but that's fine for now.
        self.chunks
            .iter()
            .flat_map(|c| c.types().into_iter())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::VecContainer;
    use std::{any::Any, num::NonZeroU32};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    struct SpaceId(usize);

    impl LocalityPath for SpaceId {
        fn map(&self, _: RegionPath) -> Option<LocalityRegion> {
            Some(LocalityRegion::Id((self.type_id(), self.0)))
        }

        fn upcast(&self) -> &dyn LocalityPath {
            self
        }
    }

    fn container() -> VecChunkedContainer<VecContainer<usize>> {
        VecChunkedContainer::new(
            Path::default().region(NonZeroU32::new(8).unwrap()).unwrap(),
            |_: &dyn LocalityPath, path| {
                Some(VecContainer::new(Locality::new_default(
                    path.leaf().unwrap(),
                )))
            },
        )
    }

    #[test]
    fn add_items() {
        let n = 20;
        let mut container = container();
        let mut access = container.as_add();

        let keys = (0..n)
            .map(|i| access.add(&SpaceId(i), i).unwrap())
            .collect::<Vec<_>>();

        for (i, key) in keys.iter().enumerate() {
            assert_eq!(access.as_ref().key(*key).fetch().item(), &i);
        }
    }

    #[test]
    fn iter() {
        let n = 20;
        let mut container = container();
        let mut access = container.as_add();

        let mut keys = (0..n)
            .map(|i| (access.add(&SpaceId(i), i).unwrap(), i))
            .collect::<Vec<_>>();

        keys.sort();

        assert_eq!(
            keys,
            access
                .as_ref()
                .ty()
                .into_iter()
                .map(|slot| (slot.key(), *slot.item()))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn get_any() {
        let mut container = container();
        let mut access = container.as_add();

        let item = 42;
        let key = access.add(&SpaceId(item), item).unwrap();

        assert_eq!(
            (access.as_ref().key(key.any()).fetch().item() as &dyn Any).downcast_ref::<usize>(),
            Some(&item)
        );
    }

    #[test]
    fn unfill_any() {
        let mut container = container();

        let item = 42;
        let key = container.as_add().add(&SpaceId(item), item).unwrap().ptr();

        container.localized_drop(key.any().ptr());
        assert!(container.get_slot(key.ptr()).is_none());
    }
}
