use super::*;

pub trait TypeContainer<T: Item> {
    type Sub: Container<T>;

    fn get(&self) -> Option<&Self::Sub>;

    fn get_mut(&mut self) -> Option<&mut Self::Sub>;

    fn fill(&mut self) -> &mut Self::Sub;
}

/// UNSAFE: Implementations MUST follow get_any_index & get_any SAFETY contracts.
pub unsafe trait MultiTypeContainer {
    /// Implementations should have #[inline(always)]
    fn region(&self) -> RegionPath;

    fn type_to_index(&self, type_id: TypeId) -> Option<usize>;

    /// Implementations should have #[inline(always)]
    /// SAFETY: Bijection between index and container MUST be enforced.
    fn get_any_index(&self, index: usize) -> Option<&dyn AnyContainer>;

    fn get_mut_any_index(&mut self, index: usize) -> Option<&mut dyn AnyContainer>;

    /// SAFETY: Bijection between key and container MUST be enforced.
    #[inline(always)]
    fn get_any(&self, key: Key) -> Option<&dyn AnyContainer> {
        let index = self.region().index_of(key);
        self.get_any_index(index)
    }

    fn get_mut_any(&mut self, key: Key) -> Option<&mut dyn AnyContainer> {
        let index = self.region().index_of(key);
        self.get_mut_any_index(index)
    }
}

// *************************** Blankets ***************************
#[macro_export]
macro_rules! single_type_container {
    (impl Container<$t:ty> ) => {
        type SlotIter<'a> = <<Self as $crate::core::container::TypeContainer<$t>>::Sub as $crate::core::container::Container<$t>>::SlotIter<'a>;

        fn get_locality(&self, key: &impl $crate::core::LocalityPath) -> Option<$crate::core::ContainerLocality<$t>> {
            $crate::core::container::TypeContainer::<$t>::get(self)?.get_locality(key)
        }

        fn iter_slot(&self, path: $crate::core::KeyPath<$t>) -> Option<Self::SlotIter<'_>> {
            $crate::core::container::TypeContainer::<$t>::get(self)?.iter_slot(path)
        }

        fn fill_slot(
            &mut self,
            key: &impl $crate::core::LocalityPath,
            item: $t,
        ) -> std::result::Result<$crate::core::Key<Ref,$t>, $t> {
            $crate::core::container::TypeContainer::<$t>::fill(self).fill_slot(key, item)
        }

        fn fill_locality(&mut self, key: &impl $crate::core::LocalityPath) -> Option<$crate::core::LocalityKey> {
            $crate::core::container::TypeContainer::<$t>::fill(self).fill_locality(key)
        }

        #[inline(always)]
        fn get_slot(&self, key: $crate::core::Key<Ptr,$t>) -> Option<$crate::core::UnsafeSlot<$t>> {
            $crate::core::container::TypeContainer::<$t>::get(self)?.get_slot(key)
        }

        fn unfill_slot(&mut self, key: $crate::core::Key<Ptr,$t>) -> Option<($t, $crate::core::ItemLocality<$t>)> {
            $crate::core::container::TypeContainer::<$t>::get_mut(self)?.unfill_slot(key)
        }

        #[inline(always)]
        fn contains_slot(&self, key: Key<Ptr, $t>) -> bool{
            $crate::core::container::TypeContainer::<$t>::get(self).filter(|sub|sub.contains_slot(key)).is_some()
        }
    };
    (impl AnyContainer<$t:ty>) => {
        #[inline(always)]
        fn any_get_slot(&self, key: $crate::core::Key) -> Option<$crate::core::AnyUnsafeSlot> {
            $crate::core::container::TypeContainer::<$t>::get(self)?.any_get_slot(key)
        }

        fn any_get_locality(
            &self,
            path: &dyn $crate::core::LocalityPath,
            ty: std::any::TypeId,
        ) -> Option<$crate::core::AnyContainerLocality> {
            $crate::core::container::TypeContainer::<$t>::get(self)?.any_get_locality(path, ty)
        }

        fn first_key(&self, key: std::any::TypeId) -> Option<$crate::core::Key<Ref>> {
            $crate::core::container::TypeContainer::<$t>::get(self)?.first_key(key)
        }

        fn next_key(&self, ty: std::any::TypeId, key: $crate::core::Key) -> Option<$crate::core::Key<Ref>> {
            $crate::core::container::TypeContainer::<$t>::get(self)?.next_key(ty, key)
        }

        fn last_key(&self, key: std::any::TypeId) -> Option<$crate::core::Key<Ref>> {
            $crate::core::container::TypeContainer::<$t>::get(self)?.last_key(key)
        }

        fn types(&self) -> std::collections::HashMap<std::any::TypeId, $crate::core::ItemTraits> {
            let mut set = std::collections::HashMap::new();
            set.insert(std::any::TypeId::of::<$t>(), $crate::core::ItemTrait::erase_type(<$t as Item>::TRAITS));
            set
        }

        fn any_fill_slot(
            &mut self,
            path: &dyn $crate::core::LocalityPath,
            item: Box<dyn std::any::Any>,
        ) -> std::result::Result<$crate::core::Key<Ref>, String> {
            if let Some(sub) = $crate::core::container::TypeContainer::<$t>::get_mut(self) {
                sub.any_fill_slot(path, item)
            } else {
                Err(format!(
                    "Context not allocated {:?}",
                    path,
                ))
            }
        }

        fn any_fill_locality(
            &mut self,
            path: &dyn $crate::core::LocalityPath,
            ty: std::any::TypeId,
        ) -> Option<$crate::core::LocalityKey> {
            $crate::core::container::TypeContainer::<$t>::fill(self).any_fill_locality(path, ty)
        }

        fn localized_drop(&mut self, key: $crate::core::Key)-> Option<Vec<PartialEdge<Key<Owned>>>> {
            $crate::core::container::TypeContainer::<$t>::get_mut(self)?.localized_drop(key)
        }
    };
}

