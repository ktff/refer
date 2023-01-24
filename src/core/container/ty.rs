use super::*;

pub trait TypeContainer<T: Item> {
    type Sub: Container<T>;

    fn get(&self) -> Option<&Self::Sub>;

    fn get_mut(&mut self) -> Option<&mut Self::Sub>;

    fn fill(&mut self) -> &mut Self::Sub;
}

pub trait MultiTypeContainer {
    /// Implementations should have #[inline(always)]
    fn region(&self) -> RegionPath;

    fn type_to_index(&self, type_id: TypeId) -> Option<usize>;

    /// Implementations should have #[inline(always)]
    fn get_any_index(&self, index: usize) -> Option<&dyn AnyContainer>;

    fn get_mut_any_index(&mut self, index: usize) -> Option<&mut dyn AnyContainer>;

    #[inline(always)]
    fn get_any(&self, key: AnyKey) -> Option<&dyn AnyContainer> {
        let index = self.region().index_of(key);
        self.get_any_index(index)
    }

    fn get_mut_any(&mut self, key: AnyKey) -> Option<&mut dyn AnyContainer> {
        let index = self.region().index_of(key);
        self.get_mut_any_index(index)
    }
}

// *************************** Blankets ***************************
#[macro_export]
macro_rules! single_type_container {
    (impl Container<$t:ty> ) => {
        type Shell = <<Self as TypeContainer<$t>>::Sub as Container<$t>>::Shell;

        type SlotIter<'a> = <<Self as TypeContainer<$t>>::Sub as Container<$t>>::SlotIter<'a>;

        fn get_context(&self, key: <$t as Item>::LocalityKey) -> Option<SlotContext<$t>> {
            TypeContainer::<$t>::get(self)?.get_context(key)
        }

        fn iter_slot(&self, path: KeyPath<$t>) -> Option<Self::SlotIter<'_>> {
            TypeContainer::<$t>::get(self)?.iter_slot(path)
        }

        fn fill_slot(
            &mut self,
            key: <$t as Item>::LocalityKey,
            item: $t,
        ) -> std::result::Result<Key<$t>, $t> {
            TypeContainer::<$t>::fill(self).fill_slot(key, item)
        }

        fn fill_context(&mut self, key: <$t as Item>::LocalityKey) {
            TypeContainer::<$t>::fill(self).fill_context(key);
        }

        #[inline(always)]
        fn get_slot(&self, key: Key<$t>) -> Option<UnsafeSlot<$t, Self::Shell>> {
            TypeContainer::<$t>::get(self)?.get_slot(key)
        }

        fn unfill_slot(&mut self, key: Key<$t>) -> Option<($t, Self::Shell, SlotContext<$t>)> {
            TypeContainer::<$t>::get_mut(self)?.unfill_slot(key)
        }
    };
    (impl AnyContainer<$t:ty>) => {
        #[inline(always)]
        fn get_slot_any(&self, key: AnyKey) -> Option<AnyUnsafeSlot> {
            self.get()?.get_slot_any(key)
        }

        fn get_context_any(&self, path: ContextPath) -> Option<AnySlotContext> {
            self.get()?.get_context_any(path)
        }

        fn first_key(&self, key: std::any::TypeId) -> Option<AnyKey> {
            self.get()?.first_key(key)
        }

        fn next_key(&self, key: AnyKey) -> Option<AnyKey> {
            self.get()?.next_key(key)
        }

        fn last_key(&self, key: std::any::TypeId) -> Option<AnyKey> {
            self.get()?.last_key(key)
        }

        fn types(&self) -> std::collections::HashMap<std::any::TypeId, ItemTraits> {
            let mut set = std::collections::HashMap::new();
            set.insert(std::any::TypeId::of::<$t>(), <$t as Item>::traits());
            set
        }

        fn fill_slot_any(
            &mut self,
            path: ContextPath,
            item: Box<dyn std::any::Any>,
        ) -> std::result::Result<AnyKey, String> {
            if let Some(sub) = self.get_mut() {
                sub.fill_slot_any(path, item)
            } else {
                Err(format!(
                    "Context not allocated {:?} on path {:?}",
                    path,
                    self.container_path()
                ))
            }
        }

        fn fill_context_any(&mut self, path: Path, ty: std::any::TypeId) -> Option<ContextPath> {
            self.fill().fill_context_any(path, ty)
        }

        fn unfill_slot_any(&mut self, key: AnyKey) {
            if let Some(container) = self.get_mut() {
                container.unfill_slot_any(key);
            }
        }
    };
}

