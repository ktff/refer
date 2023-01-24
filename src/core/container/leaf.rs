use super::*;
use crate::core::Locality;
use log::*;
use std::{num::NonZeroUsize, ops::RangeBounds};

/// A container of items.
/// Should clear on drop.
pub trait LeafContainer<T: Item> {
    /// Shell of item.
    type Shell: Shell<T = T>;

    type Iter<'a>: Iterator<Item = (NonZeroUsize, UnsafeSlot<'a, T, Self::Shell>)> + Send
    where
        Self: 'a;

    /// Implementations should have #[inline(always)]
    fn locality(&self) -> &Locality<T>;

    /// Returns first index with a slot.
    fn first(&self) -> Option<NonZeroUsize>;

    /// Returns following index after given in ascending order with a slot.
    fn next(&self, after: NonZeroUsize) -> Option<NonZeroUsize>;

    /// Returns last index with a slot.
    fn last(&self) -> Option<NonZeroUsize>;

    /// Implementations should have #[inline(always)]
    /// Bijection between index and slot MUST be enforced.
    fn get(&self, index: usize) -> Option<UnsafeSlot<T, Self::Shell>>;

    /// Iterates in ascending order for indices in range.
    /// Iterator MUST NOT return the same slot more than once.
    fn iter(&self, range: impl RangeBounds<usize>) -> Self::Iter<'_>;

    /// None if there is no more place in container.
    fn fill(&mut self, item: T) -> std::result::Result<NonZeroUsize, T>;

    /// Removes from container.
    fn unfill(&mut self, index: usize) -> Option<(T, Self::Shell)>;

    /// Unfill all slots and clear their content.
    fn clear(&mut self) {
        if let Some(mut now) = self.first() {
            loop {
                // Drop slot
                match self.unfill(now.get()) {
                    Some((mut item, mut shell)) => {
                        // Drop local
                        shell.clear(self.locality().allocator());
                        item.displace(self.locality().slot_locality(), None);
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
        type Shell = <Self as LeafContainer<$t>>::Shell;

        type SlotIter<'a> =  impl Iterator<Item = (Key<$t>, UnsafeSlot<'a, $t, Self::Shell>)> + Send
            where Self:'a;

        #[inline(always)]
        fn get_slot(&self, key: Key<$t>) -> Option<UnsafeSlot<$t, Self::Shell>> {
            let index = self.locality().locality_key().index_of(key);
            self.get(index)
        }

        fn get_locality(&self, _: impl LocalityPath) -> Option<SlotLocality<$t>> {
            Some(self.locality().slot_locality())
        }

        fn iter_slot(&self, path: KeyPath<$t>) -> Option<Self::SlotIter<'_>> {
            let leaf_path=*self.locality().locality_key();
            let range =leaf_path.range_of(path.path())?;
            Some(self.iter(range).map(move |(index,slot)|(leaf_path.key_of(index),slot)))
        }

        fn fill_slot(&mut self, _: impl LocalityPath, item: $t) -> std::result::Result<Key<$t>, $t> {
            self.fill(item).map(|index|self.locality().locality_key().key_of(index))
        }

        fn fill_locality(&mut self, _: impl LocalityPath) -> Option<LocalityKey> {
            Some(*self.locality().locality_key())
        }

        fn unfill_slot(&mut self, key: Key<$t>) -> Option<($t, Self::Shell, SlotLocality<$t>)> {
            let index = self.locality().locality_key().index_of(key);
            self.unfill(index)
                .map(move |(item, shell)| (item, shell, self.locality().slot_locality()))
        }
    };
    (impl AnyContainer<$t:ty>) => {
        fn container_path(&self) -> Path{
            self.locality().locality_key().path()
        }

        #[inline(always)]
        fn get_slot_any(&self, key: AnyKey) -> Option<AnyUnsafeSlot>{
            self.get_slot(Key::new(key.index())).map(|slot| slot.upcast())
        }

        fn get_locality_any(&self, _: &dyn LocalityPath,ty: TypeId) -> Option<AnySlotLocality>{
            if ty == TypeId::of::<$t>() {
                Some(self.locality.slot_locality().upcast())
            } else {
                None
            }
        }

        fn first_key(&self, key: TypeId) -> Option<AnyKey>{
            if key == TypeId::of::<$t>() {
                self.first().map(|index| self.locality().locality_key().key_of::<$t>(index).upcast())
            } else {
                None
            }
        }

        fn next_key(&self, _: TypeId, key: AnyKey) -> Option<AnyKey>{
            let index = self.locality().locality_key().index_of(key);
            self.next(NonZeroUsize::new(index)?).map(|index| self.locality().locality_key().key_of::<$t>(index).upcast())
        }

        fn last_key(&self, key: TypeId) -> Option<AnyKey>{
            if key == TypeId::of::<$t>() {
                self.last().map(|index| self.locality().locality_key().key_of::<$t>(index).upcast())
            } else {
                None
            }
        }

        fn types(&self) -> std::collections::HashMap<TypeId,ItemTraits>{
            let mut set = std::collections::HashMap::new();
            set.insert(TypeId::of::<$t>(),<$t as Item>::traits());
            set
        }

        fn fill_slot_any(&mut self, _: &dyn LocalityPath, item: Box<dyn std::any::Any>) -> std::result::Result<AnyKey, String>{
            match item.downcast::<$t>() {
                Ok(item)=>{
                    if let Ok(index)=self.fill(Box::into_inner(item)){
                        Ok(self.locality().locality_key().key_of::<$t>(index).upcast())
                    } else {
                        Err(format!("No more place in {:?}::{}", self.container_path(),std::any::type_name::<Self>()))
                    }
                }
                Err(error)=> {
                    Err(format!("Item type mismatch: expected {:?}, got {:?}", TypeId::of::<$t>(), error))
                }
            }
        }

        fn fill_locality_any(&mut self, _: &dyn LocalityPath,ty: TypeId) -> Option<LocalityKey>{
            if ty == TypeId::of::<$t>() {
                Some(*self.locality().locality_key())
            } else {
                None
            }

        }

        fn unfill_slot_any(&mut self, key: AnyKey){
            self.unfill_slot(Key::new(key.index()));
        }
    }
}
