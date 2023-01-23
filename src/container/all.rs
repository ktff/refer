use crate::{
    core::{
        ty::{MultiTypeContainer, TypeContainer},
        *,
    },
    type_container,
};
use log::*;
use std::{
    any::{Any, TypeId},
    collections::{hash_map::Entry, HashMap},
};

/// A container of all types backed by container family F.
pub struct AllContainer<F: Send + Sync + 'static> {
    /// T -> i
    collections: HashMap<TypeId, usize>,
    traits: HashMap<TypeId, ItemTraits>,
    /// [i] -> F::Container<T>
    mappings: Vec<Box<dyn AnyContainer>>,
    region: RegionPath,
    family: F,
}

impl<F: Send + Sync + 'static> AllContainer<F> {
    pub fn new(region: RegionPath, family: F) -> Self {
        Self {
            collections: HashMap::new(),
            traits: HashMap::new(),
            region,
            family,
            mappings: Vec::new(),
        }
    }
}

impl<F: ContainerFamily<T>, T: Item> TypeContainer<T> for AllContainer<F> {
    type Sub = F::Container;

    fn fill(&mut self) -> &mut Self::Sub {
        let index = match self.collections.entry(TypeId::of::<T>()) {
            Entry::Occupied(value) => *value.get(),
            Entry::Vacant(entry) => {
                // Add a new container
                let index = self.mappings.len();
                let path = self.region.path_of(index);
                self.mappings
                    .push(Box::new(self.family.new_container(path)) as Box<dyn AnyContainer>);
                self.traits.insert(TypeId::of::<T>(), T::traits());

                *entry.insert(index)
            }
        };

        (&mut *self.mappings[index] as &mut dyn Any)
            .downcast_mut::<F::Container>()
            .expect("Should be correct type")
    }
}

impl<F: Send + Sync + 'static> MultiTypeContainer for AllContainer<F> {
    #[inline(always)]
    fn region(&self) -> RegionPath {
        self.region
    }

    fn type_to_index(&self, type_id: TypeId) -> Option<usize> {
        self.collections.get(&type_id).copied()
    }

    #[inline(always)]
    fn get_any_index(&self, index: usize) -> Option<&dyn AnyContainer> {
        self.mappings.get(index).map(|x| &**x)
    }

    fn get_mut_any_index(&mut self, index: usize) -> Option<&mut dyn AnyContainer> {
        self.mappings.get_mut(index).map(|x| &mut **x)
    }
}

impl<F: ContainerFamily<T>, T: Item> Container<T> for AllContainer<F> {
    type_container!(impl Container<T>);
}

impl<F: Send + Sync + 'static> AnyContainer for AllContainer<F> {
    type_container!(impl AnyContainer);

    fn types(&self) -> HashMap<TypeId, ItemTraits> {
        self.traits.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::VecContainerFamily;
    use std::{any::Any, num::NonZeroU32};

    fn container() -> AllContainer<VecContainerFamily> {
        AllContainer::new(
            Path::default()
                .region(NonZeroU32::new(10).unwrap())
                .unwrap(),
            VecContainerFamily::default(),
        )
    }

    #[test]
    fn allocate_multi_type_item() {
        let mut container = container();

        let key_a = container.fill_slot((), 42).unwrap();
        let key_b = container.fill_slot((), true).unwrap();
        let key_c = container.fill_slot((), "Hello").unwrap();

        assert_eq!(
            container.access_mut().slot(key_a).get().unwrap().item(),
            &42
        );
        assert_eq!(
            container.access_mut().slot(key_b).get().unwrap().item(),
            &true
        );
        assert_eq!(
            container.access_mut().slot(key_c).get().unwrap().item(),
            &"Hello"
        );
    }

    #[test]
    fn get_any() {
        let mut container = container();

        let key_a = container.fill_slot((), 42).unwrap();
        let key_b = container.fill_slot((), true).unwrap();
        let key_c = container.fill_slot((), "Hello").unwrap();

        assert_eq!(
            (container
                .access_mut()
                .slot(key_a.any())
                .get_dyn()
                .unwrap()
                .item() as &dyn Any)
                .downcast_ref(),
            Some(&42)
        );
        assert_eq!(
            (container
                .access_mut()
                .slot(key_b.any())
                .get_dyn()
                .unwrap()
                .item() as &dyn Any)
                .downcast_ref(),
            Some(&true)
        );
        assert_eq!(
            (container
                .access_mut()
                .slot(key_c.any())
                .get_dyn()
                .unwrap()
                .item() as &dyn Any)
                .downcast_ref(),
            Some(&"Hello")
        );
    }
}