#[macro_export]
macro_rules! multi_type_container {
    (impl base Container<$t:ty> ) => {
        type Shell = <<Self as TypeContainer<$t>>::Sub as Container<$t>>::Shell;

        type SlotIter<'a> = <<Self as TypeContainer<$t>>::Sub as Container<$t>>::SlotIter<'a>;

        fn get_context(&self, key: <$t as Item>::LocalityKey) -> Option<SlotContext<$t>> {
            TypeContainer::<$t>::get(self)?.get_context(key)
        }

        fn iter_slot(&self, path: KeyPath<$t>) -> Option<Self::SlotIter<'_>> {
            TypeContainer::<$t>::get(self)?.iter_slot(path)
        }

        fn fill_slot(
            &mut self,
            key: <$t as Item>::LocalityKey,
            item: $t,
        ) -> std::result::Result<Key<$t>, $t> {
            TypeContainer::<$t>::fill(self).fill_slot(key, item)
        }

        fn fill_context(&mut self, key: <$t as Item>::LocalityKey) {
            TypeContainer::<$t>::fill(self).fill_context(key);
        }

    };
    (impl Container<$t:ty> prefer type) => {
        $crate::multi_type_container!(impl base Container<$t>);

        #[inline(always)]
        fn get_slot(&self, key: Key<$t>) -> Option<UnsafeSlot<$t, Self::Shell>> {
            TypeContainer::<$t>::get(self)?.get_slot(key)
        }


        fn unfill_slot(&mut self, key: Key<$t>) -> Option<($t, Self::Shell, SlotContext<$t>)> {
            TypeContainer::<$t>::get_mut(self)?.unfill_slot(key)
        }
    };
    (impl Container<$t:ty> prefer index) => {
        $crate::multi_type_container!(impl base Container<$t>);

        #[inline(always)]
        fn get_slot(&self, key: Key<$t>) -> Option<UnsafeSlot<$t, Self::Shell>> {
            (self.get_any(key.any())? as &dyn std::any::Any)
                .downcast_ref::<<Self as TypeContainer<$t>>::Sub>()
                .expect("Should be correct type")
                .get_slot(key)
        }


        fn unfill_slot(&mut self, key: Key<$t>) -> Option<($t, Self::Shell, SlotContext<$t>)> {
            (self.get_mut_any(key.any())? as &mut dyn std::any::Any)
                .downcast_mut::<<Self as TypeContainer<$t>>::Sub>()
                .expect("Should be correct type")
                .unfill_slot(key)
        }
    };
    (impl AnyContainer) => {
        fn container_path(&self) -> Path {
            self.region().path()
        }

        #[inline(always)]
        fn get_slot_any(&self, key: AnyKey) -> Option<AnyUnsafeSlot> {
            self.get_any(key)?.get_slot_any(key)
        }

        fn get_context_any(&self, path: ContextPath) -> Option<AnySlotContext> {
            let index = self.region().range_of(path)?;
            if index.start() == index.end() {
                self.get_any_index(*index.start())?.get_context_any(path)
            } else {
                log::warn!(
                    "Illegal ContextPath: {:?} in region: {:?}",
                    path,
                    self.region()
                );
                None
            }
        }

        fn first_key(&self, key: std::any::TypeId) -> Option<AnyKey> {
            self.get_any_index(self.type_to_index(key)?)?.first_key(key)
        }

        fn next_key(&self, key: AnyKey) -> Option<AnyKey> {
            self.get_any(key)?.next_key(key)
        }

        fn last_key(&self, key: std::any::TypeId) -> Option<AnyKey> {
            self.get_any_index(self.type_to_index(key)?)?.last_key(key)
        }

        fn fill_slot_any(
            &mut self,
            path: ContextPath,
            item: Box<dyn std::any::Any>,
        ) -> std::result::Result<AnyKey, String> {
            let region = self.region();
            let index = region.range_of(path).ok_or_else(|| {
                format!(
                    "Context path {:?} not in range of region: {:?}",
                    path,
                    region
                )
            })?;
            if index.start() == index.end() {
                self.get_mut_any_index(*index.start())
                    .ok_or_else(|| format!("Context not allocated: {:?} in region: {:?}", path,region))?
                    .fill_slot_any(path, item)
            } else {
                Err(format!(
                    "Illegal ContextPath: {:?} in region: {:?}",
                    path,
                    region
                ))
            }
        }

        fn fill_context_any(&mut self, path: Path, ty: std::any::TypeId) -> Option<ContextPath> {
            let range = self.region().range_of(path)?;
            if let Some(index) = self.type_to_index(ty) {
                // Container exists
                if range.contains(&index) {
                    self.get_mut_any_index(index)?.fill_context_any(path, ty)
                } else {
                    None
                }
            } else {
                // Container doesn't exist
                None
            }
        }

        fn unfill_slot_any(&mut self, key: AnyKey) {
            if let Some(container) = self.get_mut_any(key) {
                container.unfill_slot_any(key);
            }
        }
    };
}
