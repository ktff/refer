use super::*;
use crate::core::Context;
use log::*;
use std::{num::NonZeroUsize, ops::RangeBounds};

/// A container of items.
/// Should clear on drop.
pub trait LeafContainer<T: Item> {
    /// Shell of item.
    type Shell: Shell<T = T>;

    type Iter<'a>: Iterator<Item = (Key<T>, UnsafeSlot<'a, T, Self::Shell>)> + Send
    where
        Self: 'a;

    /// Implementations should have #[inline(always)]
    fn context(&self) -> &Context<T>;

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
    fn iter(&self, range: impl RangeBounds<usize>) -> Option<Self::Iter<'_>>;

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
                        shell.clear(self.context().allocator());
                        item.displace(self.context().slot_context(), None);
                    }
                    None => warn!(
                        "{:?}::{} returned invalid index: {}",
                        self.context().leaf_path().path(),
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

        type SlotIter<'a> = <Self as LeafContainer<$t>>::Iter<'a>;

        #[inline(always)]
        fn get_slot(&self, key: Key<$t>) -> Option<UnsafeSlot<$t, Self::Shell>> {
            let index = self.context().leaf_path().index_of(key);
            self.get(index)
        }

        fn get_context(&self, _: <$t>::LocalityKey) -> Option<SlotContext<$t>> {
            Some(self.context().slot_context())
        }

        fn iter_slot(&self, path: KeyPath<$t>) -> Option<Self::SlotIter<'_>> {
            let range = self.context().leaf_path().range_of(path.path())?;
            self.iter(range)
        }

        fn fill_slot(&mut self, _: <$t>::LocalityKey, item: $t) -> std::result::Result<Key<$t>, $t> {
            self.fill(item).map(|index|self.context().leaf_path().key_of(index))
        }

        fn fill_context(&mut self, _: <$t>::LocalityKey) {}

        fn unfill_slot(&mut self, key: Key<$t>) -> Option<($t, Self::Shell, SlotContext<$t>)> {
            let index = self.context().leaf_path().index_of(key);
            self.unfill(index)
                .map(move |(item, shell)| (item, shell, self.context().slot_context()))
        }
    };
    (impl AnyContainer<$t:ty>) => {
        fn container_path(&self) -> Path{
            self.context().leaf_path().path()
        }

        #[inline(always)]
        fn get_slot_any(&self, key: AnyKey) -> Option<AnyUnsafeSlot>{
            if let Some(key) = key.downcast::<$t>() {
                self.get_slot(key).map(|slot| slot.upcast())
            } else {
                None
            }
        }

        fn get_context_any(&self, path: AnyPath) -> Option<AnySlotContext>{
            if let Some(path) = path.downcast::<$t>() {
                if self.container_path().contains(path){
                    Some(self.context.slot_context().upcast())
                }else{
                    None
                }
            } else {
                None
            }
        }

        fn first_key(&self, key: TypeId) -> Option<AnyKey>{
            if key == TypeId::of::<$t>() {
                self.first().map(|index| self.context().leaf_path().key_of::<$t>(index).upcast())
            } else {
                None
            }
        }

        fn next_key(&self, key: AnyKey) -> Option<AnyKey>{
            if let Some(key) = key.downcast::<$t>() {
                let index = self.context().leaf_path().index_of(key);
                self.next(NonZeroUsize::new(index)?).map(|index| self.context().leaf_path().key_of::<$t>(index).upcast())
            } else {
                None
            }
        }

        fn last_key(&self, key: TypeId) -> Option<AnyKey>{
            if key == TypeId::of::<$t>() {
                self.last().map(|index| self.context().leaf_path().key_of::<$t>(index).upcast())
            } else {
                None
            }
        }

        fn types(&self) -> HashSet<TypeId>{
            let mut set = HashSet::new();
            set.insert(TypeId::of::<$t>());
            set
        }

        fn fill_slot_any(&mut self, path: AnyPath, item: Box<dyn std::any::Any>) -> std::result::Result<AnyKey, String>{
            match item.downcast::<$t>() {
                Ok(item)=>{
                    if let Some(path) = path.downcast::<$t>() {
                        if self.container_path().contains(path){
                            if let Ok(index)=self.fill(Box::into_inner(item)){
                                    Ok(self.context().leaf_path().key_of::<$t>(index).upcast())
                                } else {
                                    Err(format!("No more place in {:?}::{}", self.container_path(),std::any::type_name::<Self>()))
                                }
                        } else {
                            Err(format!("Path {:?} is not contained in {:?}::{}", path, self.container_path(),std::any::type_name::<Self>()))
                        }
                    } else {
                        Err(format!("Path type mismatch: expected {:?}, got {:?}", TypeId::of::<$t>(), path.type_id()))
                    }
                }
                Err(error)=> {
                    Err(format!("Item type mismatch: expected {:?}, got {:?}", TypeId::of::<$t>(), error))
                }
            }
        }

        fn fill_context_any(&mut self, path: AnyPath) -> AnyPath{
            path
        }

        fn unfill_slot_any(&mut self, key: AnyKey){
            if let Some(key) = key.downcast::<$t>() {
                self.unfill_slot(key);
            }
        }
    }
}
