use super::*;
use crate::core::Locality;
use log::*;
use std::{num::NonZeroUsize, ops::RangeBounds};

/// A container of items.
/// Should clear on drop.
///
/// UNSAFE: Implementations MUST follow next, get, and iter SAFETY contracts.
pub unsafe trait LeafContainer<T: Item> {
    type Iter<'a>: Iterator<Item = UnsafeSlot<'a, T>> + Send
    where
        Self: 'a;

    /// Implementations should have #[inline(always)]
    fn locality(&self) -> &Locality<T>;

    /// Returns first index with a slot.
    fn first(&self) -> Option<NonZeroUsize>;

    /// Returns following index after given in ascending order with a slot.
    ///
    /// SAFETY: MUST have bijection over input_index and output_index and input_index != output_index.
    fn next(&self, after: NonZeroUsize) -> Option<NonZeroUsize>;

    /// Returns last index with a slot.
    fn last(&self) -> Option<NonZeroUsize>;

    /// Implementations should have #[inline(always)]
    /// SAFETY: Bijection between index and slot MUST be enforced.
    fn get(&self, index: usize) -> Option<UnsafeSlot<T>>;

    /// Implementations should have #[inline(always)]
    fn contains(&self, index: usize) -> bool;

    /// Iterates in ascending order for indices in range.
    /// SAFETY: Iterator MUST NOT return the same slot more than once.
    fn iter(&self, range: impl RangeBounds<usize>) -> Self::Iter<'_>;

    /// None if there is no more place in container.
    fn fill(&mut self, item: T) -> std::result::Result<NonZeroUsize, T>;

    /// Removes from container.
    fn unfill(&mut self, index: usize) -> Option<T>;

    /// Unfill all slots and clear their content.
    fn clear(&mut self) {
        if let Some(mut now) = self.first() {
            loop {
                // Drop slot
                match self.unfill(now.get()) {
                    Some(item) => {
                        // Drop local
                        let index = self.locality().locality_key().key_of::<T>(now).index();
                        // SAFETY: Item is alive in this scope.
                        let key = unsafe { Key::new_ref(index) };
                        item.localized_drop(self.locality().item_locality(key));
                    }
                    None => warn!(
                        "{:?}::{} returned invalid index: {}",
                        self.locality().locality_key().path(),
                        std::any::type_name::<Self>(),
                        now
                    ),
                }

                // Next
                if let Some(next) = self.next(now) {
                    now = next;
                } else {
                    break;
                }
            }
        }
    }
}