#[macro_export]
macro_rules! multi_type_container {
    (impl base Container<$t:ty> ) => {
        type SlotIter<'a> = <<Self as TypeContainer<$t>>::Sub as Container<$t>>::SlotIter<'a>;

        fn get_locality(&self, key: &impl LocalityPath) -> Option<ContainerLocality<$t>> {
            TypeContainer::<$t>::get(self)?.get_locality(key)
        }

        fn iter_slot(&self, path: KeyPath<$t>) -> Option<Self::SlotIter<'_>> {
            TypeContainer::<$t>::get(self)?.iter_slot(path)
        }

        fn fill_slot(
            &mut self,
            key: &impl LocalityPath,
            item: $t,
        ) -> std::result::Result<Key<Ref,$t>, $t> {
            TypeContainer::<$t>::fill(self).fill_slot(key, item)
        }

        fn fill_locality(&mut self, key: &impl LocalityPath) -> Option<LocalityKey>{
            TypeContainer::<$t>::fill(self).fill_locality(key)
        }

    };
    (impl Container<$t:ty> prefer type) => {
        $crate::multi_type_container!(impl base Container<$t>);

        #[inline(always)]
        fn get_slot(&self, key: Key<Ptr,$t>) -> Option<UnsafeSlot<$t>> {
            TypeContainer::<$t>::get(self)?.get_slot(key)
        }


        fn unfill_slot(&mut self, key: Key<Ptr,$t>) -> Option<($t, ItemLocality<$t>)> {
            TypeContainer::<$t>::get_mut(self)?.unfill_slot(key)
        }

        #[inline(always)]
        fn contains_slot(&self, key: Key<Ptr, $t>) -> bool{
            TypeContainer::<$t>::get(self).filter(|sub|sub.contains_slot(key)).is_some()
        }
    };
    (impl Container<$t:ty> prefer index) => {
        $crate::multi_type_container!(impl base Container<$t>);

        #[inline(always)]
        fn get_slot(&self, key: Key<Ptr,$t>) -> Option<UnsafeSlot<$t>> {
            (self.get_any(key.any())? as &dyn std::any::Any)
                .downcast_ref::<<Self as TypeContainer<$t>>::Sub>()
                .expect("Should be correct type")
                .get_slot(key)
        }


        fn unfill_slot(&mut self, key: Key<Ptr,$t>) -> Option<($t, ItemLocality<$t>)> {
            (self.get_mut_any(key.any())? as &mut dyn std::any::Any)
                .downcast_mut::<<Self as TypeContainer<$t>>::Sub>()
                .expect("Should be correct type")
                .unfill_slot(key)
        }

        #[inline(always)]
        fn contains_slot(&self, key: Key<Ptr, $t>) -> bool{
            self.get_any(key.any()).filter(|sub|(*sub as &dyn std::any::Any)
                .downcast_ref::<<Self as TypeContainer<$t>>::Sub>()
                .expect("Should be correct type").contains_slot(key)
            ).is_some()
        }
    };
    (impl AnyContainer) => {
        fn container_path(&self) -> Path {
            self.region().path()
        }

        #[inline(always)]
        fn any_get_slot(&self, key: Key) -> Option<AnyUnsafeSlot> {
            self.get_any(key)?.any_get_slot(key)
        }

        fn any_get_locality(&self, path: &dyn LocalityPath, ty: std::any::TypeId) -> Option<AnyContainerLocality> {
            self.get_any_index(self.type_to_index(ty)?)?.any_get_locality(path,ty)
        }

        fn first_key(&self, key: std::any::TypeId) -> Option<Key<Ref>> {
            self.get_any_index(self.type_to_index(key)?)?.first_key(key)
        }

        fn next_key(&self, ty: std::any::TypeId, key: Key) -> Option<Key<Ref>> {
            self.get_any(key)?.next_key(ty,key)
        }

        fn last_key(&self, key: std::any::TypeId) -> Option<Key<Ref>> {
            self.get_any_index(self.type_to_index(key)?)?.last_key(key)
        }

        fn any_fill_slot(
            &mut self,
            path: &dyn LocalityPath,
            item: Box<dyn std::any::Any>,
        ) -> std::result::Result<Key<Ref>, String> {
            let borrow: &dyn std::any::Any=&*item;
            let ty=borrow.type_id();
            let region=self.region();
            if let Some(index) = self.type_to_index(ty) {
                self.get_mut_any_index(index)
                    .ok_or_else(|| format!("Context not allocated: {:?} in region: {:?}", path,region))?
                    .any_fill_slot(path, item)
            } else {
                Err(format!(
                    "Illegal LocalityKey: {:?} in region: {:?}",
                    path,
                    region
                ))
            }
        }

        fn any_fill_locality(&mut self, path: &dyn LocalityPath, ty: std::any::TypeId) -> Option<LocalityKey> {
            if let Some(index) = self.type_to_index(ty) {
                // Container exists
                self.get_mut_any_index(index)?.any_fill_locality(path, ty)
            } else {
                // Container doesn't exist
                None
            }
        }

        fn localized_drop(&mut self, key: Key) -> Option<Vec<PartialEdge<Key<Owned>>>>{
            self.get_mut_any(key)?.localized_drop(key)
        }
    };
}
