use crate::{
    core::{
        container::{ContainerFamily, MultiTypeContainer, TypeContainer},
        *,
    },
    multi_type_container,
};
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

    fn step_down(&self) -> Option<&Self::Sub> {
        let index = self.type_to_index(TypeId::of::<T>())?;
        let container = self.get_any_index(index)?;

        Some(
            (container as &dyn Any)
                .downcast_ref()
                .expect("Should be correct type"),
        )
    }

    fn step_down_mut(&mut self) -> Option<&mut Self::Sub> {
        let index = self.type_to_index(TypeId::of::<T>())?;
        let container = self.get_mut_any_index(index)?;
        Some(
            (container as &mut dyn Any)
                .downcast_mut()
                .expect("Should be correct type"),
        )
    }

    fn fill(&mut self) -> &mut Self::Sub {
        let index = match self.collections.entry(TypeId::of::<T>()) {
            Entry::Occupied(value) => *value.get(),
            Entry::Vacant(entry) => {
                // Add a new container
                let index = self.mappings.len();
                let path = self.region.path_of(index);
                self.mappings
                    .push(Box::new(self.family.new_container(path)) as Box<dyn AnyContainer>);
                self.traits
                    .insert(TypeId::of::<T>(), ItemTrait::erase_type(T::TRAITS));

                *entry.insert(index)
            }
        };

        (&mut *self.mappings[index] as &mut dyn Any)
            .downcast_mut::<F::Container>()
            .expect("Should be correct type")
    }
}

unsafe impl<F: Send + Sync + 'static> MultiTypeContainer for AllContainer<F> {
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

unsafe impl<F: ContainerFamily<T>, T: Item> Container<T> for AllContainer<F> {
    multi_type_container!(impl Container<T> prefer index);
}

unsafe impl<F: Send + Sync + 'static> AnyContainer for AllContainer<F> {
    multi_type_container!(impl AnyContainer);

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

        let key_a = container.fill_slot(&(), 42).unwrap().ptr();
        let key_b = container.fill_slot(&(), true).unwrap().ptr();
        let key_c = container.fill_slot(&(), "Hello").unwrap().ptr();

        assert_eq!(
            container.access_mut().key(key_a).get_try().unwrap().item(),
            &42
        );
        assert_eq!(
            container.access_mut().key(key_b).get_try().unwrap().item(),
            &true
        );
        assert_eq!(
            container.access_mut().key(key_c).get_try().unwrap().item(),
            &"Hello"
        );
    }

    #[test]
    fn get_any() {
        let mut container = container();

        let key_a = container.fill_slot(&(), 42u32).unwrap().ptr();
        let key_b = container.fill_slot(&(), true).unwrap().ptr();
        let key_c = container.fill_slot(&(), "Hello").unwrap().ptr();

        assert_eq!(
            (container
                .access_mut()
                .key(key_a.any())
                .get_dyn_try()
                .unwrap()
                .item() as &dyn Any)
                .downcast_ref(),
            Some(&42u32)
        );
        assert_eq!(
            (container
                .access_mut()
                .key(key_b.any())
                .get_dyn_try()
                .unwrap()
                .item() as &dyn Any)
                .downcast_ref(),
            Some(&true)
        );
        assert_eq!(
            (container
                .access_mut()
                .key(key_c.any())
                .get_dyn_try()
                .unwrap()
                .item() as &dyn Any)
                .downcast_ref(),
            Some(&"Hello")
        );
    }
}