#[macro_export]
macro_rules! leaf_container {
    (impl Drop<$($t:ty),+>) => {
        fn drop(&mut self) {
            // Clear for each type
            $(
                LeafContainer::<$t>::clear(self);
            )+
        }
    };
    (impl Container<$t:ty>) => {
        type SlotIter<'a> =  impl Iterator<Item = UnsafeSlot<'a, $t>> + Send
            where Self:'a;

        #[inline(always)]
        fn get_slot(&self, key: Key<Ptr,$t>) -> Option<UnsafeSlot<$t>> {
            let index = self.locality().locality_key().index_of(key);
            self.get(index)
        }

        fn get_locality(&self, _: &impl LocalityPath) -> Option<ContainerLocality<$t>> {
            Some(self.locality().container_locality())
        }

        fn iter_slot(&self, path: KeyPath<$t>) -> Option<Self::SlotIter<'_>> {
            let leaf_path=*self.locality().locality_key();
            let range =leaf_path.range_of(path.path())?;
            Some(self.iter(range))
        }

        fn fill_slot(&mut self, _: &impl LocalityPath, item: $t) -> std::result::Result<Key<Ref,$t>, $t> {
            // SAFETY: Item was just added to container.
            self.fill(item).map(|index|unsafe{self.locality().locality_key().key_of(index).extend()})
        }

        fn fill_locality(&mut self, _: &impl LocalityPath) -> Option<LocalityKey> {
            Some(*self.locality().locality_key())
        }

        fn unfill_slot(&mut self, key: Key<Ptr,$t>) -> Option<($t, ItemLocality<$t>)> {
            let index = self.locality().locality_key().index_of(key);
            self.unfill(index)
                // SAFETY: Locality is alive for self.
                .map(move |item| (item, self.locality().item_locality(unsafe{key.extend()})))
        }

        #[inline(always)]
        fn contains_slot(&self, key: Key<Ptr, T>) -> bool{
            let index = self.locality().locality_key().index_of(key);
            self.contains(index)
        }
    };
    (impl AnyContainer<$t:ty>) => {
        fn container_path(&self) -> Path{
            self.locality().locality_key().path()
        }

        #[inline(always)]
        fn any_get_slot(&self, key: Key) -> Option<UnsafeSlot>{
            self.get_slot(key.assume()).map(|slot| slot.any())
        }

        fn any_get_locality(&self, _: &dyn LocalityPath,ty: TypeId) -> Option<AnyContainerLocality>{
            if ty == TypeId::of::<$t>() {
                Some(self.locality().container_locality().any())
            } else {
                None
            }
        }

        fn first_key(&self, key: TypeId) -> Option<Key<Ref>>{
            if key == TypeId::of::<$t>() {
                self.first().map(|index| {
                    let key=self.locality().locality_key().key_of::<$t>(index);
                    // SAFETY: Key is valid for self.
                    unsafe{key.extend()}.any()
                })
            } else {
                None
            }
        }

        fn next_key(&self, _: TypeId, key: Key) -> Option<Key<Ref>>{
            let index = self.locality().locality_key().index_of(key);
            self.next(NonZeroUsize::new(index)?).map(|index| {
                let key=self.locality().locality_key().key_of::<$t>(index);
                // SAFETY: Key is valid for self.
                unsafe{key.extend()}.any()
            })
        }

        fn last_key(&self, key: TypeId) -> Option<Key<Ref>>{
            if key == TypeId::of::<$t>() {
                self.last().map(|index| {
                    let key=self.locality().locality_key().key_of::<$t>(index);
                    // SAFETY: Key is valid for self.
                    unsafe{key.extend()}.any()
                })
            } else {
                None
            }
        }

        fn types(&self) -> std::collections::HashMap<TypeId,ItemTraits>{
            let mut set = std::collections::HashMap::new();
            set.insert(TypeId::of::<$t>(),ItemTrait::erase_type(<$t as Item>::TRAITS));
            set
        }

        fn any_fill_slot(&mut self, _: &dyn LocalityPath, item: Box<dyn std::any::Any>) -> std::result::Result<Key<Ref>, String>{
            match item.downcast::<$t>() {
                Ok(item)=>{
                    if let Ok(index)=self.fill(Box::into_inner(item)){
                        let key=self.locality().locality_key().key_of::<$t>(index);
                        // SAFETY: Key is valid for self.
                        Ok(unsafe{key.extend()}.any())
                    } else {
                        Err(format!("No more place in {:?}::{}", self.container_path(),std::any::type_name::<Self>()))
                    }
                }
                Err(error)=> {
                    Err(format!("Item type mismatch: expected {:?}, got {:?}", TypeId::of::<$t>(), error))
                }
            }
        }

        fn any_fill_locality(&mut self, _: &dyn LocalityPath,ty: TypeId) -> Option<LocalityKey>{
            if ty == TypeId::of::<$t>() {
                Some(*self.locality().locality_key())
            } else {
                None
            }

        }

        /// Panics if item is edgeless referenced.
        fn localized_drop(&mut self, key: Key)-> Option<Vec<PartialEdge<Key<Owned>>>>{
            let (item,locality)=self.unfill_slot(key.assume())?;
            Some(item.localized_drop(locality))
        }
    }
}
