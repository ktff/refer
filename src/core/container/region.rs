use super::*;
use std::ops::RangeBounds;

/// UNSAFE: Implementations MUST follow get and iter SAFETY contracts.
pub unsafe trait RegionContainer {
    type Sub: AnyContainer;

    type Iter<'a>: DoubleEndedIterator<Item = &'a Self::Sub> + Send
    where
        Self: 'a;

    type IterMut<'a>: DoubleEndedIterator<Item = &'a mut Self::Sub> + Send
    where
        Self: 'a;

    /// Implementations should have #[inline(always)]
    fn region(&self) -> RegionPath;

    /// Implementations should have #[inline(always)]
    /// SAFETY: Bijection between index and container MUST be enforced.
    fn get(&self, index: usize) -> Option<&Self::Sub>;

    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Sub>;

    /// Iterates in ascending order for indices in range.
    /// SAFETY: Iterator MUST NOT return the same container more than once.
    fn iter(&self, range: impl RangeBounds<usize>) -> Option<Self::Iter<'_>>;

    /// Iterates in ascending order for indices in range.
    fn iter_mut(&mut self, range: impl RangeBounds<usize>) -> Option<Self::IterMut<'_>>;

    /// Index of locality
    fn locality<P: LocalityPath + ?Sized>(&self, key: &P) -> Option<&Self::Sub>;

    /// Index of locality, None if no more space
    fn fill<P: LocalityPath + ?Sized>(&mut self, key: &P) -> Option<&mut Self::Sub>;
}

// *************************** Blankets ***************************

#[macro_export]
macro_rules! region_container {
    (impl Container<$t:ty> ) => {
        type SlotIter<'a>
            = impl Iterator<Item = UnsafeSlot<'a, T>> + Send
        where
            Self: 'a;

        fn get_locality(&self, key: &impl LocalityPath) -> Option<ContainerLocality<$t>> {
            self.locality(key)?.get_locality(key)
        }

        fn iter_slot(&self, path: KeyPath<$t>) -> Option<Self::SlotIter<'_>> {
            let range = self.region().range_of(path)?;
            Some(
                self.iter(range)?
                    .filter_map(move |container| container.iter_slot(path))
                    .flatten(),
            )
        }

        fn fill_slot(
            &mut self,
            key: &impl LocalityPath,
            item: $t,
        ) -> std::result::Result<Key<Ref, $t>, $t> {
            if let Some(container) = self.fill(key) {
                container.fill_slot(key, item)
            } else {
                Err(item)
            }
        }

        fn fill_locality(&mut self, key: &impl LocalityPath) -> Option<LocalityKey> {
            self.fill(key)?.fill_locality(key)
        }

        #[inline(always)]
        fn get_slot(&self, key: Key<Ptr, $t>) -> Option<UnsafeSlot<$t>> {
            let index = self.region().index_of(key);
            self.get(index)?.get_slot(key)
        }

        fn unfill_slot(&mut self, key: Key<Ptr, $t>) -> Option<($t, ItemLocality<$t>)> {
            let index = self.region().index_of(key);
            self.get_mut(index)?.unfill_slot(key)
        }

        #[inline(always)]
        fn contains_slot(&self, key: Key<Ptr, T>) -> bool {
            let index = self.region().index_of(key);
            self.get(index)
                .filter(|sub| sub.contains_slot(key))
                .is_some()
        }

        fn slot_count(&self) -> usize {
            self.iter(..)
                .into_iter()
                .flatten()
                .map(|sub| sub.slot_count())
                .sum()
        }
    };
    (impl AnyContainer) => {
        fn container_path(&self) -> Path {
            self.region().path()
        }

        #[inline(always)]
        fn any_get_slot(&self, key: Key) -> Option<UnsafeSlot> {
            let index = self.region().index_of(key);
            self.get(index)?.any_get_slot(key)
        }

        fn any_get_locality(
            &self,
            path: &dyn LocalityPath,
            ty: std::any::TypeId,
        ) -> Option<LocalityRef> {
            self.locality(path)?.any_get_locality(path, ty)
        }

        fn first_key(&self, key: std::any::TypeId) -> Option<Key<Ref>> {
            self.iter(..)?
                .find_map(|container| container.first_key(key))
        }

        fn next_key(&self, ty: TypeId, key: Key) -> Option<Key<Ref>> {
            let index = self.region().index_of(key);
            if let Some(container) = self.get(index) {
                if let Some(key) = container.next_key(ty, key) {
                    return Some(key);
                }
            }

            self.iter(index + 1..)?
                .find_map(|container| container.first_key(ty))
        }

        fn last_key(&self, key: std::any::TypeId) -> Option<Key<Ref>> {
            self.iter(..)?
                .rev()
                .find_map(|container| container.last_key(key))
        }

        fn any_fill_slot(
            &mut self,
            path: &dyn LocalityPath,
            item: Box<dyn std::any::Any>,
        ) -> std::result::Result<Key<Ref>, String> {
            if let Some(sub) = self.fill(path) {
                sub.any_fill_slot(path, item)
            } else {
                Err(format!("Context not allocated {:?}", path))
            }
        }

        fn any_fill_locality(
            &mut self,
            path: &dyn LocalityPath,
            ty: std::any::TypeId,
        ) -> Option<LocalityKey> {
            self.fill(path)?.any_fill_locality(path, ty)
        }

        fn localized_drop(&mut self, key: Key) -> Option<Vec<Key<Owned>>> {
            let index = self.region().index_of(key);
            self.get_mut(index)?.localized_drop(key)
        }
    };
}
