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
        type SlotIter<'a> = impl Iterator<Item = (Key<T>, UnsafeSlot<'a, T>)> + Send
                                                                                where
                                                                                    Self: 'a;

        fn get_locality(&self, key: &impl LocalityPath) -> Option<SlotLocality<$t>> {
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
        ) -> std::result::Result<Key<$t>, $t> {
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
        fn get_slot(&self, key: Key<$t>) -> Option<UnsafeSlot<$t>> {
            let index = self.region().index_of(key);
            self.get(index)?.get_slot(key)
        }

        fn unfill_slot(&mut self, key: Key<$t>) -> Option<($t, SlotLocality<$t>)> {
            let index = self.region().index_of(key);
            self.get_mut(index)?.unfill_slot(key)
        }
    };
    (impl AnyContainer) => {
        fn container_path(&self) -> Path {
            self.region().path()
        }

        #[inline(always)]
        fn get_slot_any(&self, key: Key) -> Option<AnyUnsafeSlot> {
            let index = self.region().index_of(key);
            self.get(index)?.get_slot_any(key)
        }

        fn get_locality_any(
            &self,
            path: &dyn LocalityPath,
            ty: std::any::TypeId,
        ) -> Option<AnySlotLocality> {
            self.locality(path)?.get_locality_any(path, ty)
        }

        fn first_key(&self, key: std::any::TypeId) -> Option<Key> {
            self.iter(..)?
                .find_map(|container| container.first_key(key))
        }

        fn next_key(&self, ty: TypeId, key: Key) -> Option<Key> {
            let index = self.region().index_of(key);
            if let Some(container) = self.get(index) {
                if let Some(key) = container.next_key(ty, key) {
                    return Some(key);
                }
            }

            self.iter(index + 1..)?
                .find_map(|container| container.first_key(ty))
        }

        fn last_key(&self, key: std::any::TypeId) -> Option<Key> {
            self.iter(..)?
                .rev()
                .find_map(|container| container.last_key(key))
        }

        fn fill_slot_any(
            &mut self,
            path: &dyn LocalityPath,
            item: Box<dyn std::any::Any>,
        ) -> std::result::Result<Key, String> {
            if let Some(sub) = self.fill(path) {
                sub.fill_slot_any(path, item)
            } else {
                Err(format!(
                    "Context not allocated {:?} on path {:?}",
                    path,
                    self.container_path()
                ))
            }
        }

        fn fill_locality_any(
            &mut self,
            path: &dyn LocalityPath,
            ty: std::any::TypeId,
        ) -> Option<LocalityKey> {
            self.fill(path)?.fill_locality_any(path, ty)
        }

        fn unfill_slot_any(&mut self, key: Key) {
            let index = self.region().index_of(key);
            if let Some(container) = self.get_mut(index) {
                container.unfill_slot_any(key);
            }
        }
    };
}
